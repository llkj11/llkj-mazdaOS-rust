//! Source-derived `ClockTab` / `ClockSettings2` / `SelectTimeZone` rendering.
use super::{fitted, ClockPage, Color, Renderer, Screen, UiModel, CLOCK_ROWS};
use crate::{text_width, TimeFormat};

impl UiModel {
    #[must_use]
    pub fn clock_repeat_target(&self) -> Option<usize> {
        if !self.clock_child() {
            return None;
        }
        match self.clock.page {
            ClockPage::Adjust { focus }
                if (1..=4).contains(&focus)
                    && self
                        .clock
                        .button_enabled(focus, self.vehicle.is_confirmed_parked()) =>
            {
                Some(focus)
            }
            _ => None,
        }
    }
    #[must_use]
    pub fn clock_button_at(&self, x: usize, y: usize) -> Option<usize> {
        if !self.clock_child()
            || !matches!(self.clock.page, ClockPage::Adjust { .. })
            || !(380..480).contains(&y)
            || x >= 800
        {
            return None;
        }
        let button = (x / 133).min(5);
        self.clock
            .button_enabled(button, self.vehicle.is_confirmed_parked())
            .then_some(button)
    }
    pub fn focus_clock_button(&mut self, button: usize) {
        if self.clock_child()
            && matches!(self.clock.page, ClockPage::Adjust { .. })
            && self
                .clock
                .button_enabled(button, self.vehicle.is_confirmed_parked())
        {
            self.clock.page = ClockPage::Adjust { focus: button };
        }
    }
    pub(super) fn clock_child(&self) -> bool {
        self.screen == Screen::Settings
            && self.settings_tab == 4
            && self.clock.page != ClockPage::Root
    }
    pub(super) fn clock_format_pointer(&mut self, row: usize, x: usize) -> bool {
        if row != 2 || !(503..739).contains(&x) {
            return false;
        }
        if self.clock.enabled(2, self.vehicle.is_confirmed_parked()) {
            self.clock.values.format = if x < 621 {
                TimeFormat::Hours12
            } else {
                TimeFormat::Hours24
            };
        }
        true
    }
    pub(super) fn clock_pointer(&mut self, x: usize, y: usize) {
        let parked = self.vehicle.is_confirmed_parked();
        match self.clock.page {
            ClockPage::Adjust { .. } => {
                if (380..480).contains(&y) && x < 800 {
                    let button = (x / 133).min(5);
                    if self.clock.button_enabled(button, parked) {
                        self.clock.page = ClockPage::Adjust { focus: button };
                        self.clock.activate_button(button, parked);
                    }
                }
            }
            ClockPage::Zones { focus } => {
                if x < 68 && y >= 64 {
                    self.clock.page = ClockPage::Root;
                    return;
                }
                if (133..480).contains(&y) && (68..800).contains(&x) {
                    let row = focus.saturating_sub(4) + (y - 133) / 64;
                    if row < self.assets.clock_zones.len() {
                        self.clock.page = ClockPage::Zones { focus: row };
                        self.clock.active_event(
                            mazda_core::CommanderEvent::Select,
                            parked,
                            self.assets.clock_zones.len(),
                        );
                    }
                }
            }
            ClockPage::Root => {}
        }
    }
    pub(super) fn render_clock_root(&self, renderer: &mut impl Renderer) {
        for (row, label) in CLOCK_ROWS.iter().enumerate() {
            let y = 133 + row * 64;
            if self.row == row && self.settings_rows {
                self.image(
                    renderer,
                    "list-focus",
                    (68, i32::try_from(y).expect("row"), 732, 64),
                );
            }
            self.image(
                renderer,
                "list-separator",
                (68, i32::try_from(y + 62).expect("row"), 732, 2),
            );
            let color = if self.clock.enabled(row, self.vehicle.is_confirmed_parked()) {
                Color::TEXT
            } else {
                Color::rgb(146, 148, 151)
            };
            fitted(
                renderer,
                139,
                y + 14,
                self.tr("syssettings", label),
                if row == 2 {
                    364
                } else if row == 3 {
                    456
                } else {
                    542
                },
                28,
                color,
            );
            match row {
                1 | 4 => {
                    let checked = if row == 1 {
                        self.clock.values.gps_sync
                    } else {
                        self.clock.values.daylight_saving
                    };
                    let allowed = self.clock.enabled(row, self.vehicle.is_confirmed_parked());
                    let asset = match (allowed, checked) {
                        (true, true) => "settings-check",
                        (true, false) => "settings-uncheck",
                        (false, true) => "settings-check-disabled",
                        (false, false) => "settings-uncheck-disabled",
                    };
                    self.image_natural(renderer, asset, 681, i32::try_from(y + 6).expect("row"));
                }
                2 => {
                    for (index, label) in ["12h", "24h"].iter().enumerate() {
                        let label = self.tr("syssettings", label);
                        let x = 503 + index * 118;
                        let selected =
                            (index == 1) == (self.clock.values.format == TimeFormat::Hours24);
                        self.image_natural(
                            renderer,
                            if selected {
                                "settings-inline-selected"
                            } else {
                                "settings-inline"
                            },
                            i32::try_from(x - 1).expect("segment"),
                            i32::try_from(y + 9).expect("row"),
                        );
                        renderer.text(
                            x + (118 - text_width(label, 28)) / 2,
                            y + 16,
                            label,
                            28,
                            color,
                        );
                    }
                }
                3 => {
                    let label = self
                        .assets
                        .clock_zones
                        .get(self.clock.values.time_zone)
                        .map_or("---", String::as_str);
                    let text =
                        crate::typography::ellipsized(self.tr("syssettings", label), 129, 28);
                    renderer.text(724 - text_width(&text, 28), y + 14, &text, 28, color);
                }
                _ => {}
            }
        }
    }
    pub(super) fn render_clock_child(&self, renderer: &mut impl Renderer) {
        match self.clock.page {
            ClockPage::Adjust { focus } => self.render_clock_adjust(renderer, focus),
            ClockPage::Zones { focus } => self.render_clock_zones(renderer, focus),
            ClockPage::Root => {}
        }
    }
    fn render_clock_adjust(&self, renderer: &mut impl Renderer, focus: usize) {
        self.image_natural(renderer, "clock-background", 10, 64);
        renderer.text(
            90,
            75,
            self.tr("syssettings", "Adjust Time"),
            40,
            Color::TEXT,
        );
        let hour = self.clock.values.display_hour();
        let minute = self.clock.values.minute;
        for (x, digit) in [
            (174, hour / 10),
            (278, hour % 10),
            (440, minute / 10),
            (544, minute % 10),
        ] {
            let label = digit.to_string();
            renderer.text(
                x + (102 - text_width(&label, 150)) / 2,
                181,
                &label,
                150,
                Color::TEXT,
            );
        }
        self.image_natural(renderer, "clock-colon", 401, 241);
        if self.clock.values.format == TimeFormat::Hours12 {
            let label = if self.clock.values.hour < 12 {
                "AM"
            } else {
                "PM"
            };
            renderer.text(
                667 + (115 - text_width(label, 48)) / 2,
                270,
                label,
                48,
                Color::TEXT,
            );
        }
        self.image_natural(renderer, "clock-arch", 0, 374);
        self.image_natural(renderer, "clock-panel", 0, 382);
        for button in 0..6 {
            let x = button * 133;
            if button > 0 {
                self.image_natural(
                    renderer,
                    "clock-separator",
                    i32::try_from(x - 1).expect("button"),
                    396,
                );
            }
            if button == focus {
                self.image_natural(
                    renderer,
                    "clock-focus",
                    i32::try_from(x).expect("button"),
                    380,
                );
            }
            let enabled = self
                .clock
                .button_enabled(button, self.vehicle.is_confirmed_parked());
            let asset = match (button, enabled) {
                (0, _) => "clock-back",
                (1 | 3, true) => "clock-minus",
                (1 | 3, false) => "clock-minus-disabled",
                (2 | 4, true) => "clock-plus",
                (2 | 4, false) => "clock-plus-disabled",
                (_, true) => "clock-ampm",
                (_, false) => "clock-ampm-disabled",
            };
            if let Some(image) = self.assets.get(asset) {
                let (w, h) = image.size();
                renderer.image(
                    i32::try_from(x + 135_usize.saturating_sub(w) / 2).expect("icon"),
                    380 + i32::try_from(100_usize.saturating_sub(h) / 2).expect("icon"),
                    w,
                    h,
                    image,
                );
            }
        }
    }
    fn render_clock_zones(&self, renderer: &mut impl Renderer, focus: usize) {
        self.image(renderer, "list-background", (68, 64, 732, 416));
        self.image_natural(renderer, "settings-title", 69, 64);
        renderer.text(
            100,
            75,
            self.tr("syssettings", "Select Time Zone"),
            40,
            Color::TEXT,
        );
        let allowed = self.clock.enabled(3, self.vehicle.is_confirmed_parked());
        let color = if allowed {
            Color::TEXT
        } else {
            Color::rgb(146, 148, 151)
        };
        for (visible, (index, label)) in self
            .assets
            .clock_zones
            .iter()
            .enumerate()
            .skip(focus.saturating_sub(4))
            .take(6)
            .enumerate()
        {
            let y = 133 + visible * 64;
            if index == focus {
                self.image(
                    renderer,
                    "list-focus",
                    (68, i32::try_from(y).expect("row"), 732, 64),
                );
            }
            self.image(
                renderer,
                "list-separator",
                (68, i32::try_from(y + 62).expect("row"), 732, 2),
            );
            let asset = match (allowed, index == self.clock.values.time_zone) {
                (true, true) => "settings-radio",
                (true, false) => "settings-radio-empty",
                (false, true) => "settings-radio-disabled",
                (false, false) => "settings-radio-empty-disabled",
            };
            self.image_natural(renderer, asset, 95, i32::try_from(y + 14).expect("row"));
            let label = crate::typography::ellipsized(self.tr("syssettings", label), 489, 28);
            renderer.text(139, y + 14, &label, 28, color);
        }
    }
}
