//! `ChangeLanguage`, confirmation, progress and retry using owner dictionary text.
use super::{centered_lines, fitted, Color, LanguagePage, Renderer, SystemPage, UiModel};
use mazda_core::CommanderEvent;
impl UiModel {
    pub(super) fn home_label(&self) -> &str {
        let key = if self.selection() == super::Screen::Navigation {
            "NavigationTitle"
        } else {
            self.selection().label()
        };
        self.loc("system", key, self.selection().label())
    }
    pub(super) fn tab_label(&self, index: usize, fallback: &'static str) -> &str {
        let (app, key) = match index {
            0 => ("common", "HUDTab"),
            1 => ("common", "DisplayTab"),
            2 => ("syssettings", "Safety"),
            3 => ("common", "SoundTab"),
            4 => ("syssettings", "Clock"),
            5 => ("common", "VehicleTab"),
            6 => ("common", "DevicesTab"),
            _ => ("common", "SystemTab"),
        };
        self.loc(app, key, fallback)
    }
    pub(super) fn status_dictionary(&self) -> &'static str {
        if self.system_child() && self.system.page == SystemPage::Version {
            "sysupdate"
        } else if self.screen == super::Screen::Settings {
            "syssettings"
        } else if self.screen == super::Screen::Navigation {
            "emnavi"
        } else {
            "system"
        }
    }
    #[must_use]
    pub fn language_code(&self) -> &str {
        self.assets
            .languages
            .get(self.language.current)
            .map_or("en_US", |p| p.code.as_str())
    }
    /// Explicit host fixture; does not change macOS or a real CMU language.
    pub fn set_host_language(&mut self, code: &str) -> bool {
        if !self.vehicle.is_confirmed_parked() {
            return false;
        }
        let Some(index) = self
            .assets
            .languages
            .packs
            .iter()
            .position(|p| p.code == code)
        else {
            return false;
        };
        self.language.current = index;
        self.language.page = LanguagePage::Closed;
        true
    }
    pub fn set_language_change_failure(&mut self, fail: bool) {
        self.language.fail_changes = fail;
    }
    pub(super) fn tr<'a>(&'a self, app: &str, english: &'a str) -> &'a str {
        self.assets
            .languages
            .get(self.language.current)
            .map_or(english, |p| p.text(app, english))
    }
    pub(super) fn loc<'a>(&'a self, app: &str, key: &str, fallback: &'a str) -> &'a str {
        self.assets
            .languages
            .get(self.language.current)
            .map_or(fallback, |p| p.key(app, key, fallback))
    }
    pub(super) fn language_label(&self, index: usize) -> &str {
        let Some(pack) = self.assets.languages.get(index) else {
            return "---";
        };
        self.loc("syssettings", &pack.id, &pack.id)
    }
    pub(super) fn full_screen_dialog(&self) -> bool {
        self.system_child()
            && (self.language.full_screen()
                || matches!(
                    self.system.page,
                    SystemPage::Confirm { .. } | SystemPage::Progress { .. }
                ))
    }
    pub(super) fn language_pointer(&mut self, x: usize, y: usize) {
        if x >= 800 || y >= 480 {
            return;
        }
        let parked = self.vehicle.is_confirmed_parked();
        let count = self.assets.languages.packs.len();
        match self.language.page {
            LanguagePage::List { focus } if (133..480).contains(&y) && x >= 94 && parked => {
                let row = focus.saturating_sub(4) + (y - 133) / 64;
                if row < count {
                    self.language.page = LanguagePage::List { focus: row };
                    self.language.event(CommanderEvent::Select, parked, count);
                }
            }
            LanguagePage::Confirm { target, .. } | LanguagePage::Error { target, .. }
                if (69..731).contains(&x) && (382..462).contains(&y) =>
            {
                self.language.page = if matches!(self.language.page, LanguagePage::Error { .. }) {
                    LanguagePage::Error {
                        target,
                        yes: x >= 401,
                    }
                } else {
                    LanguagePage::Confirm {
                        target,
                        yes: x >= 401,
                    }
                };
                self.language.event(CommanderEvent::Select, parked, count);
            }
            _ => {}
        }
    }
    pub(super) fn render_language(&self, r: &mut impl Renderer) {
        match self.language.page {
            LanguagePage::List { focus } => self.render_language_list(r, focus),
            LanguagePage::Confirm { target, yes } | LanguagePage::Error { target, yes } => {
                self.image(r, "settings-dialog-bg", (0, 0, 800, 480));
                let error = matches!(self.language.page, LanguagePage::Error { .. });
                let text = if error {
                    self.loc(
                        "syssettings",
                        "LanguageChangeErrorText",
                        "Unable to change the language. Would you like to Retry?",
                    )
                    .to_owned()
                } else {
                    let voiced = self
                        .assets
                        .languages
                        .get(target)
                        .is_some_and(|p| p.vr || p.tts);
                    let (key, fallback) = if voiced {
                        (
                            "LanguageConfText",
                            "Are you sure you want to change the language to {languageID}?",
                        )
                    } else {
                        ("LanguageConfTextNoVR","{languageID} is a Display Only language. UK English will be used for voice functions. Change display language to  {languageID}? ")
                    };
                    self.loc("syssettings", key, fallback)
                        .replace("{languageID}", self.language_label(target))
                };
                centered_lines(r, &text, 77, 150, 646, 35, 42);
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
                        x + (330 - crate::text_width(label, 35)) / 2,
                        400,
                        label,
                        35,
                        Color::TEXT,
                    );
                }
            }
            LanguagePage::Progress { .. } => {
                self.image(r, "settings-dialog-bg", (0, 0, 800, 480));
                centered_lines(
                    r,
                    self.loc(
                        "syssettings",
                        "LanguageChangeProgressText",
                        "Changing Language",
                    ),
                    79,
                    150,
                    646,
                    35,
                    38,
                );
                self.image_natural(r, "system-meter", 344, 250);
            }
            LanguagePage::Closed => {}
        }
    }
    fn render_language_list(&self, r: &mut impl Renderer, focus: usize) {
        self.image(r, "list-background", (68, 64, 732, 416));
        self.image_natural(r, "settings-title", 69, 64);
        fitted(
            r,
            100,
            75,
            self.loc("syssettings", "ChooseLanguage", "Select Language"),
            620,
            40,
            Color::TEXT,
        );
        let parked = self.vehicle.is_confirmed_parked();
        for (visible, (index, pack)) in self
            .assets
            .languages
            .packs
            .iter()
            .enumerate()
            .skip(focus.saturating_sub(4))
            .take(6)
            .enumerate()
        {
            let y = 133 + visible * 64;
            if index == focus {
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
            let radio = match (parked, index == self.language.current) {
                (true, true) => "settings-radio",
                (true, false) => "settings-radio-empty",
                (false, true) => "settings-radio-disabled",
                (false, false) => "settings-radio-empty-disabled",
            };
            self.image_natural(r, radio, 95, i32::try_from(y + 14).expect("row"));
            let label = crate::typography::ellipsized(self.language_label(index), 489, 28);
            r.text(
                139,
                y + 14,
                &label,
                28,
                if parked {
                    Color::TEXT
                } else {
                    Color::rgb(146, 148, 151)
                },
            );
            if pack.vr {
                if let Some(icon) = self.assets.get("language-voice") {
                    let (w, h) = icon.size();
                    r.image(
                        681 + i32::try_from(53_usize.saturating_sub(w) / 2).expect("icon"),
                        i32::try_from(y + 6 + 53_usize.saturating_sub(h) / 2).expect("row"),
                        w,
                        h,
                        icon,
                    );
                }
            }
        }
    }
}
