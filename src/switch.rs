// ============================================================================
// Module:       switch
// Description:  Material Design 3 switch.
//
// Dependencies: egui; crate::tokens, crate::MaterialTheme
// ============================================================================

//! Material Design 3 switch.
//!
//! Geometry, colours and opacities follow Compose's `SwitchTokens`:
//!
//! - a 52 × 32 dp track with full corners in a 48 dp tall touch target, with
//!   a 2 dp outline drawn *inside* it when unselected;
//! - the thumb (handle) is 16 dp unselected, 24 dp selected or when showing
//!   an icon, and 28 dp while pressed. Its centre sits 16 dp from the
//!   track's start when off and 36 dp when on, whatever its size;
//! - the thumb moves and resizes with the fast spatial spring of the theme's
//!   motion scheme;
//! - selected: `primary` track, `onPrimary` thumb (`primaryContainer` when
//!   hovered, focused or pressed), `onPrimaryContainer` icon (`primary` when
//!   hovered, focused or pressed);
//! - unselected: `surfaceContainerHighest` track, `outline` outline and
//!   thumb (`onSurfaceVariant` when hovered, focused or pressed),
//!   `surfaceContainerHighest` icon;
//! - disabled: track at 12 %; selected thumb `surface` at full opacity with
//!   an `onSurface` icon at 38 %; unselected thumb `onSurface` at 38 %;
//! - a 40 dp circular state layer centred on the thumb, and a focus ring
//!   around the track.
//!
//! ## Dense
//!
//! [`MaterialSwitch::dense`] draws a smaller switch for dense layouts
//! (settings lists, toolbars, tables). Compose has no dense switch token,
//! so these values are derived, not cited: Material's density guidance
//! shrinks a component by 4 dp of height per density step (-1, -2, -3),
//! and the dense switch is two steps down (-2): the 32 dp track becomes
//! 24 dp, and the rest of the switch is scaled by the same 3/4 so it keeps
//! its proportions.
//!
//! | | Default | Dense |
//! |---|---|---|
//! | Track | 52 × 32 | 39 × 24 |
//! | Touch target height | 48 | 40 |
//! | Thumb: off / on or icon / pressed | 16 / 24 / 28 | 12 / 18 / 21 |
//! | Thumb centre: off / on | 16 / 36 | 12 / 27 |
//! | Icon | 16 | 12 |
//! | State layer | 40 | 30 |
//!
//! The track outline stays 2 dp, like other strokes under density. The
//! 40 dp touch target is below the 48 dp minimum Material recommends, so
//! use dense switches where the row around them is the larger target (a
//! clickable list item) or in pointer-driven desktop layouts.

use crate::checkbox::{focus_ring, full_corner, label_color, label_galley, TOUCH_TARGET};
use crate::tokens::state::DISABLED_CONTENT_OPACITY;
use crate::tokens::{animate_value_with_spring, ColorRole, InteractionState, SpringSpeed};
use crate::MaterialTheme;
use egui::{
    self, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget,
};

/// `SwitchTokens.TrackWidth`.
pub const TRACK_WIDTH: f32 = 52.0;
/// `SwitchTokens.TrackHeight`.
pub const TRACK_HEIGHT: f32 = 32.0;
/// `SwitchTokens.TrackOutlineWidth`: drawn inside the track.
pub const TRACK_OUTLINE_WIDTH: f32 = 2.0;
/// `SwitchTokens.UnselectedHandleWidth`.
pub const UNSELECTED_HANDLE_SIZE: f32 = 16.0;
/// `SwitchTokens.SelectedHandleWidth` (and `HandleWidth`, with an icon).
pub const SELECTED_HANDLE_SIZE: f32 = 24.0;
/// `SwitchTokens.PressedHandleWidth`.
pub const PRESSED_HANDLE_SIZE: f32 = 28.0;
/// `SwitchTokens.SelectedIconSize` / `UnselectedIconSize`.
pub const ICON_SIZE: f32 = 16.0;
/// `SwitchTokens.StateLayerSize`.
pub const STATE_LAYER_SIZE: f32 = 40.0;
/// `SwitchTokens.DisabledTrackOpacity`.
pub const DISABLED_TRACK_OPACITY: f32 = 0.12;
/// Thumb centre, from the track's start, when off: half the track height.
pub const THUMB_CENTER_OFF: f32 = TRACK_HEIGHT / 2.0;
/// Thumb centre, from the track's start, when on.
pub const THUMB_CENTER_ON: f32 = TRACK_WIDTH - TRACK_HEIGHT / 2.0;
/// Gap between the track and the label.
const LABEL_GAP: f32 = 8.0;

