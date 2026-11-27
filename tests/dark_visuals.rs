// ============================================================================
// Module:       dark_visuals (integration test)
// Description:  Integration tests: dark visuals.
//
// Dependencies: egui_material3, egui
// ============================================================================

//! Integration tests: dark visuals.
//!
//! Each test themes its own context with `apply_theme_context`, so they run
//! in parallel without sharing the process-wide default.

use egui_material3::theme::{
    apply_theme_context, theme_context_of, MaterialThemeContext, ThemeMode,
};
use egui_material3::MaterialTheme;

fn lum(c: egui::Color32) -> u32 {
    c.r() as u32 + c.g() as u32 + c.b() as u32
}

fn settings(mode: ThemeMode) -> MaterialThemeContext {
    MaterialThemeContext {
        theme_mode: mode,
        ..Default::default()
    }
}

#[test]
fn explicit_modes_use_their_scheme() {
    let ctx = egui::Context::default();
    apply_theme_context(&ctx, &settings(ThemeMode::Light), None::<fn() -> ThemeMode>);
    let light = ctx.global_style().visuals.panel_fill;
    assert!(
        !MaterialTheme::of(&ctx).is_dark,
        "Light mode theme is light"
    );
    apply_theme_context(&ctx, &settings(ThemeMode::Dark), None::<fn() -> ThemeMode>);
    let dark = ctx.global_style().visuals.panel_fill;
    assert!(MaterialTheme::of(&ctx).is_dark, "Dark mode theme is dark");
    assert!(lum(dark) < lum(light), "panel_fill not darker in Dark mode");
}

#[test]
fn auto_mode_follows_detector() {
    let ctx = egui::Context::default();
    apply_theme_context(&ctx, &settings(ThemeMode::Auto), Some(|| ThemeMode::Dark));
    let dark = ctx.global_style().visuals.panel_fill;
    let stored = theme_context_of(&ctx);
    assert_eq!(stored.theme_mode, ThemeMode::Auto, "Auto stays Auto");
    assert!(stored.dark_mode, "Auto resolved to dark");
    apply_theme_context(&ctx, &settings(ThemeMode::Auto), Some(|| ThemeMode::Light));
    let light = ctx.global_style().visuals.panel_fill;
    assert!(
        lum(dark) < lum(light),
        "Auto+Dark detector not dark: {dark:?} vs {light:?}"
    );
}

#[test]
fn clearing_returns_to_the_default() {
    let ctx = egui::Context::default();
    apply_theme_context(&ctx, &settings(ThemeMode::Dark), None::<fn() -> ThemeMode>);
    MaterialTheme::clear(&ctx);
    assert_eq!(
        theme_context_of(&ctx).theme_mode,
        MaterialThemeContext::default().theme_mode,
        "settings fall back to the default once cleared"
    );
}
