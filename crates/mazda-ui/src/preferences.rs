use mazda_core::SpeedKph;

use crate::measurement_label;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Units {
    #[default]
    Us,
    Metric,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Night,
    Day,
}

/// Session-local preferences. No vehicle configuration or filesystem access.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Preferences {
    pub units: Units,
    pub theme: Theme,
}

impl Preferences {
    #[must_use]
    pub fn speed_label(self, speed: SpeedKph) -> String {
        let (factor, unit) = match self.units {
            Units::Us => (1.0 / 1.609_344, "MPH"),
            Units::Metric => (1.0, "KM/H"),
        };
        match speed.get() {
            Some(value) => measurement_label(value * factor, unit, 10),
            None => format!("-- {unit}"),
        }
    }

    #[must_use]
    pub fn temperature_label(self, celsius: f32) -> String {
        let (value, unit) = match self.units {
            Units::Us => (celsius.mul_add(1.8, 32.0), "F"),
            Units::Metric => (celsius, "C"),
        };
        measurement_label(value, unit, 19)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_convert_real_values_and_keep_missing_values_explicit() {
        let us = Preferences::default();
        assert_eq!(us.speed_label(SpeedKph::new(80.0)), "50 MPH");
        assert_eq!(us.temperature_label(0.0), "32 F");
        assert_eq!(us.temperature_label(-40.0), "-40 F");
        assert_eq!(us.speed_label(SpeedKph::unavailable()), "-- MPH");
        assert_eq!(us.temperature_label(f32::NAN), "-- F");
        let metric = Preferences {
            units: Units::Metric,
            ..us
        };
        assert_eq!(metric.speed_label(SpeedKph::new(80.0)), "80 KM/H");
        assert_eq!(metric.temperature_label(22.0), "22 C");
    }
}