/// How much the dense switch is scaled: two -4 dp density steps take the
/// 32 dp track to 24 dp (derived from Material's density guidance; Compose
/// has no dense switch token).
pub const DENSE_SCALE: f32 = 0.75;
/// Dense track width: [`TRACK_WIDTH`] × [`DENSE_SCALE`].
pub const DENSE_TRACK_WIDTH: f32 = TRACK_WIDTH * DENSE_SCALE;
/// Dense track height: [`TRACK_HEIGHT`] minus two 4 dp density steps.
pub const DENSE_TRACK_HEIGHT: f32 = TRACK_HEIGHT * DENSE_SCALE;
/// Dense touch target height: the 48 dp target minus two 4 dp density
/// steps.
pub const DENSE_TOUCH_TARGET: f32 = 40.0;

/// The sizes one switch is drawn at.
#[derive(Clone, Copy, Debug)]
struct Geometry {
    track_width: f32,
    track_height: f32,
    touch_target: f32,
    unselected_handle: f32,
    selected_handle: f32,
    pressed_handle: f32,
    icon: f32,
    state_layer: f32,
}

impl Geometry {
    const DEFAULT: Self = Self {
        track_width: TRACK_WIDTH,
        track_height: TRACK_HEIGHT,
        touch_target: TOUCH_TARGET,
        unselected_handle: UNSELECTED_HANDLE_SIZE,
        selected_handle: SELECTED_HANDLE_SIZE,
        pressed_handle: PRESSED_HANDLE_SIZE,
        icon: ICON_SIZE,
        state_layer: STATE_LAYER_SIZE,
    };

    const DENSE: Self = Self {
        track_width: DENSE_TRACK_WIDTH,
        track_height: DENSE_TRACK_HEIGHT,
        touch_target: DENSE_TOUCH_TARGET,
        unselected_handle: UNSELECTED_HANDLE_SIZE * DENSE_SCALE,
        selected_handle: SELECTED_HANDLE_SIZE * DENSE_SCALE,
        pressed_handle: PRESSED_HANDLE_SIZE * DENSE_SCALE,
        icon: ICON_SIZE * DENSE_SCALE,
        state_layer: STATE_LAYER_SIZE * DENSE_SCALE,
    };

    const fn of(dense: bool) -> Self {
        if dense {
            Self::DENSE
        } else {
            Self::DEFAULT
        }
    }

    /// Thumb centre, from the track's start, when off.
    fn thumb_center_off(&self) -> f32 {
        self.track_height / 2.0
    }

    /// Thumb centre, from the track's start, when on.
    fn thumb_center_on(&self) -> f32 {
        self.track_width - self.track_height / 2.0
    }
}

/// Material Design 3 switch.
///
/// ```rust
/// # use egui_material3::MaterialSwitch;
/// # egui::__run_test_ui(|ui| {
/// let mut wifi = false;
/// ui.add(MaterialSwitch::new(&mut wifi).text("Wi-Fi"));
///
/// let mut locked = true;
/// ui.add(MaterialSwitch::new(&mut locked).text("Locked").enabled(false));
///
/// // A dense switch whose label is drawn elsewhere (a list row), named
/// // for screen readers.
/// let mut bluetooth = true;
/// ui.add(MaterialSwitch::new(&mut bluetooth).dense(true).accessible_label("Bluetooth"));
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct MaterialSwitch<'a> {
    selected: &'a mut bool,
    text: Option<String>,
    enabled: bool,
    selected_icon: Option<char>,
    unselected_icon: Option<char>,
    show_track_outline: bool,
    dense: bool,
    accessible_label: Option<String>,
}

impl<'a> MaterialSwitch<'a> {
    /// A switch bound to `selected`.
    pub fn new(selected: &'a mut bool) -> Self {
        Self {
            selected,
            text: None,
            enabled: true,
            selected_icon: None,
            unselected_icon: None,
            show_track_outline: true,
            dense: false,
            accessible_label: None,
        }
    }

    /// A label after the switch; also its accessible name.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Enables or disables the switch. A disabled switch is dimmed, is not
    /// focusable and ignores clicks.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// An icon on the thumb while on. The thumb is then 24 dp.
    pub fn selected_icon(mut self, icon: char) -> Self {
        self.selected_icon = Some(icon);
        self
    }

