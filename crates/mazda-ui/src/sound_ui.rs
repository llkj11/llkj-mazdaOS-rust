//! List2 style12/styleOnOff geometry from the owner's firmware.
use super::{fitted, Color, Renderer, UiModel};
use crate::SoundControl;

impl UiModel {
    pub(super) fn sound_pointer_slider(&mut self, row: usize, x: usize) -> bool {
        if !(389..736).contains(&x) {
            return false;
        }
        let Some(&control) = self.sound.profile.controls().get(row) else {
            return false;
        };
        if control.range().is_none() {
            return false;
        }
        self.sound
            .pointer_value(control, x, self.vehicle.is_confirmed_parked());
        true
    }
    pub(super) fn render_sound(&self, renderer: &mut impl Renderer) {
        let first = self.first_visible_row();
        for (visible, (row, control)) in self
            .sound
            .profile
            .controls()
            .iter()
            .enumerate()
            .skip(first)
            .enumerate()
        {
            let y = 133 + visible * 64;
            if y >= 480 {
                break;
            }
            if row == self.row && self.settings_rows {
                self.image(
                    renderer,
                    "list-focus",
                    (68, i32::try_from(y).expect("row"), 732, 64),
                );
            }
            self.image(
                renderer,
                "list-separator",
                (68, i32::try_from(y + 62).expect("row"), 732, 2),
            );
            let allowed = self
                .sound
                .allowed(*control, self.vehicle.is_confirmed_parked());
            let color = if allowed {
                Color::TEXT
            } else {
                Color::rgb(146, 148, 151)
            };
            let slider = control.range().is_some();
            fitted(
                renderer,
                139,
                y + 14,
                self.sound.profile.labels()[row],
                if slider { 241 } else { 542 },
                28,
                color,
            );
            if slider {
                self.render_sound_slider(renderer, *control, y, color);
            } else {
                let checked = self.sound.values.value(*control) == 1;
                let name = match (allowed, checked) {
                    (true, true) => "settings-check",
                    (true, false) => "settings-uncheck",
                    (false, true) => "settings-check-disabled",
                    (false, false) => "settings-uncheck-disabled",
                };
                self.image_natural(renderer, name, 681, i32::try_from(y + 6).expect("row"));
            }
        }
    }
    fn render_sound_slider(
        &self,
        renderer: &mut impl Renderer,
        control: SoundControl,
        y: usize,
        color: Color,
    ) {
        let (min, max) = control.range().expect("slider control");
        let value = self.sound.values.value(control);
        let position = usize::try_from(value - min).expect("bounded value") * 346
            / usize::try_from(max - min).expect("positive range");
        let active = self.sound.editing == Some(control);
        self.image_natural(
            renderer,
            "settings-slider-bg",
            389,
            i32::try_from(y + 35).expect("row"),
        );
        let (start, width, mirrored) = if control.pivot() {
            (position.min(173), position.abs_diff(173), position < 173)
        } else {
            (0, position, false)
        };
        if let Some(bitmap) = self.assets.get(if active {
            "settings-slider-focus"
        } else {
            "settings-slider-fill"
        }) {
            if let Some(fill) = bitmap.centered_portion(width, mirrored) {
                let (w, h) = fill.size();
                renderer.image(
                    389 + i32::try_from(start).expect("slider"),
                    i32::try_from(y + 36).expect("row"),
                    w,
                    h,
                    &fill,
                );
            }
        }
        self.render_sound_marks(renderer, control, y, color);
        self.image_natural(
            renderer,
            if active {
                "settings-knob-focus"
            } else {
                "settings-knob"
            },
            389 + i32::try_from(position).expect("slider") - if active { 22 } else { 11 },
            i32::try_from(y + 42).expect("row") - if active { 22 } else { 11 },
        );
    }
    fn render_sound_marks(
        &self,
        renderer: &mut impl Renderer,
        control: SoundControl,
        y: usize,
        color: Color,
    ) {
        if matches!(
            control,
            SoundControl::Bass | SoundControl::Treble | SoundControl::Alc
        ) {
            let divisions: usize = if control == SoundControl::Alc { 7 } else { 12 };
            for i in 0..=divisions {
                let x = 390 + i * 342 / divisions;
                if divisions == 7 || i != 6 {
                    self.image_natural(
                        renderer,
                        "settings-tick",
                        i32::try_from(x).expect("tick"),
                        i32::try_from(y + 29).expect("row"),
                    );
                }
                if divisions == 7 {
                    let number = i.to_string();
                    renderer.text(
                        x.saturating_sub(crate::text_width(&number, 21) / 2),
                        y + 7,
                        &number,
                        21,
                        color,
                    );
                }
            }
            if divisions == 12 {
                self.image_natural(
                    renderer,
                    "settings-tick-center",
                    561,
                    i32::try_from(y + 26).expect("row"),
                );
                self.image_natural(
                    renderer,
                    "settings-tick-center",
                    561,
                    i32::try_from(y + 49).expect("row"),
                );
            }
        }
        let labels = match control {
            SoundControl::Bass | SoundControl::Treble => ("-6", "0", "+6"),
            SoundControl::Fade => ("Front", "", "Rear"),
            SoundControl::Balance => ("Left", "", "Right"),
            _ => ("", "", ""),
        };
        renderer.text(389, y + 7, labels.0, 21, color);
        renderer.text(
            562 - usize::min(562, crate::text_width(labels.1, 21) / 2),
            y + 7,
            labels.1,
            21,
            color,
        );
        renderer.text(
            736 - usize::min(167, crate::text_width(labels.2, 21)),
            y + 7,
            labels.2,
            21,
            color,
        );
    }
}
