//! Bounded owner dictionaries and deterministic language service simulation.
use mazda_core::CommanderEvent;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub(crate) struct LanguagePack {
    pub id: String,
    pub code: String,
    pub vr: bool,
    pub tts: bool,
    strings: BTreeMap<(String, String), (String, String)>,
    aliases: BTreeMap<(String, String), Option<String>>,
}
impl LanguagePack {
    fn insert(&mut self, app: String, key: String, english: String, translated: String) -> bool {
        let alias = (app.clone(), english.clone());
        self.aliases
            .entry(alias)
            .and_modify(|value| {
                if value.as_ref() != Some(&translated) {
                    *value = None;
                }
            })
            .or_insert_with(|| Some(translated.clone()));
        self.strings
            .insert((app, key), (english, translated))
            .is_none()
    }
    pub fn key<'a>(&'a self, app: &str, key: &str, fallback: &'a str) -> &'a str {
        self.strings
            .get(&(app.to_owned(), key.to_owned()))
            .or_else(|| self.strings.get(&("common".into(), key.to_owned())))
            .map_or(fallback, |(_, value)| value)
    }
    pub fn text<'a>(&'a self, app: &str, english: &'a str) -> &'a str {
        self.aliases
            .get(&(app.to_owned(), english.to_owned()))
            .or_else(|| self.aliases.get(&("common".into(), english.to_owned())))
            .and_then(Option::as_deref)
            .unwrap_or(english)
    }
}
#[derive(Debug, Clone, Default)]
pub(crate) struct LanguageCatalog {
    pub packs: Vec<LanguagePack>,
}
struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let end = self.position.checked_add(count)?;
        let out = self.bytes.get(self.position..end)?;
        self.position = end;
        Some(out)
    }
    fn number(&mut self) -> Option<usize> {
        usize::try_from(u32::from_le_bytes(self.take(4)?.try_into().ok()?)).ok()
    }
    fn string(&mut self) -> Option<String> {
        let count = self.number()?;
        if count > 16384 {
            return None;
        }
        let value = std::str::from_utf8(self.take(count)?).ok()?;
        if value.chars().count() > 4096 || value.chars().any(|c| c.is_control() && c != '\n') {
            return None;
        }
        Some(value.to_owned())
    }
    fn flag(&mut self) -> Option<bool> {
        match self.take(1)?[0] {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}
impl LanguageCatalog {
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() > 4_000_000 {
            return None;
        }
        let mut r = Reader { bytes, position: 0 };
        if r.take(8)? != b"MZLANG01" {
            return None;
        }
        let count = r.number()?;
        if !(1..=64).contains(&count) {
            return None;
        }
        let mut packs = Vec::new();
        let mut codes = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for _ in 0..count {
            let id = r.string()?;
            let code = r.string()?;
            if id.is_empty()
                || id.len() > 80
                || code.is_empty()
                || code.len() > 16
                || !id.bytes().all(|b| b.is_ascii_uppercase() || b == b'_')
                || !code.bytes().all(|b| b.is_ascii_alphabetic() || b == b'_')
                || !codes.insert(code.clone())
                || !ids.insert(id.clone())
            {
                return None;
            }
            let mut pack = LanguagePack {
                id,
                code,
                vr: r.flag()?,
                tts: r.flag()?,
                strings: BTreeMap::new(),
                aliases: BTreeMap::new(),
            };
            let rows = r.number()?;
            if !(1..=4096).contains(&rows) {
                return None;
            }
            for _ in 0..rows {
                let app = r.string()?;
                let key = r.string()?;
                let en = r.string()?;
                let tr = r.string()?;
                if ![
                    "common",
                    "system",
                    "syssettings",
                    "audiosettings",
                    "emnavi",
                    "sysupdate",
                ]
                .contains(&app.as_str())
                    || key.is_empty()
                    || key.len() > 256
                    || !pack.insert(app, key, en, tr)
                {
                    return None;
                }
            }
            packs.push(pack);
        }
        (r.position == bytes.len()).then_some(Self { packs })
    }
    pub fn get(&self, index: usize) -> Option<&LanguagePack> {
        self.packs.get(index)
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum LanguagePage {
    #[default]
    Closed,
    List {
        focus: usize,
    },
    Confirm {
        target: usize,
        yes: bool,
    },
    Progress {
        target: usize,
        elapsed_ms: u64,
    },
    Error {
        target: usize,
        yes: bool,
    },
}
#[derive(Debug, Clone, Default)]
pub(crate) struct LanguageSession {
    pub current: usize,
    pub page: LanguagePage,
    pub fail_changes: bool,
}
impl LanguageSession {
    pub fn open(&mut self, count: usize) -> bool {
        if count == 0 {
            return false;
        }
        self.current = self.current.min(count - 1);
        self.page = LanguagePage::List {
            focus: self.current,
        };
        true
    }
    pub fn restrict(&mut self, parked: bool) {
        if !parked
            && matches!(
                self.page,
                LanguagePage::Confirm { .. }
                    | LanguagePage::Progress { .. }
                    | LanguagePage::Error { .. }
            )
        {
            self.page = LanguagePage::List {
                focus: self.current,
            };
        }
    }
    pub fn event(&mut self, event: CommanderEvent, parked: bool, count: usize) -> bool {
        self.restrict(parked);
        match self.page {
            LanguagePage::Closed => return false,
            LanguagePage::List { mut focus } => {
                match event {
                    CommanderEvent::Back => {
                        self.page = LanguagePage::Closed;
                        return true;
                    }
                    CommanderEvent::RotateClockwise if parked => {
                        focus = (focus + 1).min(count.saturating_sub(1));
                    }
                    CommanderEvent::RotateCounterClockwise if parked => {
                        focus = focus.saturating_sub(1);
                    }
                    CommanderEvent::Select if parked && focus < count => {
                        self.page = LanguagePage::Confirm {
                            target: focus,
                            yes: false,
                        };
                        return true;
                    }
                    _ => {}
                }
                self.page = LanguagePage::List { focus };
            }
            LanguagePage::Confirm { target, mut yes } | LanguagePage::Error { target, mut yes } => {
                let error = matches!(self.page, LanguagePage::Error { .. });
                match event {
                    CommanderEvent::RotateClockwise => yes = true,
                    CommanderEvent::RotateCounterClockwise => yes = false,
                    CommanderEvent::Back => {
                        self.page = LanguagePage::List {
                            focus: self.current,
                        };
                        return true;
                    }
                    CommanderEvent::Select => {
                        self.page = if yes && parked {
                            LanguagePage::Progress {
                                target,
                                elapsed_ms: 0,
                            }
                        } else {
                            LanguagePage::List {
                                focus: self.current,
                            }
                        };
                        return true;
                    }
                    _ => {}
                }
                self.page = if error {
                    LanguagePage::Error { target, yes }
                } else {
                    LanguagePage::Confirm { target, yes }
                };
            }
            LanguagePage::Progress { .. } => {}
        }
        true
    }
    pub fn advance(&mut self, elapsed_ms: u64, parked: bool) {
        self.restrict(parked);
        if let LanguagePage::Progress {
            target,
            elapsed_ms: previous,
        } = self.page
        {
            let elapsed_ms = previous.saturating_add(elapsed_ms);
            if elapsed_ms < 1500 {
                self.page = LanguagePage::Progress { target, elapsed_ms };
            } else if self.fail_changes {
                self.page = LanguagePage::Error { target, yes: false };
            } else {
                self.current = target;
                self.page = LanguagePage::List { focus: target };
            }
        }
    }
    pub const fn full_screen(&self) -> bool {
        matches!(
            self.page,
            LanguagePage::Confirm { .. }
                | LanguagePage::Progress { .. }
                | LanguagePage::Error { .. }
        )
    }
}
#[cfg(test)]
pub(crate) fn fixture() -> Vec<u8> {
    fn text(out: &mut Vec<u8>, value: &str) {
        out.extend(u32::try_from(value.len()).unwrap().to_le_bytes());
        out.extend(value.as_bytes());
    }
    let mut out = b"MZLANG01".to_vec();
    out.extend(2_u32.to_le_bytes());
    for (id, code, translated) in [
        ("LANGS_US_ENGLISH", "en_US", "Language A"),
        ("LANGS_CN_FRENCH", "fr_CN", "Language B"),
    ] {
        text(&mut out, id);
        text(&mut out, code);
        out.extend([1, 1]);
        out.extend(5_u32.to_le_bytes());
        let (clock, mode, system) = if code == "en_US" {
            ("Clock A", "Mode A", "System A")
        } else {
            ("Clock B", "Mode B", "System B")
        };
        for (app, key, en, tr) in [
            ("syssettings", "Language", "Language", translated),
            ("syssettings", id, id, translated),
            ("syssettings", "Clock", "Clock", clock),
            ("syssettings", "Mode", "System", mode),
            ("common", "SystemTab", "System", system),
        ] {
            for value in [app, key, en, tr] {
                text(&mut out, value);
            }
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_rejects_truncation_trailing_data_and_invalid_counts() {
        let data = fixture();
        let decoded = LanguageCatalog::decode(&data).unwrap();
        assert_eq!(decoded.packs.len(), 2);
        assert_eq!(
            decoded.get(1).unwrap().text("syssettings", "Language"),
            "Language B"
        );
        for end in 0..data.len() {
            assert!(LanguageCatalog::decode(&data[..end]).is_none());
        }
        let mut tail = data.clone();
        tail.push(0);
        assert!(LanguageCatalog::decode(&tail).is_none());
        let mut count = data;
        count[8..12].copy_from_slice(&65_u32.to_le_bytes());
        assert!(LanguageCatalog::decode(&count).is_none());
        let mut duplicate = fixture();
        let index = duplicate.windows(5).position(|w| w == b"fr_CN").unwrap();
        duplicate[index..index + 5].copy_from_slice(b"en_US");
        assert!(LanguageCatalog::decode(&duplicate).is_none());
        let mut invalid = fixture();
        let last = invalid.len() - 1;
        invalid[last] = 255;
        assert!(LanguageCatalog::decode(&invalid).is_none());
    }
    #[test]
    fn aliases_never_guess_between_ambiguous_string_ids_or_apps() {
        let mut pack = LanguageCatalog::decode(&fixture()).unwrap().packs.remove(0);
        assert!(pack.insert(
            "syssettings".into(),
            "Other".into(),
            "Language".into(),
            "Other translation".into()
        ));
        assert_eq!(pack.text("syssettings", "Language"), "Language");
        assert_eq!(pack.key("syssettings", "Language", "missing"), "Language A");
        assert_eq!(pack.text("audiosettings", "Language"), "Language");
        assert_eq!(pack.text("syssettings", "System"), "Mode A");
        assert_eq!(pack.key("common", "SystemTab", "missing"), "System A");
    }
    #[test]
    fn selection_confirms_changes_only_after_success_and_retry_keeps_old_language() {
        let mut s = LanguageSession::default();
        assert!(!s.open(0));
        assert!(s.open(3));
        s.event(CommanderEvent::RotateClockwise, true, 3);
        s.event(CommanderEvent::Select, true, 3);
        assert_eq!(
            s.page,
            LanguagePage::Confirm {
                target: 1,
                yes: false
            }
        );
        s.event(CommanderEvent::Select, true, 3);
        assert_eq!(s.current, 0);
        s.event(CommanderEvent::RotateClockwise, true, 3);
        s.event(CommanderEvent::Select, true, 3);
        s.event(CommanderEvent::RotateClockwise, true, 3);
        s.event(CommanderEvent::Select, true, 3);
        s.fail_changes = true;
        s.advance(1500, true);
        assert_eq!(s.current, 0);
        assert!(matches!(s.page, LanguagePage::Error { yes: false, .. }));
        s.event(CommanderEvent::RotateClockwise, true, 3);
        s.event(CommanderEvent::Select, true, 3);
        s.fail_changes = false;
        s.advance(1499, true);
        assert_eq!(s.current, 0);
        s.advance(1, true);
        assert_eq!(s.current, 1);
        s.event(CommanderEvent::Back, true, 3);
        assert_eq!(s.page, LanguagePage::Closed);
    }
    #[test]
    fn limits_and_park_loss_revoke_pending_language_changes() {
        let mut s = LanguageSession::default();
        s.open(3);
        for _ in 0..10 {
            s.event(CommanderEvent::RotateClockwise, true, 3);
        }
        assert_eq!(s.page, LanguagePage::List { focus: 2 });
        s.event(CommanderEvent::Select, true, 3);
        s.event(CommanderEvent::RotateClockwise, true, 3);
        s.event(CommanderEvent::Select, true, 3);
        s.advance(u64::MAX, false);
        assert_eq!(s.current, 0);
        assert_eq!(s.page, LanguagePage::List { focus: 0 });
        s.event(CommanderEvent::Select, false, 3);
        assert_eq!(s.page, LanguagePage::List { focus: 0 });
    }
}
