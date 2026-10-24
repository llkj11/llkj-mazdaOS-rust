use std::path::PathBuf;
use std::time::{Duration, Instant};
mod assets;
mod clock_hold;
use clock_hold::{ClockHold, Source as HoldSource};

use mazda_core::CommanderEvent;
use mazda_mock::{DemoScenario, MockMazda};
use mazda_ui::{Framebuffer, MediaIntent, SoundProfile, UiModel, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};

const SOURCE_REFRESH_INTERVAL: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke_test = std::env::args().any(|argument| argument == "--smoke-test");
    let arguments: Vec<_> = std::env::args_os().collect();
    let audio_profile = parse_audio_profile(&arguments)?;
    let base_title = format!(
        "Mazda Connect - Firmware UI + Inter - {} audio fixture",
        audio_profile.label()
    );
    let artwork = load_artwork(&arguments)?;
    let mut frames = 0_u32;
    let mut pointer_was_down = false;
    let mut clock_hold: Option<ClockHold> = None;
    let mut mazda = MockMazda::demo();
    let mut ui = UiModel::from_source(&mazda);
    ui.set_assets(artwork);
    configure_fixtures(&mut ui, &arguments, audio_profile)?;
    let mut framebuffer = Framebuffer::mazda_connect();
    let mut window = Window::new(
        &base_title,
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
    let mut last_notice = None;
    let mut last_frame = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        handle_keys(&window, &mut ui, &mut mazda, &mut clock_hold);

        let pointer_down = window.get_mouse_down(MouseButton::Left);
        if pointer_down && !pointer_was_down {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
                // Discard mode bounds coordinates to the logical window dimensions.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let intent = if let Some(button) = ui
                    .clock_button_at(x as usize, y as usize)
                    .filter(|b| (1..=4).contains(b))
                {
                    ui.focus_clock_button(button);
                    clock_hold = Some(ClockHold::new(button, HoldSource::Pointer, Instant::now()));
                    None
                } else {
                    ui.handle_pointer(x as usize, y as usize)
                };
                apply_media_intent(&mut mazda, intent);
                ui.refresh(&mazda);
            }
        }
        if pointer_down && pointer_was_down {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                ui.handle_pointer_drag(x as usize, y as usize);
            }
        }
        if !pointer_down && pointer_was_down {
            ui.handle_pointer_release();
        }
        pointer_was_down = pointer_down;

        if last_source_refresh.elapsed() >= SOURCE_REFRESH_INTERVAL {
            ui.refresh(&mazda);
            last_source_refresh = Instant::now();
        }
        poll_clock_hold(&mut clock_hold, &window, &mut ui, &mut mazda);
        advance_simulation(&mut last_frame, &mut ui);
        if ui.notice() != last_notice {
            window.set_title(&ui.notice().map_or_else(
                || base_title.clone(),
                |notice| format!("Host Simulation — {notice}"),
            ));
            last_notice = ui.notice();
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

fn handle_keys(
    window: &Window,
    ui: &mut UiModel,
    mazda: &mut MockMazda,
    clock_hold: &mut Option<ClockHold>,
) {
    for key in window.get_keys_pressed(KeyRepeat::Yes) {
        if let Some(event) = commander_rotation_event(key) {
            handle_event(ui, mazda, event);
        }
    }
    for key in window.get_keys_pressed(KeyRepeat::No) {
        if matches!(key, Key::Enter | Key::Space) {
            if let Some(button) = ui.clock_repeat_target() {
                *clock_hold = Some(ClockHold::new(button, HoldSource::Key(key), Instant::now()));
                continue;
            }
        }
        if let Some(scenario) = scenario_key(key) {
            mazda.set_scenario(scenario);
            ui.refresh(mazda);
        }
        if let Some(event) = commander_button_event(key) {
            handle_event(ui, mazda, event);
        }
    }
}

fn advance_simulation(last_frame: &mut Instant, ui: &mut UiModel) {
    let now = Instant::now();
    ui.advance_simulation(now.saturating_duration_since(*last_frame));
    *last_frame = now;
}

fn configure_fixtures(
    ui: &mut UiModel,
    arguments: &[std::ffi::OsString],
    audio_profile: SoundProfile,
) -> Result<(), &'static str> {
    ui.set_sound_profile(audio_profile);
    ui.set_hud_installed(arguments.iter().any(|arg| arg == "--hud"));
    ui.set_guidance_available(!arguments.iter().any(|arg| arg == "--guidance-disabled"));
    ui.set_clock_navigation_available(arguments.iter().any(|arg| arg == "--navigation-available"));
    ui.set_clock_gps_sync(arguments.iter().any(|arg| arg == "--clock-gps-sync"));
    ui.set_system_equipment(mazda_ui::SystemEquipment {
        temperature_units: arguments.iter().any(|arg| arg == "--temperature-units"),
        distance_units: arguments.iter().any(|arg| arg == "--distance-units"),
        services_ready: !arguments
            .iter()
            .any(|arg| arg == "--system-services-unavailable"),
        new_cmu: arguments.iter().any(|arg| arg == "--new-cmu"),
    });
    ui.set_language_change_failure(arguments.iter().any(|arg| arg == "--language-change-fails"));
    ui.set_projection_fixture(arguments.iter().any(|arg| arg == "--projection-fixture"));
    if let Some(index) = arguments
        .iter()
        .position(|arg| arg == "--bluetooth-fixture")
    {
        let fixture = arguments.get(index + 1).and_then(|arg| arg.to_str())
            .and_then(mazda_ui::BluetoothFixture::parse)
            .ok_or("--bluetooth-fixture requires empty, paired, ssp, pair-fails, connect-fails, disconnect-fails or delete-fails")?;
        ui.set_bluetooth_fixture(fixture);
    }
    if let Some(index) = arguments.iter().position(|arg| arg == "--music-update") {
        let fixture = arguments.get(index + 1).and_then(|arg| arg.to_str())
            .and_then(mazda_ui::MusicUpdateFixture::parse)
            .ok_or("--music-update requires no-media, available, search-fails, invalid, install-fails, retry-invalid, unavailable or needs-failsafe")?;
        ui.set_music_update_fixture(fixture);
    }
    if let Some(index) = arguments.iter().position(|arg| arg == "--language") {
        let code = arguments
            .get(index + 1)
            .and_then(|s| s.to_str())
            .ok_or("--language requires en_US, es_MX or fr_CN")?;
        if !ui.set_host_language(code) {
            return Err("--language requires an imported en_US, es_MX or fr_CN fixture");
        }
    }
    Ok(())
}

