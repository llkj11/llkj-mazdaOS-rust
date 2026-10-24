//! Host-only state model of sysupdate's music database contexts. No updater I/O.
use mazda_core::CommanderEvent;

/// Synthetic service responses. None represent attached media or a car version.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MusicUpdateFixture {
    #[default]
    NoMedia,
    Available,
    SearchFails,
    Invalid,
    InstallFails,
    RetryInvalid,
    Unavailable,
    NeedsFailsafe,
}
impl MusicUpdateFixture {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "no-media" => Self::NoMedia,
            "available" => Self::Available,
            "search-fails" => Self::SearchFails,
            "invalid" => Self::Invalid,
            "install-fails" => Self::InstallFails,
            "retry-invalid" => Self::RetryInvalid,
            "unavailable" => Self::Unavailable,
            "needs-failsafe" => Self::NeedsFailsafe,
            _ => return None,
        })
    }
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MusicPage {
    #[default]
    Closed,
    Search,
    Searching,
    Packages,
    Confirm,
    Preparing,
    Installing,
    Retrying,
    Success,
    SearchFailed,
    NoPackages,
    ValidationFailed,
    RetryValidationFailed,
    Unavailable,
    InstallFailed,
    NeedsFailsafe,
}
impl MusicPage {
    pub const fn busy(self) -> bool {
        matches!(
            self,
            Self::Searching | Self::Preparing | Self::Installing | Self::Retrying
        )
    }
    pub const fn full_screen(self) -> bool {
        matches!(
            self,
            Self::Installing | Self::Retrying | Self::Success | Self::Unavailable
        )
    }
    pub const fn two_buttons(self) -> bool {
        matches!(
            self,
            Self::Search | Self::Confirm | Self::SearchFailed | Self::InstallFailed
        )
    }
}
#[derive(Debug, Default, Clone)]
pub(crate) struct MusicUpdateSession {
    pub page: MusicPage,
    pub fixture: MusicUpdateFixture,
    /// Zero-based Dialog3 focus, or List2 current/package row.
    pub focus: usize,
    pub elapsed_ms: u64,
    pub installed: bool,
}
impl MusicUpdateSession {
    pub const VERSION: &'static str = "SIM-1";
    pub fn open(&mut self) {
        self.show(MusicPage::Search);
    }
    fn show(&mut self, page: MusicPage) {
        self.page = page;
        self.elapsed_ms = 0;
        self.focus = usize::from(page.two_buttons()); // source initialFocus: 1
    }
    pub fn restrict(&mut self, parked: bool, services: bool) {
        if (!parked || !services) && self.page.busy() {
            self.show(MusicPage::Search);
            self.focus = 0;
        }
        if (!parked || !services) && (self.page.two_buttons() || self.page == MusicPage::Packages) {
            self.focus = 0;
        }
    }
    pub fn event(&mut self, event: CommanderEvent, parked: bool, services: bool) -> bool {
        use CommanderEvent::{Back, RotateClockwise, RotateCounterClockwise, Select};
        use MusicPage::{
            Closed, Confirm, Installing, Packages, Preparing, Retrying, Search, SearchFailed,
        };
        if self.page == Closed {
            return false;
        }
        self.restrict(parked, services);
        match event {
            Back => match self.page {
                Confirm => self.show(Packages),
                Packages | SearchFailed => self.show(Search),
                Preparing | Installing | Retrying => {}
                _ => self.show(Closed),
            },
            RotateClockwise | RotateCounterClockwise => {
                if self.page.two_buttons() || self.page == Packages {
                    self.focus = usize::from(event == RotateClockwise && parked && services);
                }
            }
            Select => self.select(parked && services),
            _ => {}
        }
        true
    }
    fn select(&mut self, allowed: bool) {
        use MusicPage::{
            Closed, Confirm, InstallFailed, Installing, Packages, Preparing, Retrying, Search,
            SearchFailed, Searching,
        };
        match self.page {
            Search | SearchFailed => {
                if self.focus == 0 {
                    self.show(Closed);
                } else if allowed {
                    self.show(Searching);
                }
            }
            Packages if self.focus == 1 && allowed => self.show(Confirm),
            Confirm => {
                if self.focus == 0 {
                    self.show(Packages);
                } else if allowed {
                    self.show(Preparing);
                }
            }
            InstallFailed => {
                if self.focus == 0 {
                    self.show(Closed);
                } else if allowed {
                    self.show(Retrying);
                }
            }
            Closed | Packages | Searching | Preparing | Installing | Retrying => {}
            _ => self.show(Closed),
        }
    }
    pub fn advance(&mut self, ms: u64, parked: bool, services: bool) {
        use MusicPage::{Installing, Preparing, Retrying, Searching};
        self.restrict(parked, services);
        if !self.page.busy() {
            return;
        }
        self.elapsed_ms = self.elapsed_ms.saturating_add(ms);
        let delay = match self.page {
            Searching => 1200,
            Preparing => 800,
            _ => 4000,
        };
        if self.elapsed_ms < delay {
            return;
        }
        let next = match self.page {
            Searching => match self.fixture {
                MusicUpdateFixture::NoMedia => MusicPage::NoPackages,
                MusicUpdateFixture::SearchFails => MusicPage::SearchFailed,
                MusicUpdateFixture::NeedsFailsafe => MusicPage::NeedsFailsafe,
                _ => MusicPage::Packages,
            },
            Preparing => match self.fixture {
                MusicUpdateFixture::Invalid => MusicPage::ValidationFailed,
                MusicUpdateFixture::Unavailable => MusicPage::Unavailable,
                _ => Installing,
            },
            Installing
                if matches!(
                    self.fixture,
                    MusicUpdateFixture::InstallFails | MusicUpdateFixture::RetryInvalid
                ) =>
            {
                MusicPage::InstallFailed
            }
            Retrying if self.fixture == MusicUpdateFixture::RetryInvalid => {
                MusicPage::RetryValidationFailed
            }
            Installing | Retrying => {
                self.installed = true;
                MusicPage::Success
            }
            _ => unreachable!("busy contexts only"),
        };
        // A large elapsed step never skips visible intermediate contexts.
        self.show(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn select(s: &mut MusicUpdateSession) {
        s.event(CommanderEvent::Select, true, true);
    }
    fn install(s: &mut MusicUpdateSession) {
        s.open();
        select(s);
        s.advance(1200, true, true);
        s.event(CommanderEvent::RotateClockwise, true, true);
        select(s);
        select(s);
        s.advance(800, true, true);
    }
    #[test]
    fn normal_search_uses_no_media_and_never_claims_a_version() {
        let mut s = MusicUpdateSession::default();
        s.open();
        assert_eq!(s.focus, 1);
        select(&mut s);
        s.advance(1199, true, true);
        assert_eq!(s.page, MusicPage::Searching);
        s.advance(1, true, true);
        assert_eq!(s.page, MusicPage::NoPackages);
        assert!(!s.installed);
        select(&mut s);
        assert_eq!(s.page, MusicPage::Closed);
    }
    #[test]
    fn confirmation_back_and_success_mutate_only_the_mock_version() {
        let mut s = MusicUpdateSession {
            fixture: MusicUpdateFixture::Available,
            ..Default::default()
        };
        s.open();
        select(&mut s);
        s.advance(u64::MAX, true, true);
        select(&mut s);
        assert_eq!(s.page, MusicPage::Packages); // current row inert
        s.event(CommanderEvent::RotateClockwise, true, true);
        select(&mut s);
        assert_eq!(s.page, MusicPage::Confirm);
        assert_eq!(s.focus, 1);
        s.event(CommanderEvent::Back, true, true);
        assert_eq!(s.page, MusicPage::Packages);
        install(&mut s);
        assert_eq!(s.page, MusicPage::Installing);
        s.event(CommanderEvent::Back, true, true);
        assert_eq!(s.page, MusicPage::Installing);
        s.advance(3999, true, true);
        assert!(!s.installed);
        s.advance(1, true, true);
        assert_eq!(s.page, MusicPage::Success);
        assert!(s.installed);
    }
    #[test]
    fn fixture_failures_retry_and_cancel_preserve_current_version() {
        for (fixture, expected) in [
            (MusicUpdateFixture::Invalid, MusicPage::ValidationFailed),
            (MusicUpdateFixture::Unavailable, MusicPage::Unavailable),
        ] {
            let mut s = MusicUpdateSession {
                fixture,
                ..Default::default()
            };
            install(&mut s);
            assert_eq!(s.page, expected);
            assert!(!s.installed);
        }
        for fixture in [
            MusicUpdateFixture::InstallFails,
            MusicUpdateFixture::RetryInvalid,
        ] {
            let mut s = MusicUpdateSession {
                fixture,
                ..Default::default()
            };
            install(&mut s);
            s.advance(4000, true, true);
            assert_eq!(s.page, MusicPage::InstallFailed);
            assert!(!s.installed);
            select(&mut s);
            assert_eq!(s.page, MusicPage::Retrying);
            s.advance(4000, true, true);
            assert_eq!(
                s.page,
                if fixture == MusicUpdateFixture::RetryInvalid {
                    MusicPage::RetryValidationFailed
                } else {
                    MusicPage::Success
                }
            );
        }
        let mut s = MusicUpdateSession {
            fixture: MusicUpdateFixture::SearchFails,
            ..Default::default()
        };
        s.open();
        select(&mut s);
        s.advance(1200, true, true);
        assert_eq!(s.page, MusicPage::SearchFailed);
        select(&mut s);
        assert_eq!(s.page, MusicPage::Searching);
        s.event(CommanderEvent::Back, true, true);
        assert_eq!(s.page, MusicPage::Closed);
    }
    #[test]
    fn policy_loss_cancels_every_pending_mutation_and_bounds_timer() {
        for page in [
            MusicPage::Searching,
            MusicPage::Preparing,
            MusicPage::Installing,
            MusicPage::Retrying,
        ] {
            for (parked, services) in [(false, true), (true, false)] {
                let mut s = MusicUpdateSession {
                    page,
                    fixture: MusicUpdateFixture::Available,
                    ..Default::default()
                };
                s.advance(u64::MAX, parked, services);
                assert_eq!(s.page, MusicPage::Search);
                assert_eq!(s.focus, 0);
                assert!(!s.installed);
                s.event(CommanderEvent::RotateClockwise, parked, services);
                select(&mut s);
                assert_eq!(s.page, MusicPage::Closed);
            }
        }
        let mut packages = MusicUpdateSession {
            page: MusicPage::Packages,
            focus: 1,
            installed: true,
            ..Default::default()
        };
        packages.event(CommanderEvent::Select, false, true);
        assert_eq!(packages.page, MusicPage::Packages);
        assert_eq!(packages.focus, 0);
        assert!(packages.installed);
    }
}
