// ============================================================================
// Module:       number_field
// Description:  A Material 3 text field for numbers: drag, step keys, typing.
//
// Dependencies: egui; crate::input, crate::a11y, crate::tokens, crate::MaterialTheme
// ============================================================================

//! A numeric text field in the Material 3 filled or outlined style.
//!
//! [`MaterialNumberField`] edits any numeric type egui's `DragValue` can
//! (`f32`, `f64`, `i32`, `u8`, ...), drawn with the same container, colours
//! and label as [`crate::MaterialInput`] (Compose's `FilledTextFieldTokens`
//! and `OutlinedTextFieldTokens`), at the default 56 dp or a denser height
//! down to 32 dp:
//!
//! - **Drag** the field sideways (or up and down) to change the value one
//!   step per few points; Shift drags ten steps at a time.
//! - **Keyboard**: the field is one tab stop. Focused, it is a text field
//!   with the value selected; Up and Down change it by one step, Page Up and
//!   Page Down by ten. Enter or leaving the field commits the typed number,
//!   clamped to the range and rounded to the decimals; text that is not a
//!   number is dropped. Escape restores the value the field had on focus.
//! - **Screen readers** see a spin button with its value, range and step,
//!   and can increment, decrement or set it.

use std::ops::RangeInclusive;

use egui::emath::Numeric;
use egui::text::CCursorRange;
use egui::{
    self, Align2, Color32, CursorIcon, Key, Modifiers, Pos2, Rect, Response, Role, Sense, TextEdit,
    Ui, Vec2, Widget,
};

use crate::input::{
    container_height_for_density, container_shape, field_colors, floated_label_pos, notch_for,
    paint_label, FieldState, InputVariant, LabelGeometry, HORIZONTAL_PADDING, MIN_DENSITY,
};
use crate::tokens::TypeRole;
use crate::MaterialTheme;

/// The width of a number field without [`MaterialNumberField::width`] or
/// [`MaterialNumberField::fill_width`].
pub const DEFAULT_WIDTH: f32 = 120.0;
/// Points of drag per step by default.
pub const DRAG_POINTS_PER_STEP: f32 = 4.0;
/// Space between a prefix or suffix and the number.
const AFFIX_PADDING: f32 = 2.0;
/// Steps Page Up and Page Down (and a Shift drag) take at a time.
const PAGE_STEPS: f64 = 10.0;

/// A Material 3 text field for a number, with drag-to-change, step keys
/// and validated typing.
///
/// ```rust
/// # use egui_material3::MaterialNumberField;
/// # egui::__run_test_ui(|ui| {
/// let mut gain_db = -3.0_f64;
/// ui.add(MaterialNumberField::new(&mut gain_db)
///     .label("Gain")
///     .range(-60.0..=12.0)
///     .step(0.5)
///     .decimals(1)
///     .suffix("dB"));
///
/// let mut fps = 30_u32;
/// ui.add(MaterialNumberField::new(&mut fps)
///     .label("Frame rate")
///     .range(1..=240)
///     .compact(true));
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct MaterialNumberField<'a, N: Numeric> {
    value: &'a mut N,
    variant: InputVariant,
    label: Option<String>,
    range: RangeInclusive<f64>,
    step: f64,
    decimals: Option<usize>,
    prefix: Option<String>,
    suffix: Option<String>,
    density: i8,
    width: Option<f32>,
    fill_width: bool,
    enabled: bool,
    drag_points_per_step: f32,
    id_salt: Option<egui::IdSalt>,
}

/// What a number field remembers between frames.
#[derive(Clone, Debug, Default)]
struct NumberFieldState {
    /// The field had focus (was a text field) last frame.
    editing: bool,
    /// The text being typed.
    text: String,
    /// The value when the field took focus, for Escape.
    original: f64,
    /// Drag distance not yet turned into steps.
    drag: f32,
}

impl<'a, N: Numeric> MaterialNumberField<'a, N> {
    /// An outlined number field editing `value`.
    pub fn new(value: &'a mut N) -> Self {
        Self {
            value,
            variant: InputVariant::Outlined,
            label: None,
            range: N::MIN.to_f64()..=N::MAX.to_f64(),
            step: 1.0,
            decimals: None,
            prefix: None,
            suffix: None,
            density: 0,
            width: None,
            fill_width: false,
            enabled: true,
            drag_points_per_step: DRAG_POINTS_PER_STEP,
            id_salt: None,
        }
    }

    /// A filled number field editing `value`.
    pub fn filled(value: &'a mut N) -> Self {
        Self::new(value).variant(InputVariant::Filled)
    }

