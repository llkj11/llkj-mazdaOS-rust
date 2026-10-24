use std::time::{Duration, Instant};

use mazda_core::CommanderEvent;
use mazda_mock::{DemoScenario, MockMazda};
use mazda_ui::{Framebuffer, MediaIntent, UiModel, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};

const SOURCE_REFRESH_INTERVAL: Duration = Duration::from_millis(100);

fn main() -> Result<(), minifb::Error> {
    let smoke_test = std::env::args().any(|argument| argument == "--smoke-test");
    let mut frames = 0_u32;
    let mut pointer_was_down = false;
    let mut mazda = MockMazda::demo();
    let mut ui = UiModel::from_source(&mazda);
    let mut framebuffer = Framebuffer::mazda_connect();
    let mut window = Window::new(
        "Mazda OS - 2018 Mazda6 - Host Simulator",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        WindowOptions {
            resize: false,
            scale: Scale::X1,
            ..WindowOptions::default()
        },
    )?;

    window.set_target_fps(60);
    let mut last_source_refresh = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for key in window.get_keys_pressed(KeyRepeat::Yes) {
            if let Some(event) = commander_rotation_event(key) {
                handle_event(&mut ui, &mut mazda, event);
            }
        }
        for key in window.get_keys_pressed(KeyRepeat::No) {
            if let Some(scenario) = scenario_key(key) {
                mazda.set_scenario(scenario);
                ui.refresh(&mazda);
            }
            if let Some(event) = commander_button_event(key) {
                handle_event(&mut ui, &mut mazda, event);
            }
        }

        let pointer_down = window.get_mouse_down(MouseButton::Left);
        if pointer_down && !pointer_was_down {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
                // Discard mode bounds coordinates to the logical window dimensions.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let intent = ui.handle_pointer(x as usize, y as usize);
                apply_media_intent(&mut mazda, intent);
                ui.refresh(&mazda);
            }
        }
        pointer_was_down = pointer_down;

        if last_source_refresh.elapsed() >= SOURCE_REFRESH_INTERVAL {
            ui.refresh(&mazda);
            last_source_refresh = Instant::now();
        }
        ui.render(&mut framebuffer);
        window.update_with_buffer(framebuffer.pixels(), DISPLAY_WIDTH, DISPLAY_HEIGHT)?;
        frames += 1;
        if smoke_test && frames >= 90 {
            println!("Rendered {frames} native frames at {DISPLAY_WIDTH}x{DISPLAY_HEIGHT}; host mock only.");
            break;
        }
    }

    Ok(())
}

fn handle_event(ui: &mut UiModel, mock: &mut MockMazda, event: CommanderEvent) {
    apply_media_intent(mock, ui.handle_commander(event));
    ui.refresh(mock);
}

fn apply_media_intent(mock: &mut MockMazda, intent: Option<MediaIntent>) {
    match intent {
        Some(MediaIntent::Previous) => mock.previous_track(),
        Some(MediaIntent::TogglePlayback) => mock.toggle_playback(),
        Some(MediaIntent::Next) => mock.next_track(),
        None => {}
    }
}

const fn scenario_key(key: Key) -> Option<DemoScenario> {
    match key {
        Key::Key1 => Some(DemoScenario::Parked),
        Key::Key2 => Some(DemoScenario::Driving),
        Key::Key3 => Some(DemoScenario::Unavailable),
        Key::Key4 => Some(DemoScenario::Reverse),
        _ => None,
    }
}

const fn commander_rotation_event(key: Key) -> Option<CommanderEvent> {
    match key {
        Key::Right | Key::Down => Some(CommanderEvent::RotateClockwise),
        Key::Left | Key::Up => Some(CommanderEvent::RotateCounterClockwise),
        _ => None,
    }
}

