//! NA 70.00.100A audiosettings `SoundTab`, using host-session values only.
use mazda_core::CommanderEvent;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SoundProfile {
    #[default]
    Standard,
    Bose,
    BoseJ12,
}
impl SoundProfile {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Bose => "Bose",
            Self::BoseJ12 => "Bose J12",
        }
    }
    #[must_use]
    pub const fn controls(self) -> &'static [SoundControl] {
        use SoundControl::{
            Alc, AudioPilot, Balance, Bass, Beep, Centerpoint, Fade, Guidance, Treble,
        };
        match self {
            Self::Standard => &[Bass, Treble, Fade, Balance, Alc, Guidance, Beep],
            Self::Bose => &[
                Bass,
                Treble,
                Fade,
                Balance,
                Guidance,
                Centerpoint,
                AudioPilot,
                Beep,
            ],
            Self::BoseJ12 => &[Bass, Treble, Fade, Balance, Guidance, AudioPilot, Beep],
        }
    }
    pub(crate) const fn labels(self) -> &'static [&'static str] {
        match self {
            Self::Standard => &[
                "Bass",
                "Treble",
                "Fade",
                "Balance",
                "ALC",
                "Guidance Volume",
                "Beep",
            ],
            Self::Bose => &[
                "Bass",
                "Treble",
                "Fade",
                "Balance",
                "Guidance Volume",
                "Bose® Centerpoint",
                "Bose® AudioPilot",
                "Beep",
            ],
            Self::BoseJ12 => &[
                "Bass",
                "Treble",
                "Fade",
                "Balance",
                "Guidance Volume",
                "Bose® AudioPilot",
                "Beep",
            ],
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundControl {
    Bass,
    Treble,
    Fade,
    Balance,
    Alc,
    Guidance,
    Centerpoint,
    AudioPilot,
    Beep,
}
impl SoundControl {
    #[must_use]
    pub const fn range(self) -> Option<(i16, i16)> {
        match self {
            Self::Bass | Self::Treble => Some((-6, 6)),
            Self::Fade | Self::Balance => Some((-8, 8)),
            Self::Alc => Some((0, 7)),
            Self::Guidance => Some((0, 63)),
            _ => None,
        }
    }
    pub(crate) const fn pivot(self) -> bool {
        matches!(self, Self::Bass | Self::Treble | Self::Fade | Self::Balance)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundPreferences {
    pub bass: i8,
    pub treble: i8,
    pub fade: i8,
    pub balance: i8,
    pub alc: u8,
    pub guidance: u8,
    pub centerpoint: bool,
    pub audio_pilot: bool,
    pub beep: bool,
}
impl Default for SoundPreferences {
    fn default() -> Self {
        Self {
            bass: 0,
            treble: 0,
            fade: 0,
            balance: 0,
            alc: 0,
            guidance: 30,
            centerpoint: false,
            audio_pilot: false,
            beep: false,
        }
    }
}
impl SoundPreferences {
    #[must_use]
    pub fn value(self, control: SoundControl) -> i16 {
        match control {
            SoundControl::Bass => i16::from(self.bass),
            SoundControl::Treble => i16::from(self.treble),
            SoundControl::Fade => i16::from(self.fade),
            SoundControl::Balance => i16::from(self.balance),
            SoundControl::Alc => i16::from(self.alc),
            SoundControl::Guidance => i16::from(self.guidance),
            SoundControl::Centerpoint => i16::from(self.centerpoint),
            SoundControl::AudioPilot => i16::from(self.audio_pilot),
            SoundControl::Beep => i16::from(self.beep),
        }
    }
    fn set(&mut self, control: SoundControl, value: i16) {
        let value = control
            .range()
            .map_or(value.clamp(0, 1), |(min, max)| value.clamp(min, max));
        match control {
            SoundControl::Bass => self.bass = i8::try_from(value).expect("bounded tone value"),
            SoundControl::Treble => self.treble = i8::try_from(value).expect("bounded tone value"),
            SoundControl::Fade => self.fade = i8::try_from(value).expect("bounded pivot value"),
            SoundControl::Balance => {
                self.balance = i8::try_from(value).expect("bounded pivot value");
            }
            SoundControl::Alc => self.alc = u8::try_from(value).expect("bounded ALC"),
            SoundControl::Guidance => {
                self.guidance = u8::try_from(value).expect("bounded guidance");
            }
            SoundControl::Centerpoint => self.centerpoint = value == 1,
            SoundControl::AudioPilot => self.audio_pilot = value == 1,
            SoundControl::Beep => self.beep = value == 1,
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct SoundSession {
    pub profile: SoundProfile,
    pub values: SoundPreferences,
    pub editing: Option<SoundControl>,
    pub pointer_drag: bool,
    pub guidance_enabled: bool,
}
impl Default for SoundSession {
    fn default() -> Self {
        Self {
            profile: SoundProfile::default(),
            values: SoundPreferences::default(),
            editing: None,
            pointer_drag: false,
            guidance_enabled: true,
        }
    }
}
impl SoundSession {
    pub fn cancel_interaction(&mut self) {
        self.editing = None;
        self.pointer_drag = false;
    }
    pub fn allowed(&self, control: SoundControl, parked: bool) -> bool {
        parked
            && (control != SoundControl::Guidance || self.guidance_enabled)
            && self.profile.controls().contains(&control)
    }
    pub fn select(&mut self, row: usize, parked: bool) -> Option<&'static str> {
        let control = *self.profile.controls().get(row)?;
        if !parked {
            self.cancel_interaction();
            return Some("Preferences locked: valid zero speed and Park required");
        }
        if !self.allowed(control, parked) {
            self.cancel_interaction();
            return Some("Guidance Volume is disabled by the host service fixture");
        }
        if control.range().is_some() {
            self.editing = Some(control);
        } else {
            self.values.set(control, 1 - self.values.value(control));
        }
        None
    }
    pub fn active_event(&mut self, event: CommanderEvent, parked: bool) -> bool {
        let Some(control) = self.editing else {
            return false;
        };
        if !self.allowed(control, parked) {
            self.cancel_interaction();
            return false;
        }
        match event {
            CommanderEvent::RotateClockwise => {
                self.values.set(control, self.values.value(control) + 1);
            }
            CommanderEvent::RotateCounterClockwise => {
                self.values.set(control, self.values.value(control) - 1);
            }
            CommanderEvent::Select | CommanderEvent::Back => self.cancel_interaction(),
            _ => {}
        }
        true
    }
    pub fn pointer_value(&mut self, control: SoundControl, x: usize, parked: bool) {
        if !self.allowed(control, parked) {
            self.cancel_interaction();
            return;
        }
        let Some((min, max)) = control.range() else {
            return;
        };
        let position = x.saturating_sub(389).min(346);
        let span = usize::try_from(max - min).expect("positive slider range");
        self.values.set(
            control,
            min + i16::try_from((position * span + 173) / 346).expect("bounded slider"),
        );
        self.editing = Some(control);
        self.pointer_drag = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_profiles_keep_order_ranges_and_only_supported_features() {
        assert_eq!(SoundProfile::Standard.controls().len(), 7);
        assert_eq!(SoundProfile::Bose.controls().len(), 8);
        assert_eq!(SoundProfile::BoseJ12.controls().len(), 7);
        assert_eq!(SoundProfile::Bose.controls()[4], SoundControl::Guidance);
        assert!(!SoundProfile::Standard
            .controls()
            .contains(&SoundControl::AudioPilot));
        assert!(!SoundProfile::Bose.controls().contains(&SoundControl::Alc));
        assert!(!SoundProfile::BoseJ12
            .controls()
            .contains(&SoundControl::Centerpoint));
        for profile in [
            SoundProfile::Standard,
            SoundProfile::Bose,
            SoundProfile::BoseJ12,
        ] {
            assert_eq!(profile.labels().len(), profile.controls().len());
        }
    }
    #[test]
    fn all_slider_types_clamp_and_pivot_reaches_zero() {
        for row in 0..6 {
            let mut state = SoundSession::default();
            let control = state.profile.controls()[row];
            let (min, max) = control.range().unwrap();
            state.select(row, true);
            for _ in 0..100 {
                assert!(state.active_event(CommanderEvent::RotateClockwise, true));
            }
            assert_eq!(state.values.value(control), max);
            for _ in 0..100 {
                state.active_event(CommanderEvent::RotateCounterClockwise, true);
            }
            assert_eq!(state.values.value(control), min);
            state.active_event(CommanderEvent::Back, true);
            assert_eq!(state.editing, None);
            state.pointer_value(control, usize::MAX, true);
            assert_eq!(state.values.value(control), max);
            state.pointer_value(control, 0, true);
            assert_eq!(state.values.value(control), min);
            if control.pivot() {
                state.pointer_value(control, 562, true);
                assert_eq!(state.values.value(control), 0);
            }
        }
    }
    #[test]
    fn toggle_values_and_restriction_revocation_are_session_local() {
        let mut state = SoundSession {
            profile: SoundProfile::Bose,
            ..SoundSession::default()
        };
        for row in 5..8 {
            state.select(row, true);
            assert_eq!(state.values.value(state.profile.controls()[row]), 1);
            state.select(row, true);
            assert_eq!(state.values.value(state.profile.controls()[row]), 0);
        }
        state.select(0, true);
        let before = state.values;
        state.active_event(CommanderEvent::RotateClockwise, false);
        assert_eq!(state.values, before);
        assert_eq!(state.editing, None);
        assert!(state.select(5, false).is_some());
        assert!(!state.values.centerpoint);
    }
    #[test]
    fn guidance_service_flag_blocks_keyboard_and_pointer_adjustment() {
        let mut state = SoundSession {
            guidance_enabled: false,
            ..SoundSession::default()
        };
        assert!(state.select(5, true).is_some());
        state.pointer_value(SoundControl::Guidance, usize::MAX, true);
        assert_eq!(state.values.guidance, 30);
        assert_eq!(state.editing, None);
        state.guidance_enabled = true;
        state.select(5, true);
        state.guidance_enabled = false;
        state.active_event(CommanderEvent::RotateClockwise, true);
        assert_eq!(state.values.guidance, 30);
        assert_eq!(state.editing, None);
    }
}