    /// An outlined number field editing `value` (the same as [`Self::new`]).
    pub fn outlined(value: &'a mut N) -> Self {
        Self::new(value)
    }

    /// Filled or outlined.
    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The label: floated above the number at the default density, and the
    /// accessible name at every density.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The allowed values (default: the whole type). Typed, dragged and
    /// stepped values are clamped to it; a value already outside it is
    /// left alone until it is changed.
    pub fn range(mut self, range: RangeInclusive<N>) -> Self {
        let (a, b) = (range.start().to_f64(), range.end().to_f64());
        self.range = a.min(b)..=a.max(b);
        self
    }

    /// How much one drag step, Up or Down changes the value (default 1).
    /// Non-finite or zero steps are ignored.
    pub fn step(mut self, step: f64) -> Self {
        if step.is_finite() && step != 0.0 {
            self.step = step.abs();
        }
        self
    }

    /// Show and round to exactly this many decimals. By default an integer
    /// type shows none and a float shows as many as it needs.
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = Some(decimals.min(15));
        self
    }

    /// Text before the number, such as `$`.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Text after the number, such as `px`.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Density from `0` (56 dp) to `-3` (32 dp), as
    /// [`crate::MaterialInput::density`]; below zero the label is not
    /// painted.
    pub fn density(mut self, density: i8) -> Self {
        self.density = density.clamp(MIN_DENSITY, 0);
        self
    }

    /// Compact mode: 32 dp, no painted label (density `-3`).
    pub fn compact(mut self, compact: bool) -> Self {
        self.density = if compact { MIN_DENSITY } else { 0 };
        self
    }

    /// A fixed width (default [`DEFAULT_WIDTH`]).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Fill the available width, overriding [`Self::width`].
    pub fn fill_width(mut self, fill: bool) -> Self {
        self.fill_width = fill;
        self
    }

    /// Whether the field can be changed.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Points of drag per step (default [`DRAG_POINTS_PER_STEP`]).
    pub fn drag_points_per_step(mut self, points: f32) -> Self {
        if points.is_finite() && points > 0.0 {
            self.drag_points_per_step = points;
        }
        self
    }

    /// Derive the field's id from `salt` instead of its position in the
    /// layout.
    pub fn id_salt(mut self, salt: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(egui::IdSalt::new(salt));
        self
    }
}

/// How a field's numbers are rounded, shown and read back.
#[derive(Clone, Debug)]
struct NumberFormat {
    range: RangeInclusive<f64>,
    step: f64,
    decimals: Option<usize>,
    integral: bool,
    prefix: Option<String>,
    suffix: Option<String>,
}

impl NumberFormat {
    /// The decimals a value is rounded to after a step: the fixed decimals,
    /// or enough for both the step and the value.
    fn round(&self, value: f64) -> f64 {
        if self.integral {
            return value.round();
        }
        let decimals = self
            .decimals
            .unwrap_or_else(|| decimals_of(self.step).max(decimals_of(value)));
        egui::emath::round_to_decimals(value, decimals)
    }

    fn clamp(&self, value: f64) -> f64 {
        value.clamp(*self.range.start(), *self.range.end())
    }

    /// `value` moved by `steps` steps, rounded and clamped.
    fn stepped(&self, value: f64, steps: f64) -> f64 {
        self.clamp(self.round(value + steps * self.step))
    }

    /// The number as shown, without prefix or suffix.
    fn format(&self, value: f64) -> String {
        match (self.integral, self.decimals) {
            (true, _) => format!("{}", value.round()),
            (false, Some(decimals)) => format!("{value:.decimals$}"),
            (false, None) => format!("{value}"),
        }
    }

    /// The number typed in `text` (prefix, suffix and spaces allowed),
    /// clamped and rounded; `None` when it is not a number.
    fn parse(&self, text: &str) -> Option<f64> {
        self.parse_unclamped(text).map(|v| self.clamp(v))
    }

    /// [`Self::parse`] without the clamp.
    fn parse_unclamped(&self, text: &str) -> Option<f64> {
        let mut text = text.trim();
        if let Some(prefix) = self.prefix.as_deref().filter(|p| !p.is_empty()) {
            text = text.strip_prefix(prefix).unwrap_or(text);
        }
        if let Some(suffix) = self.suffix.as_deref().filter(|s| !s.is_empty()) {
            text = text.strip_suffix(suffix).unwrap_or(text);
        }
        let cleaned: String = text
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| if c == '\u{2212}' { '-' } else { c })
            .collect();
        let value: f64 = cleaned.parse().ok()?;
        if !value.is_finite() {
            return None;
        }
        let value = match (self.integral, self.decimals) {
            (true, _) => value.round(),
            (false, Some(decimals)) => egui::emath::round_to_decimals(value, decimals),
            (false, None) => value,
        };
        Some(value)
    }
}

