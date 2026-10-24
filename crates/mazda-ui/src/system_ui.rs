//! Source-guided `SystemTab`, About, disclosure, version and reset rendering/routes.
use super::{centered_lines, fitted, Color, Renderer, Screen, SystemPage, UiModel};
use crate::{text_width, Rect, SystemControl, SystemEquipment, SystemPreferences};

impl UiModel {
    #[must_use]
    pub const fn system_preferences(&self) -> SystemPreferences {
        self.system.values
    }
    pub fn set_system_equipment(&mut self, equipment: SystemEquipment) {
        self.system.equipment = equipment;
        self.system.page = SystemPage::Root;
        self.language.page = super::LanguagePage::Closed;
        self.row = self.row.min(self.rows().len().saturating_sub(1));
    }
    /// Advance deterministic host operations only; no real clock/settings transport.
    pub fn advance_simulation(&mut self, elapsed: std::time::Duration) {
        let elapsed_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);
        self.language
            .advance(elapsed_ms, self.vehicle.is_confirmed_parked());
        if self
            .system
            .advance(elapsed_ms, self.vehicle.is_confirmed_parked())
        {
            self.display = crate::settings::DisplaySession::default();
            self.sound.values = crate::SoundPreferences::default();
            self.sound.cancel_interaction();
            self.clock.values = crate::ClockPreferences::default();
            if !self.assets.clock_zones.is_empty() {
                self.clock.values.time_zone = self
                    .clock
                    .values
                    .time_zone
                    .min(self.assets.clock_zones.len() - 1);
            }
            self.clock.page = super::ClockPage::Root;
            self.system.values = SystemPreferences::default();
            self.language.current = 0;
            self.language.page = super::LanguagePage::Closed;
        }
    }
    pub(super) fn system_child(&self) -> bool {
        self.screen == Screen::Settings
            && self.settings_tab == 7
            && (self.system.page != SystemPage::Root
                || self.language.page != super::LanguagePage::Closed)
    }
    fn legal_title_lines(&self) -> Vec<String> {
        crate::typography::wrapped_lines(
            self.tr("syssettings", "Agreements and Disclaimers"),
            650,
            40,
        )
        .into_iter()
        .take(2)
        .collect()
    }
    fn legal_header_height(&self) -> usize {
        self.legal_title_lines().len() * 48 + 15
    }
    pub(super) fn legal_max_scroll(&self) -> usize {
        (self.legal_header_height() + 25 + self.assets.legal_lines.len() * 34 + 28_usize)
            .saturating_sub(416)
    }
    pub(super) fn system_units_pointer(&mut self, row: usize, x: usize) -> bool {
        if !(503..739).contains(&x) {
            return false;
        }
        let Some(&control) = self.system.controls().get(row) else {
            return false;
        };
        if !matches!(
            control,
            SystemControl::Temperature | SystemControl::Distance
        ) {
            return false;
        }
        if self
            .system
            .allowed(control, self.vehicle.is_confirmed_parked())
        {
            if control == SystemControl::Temperature {
                self.system.values.celsius = x >= 621;
            } else {
                self.system.values.kilometers = x >= 621;
            }
        }
        true
    }
    pub(super) fn system_pointer(&mut self, x: usize, y: usize) {
        if x >= 800 || y >= 480 {
            return;
        }
        let event = mazda_core::CommanderEvent::Select;
        match self.system.page {
            SystemPage::About { .. } if (133..261).contains(&y) && x >= 68 => {
                self.system.page = SystemPage::About {
                    focus: (y - 133) / 64,
                };
                self.system.active_event(
                    event,
                    self.vehicle.is_confirmed_parked(),
                    self.legal_max_scroll(),
                );
            }
            SystemPage::Confirm { .. } if (69..731).contains(&x) && y >= 382 => {
                self.system.page = SystemPage::Confirm { yes: x >= 401 };
                self.system.active_event(
                    event,
                    self.vehicle.is_confirmed_parked(),
                    self.legal_max_scroll(),
                );
            }
            SystemPage::Version if (69..731).contains(&x) && y >= 382 => {
                self.system.page = SystemPage::About { focus: 1 };
            }
            SystemPage::Legal { scroll } if x >= 746 && y >= 64 => {
                let next = if y < 272 {
                    scroll.saturating_sub(28)
                } else {
                    scroll.saturating_add(28).min(self.legal_max_scroll())
                };
                self.system.page = SystemPage::Legal { scroll: next };
            }
            _ => {}
        }
        if x < 60 && y >= 64 && !matches!(self.system.page, SystemPage::Progress { .. }) {
            self.system.active_event(
                mazda_core::CommanderEvent::Back,
                self.vehicle.is_confirmed_parked(),
                self.legal_max_scroll(),
            );
        }
    }
    fn system_row_state(&self, control: SystemControl) -> (bool, Color) {
        let allowed = self
            .system
            .allowed(control, self.vehicle.is_confirmed_parked());
        (
            allowed,
            if allowed {
                Color::TEXT
            } else {
                Color::rgb(146, 148, 151)
            },
        )
    }
    pub(super) fn render_system_root(&self, r: &mut impl Renderer) {
        for (visible, (row, &control)) in self
            .system
            .controls()
            .iter()
            .enumerate()
            .skip(self.first_visible_row())
            .take(6)
            .enumerate()
        {
            let y = 133 + visible * 64;
            if row == self.row && self.settings_rows {
                self.image(
                    r,
                    "list-focus",
                    (68, i32::try_from(y).expect("row"), 732, 64),
                );
            }
            self.image(
                r,
                "list-separator",
                (68, i32::try_from(y + 62).expect("row"), 732, 2),
            );
            let (allowed, color) = self.system_row_state(control);
            let width = if matches!(
                control,
                SystemControl::Temperature | SystemControl::Distance
            ) {
                364
            } else if control == SystemControl::Language {
                456
            } else {
                542
            };
            fitted(
                r,
                139,
                y + 14,
                self.tr("syssettings", control.label()),
                width,
                28,
                color,
            );
            match control {
                SystemControl::ToolTips | SystemControl::VrExamples => {
                    let checked = if control == SystemControl::ToolTips {
                        self.system.values.tool_tips
                    } else {
                        self.system.values.vr_examples
                    };
                    let asset = match (allowed, checked) {
                        (true, true) => "settings-check",
                        (true, false) => "settings-uncheck",
                        (false, true) => "settings-check-disabled",
                        (false, false) => "settings-uncheck-disabled",
                    };
                    self.image_natural(r, asset, 681, i32::try_from(y + 6).expect("row"));
                }
                SystemControl::Temperature | SystemControl::Distance => {
                    let (labels, second) = if control == SystemControl::Temperature {
                        (["°F", "°C"], self.system.values.celsius)
                    } else {
                        (["mi", "km"], self.system.values.kilometers)
                    };
                    for (index, label) in labels.iter().enumerate() {
                        let x = 503 + index * 118;
                        self.image_natural(
                            r,
                            if (index == 1) == second {
                                "settings-inline-selected"
                            } else {
                                "settings-inline"
                            },
                            i32::try_from(x - 1).expect("segment"),
                            i32::try_from(y + 9).expect("row"),
                        );
                        r.text(
                            x + (118 - text_width(label, 28)) / 2,
                            y + 16,
                            label,
                            28,
                            color,
                        );
                    }
                }
                SystemControl::Language => {
                    let label = crate::typography::ellipsized(
                        self.language_label(self.language.current),
                        129,
                        28,
                    );
                    r.text(724 - text_width(&label, 28), y + 14, &label, 28, color);
                }
                _ => {}
            }
        }
    }
    pub(super) fn render_system_child(&self, r: &mut impl Renderer) {
        if self.language.page != super::LanguagePage::Closed {
            self.render_language(r);
            return;
        }
        match self.system.page {
            SystemPage::About { focus } => {
                self.image(r, "list-background", (68, 64, 732, 416));
                self.image_natural(r, "settings-title", 69, 64);
                r.text(100, 75, self.tr("syssettings", "About"), 40, Color::TEXT);
                for (row, label) in ["Agreements and Disclaimers", "Version Information"]
                    .iter()
                    .enumerate()
                {
                    let y = 133 + row * 64;
                    if focus == row {
                        self.image(
                            r,
                            "list-focus",
                            (68, i32::try_from(y).expect("row"), 732, 64),
                        );
                    }
                    self.image(
                        r,
                        "list-separator",
                        (68, i32::try_from(y + 62).expect("row"), 732, 2),
                    );
                    fitted(
                        r,
                        139,
                        y + 14,
                        self.tr("syssettings", label),
                        542,
                        28,
                        Color::TEXT,
                    );
                }
            }
            SystemPage::Legal { scroll } => self.render_legal(r, scroll),
            SystemPage::Version => self.render_version(r),
            SystemPage::Confirm { yes } => {
                self.image(r, "settings-dialog-bg", (0, 0, 800, 480));
                centered_lines(
                    r,
                    self.tr(
                        "syssettings",
                        "Are you sure you want to restore all settings to the factory defaults?",
                    ),
                    77,
                    150,
                    646,
                    35,
                    42,
                );
                for (index, label) in ["No", "Yes"].iter().enumerate() {
                    let label = self.tr("common", label);
                    let x = 69 + index * 332;
                    self.image_natural(
                        r,
                        if (index == 1) == yes {
                            "settings-dialog-button-focus"
                        } else {
                            "settings-dialog-button"
                        },
                        i32::try_from(x).expect("button"),
                        382,
                    );
                    r.text(
                        x + (330 - text_width(label, 35)) / 2,
                        400,
                        label,
                        35,
                        Color::TEXT,
                    );
                }
            }
            SystemPage::Progress { .. } => {
                self.image(r, "settings-dialog-bg", (0, 0, 800, 480));
                centered_lines(r, "Restoring all settings", 79, 150, 646, 35, 38);
                self.image_natural(r, "system-meter", 344, 250);
            }
            SystemPage::Root => {}
        }
    }
    fn render_version(&self, r: &mut impl Renderer) {
        self.image_natural(r, "system-dialog-bg", 0, 64);
        for (text, y) in [
            ("OS Version Not Available", 150),
            ("Music Database Version Not Available", 242),
            ("Fail-Safe Version Not Available", 336),
        ] {
            if y == 336 {
                let label = crate::typography::ellipsized(self.tr("sysupdate", text), 646, 35);
                r.text(
                    77 + (646 - text_width(&label, 35)) / 2,
                    y,
                    &label,
                    35,
                    Color::TEXT,
                );
            } else {
                centered_lines(r, self.tr("sysupdate", text), 77, y, 646, 35, 38);
            }
        }
        self.image_natural(r, "system-dialog-ok", 69, 382);
        let label = self.loc("common", "Ok", "OK");
        r.text(
            (800 - text_width(label, 35)) / 2,
            400,
            label,
            35,
            Color::TEXT,
        );
    }
    fn render_legal(&self, r: &mut impl Renderer, scroll: usize) {
        self.image(r, "legal-background", (60, 64, 740, 416));
        self.image(r, "legal-focus", (60, 64, 740, 416));
        let offset = i32::try_from(scroll.min(self.legal_max_scroll())).expect("bounded text");
        let header = self.legal_header_height();
        if offset < i32::try_from(header).expect("title") {
            self.image(r, "legal-header", (60, 64 - offset, 730, header));
        }
        for (row, line) in self.legal_title_lines().iter().enumerate() {
            r.text_clipped(
                (90, 72 + i32::try_from(row * 48).expect("title") - offset),
                line,
                40,
                Color::TEXT,
                Rect::new(60, 64, 740, 416),
            );
        }
        for (row, text) in self.assets.legal_lines.iter().enumerate() {
            let y = i32::try_from(64 + header + 25 + row * 34).expect("bounded text") - offset;
            if (-34..480).contains(&y) {
                r.text_clipped((100, y), text, 28, Color::TEXT, Rect::new(60, 64, 740, 416));
            }
        }
        self.image_natural(r, "legal-scroll-bg", 746, 64);
        let height = (416 * 416 / (self.legal_max_scroll() + 416)).max(8);
        let y = 64
            + (416 - height) * scroll.min(self.legal_max_scroll()) / self.legal_max_scroll().max(1);
        self.image(
            r,
            "legal-scroll",
            (746, i32::try_from(y).expect("scroll"), 31, height),
        );
        self.image_natural(r, "legal-shadow", 65, 456);
    }
}
