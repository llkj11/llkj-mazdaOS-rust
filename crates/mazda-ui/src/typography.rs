//! Bundled Inter outlines, shaping and bounded caches. No system font or network lookup.
// Font sizes are capped at 96 logical pixels and runs at 256 Unicode scalars.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use swash::scale::{image::Image, Render, ScaleContext, Source};
use swash::shape::ShapeContext;
use swash::zeno::Format;
use swash::{FontRef, GlyphId};

const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/Inter.ttf");
const MAX_RUNS: usize = 64;
const MAX_GLYPHS: usize = 512;
pub(crate) const MAX_CHARACTERS: usize = 256;

static FONT: OnceLock<FontRef<'static>> = OnceLock::new();
static REFERENCE_FONT: OnceLock<Vec<u8>> = OnceLock::new();

/// Offline comparison only: install an owner-supplied reference font before any text is rendered.
/// The normal simulator always uses bundled Inter. No font instructions are executed.
#[must_use]
pub fn install_reference_font(bytes: Vec<u8>) -> bool {
    if FONT.get().is_some() || bytes.len() > 32_000_000 || FontRef::from_index(&bytes, 0).is_none()
    {
        return false;
    }
    REFERENCE_FONT.set(bytes).is_ok()
}
fn font() -> FontRef<'static> {
    *FONT.get_or_init(|| {
        FontRef::from_index(REFERENCE_FONT.get().map_or(FONT_BYTES, Vec::as_slice), 0)
            .expect("validated bundled or owner reference font")
    })
}

pub(crate) struct PositionedGlyph {
    pub x: i32,
    pub y: i32,
    pub image: Arc<Image>,
}

pub(crate) struct TextRun {
    pub width: usize,
    pub glyphs: Vec<PositionedGlyph>,
}

#[derive(Default)]
struct TextCache {
    shape: ShapeContext,
    scale: ScaleContext,
    runs: HashMap<(String, usize), Arc<TextRun>>,
    glyphs: HashMap<(GlyphId, usize), Arc<Image>>,
}

impl TextCache {
    fn run(&mut self, text: &str, size: usize) -> Arc<TextRun> {
        let size = size.clamp(8, 96);
        let charmap = font().charmap();
        let text: String = text
            .chars()
            .take(MAX_CHARACTERS)
            .map(|ch| {
                if ch.is_control() {
                    ' '
                } else if charmap.map(ch) == 0 {
                    '?'
                } else {
                    ch
                }
            })
            .collect();
        let key = (text, size);
        if let Some(run) = self.runs.get(&key) {
            return Arc::clone(run);
        }
        if self.runs.len() >= MAX_RUNS || self.glyphs.len() >= MAX_GLYPHS {
            self.runs.clear();
            self.glyphs.clear();
        }

        let px = size as f32;
        let weight = if size >= 20 { 500.0 } else { 450.0 };
        let variations = [("wght", weight), ("opsz", px)];
        let mut shaper = self
            .shape
            .builder(font())
            .size(px)
            .variations(variations)
            .build();
        let baseline = shaper.metrics().ascent.ceil() as i32;
        shaper.add_str(&key.0);
        let mut pen = 0.0_f32;
        let mut positions = Vec::new();
        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                positions.push((glyph.id, pen + glyph.x, glyph.y));
                pen += glyph.advance;
            }
        });

        let mut glyphs = Vec::with_capacity(positions.len());
        for (id, x, y) in positions {
            let image = if let Some(image) = self.glyphs.get(&(id, size)) {
                Arc::clone(image)
            } else {
                let mut scaler = self
                    .scale
                    .builder(font())
                    .size(px)
                    .hint(false)
                    .variations(variations)
                    .build();
                let Some(image) = Render::new(&[Source::Outline])
                    .format(Format::Alpha)
                    .render(&mut scaler, id)
                else {
                    continue;
                };
                let image = Arc::new(image);
                self.glyphs.insert((id, size), Arc::clone(&image));
                image
            };
            glyphs.push(PositionedGlyph {
                x: x.round() as i32 + image.placement.left,
                y: baseline - y.round() as i32 - image.placement.top,
                image,
            });
        }
        let run = Arc::new(TextRun {
            width: pen.ceil() as usize,
            glyphs,
        });
        self.runs.insert(key, Arc::clone(&run));
        run
    }
}

pub(crate) fn text_run(text: &str, size: usize) -> Arc<TextRun> {
    static CACHE: OnceLock<Mutex<TextCache>> = OnceLock::new();
    CACHE
        .get_or_init(|| Mutex::new(TextCache::default()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .run(text, size)
}

#[must_use]
pub fn text_width(text: &str, size: usize) -> usize {
    text_run(text, size).width
}

#[must_use]
pub fn fit_text(text: &str, max_width: usize, preferred_size: usize) -> (String, usize) {
    let text: String = text.chars().take(MAX_CHARACTERS).collect();
    let preferred_size = preferred_size.clamp(8, 96);
    let minimum = preferred_size.min(16);
    for size in (minimum..=preferred_size).rev() {
        if text_width(&text, size) <= max_width {
            return (text, size);
        }
    }
    let mut text = text;
    while !text.is_empty() {
        text.pop();
        let candidate = format!("{text}…");
        if text_width(&candidate, minimum) <= max_width {
            return (candidate, minimum);
        }
    }
    (String::new(), minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proportional_outlines_and_real_metrics_fit_unicode_text() {
        assert!(text_width("WWWW", 20) > text_width("iiii", 20) * 2);
        assert!(text_width("Vehicle", 28) > text_width("Vehicle", 16));
        for text in [
            "Everything in Its Right Place",
            "Été à Montréal",
            "A".repeat(256).as_str(),
        ] {
            let (fitted, size) = fit_text(text, 224, 24);
            assert!(text_width(&fitted, size) <= 224);
        }
        assert_eq!(fit_text("Hello", 0, 24).0, "");
    }

    #[test]
    fn cache_reuses_runs_and_bounds_metadata_and_storage() {
        let mut cache = TextCache::default();
        let a = cache.run("Home", 20);
        let b = cache.run("Home", 20);
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(
            cache.run(&"W".repeat(1_000_000), 16).width,
            cache.run(&"W".repeat(256), 16).width
        );
        for index in 0..200 {
            cache.run(&format!("Title {index}"), 20);
        }
        assert!(cache.runs.len() <= MAX_RUNS);
        assert!(cache.glyphs.len() <= MAX_GLYPHS + MAX_CHARACTERS);
    }
}