    /// An icon on the thumb while off. The thumb is then 24 dp, not 16.
    pub fn unselected_icon(mut self, icon: char) -> Self {
        self.unselected_icon = Some(icon);
        self
    }

    /// Icons for both states.
    pub fn with_icons(mut self, selected: char, unselected: char) -> Self {
        self.selected_icon = Some(selected);
        self.unselected_icon = Some(unselected);
        self
    }

    /// Whether to draw the unselected track outline (default `true`).
    pub fn show_track_outline(mut self, show: bool) -> Self {
        self.show_track_outline = show;
        self
    }

    /// Draws the dense switch (default `false`): a 39 × 24 dp track in a
    /// 40 dp tall touch target, the rest scaled to match. See the
    /// [module docs](self#dense) for every value and where they come from.
    pub fn dense(mut self, dense: bool) -> Self {
        self.dense = dense;
        self
    }

    /// The name screen readers announce for the switch, without drawing a
    /// label: for a switch whose visible label is somewhere else (a list
    /// row's headline, a table column header). It takes precedence over
    /// [`Self::text`] as the accessible name; that text is still drawn.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }
}

/// The colours of one switch state.
struct Colors {
    track: Color32,
    outline: Color32,
    handle: Color32,
    icon: Color32,
}

fn colors(theme: &MaterialTheme, selected: bool, enabled: bool, active: bool) -> Colors {
    let c = |role| theme.color(role);
    let dim = |color: Color32| color.gamma_multiply(DISABLED_CONTENT_OPACITY);
    match (enabled, selected) {
        (false, true) => Colors {
            track: c(ColorRole::OnSurface).gamma_multiply(DISABLED_TRACK_OPACITY),
            outline: Color32::TRANSPARENT,
            handle: c(ColorRole::Surface),
            icon: dim(c(ColorRole::OnSurface)),
        },
        (false, false) => Colors {
            track: c(ColorRole::SurfaceContainerHighest).gamma_multiply(DISABLED_TRACK_OPACITY),
            outline: c(ColorRole::OnSurface).gamma_multiply(DISABLED_TRACK_OPACITY),
            handle: dim(c(ColorRole::OnSurface)),
            icon: dim(c(ColorRole::SurfaceContainerHighest)),
        },
        (true, true) => Colors {
            track: c(ColorRole::Primary),
            outline: Color32::TRANSPARENT,
            handle: if active {
                c(ColorRole::PrimaryContainer)
            } else {
                c(ColorRole::OnPrimary)
            },
            icon: if active {
                c(ColorRole::Primary)
            } else {
                c(ColorRole::OnPrimaryContainer)
            },
        },
        (true, false) => Colors {
            track: c(ColorRole::SurfaceContainerHighest),
            outline: c(ColorRole::Outline),
            handle: if active {
                c(ColorRole::OnSurfaceVariant)
            } else {
                c(ColorRole::Outline)
            },
            icon: c(ColorRole::SurfaceContainerHighest),
        },
    }
}

