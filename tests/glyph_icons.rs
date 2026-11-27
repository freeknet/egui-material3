// ============================================================================
// Module:       glyph_icons (integration test)
// Description:  Icons drawn from a font the app registered itself.
//
// Dependencies: egui, egui_material3
// ============================================================================

//! `Icon::Glyph`: a character from an app-registered font family, drawn by
//! every widget that takes an `Icon` in that family, at the icon size and
//! in the widget's icon tint.

use egui::epaint::{ClippedShape, Shape as PaintShape, TextShape};
use egui::{FontData, FontDefinitions, FontFamily, Pos2, Rect};
use egui_material3::tokens::ColorRole;
use egui_material3::{
    assist_chip, fab_primary, Icon, MaterialIconButton, MaterialTheme, NavigationBar,
    NavigationItem, Segment, SegmentedButton,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// The app's own icon family. It reuses egui's bundled Hack font, so the
/// test needs no font file of its own.
fn app_family() -> FontFamily {
    FontFamily::Name("app-icons".into())
}

/// A context with the `app-icons` family registered and the baseline theme.
fn context() -> Result<egui::Context, String> {
    let ctx = egui::Context::default();
    let mut fonts = FontDefinitions::default();
    let hack = fonts
        .font_data
        .get("Hack")
        .cloned()
        .ok_or("egui's default fonts include Hack")?;
    fonts.font_data.insert(
        "app-icons-font".to_owned(),
        std::sync::Arc::new(FontData::clone(&hack)),
    );
    fonts
        .families
        .insert(app_family(), vec!["app-icons-font".to_owned()]);
    ctx.set_fonts(fonts);
    MaterialTheme::set(&ctx, MaterialTheme::baseline(false));
    Ok(ctx)
}

/// Runs two frames (the first only loads the fonts) and returns the shapes.
fn shapes(ctx: &egui::Context, app: &mut dyn FnMut(&mut egui::Ui)) -> Vec<ClippedShape> {
    let input = || egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 400.0))),
        ..Default::default()
    };
    // The first frame only loads the fonts; its output is not needed.
    ctx.run_ui(input(), |ui| app(ui)).textures_delta.clear();
    let mut output = ctx.run_ui(input(), |ui| app(ui));
    output.textures_delta.clear();
    output.shapes
}

fn texts(shapes: &[ClippedShape]) -> Vec<TextShape> {
    shapes
        .iter()
        .filter_map(|c| match &c.shape {
            PaintShape::Text(t) => Some(t.clone()),
            PaintShape::Noop
            | PaintShape::Vec(_)
            | PaintShape::Circle(_)
            | PaintShape::Ellipse(_)
            | PaintShape::LineSegment { .. }
            | PaintShape::Path(_)
            | PaintShape::Band(_)
            | PaintShape::Rect(_)
            | PaintShape::Mesh(_)
            | PaintShape::QuadraticBezier(_)
            | PaintShape::CubicBezier(_)
            | PaintShape::Callback(_) => None,
        })
        .collect()
}

/// The text shape that draws `ch` in the app family, if any.
fn glyph_shape(shapes: &[ClippedShape], ch: char) -> Option<TextShape> {
    texts(shapes).into_iter().find(|t| {
        t.galley.text() == ch.to_string()
            && t.galley
                .job
                .sections
                .first()
                .is_some_and(|s| s.format.font_id.family == app_family())
    })
}

#[test]
fn glyph_constructor_builds_the_variant() {
    assert_eq!(
        Icon::glyph('x', app_family()),
        Icon::Glyph {
            ch: 'x',
            family: app_family()
        },
        "Icon::glyph is shorthand for the variant"
    );
}

#[test]
fn icon_button_paints_a_glyph_in_its_family() -> TestResult {
    let ctx = context()?;
    let shapes = shapes(&ctx, &mut |ui| {
        ui.add(MaterialIconButton::standard(Icon::glyph('Q', app_family())).label("Quit"));
    });
    let glyph = glyph_shape(&shapes, 'Q').ok_or("the glyph is painted in app-icons")?;
    let format = glyph
        .galley
        .job
        .sections
        .first()
        .ok_or("one section")?
        .format
        .clone();
    // IconButtonTokens / StandardIconButtonTokens: 24 dp icon in
    // onSurfaceVariant.
    assert_eq!(format.font_id.size, 24.0, "drawn at the icon size");
    assert_eq!(
        format.color,
        MaterialTheme::baseline(false).color(ColorRole::OnSurfaceVariant),
        "drawn in the standard icon button's icon colour"
    );
    Ok(())
}

#[test]
fn chip_leading_glyph_is_painted_in_its_family() -> TestResult {
    let ctx = context()?;
    let shapes = shapes(&ctx, &mut |ui| {
        ui.add(assist_chip("Run").leading_icon(Icon::glyph('R', app_family())));
    });
    let glyph = glyph_shape(&shapes, 'R').ok_or("the chip paints the leading glyph")?;
    let size = glyph
        .galley
        .job
        .sections
        .first()
        .map(|s| s.format.font_id.size);
    // AssistChipTokens.IconSize: 18 dp.
    assert_eq!(size, Some(18.0), "chip icons are 18 dp");
    Ok(())
}

#[test]
fn fab_navigation_and_segments_take_glyphs() -> TestResult {
    let ctx = context()?;
    let mut selected = 0;
    let mut segment = 0;
    let shapes = shapes(&ctx, &mut |ui| {
        ui.add(fab_primary().icon(Icon::glyph('F', app_family())));
        ui.add(
            NavigationBar::new(&mut selected)
                .item(NavigationItem::new(Icon::glyph('N', app_family()), "Home"))
                .item(NavigationItem::new("mail", "Mail")),
        );
        ui.add(
            SegmentedButton::single(&mut segment)
                .segment(Segment::new("Day").icon(Icon::glyph('D', app_family())))
                .segment(Segment::new("Week")),
        );
    });
    assert!(
        glyph_shape(&shapes, 'F').is_some(),
        "the FAB paints its glyph"
    );
    assert!(
        glyph_shape(&shapes, 'N').is_some(),
        "the navigation item paints its glyph"
    );
    // Segment 0 is selected and shows the check instead; segment icons only
    // show unselected, so select the other one.
    let mut segment = 1;
    let shapes2 = self::shapes(&ctx, &mut |ui| {
        ui.add(
            SegmentedButton::single(&mut segment)
                .segment(Segment::new("Day").icon(Icon::glyph('D', app_family())))
                .segment(Segment::new("Week")),
        );
    });
    assert!(
        glyph_shape(&shapes2, 'D').is_some(),
        "an unselected segment paints its glyph"
    );
    Ok(())
}
