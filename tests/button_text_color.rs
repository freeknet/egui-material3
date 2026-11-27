// ============================================================================
// Module:       button_text_color (integration test)
// Description:  Integration tests: button text color.
//
// Dependencies: egui_material3, egui
// ============================================================================

//! Integration tests: button text color.
//!
//! Filled button text colour in light and dark modes. The colours are read
//! from a `MaterialThemeContext` value or a context's own theme, never the
//! process-wide default, so the tests run in parallel.

use egui_material3::theme::{
    apply_theme_context, theme_context_of, MaterialThemeContext, ThemeMode,
};

fn on_primary_pair(dark: bool) -> (egui::Color32, egui::Color32) {
    let theme = MaterialThemeContext {
        dark_mode: dark,
        ..Default::default()
    };
    (
        theme.get_color_by_name("onPrimary"),
        theme.get_color_by_name("onPrimaryFixed"),
    )
}

#[test]
fn test_filled_button_text_color_light_mode() {
    let (on_primary, on_primary_fixed) = on_primary_pair(false);
    assert_ne!(
        on_primary, on_primary_fixed,
        "onPrimary and onPrimaryFixed should differ in light mode"
    );
}

#[test]
fn test_filled_button_text_color_dark_mode() {
    let (on_primary, on_primary_fixed) = on_primary_pair(true);
    assert_ne!(
        on_primary, on_primary_fixed,
        "onPrimary and onPrimaryFixed should differ in dark mode"
    );
}

#[test]
fn test_theme_dark_mode_detection() {
    let ctx = egui::Context::default();
    let mut theme = MaterialThemeContext {
        theme_mode: ThemeMode::Light,
        ..Default::default()
    };
    apply_theme_context(&ctx, &theme, None::<fn() -> ThemeMode>);
    assert!(!theme_context_of(&ctx).dark_mode, "Should be in light mode");

    theme.theme_mode = ThemeMode::Dark;
    apply_theme_context(&ctx, &theme, None::<fn() -> ThemeMode>);
    assert!(theme_context_of(&ctx).dark_mode, "Should be in dark mode");
}