fn load_artwork(
    arguments: &[std::ffi::OsString],
) -> Result<mazda_ui::StockAssets, Box<dyn std::error::Error>> {
    let explicit_assets = arguments
        .iter()
        .position(|arg| arg == "--assets")
        .and_then(|index| arguments.get(index + 1))
        .map(PathBuf::from);
    let packaged_assets = std::env::current_exe()?
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("Resources/firmware-assets"));
    let asset_directory = explicit_assets
        .or_else(|| packaged_assets.filter(|p| p.is_dir()))
        .unwrap_or_else(|| PathBuf::from("target/owner-firmware/render-assets"));
    let artwork=assets::load(&asset_directory).map_err(|error|std::io::Error::new(error.kind(),format!("Cannot load owner firmware artwork at {}: {error}. Run tools/import-firmware-ui.py first.",asset_directory.display())))?;
    Ok(artwork)
}

fn poll_clock_hold(
    hold: &mut Option<ClockHold>,
    window: &Window,
    ui: &mut UiModel,
    mock: &mut MockMazda,
) {
    if let Some(held) = hold.as_mut() {
        let down = match held.source {
            HoldSource::Key(key) => window.is_key_down(key),
            HoldSource::Pointer => window.get_mouse_down(MouseButton::Left),
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let inside = match held.source {
            HoldSource::Key(_) => true,
            HoldSource::Pointer => {
                window
                    .get_mouse_pos(MouseMode::Discard)
                    .and_then(|(x, y)| ui.clock_button_at(x as usize, y as usize))
                    == Some(held.button)
            }
        };
        let (changes, finished) = held.poll(ui.clock_repeat_target(), down, inside, Instant::now());
        for _ in 0..changes {
            handle_event(ui, mock, CommanderEvent::Select);
        }
        if finished {
            *hold = None;
        }
    }
}

fn parse_audio_profile(arguments: &[std::ffi::OsString]) -> Result<SoundProfile, &'static str> {
    let Some(index) = arguments.iter().position(|arg| arg == "--audio-profile") else {
        return Ok(SoundProfile::Standard);
    };
    match arguments.get(index + 1).and_then(|arg| arg.to_str()) {
        Some("standard") => Ok(SoundProfile::Standard),
        Some("bose") => Ok(SoundProfile::Bose),
        Some("bose-j12") => Ok(SoundProfile::BoseJ12),
        _ => Err("--audio-profile requires standard, bose or bose-j12"),
    }
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
    use mazda_core::MazdaReadOnly;
    use mazda_mock::{DemoScenario, MockMazda};
    use mazda_ui::UiModel;
    use minifb::Key;

    #[test]
    fn home_routes_to_firmware_domains_and_preserves_mock_observations() {
        use mazda_ui::Screen;
        let mut mock = MockMazda::demo();
        let original = mock.vehicle_snapshot();
        let original_media = mock.media_state();
        let mut ui = UiModel::from_source(&mock);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        assert_eq!(ui.screen(), Screen::Communication);
        handle_event(&mut ui, &mut mock, CommanderEvent::Back);
        assert_eq!(ui.selection(), Screen::Communication);
        handle_event(&mut ui, &mut mock, CommanderEvent::Navigation);
        assert_eq!(ui.screen(), Screen::Navigation);
        handle_event(&mut ui, &mut mock, CommanderEvent::Music);
        assert_eq!(ui.screen(), Screen::Entertainment);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        assert!(ui.notice().is_some());
        assert_eq!(mock.vehicle_snapshot(), original);
        assert_eq!(mock.media_state(), original_media);
    }
    #[test]
    fn unavailable_observations_cannot_authorize_preferences() {
        let mut mock = MockMazda::demo();
        let mut ui = UiModel::from_source(&mock);
        for _ in 0..2 {
            handle_event(&mut ui, &mut mock, CommanderEvent::RotateClockwise);
        }
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        mock.set_scenario(DemoScenario::Unavailable);
        ui.refresh(&mock);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        assert!(ui.notice().unwrap().contains("locked"));
        assert!(!mock.vehicle_snapshot().is_confirmed_parked());
        assert_eq!(scenario_key(Key::Key3), Some(DemoScenario::Unavailable));
        mock.set_scenario(DemoScenario::Parked);
        ui.refresh(&mock);
        assert_eq!(ui.notice(), None);
    }
    #[test]
    fn driving_revokes_sound_editing_without_touching_source_observations() {
        let mut mock = MockMazda::demo();
        let media = mock.media_state();
        let mut ui = UiModel::from_source(&mock);
        for _ in 0..2 {
            handle_event(&mut ui, &mut mock, CommanderEvent::RotateClockwise);
        }
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        for _ in 0..2 {
            handle_event(&mut ui, &mut mock, CommanderEvent::RotateClockwise);
        }
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        handle_event(&mut ui, &mut mock, CommanderEvent::RotateClockwise);
        assert_eq!(ui.sound_preferences().bass, 1);
        mock.set_scenario(DemoScenario::Driving);
        ui.refresh(&mock);
        assert!(!ui.sound_adjusting());
        let before = ui.sound_preferences();
        handle_event(&mut ui, &mut mock, CommanderEvent::Select);
        assert_eq!(ui.sound_preferences(), before);
        assert!(ui.notice().unwrap().contains("locked"));
        assert_eq!(mock.media_state(), media);
    }
    #[test]
    fn equipment_arguments_are_explicit_and_invalid_profiles_fail() {
        let args = |items: &[&str]| {
            items
                .iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            super::parse_audio_profile(&args(&[])),
            Ok(mazda_ui::SoundProfile::Standard)
        );
        assert_eq!(
            super::parse_audio_profile(&args(&["--audio-profile", "bose"])),
            Ok(mazda_ui::SoundProfile::Bose)
        );
        assert_eq!(
            super::parse_audio_profile(&args(&["--audio-profile", "bose-j12"])),
            Ok(mazda_ui::SoundProfile::BoseJ12)
        );
        assert!(super::parse_audio_profile(&args(&["--audio-profile"])).is_err());
        assert!(super::parse_audio_profile(&args(&["--audio-profile", "typo"])).is_err());
    }
    #[test]
    fn language_fixture_arguments_fail_before_window_creation_without_valid_owner_data() {
        let mock = MockMazda::demo();
        let mut ui = UiModel::from_source(&mock);
        for values in [
            vec!["app", "--language"],
            vec!["app", "--language", "missing"],
            vec!["app", "--language", "fr_CN"],
        ] {
            let args: Vec<_> = values.iter().map(std::ffi::OsString::from).collect();
            assert!(
                super::configure_fixtures(&mut ui, &args, mazda_ui::SoundProfile::Standard)
                    .is_err()
            );
            assert_eq!(ui.language_code(), "en_US");
        }
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
    #[test]
    fn music_fixture_parser_rejects_missing_or_arbitrary_inputs() {
        let mock = MockMazda::demo();
        let mut ui = UiModel::from_source(&mock);
        for values in [
            vec!["app", "--music-update"],
            vec!["app", "--music-update", "/Volumes/USB/update"],
        ] {
            let args: Vec<_> = values.iter().map(std::ffi::OsString::from).collect();
            assert!(
                super::configure_fixtures(&mut ui, &args, mazda_ui::SoundProfile::Standard)
                    .is_err()
            );
        }
        for value in [
            "no-media",
            "available",
            "search-fails",
            "invalid",
            "install-fails",
            "retry-invalid",
            "unavailable",
            "needs-failsafe",
        ] {
            let args = ["app", "--music-update", value].map(std::ffi::OsString::from);
            assert!(
                super::configure_fixtures(&mut ui, &args, mazda_ui::SoundProfile::Standard).is_ok()
            );
        }
    }
    #[test]
    fn bluetooth_fixtures_reject_unknown_inputs_and_never_inspect_real_devices() {
        let mock = MockMazda::demo();
        let mut ui = UiModel::from_source(&mock);
        for values in [
            vec!["app", "--bluetooth-fixture"],
            vec!["app", "--bluetooth-fixture", "/dev/device"],
        ] {
            let args: Vec<_> = values.iter().map(std::ffi::OsString::from).collect();
            assert!(
                super::configure_fixtures(&mut ui, &args, mazda_ui::SoundProfile::Standard)
                    .is_err()
            );
        }
        for value in [
            "empty",
            "paired",
            "ssp",
            "pair-fails",
            "connect-fails",
            "disconnect-fails",
            "delete-fails",
        ] {
            let args = ["app", "--bluetooth-fixture", value].map(std::ffi::OsString::from);
            assert!(
                super::configure_fixtures(&mut ui, &args, mazda_ui::SoundProfile::Standard).is_ok()
            );
        }
    }
}
