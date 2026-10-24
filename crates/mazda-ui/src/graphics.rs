use crate::typography::text_run;
use crate::Bitmap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(u32);
impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const BACKGROUND: Self = Self::rgb(13, 18, 26);
    pub const NAVIGATION: Self = Self::rgb(17, 24, 34);
    pub const PANEL: Self = Self::rgb(24, 33, 46);
    pub const PANEL_ACTIVE: Self = Self::rgb(35, 49, 65);
    pub const BORDER: Self = Self::rgb(44, 59, 77);
    pub const TEXT: Self = Self::rgb(255, 255, 255);
    pub const TEXT_MUTED: Self = Self::rgb(154, 172, 192);
    pub const ACCENT: Self = Self::rgb(105, 208, 229);
    pub const ACCENT_SOFT: Self = Self::rgb(29, 75, 89);
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self((r as u32) << 16 | (g as u32) << 8 | b as u32)
    }
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}
impl Rect {
    #[must_use]
    pub const fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    #[must_use]
    pub fn contains(self, x: usize, y: usize) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x.saturating_add(self.width)
            && y < self.y.saturating_add(self.height)
    }
}

/// Backend-neutral logical pixels. Text sizes are pixels per em, rather than bitmap multiples.
pub trait Renderer {
    fn size(&self) -> (usize, usize);
    fn clear(&mut self, color: Color);
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn text(&mut self, x: usize, y: usize, text: &str, size: usize, color: Color);
    fn image(&mut self, x: i32, y: i32, width: usize, height: usize, bitmap: &Bitmap);
}

#[derive(Debug, Clone)]
pub struct Framebuffer {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}
impl Framebuffer {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width.saturating_mul(height)],
        }
    }
    #[must_use]
    pub fn mazda_connect() -> Self {
        Self::new(crate::DISPLAY_WIDTH, crate::DISPLAY_HEIGHT)
    }
    #[must_use]
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }
}

impl Renderer for Framebuffer {
    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    fn clear(&mut self, color: Color) {
        self.pixels.fill(color.raw());
    }
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y.min(self.height)..rect.y.saturating_add(rect.height).min(self.height) {
            for x in rect.x.min(self.width)..rect.x.saturating_add(rect.width).min(self.width) {
                self.pixels[y * self.width + x] = color.raw();
            }
        }
    }
    fn image(&mut self, x: i32, y: i32, width: usize, height: usize, bitmap: &Bitmap) {
        if width == 0 || height == 0 || width > 1600 || height > 960 {
            return;
        }
        for dy in 0..height {
            let py = i64::from(y) + i64::try_from(dy).expect("bounded image height");
            let Ok(py) = usize::try_from(py) else {
                continue;
            };
            if py >= self.height {
                continue;
            }
            for dx in 0..width {
                let px = i64::from(x) + i64::try_from(dx).expect("bounded image width");
                let Ok(px) = usize::try_from(px) else {
                    continue;
                };
                if px >= self.width {
                    continue;
                }
                let source =
                    ((dy * bitmap.height / height) * bitmap.width + dx * bitmap.width / width) * 4;
                let rgba = &bitmap.rgba[source..source + 4];
                let pixel = &mut self.pixels[py * self.width + px];
                *pixel = blend(*pixel, Color::rgb(rgba[0], rgba[1], rgba[2]).raw(), rgba[3]);
            }
        }
    }
    fn text(&mut self, x: usize, y: usize, text: &str, size: usize, color: Color) {
        if x >= self.width
            || y >= self.height
            || !(8..=crate::typography::MAX_FONT_SIZE).contains(&size)
        {
            return;
        }
        let run = text_run(text, size);
        let origin_x = i64::try_from(x).expect("visible framebuffer coordinate");
        let origin_y = i64::try_from(y).expect("visible framebuffer coordinate");
        for glyph in &run.glyphs {
            let w = usize::try_from(glyph.image.placement.width).expect("bounded glyph width");
            for (index, alpha) in glyph.image.data.iter().copied().enumerate() {
                if alpha == 0 || w == 0 {
                    continue;
                }
                let px = origin_x
                    + i64::from(glyph.x)
                    + i64::try_from(index % w).expect("glyph coordinate");
                let py = origin_y
                    + i64::from(glyph.y)
                    + i64::try_from(index / w).expect("glyph coordinate");
                let (Ok(px), Ok(py)) = (usize::try_from(px), usize::try_from(py)) else {
                    continue;
                };
                if px < self.width && py < self.height {
                    let pixel = &mut self.pixels[py * self.width + px];
                    *pixel = blend(*pixel, color.raw(), alpha);
                }
            }
        }
    }
}

fn blend(background: u32, foreground: u32, alpha: u8) -> u32 {
    let a = u32::from(alpha);
    [0, 8, 16]
        .into_iter()
        .map(|shift| {
            ((((foreground >> shift) & 255) * a + ((background >> shift) & 255) * (255 - a) + 127)
                / 255)
                << shift
        })
        .fold(0, |pixel, channel| pixel | channel)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outlines_are_antialiased_and_clip_extreme_coordinates() {
        let mut frame = Framebuffer::new(100, 48);
        frame.text(2, 0, "Été", 26, Color::TEXT);
        assert!(frame
            .pixels()
            .iter()
            .any(|p| *p != 0 && *p != Color::TEXT.raw()));
        let before = frame.pixels().to_vec();
        frame.text(usize::MAX, 0, "A", 24, Color::TEXT);
        frame.text(0, usize::MAX, "A", 24, Color::TEXT);
        frame.text(0, 0, "A", usize::MAX, Color::TEXT);
        assert_eq!(frame.pixels(), before);
    }
    #[test]
    fn unsupported_characters_have_a_visible_consistent_fallback() {
        let mut a = Framebuffer::new(48, 48);
        let mut b = a.clone();
        a.text(0, 0, "🚗", 24, Color::TEXT);
        b.text(0, 0, "?", 24, Color::TEXT);
        assert_eq!(a.pixels(), b.pixels());
        assert!(a.pixels().iter().any(|p| *p != 0));
    }
    #[test]
    fn rectangles_clip_and_zero_surfaces_accept_text() {
        let mut frame = Framebuffer::new(4, 4);
        frame.fill_rect(Rect::new(3, 3, usize::MAX, usize::MAX), Color::ACCENT);
        assert_eq!(frame.pixels().iter().filter(|p| **p != 0).count(), 1);
        Framebuffer::new(0, 0).text(0, 0, "A", 24, Color::TEXT);
    }
    #[test]
    fn owner_images_blend_alpha_scale_and_clip_negative_positions() {
        let mut data = b"MZRGBA01".to_vec();
        data.extend(1_u32.to_le_bytes());
        data.extend(1_u32.to_le_bytes());
        data.extend([255, 0, 0, 128]);
        let bitmap = Bitmap::decode(&data).unwrap();
        let mut frame = Framebuffer::new(2, 2);
        frame.image(-1, 0, 2, 1, &bitmap);
        assert_eq!(frame.pixels(), [0x0080_0000, 0, 0, 0]);
        frame.image(i32::MIN, i32::MAX, 2, 2, &bitmap);
        assert_eq!(frame.pixels(), [0x0080_0000, 0, 0, 0]);
        frame.image(0, 0, usize::MAX, usize::MAX, &bitmap);
        assert_eq!(frame.pixels(), [0x0080_0000, 0, 0, 0]);
    }
}
