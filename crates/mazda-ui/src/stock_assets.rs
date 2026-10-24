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
    #[must_use]
    pub const fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

#[derive(Debug, Clone, Default)]
pub struct StockAssets(BTreeMap<String, Bitmap>);
impl StockAssets {
    pub const NAMES: [&'static str; 29] = [
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
}
