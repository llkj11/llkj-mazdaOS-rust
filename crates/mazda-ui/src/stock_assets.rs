//! Owner-supplied artwork decoded offline, never downloaded or executed by the UI.
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Bitmap {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) rgba: Vec<u8>,
}
impl Bitmap {
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
    #[must_use]
    pub const fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

#[derive(Debug, Clone, Default)]
pub struct StockAssets(BTreeMap<String, Bitmap>);
impl StockAssets {
    pub const NAMES: [&'static str; 41] = [
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
        self.0.insert(name.to_owned(), bitmap);
    }
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Bitmap> {
        self.0.get(name)
    }
    #[must_use]
    pub fn is_complete(&self) -> bool {
        Self::NAMES.iter().all(|name| self.0.contains_key(*name))
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
    }
}
