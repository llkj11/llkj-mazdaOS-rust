//! Native port of the owner's NA 70.00.100A home/menu geometry.
//! Source mapping is documented in `docs/STOCK_UI_PORT.md`. No firmware JS executes.
use crate::clock::{ClockPage, ClockSession, CLOCK_ROWS};
use crate::languages::{LanguagePage, LanguageSession};
use crate::music_update::{MusicPage, MusicUpdateSession};
use crate::settings::DisplaySession;
use crate::sound::SoundSession;
use crate::system_settings::{SystemPage, SystemSession};
use crate::{
    fit_text, text_width, Color, DisplayPower, DisplayPreferences, Renderer, SoundPreferences,
    SoundProfile, StockAssets,
};
use mazda_core::{CommanderEvent, MazdaReadOnly, MediaState, VehicleSnapshot};
use std::sync::Arc;
#[path = "clock_ui.rs"]
mod clock_ui;
#[path = "language_ui.rs"]
mod language_ui;
#[path = "music_update_ui.rs"]
mod music_update_ui;
#[path = "sound_ui.rs"]
mod sound_ui;
#[path = "system_ui.rs"]
mod system_ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaIntent {
    Previous,
    TogglePlayback,
    Next,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Applications,
    Entertainment,
    Communication,
    Navigation,
    Settings,
}
impl Screen {
    pub const DOMAINS: [Self; 5] = [
        Self::Applications,
        Self::Entertainment,
        Self::Communication,
        Self::Navigation,
        Self::Settings,
    ];
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Home => "",
            Self::Applications => "Applications",
            Self::Entertainment => "Entertainment",
            Self::Communication => "Communication",
            Self::Navigation => "Navigation",
            Self::Settings => "Settings",
        }
    }
}
// Positions from MainMenuCtrl.css. Focus rectangles include the CSS transform about center.
type ImageRect = (i32, i32, usize, usize);
type Coin = (&'static str, ImageRect, ImageRect, (i32, i32));
const COINS: [Coin; 5] = [
    (
        "Apps",
        (-35, 336, 114, 122),
        (-3, 284, 137, 146),
        (-124, 132),
    ),
    ("Audio", (105, 228, 142, 152), (91, 201, 170, 182), (-6, 77)),
    ("Com", (330, 197, 142, 152), (316, 170, 170, 182), (220, 46)),
    ("Nav", (549, 228, 142, 152), (535, 201, 170, 182), (439, 77)),
    (
        "Set",
        (721, 336, 114, 122),
        (666, 284, 137, 146),
        (561, 132),
    ),
];
// Read from systemApp's visible master lists and en_US localization.
const APPLICATIONS: [&str; 3] = ["Fuel Economy Monitor", "Warning Guidance", "Maintenance"];
const ENTERTAINMENT: [&str; 7] = ["FM", "AM", "Stitcher", "Bluetooth", "USB 1", "USB 2", "AUX"];
const COMMUNICATION: [&str; 7] = [
    "Notifications",
    "Favorites",
    "Contacts",
    "Call History",
    "Dial Phone",
    "Text Messages",
    "Settings",
];
const SETTINGS: [&str; 8] = [
    "AD-Disp", "Display", "Safety", "Sound", "Clock", "Vehicle", "Devices", "System",
];
const DISPLAY: [&str; 6] = [
    "Turn Display Off",
    "Turn Display Off and Show Clock",
    "System",
    "Brightness",
    "Contrast",
    "Reset",
];
#[derive(Debug, Clone, Copy)]
enum TabFavor {
    Second,
    Third,
}

#[derive(Debug, Clone)]
pub struct UiModel {
    screen: Screen,
    home_focus: usize,
    row: usize,
    assets: Arc<StockAssets>,
    vehicle: VehicleSnapshot,
    media: MediaState,
    settings_rows: bool,
    settings_tab: usize,
    hud_installed: bool,
    tab_favor: TabFavor,
    notice: Option<&'static str>,
    display: DisplaySession,
    sound: SoundSession,
    clock: ClockSession,
    system: SystemSession,
    language: LanguageSession,
    music_update: MusicUpdateSession,
}
impl UiModel {
    #[must_use]
    pub fn from_source(source: &impl MazdaReadOnly) -> Self {
        Self {
            screen: Screen::Home,
            home_focus: 2,
            row: 0,
            assets: Arc::new(StockAssets::default()),
            vehicle: source.vehicle_snapshot(),
            media: source.media_state(),
            settings_rows: false,
            settings_tab: 1,
            hud_installed: false,
            tab_favor: TabFavor::Second,
            notice: None,
            display: DisplaySession::default(),
            sound: SoundSession::default(),
            clock: ClockSession::default(),
            system: SystemSession::default(),
            language: LanguageSession::default(),
            music_update: MusicUpdateSession::default(),
        }
    }
    pub fn set_assets(&mut self, assets: StockAssets) {
        if !assets.clock_zones.is_empty() {
            self.clock.values.time_zone = self
                .clock
                .values
                .time_zone
                .min(assets.clock_zones.len() - 1);
        }
        self.assets = Arc::new(assets);
        self.language.current = self
            .language
            .current
            .min(self.assets.languages.packs.len().saturating_sub(1));
        self.language.page = LanguagePage::Closed;
        self.music_update.page = MusicPage::Closed;
    }
    pub fn refresh(&mut self, source: &impl MazdaReadOnly) {
        self.vehicle = source.vehicle_snapshot();
        self.media = source.media_state();
        self.clock.restrict(self.vehicle.is_confirmed_parked());
        self.system.restrict(self.vehicle.is_confirmed_parked());
        self.language.restrict(self.vehicle.is_confirmed_parked());
        self.music_update.restrict(
            self.vehicle.is_confirmed_parked(),
            self.system.equipment.services_ready,
        );
        if !self.vehicle.is_confirmed_parked() {
            self.display.cancel_interaction();
            self.sound.cancel_interaction();
        } else if self.notice == Some("Preferences locked: valid zero speed and Park required") {
            self.notice = None;
        }
    }
    #[must_use]
    pub const fn screen(&self) -> Screen {
        self.screen
    }
    #[must_use]
    pub fn selection(&self) -> Screen {
        Screen::DOMAINS[self.home_focus]
    }
    #[must_use]
    pub const fn selected_row(&self) -> usize {
        self.row
    }
    #[must_use]
    pub const fn notice(&self) -> Option<&'static str> {
        self.notice
    }

    #[must_use]
    pub const fn display_preferences(&self) -> DisplayPreferences {
        self.display.values
    }
    #[must_use]
    pub const fn display_adjusting(&self) -> bool {
        self.display.editing.is_some()
    }
    #[must_use]
    pub const fn sound_preferences(&self) -> SoundPreferences {
        self.sound.values
    }
    #[must_use]
    pub const fn sound_profile(&self) -> SoundProfile {
        self.sound.profile
    }
    #[must_use]
    pub const fn sound_adjusting(&self) -> bool {
        self.sound.editing.is_some()
    }
    #[must_use]
    pub const fn clock_preferences(&self) -> crate::ClockPreferences {
        self.clock.values
    }
    pub fn set_clock_navigation_available(&mut self, available: bool) {
        self.clock.navigation_available = available;
        self.clock.restrict(self.vehicle.is_confirmed_parked());
    }
    pub fn set_clock_gps_sync(&mut self, enabled: bool) {
        self.clock.values.gps_sync = enabled;
        self.clock.restrict(self.vehicle.is_confirmed_parked());
    }
    pub fn set_sound_profile(&mut self, profile: SoundProfile) {
        self.sound.cancel_interaction();
        self.sound.profile = profile;
        if self.screen == Screen::Settings && self.settings_tab == 3 {
            self.row = 0;
        }
    }
    pub fn set_guidance_available(&mut self, enabled: bool) {
        self.sound.guidance_enabled = enabled;
        if !enabled && self.sound.editing == Some(crate::SoundControl::Guidance) {
            self.sound.cancel_interaction();
        } else if self.notice == Some("Guidance Volume is disabled by the host service fixture") {
            self.notice = None;
        }
    }
    pub fn set_hud_installed(&mut self, installed: bool) {
        self.hud_installed = installed;
        if !installed && self.settings_tab == 0 {
            self.settings_tab = 1;
            self.settings_rows = false;
            self.row = 0;
        }
    }
    fn settings_tabs(&self) -> &'static [usize] {
        if self.hud_installed {
            &[0, 1, 2, 3, 4, 5, 6, 7]
        } else {
            &[1, 2, 3, 4, 5, 6, 7]
        }
    }
    fn first_visible_tab(&self) -> usize {
        self.settings_tabs()
            .iter()
            .position(|index| *index == self.settings_tab)
            .unwrap_or(0)
            .saturating_sub(match self.tab_favor {
                TabFavor::Second => 1,
                TabFavor::Third => 2,
            })
            .min(self.settings_tabs().len() - 4)
    }
    fn rotate_settings_tab(&mut self, forward: bool) {
        self.tab_favor = if forward {
            TabFavor::Third
        } else {
            TabFavor::Second
        };
        self.settings_tab = if forward {
            (self.settings_tab + 1).min(7)
        } else {
            self.settings_tab
                .saturating_sub(1)
                .max(usize::from(!self.hud_installed))
        };
    }
    fn pointer_settings_tab(&mut self, x: usize) {
        self.display.cancel_interaction();
        self.sound.cancel_interaction();
        let start = self.first_visible_tab();
        if (52..136).contains(&x) {
            if start > 0 {
                self.settings_tab = self.settings_tabs()[start - 1];
                self.tab_favor = TabFavor::Third;
            }
        } else if (708..794).contains(&x) {
            if start + 4 < self.settings_tabs().len() {
                self.settings_tab = self.settings_tabs()[start + 4];
                self.tab_favor = TabFavor::Second;
            }
        } else if (136..708).contains(&x) {
            let target = self.settings_tabs()[start + (x - 136) / 143];
            if target != self.settings_tab {
                self.tab_favor = if target > self.settings_tab {
                    TabFavor::Third
                } else {
                    TabFavor::Second
                };
            }
            self.settings_tab = target;
        }
        self.settings_rows = false;
    }
    pub fn handle_pointer_drag(&mut self, x: usize, _y: usize) {
        if self.sound.pointer_drag {
            if let Some(control) = self.sound.editing {
                self.sound
                    .pointer_value(control, x, self.vehicle.is_confirmed_parked());
            }
        }
        if self.display.pointer_drag {
            if let Some(row) = self.display.editing {
                self.display
                    .pointer_value(row, x, self.vehicle.is_confirmed_parked());
            }
        }
    }
    pub fn handle_pointer_release(&mut self) {
        if self.sound.pointer_drag {
            self.sound.cancel_interaction();
        }
        if self.display.pointer_drag {
            self.display.editing = None;
            self.display.pointer_drag = false;
        }
    }
    fn open(&mut self, screen: Screen) {
        self.display.cancel_interaction();
        self.sound.cancel_interaction();
        self.clock.page = ClockPage::Root;
        self.system.page = SystemPage::Root;
        self.language.page = LanguagePage::Closed;
        self.music_update.page = MusicPage::Closed;
        self.screen = screen;
        self.row = 0;
        self.settings_rows = false;
        self.notice = if screen == Screen::Navigation {
            Some("Host preview: Mazda Compass fallback; maps and GPS are not connected")
        } else {
            None
        };
        if let Some(index) = Screen::DOMAINS.iter().position(|s| *s == screen) {
            self.home_focus = index;
        }
    }
    fn rows(&self) -> &'static [&'static str] {
        match self.screen {
            Screen::Applications => &APPLICATIONS,
            Screen::Entertainment => &ENTERTAINMENT,
            Screen::Communication => &COMMUNICATION,
            Screen::Settings if self.settings_tab == 1 => &DISPLAY,
            Screen::Settings if self.settings_tab == 3 => self.sound.profile.labels(),
            Screen::Settings if self.settings_tab == 4 => &CLOCK_ROWS,
            Screen::Settings if self.settings_tab == 7 => self.system.labels(),
            _ => &[],
        }
    }
    fn first_visible_row(&self) -> usize {
        self.row.saturating_sub(
            if self.screen == Screen::Settings && self.settings_tab == 1 {
                2
            } else if self.screen == Screen::Settings {
                4
            } else {
                5
            },
        )
    }
    fn row_enabled(&self, row: usize) -> bool {
        let parked = self.vehicle.is_confirmed_parked();
        match (self.screen, self.settings_tab) {
            (Screen::Settings, 3) => self
                .sound
                .profile
                .controls()
                .get(row)
                .is_some_and(|&control| self.sound.allowed(control, parked)),
            (Screen::Settings, 4) => self.clock.enabled(row, parked),
            (Screen::Settings, 7) => self
                .system
                .controls()
                .get(row)
                .is_some_and(|&control| self.system.allowed(control, parked)),
            (Screen::Settings, _) => parked,
            _ => true,
        }
    }
    fn rotate_row(&mut self, forward: bool) {
        let next = if forward {
            (self.row + 1..self.rows().len()).find(|&row| self.row_enabled(row))
        } else {
            (0..self.row).rev().find(|&row| self.row_enabled(row))
        };
        if let Some(next) = next {
            self.row = next;
        }
    }
    pub fn handle_commander(&mut self, event: CommanderEvent) -> Option<MediaIntent> {
        self.notice = None;
        if self.display.values.power != DisplayPower::On {
            self.display.values.power = DisplayPower::On;
            if event == CommanderEvent::Home {
                self.open(Screen::Home);
            }
            return None;
        }
        if self.screen == Screen::Settings
            && !matches!(
                event,
                CommanderEvent::Home
                    | CommanderEvent::Music
                    | CommanderEvent::Navigation
                    | CommanderEvent::Favorites
            )
            && self.settings_active_event(event)
        {
            return None;
        }
        match event {
            CommanderEvent::Home => {
                self.open(Screen::Home);
                return None;
            }
            CommanderEvent::Music => {
                self.open(Screen::Entertainment);
                return None;
            }
            CommanderEvent::Navigation => {
                self.open(Screen::Navigation);
                return None;
            }
            // Favorite storage/services are not ported. Do not relabel Favorites as Settings.
            CommanderEvent::Favorites => {
                self.notice = Some("Favorites service is not connected");
                return None;
            }
            CommanderEvent::Back => {
                if self.screen == Screen::Settings && self.settings_rows {
                    self.settings_rows = false;
                } else {
                    self.open(Screen::Home);
                }
                return None;
            }
            _ => {}
        }
        let clockwise = event == CommanderEvent::RotateClockwise;
        let counterclockwise = event == CommanderEvent::RotateCounterClockwise;
        if self.screen == Screen::Home {
            if clockwise {
                self.home_focus = (self.home_focus + 1).min(4);
            }
            if counterclockwise {
                self.home_focus = self.home_focus.saturating_sub(1);
            }
            if event == CommanderEvent::Select {
                self.open(self.selection());
            }
        } else if self.screen == Screen::Settings && !self.settings_rows {
            if clockwise {
                self.rotate_settings_tab(true);
            }
            if counterclockwise {
                self.rotate_settings_tab(false);
            }
            if event == CommanderEvent::Select {
                if self.rows().is_empty() {
                    self.notice = Some("This settings tab has not been ported");
                } else if let Some(first) =
                    (0..self.rows().len()).find(|&row| self.row_enabled(row))
                {
                    self.settings_rows = true;
                    self.row = first;
                } else {
                    self.notice = Some("Preferences locked: valid zero speed and Park required");
                }
            }
        } else if !self.rows().is_empty() {
            if clockwise {
                self.rotate_row(true);
            }
            if counterclockwise {
                self.rotate_row(false);
            }
            if event == CommanderEvent::Select {
                self.select_row();
            }
        }
        None
    }
    fn settings_active_event(&mut self, event: CommanderEvent) -> bool {
        let parked = self.vehicle.is_confirmed_parked();
        if self.settings_tab == 7
            && self
                .music_update
                .event(event, parked, self.system.equipment.services_ready)
        {
            return true;
        }
        if self.settings_tab == 7
            && self
                .language
                .event(event, parked, self.assets.languages.packs.len())
        {
            return true;
        }
        match self.settings_tab {
            1 => self.display.active_event(event, parked),
            3 => self.sound.active_event(event, parked),
            4 => self
                .clock
                .active_event(event, parked, self.assets.clock_zones.len()),
            7 => self
                .system
                .active_event(event, parked, self.legal_max_scroll()),
            _ => false,
        }
    }
    fn select_row(&mut self) {
        if self.screen == Screen::Settings
            && self.settings_tab == 7
            && self.system.controls().get(self.row) == Some(&crate::SystemControl::MusicDatabase)
            && self.vehicle.is_confirmed_parked()
        {
            self.music_update.open();
            self.music_update
                .restrict(true, self.system.equipment.services_ready);
            self.notice = None;
            return;
        }
        if self.screen == Screen::Settings
            && self.settings_tab == 7
            && self.system.controls().get(self.row) == Some(&crate::SystemControl::Language)
            && self.vehicle.is_confirmed_parked()
        {
            self.notice = if self.language.open(self.assets.languages.packs.len()) {
                None
            } else {
                Some("Original language dictionaries are not loaded")
            };
            return;
        }
        self.notice = match (self.screen, self.settings_tab) {
            (Screen::Settings, 1) => self
                .display
                .select(self.row, self.vehicle.is_confirmed_parked()),
            (Screen::Settings, 3) => self
                .sound
                .select(self.row, self.vehicle.is_confirmed_parked()),
            (Screen::Settings, 4) => self.clock.select(
                self.row,
                self.vehicle.is_confirmed_parked(),
                self.assets.clock_zones.len(),
            ),
            (Screen::Settings, 7) => self
                .system
                .select(self.row, self.vehicle.is_confirmed_parked()),
            _ => Some("Menu restored; service is not connected in this host preview"),
        };
    }
    pub fn handle_pointer(&mut self, x: usize, y: usize) -> Option<MediaIntent> {
        if self.pointer_overlay(x, y) {
            return None;
        }
        if self.screen == Screen::Home {
            // Match MainMenuCtrl._computeHitAreas, which has separate JS focus offsets.
            if x >= 800 || y >= 480 {
                return None;
            }
            let (Ok(x), Ok(y)) = (i64::try_from(x), i64::try_from(y)) else {
                return None;
            };
            let offsets = [(30, -53), (-15, -30), (-15, -30), (-15, -30), (-58, -53)];
            for (index, coin) in COINS.iter().enumerate() {
                let (left, top, width, height) = coin.1;
                let (Ok(width), Ok(height)) = (i64::try_from(width), i64::try_from(height)) else {
                    continue;
                };
                // Tenths of a pixel retain the firmware's 1.2 scaling without float rounding.
                let (cx, cy, radius) = if index == self.home_focus {
                    (
                        i64::from(left) * 10 + width * 6 + offsets[index].0 * 10,
                        i64::from(top) * 10 + height * 6 + offsets[index].1 * 10,
                        height * 6,
                    )
                } else {
                    (
                        i64::from(left) * 10 + width * 5,
                        i64::from(top) * 10 + height * 5,
                        height * 5,
                    )
                };
                let dx = x * 10 - cx;
                let dy = y * 10 - cy;
                if dx * dx + dy * dy <= radius * radius {
                    self.open(Screen::DOMAINS[index]);
                    return None;
                }
            }
        } else if self.screen == Screen::Settings && (72..134).contains(&y) {
            self.pointer_settings_tab(x);
        } else if !self.rows().is_empty() && (68..800).contains(&x) {
            let top = if self.screen == Screen::Settings && self.settings_tab == 1 {
                256
            } else if self.screen == Screen::Settings {
                133
            } else {
                64
            };
            if y >= top && y < 480 {
                let first = self.first_visible_row();
                let row = first + (y - top) / 64;
                if row < self.rows().len() {
                    if !self.row_enabled(row) {
                        return None;
                    }
                    self.row = row;
                    if self.screen == Screen::Settings {
                        self.settings_rows = true;
                        if self.settings_tab == 7 && self.system_units_pointer(row, x) {
                            return None;
                        }
                        if self.settings_tab == 4 && self.clock_format_pointer(row, x) {
                            return None;
                        }
                        if self.settings_tab == 3 && self.sound_pointer_slider(row, x) {
                            return None;
                        }
                        if self.settings_tab == 1
                            && ((row == 2 && (384..738).contains(&x))
                                || ((row == 3 || row == 4) && (389..735).contains(&x)))
                        {
                            self.display
                                .pointer_value(row, x, self.vehicle.is_confirmed_parked());
                            return None;
                        }
                    }
                    return self.handle_commander(CommanderEvent::Select);
                }
            }
        }
        None
    }
    fn pointer_overlay(&mut self, x: usize, y: usize) -> bool {
        if self.display.values.power != DisplayPower::On {
            self.display.values.power = DisplayPower::On;
            return true;
        }
        if self.screen == Screen::Settings && self.display.reset.is_some() {
            if x < 94 && (64..480).contains(&y) {
                self.display.reset = None;
                return true;
            }
            if (382..462).contains(&y) && (69..731).contains(&x) {
                self.display.reset = Some(x >= 401);
                self.display
                    .active_event(CommanderEvent::Select, self.vehicle.is_confirmed_parked());
            }
            return true;
        }
        if self.screen != Screen::Home && x < 59 && y < 64 {
            if self.full_screen_dialog() {
                return true;
            }
            self.open(Screen::Home);
            return true;
        }
        let tab_arrow = self.screen == Screen::Settings
            && !self.clock_child()
            && !self.system_child()
            && x >= 52
            && (64..133).contains(&y);
        if self.left_back_visible() && x < 94 && (64..480).contains(&y) && !tab_arrow {
            self.handle_commander(CommanderEvent::Back);
            return true;
        }
        if self.system_child() {
            if self.music_update.page != MusicPage::Closed {
                self.music_update_pointer(x, y);
                return true;
            }
            if self.language.page != LanguagePage::Closed {
                self.language_pointer(x, y);
                return true;
            }
            self.system_pointer(x, y);
            return true;
        }
        if self.clock_child() {
            self.clock_pointer(x, y);
            return true;
        }
        false
    }
    pub fn render(&self, renderer: &mut impl Renderer) {
        renderer.clear(Color::BLACK);
        if self.display.values.power != DisplayPower::On {
            self.image(renderer, "settings-off", (0, 0, 800, 480));
            if self.display.values.power == DisplayPower::Clock {
                let label = self.clock.values.label();
                renderer.text(788 - text_width(&label, 38), 8, &label, 38, Color::TEXT);
            }
            return;
        }
        if self.screen == Screen::Settings && self.display.reset.is_some() {
            self.render_display_reset(renderer);
            self.image_natural(renderer, "left-back", 0, 64);
            return;
        } else if self.full_screen_dialog() {
            self.render_system_child(renderer);
            self.image_natural(
                renderer,
                if matches!(self.system.page, SystemPage::Progress { .. })
                    || matches!(self.language.page, LanguagePage::Progress { .. })
                    || self.music_update.page.busy()
                {
                    "left-back-disabled"
                } else {
                    "left-back"
                },
                0,
                64,
            );
            return;
        }
        self.image(renderer, "background", (0, 0, 800, 480));
        if self.screen == Screen::Home {
            self.render_home(renderer);
        } else {
            self.render_page(renderer);
        }
        if self.left_back_visible() {
            self.image_natural(renderer, "left-back", 0, 64);
        }
        self.image(renderer, "status", (0, 0, 800, 64));
        if self.screen != Screen::Home {
            self.image_natural(renderer, "home", 1, 1);
            fitted(
                renderer,
                117,
                8,
                self.tr(
                    self.status_dictionary(),
                    if self.screen == Screen::Navigation {
                        "Compass"
                    } else if self.clock_child() {
                        "System Settings"
                    } else if self.system_child() {
                        if self.system.page == SystemPage::Version
                            || self.music_update.page != MusicPage::Closed
                        {
                            "System Update"
                        } else if matches!(
                            self.system.page,
                            SystemPage::Confirm { .. } | SystemPage::Progress { .. }
                        ) {
                            "Settings"
                        } else {
                            "System Settings"
                        }
                    } else {
                        self.screen.label()
                    },
                ),
                420,
                38,
                Color::TEXT,
            );
        }
        // Deterministic host clock: owner settings never change the Mac or the car.
        let label = self.clock.values.label();
        renderer.text(788 - text_width(&label, 38), 8, &label, 38, Color::TEXT);
    }
    fn image(&self, r: &mut impl Renderer, name: &str, rect: (i32, i32, usize, usize)) {
        if let Some(bitmap) = self.assets.get(name) {
            r.image(rect.0, rect.1, rect.2, rect.3, bitmap);
        }
    }
    fn left_back_visible(&self) -> bool {
        self.screen != Screen::Home
            && !(self.clock_child() && matches!(self.clock.page, ClockPage::Adjust { .. }))
    }
    fn image_natural(&self, r: &mut impl Renderer, name: &str, x: i32, y: i32) {
        if let Some(bitmap) = self.assets.get(name) {
            let (width, height) = bitmap.size();
            r.image(x, y, width, height, bitmap);
        }
    }
    fn render_home(&self, r: &mut impl Renderer) {
        self.image(r, "ellipse", (0, 257, 800, 304));
        let selected = COINS[self.home_focus];
        self.image(r, "glow", (selected.3 .0, selected.3 .1, 363, 323));
        for index in [1, 2, 3, 4, 0] {
            let (name, normal, focused, _) = COINS[index];
            self.image(
                r,
                &if index == self.home_focus {
                    format!("{name}_Focus")
                } else {
                    name.to_owned()
                },
                if index == self.home_focus {
                    focused
                } else {
                    normal
                },
            );
        }
        let label = self.home_label();
        let width = text_width(label, 40);
        r.text(108 + (586 - width) / 2, 403, label, 40, Color::TEXT);
    }
    fn render_compass(&self, r: &mut impl Renderer) {
        // Firmware Compass context, explicitly using its invalid-location state.
        self.image_natural(r, "compass-bg", 27, 64);
        for x in [58, 306, 554] {
            self.image_natural(r, "compass-step", x, 64);
        }
        self.image_natural(r, "compass-rail", 80, 240);
        self.image_natural(r, "compass-needle", 428, 64);
        let disabled = Color::rgb(67, 67, 67);
        for (center, label, size, y) in [
            (182, "W", 85, 129),
            (306, "NW", 28, 187),
            (430, "N", 85, 129),
            (554, "NE", 28, 187),
            (678, "E", 85, 129),
        ] {
            let label = self.tr("emnavi", label);
            r.text(
                center - text_width(label, size) / 2,
                y,
                label,
                size,
                disabled,
            );
        }
        self.image_natural(r, "compass-foreground", 74, 65);
        let text="Your Mazda is not equipped with a Navigation system. However, an integrated Navigation system is available through your Mazda dealer as a Genuine Mazda Accessory.";
        let text = self.tr("emnavi", text);
        let (lines, size) = crate::typography::fit_block(text, 630, 3, 22);
        for (row, line) in lines.iter().enumerate() {
            r.text(
                115 + (630 - text_width(line, size)) / 2,
                276 + row * ((size * 14 + 5) / 10),
                line,
                size,
                Color::TEXT,
            );
        }
        self.image(r, "compass-line", (60, 376, 740, 3));
        for (x, label) in [(104, "Latitude"), (329, "Longitude"), (577, "Elevation")] {
            r.text(x, 411, self.tr("emnavi", label), 22, Color::TEXT);
            r.text(x, 440, "---", 30, Color::TEXT);
        }
    }
    fn render_display(&self, r: &mut impl Renderer) {
        if let Some(bitmap) = self.assets.get("settings-test") {
            let preview =
                bitmap.adjusted(self.display.values.brightness, self.display.values.contrast);
            r.image(69, 136, 713, 118, &preview);
        }
        let color = if self.vehicle.is_confirmed_parked() {
            Color::TEXT
        } else {
            Color::rgb(146, 148, 151)
        };
        let first = self.first_visible_row();
        for (visible, (index, label)) in DISPLAY.iter().enumerate().skip(first).enumerate() {
            let y = 256 + visible * 64;
            if y >= 480 {
                break;
            }
            if index == self.row && self.settings_rows {
                self.image(
                    r,
                    "list-focus",
                    (68, i32::try_from(y).unwrap_or(0), 732, 64),
                );
            }
            self.image(
                r,
                "list-separator",
                (68, i32::try_from(y + 62).unwrap_or(0), 732, 2),
            );
            fitted(
                r,
                139,
                y + 14,
                self.tr("syssettings", label),
                if index == 2 || index == 3 || index == 4 {
                    241
                } else {
                    605
                },
                28,
                color,
            );
            if index == 2 {
                for (button, label) in ["Auto", "Day", "Night"].into_iter().enumerate() {
                    let label = self.tr("syssettings", label);
                    let x = 384 + button * 118;
                    self.image_natural(
                        r,
                        if button == self.display.values.mode.index() {
                            "settings-inline-selected"
                        } else {
                            "settings-inline"
                        },
                        i32::try_from(x).unwrap_or(0) - 1,
                        i32::try_from(y + 9).unwrap_or(0),
                    );
                    r.text(
                        x + (118 - text_width(label, 28)) / 2,
                        y + 16,
                        label,
                        28,
                        color,
                    );
                }
            } else if index == 3 || index == 4 {
                self.render_display_slider(r, index, y, color);
            }
        }
    }
    fn render_display_slider(&self, r: &mut impl Renderer, index: usize, y: usize, color: Color) {
        let value = if index == 3 {
            self.display.values.brightness
        } else {
            self.display.values.contrast
        };
        let active = self.display.editing == Some(index);
        let filled = usize::try_from(i16::from(value) + 5).unwrap_or(0) * 344 / 10;
        self.image_natural(
            r,
            "settings-slider-bg",
            389,
            i32::try_from(y + 35).unwrap_or(0),
        );
        if let Some(bitmap) = self.assets.get(if active {
            "settings-slider-focus"
        } else {
            "settings-slider-fill"
        }) {
            if let Some(portion) = bitmap.left_portion(filled) {
                let (w, h) = portion.size();
                r.image(390, i32::try_from(y + 37).unwrap_or(0), w, h, &portion);
            }
        }
        let center = 390 + i32::try_from(filled).unwrap_or(0);
        self.image_natural(
            r,
            if active {
                "settings-knob-focus"
            } else {
                "settings-knob"
            },
            center - if active { 22 } else { 11 },
            i32::try_from(y + 42).unwrap_or(0) - if active { 22 } else { 11 },
        );
        r.text(389, y + 1, "-", 18, color);
        r.text(556, y + 1, "0", 18, color);
        r.text(727, y + 1, "+", 18, color);
    }
    fn render_display_reset(&self, r: &mut impl Renderer) {
        self.image(r, "settings-dialog-bg", (0, 0, 800, 480));
        centered_lines(
            r,
            self.tr(
                "syssettings",
                "Are you sure you want to restore Display settings to the factory defaults?",
            ),
            77,
            150,
            646,
            35,
            42,
        );
        for (index, label) in ["No", "Yes"].into_iter().enumerate() {
            let label = self.tr("common", label);
            let x = 69 + index * 332;
            let focused = self.display.reset == Some(index == 1);
            self.image_natural(
                r,
                if focused {
                    "settings-dialog-button-focus"
                } else {
                    "settings-dialog-button"
                },
                i32::try_from(x).unwrap_or(0),
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
    fn render_page(&self, r: &mut impl Renderer) {
        if self.system_child() {
            self.render_system_child(r);
            return;
        }
        if self.clock_child() {
            self.render_clock_child(r);
            return;
        }
        if self.screen == Screen::Navigation {
            self.render_compass(r);
            return;
        }
        self.image(r, "list-background", (68, 64, 732, 416));
        if self.screen == Screen::Settings {
            self.image_natural(r, "tab-prev", 52, 72);
            self.image_natural(r, "tab-next", 708, 72);
            let start = self.first_visible_tab();
            for (column, &index) in self.settings_tabs().iter().skip(start).take(4).enumerate() {
                let x = 136 + column * 143;
                self.image(
                    r,
                    if index == self.settings_tab {
                        "tab-focus"
                    } else {
                        "tab"
                    },
                    (i32::try_from(x).expect("tab coordinate"), 74, 143, 60),
                );
                let (label, size) = fit_text(self.tab_label(index, SETTINGS[index]), 131, 30);
                r.text(
                    x + (143 - text_width(&label, size)) / 2,
                    86,
                    &label,
                    size,
                    Color::TEXT,
                );
            }
            if self.settings_tab == 1 {
                self.render_display(r);
            } else if self.settings_tab == 3 {
                self.render_sound(r);
            } else if self.settings_tab == 4 {
                self.render_clock_root(r);
            } else if self.settings_tab == 7 {
                self.render_system_root(r);
            }
            return;
        }
        let top = if self.screen == Screen::Settings {
            144
        } else {
            64
        };
        let first = self.first_visible_row();
        for (visible, (index, label)) in self.rows().iter().enumerate().skip(first).enumerate() {
            let y = top + visible * 64;
            if y >= 480 {
                break;
            }
            if index == self.row && (self.screen != Screen::Settings || self.settings_rows) {
                self.image(
                    r,
                    "list-focus",
                    (68, i32::try_from(y).expect("row coordinate"), 732, 64),
                );
            }
            self.image(
                r,
                "list-separator",
                (68, i32::try_from(y + 62).expect("row coordinate"), 732, 2),
            );
            let label = self.tr("system", label);
            let text = if self.screen == Screen::Applications {
                format!("{}.  {label}", index + 1)
            } else {
                label.to_owned()
            };
            fitted(r, 139, y + 9, &text, 605, 35, Color::TEXT);
        }
    }
}
fn fitted(
    r: &mut impl Renderer,
    x: usize,
    y: usize,
    text: &str,
    width: usize,
    size: usize,
    color: Color,
) {
    let (label, size) = fit_text(text, width, size);
    r.text(x, y, &label, size, color);
}

fn centered_lines(
    r: &mut impl Renderer,
    text: &str,
    x: usize,
    y: usize,
    width: usize,
    size: usize,
    line_height: usize,
) {
    let mut line = String::new();
    let mut row = 0;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if text_width(&candidate, size) > width && !line.is_empty() {
            let bounded = crate::typography::ellipsized(&line, width, size);
            r.text(
                x + (width - text_width(&bounded, size)) / 2,
                y + row * line_height,
                &bounded,
                size,
                Color::TEXT,
            );
            row += 1;
            word.clone_into(&mut line);
        } else {
            line = candidate;
        }
    }
    let bounded = crate::typography::ellipsized(&line, width, size);
    r.text(
        x + (width - text_width(&bounded, size)) / 2,
        y + row * line_height,
        &bounded,
        size,
        Color::TEXT,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mazda_core::{Gear, PlaybackState, SpeedKph, TemperatureC};
    #[test]
    fn oversized_localized_dialog_words_are_bounded_without_alignment_underflow() {
        let mut frame = crate::Framebuffer::mazda_connect();
        centered_lines(&mut frame, &"W".repeat(1024), 77, 150, 646, 35, 42);
        assert!(frame.pixels().iter().any(|&p| p != 0));
        for (index, &pixel) in frame.pixels().iter().enumerate() {
            if !(77..723).contains(&(index % 800)) {
                assert_eq!(pixel, 0);
            }
        }
    }
    struct Fixture;
    impl MazdaReadOnly for Fixture {
        fn vehicle_snapshot(&self) -> VehicleSnapshot {
            VehicleSnapshot {
                speed: SpeedKph::new(0.0),
                gear: Gear::Park,
                outside_temperature: TemperatureC::new(20.0),
            }
        }
        fn media_state(&self) -> MediaState {
            MediaState {
                title: None,
                artist: None,
                album: None,
                playback: PlaybackState::Stopped,
            }
        }
        fn next_commander_event(&mut self) -> Option<CommanderEvent> {
            None
        }
    }
    #[test]
    fn firmware_home_defaults_to_communication_and_clamps_at_both_ends() {
        let mut ui = UiModel::from_source(&Fixture);
        assert_eq!(ui.selection(), Screen::Communication);
        for _ in 0..10 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        assert_eq!(ui.selection(), Screen::Settings);
        for _ in 0..10 {
            ui.handle_commander(CommanderEvent::RotateCounterClockwise);
        }
        assert_eq!(ui.selection(), Screen::Applications);
    }
    #[test]
    fn five_domains_route_and_back_restores_the_selected_home_coin() {
        for (index, screen) in Screen::DOMAINS.into_iter().enumerate() {
            let mut ui = UiModel::from_source(&Fixture);
            ui.home_focus = index;
            ui.handle_commander(CommanderEvent::Select);
            assert_eq!(ui.screen(), screen);
            ui.handle_commander(CommanderEvent::Back);
            assert_eq!(ui.screen(), Screen::Home);
            assert_eq!(ui.selection(), screen);
        }
    }
    #[test]
    fn shortcuts_preserve_domains_and_favorites_is_not_repurposed() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.handle_commander(CommanderEvent::Music);
        assert_eq!(ui.screen(), Screen::Entertainment);
        ui.handle_commander(CommanderEvent::Navigation);
        assert_eq!(ui.screen(), Screen::Navigation);
        ui.handle_commander(CommanderEvent::Favorites);
        assert_eq!(ui.screen(), Screen::Navigation);
        assert!(ui.notice().is_some());
    }
    #[test]
    fn circular_pointer_hit_tests_reject_empty_space_and_extreme_input() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.handle_pointer(400, 260);
        assert_eq!(ui.screen(), Screen::Communication);
        ui.handle_commander(CommanderEvent::Home);
        ui.handle_pointer(400, 169);
        assert_eq!(ui.screen(), Screen::Communication);
        ui.handle_commander(CommanderEvent::Home);
        ui.handle_pointer(400, 100);
        assert_eq!(ui.screen(), Screen::Home);
        ui.handle_pointer(usize::MAX, usize::MAX);
        assert_eq!(ui.screen(), Screen::Home);
    }
    #[test]
    fn original_list_order_and_scroll_keep_every_item_reachable_without_writes() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Entertainment);
        assert_eq!(
            ui.rows(),
            ["FM", "AM", "Stitcher", "Bluetooth", "USB 1", "USB 2", "AUX"]
        );
        for _ in 0..12 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        assert_eq!(ui.row, 6);
        assert_eq!(ui.handle_commander(CommanderEvent::Select), None);
        assert!(ui.notice().is_some());
        let snapshot = ui.vehicle.clone();
        ui.open(Screen::Settings);
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.vehicle, snapshot);
    }
    #[test]
    fn display_list_scrolls_the_last_item_into_view_and_back_keeps_the_tab() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Settings);
        assert_eq!(ui.rows().len(), 6);
        ui.handle_commander(CommanderEvent::Select);
        for _ in 0..8 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        assert_eq!(ui.rows()[ui.row], "Reset");
        assert_eq!(ui.first_visible_row(), 3);
        ui.handle_commander(CommanderEvent::Back);
        assert_eq!(ui.screen(), Screen::Settings);
        assert!(!ui.settings_rows);
    }
    #[test]
    fn display_pointer_drag_wakes_and_reset_routes_keep_observations_read_only() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_pointer(700, 410);
        assert_eq!(ui.display_preferences().mode, crate::DisplayMode::Night);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        ui.handle_pointer(560, 420);
        ui.handle_pointer_drag(733, 420);
        ui.handle_pointer_release();
        assert_eq!(ui.display_preferences().brightness, 5);
        assert!(!ui.display_adjusting());
        assert_eq!(ui.row, 3);
        for _ in 0..2 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.display.reset, Some(false));
        ui.handle_pointer(570, 420);
        assert_eq!(ui.display_preferences(), DisplayPreferences::default());
        for _ in 0..4 {
            ui.handle_commander(CommanderEvent::RotateCounterClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.display_preferences().power, DisplayPower::Clock);
        ui.handle_pointer(700, 410);
        assert_eq!(ui.display_preferences().power, DisplayPower::On);
        assert_eq!(ui.display_preferences().mode, crate::DisplayMode::Auto);
        assert_eq!(ui.vehicle, Fixture.vehicle_snapshot());
    }
    #[test]
    fn sound_rows_edit_scroll_and_return_through_the_original_tab_bar() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Settings);
        for _ in 0..2 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        assert_eq!(ui.settings_tab, 3);
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        assert!(ui.sound_adjusting());
        assert_eq!(ui.sound_preferences().bass, 1);
        assert_eq!(ui.row, 0);
        ui.handle_commander(CommanderEvent::Back);
        assert!(ui.settings_rows);
        assert!(!ui.sound_adjusting());
        for _ in 0..20 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        assert_eq!(ui.rows()[ui.row], "Beep");
        assert_eq!(ui.first_visible_row(), 2);
        ui.handle_commander(CommanderEvent::Select);
        assert!(ui.sound_preferences().beep);
        ui.handle_commander(CommanderEvent::Back);
        assert!(!ui.settings_rows);
        assert_eq!(ui.settings_tab, 3);
        ui.handle_commander(CommanderEvent::Back);
        assert_eq!(ui.screen(), Screen::Home);
        assert_eq!(ui.selection(), Screen::Settings);
        assert_eq!(ui.vehicle, Fixture.vehicle_snapshot());
    }
    #[test]
    fn sound_drag_stays_on_its_control_and_tab_change_revokes_it() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Settings);
        ui.settings_tab = 3;
        ui.handle_pointer(400, 160);
        ui.handle_pointer_drag(735, 160);
        assert_eq!(ui.sound_preferences().bass, 6);
        assert!(ui.sound_adjusting());
        ui.set_guidance_available(false);
        assert!(ui.sound_adjusting()); // Guidance service loss must not revoke tone adjustment.
        ui.handle_pointer_release();
        assert!(!ui.sound_adjusting());
        ui.handle_pointer(562, 160);
        assert_eq!(ui.sound_preferences().bass, 0);
        ui.handle_pointer(750, 100);
        assert_eq!(ui.settings_tab, 6);
        assert!(!ui.sound_adjusting());
        ui.handle_pointer_drag(735, 160);
        assert_eq!(ui.sound_preferences().bass, 0);
    }
    #[test]
    fn equipment_tabs_and_profile_changes_do_not_expose_unsupported_controls() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Settings);
        assert_eq!(ui.settings_tabs(), [1, 2, 3, 4, 5, 6, 7]);
        ui.handle_commander(CommanderEvent::RotateCounterClockwise);
        assert_eq!(ui.settings_tab, 1);
        ui.handle_pointer(450, 100);
        assert_eq!(ui.settings_tab, 3);
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_commander(CommanderEvent::Select);
        assert!(ui.sound_adjusting());
        ui.set_sound_profile(SoundProfile::Bose);
        assert!(!ui.sound_adjusting());
        assert_eq!(ui.rows().len(), 8);
        assert_eq!(ui.row, 0);
        ui.set_hud_installed(true);
        assert_eq!(ui.settings_tabs().len(), 8);
        ui.settings_tab = 0;
        ui.set_hud_installed(false);
        assert_eq!(ui.settings_tab, 1);
        assert!(!ui.settings_rows);
    }
    #[test]
    fn source_tab_history_and_arrow_jumps_keep_the_original_visible_context() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Settings);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        assert_eq!(ui.settings_tab, 3);
        assert_eq!(ui.first_visible_tab(), 0);
        ui.handle_pointer(750, 100);
        assert_eq!(ui.settings_tab, 5);
        assert_eq!(ui.first_visible_tab(), 3);
        ui.handle_pointer(70, 100);
        assert_eq!(ui.settings_tab, 3);
        assert_eq!(ui.first_visible_tab(), 0);
        ui.handle_pointer(70, 100);
        assert_eq!(ui.settings_tab, 3);
        ui.settings_tab = 7;
        ui.tab_favor = TabFavor::Second;
        ui.handle_pointer(750, 100);
        assert_eq!(ui.settings_tab, 7);
    }
    #[test]
    fn clock_contexts_route_back_to_their_origin_and_keep_source_observations_read_only() {
        let mut ui = UiModel::from_source(&Fixture);
        let mut assets = StockAssets::default();
        assert!(assets.load_clock_zones(b"MZTIMEZONES1\nWest\nCentral\nEast\n"));
        ui.set_assets(assets);
        ui.open(Screen::Settings);
        for _ in 0..3 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        assert_eq!(ui.settings_tab, 4);
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.clock.page, ClockPage::Adjust { focus: 0 });
        for _ in 0..2 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.clock_preferences().hour, 13);
        assert_eq!(ui.clock_preferences().label(), "1:00");
        ui.handle_commander(CommanderEvent::Back);
        assert_eq!(ui.row, 0);
        assert!(ui.settings_rows);
        for _ in 0..3 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.clock.page, ClockPage::Zones { focus: 2 });
        ui.handle_commander(CommanderEvent::RotateCounterClockwise);
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.clock_preferences().time_zone, 1);
        ui.handle_commander(CommanderEvent::Back);
        assert_eq!(ui.row, 3);
        assert!(ui.settings_rows);
        ui.handle_commander(CommanderEvent::Home);
        assert_eq!(ui.clock.page, ClockPage::Root);
        assert_eq!(ui.vehicle, Fixture.vehicle_snapshot());
    }
    #[test]
    fn clock_pointer_paths_restrictions_and_large_digit_rendering_are_bounded() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Settings);
        ui.settings_tab = 4;
        ui.settings_rows = true;
        ui.handle_pointer(680, 280);
        assert_eq!(ui.clock_preferences().format, crate::TimeFormat::Hours24);
        ui.row = 0;
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.clock_button_at(usize::MAX, 400), None);
        assert_eq!(ui.clock_button_at(680, 420), None);
        ui.focus_clock_button(2);
        assert_eq!(ui.clock_repeat_target(), Some(2));
        ui.handle_pointer(300, 420);
        assert_eq!(ui.clock_preferences().hour, 13);
        let mut frame = crate::Framebuffer::mazda_connect();
        ui.render(&mut frame);
        assert!(frame.pixels().iter().any(|&p| p != 0));
        ui.set_clock_gps_sync(true);
        assert_eq!(ui.clock_repeat_target(), None);
        assert_eq!(ui.clock.page, ClockPage::Adjust { focus: 0 });
        ui.handle_pointer(300, 420);
        assert_eq!(ui.clock_preferences().hour, 13);
        ui.handle_pointer(20, 420);
        assert_eq!(ui.clock.page, ClockPage::Root);
        assert_eq!(ui.clock_button_at(300, 420), None);
    }
    #[test]
    fn clock_list_focus_skips_source_disabled_gps_and_navigation_rows() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Settings);
        ui.settings_tab = 4;
        ui.set_clock_gps_sync(true);
        ui.set_clock_navigation_available(true);
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.row, 1);
        ui.handle_commander(CommanderEvent::RotateCounterClockwise);
        assert_eq!(ui.row, 1);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        assert_eq!(ui.row, 2);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        assert_eq!(ui.row, 2);
        ui.handle_pointer(200, 160);
        assert_eq!(ui.row, 2);
        assert_eq!(ui.clock.page, ClockPage::Root);
    }
    #[test]
    fn system_reset_confirms_then_resets_host_preferences_and_preserves_equipment_and_observations()
    {
        let mut ui = UiModel::from_source(&Fixture);
        ui.set_sound_profile(crate::SoundProfile::Bose);
        ui.set_guidance_available(false);
        ui.clock.navigation_available = true;
        ui.display.values.brightness = 4;
        ui.sound.values.bass = -5;
        ui.clock.values.hour = 22;
        ui.system.values.tool_tips = true;
        ui.open(Screen::Settings);
        ui.settings_tab = 7;
        ui.settings_rows = true;
        ui.row = 4;
        let vehicle = ui.vehicle.clone();
        let media = ui.media.clone();
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.system.page, SystemPage::Confirm { yes: false });
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.sound.values.bass, -5);
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.system.page, SystemPage::Progress { elapsed_ms: 0 });
        ui.advance_simulation(std::time::Duration::from_millis(1499));
        assert_eq!(ui.clock.values.hour, 22);
        ui.advance_simulation(std::time::Duration::from_millis(1));
        assert_eq!(ui.display.values, crate::DisplayPreferences::default());
        assert_eq!(ui.sound.values, crate::SoundPreferences::default());
        assert_eq!(ui.clock.values, crate::ClockPreferences::default());
        assert_eq!(ui.system.values, crate::SystemPreferences::default());
        assert_eq!(ui.sound.profile, crate::SoundProfile::Bose);
        assert!(!ui.sound.guidance_enabled);
        assert!(ui.clock.navigation_available);
        assert_eq!(ui.row, 4);
        assert_eq!(ui.system.page, SystemPage::Root);
        assert_eq!(ui.vehicle, vehicle);
        assert_eq!(ui.media, media);
    }
    #[test]
    fn system_pointer_routes_and_service_loss_cancel_reset_without_changes() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Settings);
        ui.settings_tab = 7;
        ui.settings_rows = true;
        ui.set_system_equipment(crate::SystemEquipment {
            temperature_units: true,
            distance_units: true,
            ..Default::default()
        });
        ui.system.values.celsius = false;
        ui.system.values.kilometers = false;
        ui.handle_pointer(680, 133 + 3 * 64 + 20);
        assert!(ui.system.values.celsius);
        assert!(!ui.system.values.kilometers);
        ui.row = 6;
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_pointer(600, 420);
        assert!(matches!(ui.system.page, SystemPage::Progress { .. }));
        ui.system.equipment.services_ready = false;
        ui.advance_simulation(std::time::Duration::MAX);
        assert_eq!(ui.system.page, SystemPage::Root);
        assert!(ui.system.values.celsius);
        ui.row = 7;
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.system.page, SystemPage::About { focus: 0 });
        ui.handle_pointer(300, 220);
        assert_eq!(ui.system.page, SystemPage::Version);
        ui.handle_pointer(400, 420);
        assert_eq!(ui.system.page, SystemPage::About { focus: 1 });
        ui.handle_commander(CommanderEvent::Back);
        assert_eq!(ui.row, 7);
        let before = ui.system.values;
        ui.handle_pointer(usize::MAX, usize::MAX);
        assert_eq!(ui.system.values, before);
    }
    #[test]
    fn full_screen_reset_hides_status_and_left_back_cancels_without_confirmation() {
        let mut ui = UiModel::from_source(&Fixture);
        let mut assets = StockAssets::default();
        let mut bytes = b"MZRGBA01".to_vec();
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([255, 0, 0, 255]);
        assets.insert("status", crate::Bitmap::decode(&bytes).unwrap());
        ui.set_assets(assets);
        ui.open(Screen::Settings);
        ui.settings_tab = 7;
        ui.settings_rows = true;
        ui.row = 4;
        let mut frame = crate::Framebuffer::mazda_connect();
        ui.render(&mut frame);
        assert_eq!(frame.pixels()[400], 0x00ff_0000);
        ui.handle_commander(CommanderEvent::Select);
        ui.render(&mut frame);
        assert_eq!(frame.pixels()[400], 0);
        ui.handle_pointer(10, 200);
        assert_eq!(ui.system.page, SystemPage::Root);
        ui.settings_tab = 1;
        ui.display.values.brightness = 3;
        ui.display.reset = Some(false);
        ui.render(&mut frame);
        assert_eq!(frame.pixels()[400], 0);
        ui.handle_pointer(10, 200);
        assert_eq!(ui.display.reset, None);
        assert_eq!(ui.display.values.brightness, 3);
    }
    #[test]
    fn language_route_changes_real_lookup_after_confirmation_and_success_only() {
        let mut ui = UiModel::from_source(&Fixture);
        let mut assets = StockAssets::default();
        assert!(assets.load_languages(&crate::languages::fixture()));
        assert!(!assets.load_languages(b"invalid"));
        ui.set_assets(assets);
        ui.open(Screen::Settings);
        ui.settings_tab = 7;
        ui.settings_rows = true;
        ui.row = 2;
        let observations = ui.vehicle.clone();
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.language.page, LanguagePage::List { focus: 0 });
        ui.handle_commander(CommanderEvent::RotateClockwise);
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(
            ui.language.page,
            LanguagePage::Confirm {
                target: 1,
                yes: false
            }
        );
        assert_eq!(ui.tr("syssettings", "Language"), "Language A");
        ui.handle_commander(CommanderEvent::RotateClockwise);
        ui.handle_commander(CommanderEvent::Select);
        ui.set_language_change_failure(true);
        ui.advance_simulation(std::time::Duration::from_secs(2));
        assert!(matches!(
            ui.language.page,
            LanguagePage::Error { yes: false, .. }
        ));
        assert_eq!(ui.language_code(), "en_US");
        ui.handle_commander(CommanderEvent::RotateClockwise);
        ui.handle_commander(CommanderEvent::Select);
        ui.set_language_change_failure(false);
        ui.advance_simulation(std::time::Duration::from_secs(2));
        assert_eq!(ui.language_code(), "fr_CN");
        assert_eq!(ui.tr("syssettings", "Language"), "Language B");
        assert_eq!(ui.tab_label(7, "System"), "System B");
        assert_eq!(ui.tr("syssettings", "System"), "Mode B");
        assert_eq!(ui.tab_label(4, "Clock"), "Clock B");
        assert_eq!(ui.vehicle, observations);
        ui.handle_commander(CommanderEvent::Back);
        assert_eq!(ui.row, 2);
        assert!(ui.settings_rows);
        assert!(!ui.set_host_language("missing"));
        assert_eq!(ui.language_code(), "fr_CN");
        ui.row = 4;
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        ui.handle_commander(CommanderEvent::Select);
        ui.advance_simulation(std::time::Duration::from_secs(2));
        assert_eq!(ui.language_code(), "en_US");
    }
    #[test]
    fn language_pointer_confirmation_and_global_exit_preserve_selected_locale() {
        let mut ui = UiModel::from_source(&Fixture);
        let mut assets = StockAssets::default();
        assets.load_languages(&crate::languages::fixture());
        ui.set_assets(assets);
        ui.open(Screen::Settings);
        ui.settings_tab = 7;
        ui.settings_rows = true;
        ui.row = 2;
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_pointer(300, 220);
        assert_eq!(
            ui.language.page,
            LanguagePage::Confirm {
                target: 1,
                yes: false
            }
        );
        ui.handle_pointer(200, 420);
        assert_eq!(ui.language.page, LanguagePage::List { focus: 0 });
        assert_eq!(ui.language_code(), "en_US");
        ui.handle_pointer(300, 220);
        ui.handle_pointer(600, 420);
        assert!(matches!(ui.language.page, LanguagePage::Progress { .. }));
        ui.handle_commander(CommanderEvent::Home);
        ui.advance_simulation(std::time::Duration::MAX);
        assert_eq!(ui.language_code(), "en_US");
        assert_eq!(ui.language.page, LanguagePage::Closed);
        ui.handle_pointer(usize::MAX, usize::MAX);
        assert_eq!(ui.language_code(), "en_US");
    }
    #[test]
    fn music_update_routes_preserve_system_selection_and_require_policy() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Settings);
        ui.settings_tab = 7;
        ui.settings_rows = true;
        ui.row = 3;
        ui.handle_commander(CommanderEvent::Select);
        assert_eq!(ui.music_update.page, MusicPage::Search);
        assert_eq!(ui.music_update.focus, 1);
        assert_eq!(ui.status_dictionary(), "sysupdate");
        ui.handle_pointer(140, 420);
        assert_eq!(ui.music_update.page, MusicPage::Closed);
        assert_eq!(ui.row, 3);
        ui.set_music_update_fixture(crate::MusicUpdateFixture::Available);
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_pointer(600, 420);
        ui.advance_simulation(std::time::Duration::from_millis(1200));
        ui.handle_pointer(500, 220);
        assert_eq!(ui.music_update.page, MusicPage::Confirm);
        ui.handle_pointer(140, 420);
        assert_eq!(ui.music_update.page, MusicPage::Packages);
        ui.handle_pointer(500, 220);
        ui.handle_pointer(600, 420);
        ui.advance_simulation(std::time::Duration::from_millis(800));
        assert!(ui.full_screen_dialog());
        ui.handle_pointer(140, 420);
        assert_eq!(ui.music_update.page, MusicPage::Installing);
        ui.system.equipment.services_ready = false;
        ui.advance_simulation(std::time::Duration::MAX);
        assert_eq!(ui.music_update.page, MusicPage::Search);
        assert_eq!(ui.simulated_music_version(), None);
        ui.handle_pointer(600, 420);
        assert_eq!(ui.music_update.page, MusicPage::Closed);
        assert_eq!(ui.vehicle, Fixture.vehicle_snapshot());
    }
    #[test]
    fn music_update_all_source_contexts_render_and_home_cancels_host_work() {
        let mut ui = UiModel::from_source(&Fixture);
        ui.open(Screen::Settings);
        ui.settings_tab = 7;
        for page in [
            MusicPage::Search,
            MusicPage::Searching,
            MusicPage::Packages,
            MusicPage::Confirm,
            MusicPage::Preparing,
            MusicPage::Installing,
            MusicPage::Retrying,
            MusicPage::Success,
            MusicPage::SearchFailed,
            MusicPage::NoPackages,
            MusicPage::ValidationFailed,
            MusicPage::RetryValidationFailed,
            MusicPage::Unavailable,
            MusicPage::InstallFailed,
            MusicPage::NeedsFailsafe,
        ] {
            ui.music_update.page = page;
            ui.render(&mut crate::Framebuffer::mazda_connect());
        }
        ui.music_update.page = MusicPage::Installing;
        ui.handle_commander(CommanderEvent::Home);
        ui.advance_simulation(std::time::Duration::MAX);
        assert_eq!(ui.screen(), Screen::Home);
        assert_eq!(ui.music_update.page, MusicPage::Closed);
        assert_eq!(ui.simulated_music_version(), None);
    }
}
