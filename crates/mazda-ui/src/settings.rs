//! Session-only Settings state. No OEM transport or vehicle-write capability.
use mazda_core::CommanderEvent;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DisplayMode {
    #[default]
    Auto,
    Day,
    Night,
}
impl DisplayMode {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Auto => 0,
            Self::Day => 1,
            Self::Night => 2,
        }
    }
    const fn next(self) -> Self {
        match self {
            Self::Auto => Self::Day,
            Self::Day => Self::Night,
            Self::Night => Self::Auto,
        }
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DisplayPower {
    #[default]
    On,
    Off,
    Clock,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DisplayPreferences {
    pub mode: DisplayMode,
    pub brightness: i8,
    pub contrast: i8,
    pub power: DisplayPower,
}
#[derive(Debug, Clone, Default)]
pub(crate) struct DisplaySession {
    pub values: DisplayPreferences,
    pub editing: Option<usize>,
    pub reset: Option<bool>,
    pub pointer_drag: bool,
}
impl DisplaySession {
    pub fn cancel_interaction(&mut self) {
        self.editing = None;
        self.reset = None;
        self.pointer_drag = false;
    }
    pub fn select(&mut self, row: usize, parked: bool) -> Option<&'static str> {
        if !parked {
            self.cancel_interaction();
            return Some("Preferences locked: valid zero speed and Park required");
        }
        match row {
            0 => self.values.power = DisplayPower::Off,
            1 => self.values.power = DisplayPower::Clock,
            2 => self.values.mode = self.values.mode.next(),
            3 | 4 => self.editing = Some(row),
            5 => self.reset = Some(false),
            _ => {}
        }
        None
    }
    pub fn active_event(&mut self, event: CommanderEvent, parked: bool) -> bool {
        if !parked {
            self.cancel_interaction();
            return false;
        }
        if let Some(yes) = self.reset {
            match event {
                CommanderEvent::RotateClockwise => self.reset = Some(true),
                CommanderEvent::RotateCounterClockwise => self.reset = Some(false),
                CommanderEvent::Select => {
                    if yes {
                        self.values = DisplayPreferences::default();
                    }
                    self.reset = None;
                }
                CommanderEvent::Back => self.reset = None,
                _ => {}
            }
            return true;
        }
        if let Some(row) = self.editing {
            let value = if row == 3 {
                &mut self.values.brightness
            } else {
                &mut self.values.contrast
            };
            match event {
                CommanderEvent::RotateClockwise => *value = (*value + 1).min(5),
                CommanderEvent::RotateCounterClockwise => *value = (*value - 1).max(-5),
                CommanderEvent::Select | CommanderEvent::Back => {
                    self.editing = None;
                    self.pointer_drag = false;
                }
                _ => {}
            }
            return true;
        }
        false
    }
    pub fn pointer_value(&mut self, row: usize, x: usize, parked: bool) {
        if !parked {
            self.cancel_interaction();
            return;
        }
        if row == 2 {
            self.values.mode = match x.saturating_sub(384) / 118 {
                0 => DisplayMode::Auto,
                1 => DisplayMode::Day,
                _ => DisplayMode::Night,
            };
        } else if row == 3 || row == 4 {
            let position = x.saturating_sub(389).min(344);
            let value = i8::try_from((position * 10 + 172) / 344).unwrap_or(10) - 5;
            if row == 3 {
                self.values.brightness = value;
            } else {
                self.values.contrast = value;
            }
            self.editing = Some(row);
            self.pointer_drag = true;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_sliders_edit_without_moving_list_and_keep_changes_on_exit() {
        let mut state = DisplaySession::default();
        state.select(3, true);
        for _ in 0..50 {
            assert!(state.active_event(CommanderEvent::RotateClockwise, true));
        }
        assert_eq!(state.values.brightness, 5);
        for _ in 0..50 {
            state.active_event(CommanderEvent::RotateCounterClockwise, true);
        }
        assert_eq!(state.values.brightness, -5);
        state.active_event(CommanderEvent::Back, true);
        assert_eq!(state.editing, None);
        assert_eq!(state.values.brightness, -5);
    }
    #[test]
    fn reset_starts_with_no_cancel_preserves_values_and_yes_restores_only_display() {
        let mut state = DisplaySession::default();
        state.select(2, true);
        state.pointer_value(3, 733, true);
        state.cancel_interaction();
        state.select(5, true);
        assert_eq!(state.reset, Some(false));
        state.active_event(CommanderEvent::Select, true);
        assert_eq!(state.values.mode, DisplayMode::Day);
        assert_eq!(state.values.brightness, 5);
        state.select(5, true);
        state.active_event(CommanderEvent::RotateClockwise, true);
        state.active_event(CommanderEvent::Select, true);
        assert_eq!(state.values, DisplayPreferences::default());
    }
    #[test]
    fn restrictions_revoke_active_edit_dialog_and_pointer_drag_without_more_mutation() {
        let mut state = DisplaySession::default();
        state.select(3, true);
        state.active_event(CommanderEvent::RotateClockwise, true);
        let before = state.values;
        state.active_event(CommanderEvent::RotateClockwise, false);
        assert_eq!(state.values, before);
        assert_eq!(state.editing, None);
        state.select(5, true);
        state.active_event(CommanderEvent::Select, false);
        assert_eq!(state.reset, None);
        state.pointer_value(4, usize::MAX, false);
        assert_eq!(state.values, before);
        assert!(state.select(0, false).is_some());
        assert_eq!(state.values.power, DisplayPower::On);
    }
    #[test]
    fn pointer_choices_and_slider_endpoints_use_original_control_ranges() {
        let mut state = DisplaySession::default();
        for (x, mode) in [
            (385, DisplayMode::Auto),
            (520, DisplayMode::Day),
            (700, DisplayMode::Night),
        ] {
            state.pointer_value(2, x, true);
            assert_eq!(state.values.mode, mode);
        }
        for (x, value) in [(0, -5), (389, -5), (561, 0), (733, 5), (usize::MAX, 5)] {
            state.pointer_value(4, x, true);
            assert_eq!(state.values.contrast, value);
        }
    }
}
