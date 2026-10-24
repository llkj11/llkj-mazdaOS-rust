use std::collections::VecDeque;

use mazda_core::{
    CommanderEvent, Gear, MazdaReadOnly, MediaState, PlaybackState, SpeedKph, TemperatureC,
    VehicleSnapshot,
};

#[derive(Debug, Clone)]
pub struct MockMazda {
    vehicle: VehicleSnapshot,
    media: MediaState,
    commander_events: VecDeque<CommanderEvent>,
    track_index: usize,
}

/// Host fixtures only. These never write to any Mazda transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoScenario {
    Parked,
    Driving,
    Unavailable,
    Reverse,
}

const TRACKS: [(&str, &str, &str); 3] = [
    ("Everything in Its Right Place", "Radiohead", "Kid A"),
    ("A Walk", "Tycho", "Dive"),
    (
        "Your Hand in Mine",
        "Explosions in the Sky",
        "The Earth Is Not a Cold Dead Place",
    ),
];

impl MockMazda {
    #[must_use]
    pub fn demo() -> Self {
        Self {
            track_index: 0,
            vehicle: VehicleSnapshot {
                speed: SpeedKph::new(0.0),
                gear: Gear::Park,
                outside_temperature: TemperatureC::new(22.0),
            },
            media: MediaState {
                title: Some("Everything in Its Right Place".to_owned()),
                artist: Some("Radiohead".to_owned()),
                album: Some("Kid A".to_owned()),
                playback: PlaybackState::Playing,
            },
            commander_events: [
                CommanderEvent::RotateClockwise,
                CommanderEvent::RotateClockwise,
                CommanderEvent::Select,
                CommanderEvent::Home,
            ]
            .into(),
        }
    }

    /// Mutation exists only on the mock fixture so tests and simulators can create scenarios.
    pub fn set_vehicle_snapshot(&mut self, vehicle: VehicleSnapshot) {
        self.vehicle = vehicle;
    }

    /// Mutation exists only on the mock fixture so tests and simulators can create scenarios.
    pub fn set_media_state(&mut self, media: MediaState) {
        self.media = media;
    }

    pub fn push_commander_event(&mut self, event: CommanderEvent) {
        self.commander_events.push_back(event);
    }

    /// Selects synthetic observations; it does not connect to a vehicle.
    pub fn set_scenario(&mut self, scenario: DemoScenario) {
        let (speed, gear, temperature) = match scenario {
            DemoScenario::Parked => (SpeedKph::new(0.0), Gear::Park, 22.0),
            DemoScenario::Driving => (SpeedKph::new(80.0), Gear::Drive, 22.0),
            DemoScenario::Unavailable => (SpeedKph::unavailable(), Gear::Unknown, f32::NAN),
            DemoScenario::Reverse => (SpeedKph::new(0.0), Gear::Reverse, 22.0),
        };
        self.vehicle = VehicleSnapshot {
            speed,
            gear,
            outside_temperature: TemperatureC::new(temperature),
        };
    }

    pub fn toggle_playback(&mut self) {
        self.media.playback = match self.media.playback {
            PlaybackState::Playing => PlaybackState::Paused,
            PlaybackState::Paused | PlaybackState::Stopped => PlaybackState::Playing,
        };
    }

    pub fn next_track(&mut self) {
        self.track_index = (self.track_index + 1) % TRACKS.len();
        self.load_track();
    }

    pub fn previous_track(&mut self) {
        self.track_index = (self.track_index + TRACKS.len() - 1) % TRACKS.len();
        self.load_track();
    }

    fn load_track(&mut self) {
        let (title, artist, album) = TRACKS[self.track_index];
        self.media = MediaState {
            title: Some(title.to_owned()),
            artist: Some(artist.to_owned()),
            album: Some(album.to_owned()),
            playback: PlaybackState::Playing,
        };
    }
}

impl MazdaReadOnly for MockMazda {
    fn vehicle_snapshot(&self) -> VehicleSnapshot {
        self.vehicle.clone()
    }

    fn media_state(&self) -> MediaState {
        self.media.clone()
    }

    fn next_commander_event(&mut self) -> Option<CommanderEvent> {
        self.commander_events.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_controls_update_the_read_only_observation_and_wrap() {
        let mut mazda = MockMazda::demo();
        let original = mazda.media_state();
        mazda.toggle_playback();
        assert_eq!(mazda.media_state().playback, PlaybackState::Paused);
        mazda.toggle_playback();
        assert_eq!(mazda.media_state(), original);
        mazda.previous_track();
        assert_eq!(
            mazda.media_state().title.as_deref(),
            Some("Your Hand in Mine")
        );
        mazda.next_track();
        assert_eq!(mazda.media_state(), original);
        mazda.next_track();
        assert_eq!(mazda.media_state().title.as_deref(), Some("A Walk"));
        assert!(mazda.vehicle_snapshot().is_confirmed_parked());
    }

    #[test]
    fn scenario_changes_preserve_media_and_unavailable_is_not_zero() {
        let mut mazda = MockMazda::demo();
        let media = mazda.media_state();
        for scenario in [
            DemoScenario::Driving,
            DemoScenario::Unavailable,
            DemoScenario::Reverse,
        ] {
            mazda.set_scenario(scenario);
            assert!(!mazda.vehicle_snapshot().is_confirmed_parked());
            assert_eq!(mazda.media_state(), media);
        }
        mazda.set_scenario(DemoScenario::Unavailable);
        assert_eq!(mazda.vehicle_snapshot().speed.get(), None);
        assert!(mazda.vehicle_snapshot().outside_temperature.get().is_nan());
        mazda.set_scenario(DemoScenario::Parked);
        assert!(mazda.vehicle_snapshot().is_confirmed_parked());
    }

    #[test]
    fn demo_scenario_is_stationary_and_playing_media() {
        let mazda = MockMazda::demo();
        let vehicle = mazda.vehicle_snapshot();
        let media = mazda.media_state();

        assert_eq!(vehicle.gear, Gear::Park);
        assert_eq!(vehicle.speed.get(), Some(0.0));
        assert_eq!(media.playback, PlaybackState::Playing);
    }

    #[test]
    fn commander_events_are_deterministic() {
        let mut mazda = MockMazda::demo();

        assert_eq!(
            mazda.next_commander_event(),
            Some(CommanderEvent::RotateClockwise)
        );
        assert_eq!(
            mazda.next_commander_event(),
            Some(CommanderEvent::RotateClockwise)
        );
        assert_eq!(mazda.next_commander_event(), Some(CommanderEvent::Select));
        assert_eq!(mazda.next_commander_event(), Some(CommanderEvent::Home));
        assert_eq!(mazda.next_commander_event(), None);
    }
}
