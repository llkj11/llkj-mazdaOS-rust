//! NA Devices tab and source btpairing List2/Dialog3/DialPad2 host views.
use super::{dialog_block, dialog_line, fitted, BtPage, Color, Renderer, Screen, UiModel};
use crate::{text_width, BluetoothFixture};
use mazda_core::CommanderEvent;

impl UiModel {
    pub fn set_bluetooth_fixture(&mut self, fixture: BluetoothFixture) {
        self.bluetooth.configure(fixture);
    }
    pub fn set_projection_fixture(&mut self, available: bool) {
        self.bluetooth.projection.available = available;
        self.bluetooth.page = BtPage::Closed;
        self.row = self.row.min(self.rows().len().saturating_sub(1));
    }
    #[must_use]
    pub const fn bluetooth_enabled(&self) -> bool {
        self.bluetooth.enabled
    }
    #[must_use]
    pub fn bluetooth_paired_count(&self) -> usize {
        self.bluetooth.devices.len()
    }
    pub(super) fn devices_labels(&self) -> &'static [&'static str] {
        if self.bluetooth.projection.available {
            &["Bluetooth", "Android Auto", "Apple CarPlay"]
        } else {
            &["Bluetooth"]
        }
    }
    pub(super) fn devices_child(&self) -> bool {
        self.screen == Screen::Settings
            && self.settings_tab == 6
            && self.bluetooth.page != BtPage::Closed
    }
    pub(super) fn select_devices_row(&mut self) {
        if self.row == 0 {
            self.bluetooth.show(BtPage::Manager);
            self.notice = None;
        } else if self.row_enabled(self.row) && self.bluetooth.projection.available {
            if self.row == 1 {
                self.bluetooth.projection.android_auto = !self.bluetooth.projection.android_auto;
            } else if self.bluetooth.projection.carplay {
                self.bluetooth.projection.carplay = false;
                self.bluetooth.show(BtPage::ConfirmCarPlay);
            } else {
                self.bluetooth.projection.carplay = true;
            }
        }
    }
    fn bt<'a>(&'a self, key: &'a str) -> &'a str {
        self.loc("btpairing", key, key)
    }
    fn bt_substitute(&self, key: &str) -> String {
        self.bt(key)
            .replace("{deviceName}", self.bluetooth.name())
            .replace("{DeviceName}", self.bluetooth.name())
    }
    fn bluetooth_checkbox(
        &self,
        renderer: &mut impl Renderer,
        y: usize,
        checked: bool,
        enabled: bool,
    ) {
        let name = match (checked, enabled) {
            (true, true) => "settings-check",
            (false, true) => "settings-uncheck",
            (true, false) => "settings-check-disabled",
            (false, false) => "settings-uncheck-disabled",
        };
        self.bt_icon(renderer, name, 681, y + 5);
    }
    fn bt_icon(&self, renderer: &mut impl Renderer, name: &str, x: usize, y: usize) {
        if let Some(bitmap) = self.assets.get(name) {
            let (width, height) = bitmap.size();
            renderer.image(
                i32::try_from(x + 53_usize.saturating_sub(width) / 2).expect("cell"),
                i32::try_from(y + 53_usize.saturating_sub(height) / 2).expect("cell"),
                width,
                height,
                bitmap,
            );
        }
    }
    pub(super) fn render_devices_root(&self, renderer: &mut impl Renderer) {
        for (row, key) in ["Bluetooth", "AndroidAuto", "CarPlayApple"]
            .iter()
            .take(self.devices_labels().len())
            .enumerate()
        {
            let y = 133 + row * 64;
            self.bt_list_background(renderer, y, self.settings_rows && row == self.row);
            let enabled = self.row_enabled(row);
            fitted(
                renderer,
                139,
                y + 17,
                self.loc("syssettings", key, key),
                542,
                28,
                if enabled {
                    Color::TEXT
                } else {
                    Color::rgb(146, 148, 151)
                },
            );
            if row != 0 {
                self.bluetooth_checkbox(
                    renderer,
                    y,
                    if row == 1 {
                        self.bluetooth.projection.android_auto
                    } else {
                        self.bluetooth.projection.carplay
                    },
                    enabled,
                );
            }
        }
    }
    fn bt_list_background(&self, renderer: &mut impl Renderer, y: usize, focused: bool) {
        if focused {
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
    }
    pub(super) fn bluetooth_pointer(&mut self, x: usize, y: usize) {
        if x >= 800 || y >= 480 {
            return;
        }
        let parked = self.vehicle.is_confirmed_parked();
        let ready = self.system.equipment.services_ready;
        if self.bluetooth.page == BtPage::Keypad {
            if x < 61 && (245..333).contains(&y) {
                self.bluetooth.event(CommanderEvent::Back, parked, ready);
                return;
            }
            if let Some(key) = (0..13).find(|&key| {
                let (bx, by) = keypad_position(key);
                (bx..bx + 126).contains(&x) && (by..by + 79).contains(&y)
            }) {
                if key == 12 && self.bluetooth.draft_code.len() != 4 {
                    return;
                }
                self.bluetooth.focus = key;
            } else {
                return;
            }
        } else if self.bluetooth.page.is_list() {
            if x < 94 || y < 133 {
                return;
            }
            let row = self.bluetooth.focus.saturating_sub(4) + (y - 133) / 64;
            if !self.bluetooth.row_enabled(row, parked, ready) {
                return;
            }
            self.bluetooth.focus = row;
        } else {
            let count = self.bluetooth.page.button_count();
            if count == 0 || !(69..731).contains(&x) || !(382..462).contains(&y) {
                return;
            }
            self.bluetooth.focus = ((x - 69) / (664 / count)).min(count - 1);
        }
        self.bluetooth.event(CommanderEvent::Select, parked, ready);
    }
    pub(super) fn render_bluetooth(&self, renderer: &mut impl Renderer) {
        if self.bluetooth.page.is_list() {
            self.render_bluetooth_list(renderer);
            return;
        }
        if self.bluetooth.page == BtPage::Keypad {
            self.render_pair_keypad(renderer);
            return;
        }
        if self.bluetooth.page == BtPage::ConfirmCarPlay {
            self.image(renderer, "settings-dialog-bg", (0, 0, 800, 480));
            dialog_block(
                renderer,
                self.loc("syssettings", "AppleCarPlayDisable", "AppleCarPlayDisable"),
                150,
                233,
            );
        } else {
            self.image_natural(renderer, "system-dialog-bg", 0, 64);
            self.render_bluetooth_dialog(renderer);
        }
        self.render_bluetooth_buttons(renderer);
    }
    fn render_bluetooth_list(&self, renderer: &mut impl Renderer) {
        self.image(renderer, "list-background", (68, 64, 732, 416));
        self.image_natural(renderer, "settings-title", 69, 64);
        let title = match self.bluetooth.page {
            BtPage::Manager => self.bt("BluetoothConnection").to_owned(),
            BtPage::Settings => self.bt("Settings").to_owned(),
            BtPage::Profiles => self.bt_substitute("ConnectoinTypeTitle"),
            _ => self.bt("SelectDeviceToRemove").to_owned(),
        };
        fitted(
            renderer,
            100,
            75,
            &title,
            if self.bluetooth.page == BtPage::DeleteList {
                688
            } else {
                549
            },
            40,
            Color::TEXT,
        );
        let first = self.bluetooth.focus.saturating_sub(4);
        for row in (first..self.bluetooth.row_count()).take(5) {
            let y = 133 + (row - first) * 64;
            let enabled = self.bluetooth.row_enabled(
                row,
                self.vehicle.is_confirmed_parked(),
                self.system.equipment.services_ready,
            );
            self.bt_list_background(renderer, y, row == self.bluetooth.focus && enabled);
            let label = self.bluetooth_row_label(row);
            let indented = (self.bluetooth.page == BtPage::Manager && row >= 3)
                || (self.bluetooth.page == BtPage::DeleteList
                    && row < self.bluetooth.devices.len());
            fitted(
                renderer,
                if indented { 177 } else { 139 },
                y + 12,
                &label,
                match self.bluetooth.page {
                    BtPage::DeleteList => {
                        if indented {
                            567
                        } else {
                            605
                        }
                    }
                    BtPage::Profiles => 489,
                    _ => {
                        if indented {
                            451
                        } else {
                            542
                        }
                    }
                },
                35,
                if enabled {
                    Color::TEXT
                } else {
                    Color::rgb(146, 148, 151)
                },
            );
            self.bluetooth_row_icons(renderer, row, y, enabled);
        }
    }
    fn bluetooth_row_label(&self, row: usize) -> String {
        let key = match self.bluetooth.page {
            BtPage::Manager => match row {
                0 => "Bluetooth",
                1 => "Settings",
                2 => "AddNewDevice",
                _ => return self.bluetooth.devices[row - 3].name.to_owned(),
            },
            BtPage::Settings => ["BluetoothInformation", "ChangePaircode", "RemoveDevice"][row],
            BtPage::Profiles => [
                "PhoneAndAudio",
                "PhoneOnly",
                "AudioOnly",
                "Disconnect",
                "RemovePairDevice",
            ][row],
            _ => {
                if let Some(d) = self.bluetooth.devices.get(row) {
                    return d.name.to_owned();
                }
                "RemoveDevice"
            }
        };
        self.bt(key).to_owned()
    }
    fn bluetooth_row_icons(
        &self,
        renderer: &mut impl Renderer,
        row: usize,
        y: usize,
        enabled: bool,
    ) {
        if self.bluetooth.page == BtPage::DeleteList {
            self.bt_icon(renderer, "bt-remove", 86, y + 5);
        } else if self.bluetooth.page == BtPage::Manager {
            if row == 0 {
                self.bluetooth_checkbox(renderer, y, self.bluetooth.enabled, enabled);
            } else if row >= 3 {
                let roles = self.bluetooth.devices[row - 3].roles;
                if roles != 0 {
                    self.bt_icon(renderer, "bt-check", 86, y + 5);
                }
                self.bt_role_icons(renderer, roles, y);
            }
        } else if self.bluetooth.page == BtPage::Profiles && row < 3 {
            let roles = [3, 1, 2][row];
            let checked = self.bluetooth.devices[self.bluetooth.target].roles == roles;
            let name = match (checked, enabled) {
                (true, true) => "settings-radio",
                (false, true) => "settings-radio-empty",
                (true, false) => "settings-radio-disabled",
                (false, false) => "settings-radio-empty-disabled",
            };
            self.bt_icon(renderer, name, 86, y + 5);
            self.bt_role_icons(renderer, roles, y);
        }
    }
    fn bt_role_icons(&self, renderer: &mut impl Renderer, roles: u8, y: usize) {
        if roles & 1 != 0 {
            self.bt_icon(renderer, "bt-phone", 628, y + 5);
        }
        if roles & 2 != 0 {
            self.bt_icon(
                renderer,
                "bt-audio",
                if roles & 1 != 0 { 681 } else { 628 },
                y + 5,
            );
        }
    }
    fn render_bluetooth_dialog(&self, renderer: &mut impl Renderer) {
        match self.bluetooth.page {
            BtPage::Discoverable => {
                dialog_block(
                    renderer,
                    &self.bt("Discoverable").replace("{modelName}", "Mazda"),
                    150,
                    83,
                );
                let seconds = 180_u64.saturating_sub(self.bluetooth.elapsed_ms / 1000);
                dialog_line(
                    renderer,
                    &self
                        .bt("DiscoverableTimeout")
                        .replace("{discoverableTimeout}", &seconds.to_string()),
                    326,
                );
            }
            BtPage::NumericConfirm => {
                dialog_block(renderer, self.bt("NumericConfirmSSP"), 133, 120);
                dialog_line(renderer, "123456", 271);
            }
            BtPage::LegacyConfirm => {
                dialog_block(renderer, self.bt("NumericConfirmLegacy"), 133, 120);
                dialog_line(renderer, "1234", 271); // Synthetic incoming code, separate from the editor PIN.
            }
            BtPage::Information => {
                dialog_line(renderer, self.bt("BluetoothInformation"), 164);
                dialog_line(renderer, "Mazda", 229);
                dialog_line(renderer, "-----", 295); // No real MAC address is read or invented.
            }
            BtPage::Pairing | BtPage::PairSuccess | BtPage::PreparingPair => {
                let (a, bitmap) = match self.bluetooth.page {
                    BtPage::Pairing => ("PairingInProcess", "PairingInProcessConnection"),
                    BtPage::PairSuccess => ("PairSuccess", "PairSuccessConnection"),
                    _ => ("DisconnectingToPair", ""),
                };
                dialog_line(renderer, self.bt(a), 150);
                self.image_natural(renderer, "update-spinner", 344, 195);
                dialog_line(renderer, self.bt(bitmap), 314);
            }
            BtPage::Connecting | BtPage::Disconnecting | BtPage::Deleting | BtPage::Cancelling => {
                dialog_block(renderer, &self.bluetooth_message(), 150, 81);
                self.image_natural(renderer, "update-spinner", 344, 250);
            }
            page if page.failure() => {
                let height = if matches!(page, BtPage::PairFailed | BtPage::ConnectFailed) {
                    117
                } else {
                    233
                };
                dialog_block(renderer, &self.bluetooth_message(), 150, height);
                if height == 117 {
                    dialog_block(
                        renderer,
                        self.bt(if page == BtPage::PairFailed && self.bluetooth.timed_out {
                            "TimeLimitExceeded"
                        } else {
                            "UnknownError"
                        }),
                        267,
                        117,
                    );
                }
            }
            _ => dialog_block(renderer, &self.bluetooth_message(), 150, 233),
        }
    }
    fn bluetooth_message(&self) -> String {
        let key = match self.bluetooth.page {
            BtPage::Connecting => "ConnectingDevice",
            BtPage::Connected => "ConnectSuccess",
            BtPage::Disconnecting => "DisconnectingDevice",
            BtPage::Disconnected => "DisconnectSuccess",
            BtPage::Deleting => "DeleteDevice",
            BtPage::Cancelling => "PairCancelling",
            BtPage::PairFailed => "PairingDevice",
            BtPage::ConnectFailed => "ConnectFailed",
            BtPage::DisconnectFailed => "DisconnectFailed",
            BtPage::PrePairFailed => "NoPairDisconnectFail",
            BtPage::DeleteFailed => "DeleteFailed",
            BtPage::ConfirmDelete if self.bluetooth.all => "All",
            BtPage::ConfirmDelete => "ConfirmDelete",
            BtPage::Deleted if self.bluetooth.result_name.is_empty() => "allDevices",
            BtPage::Deleted => {
                return self
                    .bt("DeleteComplete")
                    .replace("{deviceName}", self.bluetooth.result_name)
            }
            _ => "",
        };
        self.bt_substitute(key)
    }
    fn render_bluetooth_buttons(&self, renderer: &mut impl Renderer) {
        let page = self.bluetooth.page;
        let labels: &[(&str, &str)] = match page {
            BtPage::NumericConfirm => &[("common", "Cancel"), ("common", "No"), ("common", "Yes")],
            BtPage::ConfirmDelete => &[("common", "No"), ("common", "Yes")],
            BtPage::Information => &[("common", "Ok")],
            BtPage::ConfirmCarPlay => &[("common", "Yes")],
            BtPage::Discoverable | BtPage::LegacyConfirm | BtPage::PreparingPair => {
                &[("common", "Cancel")]
            }
            page if page.failure() => &[("btpairing", "Back"), ("btpairing", "Retry")],
            _ => &[],
        };
        for (index, (app, key)) in labels.iter().enumerate() {
            let count = labels.len();
            let width = [0, 662, 330, 220][count];
            let x = 69 + index * (width + if count == 3 { 1 } else { 2 });
            let allowed = !page.failure()
                || index == 0
                || (self.vehicle.is_confirmed_parked() && self.system.equipment.services_ready);
            let selected = index == self.bluetooth.focus && allowed;
            let name = match (count, selected) {
                (1, _) => "system-dialog-ok",
                (2, true) => "settings-dialog-button-focus",
                (2, false) => "settings-dialog-button",
                (3, true) => "bt-three-focus",
                _ => "bt-three",
            };
            self.image_natural(renderer, name, i32::try_from(x).expect("button"), 382);
            let label = crate::typography::ellipsized(self.loc(app, key, key), width - 20, 35);
            renderer.text(
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
    fn keypad_image(
        &self,
        renderer: &mut impl Renderer,
        name: &str,
        x: usize,
        y: usize,
        disabled: bool,
    ) {
        if let Some(bitmap) = self.assets.get(name) {
            let (width, height) = bitmap.size();
            if disabled {
                renderer.image(
                    i32::try_from(x).expect("key"),
                    i32::try_from(y).expect("key"),
                    width,
                    height,
                    &bitmap.with_opacity(128),
                );
            } else {
                self.image_natural(
                    renderer,
                    name,
                    i32::try_from(x).expect("key"),
                    i32::try_from(y).expect("key"),
                );
            }
        }
    }
    fn render_pair_keypad(&self, renderer: &mut impl Renderer) {
        self.image_natural(renderer, "bt-keypad-bg", 0, 64);
        self.image_natural(renderer, "bt-keypad-back", 5, 260);
        for (i, digit) in self.bluetooth.draft_code.chars().enumerate() {
            renderer.text(
                208 + i * 94 + (94 - text_width(&digit.to_string(), 40)) / 2,
                91,
                &digit.to_string(),
                40,
                Color::TEXT,
            );
        }
        for key in 0..13 {
            let (x, y) = keypad_position(key);
            let disabled = key == 12 && self.bluetooth.draft_code.len() != 4;
            self.keypad_image(renderer, "bt-keypad-key", x, y, disabled);
            if self.bluetooth.focus == key {
                self.image_natural(
                    renderer,
                    "bt-keypad-focus",
                    i32::try_from(x).expect("key"),
                    i32::try_from(y).expect("key"),
                );
            }
            let icon = match key {
                0 => Some("bt-keypad-clear"),
                10 => Some("bt-keypad-cancel"),
                12 => Some("bt-keypad-ok"),
                _ => None,
            };
            if let Some(icon) = icon {
                if let Some(bitmap) = self.assets.get(icon) {
                    let (width, height) = bitmap.size();
                    let dimmed = bitmap.with_opacity(128);
                    let bitmap = if disabled { &dimmed } else { bitmap };
                    renderer.image(
                        i32::try_from(x + 126_usize.saturating_sub(width) / 2).expect("key"),
                        i32::try_from(y + 79_usize.saturating_sub(height) / 2).expect("key"),
                        width,
                        height,
                        bitmap,
                    );
                }
            } else {
                let label = if key == 11 {
                    "0".into()
                } else {
                    key.to_string()
                };
                renderer.text(
                    x + (126 - text_width(&label, 40)) / 2,
                    y + 10,
                    &label,
                    40,
                    Color::TEXT,
                );
            }
        }
    }
}
fn keypad_position(key: usize) -> (usize, usize) {
    match key {
        0 => (590, 76),
        10 => (209, 396),
        11 => (336, 396),
        12 => (463, 396),
        n => (209 + (n - 1) % 3 * 127, 156 + (n - 1) / 3 * 80),
    }
}
