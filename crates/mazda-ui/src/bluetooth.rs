//! Firmware btpairing routes with finite host devices. No Bluetooth/OS APIs.
use mazda_core::CommanderEvent;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothFixture {
    #[default]
    Empty,
    Paired,
    Ssp,
    Legacy,
    PairFails,
    ConnectFails,
    DisconnectFails,
    DeleteFails,
}
impl BluetoothFixture {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "empty" => Self::Empty,
            "paired" => Self::Paired,
            "ssp" => Self::Ssp,
            "legacy" => Self::Legacy,
            "pair-fails" => Self::PairFails,
            "connect-fails" => Self::ConnectFails,
            "disconnect-fails" => Self::DisconnectFails,
            "delete-fails" => Self::DeleteFails,
            _ => return None,
        })
    }
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BtPage {
    #[default]
    Closed,
    Manager,
    Settings,
    Information,
    Profiles,
    Discoverable,
    NumericConfirm,
    LegacyConfirm,
    Pairing,
    PairSuccess,
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
    ConfirmDelete,
    Deleting,
    Deleted,
    DeleteList,
    PairFailed,
    ConnectFailed,
    DisconnectFailed,
    DeleteFailed,
    Cancelling,
    PreparingPair,
    PrePairFailed,
    ConfirmCarPlay,
    Keypad,
}
impl BtPage {
    pub const fn is_list(self) -> bool {
        matches!(
            self,
            Self::Manager | Self::Settings | Self::Profiles | Self::DeleteList
        )
    }
    pub const fn pending(self) -> bool {
        matches!(
            self,
            Self::Discoverable
                | Self::NumericConfirm
                | Self::LegacyConfirm
                | Self::Pairing
                | Self::PairSuccess
                | Self::Connecting
                | Self::Disconnecting
                | Self::Deleting
                | Self::Cancelling
                | Self::PreparingPair
        )
    }
    pub const fn result(self) -> bool {
        matches!(self, Self::Connected | Self::Disconnected | Self::Deleted)
    }
    pub const fn failure(self) -> bool {
        matches!(
            self,
            Self::PairFailed
                | Self::ConnectFailed
                | Self::DisconnectFailed
                | Self::DeleteFailed
                | Self::PrePairFailed
        )
    }
    pub const fn button_count(self) -> usize {
        match self {
            Self::NumericConfirm => 3,
            Self::ConfirmDelete
            | Self::PairFailed
            | Self::ConnectFailed
            | Self::DisconnectFailed
            | Self::DeleteFailed
            | Self::PrePairFailed => 2,
            Self::Information
            | Self::Discoverable
            | Self::LegacyConfirm
            | Self::PreparingPair
            | Self::ConfirmCarPlay => 1,
            _ => 0,
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct BtDevice {
    pub name: &'static str,
    pub phone_supported: bool,
    /// Phone=1, Audio=2; separate devices can own the two profiles.
    pub roles: u8,
}
#[derive(Debug, Default, Clone)]
pub(crate) struct ProjectionState {
    pub available: bool,
    pub android_auto: bool,
    pub carplay: bool,
}
#[derive(Debug, Clone)]
pub(crate) struct BluetoothSession {
    pub page: BtPage,
    pub fixture: BluetoothFixture,
    pub enabled: bool,
    pub devices: Vec<BtDevice>,
    pub focus: usize,
    pub target: usize,
    pub desired_roles: u8,
    pub elapsed_ms: u64,
    pub all: bool,
    delete_origin: DeleteOrigin,
    pub timed_out: bool,
    pub result_name: &'static str,
    pub projection: ProjectionState,
    pub pair_code: String,
    pub draft_code: String,
}
#[derive(Debug, Default, Clone, Copy)]
enum DeleteOrigin {
    #[default]
    Profile,
    List,
}
impl Default for BluetoothSession {
    fn default() -> Self {
        Self {
            page: BtPage::Closed,
            fixture: BluetoothFixture::Empty,
            enabled: true,
            devices: Vec::new(),
            focus: 0,
            target: 0,
            desired_roles: 3,
            elapsed_ms: 0,
            all: false,
            delete_origin: DeleteOrigin::Profile,
            timed_out: false,
            result_name: "",
            projection: ProjectionState::default(),
            pair_code: "0000".into(),
            draft_code: String::new(),
        }
    }
}
impl BluetoothSession {
    pub fn configure(&mut self, fixture: BluetoothFixture) {
        self.fixture = fixture;
        self.devices.clear();
        if matches!(
            fixture,
            BluetoothFixture::Paired
                | BluetoothFixture::ConnectFails
                | BluetoothFixture::DisconnectFails
                | BluetoothFixture::DeleteFails
        ) {
            self.devices = vec![
                BtDevice {
                    name: "Simulated Phone",
                    phone_supported: true,
                    roles: 3,
                },
                BtDevice {
                    name: "Simulated Player",
                    phone_supported: false,
                    roles: 0,
                },
            ];
        }
        self.enabled = true;
        self.show(BtPage::Closed);
    }
    pub fn show(&mut self, page: BtPage) {
        if page != BtPage::PairFailed {
            self.timed_out = false;
        }
        self.page = page;
        self.elapsed_ms = 0;
        self.focus = if page == BtPage::NumericConfirm {
            2
        } else {
            usize::from(page.failure())
        };
        if page == BtPage::Profiles
            && self
                .devices
                .get(self.target)
                .is_some_and(|d| !d.phone_supported)
        {
            self.focus = 2;
        }
        if page == BtPage::Keypad {
            self.draft_code = self.pair_code.clone();
            self.focus = 1;
        }
    }
    pub fn name(&self) -> &'static str {
        self.devices
            .get(self.target)
            .map_or("Simulated Phone", |d| d.name)
    }
    pub fn row_count(&self) -> usize {
        match self.page {
            BtPage::Manager => 3 + if self.enabled { self.devices.len() } else { 0 },
            BtPage::Settings => 3,
            BtPage::Profiles => 5,
            BtPage::DeleteList => self.devices.len() + 1,
            _ => 0,
        }
    }
    pub fn row_enabled(&self, row: usize, parked: bool, ready: bool) -> bool {
        if row >= self.row_count() {
            return false;
        }
        if self.page == BtPage::Settings && row == 0 {
            return true;
        }
        if !parked || !ready {
            return false;
        }
        match self.page {
            BtPage::Manager => row != 2 || self.enabled,
            BtPage::Settings => row != 2 || (self.enabled && !self.devices.is_empty()),
            BtPage::Profiles => self.devices.get(self.target).is_some_and(|d| match row {
                0 | 1 => self.enabled && d.phone_supported,
                2 => self.enabled,
                3 => self.enabled && d.roles != 0,
                _ => true,
            }),
            BtPage::DeleteList => !self.devices.is_empty(),
            _ => false,
        }
    }
    pub fn restrict(&mut self, parked: bool, ready: bool) {
        if (!parked || !ready) && self.page.pending() {
            self.show(BtPage::Manager);
        }
        if (!parked || !ready) && self.page == BtPage::ConfirmDelete {
            self.show(BtPage::Manager);
        }
        if (!parked || !ready) && self.page == BtPage::Keypad {
            self.show(BtPage::Settings);
        }
        if (!parked || !ready) && self.page.failure() {
            self.focus = 0;
        }
        if self.page.is_list() && !self.row_enabled(self.focus, parked, ready) {
            self.focus = (0..self.row_count())
                .find(|&row| self.row_enabled(row, parked, ready))
                .unwrap_or(0);
        }
    }
    pub fn rotate(&mut self, forward: bool, parked: bool, ready: bool) {
        let count = if self.page == BtPage::Keypad {
            13
        } else if self.page.is_list() {
            self.row_count()
        } else {
            self.page.button_count()
        };
        for distance in 1..count {
            let Some(next) = (if forward {
                self.focus.checked_add(distance)
            } else {
                self.focus.checked_sub(distance)
            }) else {
                continue;
            };
            if next < count
                && (self.page != BtPage::Keypad || next != 12 || self.draft_code.len() == 4)
                && (!self.page.is_list() || self.row_enabled(next, parked, ready))
            {
                self.focus = next;
                break;
            }
        }
    }
    pub fn event(&mut self, event: CommanderEvent, parked: bool, ready: bool) -> bool {
        if self.page == BtPage::Closed {
            return false;
        }
        let previous_page = self.page;
        self.restrict(parked, ready);
        if previous_page != self.page {
            return true;
        }
        match event {
            CommanderEvent::Back => self.back(),
            CommanderEvent::RotateClockwise => self.rotate(true, parked, ready),
            CommanderEvent::RotateCounterClockwise => self.rotate(false, parked, ready),
            CommanderEvent::Select => {
                if self.page.is_list() {
                    if self.row_enabled(self.focus, parked, ready) {
                        self.select_list();
                    }
                } else {
                    self.select_dialog(parked && ready);
                }
            }
            _ => {}
        }
        true
    }
    fn back(&mut self) {
        match self.page {
            BtPage::Manager | BtPage::ConfirmCarPlay => self.show(BtPage::Closed),
            BtPage::Settings => {
                self.show(BtPage::Manager);
                self.focus = 1;
            }
            BtPage::Profiles => {
                self.show(BtPage::Manager);
                self.focus = 3 + self.target;
            }
            BtPage::DeleteList => {
                self.show(BtPage::Settings);
                self.focus = 2;
            }
            BtPage::Information => self.show(BtPage::Settings),
            BtPage::Keypad => {
                self.show(BtPage::Settings);
                self.focus = 1;
            }
            BtPage::Discoverable
            | BtPage::NumericConfirm
            | BtPage::LegacyConfirm
            | BtPage::PreparingPair => {
                self.show(BtPage::Cancelling);
            }
            BtPage::ConfirmDelete | BtPage::DeleteFailed => self.return_delete_origin(),
            BtPage::ConnectFailed => {
                self.show(BtPage::Profiles);
                self.focus = match self.desired_roles {
                    1 => 1,
                    2 => 2,
                    _ => 0,
                };
            }
            BtPage::DisconnectFailed => {
                self.show(BtPage::Profiles);
                self.focus = 3;
            }
            page if page.failure() || page.result() => self.show(BtPage::Manager),
            _ => {}
        }
    }
    fn return_delete_origin(&mut self) {
        match self.delete_origin {
            DeleteOrigin::Profile => {
                self.show(BtPage::Profiles);
                self.focus = 4;
            }
            DeleteOrigin::List => {
                self.show(BtPage::DeleteList);
                self.focus = if self.all {
                    self.devices.len()
                } else {
                    self.target
                };
            }
        }
    }
    fn select_list(&mut self) {
        match self.page {
            BtPage::Manager => match self.focus {
                0 => {
                    self.enabled = !self.enabled;
                    if !self.enabled {
                        for d in &mut self.devices {
                            d.roles = 0;
                        }
                    }
                }
                1 => self.show(BtPage::Settings),
                2 => self.show(if self.devices.iter().any(|d| d.roles != 0) {
                    BtPage::PreparingPair
                } else {
                    BtPage::Discoverable
                }),
                row => {
                    self.target = row - 3;
                    self.show(BtPage::Profiles);
                }
            },
            BtPage::Settings => match self.focus {
                0 => self.show(BtPage::Information),
                1 => self.show(BtPage::Keypad),
                2 => self.show(BtPage::DeleteList),
                _ => {}
            },
            BtPage::Profiles => match self.focus {
                row @ 0..=2 => {
                    self.desired_roles = [3, 1, 2][row];
                    if self.devices[self.target].roles != self.desired_roles {
                        self.show(BtPage::Connecting);
                    }
                }
                3 => self.show(BtPage::Disconnecting),
                _ => {
                    self.all = false;
                    self.delete_origin = DeleteOrigin::Profile;
                    self.show(BtPage::ConfirmDelete);
                }
            },
            BtPage::DeleteList => {
                self.delete_origin = DeleteOrigin::List;
                self.all = self.focus == self.devices.len();
                if !self.all {
                    self.target = self.focus;
                }
                self.show(BtPage::ConfirmDelete);
            }
            _ => {}
        }
    }
    fn select_dialog(&mut self, allowed: bool) {
        use BtPage::{ConfirmDelete, NumericConfirm};
        match self.page {
            BtPage::Keypad if allowed => match self.focus {
                0 => {
                    self.draft_code.pop();
                }
                10 => {
                    self.show(BtPage::Settings);
                    self.focus = 1;
                }
                12 if self.draft_code.len() == 4 => {
                    self.pair_code = self.draft_code.clone();
                    self.show(BtPage::Settings);
                    self.focus = 1;
                }
                digit @ 1..=9 if self.draft_code.len() < 4 => self
                    .draft_code
                    .push(char::from(b'0' + u8::try_from(digit).expect("digit"))),
                11 if self.draft_code.len() < 4 => self.draft_code.push('0'),
                _ => {}
            },
            BtPage::Information => self.show(BtPage::Settings),
            BtPage::Discoverable | BtPage::LegacyConfirm | BtPage::PreparingPair => {
                self.show(BtPage::Cancelling);
            }
            NumericConfirm => self.show(if self.focus == 2 && allowed {
                BtPage::Pairing
            } else {
                BtPage::Cancelling
            }),
            ConfirmDelete => {
                if self.focus == 1 && allowed {
                    self.show(BtPage::Deleting);
                } else {
                    self.back();
                }
            }
            BtPage::ConfirmCarPlay => {
                self.show(BtPage::Closed);
            }
            page if page.failure() => {
                if self.focus == 0 || !allowed {
                    self.back();
                } else {
                    self.show(match page {
                        BtPage::PairFailed => BtPage::Discoverable,
                        BtPage::ConnectFailed => BtPage::Connecting,
                        BtPage::DisconnectFailed => BtPage::Disconnecting,
                        BtPage::PrePairFailed => BtPage::PreparingPair,
                        _ => BtPage::Deleting,
                    });
                }
            }
            _ => {}
        }
    }
    pub fn advance(&mut self, ms: u64, parked: bool, ready: bool) {
        self.restrict(parked, ready);
        if !self.page.pending() && !self.page.result() {
            return;
        }
        self.elapsed_ms = self.elapsed_ms.saturating_add(ms);
        if self.page == BtPage::Discoverable {
            let incoming = matches!(
                self.fixture,
                BluetoothFixture::Ssp | BluetoothFixture::Legacy | BluetoothFixture::PairFails
            );
            if incoming && self.elapsed_ms >= 1500 {
                self.show(if self.fixture == BluetoothFixture::Legacy {
                    BtPage::LegacyConfirm
                } else {
                    BtPage::NumericConfirm
                });
            } else if self.elapsed_ms >= 180_000 {
                self.timed_out = true;
                self.show(BtPage::PairFailed);
            }
            return;
        }
        if matches!(self.page, BtPage::NumericConfirm | BtPage::LegacyConfirm) {
            if self.elapsed_ms >= 30_000 {
                self.timed_out = true;
                self.show(BtPage::PairFailed);
            }
            return;
        }
        let limit = if self.page.result() || self.page == BtPage::PairSuccess {
            3000
        } else {
            1500
        };
        if self.elapsed_ms >= limit {
            self.complete();
        }
    }
    fn complete(&mut self) {
        use BtPage::{
            ConnectFailed, Connected, Connecting, DeleteFailed, Deleted, Deleting,
            DisconnectFailed, Disconnected, Disconnecting, Manager, PairFailed, PairSuccess,
            Pairing, PrePairFailed, PreparingPair,
        };
        let next = match self.page {
            PreparingPair if self.fixture == BluetoothFixture::DisconnectFails => PrePairFailed,
            PreparingPair => {
                for d in &mut self.devices {
                    d.roles = 0;
                }
                BtPage::Discoverable
            }
            Pairing if self.fixture == BluetoothFixture::PairFails => PairFailed,
            Pairing => {
                if let Some(index) = self
                    .devices
                    .iter()
                    .position(|d| d.name == "Simulated Phone")
                {
                    self.target = index;
                } else {
                    self.target = self.devices.len();
                    self.devices.push(BtDevice {
                        name: "Simulated Phone",
                        phone_supported: true,
                        roles: 0,
                    });
                }
                self.desired_roles = 3;
                PairSuccess
            }
            PairSuccess => Connecting,
            Connecting if self.fixture == BluetoothFixture::ConnectFails => ConnectFailed,
            Connecting => {
                for d in &mut self.devices {
                    d.roles &= !self.desired_roles;
                }
                if let Some(d) = self.devices.get_mut(self.target) {
                    d.roles = self.desired_roles;
                }
                Connected
            }
            Disconnecting if self.fixture == BluetoothFixture::DisconnectFails => DisconnectFailed,
            Disconnecting => {
                if let Some(d) = self.devices.get_mut(self.target) {
                    d.roles = 0;
                }
                Disconnected
            }
            Deleting if self.fixture == BluetoothFixture::DeleteFails => DeleteFailed,
            Deleting => {
                self.result_name = if self.all { "" } else { self.name() };
                if self.all {
                    self.devices.clear();
                } else if self.target < self.devices.len() {
                    self.devices.remove(self.target);
                }
                self.target = 0;
                Deleted
            }
            _ => Manager,
        };
        self.show(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn select(session: &mut BluetoothSession) {
        session.event(CommanderEvent::Select, true, true);
    }
    fn configured(fixture: BluetoothFixture) -> BluetoothSession {
        let mut session = BluetoothSession::default();
        session.configure(fixture);
        session.show(BtPage::Manager);
        session
    }
    #[test]
    fn manager_off_keeps_pair_records_and_hides_devices_and_disables_add() {
        let mut session = configured(BluetoothFixture::Paired);
        assert_eq!(session.row_count(), 5);
        select(&mut session);
        assert!(!session.enabled);
        assert_eq!(session.row_count(), 3);
        assert_eq!(session.devices.len(), 2);
        assert!(session.devices.iter().all(|d| d.roles == 0));
        assert!(!session.row_enabled(2, true, true));
        select(&mut session);
        assert!(session.enabled);
        assert_eq!(session.row_count(), 5);
    }
    #[test]
    fn phone_and_audio_roles_are_independent_and_unsupported_rows_are_skipped() {
        let mut session = configured(BluetoothFixture::Paired);
        session.target = 1;
        session.show(BtPage::Profiles);
        session.event(CommanderEvent::Select, true, true); // skip unsupported phone rows
        assert_eq!(session.page, BtPage::Connecting);
        assert_eq!(session.desired_roles, 2);
        session.advance(1499, true, true);
        assert_eq!(session.devices[0].roles, 3);
        session.advance(1, true, true);
        assert_eq!(session.page, BtPage::Connected);
        assert_eq!(session.devices[0].roles, 1);
        assert_eq!(session.devices[1].roles, 2);
        session.advance(3000, true, true);
        session.target = 0;
        session.show(BtPage::Profiles);
        session.focus = 3;
        select(&mut session);
        session.advance(1500, true, true);
        assert_eq!(session.page, BtPage::Disconnected);
        assert_eq!(session.devices[1].roles, 2);
    }
    #[test]
    fn ssp_requires_confirmation_and_pairing_and_connection_complete_separately() {
        let mut session = configured(BluetoothFixture::Ssp);
        session.focus = 2;
        select(&mut session);
        session.advance(1500, true, true);
        assert_eq!(session.page, BtPage::NumericConfirm);
        assert_eq!(session.focus, 2);
        session.event(CommanderEvent::RotateCounterClockwise, true, true);
        select(&mut session);
        assert_eq!(session.page, BtPage::Cancelling);
        session.advance(1500, true, true);
        assert!(session.devices.is_empty());
        session.focus = 2;
        select(&mut session);
        session.advance(1500, true, true);
        select(&mut session);
        session.advance(1499, true, true);
        assert!(session.devices.is_empty());
        session.advance(1, true, true);
        assert_eq!(session.page, BtPage::PairSuccess);
        assert_eq!(session.devices[0].roles, 0);
        session.advance(3000, true, true);
        assert_eq!(session.page, BtPage::Connecting);
        session.advance(1500, true, true);
        assert_eq!(session.page, BtPage::Connected);
        assert_eq!(session.devices[0].roles, 3);
    }
    #[test]
    fn legacy_pairing_only_offers_cancel_and_never_accepts_on_the_head_unit() {
        for cancel in [CommanderEvent::Select, CommanderEvent::Back] {
            let mut session = configured(BluetoothFixture::Legacy);
            session.focus = 2;
            select(&mut session);
            assert_eq!(session.page, BtPage::Discoverable);
            session.advance(1499, true, true);
            assert_eq!(session.page, BtPage::Discoverable);
            session.advance(1, true, true);
            assert_eq!(session.page, BtPage::LegacyConfirm);
            assert_eq!(session.page.button_count(), 1);
            assert_eq!(session.focus, 0);
            session.rotate(true, true, true);
            session.rotate(false, true, true);
            assert_eq!(session.focus, 0);
            session.advance(29_999, true, true);
            assert_eq!(session.page, BtPage::LegacyConfirm);
            assert!(session.devices.is_empty());
            session.event(cancel, true, true);
            assert_eq!(session.page, BtPage::Cancelling);
            session.advance(1500, true, true);
            assert_eq!(session.page, BtPage::Manager);
            assert!(session.devices.is_empty());
            assert_eq!(session.pair_code, "0000");
        }
    }
    #[test]
    fn search_and_numeric_confirmation_timeout_without_adding_a_device() {
        for (page, limit) in [
            (BtPage::Discoverable, 180_000),
            (BtPage::NumericConfirm, 30_000),
            (BtPage::LegacyConfirm, 30_000),
        ] {
            let mut session = configured(BluetoothFixture::Empty);
            session.show(page);
            session.advance(limit - 1, true, true);
            assert_eq!(session.page, page);
            session.advance(1, true, true);
            assert_eq!(session.page, BtPage::PairFailed);
            assert_eq!(session.focus, 1);
            assert!(session.devices.is_empty());
        }
    }
    #[test]
    fn unpair_no_cancel_and_failed_attempts_preserve_records_until_success() {
        let mut session = configured(BluetoothFixture::DeleteFails);
        session.target = 1;
        session.show(BtPage::ConfirmDelete);
        assert_eq!(session.focus, 0);
        select(&mut session);
        assert_eq!(session.devices.len(), 2);
        session.show(BtPage::ConfirmDelete);
        session.focus = 1;
        select(&mut session);
        session.advance(1500, true, true);
        assert_eq!(session.page, BtPage::DeleteFailed);
        assert_eq!(session.devices.len(), 2);
        session.fixture = BluetoothFixture::Paired;
        select(&mut session);
        session.advance(1500, true, true);
        assert_eq!(session.page, BtPage::Deleted);
        assert_eq!(session.result_name, "Simulated Player");
        assert_eq!(session.devices.len(), 1);
        session.show(BtPage::DeleteList);
        session.focus = 0;
        select(&mut session);
        select(&mut session);
        assert_eq!(session.page, BtPage::DeleteList);
        assert_eq!(session.focus, 0);
        session.all = true;
        session.show(BtPage::ConfirmDelete);
        session.focus = 1;
        select(&mut session);
        session.advance(1500, true, true);
        assert!(session.devices.is_empty());
        assert_eq!(session.result_name, "");
    }
    #[test]
    fn failed_connect_disconnect_and_pre_pair_do_not_change_profile_ownership() {
        for (fixture, start, end) in [
            (
                BluetoothFixture::ConnectFails,
                BtPage::Connecting,
                BtPage::ConnectFailed,
            ),
            (
                BluetoothFixture::DisconnectFails,
                BtPage::Disconnecting,
                BtPage::DisconnectFailed,
            ),
            (
                BluetoothFixture::DisconnectFails,
                BtPage::PreparingPair,
                BtPage::PrePairFailed,
            ),
        ] {
            let mut session = configured(fixture);
            session.show(start);
            session.advance(u64::MAX, true, true);
            assert_eq!(session.page, end);
            assert_eq!(session.devices[0].roles, 3);
            assert_eq!(session.devices[1].roles, 0);
            select(&mut session);
            assert_eq!(session.page, start);
        }
        let mut session = configured(BluetoothFixture::PairFails);
        session.show(BtPage::Pairing);
        session.advance(1500, true, true);
        assert_eq!(session.page, BtPage::PairFailed);
        assert!(session.devices.is_empty());
    }
    #[test]
    fn loss_of_park_or_services_cancels_pending_operations_without_late_mutation() {
        for page in [
            BtPage::Discoverable,
            BtPage::NumericConfirm,
            BtPage::LegacyConfirm,
            BtPage::Pairing,
            BtPage::PairSuccess,
            BtPage::Connecting,
            BtPage::Disconnecting,
            BtPage::Deleting,
            BtPage::PreparingPair,
        ] {
            for (parked, ready) in [(false, true), (true, false)] {
                let mut session = configured(BluetoothFixture::Paired);
                session.show(page);
                session.advance(u64::MAX, parked, ready);
                assert_eq!(session.page, BtPage::Manager);
                assert_eq!(session.devices.len(), 2);
                assert_eq!(session.devices[0].roles, 3);
                session.advance(u64::MAX, true, true);
                assert_eq!(session.devices[0].roles, 3);
            }
        }
    }
    #[test]
    fn pin_editor_bounds_four_digits_requires_ok_and_cancel_preserves_the_code() {
        let mut session = configured(BluetoothFixture::Empty);
        session.show(BtPage::Keypad);
        assert_eq!(session.focus, 1);
        session.focus = 0;
        for _ in 0..6 {
            select(&mut session);
        }
        assert_eq!(session.draft_code, "");
        session.focus = 12;
        select(&mut session);
        assert_eq!(session.page, BtPage::Keypad);
        for digit in [1, 2, 3, 4, 5] {
            session.focus = digit;
            select(&mut session);
        }
        assert_eq!(session.draft_code, "1234");
        assert_eq!(session.pair_code, "0000");
        session.focus = 10;
        select(&mut session);
        assert_eq!(session.pair_code, "0000");
        session.show(BtPage::Keypad);
        session.draft_code = "1234".into();
        session.focus = 12;
        select(&mut session);
        assert_eq!(session.pair_code, "1234");
        session.show(BtPage::Keypad);
        session.draft_code = "9999".into();
        session.event(CommanderEvent::Select, false, true);
        assert_eq!(session.page, BtPage::Settings);
        assert_eq!(session.pair_code, "1234");
    }
}
