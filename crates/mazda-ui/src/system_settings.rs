//! NA `SystemTab` preferences and host-only reset flow; no OS or vehicle writes.
use mazda_core::CommanderEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemControl {
    ToolTips,
    VrExamples,
    Language,
    Temperature,
    Distance,
    MusicDatabase,
    Restore,
    About,
}
impl SystemControl {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::ToolTips => "Tool Tips",
            Self::VrExamples => "VR Examples",
            Self::Language => "Language",
            Self::Temperature => "Temperature ",
            Self::Distance => "Distance",
            Self::MusicDatabase => "Music Database Update",
            Self::Restore => "Restore All Factory Settings",
            Self::About => "About",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Independent source preferences"
)]
pub struct SystemPreferences {
    pub tool_tips: bool,
    pub vr_examples: bool,
    pub celsius: bool,
    pub kilometers: bool,
}
impl Default for SystemPreferences {
    fn default() -> Self {
        Self {
            tool_tips: false,
            vr_examples: false,
            celsius: true,
            kilometers: true,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Independent source availability messages"
)]
pub struct SystemEquipment {
    pub temperature_units: bool,
    pub distance_units: bool,
    pub services_ready: bool,
    pub new_cmu: bool,
}
impl Default for SystemEquipment {
    fn default() -> Self {
        Self {
            temperature_units: false,
            distance_units: false,
            services_ready: true,
            new_cmu: false,
        }
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum SystemPage {
    #[default]
    Root,
    About {
        focus: usize,
    },
    Legal {
        scroll: usize,
    },
    Version,
    Confirm {
        yes: bool,
    },
    Progress {
        elapsed_ms: u64,
    },
}
#[derive(Debug, Clone, Default)]
pub(crate) struct SystemSession {
    pub values: SystemPreferences,
    pub equipment: SystemEquipment,
    pub page: SystemPage,
}
impl SystemSession {
    pub fn controls(&self) -> &'static [SystemControl] {
        use SystemControl::{
            About, Distance, Language, MusicDatabase, Restore, Temperature, ToolTips, VrExamples,
        };
        match (
            self.equipment.temperature_units,
            self.equipment.distance_units,
        ) {
            (false, false) => &[
                ToolTips,
                VrExamples,
                Language,
                MusicDatabase,
                Restore,
                About,
            ],
            (true, false) => &[
                ToolTips,
                VrExamples,
                Language,
                Temperature,
                MusicDatabase,
                Restore,
                About,
            ],
            (false, true) => &[
                ToolTips,
                VrExamples,
                Language,
                Distance,
                MusicDatabase,
                Restore,
                About,
            ],
            (true, true) => &[
                ToolTips,
                VrExamples,
                Language,
                Temperature,
                Distance,
                MusicDatabase,
                Restore,
                About,
            ],
        }
    }
    pub fn labels(&self) -> &'static [&'static str] {
        use SystemControl::{
            About, Distance, Language, MusicDatabase, Restore, Temperature, ToolTips, VrExamples,
        };
        const BASE: [&str; 6] = [
            ToolTips.label(),
            VrExamples.label(),
            Language.label(),
            MusicDatabase.label(),
            Restore.label(),
            About.label(),
        ];
        const TEMP: [&str; 7] = [
            ToolTips.label(),
            VrExamples.label(),
            Language.label(),
            Temperature.label(),
            MusicDatabase.label(),
            Restore.label(),
            About.label(),
        ];
        const DIST: [&str; 7] = [
            ToolTips.label(),
            VrExamples.label(),
            Language.label(),
            Distance.label(),
            MusicDatabase.label(),
            Restore.label(),
            About.label(),
        ];
        const BOTH: [&str; 8] = [
            ToolTips.label(),
            VrExamples.label(),
            Language.label(),
            Temperature.label(),
            Distance.label(),
            MusicDatabase.label(),
            Restore.label(),
            About.label(),
        ];
        match (
            self.equipment.temperature_units,
            self.equipment.distance_units,
        ) {
            (false, false) => &BASE,
            (true, false) => &TEMP,
            (false, true) => &DIST,
            (true, true) => &BOTH,
        }
    }
    pub fn allowed(&self, control: SystemControl, parked: bool) -> bool {
        parked
            && match control {
                SystemControl::Restore => self.equipment.services_ready,
                SystemControl::Distance => !self.equipment.new_cmu || self.equipment.services_ready,
                _ => true,
            }
    }
    pub fn restrict(&mut self, parked: bool) {
        if !self.allowed(SystemControl::Restore, parked)
            && matches!(
                self.page,
                SystemPage::Confirm { .. } | SystemPage::Progress { .. }
            )
        {
            self.page = SystemPage::Root;
        }
    }
    pub fn select(&mut self, row: usize, parked: bool) -> Option<&'static str> {
        let control = *self.controls().get(row)?;
        if !self.allowed(control, parked) {
            return Some("Preferences locked: valid Park and simulated services required");
        }
        match control {
            SystemControl::ToolTips => self.values.tool_tips = !self.values.tool_tips,
            SystemControl::VrExamples => self.values.vr_examples = !self.values.vr_examples,
            SystemControl::Temperature => self.values.celsius = !self.values.celsius,
            SystemControl::Distance => self.values.kilometers = !self.values.kilometers,
            SystemControl::Restore => self.page = SystemPage::Confirm { yes: false },
            SystemControl::About => self.page = SystemPage::About { focus: 0 },
            SystemControl::Language => return Some("Language selection has not yet been ported"),
            SystemControl::MusicDatabase => {
                return Some("Music database update service is not connected in this host preview")
            }
        }
        None
    }
    pub fn active_event(&mut self, event: CommanderEvent, parked: bool, max_scroll: usize) -> bool {
        self.restrict(parked);
        match self.page {
            SystemPage::Root => return false,
            SystemPage::About { mut focus } => {
                match event {
                    CommanderEvent::RotateClockwise => focus = 1,
                    CommanderEvent::RotateCounterClockwise => focus = 0,
                    CommanderEvent::Back => {
                        self.page = SystemPage::Root;
                        return true;
                    }
                    CommanderEvent::Select => {
                        self.page = if focus == 0 {
                            SystemPage::Legal { scroll: 0 }
                        } else {
                            SystemPage::Version
                        };
                        return true;
                    }
                    _ => {}
                }
                self.page = SystemPage::About { focus };
            }
            SystemPage::Legal { mut scroll } => {
                match event {
                    CommanderEvent::RotateClockwise => {
                        scroll = scroll.saturating_add(28).min(max_scroll);
                    }
                    CommanderEvent::RotateCounterClockwise => scroll = scroll.saturating_sub(28),
                    CommanderEvent::Back => {
                        self.page = SystemPage::About { focus: 0 };
                        return true;
                    }
                    _ => {}
                }
                self.page = SystemPage::Legal { scroll };
            }
            SystemPage::Version => {
                if matches!(event, CommanderEvent::Back | CommanderEvent::Select) {
                    self.page = SystemPage::About { focus: 1 };
                }
            }
            SystemPage::Confirm { mut yes } => {
                match event {
                    CommanderEvent::RotateClockwise => yes = true,
                    CommanderEvent::RotateCounterClockwise => yes = false,
                    CommanderEvent::Back => {
                        self.page = SystemPage::Root;
                        return true;
                    }
                    CommanderEvent::Select => {
                        self.page = if yes {
                            SystemPage::Progress { elapsed_ms: 0 }
                        } else {
                            SystemPage::Root
                        };
                        return true;
                    }
                    _ => {}
                }
                self.page = SystemPage::Confirm { yes };
            }
            SystemPage::Progress { .. } => {}
        }
        true
    }
    /// Deterministic mock completion, not OEM reset timing. Caller resets only host preferences.
    pub fn advance(&mut self, elapsed_ms: u64, parked: bool) -> bool {
        self.restrict(parked);
        if let SystemPage::Progress {
            elapsed_ms: previous,
        } = self.page
        {
            let next = previous.saturating_add(elapsed_ms);
            if next >= 1500 {
                self.page = SystemPage::Root;
                return true;
            }
            self.page = SystemPage::Progress { elapsed_ms: next };
        }
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conditional_unit_rows_preserve_na_order_and_independent_preferences() {
        let mut s = SystemSession::default();
        assert_eq!(s.controls().len(), 6);
        for (temperature, distance, count) in [(true, false, 7), (false, true, 7), (true, true, 8)]
        {
            s.equipment.temperature_units = temperature;
            s.equipment.distance_units = distance;
            assert_eq!(s.controls().len(), count);
            assert_eq!(s.controls().last(), Some(&SystemControl::About));
            assert_eq!(
                s.labels(),
                s.controls().iter().map(|c| c.label()).collect::<Vec<_>>()
            );
        }
        s.select(0, true);
        s.select(1, true);
        s.select(3, true);
        s.select(4, true);
        assert_eq!(
            s.values,
            SystemPreferences {
                tool_tips: true,
                vr_examples: true,
                celsius: false,
                kilometers: false
            }
        );
        let before = s.values;
        s.select(0, false);
        assert_eq!(s.values, before);
    }
    #[test]
    fn about_nested_back_routes_and_scroll_are_bounded() {
        let mut s = SystemSession::default();
        s.select(5, true);
        s.active_event(CommanderEvent::Select, true, 100);
        for _ in 0..20 {
            s.active_event(CommanderEvent::RotateClockwise, true, 100);
        }
        assert_eq!(s.page, SystemPage::Legal { scroll: 100 });
        s.active_event(CommanderEvent::Back, true, 100);
        assert_eq!(s.page, SystemPage::About { focus: 0 });
        s.active_event(CommanderEvent::RotateClockwise, true, 100);
        s.active_event(CommanderEvent::Select, true, 100);
        assert_eq!(s.page, SystemPage::Version);
        s.active_event(CommanderEvent::Select, true, 100);
        assert_eq!(s.page, SystemPage::About { focus: 1 });
        s.active_event(CommanderEvent::Back, true, 100);
        assert_eq!(s.page, SystemPage::Root);
    }
    #[test]
    fn reset_defaults_to_no_requires_explicit_yes_and_revokes_before_completion() {
        let mut s = SystemSession::default();
        s.select(4, true);
        assert_eq!(s.page, SystemPage::Confirm { yes: false });
        s.active_event(CommanderEvent::Select, true, 0);
        assert_eq!(s.page, SystemPage::Root);
        s.select(4, true);
        s.active_event(CommanderEvent::RotateClockwise, true, 0);
        s.active_event(CommanderEvent::Select, true, 0);
        assert!(!s.advance(1499, true));
        assert!(s.advance(1, true));
        s.select(4, true);
        s.active_event(CommanderEvent::RotateClockwise, true, 0);
        s.active_event(CommanderEvent::Select, true, 0);
        assert!(!s.advance(2000, false));
        assert_eq!(s.page, SystemPage::Root);
        s.equipment.services_ready = false;
        assert!(s.select(4, true).is_some());
        s.equipment.new_cmu = true;
        assert!(!s.allowed(SystemControl::Distance, true));
        s.equipment.new_cmu = false;
        assert!(s.allowed(SystemControl::Distance, true));
    }
}