/// The fewest decimals (at most 15) that write `value` exactly enough.
fn decimals_of(value: f64) -> usize {
    let value = value.abs();
    (0..15)
        .find(|&d| {
            let scaled = value * 10f64.powi(d as i32);
            (scaled - scaled.round()).abs() < 1e-6
        })
        .unwrap_or(15)
}

impl<N: Numeric> Widget for MaterialNumberField<'_, N> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            value,
            variant,
            label,
            range,
            step,
            decimals,
            prefix,
            suffix,
            density,
            width,
            fill_width,
            enabled,
            drag_points_per_step,
            id_salt,
        } = self;
        let format = NumberFormat {
            range: range.clone(),
            step,
            decimals,
            integral: N::INTEGRAL,
            prefix: prefix.clone(),
            suffix: suffix.clone(),
        };

        let ctx = ui.ctx().clone();
        let theme = MaterialTheme::of(&ctx);
        let enabled = enabled && ui.is_enabled();
        let body = TypeRole::BodyLarge.style();
        let font = body.font_id(&ctx);

        let width = if fill_width {
            ui.available_width()
        } else {
            width.unwrap_or(DEFAULT_WIDTH)
        };
        let height = container_height_for_density(density);
        let dense = density < 0;
        let painted_label = label.as_deref().filter(|_| !dense);

        let (rect, allocated) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        // Not the allocation's id: that one senses hover only, and a
        // hover-only widget with the focused id would drop its focus.
        let id = id_salt
            .map_or(allocated.id, |salt| ui.make_persistent_id(salt))
            .with("number_field");
        let focused = enabled
            && ui.memory_mut(|m| {
                m.interested_in_focus(id, ui.layer_id());
                m.has_focus(id)
            });
        let mut state = ctx
            .data(|d| d.get_temp::<NumberFieldState>(id))
            .unwrap_or_default();

        let old = value.to_f64();
        let mut current = old;
        let escape = ui.input(|i| i.key_pressed(Key::Escape));

        // Focus just arrived: start editing the shown value, all selected.
        if focused && !state.editing {
            state.editing = true;
            state.text = format.format(current);
            state.original = current;
            let mut edit_state = TextEdit::load_state(&ctx, id).unwrap_or_default();
            edit_state
                .cursor
                .set_char_range(Some(CCursorRange::select_all_str(&state.text)));
            edit_state.store(&ctx, id);
        }
        // Focus left (Tab, a click elsewhere): commit what was typed.
        if !focused && state.editing {
            state.editing = false;
            if escape {
                current = state.original;
            } else if let Some(typed) = format.parse(&state.text) {
                current = typed;
            }
        }

        // Steps from the keyboard and from assistive technology.
        let steps = ui.input_mut(|i| {
            use egui::accesskit::Action;
            let mut steps = 0.0;
            if focused {
                let count = |i: &mut egui::InputState, key| {
                    i.count_and_consume_key(Modifiers::NONE, key) as f64
                };
                steps += count(i, Key::ArrowUp) - count(i, Key::ArrowDown);
                steps += PAGE_STEPS * (count(i, Key::PageUp) - count(i, Key::PageDown));
            }
            if enabled {
                steps += i.num_accesskit_action_requests(id, Action::Increment) as f64
                    - i.num_accesskit_action_requests(id, Action::Decrement) as f64;
            }
            steps
        });
        if steps != 0.0 {
            // Step from what is typed when it is a number.
            let base = if focused {
                format.parse(&state.text).unwrap_or(current)
            } else {
                current
            };
            current = format.stepped(base, steps);
            if focused {
                state.text = format.format(current);
            }
        }
        if enabled {
            let set_value = ui.input(|i| {
                use egui::accesskit::{Action, ActionData};
                i.accesskit_action_requests(id, Action::SetValue)
                    .filter_map(|r| match r.data {
                        Some(ActionData::NumericValue(v)) => Some(v),
                        Some(_) | None => None,
                    })
                    .last()
            });
            if let Some(v) = set_value.filter(|v| v.is_finite()) {
                current = format.clamp(format.round(v));
                if focused {
                    state.text = format.format(current);
                }
            }
        }

        // Layout: the number under a floated filled label, else centred.
        let measure_painter = ui.painter().clone();
        let measure = |s: &str| {
            measure_painter
                .layout_no_wrap(s.to_owned(), font.clone(), Color32::WHITE)
                .size()
                .x
        };
        let prefix_width = prefix
            .as_deref()
            .map_or(0.0, |p| measure(p) + AFFIX_PADDING);
        let suffix_width = suffix
            .as_deref()
            .map_or(0.0, |s| measure(s) + AFFIX_PADDING);
        let input_top = if variant == InputVariant::Filled && painted_label.is_some() {
            rect.top() + 24.0
        } else {
            rect.center().y - body.line_height / 2.0
        };
        let input_rect = Rect::from_min_size(
            Pos2::new(rect.left() + HORIZONTAL_PADDING + prefix_width, input_top),
            Vec2::new(
                (width - 2.0 * HORIZONTAL_PADDING - prefix_width - suffix_width).max(1.0),
                body.line_height,
            ),
        );
        let container_slot = ui.painter().add(egui::Shape::Noop);

        let mut response = if focused {
            let colors = field_colors(
                &theme,
                variant,
                FieldState {
                    enabled,
                    error: false,
                    focused: true,
                    hovered: false,
                },
            );
            let mut text = std::mem::take(&mut state.text);
            let edit = {
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(input_rect));
                let ui = &mut child;
                ui.visuals_mut().text_cursor.stroke.color = colors.caret;
                ui.visuals_mut().selection.bg_fill = colors.caret.gamma_multiply(0.4);
                ui.add(
                    TextEdit::singleline(&mut text)
                        .id(id)
                        .font(font.clone())
                        .text_color(colors.input)
                        .frame(egui::Frame::NONE)
                        .margin(egui::Margin::ZERO)
                        .desired_width(input_rect.width())
                        .min_size(input_rect.size())
                        .align(Align2::LEFT_CENTER),
                )
            };
            if edit.changed() {
                // Live while typing, but only a number already in range:
                // clamping half-typed text would fight the typing.
                if let Some(typed) = format.parse_unclamped(&text) {
                    if range.contains(&typed) {
                        current = typed;
                    }
                }
            }
            if edit.lost_focus() {
                state.editing = false;
                if escape {
                    current = state.original;
                } else if let Some(typed) = format.parse(&text) {
                    current = typed;
                }
            }
            state.text = text;
            edit
        } else {
            let response = ui.interact(
                rect,
                id,
                crate::a11y::sense(enabled, Sense::click_and_drag()),
            );
            if response.clicked() {
                ctx.memory_mut(|m| m.request_focus(id));
            }
            if response.dragged() {
                let delta = response.drag_delta();
                state.drag += delta.x - delta.y;
                let whole = (state.drag / drag_points_per_step).trunc();
                if whole != 0.0 {
                    state.drag -= whole * drag_points_per_step;
                    let per = if ui.input(|i| i.modifiers.shift) {
                        PAGE_STEPS
                    } else {
                        1.0
                    };
                    current = format.stepped(current, f64::from(whole) * per);
                }
            }
            if response.drag_stopped() {
                state.drag = 0.0;
            }
            if enabled && (response.hovered() || response.dragged()) {
                ctx.set_cursor_icon(CursorIcon::ResizeHorizontal);
            }
            response
        };

        if current != old {
            *value = N::from_f64(current);
            response.mark_changed();
        }
        let shown_value = value.to_f64();

        // Paint: container, label, number, prefix and suffix.
        let hovered = enabled && response.hovered();
        let colors = field_colors(
            &theme,
            variant,
            FieldState {
                enabled,
                error: false,
                focused,
                hovered,
            },
        );
        let painter = ui.painter().clone();
        let label_geometry = LabelGeometry {
            rest: Pos2::new(rect.left() + HORIZONTAL_PADDING, rect.center().y),
            floated: floated_label_pos(variant, rect, HORIZONTAL_PADDING),
        };
        let notch = match (painted_label, variant) {
            (Some(label), InputVariant::Outlined) => {
                notch_for(&painter, &ctx, label, label_geometry.floated, 1.0)
            }
            (Some(_) | None, InputVariant::Outlined | InputVariant::Filled) => None,
        };
        painter.set(
            container_slot,
            container_shape(rect, variant, &colors, notch),
        );
        if let Some(label) = painted_label {
            paint_label(&painter, &ctx, label, label_geometry, 1.0, colors.label);
        }
        let shown_text = if focused {
            state.text.clone()
        } else {
            format.format(shown_value)
        };
        if !focused {
            painter
                .with_clip_rect(input_rect.intersect(ui.clip_rect()))
                .text(
                    Pos2::new(input_rect.left(), input_rect.center().y),
                    Align2::LEFT_CENTER,
                    &shown_text,
                    font.clone(),
                    colors.input,
                );
        }
        if let Some(prefix) = &prefix {
            painter.text(
                Pos2::new(input_rect.left() - prefix_width, input_rect.center().y),
                Align2::LEFT_CENTER,
                prefix,
                font.clone(),
                colors.affix,
            );
        }
        if let Some(suffix) = &suffix {
            let x = (input_rect.left() + measure(&shown_text)).min(input_rect.right());
            painter.text(
                Pos2::new(x + AFFIX_PADDING, input_rect.center().y),
                Align2::LEFT_CENTER,
                suffix,
                font.clone(),
                colors.affix,
            );
        }

        ctx.data_mut(|d| d.insert_temp(id, state));

        // A spin button with its value, range and step.
        let name = label.clone().unwrap_or_default();
        crate::a11y::describe(&response, Role::SpinButton, enabled, &name, None);
        let value_text = format!(
            "{}{shown_text}{}",
            prefix.as_deref().unwrap_or_default(),
            suffix.as_deref().unwrap_or_default()
        );
        ctx.accesskit_node_builder(id, |node| {
            use egui::accesskit::Action;
            node.set_role(Role::SpinButton);
            node.set_numeric_value(shown_value);
            if range.start().is_finite() {
                node.set_min_numeric_value(*range.start());
            }
            if range.end().is_finite() {
                node.set_max_numeric_value(*range.end());
            }
            node.set_numeric_value_step(step);
            node.clear_label();
            if let Some(label) = &label {
                node.set_label(label.clone());
            }
            if enabled {
                node.add_action(Action::SetValue);
                if shown_value < *range.end() {
                    node.add_action(Action::Increment);
                }
                if shown_value > *range.start() {
                    node.add_action(Action::Decrement);
                }
            } else {
                node.set_disabled();
            }
            if !focused {
                node.set_value(value_text);
            }
        });

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format(step: f64, decimals: Option<usize>, integral: bool) -> NumberFormat {
        NumberFormat {
            range: 0.0..=100.0,
            step,
            decimals,
            integral,
            prefix: Some("$".to_owned()),
            suffix: Some("px".to_owned()),
        }
    }

    #[test]
    fn steps_round_away_float_noise_and_clamp() {
        let f = format(0.1, None, false);
        assert_eq!(
            f.stepped(0.2, 1.0),
            0.3,
            "0.2 + 0.1 is 0.3, not 0.30000000000000004"
        );
        assert_eq!(f.stepped(99.95, 10.0), 100.0, "clamped to the top");
        assert_eq!(f.stepped(0.05, -10.0), 0.0, "clamped to the bottom");
    }

    #[test]
    fn parse_accepts_affixes_and_rejects_junk() {
        let f = format(1.0, Some(2), false);
        assert_eq!(
            f.parse(" $ 12.345 px"),
            Some(12.35),
            "affixes stripped, rounded"
        );
        assert_eq!(f.parse("\u{2212}5"), Some(0.0), "unicode minus, clamped");
        assert_eq!(f.parse("1e9"), Some(100.0), "clamped");
        assert_eq!(f.parse("abc"), None, "not a number");
        assert_eq!(f.parse("inf"), None, "not finite");
        assert_eq!(f.parse(""), None, "empty");
    }

    #[test]
    fn any_text_parses_inside_the_range_or_not_at_all() {
        let f = format(0.1, None, false);
        let long_nines = "9".repeat(400);
        let samples = [
            "",
            " ",
            "-",
            "+",
            ".",
            "1e400",
            "-1e400",
            "NaN",
            "\u{0}",
            "1.2.3",
            "--1",
            "$",
            "px",
            "$px",
            "12px$",
            "\u{2212}\u{2212}1",
            "\u{202e}9",
            "0x10",
            "1_000",
            "\u{661}\u{662}",
            long_nines.as_str(),
            "-0",
            "100.0000001",
        ];
        for text in samples {
            if let Some(v) = f.parse(text) {
                assert!(
                    (0.0..=100.0).contains(&v),
                    "{text:?} parsed to {v} outside the range"
                );
            }
        }
    }

    #[test]
    fn integers_show_no_decimals() {
        let f = format(1.0, None, true);
        assert_eq!(f.format(42.0), "42", "integer");
        assert_eq!(f.parse("41.6"), Some(42.0), "rounded");
        let fixed = format(0.5, Some(1), false);
        assert_eq!(fixed.format(2.0), "2.0", "fixed decimals");
        assert_eq!(decimals_of(0.25), 2, "two decimals");
    }
}
