//! Owner-supplied artwork decoded offline, never downloaded or executed by the UI.
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Bitmap {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) rgba: Vec<u8>,
}
impl Bitmap {
    pub(crate) fn with_opacity(&self, opacity: u8) -> Self {
        let mut copy = self.clone();
        for pixel in copy.rgba.as_chunks_mut::<4>().0 {
            pixel[3] = u8::try_from(u16::from(pixel[3]) * u16::from(opacity) / 255)
                .expect("bounded alpha");
        }
        copy
    }
    /// Accept only our bounded raw RGBA format, produced by the offline importer.
    #[must_use]
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.get(..8)? != b"MZRGBA01" {
            return None;
        }
        let width = usize::try_from(u32::from_le_bytes(bytes.get(8..12)?.try_into().ok()?)).ok()?;
        let height =
            usize::try_from(u32::from_le_bytes(bytes.get(12..16)?.try_into().ok()?)).ok()?;
        if !(1..=1600).contains(&width)
            || !(1..=960).contains(&height)
            || bytes.len() != 16 + width * height * 4
        {
            return None;
        }
        Some(Self {
            width,
            height,
            rgba: bytes[16..].to_vec(),
        })
    }
    pub(crate) fn adjusted(&self, brightness: i8, contrast: i8) -> Self {
        let mut result = self.clone();
        for pixel in result.rgba.as_chunks_mut::<4>().0 {
            for channel in &mut pixel[..3] {
                let value = (i16::from(*channel) - 128) * (10 + i16::from(contrast)) / 10
                    + 128
                    + i16::from(brightness) * 12;
                *channel = u8::try_from(value.clamp(0, 255)).unwrap_or(0);
            }
        }
        result
    }
    pub(crate) fn left_portion(&self, width: usize) -> Option<Self> {
        let width = width.min(self.width);
        if width == 0 {
            return None;
        }
        let mut rgba = Vec::with_capacity(width * self.height * 4);
        for row in self.rgba.chunks_exact(self.width * 4) {
            rgba.extend_from_slice(&row[..width * 4]);
        }
        Some(Self {
            width,
            height: self.height,
            rgba,
        })
    }
    /// CSS centered background cropping, optionally mirrored for a left pivot fill.
    pub(crate) fn centered_portion(&self, width: usize, mirrored: bool) -> Option<Self> {
        let width = width.min(self.width);
        if width == 0 {
            return None;
        }
        let left = (self.width - width) / 2;
        let mut rgba = Vec::with_capacity(width * self.height * 4);
        for row in self.rgba.chunks_exact(self.width * 4) {
            let region = &row[left * 4..(left + width) * 4];
            if mirrored {
                for pixel in region.as_chunks::<4>().0.iter().rev() {
                    rgba.extend_from_slice(pixel);
                }
            } else {
                rgba.extend_from_slice(region);
            }
        }
        Some(Self {
            width,
            height: self.height,
            rgba,
        })
    }
    #[must_use]
    pub const fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

