//! sysupdate music contexts, original Dialog3/List2 geometry, host-only fixtures.
use super::{dialog_block, dialog_line, fitted, Color, MusicPage, Renderer, UiModel};
use crate::{text_width, MusicUpdateFixture};
use mazda_core::CommanderEvent;

impl UiModel {
    pub fn set_music_update_fixture(&mut self, fixture: MusicUpdateFixture) {
        self.music_update.fixture = fixture;
        self.music_update.page = MusicPage::Closed;
    }
    /// Only the simulator's successful synthetic operation changes this value.
    #[must_use]
    pub const fn simulated_music_version(&self) -> Option<&'static str> {
        if self.music_update.installed {
            Some(crate::music_update::MusicUpdateSession::VERSION)
        } else {
            None
        }
    }
    pub(super) fn music_update_pointer(&mut self, x: usize, y: usize) {
        if x >= 800 || y >= 480 {
            return;
        }
        let page = self.music_update.page;
        if page == MusicPage::Packages {
            if x >= 94 && (133..261).contains(&y) {
                self.music_update.focus = (y - 133) / 64;
            } else {
                return;
            }
        } else if (69..731).contains(&x)
            && (382..462).contains(&y)
            && (!page.busy() || page == MusicPage::Searching)
        {
            self.music_update.focus = usize::from(page.two_buttons() && x >= 401);
        } else {
            return;
        }
        self.music_update.event(
            CommanderEvent::Select,
            self.vehicle.is_confirmed_parked(),
            self.system.equipment.services_ready,
        );
    }
    fn update_text(&self, key: &str) -> String {
        // Exact source IDs; English fallback comes from the owner-local dictionaries.
        self.loc("sysupdate", key, key).to_owned()
    }
    fn update_substitute(&self, key: &str, token: &str, value: &str) -> String {
        self.update_text(key).replace(token, value)
    }
    pub(super) fn render_music_update(&self, r: &mut impl Renderer) {
        if self.music_update.page == MusicPage::Packages {
            self.render_music_packages(r);
            return;
        }
        if self.music_update.page.full_screen() {
            self.image(r, "settings-dialog-bg", (0, 0, 800, 480));
        } else {
            self.image_natural(r, "system-dialog-bg", 0, 64);
        }
        self.render_update_content(r);
        self.render_update_buttons(r);
    }
    fn render_update_content(&self, r: &mut impl Renderer) {
        use MusicPage::{Confirm, Installing, Preparing, Retrying, Searching};
        let version = crate::music_update::MusicUpdateSession::VERSION;
        match self.music_update.page {
            Confirm => {
                let current = self.simulated_music_version().unwrap_or("-----");
                dialog_block(
                    r,
                    &self.update_substitute("CurrentDatabase", "{CurrentVersion}", current),
                    150,
                    83,
                );
                dialog_block(
                    r,
                    &self.update_substitute("InstallDatabase", "{InstallVersion}", version),
                    242,
                    83,
                );
                dialog_line(r, &self.update_text("SystemNA"), 336);
            }
            Searching | Preparing => {
                let key = if self.music_update.page == Searching {
                    "SearchingForMusicUpdates"
                } else {
                    "RebootReflash"
                };
                dialog_line(r, &self.update_text(key), 150);
                self.image_natural(r, "update-spinner", 344, 195);
            }
            Installing | Retrying => {
                dialog_block(
                    r,
                    &self.update_substitute("InstallMusic", "{MusicVersion}", version),
                    150,
                    80,
                );
                self.image_natural(r, "update-progress-bg", 79, 292);
                let width =
                    usize::try_from(self.music_update.elapsed_ms.min(4000)).expect("bounded") * 640
                        / 4000;
                if let Some(fill) = self
                    .assets
                    .get("update-progress-fill")
                    .and_then(|b| b.centered_portion(width, false))
                {
                    r.image(80, 294, width, 8, &fill);
                }
                dialog_line(r, &self.update_text("DonotRestart"), 314);
            }
            MusicPage::NoPackages | MusicPage::NeedsFailsafe => {
                let key = if self.music_update.page == MusicPage::NoPackages {
                    "NoNewMusicPackages"
                } else {
                    "NeedFailSafeImageUpdate"
                };
                dialog_block(r, &self.update_text(key), 150, 233);
            }
            MusicPage::RetryValidationFailed => {
                fitted(
                    r,
                    125,
                    96,
                    &self.update_text("ValidationFailed"),
                    615,
                    40,
                    Color::TEXT,
                );
                self.image_natural(r, "update-title-separator", 60, 147);
                dialog_line(r, &self.update_text("AUTH_FAILED"), 229);
                dialog_block(r, &self.update_text("SysCatalogRestart"), 295, 102);
            }
            _ => {
                let (a, b) = self.update_message();
                dialog_block(r, &a, 150, 117);
                dialog_block(r, &b, 267, 117);
            }
        }
    }
    fn update_message(&self) -> (String, String) {
        let (a, b) = match self.music_update.page {
            MusicPage::Search => ("SearchMusicDBUpdate", "WebsiteInformation"),
            MusicPage::SearchFailed => ("SearchFailed", "SEARCH_FAILED"),
            MusicPage::ValidationFailed => ("ValidationFailed", "AUTH_FAILED"),
            MusicPage::Unavailable | MusicPage::InstallFailed => {
                ("FunctionNA", "SysCatalogRestart")
            }
            MusicPage::Success => {
                return (
                    self.update_substitute(
                        "MusicInstallSuccess",
                        "{MusicVersion}",
                        crate::music_update::MusicUpdateSession::VERSION,
                    ),
                    self.update_text("SystemRestart"),
                )
            }
            _ => ("", ""),
        };
        (self.update_text(a), self.update_text(b))
    }
    fn render_update_buttons(&self, r: &mut impl Renderer) {
        let page = self.music_update.page;
        let labels: &[(&str, &str)] = match page {
            MusicPage::Search => &[("common", "Cancel"), ("sysupdate", "Search")],
            MusicPage::SearchFailed | MusicPage::InstallFailed => {
                &[("common", "Cancel"), ("sysupdate", "Retry")]
            }
            MusicPage::Confirm => &[("common", "Back"), ("sysupdate", "Install")],
            MusicPage::Searching => &[("common", "Cancel")],
            MusicPage::Preparing | MusicPage::Installing | MusicPage::Retrying => &[],
            _ => &[("common", "Ok")],
        };
        for (index, (app, key)) in labels.iter().enumerate() {
            let two = labels.len() == 2;
            let width = if two { 330 } else { 662 };
            let x = 69 + index * 332;
            let allowed = index == 0
                || (self.vehicle.is_confirmed_parked() && self.system.equipment.services_ready);
            let selected = index == self.music_update.focus;
            self.image_natural(
                r,
                if !two {
                    "system-dialog-ok"
                } else if selected && allowed {
                    "settings-dialog-button-focus"
                } else {
                    "settings-dialog-button"
                },
                i32::try_from(x).expect("button"),
                382,
            );
            let label = crate::typography::ellipsized(self.loc(app, key, key), width - 24, 35);
            r.text(
                x + width.saturating_sub(text_width(&label, 35)) / 2,
                400,
                &label,
                35,
                if allowed {
                    Color::TEXT
                } else {
                    Color::rgb(146, 148, 151)
                },
            );
        }
    }
    fn render_music_packages(&self, r: &mut impl Renderer) {
        self.image(r, "list-background", (68, 64, 732, 416));
        self.image_natural(r, "settings-title", 69, 64);
        fitted(
            r,
            100,
            75,
            &self.update_text("AvailablePackages"),
            688,
            40,
            Color::TEXT,
        );
        for (row, key) in ["CurrentVersion", "GracenoteVersion"].iter().enumerate() {
            let y = 133 + row * 64;
            if row == self.music_update.focus {
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
            let allowed = row == 0
                || (self.vehicle.is_confirmed_parked() && self.system.equipment.services_ready);
            let color = if allowed {
                Color::TEXT
            } else {
                Color::rgb(146, 148, 151)
            };
            fitted(r, 139, y + 12, &self.update_text(key), 456, 35, color);
            let version = if row == 0 {
                self.simulated_music_version().unwrap_or("")
            } else {
                crate::music_update::MusicUpdateSession::VERSION
            };
            let label = crate::typography::ellipsized(version, 129, 28);
            r.text(
                724_usize.saturating_sub(text_width(&label, 28)),
                y + 17,
                &label,
                28,
                color,
            );
        }
        // Source marks Current as non-activating; no caret is drawn for either row.
    }
}
