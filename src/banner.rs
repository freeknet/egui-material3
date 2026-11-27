// ============================================================================
// Module:       banner
// Description:  An inline status banner with tones, actions and dismiss.
//
// Dependencies: egui; crate::icon, crate::paint, crate::tokens, crate::MaterialTheme
// ============================================================================

//! [`MaterialBanner`]: a message that sits in the layout (not an overlay),
//! with an optional leading icon, up to two actions and a dismiss button.
//!
//! Material 3 has no banner component: Compose's material3 has none, and
//! there are no `BannerTokens`. This one follows the Material 2 banner's
//! anatomy (leading icon, message, trailing text-button actions that drop
//! below the message when the row is too narrow) and takes its container
//! from the M3 filled card (`FilledCardTokens.ContainerShape` =
//! `CornerMedium`, 12 dp, no elevation) with M3 roles for colour:
//!
//! | Tone | Container | Content |
//! |---|---|---|
//! | [`BannerTone::Info`] | `secondaryContainer` | `onSecondaryContainer` |
//! | [`BannerTone::Warning`] | [`MaterialTheme::warning`] container | its on-container |
//! | [`BannerTone::Error`] | `errorContainer` | `onErrorContainer` |
//! | [`BannerTone::Success`] | [`MaterialTheme::success`] container | its on-container |
//!
//! Text is `BodyMedium`, actions `LabelLarge` text buttons 40 dp high with
//! 12 dp horizontal padding (`ButtonSmallTokens` / `TextButton`), the
//! dismiss button a 40 dp standard icon button with a 24 dp `close` icon
//! (`IconButtonSmallTokens`). Paddings (16 dp start, 8 dp end, 16 dp
//! between icon and text) are the M2 banner's.
//!
//! ```
//! # egui::__run_test_ui(|ui| {
//! use egui_material3::{BannerExt, BannerTone, MaterialBanner};
//!
//! let banner = ui.add_banner(
//!     MaterialBanner::new("You are offline. Changes will sync later.")
//!         .tone(BannerTone::Warning)
//!         .icon("cloud_off")
//!         .action("Retry")
//!         .dismissible(true),
//! );
//! if banner.action(0) {
//!     // retry
//! }
//! if banner.dismissed() {
//!     // hide it
//! }
//! # });
//! ```

use egui::accesskit;
use egui::{Color32, CornerRadius, Id, IdSalt, Pos2, Rect, Response, Role, Sense, Ui, Vec2};

use crate::icon::{paint_icon, Icon, SymbolOptions};
use crate::paint::{paint_focus_ring_if_focused, paint_state_layer};
use crate::tokens::{ColorRole, InteractionState, LayoutDirection, Shape, TypeRole};
use crate::MaterialTheme;

/// M2 banner: padding before the icon (or the text, without one).
const START_PADDING: f32 = 16.0;
/// M2 banner: padding after the last action.
const END_PADDING: f32 = 8.0;
/// M2 banner: space between the icon and the text.
const ICON_TEXT_GAP: f32 = 16.0;
/// `IconButtonSmallTokens.IconSize` and the leading icon's size.
const ICON_SIZE: f32 = 24.0;
/// `ButtonSmallTokens.ContainerHeight` and `IconButtonSmallTokens`
/// container: the action and dismiss targets are 40 dp.
const ACTION_HEIGHT: f32 = 40.0;
/// `TextButton` content padding, horizontal.
const ACTION_PADDING: f32 = 12.0;
/// Space between actions.
const ACTION_GAP: f32 = 8.0;
/// Vertical padding around the text.
const TEXT_VERTICAL_PADDING: f32 = 16.0;
/// Below this width the message gets the whole row and the actions move to
/// a second row, as the M2 banner does on narrow screens.
const MIN_MESSAGE_WIDTH: f32 = 200.0;
/// At most this many actions are shown.
pub const MAX_ACTIONS: usize = 2;

/// A banner's colour tone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BannerTone {
    /// Neutral information: `secondaryContainer`.
    #[default]
    Info,
    /// Something needs attention: the theme's harmonised warning container.
    Warning,
    /// Something failed: `errorContainer`.
    Error,
    /// Something worked: the theme's harmonised success container.
    Success,
}

impl BannerTone {
    /// The container and content colours in `theme`.
    pub fn colors(self, theme: &MaterialTheme) -> (Color32, Color32) {
        match self {
            BannerTone::Info => (
                theme.color(ColorRole::SecondaryContainer),
                theme.color(ColorRole::OnSecondaryContainer),
            ),
            BannerTone::Warning => {
                let group = theme.warning();
                (group.color_container, group.on_color_container)
            }
            BannerTone::Error => (
                theme.color(ColorRole::ErrorContainer),
                theme.color(ColorRole::OnErrorContainer),
            ),
            BannerTone::Success => {
                let group = theme.success();
                (group.color_container, group.on_color_container)
            }
        }
    }