#[derive(Debug, Clone, Default)]
pub struct StockAssets {
    images: BTreeMap<String, Bitmap>,
    pub(crate) clock_zones: Vec<String>,
    pub(crate) legal_lines: Vec<String>,
    pub(crate) languages: crate::languages::LanguageCatalog,
}
impl StockAssets {
    pub const NAMES: [&'static str; 94] = [
        "bt-remove",
        "bt-phone",
        "bt-audio",
        "bt-check",
        "bt-three",
        "bt-three-focus",
        "bt-keypad-bg",
        "bt-keypad-key",
        "bt-keypad-focus",
        "bt-keypad-clear",
        "bt-keypad-cancel",
        "bt-keypad-ok",
        "bt-keypad-back",
        "update-spinner",
        "update-progress-bg",
        "update-progress-fill",
        "update-title-separator",
        "language-voice",
        "left-back",
        "left-back-disabled",
        "system-dialog-bg",
        "system-dialog-ok",
        "system-meter",
        "legal-background",
        "legal-focus",
        "legal-header",
        "legal-scroll-bg",
        "legal-scroll",
        "legal-shadow",
        "clock-background",
        "clock-colon",
        "clock-panel",
        "clock-arch",
        "clock-separator",
        "clock-focus",
        "clock-back",
        "clock-minus",
        "clock-minus-disabled",
        "clock-plus",
        "clock-plus-disabled",
        "clock-ampm",
        "clock-ampm-disabled",
        "settings-radio",
        "settings-radio-empty",
        "settings-radio-disabled",
        "settings-radio-empty-disabled",
        "settings-title",
        "settings-check",
        "settings-uncheck",
        "settings-check-disabled",
        "settings-uncheck-disabled",
        "settings-tick",
        "settings-tick-center",
        "compass-bg",
        "compass-foreground",
        "compass-rail",
        "compass-step",
        "compass-needle",
        "compass-line",
        "tab-prev",
        "tab-next",
        "tab",
        "tab-focus",
        "background",
        "status",
        "ellipse",
        "glow",
        "home",
        "list-background",
        "list-focus",
        "list-separator",
        "back",
        "Apps",
        "Apps_Focus",
        "Audio",
        "Audio_Focus",
        "Com",
        "Com_Focus",
        "Nav",
        "Nav_Focus",
        "Set",
        "Set_Focus",
        "settings-test",
        "settings-inline",
        "settings-inline-selected",
        "settings-slider-bg",
        "settings-slider-fill",
        "settings-slider-focus",
        "settings-knob",
        "settings-knob-focus",
        "settings-dialog-bg",
        "settings-dialog-button",
        "settings-dialog-button-focus",
        "settings-off",
    ];
    pub fn insert(&mut self, name: &str, bitmap: Bitmap) {
        self.images.insert(name.to_owned(), bitmap);
    }
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Bitmap> {
        self.images.get(name)
    }
    #[must_use]
    pub fn is_complete(&self) -> bool {
        Self::NAMES
            .iter()
            .all(|name| self.images.contains_key(*name))
            && !self.clock_zones.is_empty()
            && !self.legal_lines.is_empty()
            && !self.languages.packs.is_empty()
    }
    pub fn load_languages(&mut self, bytes: &[u8]) -> bool {
        let Some(catalog) = crate::languages::LanguageCatalog::decode(bytes) else {
            return false;
        };
        self.languages = catalog;
        true
    }
    /// Owner-only disclosure text, never a script or external link to follow.
    pub fn load_legal_text(&mut self, bytes: &[u8]) -> bool {
        if bytes.len() > 132_000 {
            return false;
        }
        let Some(text) = std::str::from_utf8(bytes)
            .ok()
            .and_then(|s| s.strip_prefix("MZLEGAL1\n"))
        else {
            return false;
        };
        if text.is_empty()
            || text.chars().count() > 32_768
            || text.chars().any(|c| c.is_control() && c != '\n')
        {
            return false;
        }
        self.legal_lines = crate::typography::wrapped_lines(text, 640, 28);
        true
    }
    /// Bounded owner-derived text, with no JavaScript, expression or timezone execution.
    pub fn load_clock_zones(&mut self, bytes: &[u8]) -> bool {
        if bytes.len() > 132_000 {
            return false;
        }
        let Some(text) = std::str::from_utf8(bytes)
            .ok()
            .and_then(|s| s.strip_prefix("MZTIMEZONES1\n"))
        else {
            return false;
        };
        let rows: Vec<_> = text.split_terminator('\n').collect();
        if rows.is_empty()
            || rows.len() > 128
            || rows.iter().any(|row| {
                row.is_empty() || row.chars().count() > 256 || row.chars().any(char::is_control)
            })
        {
            return false;
        }
        self.clock_zones = rows.into_iter().map(str::to_owned).collect();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_decoder_rejects_unbounded_and_truncated_data() {
        assert!(Bitmap::decode(b"arbitrary firmware").is_none());
        for (width, height) in [(0_u32, 1_u32), (1601, 1), (1, 961), (u32::MAX, u32::MAX)] {
            let mut data = b"MZRGBA01".to_vec();
            data.extend(width.to_le_bytes());
            data.extend(height.to_le_bytes());
            assert!(Bitmap::decode(&data).is_none());
        }
        let mut data = b"MZRGBA01".to_vec();
        data.extend(1_u32.to_le_bytes());
        data.extend(1_u32.to_le_bytes());
        assert!(Bitmap::decode(&data).is_none());
        data.extend([255, 0, 0, 255]);
        assert_eq!(Bitmap::decode(&data).unwrap().size(), (1, 1));
    }
    #[test]
    fn preview_adjustment_preserves_alpha_and_crop_keeps_row_stride() {
        let original = Bitmap {
            width: 2,
            height: 2,
            rgba: vec![
                10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160,
            ],
        };
        assert_eq!(original.adjusted(0, 0).rgba, original.rgba);
        let changed = original.adjusted(5, 5);
        assert_eq!(changed.rgba[3], 40);
        assert_eq!(changed.rgba[15], 160);
        assert_ne!(changed.rgba, original.rgba);
        let crop = original.left_portion(1).unwrap();
        assert_eq!(crop.rgba, vec![10, 20, 30, 40, 90, 100, 110, 120]);
        assert!(original.left_portion(0).is_none());
        assert_eq!(
            original.left_portion(usize::MAX).unwrap().rgba,
            original.rgba
        );
        assert_eq!(
            original.centered_portion(2, true).unwrap().rgba,
            vec![50, 60, 70, 80, 10, 20, 30, 40, 130, 140, 150, 160, 90, 100, 110, 120]
        );
        assert_eq!(
            original.centered_portion(1, false).unwrap().rgba,
            vec![10, 20, 30, 40, 90, 100, 110, 120]
        );
        assert!(original.centered_portion(0, true).is_none());
    }
    #[test]
    fn owner_zone_text_is_bounded_and_invalid_input_preserves_existing_names() {
        let mut assets = StockAssets::default();
        assert!(assets.load_clock_zones(b"MZTIMEZONES1\nEastern\nPacific\n"));
        let before = assets.clock_zones.clone();
        for invalid in [
            b"bad header".as_slice(),
            b"MZTIMEZONES1\n",
            b"MZTIMEZONES1\nunsafe\tlabel\n",
            b"MZTIMEZONES1\n\xff\n",
        ] {
            assert!(!assets.load_clock_zones(invalid));
            assert_eq!(assets.clock_zones, before);
        }
        assert!(!assets.load_clock_zones(format!("MZTIMEZONES1\n{}\n", "x".repeat(257)).as_bytes()));
        assert!(
            !assets.load_clock_zones(format!("MZTIMEZONES1\n{}", "label\n".repeat(129)).as_bytes())
        );
    }
    #[test]
    fn disclosure_text_is_bounded_wrapped_and_invalid_input_preserves_previous_lines() {
        let mut assets = StockAssets::default();
        assert!(assets.load_legal_text(b"MZLEGAL1\nFirst paragraph.\n\nSecond paragraph."));
        assert_eq!(
            assets.legal_lines,
            ["First paragraph.", "", "Second paragraph."]
        );
        let before = assets.legal_lines.clone();
        for input in [
            b"wrong".as_slice(),
            b"MZLEGAL1\n",
            b"MZLEGAL1\nunsafe\x00text",
            b"MZLEGAL1\n\xff",
        ] {
            assert!(!assets.load_legal_text(input));
            assert_eq!(assets.legal_lines, before);
        }
        assert!(!assets.load_legal_text(format!("MZLEGAL1\n{}", "x".repeat(32769)).as_bytes()));
        assert!(assets.load_legal_text(format!("MZLEGAL1\n{}", "é".repeat(400)).as_bytes()));
        assert!(assets
            .legal_lines
            .iter()
            .all(|line| crate::text_width(line, 28) <= 640 && line.chars().count() <= 256));
    }
    #[test]
    fn css_half_opacity_preserves_rgb_and_bounds_alpha_without_changing_source() {
        let bitmap = Bitmap {
            width: 2,
            height: 1,
            rgba: vec![10, 20, 30, 255, 40, 50, 60, 80],
        };
        assert_eq!(
            bitmap.with_opacity(128).rgba,
            [10, 20, 30, 128, 40, 50, 60, 40]
        );
        assert_eq!(bitmap.rgba, [10, 20, 30, 255, 40, 50, 60, 80]);
    }
}
