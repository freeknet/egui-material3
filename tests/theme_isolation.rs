// ============================================================================
// Module:       theme_isolation (integration test)
// Description:  Two egui contexts with different themes paint their own
//               colours, with no cross-talk through the process-wide theme.
//
// Dependencies: egui, egui_material3
// ============================================================================

//! Per-context theme isolation.
//!
//! Each context gets a "marker" theme: every colour role is a distinct,
//! recognisable colour (one family per theme). A representative set of
//! widgets is drawn in two contexts, interleaved frame by frame and on two
//! threads at once, while the process-wide default holds a third marker
//! theme. Every opaque colour a widget paints that belongs to a marker
//! family must belong to its own context's family: a widget that read the
//! process-wide theme (or another context's) would paint a foreign marker.

use egui::epaint::{ClippedShape, ColorMode, Shape};
use egui::{Color32, Pos2, Rect, Vec2};
use egui_material3::datatable::data_table;
use egui_material3::theme::MaterialThemeContext;
use egui_material3::{
    assist_chip, checkbox, fab_primary_container, filter_chip, slider, switch, ColorRole, ListItem,
    MaterialButton, MaterialDialog, MaterialList, MaterialTabs, MaterialTheme, MaterialTopAppBar,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// The marker families: the colour of role number `index` in family
/// `family` (0 and 1 are the two contexts, 2 the process-wide default).
fn marker(family: usize, index: usize) -> Color32 {
    let low = u8::try_from(index * 4 + 2).unwrap_or(u8::MAX);
    match family {
        0 => Color32::from_rgb(low, 41, 220),
        1 => Color32::from_rgb(220, 41, low),
        _ => Color32::from_rgb(41, 220, low),
    }
}

/// A theme whose every role is a marker of `family`.
fn marker_theme(family: usize) -> MaterialTheme {
    let mut theme = MaterialTheme::baseline(false);
    for (index, role) in ColorRole::ALL.iter().enumerate() {
        theme.colors.set(*role, marker(family, index));
    }
    theme
}

/// The family `color` (unmultiplied, opaque enough to read back exactly)
/// belongs to, if any.
fn family_of(color: Color32) -> Option<usize> {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    if a < 200 {
        return None;
    }
    let near = |x: u8, y: u8| x.abs_diff(y) <= 1;
    (0..3).find(|&family| {
        (0..ColorRole::ALL.len()).any(|index| {
            let [mr, mg, mb, _] = marker(family, index).to_array();
            near(r, mr) && near(g, mg) && near(b, mb)
        })
    })
}

/// Every colour a shape paints with.
fn colors(shape: &Shape, out: &mut Vec<Color32>) {
    if let Shape::Vec(shapes) = shape {
        for s in shapes {
            colors(s, out);
        }
    } else if let Shape::Rect(r) = shape {
        out.extend([r.fill, r.stroke.color]);
    } else if let Shape::Circle(c) = shape {
        out.extend([c.fill, c.stroke.color]);
    } else if let Shape::Ellipse(e) = shape {
        out.extend([e.fill, e.stroke.color]);
    } else if let Shape::LineSegment { stroke, .. } = shape {
        out.push(stroke.color);
    } else if let Shape::Path(p) = shape {
        out.push(p.fill);
        if let ColorMode::Solid(c) = p.stroke.color {
            out.push(c);
        }
    } else if let Shape::Text(t) = shape {
        out.push(t.fallback_color);
        out.extend(t.override_text_color);
        out.extend(t.galley.job.sections.iter().map(|s| s.format.color));
    } else if let Shape::Mesh(m) = shape {
        out.extend(m.vertices.iter().map(|v| v.color));
    }
}

/// The widgets every context draws, one per test frame sequence.
#[derive(Clone, Copy, Debug)]
enum Widget {
    Button,
    Chip,
    FilterChip,
    ListItem,
    Dialog,
    Tabs,
    DataTable,
    Checkbox,
    Switch,
    Slider,
    Fab,
    TopAppBar,
}

const WIDGETS: [Widget; 12] = [
    Widget::Button,
    Widget::Chip,
    Widget::FilterChip,
    Widget::ListItem,
    Widget::Dialog,
    Widget::Tabs,
    Widget::DataTable,
    Widget::Checkbox,
    Widget::Switch,
    Widget::Slider,
    Widget::Fab,
    Widget::TopAppBar,
];

fn draw(widget: Widget, ui: &mut egui::Ui) {
    match widget {
        Widget::Button => {
            ui.add(MaterialButton::filled("Filled"));
        }
        Widget::Chip => {
            ui.add(assist_chip("Assist"));
        }
        Widget::FilterChip => {
            let mut on = true;
            ui.add(filter_chip("Filter", &mut on));
        }
        Widget::ListItem => {
            ui.add(
                MaterialList::new()
                    .item(ListItem::new("Plain"))
                    .item(ListItem::new("Picked").selected(true)),
            );
        }
        Widget::Dialog => {
            let mut open = true;
            MaterialDialog::new(egui::Id::unique("isolation_dialog"), "Title", &mut open)
                .text_action("OK", || {})
                .show(ui.ctx());
        }
        Widget::Tabs => {
            let mut index = 0;
            ui.add(
                MaterialTabs::primary(&mut index)
                    .tab("Home")
                    .tab("Settings"),
            );
        }
        Widget::DataTable => {
            data_table()
                .id(egui::Id::unique("isolation_table"))
                .column("Name", 120.0, false)
                .column("Value", 80.0, true)
                .row(|row| row.cell("A").cell("1"))
                .row(|row| row.cell("B").cell("2"))
                .show(ui);
        }
        Widget::Checkbox => {
            let mut checked = true;
            ui.add(checkbox(&mut checked, "Checked"));
        }
        Widget::Switch => {
            let mut on = true;
            ui.add(switch(&mut on));
        }
        Widget::Slider => {
            let mut value = 0.5;
            ui.add(slider(&mut value, 0.0..=1.0));
        }
        Widget::Fab => {
            ui.add(fab_primary_container().icon("add"));
        }
        Widget::TopAppBar => {
            ui.add(MaterialTopAppBar::large("Inbox").id_salt("isolation_bar"));
        }
    }
}

/// One context and its clock.
struct Driver {
    ctx: egui::Context,
    time: f64,
}

impl Driver {
    fn new(theme: MaterialTheme) -> Self {
        let ctx = egui::Context::default();
        MaterialTheme::set(&ctx, theme);
        Self { ctx, time: 0.0 }
    }

    fn frame(&mut self, widget: Widget) -> Vec<ClippedShape> {
        self.time += 1.0 / 60.0;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 500.0))),
            time: Some(self.time),
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(input, |ui| draw(widget, ui));
        output.textures_delta.clear();
        output.shapes
    }
}