    /// The icon shown when none is set with [`MaterialBanner::icon`] and
    /// [`MaterialBanner::default_icon`] is on.
    fn default_icon(self) -> &'static str {
        match self {
            BannerTone::Info => "info",
            BannerTone::Warning => "warning",
            BannerTone::Error => "error",
            BannerTone::Success => "check_circle",
        }
    }
}

/// An inline banner. Add it with [`MaterialBanner::show`] (or
/// `ui.add_banner(..)` from [`BannerExt`]) to get a [`BannerResponse`].
#[must_use = "show it with `banner.show(ui)`"]
pub struct MaterialBanner {
    message: String,
    tone: BannerTone,
    icon: Option<Icon>,
    default_icon: bool,
    actions: Vec<String>,
    dismissible: bool,
    dismiss_label: String,
    id_salt: Option<IdSalt>,
}

impl MaterialBanner {
    /// An info banner with `message`, no icon, actions or dismiss button.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            tone: BannerTone::Info,
            icon: None,
            default_icon: false,
            actions: Vec::new(),
            dismissible: false,
            dismiss_label: "Dismiss".to_owned(),
            id_salt: None,
        }
    }

    /// The colour tone (default [`BannerTone::Info`]).
    pub fn tone(mut self, tone: BannerTone) -> Self {
        self.tone = tone;
        self
    }

    /// A leading icon: a Material Symbols name or any [`Icon`], drawn in
    /// the tone's content colour.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Without an [`icon`](Self::icon), show the tone's own (`info`,
    /// `warning`, `error`, `check_circle`). Off by default.
    pub fn default_icon(mut self, on: bool) -> Self {
        self.default_icon = on;
        self
    }

    /// Adds an action (a text button). Up to [`MAX_ACTIONS`]; more are
    /// ignored. Actions are reported by index, in the order added.
    pub fn action(mut self, label: impl Into<String>) -> Self {
        if self.actions.len() < MAX_ACTIONS {
            self.actions.push(label.into());
        }
        self
    }

    /// Shows a trailing dismiss (`close`) button.
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// The dismiss button's accessible label (default "Dismiss").
    pub fn dismiss_label(mut self, label: impl Into<String>) -> Self {
        self.dismiss_label = label.into();
        self
    }

    /// Salts the banner's id, for several banners with the same message.
    pub fn id_salt(mut self, salt: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(&salt));
        self
    }

    fn resolved_icon(&self) -> Option<Icon> {
        match (&self.icon, self.default_icon) {
            (Some(icon), _) => Some(icon.clone()),
            (None, true) => Some(Icon::from(self.tone.default_icon())),
            (None, false) => None,
        }
    }

    /// Lays the banner out across the available width and paints it.
    pub fn show(self, ui: &mut Ui) -> BannerResponse {
        let theme = MaterialTheme::of(ui.ctx());
        let (container, content) = self.tone.colors(&theme);
        let rtl = theme.layout_direction == LayoutDirection::Rtl;
        let id: Id = match self.id_salt {
            Some(salt) => ui.scope_id().with_salt(salt),
            None => ui
                .scope_id()
                .with(("egui_material3::banner", &self.message)),
        };
        let width = ui.available_width().max(0.0);
        let icon = self.resolved_icon();

        // Action and dismiss widths.
        let painter = ui.painter().clone();
        let action_galleys: Vec<_> = self
            .actions
            .iter()
            .map(|label| {
                painter.layout_job(TypeRole::LabelLarge.style().layout_job(
                    ui.ctx(),
                    label.clone(),
                    content,
                ))
            })
            .collect();
        let action_widths: Vec<f32> = action_galleys
            .iter()
            .map(|g| g.size().x + 2.0 * ACTION_PADDING)
            .collect();
        let dismiss_width = if self.dismissible { ACTION_HEIGHT } else { 0.0 };
        let gaps = action_widths.len() + usize::from(self.dismissible);
        let trailing_width = action_widths.iter().sum::<f32>()
            + dismiss_width
            + ACTION_GAP * gaps.saturating_sub(1) as f32;

        let text_start = START_PADDING
            + if icon.is_some() {
                ICON_SIZE + ICON_TEXT_GAP
            } else {
                0.0
            };
        let inline_width = width - text_start - END_PADDING - trailing_width - ACTION_GAP;
        let stacked = trailing_width > 0.0 && inline_width < MIN_MESSAGE_WIDTH;
        let message_width = if stacked || trailing_width <= 0.0 {
            width - text_start - START_PADDING
        } else {
            inline_width
        }
        .max(1.0);

        let mut job =
            TypeRole::BodyMedium
                .style()
                .layout_job(ui.ctx(), self.message.clone(), content);
        job.wrap.max_width = message_width;
        let galley = painter.layout_job(job);
        let text_height = galley.size().y;
        let top_row = (text_height + 2.0 * TEXT_VERTICAL_PADDING).max(ACTION_HEIGHT + 2.0 * 8.0);
        let height = if stacked {
            text_height + TEXT_VERTICAL_PADDING + 4.0 + ACTION_HEIGHT + 8.0
        } else {
            top_row
        };

        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        crate::a11y::describe(&response, Role::Status, true, &self.message, None);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_live(accesskit::Live::Polite);
        });

        // Maps an x measured from the start edge to the screen.
        let x_at = |from_start: f32, w: f32| {
            if rtl {
                rect.max.x - from_start - w
            } else {
                rect.min.x + from_start
            }
        };

        let mut clicked_action = None;
        let mut dismissed = false;
        if ui.is_rect_visible(rect) {
            let corner = Shape::MEDIUM.corner_radius(rect, theme.layout_direction);
            painter.rect_filled(rect, corner, container);
        }

        let text_top = if stacked {
            rect.min.y + TEXT_VERTICAL_PADDING
        } else {
            rect.center().y - text_height / 2.0
        };
        if let Some(icon) = &icon {
            let icon_rect = Rect::from_min_size(
                Pos2::new(
                    x_at(START_PADDING, ICON_SIZE),
                    if stacked {
                        text_top + (text_height.min(ICON_SIZE * 2.0) - ICON_SIZE) / 2.0
                    } else {
                        rect.center().y - ICON_SIZE / 2.0
                    },
                ),
                Vec2::splat(ICON_SIZE),
            );
            paint_icon(
                &painter,
                icon_rect,
                icon,
                &SymbolOptions::default(),
                content,
            );
        }
        let text_x = x_at(text_start, galley.size().x);
        painter.galley(Pos2::new(text_x, text_top), galley, content);

        // Trailing controls, laid out from the end edge.
        let controls_center_y = if stacked {
            rect.max.y - 8.0 - ACTION_HEIGHT / 2.0
        } else {
            rect.center().y
        };
        let mut from_end = END_PADDING;
        let end_x = |from_end: f32, w: f32| {
            if rtl {
                rect.min.x + from_end
            } else {
                rect.max.x - from_end - w
            }
        };
        if self.dismissible {
            let target = Rect::from_min_size(
                Pos2::new(
                    end_x(from_end, ACTION_HEIGHT),
                    controls_center_y - ACTION_HEIGHT / 2.0,
                ),
                Vec2::splat(ACTION_HEIGHT),
            );
            let r = ui.interact(target, id.with("dismiss"), Sense::click());
            crate::a11y::button(&r, true, &self.dismiss_label);
            let radius = CornerRadius::same((ACTION_HEIGHT / 2.0) as u8);
            paint_state_layer(
                &painter,
                target,
                radius,
                content,
                InteractionState::from_response(&r),
            );
            paint_focus_ring_if_focused(&painter, &r, radius);
            paint_icon(
                &painter,
                Rect::from_center_size(target.center(), Vec2::splat(ICON_SIZE)),
                &Icon::from("close"),
                &SymbolOptions::default(),
                content,
            );
            dismissed = r.clicked();
            from_end += ACTION_HEIGHT + ACTION_GAP;
        }
        // The last action sits next to the end edge (or the dismiss button).
        for (index, (galley, w)) in action_galleys
            .into_iter()
            .zip(action_widths.iter().copied())
            .enumerate()
            .rev()
        {
            let target = Rect::from_min_size(
                Pos2::new(end_x(from_end, w), controls_center_y - ACTION_HEIGHT / 2.0),
                Vec2::new(w, ACTION_HEIGHT),
            );
            let label = self.actions.get(index).map_or("", String::as_str);
            let r = ui.interact(target, id.with(("action", index)), Sense::click());
            crate::a11y::button(&r, true, label);
            let radius = CornerRadius::same((ACTION_HEIGHT / 2.0) as u8);
            paint_state_layer(
                &painter,
                target,
                radius,
                content,
                InteractionState::from_response(&r),
            );
            paint_focus_ring_if_focused(&painter, &r, radius);
            painter.galley(target.center() - galley.size() / 2.0, galley, content);
            if r.clicked() {
                clicked_action = Some(index);
            }
            from_end += w + ACTION_GAP;
        }

        BannerResponse {
            response,
            clicked_action,
            dismissed,
        }
    }
}

/// What a banner did this frame.
#[derive(Clone, Debug)]
pub struct BannerResponse {
    /// The banner's own response (its whole rect; hover only).
    pub response: Response,
    clicked_action: Option<usize>,
    dismissed: bool,
}

impl BannerResponse {
    /// Whether the dismiss button was clicked this frame.
    pub fn dismissed(&self) -> bool {
        self.dismissed
    }

    /// Whether action `index` was clicked this frame.
    pub fn action(&self, index: usize) -> bool {
        self.clicked_action == Some(index)
    }

    /// The index of the action clicked this frame, if any.
    pub fn clicked_action(&self) -> Option<usize> {
        self.clicked_action
    }
}

/// `ui.add_banner(banner)`: the same as `banner.show(ui)`.
pub trait BannerExt {
    /// Shows `banner` and returns what it did.
    fn add_banner(&mut self, banner: MaterialBanner) -> BannerResponse;
}

impl BannerExt for Ui {
    fn add_banner(&mut self, banner: MaterialBanner) -> BannerResponse {
        banner.show(self)
    }
}
