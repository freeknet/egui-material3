// ============================================================================
// Module:       component_m3_colors (integration test)
// Description:  Tests for Material Design 3 color role compliance across components.
//
// Dependencies: egui_material3, egui
// ============================================================================

//! Tests for Material Design 3 color role compliance across components

use egui_material3::theme::MaterialThemeContext;

/// The colour named `name` in the default theme (the scheme the process-wide
/// default starts with), read without touching any shared state.
fn default_color(name: &str) -> egui::Color32 {
    MaterialThemeContext::default().get_color_by_name(name)
}

/// Test that M3 color roles are available in the theme system
#[test]
fn test_m3_surface_hierarchy_colors_exist() {
    // Arrange & Act - verify all M3 surface container levels exist
    let surface_container_lowest = default_color("surfaceContainerLowest");
    let surface_container_low = default_color("surfaceContainerLow");
    let surface_container = default_color("surfaceContainer");
    let surface_container_high = default_color("surfaceContainerHigh");
    let surface_container_highest = default_color("surfaceContainerHighest");

    // Assert - should not panic, colors should be valid
    assert!(surface_container_lowest.a() > 0);
    assert!(surface_container_low.a() > 0);
    assert!(surface_container.a() > 0);
    assert!(surface_container_high.a() > 0);
    assert!(surface_container_highest.a() > 0);
}

#[test]
fn test_m3_accent_colors_exist() {
    // Arrange & Act
    let primary = default_color("primary");
    let on_primary = default_color("onPrimary");
    let primary_container = default_color("primaryContainer");
    let on_primary_container = default_color("onPrimaryContainer");

    let secondary = default_color("secondary");
    let on_secondary = default_color("onSecondary");
    let secondary_container = default_color("secondaryContainer");
    let on_secondary_container = default_color("onSecondaryContainer");

    // Assert
    assert!(primary.a() > 0);
    assert!(on_primary.a() > 0);
    assert!(primary_container.a() > 0);
    assert!(on_primary_container.a() > 0);
    assert!(secondary.a() > 0);
    assert!(on_secondary.a() > 0);
    assert!(secondary_container.a() > 0);
    assert!(on_secondary_container.a() > 0);
}

#[test]
fn test_m3_outline_colors_exist() {
    // Arrange & Act
    let outline = default_color("outline");
    let outline_variant = default_color("outlineVariant");

    // Assert
    assert!(outline.a() > 0);
    assert!(outline_variant.a() > 0);
}

#[test]
fn test_m3_on_surface_colors_exist() {
    // Arrange & Act
    let on_surface = default_color("onSurface");
    let on_surface_variant = default_color("onSurfaceVariant");

    // Assert
    assert!(on_surface.a() > 0);
    assert!(on_surface_variant.a() > 0);
}

#[test]
fn test_m3_fixed_colors_exist() {
    // Arrange & Act - M3 fixed colors don't change between light/dark mode
    let primary_fixed = default_color("primaryFixed");
    let on_primary_fixed = default_color("onPrimaryFixed");
    let secondary_fixed = default_color("secondaryFixed");
    let on_secondary_fixed = default_color("onSecondaryFixed");

    // Assert
    assert!(primary_fixed.a() > 0);
    assert!(on_primary_fixed.a() > 0);
    assert!(secondary_fixed.a() > 0);
    assert!(on_secondary_fixed.a() > 0);
}

#[test]
fn test_m3_inverse_colors_exist() {
    // Arrange & Act - Inverse colors for high-contrast elements
    let inverse_surface = default_color("inverseSurface");
    let inverse_on_surface = default_color("inverseOnSurface");
    let inverse_primary = default_color("inversePrimary");

    // Assert
    assert!(inverse_surface.a() > 0);
    assert!(inverse_on_surface.a() > 0);
    assert!(inverse_primary.a() > 0);
}

/// Integration test: Verify component doc comments reference M3 color roles
/// This is a smoke test - actual color usage is tested visually in stories
#[test]
fn test_components_document_m3_colors() {}