/// The marker families painted in `shapes`, with a count per family.
fn families(shapes: &[ClippedShape]) -> [usize; 3] {
    let mut painted = Vec::new();
    for clipped in shapes {
        colors(&clipped.shape, &mut painted);
    }
    let mut counts = [0; 3];
    for family in painted.into_iter().filter_map(family_of) {
        if let Some(count) = counts.get_mut(family) {
            *count += 1;
        }
    }
    counts
}

/// Checks that `counts`, painted by `widget` in the context of `family`,
/// hold only that family's markers, and at least one.
fn assert_own(widget: Widget, family: usize, counts: [usize; 3]) {
    for (other, count) in counts.iter().enumerate() {
        if other == family {
            assert!(
                *count > 0,
                "{widget:?} in context {family} painted none of its own theme's colours"
            );
        } else {
            assert_eq!(
                *count, 0,
                "{widget:?} in context {family} painted {count} colours of theme {other}"
            );
        }
    }
}

/// Makes the process-wide default the family-2 marker theme, so a widget
/// that read it instead of its context would be caught.
#[expect(
    deprecated,
    reason = "the test sets the process-wide default through the deprecated shim on purpose"
)]
fn poison_process_default() {
    let mut default = MaterialThemeContext::default();
    for (index, role) in ColorRole::ALL.iter().enumerate() {
        default
            .selected_colors
            .insert(role.name().to_owned(), marker(2, index));
    }
    egui_material3::update_global_theme(default);
}

#[test]
fn process_default_is_the_fallback_only() {
    poison_process_default();
    let bare = egui::Context::default();
    assert_eq!(
        MaterialTheme::of(&bare).color(ColorRole::Primary),
        marker(2, 0),
        "a context with no theme follows the process-wide default"
    );
    let themed = Driver::new(marker_theme(0));
    assert_eq!(
        MaterialTheme::of(&themed.ctx).color(ColorRole::Primary),
        marker(0, 0),
        "a context with its own theme ignores the default"
    );
}

#[test]
fn two_contexts_interleaved_paint_their_own_colours() -> TestResult {
    poison_process_default();
    let mut a = Driver::new(marker_theme(0));
    let mut b = Driver::new(marker_theme(1));
    for widget in WIDGETS {
        // A second of frames, so dialogs and springs settle; A and B take
        // turns every frame.
        let mut last = ([0; 3], [0; 3]);
        for _ in 0..60 {
            last = (families(&a.frame(widget)), families(&b.frame(widget)));
        }
        assert_own(widget, 0, last.0);
        assert_own(widget, 1, last.1);
    }

    // The filled button's container is exactly its context's primary.
    let fill_of = |driver: &mut Driver| {
        let mut painted = Vec::new();
        for clipped in driver.frame(Widget::Button) {
            colors(&clipped.shape, &mut painted);
        }
        painted
    };
    assert!(
        fill_of(&mut a).contains(&marker_theme(0).color(ColorRole::Primary)),
        "context 0's filled button is its own primary"
    );
    assert!(
        fill_of(&mut b).contains(&marker_theme(1).color(ColorRole::Primary)),
        "context 1's filled button is its own primary"
    );
    Ok(())
}

#[test]
fn two_contexts_on_two_threads_paint_their_own_colours() -> TestResult {
    poison_process_default();
    let run = |family: usize| {
        move || {
            let mut driver = Driver::new(marker_theme(family));
            for widget in WIDGETS {
                let mut last = [0; 3];
                for _ in 0..60 {
                    last = families(&driver.frame(widget));
                }
                assert_own(widget, family, last);
            }
        }
    };
    let a = std::thread::spawn(run(0));
    let b = std::thread::spawn(run(1));
    a.join().map_err(|_| "context 0's thread failed")?;
    b.join().map_err(|_| "context 1's thread failed")?;
    Ok(())
}