impl Widget for MaterialSwitch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = MaterialTheme::of(ui.ctx());
        let text_color = label_color(&theme, self.enabled);
        let galley = self
            .text
            .as_deref()
            .and_then(|text| label_galley(ui, text, text_color));
        let geometry = Geometry::of(self.dense);
        let label_width = galley.as_ref().map_or(0.0, |g| LABEL_GAP + g.size().x);
        let desired_size = Vec2::new(geometry.track_width + label_width, geometry.touch_target);

        let (rect, mut response) = ui.allocate_exact_size(
            desired_size,
            crate::a11y::sense(self.enabled, Sense::click()),
        );

        if response.clicked() && self.enabled {
            *self.selected = !*self.selected;
            response.mark_changed();
        }
        let selected = *self.selected;
        crate::a11y::toggle(
            &response,
            egui::Role::Switch,
            self.enabled,
            self.accessible_label
                .as_deref()
                .or(self.text.as_deref())
                .unwrap_or_default(),
            selected,
        );

        let interaction = InteractionState::from_response(&response);
        let active = interaction.hovered || interaction.focused || interaction.pressed;
        let icon = if selected {
            self.selected_icon
        } else {
            self.unselected_icon
        };

        // Position and size both follow the fast spatial spring.
        let spring = theme.motion.spatial(SpringSpeed::Fast);
        let ctx = ui.ctx().clone();
        let progress = animate_value_with_spring(
            &ctx,
            response.id.with("thumb_position"),
            if selected { 1.0 } else { 0.0 },
            spring,
        );
        let target_size = if interaction.pressed {
            geometry.pressed_handle
        } else if selected || icon.is_some() {
            geometry.selected_handle
        } else {
            geometry.unselected_handle
        };
        let handle_size =
            animate_value_with_spring(&ctx, response.id.with("thumb_size"), target_size, spring);

        let track_rect = Rect::from_min_size(
            Pos2::new(rect.min.x, rect.center().y - geometry.track_height / 2.0),
            Vec2::new(geometry.track_width, geometry.track_height),
        );
        let (start, end) = match theme.layout_direction {
            crate::tokens::LayoutDirection::Ltr => (
                track_rect.min.x + geometry.thumb_center_off(),
                track_rect.min.x + geometry.thumb_center_on(),
            ),
            crate::tokens::LayoutDirection::Rtl => (
                track_rect.max.x - geometry.thumb_center_off(),
                track_rect.max.x - geometry.thumb_center_on(),
            ),
        };
        let thumb_center = Pos2::new(start + (end - start) * progress, track_rect.center().y);

        let off = colors(&theme, false, self.enabled, active);
        let on = colors(&theme, true, self.enabled, active);
        let t = progress.clamp(0.0, 1.0);
        let mix = |a: Color32, b: Color32| a.lerp_to_gamma(b, t);
        let track = mix(off.track, on.track);
        let outline = mix(off.outline, on.outline);
        let handle = mix(off.handle, on.handle);
        let icon_color = if selected { on.icon } else { off.icon };

        let painter = ui.painter();
        let track_corner = full_corner(geometry.track_height);
        painter.rect_filled(track_rect, track_corner, track);
        if self.show_track_outline && outline.a() > 0 {
            painter.rect_stroke(
                track_rect,
                track_corner,
                Stroke::new(TRACK_OUTLINE_WIDTH, outline),
                StrokeKind::Inside,
            );
        }

        let layer_color = if selected {
            theme.color(ColorRole::Primary)
        } else {
            theme.color(ColorRole::OnSurface)
        };
        crate::paint::paint_state_layer(
            painter,
            Rect::from_center_size(thumb_center, Vec2::splat(geometry.state_layer)),
            full_corner(geometry.state_layer),
            layer_color,
            interaction,
        );

        painter.circle_filled(thumb_center, handle_size / 2.0, handle);
        if let Some(icon) = icon {
            painter.text(
                thumb_center,
                egui::Align2::CENTER_CENTER,
                icon.to_string(),
                FontId::proportional(geometry.icon),
                icon_color,
            );
        }

        focus_ring(
            painter,
            &theme,
            &response,
            track_rect,
            geometry.track_height,
        );

        if let Some(galley) = galley {
            let pos = Pos2::new(
                track_rect.max.x + LABEL_GAP,
                rect.center().y - galley.size().y / 2.0,
            );
            painter.galley(pos, galley, text_color);
        }

        response
    }
}

/// Shorthand for [`MaterialSwitch::new`].
pub fn switch(selected: &mut bool) -> MaterialSwitch<'_> {
    MaterialSwitch::new(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumb_centres_match_compose() {
        assert_eq!(THUMB_CENTER_OFF, 16.0, "off thumb centre");
        assert_eq!(THUMB_CENTER_ON, 36.0, "on thumb centre");
    }

    #[test]
    fn dense_geometry_is_two_density_steps_down() {
        let dense = Geometry::DENSE;
        assert_eq!(dense.track_height, TRACK_HEIGHT - 2.0 * 4.0, "24 dp track");
        assert_eq!(dense.touch_target, TOUCH_TARGET - 2.0 * 4.0, "40 dp target");
        assert_eq!(dense.track_width, 39.0, "39 dp track width");
        assert_eq!(
            [
                dense.unselected_handle,
                dense.selected_handle,
                dense.pressed_handle
            ],
            [12.0, 18.0, 21.0],
            "dense thumb sizes"
        );
        assert_eq!(dense.icon, 12.0, "dense icon");
        assert_eq!(dense.state_layer, 30.0, "dense state layer");
        assert_eq!(dense.thumb_center_off(), 12.0, "dense off thumb centre");
        assert_eq!(dense.thumb_center_on(), 27.0, "dense on thumb centre");
        let default = Geometry::DEFAULT;
        assert_eq!(default.thumb_center_off(), THUMB_CENTER_OFF, "default off");
        assert_eq!(default.thumb_center_on(), THUMB_CENTER_ON, "default on");
    }
}
