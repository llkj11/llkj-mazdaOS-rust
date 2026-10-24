//! Clock settings session; no host clock, GPS or vehicle-setting writes.
use mazda_core::CommanderEvent;

pub(crate) const CLOCK_ROWS: [&str; 5] = [
    "Adjust Time",
    "GPS Sync",
    "Time Format",
    "Time Zone Select",
    "Daylight Savings Time",
];
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TimeFormat {
    #[default]
    Hours12,
    Hours24,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockPreferences {
    pub hour: u8,
    pub minute: u8,
    pub format: TimeFormat,
    pub gps_sync: bool,
    pub daylight_saving: bool,
    pub time_zone: usize,
}
impl Default for ClockPreferences {
    fn default() -> Self {
        Self {
            hour: 12,
            minute: 0,
            format: TimeFormat::Hours12,
            gps_sync: false,
            daylight_saving: false,
            time_zone: 14,
        }
    }
}
impl ClockPreferences {
    #[must_use]
    pub fn label(self) -> String {
        format!("{}:{:02}", self.display_hour(), self.minute)
    }
    #[must_use]
    pub const fn display_hour(self) -> u8 {
        match self.format {
            TimeFormat::Hours24 => self.hour,
            TimeFormat::Hours12 => {
                if self.hour.is_multiple_of(12) {
                    12
                } else {
                    self.hour % 12
                }
            }
        }
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ClockPage {
    #[default]
    Root,
    Adjust {
        focus: usize,
    },
    Zones {
        focus: usize,
    },
}
#[derive(Debug, Clone, Default)]
pub(crate) struct ClockSession {
    pub values: ClockPreferences,
    pub page: ClockPage,
    pub navigation_available: bool,
}
impl ClockSession {
    pub fn enabled(&self, row: usize, parked: bool) -> bool {
        parked
            && match row {
                0 => !self.values.gps_sync,
                1 | 2 => true,
                3 | 4 => !self.values.gps_sync || !self.navigation_available,
                _ => false,
            }
    }
    pub fn button_enabled(&self, button: usize, parked: bool) -> bool {
        button == 0
            || (button < 6
                && self.enabled(0, parked)
                && (button != 5 || self.values.format == TimeFormat::Hours12))
    }
    pub fn restrict(&mut self, parked: bool) {
        if !self.enabled(0, parked) && matches!(self.page, ClockPage::Adjust { .. }) {
            self.page = ClockPage::Adjust { focus: 0 };
        }
    }
    pub fn select(&mut self, row: usize, parked: bool, zones: usize) -> Option<&'static str> {
        if !parked {
            return Some("Preferences locked: valid zero speed and Park required");
        }
        if !self.enabled(row, parked) {
            return Some("Clock option is controlled by GPS Sync");
        }
        match row {
            0 => self.page = ClockPage::Adjust { focus: 0 },
            1 => self.values.gps_sync = !self.values.gps_sync,
            2 => {
                self.values.format = match self.values.format {
                    TimeFormat::Hours12 => TimeFormat::Hours24,
                    TimeFormat::Hours24 => TimeFormat::Hours12,
                }
            }
            3 => {
                if zones == 0 {
                    return Some("Original time-zone labels are not loaded");
                }
                self.page = ClockPage::Zones {
                    focus: self.values.time_zone.min(zones - 1),
                };
            }
            4 => self.values.daylight_saving = !self.values.daylight_saving,
            _ => {}
        }
        None
    }
    pub fn activate_button(&mut self, button: usize, parked: bool) {
        if !self.button_enabled(button, parked) {
            return;
        }
        match button {
            0 => self.page = ClockPage::Root,
            1 | 2 => {
                let step = if button == 1 { -1 } else { 1 };
                self.values.hour = if self.values.format == TimeFormat::Hours12 {
                    let period = self.values.hour / 12 * 12;
                    period
                        + u8::try_from((i16::from(self.values.hour % 12) + step).rem_euclid(12))
                            .expect("bounded hour")
                } else {
                    u8::try_from((i16::from(self.values.hour) + step).rem_euclid(24))
                        .expect("bounded hour")
                };
            }
            3 | 4 => {
                let step = if button == 3 { -1 } else { 1 };
                self.values.minute =
                    u8::try_from((i16::from(self.values.minute) + step).rem_euclid(60))
                        .expect("bounded minute");
            }
            5 => self.values.hour = (self.values.hour + 12) % 24,
            _ => {}
        }
    }
    pub fn active_event(&mut self, event: CommanderEvent, parked: bool, zones: usize) -> bool {
        match self.page {
            ClockPage::Root => return false,
            ClockPage::Adjust { mut focus } => match event {
                CommanderEvent::Back => self.page = ClockPage::Root,
                CommanderEvent::Select => self.activate_button(focus, parked),
                CommanderEvent::RotateClockwise | CommanderEvent::RotateCounterClockwise => {
                    let next = if event == CommanderEvent::RotateClockwise {
                        (focus + 1..6).find(|&i| self.button_enabled(i, parked))
                    } else {
                        (0..focus).rev().find(|&i| self.button_enabled(i, parked))
                    };
                    if let Some(next) = next {
                        focus = next;
                    }
                    self.page = ClockPage::Adjust { focus };
                }
                _ => {}
            },
            ClockPage::Zones { mut focus } => match event {
                CommanderEvent::Back => self.page = ClockPage::Root,
                CommanderEvent::RotateClockwise => {
                    focus = (focus + 1).min(zones.saturating_sub(1));
                    self.page = ClockPage::Zones { focus };
                }
                CommanderEvent::RotateCounterClockwise => {
                    focus = focus.saturating_sub(1);
                    self.page = ClockPage::Zones { focus };
                }
                CommanderEvent::Select if self.enabled(3, parked) && focus < zones => {
                    self.values.time_zone = focus;
                }
                _ => {}
            },
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_hour_minute_wrap_and_separate_meridiem_are_preserved() {
        let mut clock = ClockSession::default();
        clock.values.hour = 0;
        clock.activate_button(1, true);
        assert_eq!(clock.values.hour, 11);
        clock.activate_button(2, true);
        assert_eq!(clock.values.hour, 0);
        clock.activate_button(5, true);
        assert_eq!(clock.values.hour, 12);
        clock.activate_button(3, true);
        assert_eq!(clock.values.minute, 59);
        assert_eq!(clock.values.hour, 12);
        clock.activate_button(4, true);
        assert_eq!(clock.values.minute, 0);
        clock.values.format = TimeFormat::Hours24;
        clock.values.hour = 0;
        clock.activate_button(1, true);
        assert_eq!(clock.values.hour, 23);
        clock.activate_button(2, true);
        assert_eq!(clock.values.hour, 0);
        clock.activate_button(5, true);
        assert_eq!(clock.values.hour, 0);
    }
    #[test]
    fn gps_sync_immediately_applies_navigation_dependent_restrictions() {
        let mut clock = ClockSession::default();
        clock.select(1, true, 101);
        assert!(clock.values.gps_sync);
        assert!(!clock.enabled(0, true));
        assert!(clock.enabled(3, true));
        assert!(clock.enabled(4, true));
        clock.navigation_available = true;
        assert!(!clock.enabled(3, true));
        assert!(!clock.enabled(4, true));
        assert!(clock.select(0, true, 101).is_some());
        clock.select(1, true, 101);
        assert!(!clock.values.gps_sync);
        assert!(clock.enabled(0, true));
        assert!(clock.enabled(3, true));
    }
    #[test]
    fn adjust_controls_skip_disabled_ampm_and_back_survives_restrictions() {
        let mut clock = ClockSession::default();
        clock.values.format = TimeFormat::Hours24;
        clock.select(0, true, 101);
        for _ in 0..20 {
            clock.active_event(CommanderEvent::RotateClockwise, true, 101);
        }
        assert_eq!(clock.page, ClockPage::Adjust { focus: 4 });
        clock.restrict(false);
        assert_eq!(clock.page, ClockPage::Adjust { focus: 0 });
        let before = clock.values;
        clock.activate_button(2, false);
        assert_eq!(clock.values, before);
        clock.active_event(CommanderEvent::Select, false, 101);
        assert_eq!(clock.page, ClockPage::Root);
    }
    #[test]
    fn full_zone_range_radio_selection_and_empty_catalog_are_bounded() {
        let mut clock = ClockSession::default();
        assert!(clock.select(3, true, 0).is_some());
        clock.select(3, true, 101);
        assert_eq!(clock.page, ClockPage::Zones { focus: 14 });
        for _ in 0..150 {
            clock.active_event(CommanderEvent::RotateClockwise, true, 101);
        }
        clock.active_event(CommanderEvent::Select, true, 101);
        assert_eq!(clock.values.time_zone, 100);
        assert_eq!(clock.page, ClockPage::Zones { focus: 100 });
        for _ in 0..150 {
            clock.active_event(CommanderEvent::RotateCounterClockwise, true, 101);
        }
        clock.active_event(CommanderEvent::Select, false, 101);
        assert_eq!(clock.values.time_zone, 100);
        clock.active_event(CommanderEvent::Back, true, 101);
        assert_eq!(clock.page, ClockPage::Root);
    }
    #[test]
    fn format_and_daylight_are_local_preferences_and_status_label_uses_them() {
        let mut clock = ClockSession::default();
        clock.values.hour = 23;
        assert_eq!(clock.values.label(), "11:00");
        clock.select(2, true, 101);
        assert_eq!(clock.values.label(), "23:00");
        clock.select(4, true, 101);
        assert!(clock.values.daylight_saving);
        let before = clock.values;
        assert!(clock.select(4, false, 101).is_some());
        assert_eq!(clock.values, before);
    }
}
