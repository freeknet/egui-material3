// ============================================================================
// Module:       banners (integration test)
// Description:  The inline banner: tones, layout, actions, dismiss, a11y.
//
// Dependencies: egui, egui_kittest, egui_material3
// ============================================================================

//! `MaterialBanner`: theme-derived tone colours, the 12 dp filled-card
//! container, the action and dismiss buttons and what they report, the
//! narrow-width stacked layout, and the AccessKit tree.

mod support;

use std::cell::RefCell;

use egui::accesskit::Role;
use egui::epaint::{ClippedShape, RectShape, Shape as PaintShape};
use egui::{Event, Pos2, Rect};
use egui_kittest::kittest::{NodeT, Queryable};
use egui_material3::tokens::ColorRole;
use egui_material3::{BannerExt, BannerResponse, BannerTone, MaterialBanner, MaterialTheme};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn rects(shapes: &[ClippedShape]) -> Vec<RectShape> {
    shapes
        .iter()
        .filter_map(|c| match &c.shape {
            PaintShape::Rect(r) => Some(r.clone()),
            PaintShape::Noop
            | PaintShape::Vec(_)
            | PaintShape::Circle(_)
            | PaintShape::Ellipse(_)
            | PaintShape::LineSegment { .. }
            | PaintShape::Path(_)
            | PaintShape::Band(_)
            | PaintShape::Text(_)
            | PaintShape::Mesh(_)
            | PaintShape::QuadraticBezier(_)
            | PaintShape::CubicBezier(_)
            | PaintShape::Callback(_) => None,
        })
        .collect()
}

/// Runs a frame of `banner` in a `width`-wide area and returns the shapes,
/// the banner's response and its rect.
fn draw(
    ctx: &egui::Context,
    width: f32,
    events: Vec<Event>,
    make: &dyn Fn() -> MaterialBanner,
) -> (Vec<ClippedShape>, Option<BannerResponse>) {
    let out = RefCell::new(None);
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(width, 400.0))),
        events,
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| {
        *out.borrow_mut() = Some(ui.add_banner(make()));
    });
    output.textures_delta.clear();
    let shapes = output.shapes;
    (shapes, out.into_inner())
}

fn context(theme: MaterialTheme) -> egui::Context {
    let ctx = egui::Context::default();
    MaterialTheme::set(&ctx, theme);
    ctx
}

#[test]
fn tones_use_theme_roles_and_a_medium_container() -> TestResult {
    for (name, theme) in support::themes() {
        let expected = [
            (BannerTone::Info, theme.color(ColorRole::SecondaryContainer)),
            (BannerTone::Error, theme.color(ColorRole::ErrorContainer)),
            (BannerTone::Warning, theme.warning().color_container),
            (BannerTone::Success, theme.success().color_container),
        ];
        for (tone, fill) in expected {
            let ctx = context(theme.clone());
            let (shapes, response) = draw(&ctx, 600.0, vec![], &|| {
                MaterialBanner::new("Saved").tone(tone)
            });
            let rect = response.ok_or("banner shown")?.response.rect;
            let container = rects(&shapes)
                .into_iter()
                .find(|r| r.fill == fill)
                .ok_or_else(|| format!("{tone:?} ({name}) paints its container"))?;
            assert_eq!(
                container.rect, rect,
                "{tone:?}: the container is the banner"
            );
            // FilledCardTokens.ContainerShape = CornerMedium (12).
            assert_eq!(
                container.corner_radius,
                egui::CornerRadius::same(12),
                "{tone:?}: 12 dp corners"
            );
            assert_eq!(rect.width(), 600.0, "{tone:?}: fills the width");
        }
    }
    Ok(())
}

#[test]
fn actions_and_dismiss_report_clicks() -> TestResult {
    let make = || {
        MaterialBanner::new("Update available")
            .icon("system_update")
            .action("Later")
            .action("Install")
            .action("Ignored third action")
            .dismissible(true)
    };
    let last = std::rc::Rc::new(RefCell::new(None::<BannerResponse>));
    let sink = last.clone();
    let mut harness = support::harness_for(
        MaterialTheme::baseline(false),
        egui::vec2(700.0, 200.0),
        move |ui| {
            *sink.borrow_mut() = Some(ui.add_banner(make()));
        },
    );
    let idle = last.borrow().clone().ok_or("banner shown")?;
    assert!(
        !idle.dismissed() && idle.clicked_action().is_none(),
        "nothing clicked yet"
    );
    assert!(
        harness.query_by_label("Ignored third action").is_none(),
        "at most two actions"
    );
    let install = harness.get_by_label("Install").rect();
    let dismiss = harness.get_by_label("Dismiss").rect();
    assert!(
        install.max.x < dismiss.min.x,
        "the dismiss button is at the end"
    );
    assert_eq!(
        harness.get_by_label("Install").accesskit_node().role(),
        Role::Button,
        "actions are buttons"
    );
    assert_eq!(
        harness
            .get_by_label("Update available")
            .accesskit_node()
            .role(),
        Role::Status,
        "the banner is a status"
    );

    // Watch every frame of the click for the one that reports it.
    let clicked_action = |h: &mut egui_kittest::Harness<'_>, label: &str| {
        h.get_by_label(label).click();
        let mut seen = (None, false);
        for _ in 0..3 {
            h.step();
            if let Some(r) = last.borrow().as_ref() {
                seen.0 = seen.0.or(r.clicked_action());
                seen.1 |= r.dismissed();
            }
        }
        seen
    };
    assert_eq!(
        clicked_action(&mut harness, "Install"),
        (Some(1), false),
        "Install is action 1, and it does not dismiss"
    );
    assert_eq!(
        clicked_action(&mut harness, "Dismiss"),
        (None, true),
        "the close button dismisses"
    );
    Ok(())
}

#[test]
fn narrow_banners_move_actions_below_the_message() -> TestResult {
    let ctx = context(MaterialTheme::baseline(false));
    let make = || {
        MaterialBanner::new("Storage almost full")
            .action("Manage storage")
            .action("Not now")
    };
    let (_, wide) = draw(&ctx, 800.0, vec![], &make);
    let (_, narrow) = draw(&ctx, 300.0, vec![], &make);
    let wide = wide.ok_or("banner shown")?.response.rect;
    let narrow = narrow.ok_or("banner shown")?.response.rect;
    assert!(
        narrow.height() > wide.height(),
        "a second row for the actions: {} vs {}",
        narrow.height(),
        wide.height()
    );
    assert!(wide.height() >= 56.0, "a 40 dp target and its padding fit");
    Ok(())
}
