// ============================================================================
// Module:       material_theme
// Description:  MaterialTheme, the resolved theme stored per egui Context.
//
// Dependencies: egui; crate::color, crate::theme, crate::tokens
// ============================================================================

//! [`MaterialTheme`]: the active theme, stored per egui [`egui::Context`].
//!
//! The theme lives in the context's data (`ctx.data`), so two contexts (two
//! viewports, or two tests running in parallel) can hold different themes.
//! A context that has none set falls back to the process-wide theme that
//! [`crate::theme`] manages, so code written against the old global API
//! keeps working unchanged.
//!
//! # Success and warning colours
//!
//! Material 3 has no success or warning roles (Compose's `ColorScheme` and
//! `ColorSchemeKeyTokens` stop at `error`). Theme Builder adds them as
//! *extended colours*: a fixed design colour harmonised towards the
//! theme's source colour and expanded into a tonal palette
//! ([`crate::color::custom_color`], MCU's `customColor`). [`MaterialTheme::success`]
//! and [`MaterialTheme::warning`] do the same with [`SUCCESS_SEED`] and
//! [`WARNING_SEED`], harmonised towards the theme's `primary`, so status
//! colours follow the theme (and its light or dark mode) instead of being
//! fixed RGB values. [`MaterialTheme::extended`] does it for any colour.

use std::sync::Arc;

use egui::{Color32, Context, Id};

use crate::color::{custom_color, ColorGroup};
use crate::theme::{ContrastLevel, MaterialThemeContext, MaterialThemeFile, ThemeMode};
use crate::tokens::{ColorRole, ColorScheme, LayoutDirection, MotionScheme};

/// The resolved Material 3 theme a widget draws with.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialTheme {
    /// One colour per role.
    pub colors: ColorScheme,
    /// Whether `colors` is a dark scheme.
    pub is_dark: bool,
    /// Standard or Expressive springs.
    pub motion: MotionScheme,
    /// Left-to-right or right-to-left.
    pub layout_direction: LayoutDirection,
}

impl Default for MaterialTheme {
    fn default() -> Self {
        Self::baseline(false)
    }
}

/// The design colour [`MaterialTheme::success`] harmonises: Material's
/// green 500 (`#4CAF50`). Not a Compose token; Material 3 has no success
/// role.
pub const SUCCESS_SEED: Color32 = Color32::from_rgb(0x4C, 0xAF, 0x50);
/// The design colour [`MaterialTheme::warning`] harmonises: Material's
/// amber 500 (`#FFC107`). Not a Compose token; Material 3 has no warning
/// role.
pub const WARNING_SEED: Color32 = Color32::from_rgb(0xFF, 0xC1, 0x07);

fn theme_id() -> Id {
    Id::unique("egui_material3::MaterialTheme")
}

impl MaterialTheme {
    /// Compose's baseline scheme, light or dark.
    pub fn baseline(dark: bool) -> Self {
        Self {
            colors: ColorScheme::baseline(dark),
            is_dark: dark,
            motion: MotionScheme::default(),
            layout_direction: LayoutDirection::default(),
        }
    }

    /// A theme from a Theme Builder export, picking the light or dark scheme
    /// at `contrast`. Roles missing from the file keep their baseline value.
    pub fn from_theme_file(file: &MaterialThemeFile, dark: bool, contrast: ContrastLevel) -> Self {
        let context = MaterialThemeContext {
            theme_mode: if dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            },
            contrast_level: contrast,
            material_theme: Some(file.clone()),
            dark_mode: dark,
            ..Default::default()
        };
        Self::from_context(&context)
    }

    /// The theme a [`MaterialThemeContext`] describes, at its current
    /// light/dark state.
    pub fn from_context(context: &MaterialThemeContext) -> Self {
        let dark = match context.theme_mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::Auto => context.dark_mode,
        };
        Self {
            colors: context.color_scheme_for_mode(dark),
            ..Self::baseline(dark)
        }
    }

    /// The theme for `ctx`: the one set with [`MaterialTheme::set`], or else
    /// the process-wide theme from [`crate::theme`].
    pub fn of(ctx: &Context) -> Arc<MaterialTheme> {
        if let Some(theme) = ctx.data(|d| d.get_temp::<Arc<MaterialTheme>>(theme_id())) {
            return theme;
        }
        Arc::new(Self::process_default())
    }

    /// The theme of the process-wide default, for code that has no context
    /// at hand (the deprecated shims).
    pub(crate) fn process_default() -> MaterialTheme {
        Self::from_context(&crate::theme::process_default())
    }

    /// Sets the theme for `ctx`. Widgets drawn in that context use it from
    /// then on; other contexts are not affected.
    pub fn set(ctx: &Context, theme: MaterialTheme) {
        ctx.data_mut(|d| d.insert_temp(theme_id(), Arc::new(theme)));
    }

    /// Removes the theme set for `ctx` (and the settings
    /// [`apply_theme_context`](crate::theme::apply_theme_context) stored with
    /// it), going back to the process-wide one.
    pub fn clear(ctx: &Context) {
        ctx.data_mut(|d| d.remove::<Arc<MaterialTheme>>(theme_id()));
        crate::theme::clear_context_settings(ctx);
    }

    /// The colour of `role`.
    #[inline]
    pub fn color(&self, role: ColorRole) -> Color32 {
        self.colors.get(role)
    }

    /// `color` as an extended colour of this theme: harmonised towards the
    /// theme's `primary` (MCU's `Blend.harmonize`) and resolved to its four
    /// roles in this theme's light or dark mode (tones 40/100/90/10 light,
    /// 80/20/30/90 dark, as Theme Builder exports them). The contrast level
    /// does not change extended colours, as in Theme Builder.
    ///
    /// ```
    /// use egui_material3::MaterialTheme;
    ///
    /// let theme = MaterialTheme::baseline(false);
    /// let brand = theme.extended(egui::Color32::from_rgb(0x00, 0x96, 0x88));
    /// assert_ne!(brand.color_container, brand.on_color_container);
    /// ```
    pub fn extended(&self, color: Color32) -> ColorGroup {
        let group = custom_color("", self.color(ColorRole::Primary), color, true);
        if self.is_dark {
            group.dark
        } else {
            group.light
        }
    }

    /// The success colour group: [`SUCCESS_SEED`] as an extended colour of
    /// this theme ([`extended`](Self::extended)). Use `color`/`on_color`
    /// for strong indicators (a badge), `color_container` /
    /// `on_color_container` for tinted surfaces (a banner).
    pub fn success(&self) -> ColorGroup {
        self.extended(SUCCESS_SEED)
    }

    /// The warning colour group: [`WARNING_SEED`] as an extended colour of
    /// this theme ([`extended`](Self::extended)).
    pub fn warning(&self) -> ColorGroup {
        self.extended(WARNING_SEED)
    }
}