const fn commander_button_event(key: Key) -> Option<CommanderEvent> {
    match key {
        Key::Enter | Key::Space => Some(CommanderEvent::Select),
        Key::Backspace => Some(CommanderEvent::Back),
        Key::H => Some(CommanderEvent::Home),
        Key::M => Some(CommanderEvent::Music),
        Key::N => Some(CommanderEvent::Navigation),
        Key::F => Some(CommanderEvent::Favorites),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{commander_button_event, commander_rotation_event};
    use super::{handle_event, scenario_key};
    use mazda_core::CommanderEvent;
    use mazda_core::{MazdaReadOnly, PlaybackState};
    use mazda_mock::{DemoScenario, MockMazda};
    use mazda_ui::{Framebuffer, Theme, UiModel, Units};
    use minifb::Key;

    #[test]
    fn commander_operates_mock_media_and_back_returns_to_page_navigation() {
        let mut mock = MockMazda::demo();
        let mut ui = UiModel::from_source(&mock);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        assert_eq!(mock.media_state().playback, PlaybackState::Paused);
        handle_event(&mut ui, &mut mock, CommanderEvent::RotateClockwise);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        assert_eq!(mock.media_state().title.as_deref(), Some("A Walk"));
        handle_event(&mut ui, &mut mock, CommanderEvent::Back);
        handle_event(&mut ui, &mut mock, CommanderEvent::RotateClockwise);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        assert_eq!(ui.screen(), mazda_ui::Screen::Drive);
        assert!(mock.vehicle_snapshot().is_confirmed_parked());
    }

    #[test]
    fn settings_revoke_focus_on_data_loss_and_cannot_change_until_parked() {
        let mut mock = MockMazda::demo();
        let mut ui = UiModel::from_source(&mock);
        handle_event(&mut ui, &mut mock, CommanderEvent::Favorites);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        assert_eq!(ui.preferences().units, Units::Metric);
        for scenario in [
            DemoScenario::Driving,
            DemoScenario::Unavailable,
            DemoScenario::Reverse,
        ] {
            mock.set_scenario(scenario);
            ui.refresh(&mock);
            for _ in 0..3 {
                handle_event(&mut ui, &mut mock, CommanderEvent::Select);
            }
            assert_eq!(ui.preferences().units, Units::Metric);
            assert_eq!(ui.preferences().theme, Theme::Night);
            ui.render(&mut Framebuffer::mazda_connect());
        }
        mock.set_scenario(DemoScenario::Parked);
        ui.refresh(&mock);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        handle_event(&mut ui, &mut mock, CommanderEvent::RotateClockwise);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        assert_eq!(ui.preferences().theme, Theme::Day);
        assert_eq!(ui.preferences().units, Units::Metric);
        assert_eq!(scenario_key(Key::Key3), Some(DemoScenario::Unavailable));
        assert_eq!(scenario_key(Key::Enter), None);
    }

    #[test]
    fn appearance_changes_pixels_and_unit_changes_do_not_mutate_source_data() {
        let mut mock = MockMazda::demo();
        let original = mock.vehicle_snapshot();
        let mut ui = UiModel::from_source(&mock);
        handle_event(&mut ui, &mut mock, CommanderEvent::Favorites);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        let mut night = Framebuffer::mazda_connect();
        ui.render(&mut night);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        handle_event(&mut ui, &mut mock, CommanderEvent::RotateClockwise);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        let mut day = Framebuffer::mazda_connect();
        ui.render(&mut day);
        assert_ne!(night.pixels()[0], day.pixels()[0]);
        assert_eq!(mock.vehicle_snapshot(), original);
    }

    #[test]
    fn pointer_actions_follow_the_same_settings_policy() {
        let mut mock = MockMazda::demo();
        let mut ui = UiModel::from_source(&mock);
        assert_eq!(
            ui.handle_pointer(420, 270),
            Some(mazda_ui::MediaIntent::TogglePlayback)
        );
        ui.handle_pointer(50, 320);
        assert_eq!(ui.screen(), mazda_ui::Screen::Settings);
        ui.handle_pointer(250, 150);
        assert_eq!(ui.preferences().units, Units::Metric);
        mock.set_scenario(DemoScenario::Unavailable);
        ui.refresh(&mock);
        ui.handle_pointer(250, 150);
        ui.handle_pointer(250, 240);
        assert_eq!(ui.preferences().units, Units::Metric);
        assert_eq!(ui.preferences().theme, Theme::Night);
        assert_eq!(ui.handle_pointer(usize::MAX, usize::MAX), None);
    }

    #[test]
    fn rotation_keys_map_only_to_repeatable_commander_events() {
        assert_eq!(
            commander_rotation_event(Key::Right),
            Some(CommanderEvent::RotateClockwise)
        );
        assert_eq!(
            commander_rotation_event(Key::Down),
            Some(CommanderEvent::RotateClockwise)
        );
        assert_eq!(
            commander_rotation_event(Key::Left),
            Some(CommanderEvent::RotateCounterClockwise)
        );
        assert_eq!(
            commander_rotation_event(Key::Up),
            Some(CommanderEvent::RotateCounterClockwise)
        );
        assert_eq!(commander_rotation_event(Key::Enter), None);
    }

    #[test]
    fn button_keys_map_only_to_single_press_commander_events() {
        let cases = [
            (Key::Enter, CommanderEvent::Select),
            (Key::Space, CommanderEvent::Select),
            (Key::Backspace, CommanderEvent::Back),
            (Key::H, CommanderEvent::Home),
            (Key::M, CommanderEvent::Music),
            (Key::N, CommanderEvent::Navigation),
            (Key::F, CommanderEvent::Favorites),
        ];

        for (key, event) in cases {
            assert_eq!(commander_button_event(key), Some(event));
        }
        assert_eq!(commander_button_event(Key::Right), None);
    }
}
