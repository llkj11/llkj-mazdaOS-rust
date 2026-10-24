//! Shared source-sized Dialog3 typography, centered within the original boxes.
use crate::{text_width, Color, Renderer};
pub(super) fn dialog_block(r: &mut impl Renderer, text: &str, y: usize, height: usize) {
    // Keep the source center at x=400; reserve ink clearance from the shared Back arc.
    let lines = crate::typography::fit_block(text, 616, height / 38, 35);
    let top = y + height.saturating_sub(lines.0.len() * 38) / 2;
    for (index, line) in lines.0.iter().enumerate() {
        let label = crate::typography::ellipsized(line, 616, lines.1);
        r.text(
            77 + 646_usize.saturating_sub(text_width(&label, lines.1)) / 2,
            top + index * 38,
            &label,
            lines.1,
            Color::TEXT,
        );
    }
}
pub(super) fn dialog_line(r: &mut impl Renderer, text: &str, y: usize) {
    let label = crate::typography::ellipsized(text, 616, 35);
    r.text(
        77 + 646_usize.saturating_sub(text_width(&label, 35)) / 2,
        y,
        &label,
        35,
        Color::TEXT,
    );
}
