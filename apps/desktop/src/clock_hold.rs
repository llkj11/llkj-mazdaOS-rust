//! Local input timing from `ButtonCtrl`'s 1500ms hold and `ClockSettings2`'s 250ms tick.
use minifb::Key;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub enum Source {
    Key(Key),
    Pointer,
}
pub struct ClockHold {
    pub source: Source,
    pub button: usize,
    started: Instant,
    delivered: u32,
}
impl ClockHold {
    pub fn new(button: usize, source: Source, now: Instant) -> Self {
        Self {
            source,
            button,
            started: now,
            delivered: 0,
        }
    }
    pub fn poll(
        &mut self,
        target: Option<usize>,
        down: bool,
        inside: bool,
        now: Instant,
    ) -> (u32, bool) {
        if target != Some(self.button) || !inside {
            return (0, true);
        }
        let elapsed = now.saturating_duration_since(self.started);
        if !down {
            return (u32::from(elapsed < Duration::from_millis(1500)), true);
        }
        let due = repeat_count(elapsed);
        let changes = due.saturating_sub(self.delivered).min(8);
        self.delivered = due;
        (changes, false)
    }
}
fn repeat_count(elapsed: Duration) -> u32 {
    if elapsed < Duration::from_millis(1750) {
        return 0;
    }
    u32::try_from((elapsed.as_millis() - 1750) / 250 + 1).unwrap_or(u32::MAX)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn short_release_and_original_hold_cadence_are_distinct() {
        let start = Instant::now();
        let mut short = ClockHold::new(2, Source::Key(Key::Enter), start);
        assert_eq!(
            short.poll(Some(2), true, true, start + Duration::from_millis(1000)),
            (0, false)
        );
        assert_eq!(
            short.poll(Some(2), false, true, start + Duration::from_millis(1001)),
            (1, true)
        );
        let mut held = ClockHold::new(2, Source::Pointer, start);
        for (ms, count) in [
            (1500, 0),
            (1749, 0),
            (1750, 1),
            (1900, 0),
            (2000, 1),
            (2250, 1),
        ] {
            assert_eq!(
                held.poll(Some(2), true, true, start + Duration::from_millis(ms)),
                (count, false)
            );
        }
        assert_eq!(
            held.poll(Some(2), false, true, start + Duration::from_millis(2300)),
            (0, true)
        );
    }
    #[test]
    fn focus_loss_pointer_departure_and_long_stalls_cannot_redirect_or_flood_edits() {
        let start = Instant::now();
        let mut hold = ClockHold::new(1, Source::Pointer, start);
        assert_eq!(
            hold.poll(Some(2), true, true, start + Duration::from_secs(2)),
            (0, true)
        );
        assert_eq!(
            hold.poll(Some(1), false, false, start + Duration::from_millis(10)),
            (0, true)
        );
        assert_eq!(
            hold.poll(None, true, true, start + Duration::from_secs(2)),
            (0, true)
        );
        assert_eq!(
            hold.poll(Some(1), true, true, start + Duration::from_secs(1000)),
            (8, false)
        );
        assert_eq!(
            hold.poll(Some(1), true, true, start + Duration::from_secs(1000)),
            (0, false)
        );
    }
}
