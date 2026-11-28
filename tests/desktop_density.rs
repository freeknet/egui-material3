// ============================================================================
// Module:       desktop_density (integration test)
// Description:  Dense desktop gaps: virtualised list rows, short list rows,
//               desktop text-field densities, dense icon buttons, search
//               sections and pointer-anchored tooltips.
//
// Dependencies: egui, egui_kittest, egui_material3
// ============================================================================

//! The density gaps a desktop app reported, each checked headlessly:
//! `MaterialList::show_rows` (only the viewport's rows are built, with
//! responses, selection, keyboard roving and AccessKit set positions),
//! `ListItem::row_height` below the text's natural height, the 28 and
//! 24 dp text-field densities with height, font and glyph-icon overrides,
//! icon-button icon size and the optional 48 dp touch target, search
//! section headings and the empty line, and the pointer-anchored tooltip.

mod support;

use std::cell::{Cell, RefCell};

use egui::accesskit::{Role, Toggled};
use egui::epaint::{ClippedShape, Shape as PaintShape, TextShape};
use egui::{Event, FontData, FontDefinitions, FontFamily, Key, Pos2, Rect, ScrollArea, Vec2};
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::Harness;
use egui_material3::input::{container_height_for_density, MIN_DESKTOP_DENSITY};
use egui_material3::search::{self, MaterialSearch};
use egui_material3::{
    Icon, ListItem, MaterialIconButton, MaterialInput, MaterialList, MaterialTheme,
    MaterialTooltip, TooltipAnchor,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn light() -> MaterialTheme {
    MaterialTheme::baseline(false)
}

fn texts(shapes: &[ClippedShape]) -> Vec<TextShape> {
    fn collect(shape: &PaintShape, out: &mut Vec<TextShape>) {
        match shape {
            PaintShape::Text(t) => out.push(t.clone()),
            PaintShape::Vec(shapes) => shapes.iter().for_each(|s| collect(s, out)),
            PaintShape::Noop
            | PaintShape::Circle(_)
            | PaintShape::Ellipse(_)
            | PaintShape::LineSegment { .. }
            | PaintShape::Path(_)
            | PaintShape::Band(_)
            | PaintShape::Rect(_)
            | PaintShape::Mesh(_)
            | PaintShape::QuadraticBezier(_)
            | PaintShape::CubicBezier(_)
            | PaintShape::Callback(_) => {}
        }
    }
    let mut out = Vec::new();
    for clipped in shapes {
        collect(&clipped.shape, &mut out);
    }
    out
}

/// The first text shape whose galley says `text`.
fn text_shape(shapes: &[ClippedShape], text: &str) -> Option<TextShape> {
    texts(shapes)
        .into_iter()
        .find(|t| t.galley.job.text == text)
}

// ---------------------------------------------------------------------------
// MaterialList::show_rows

const ROWS: usize = 100_000;
const ROW_HEIGHT: f32 = 28.0;
const VIEW_HEIGHT: f32 = 200.0;

/// What the virtualised list did, across frames.
#[derive(Default)]
struct Rows {
    /// Rows `build_row` was called for in the last frame.
    built: RefCell<Vec<usize>>,
    /// The visible range of the last frame.
    visible: RefCell<std::ops::Range<usize>>,
    /// Rows reported in the last frame.
    reported: RefCell<Vec<usize>>,
    /// The list's height.
    list_height: Cell<f32>,
    /// The scroll offset after the last frame.
    offset: Cell<f32>,
    /// The scroll area's visible rect.
    view: Cell<Option<Rect>>,
    /// The selected row.
    selected: Cell<Option<usize>>,
    /// A row that is disabled.
    disabled: Cell<Option<usize>>,
    /// Space above the list inside the scroll area.
    space_above: Cell<f32>,
    /// The offset the scroll area starts at.
    start_offset: Cell<f32>,
}

impl Rows {
    fn draw(&self, ui: &mut egui::Ui) {
        self.built.borrow_mut().clear();
        let out = ScrollArea::vertical()
            .id_salt("rows")
            .max_height(VIEW_HEIGHT)
            .auto_shrink(false)
            .vertical_scroll_offset(self.start_offset.get())
            .show_viewport(ui, |ui, viewport| {
                ui.add_space(self.space_above.get());
                let selected = self.selected.get();
                let disabled = self.disabled.get();
                let shown = MaterialList::new()
                    .id(egui::Id::unique("virtual"))
                    .dividers(false)
                    .show_rows(ui, viewport, ROWS, ROW_HEIGHT, |index| {
                        self.built.borrow_mut().push(index);
                        ListItem::new(format!("Row {index}"))
                            .dense(true)
                            .selected(selected == Some(index))
                            .enabled(disabled != Some(index))
                    });
                for (index, row) in &shown.rows {
                    if row.clicked() {
                        self.selected.set(Some(*index));
                    }
                }
                *self.visible.borrow_mut() = shown.visible.clone();
                *self.reported.borrow_mut() = shown.rows.iter().map(|(i, _)| *i).collect();
                self.list_height.set(shown.response.rect.height());
            });
        self.offset.set(out.state.clamped_offset().y);
        self.view.set(Some(out.inner_rect));
        // The start offset only applies once.
        self.start_offset.set(out.state.clamped_offset().y);
    }
}

fn rows_harness(rows: &Rows) -> Harness<'_> {
    support::harness_for(light(), Vec2::new(400.0, 300.0), move |ui| rows.draw(ui))
}

#[test]
fn virtual_list_builds_only_the_rows_in_view() -> TestResult {
    let rows = Rows::default();
    let harness = rows_harness(&rows);
    assert_eq!(
        rows.list_height.get(),
        ROWS as f32 * ROW_HEIGHT,
        "the list is as tall as all its rows"
    );
    // 200 / 28 = 7.1: rows 0..=7 are (partly) on screen.
    assert_eq!(*rows.visible.borrow(), 0..8, "rows 0 to 7 are visible");
    let built = rows.built.borrow().clone();
    assert!(
        built.len() <= 8 && built.iter().all(|&i| i < 8),
        "only the visible rows are built, not {ROWS}: {built:?}"
    );
    assert_eq!(
        *rows.reported.borrow(),
        (0..8).collect::<Vec<_>>(),
        "every visible row has a response"
    );
    let row = harness
        .query_by_label("Row 3")
        .ok_or("a visible row is an AccessKit node")?;
    assert_eq!(
        row.accesskit_node().role(),
        Role::ListItem,
        "a row is a list item"
    );
    assert_eq!(
        row.accesskit_node().position_in_set(),
        Some(4),
        "row 3 is the fourth of the set"
    );
    assert_eq!(
        row.accesskit_node().size_of_set(),
        Some(ROWS),
        "the set has every row, built or not"
    );
    assert!(
        (row.rect().height() - ROW_HEIGHT).abs() < 0.01,
        "rows are exactly the row height"
    );
    assert!(
        harness.query_by_label("Row 50").is_none(),
        "a row out of view has no node"
    );
    Ok(())
}

#[test]
fn virtual_list_follows_the_scroll_offset() -> TestResult {
    let rows = Rows::default();
    rows.start_offset.set(1000.0 * ROW_HEIGHT);
    let harness = rows_harness(&rows);
    assert_eq!(
        *rows.visible.borrow(),
        1000..1008,
        "scrolled 1000 rows down, rows 1000 to 1007 show"
    );
    let row = harness
        .query_by_label("Row 1000")
        .ok_or("the first row in view has a node")?;
    assert!(
        rows.view
            .get()
            .is_some_and(|view| (row.rect().top() - view.top()).abs() < 0.5),
        "the first row in view sits at the top of the scroll area"
    );
    assert!(
        harness.query_by_label("Row 0").is_none(),
        "row 0 is out of view"
    );
    Ok(())
}

#[test]
fn virtual_list_below_other_content_offsets_its_viewport() {
    let rows = Rows::default();
    rows.space_above.set(100.0);
    let _harness = rows_harness(&rows);
    // 100 points of the 200 show the list: 100 / 28 = 3.6, rows 0..=3.
    assert_eq!(
        *rows.visible.borrow(),
        0..4,
        "the viewport is measured from the list's top"
    );
}

#[test]
fn visible_row_range_counts_dividers_and_gaps() {
    let plain = MaterialList::new().dividers(false);
    let divided = MaterialList::new();
    let segmented = MaterialList::new().segmented(true);
    let view = egui::Rangef::new(290.0, 300.0);
    assert_eq!(
        plain.visible_row_range(view, 100, 28.0),
        10..11,
        "28 dp stride"
    );
    assert_eq!(
        divided.visible_row_range(view, 100, 28.0),
        10..11,
        "29 dp stride with the 1 dp divider"
    );
    assert_eq!(
        segmented.visible_row_range(egui::Rangef::new(300.0, 310.0), 100, 28.0),
        10..11,
        "30 dp stride with the 2 dp segmented gap"
    );
    assert_eq!(
        plain.visible_row_range(egui::Rangef::new(-50.0, 1e9), 5, 28.0),
        0..5,
        "the range clamps to the row count"
    );
    assert_eq!(
        plain.visible_row_range(egui::Rangef::new(10.0, 10.0), 5, 28.0),
        0..0,
        "an empty viewport shows no rows"
    );
}

#[test]
fn virtual_list_rows_click_select_and_report_selection() -> TestResult {
    let rows = Rows::default();
    let mut harness = rows_harness(&rows);
    harness
        .query_by_label("Row 2")
        .ok_or("row 2 is shown")?
        .click();
    harness.run();
    assert_eq!(rows.selected.get(), Some(2), "a click reports its row");
    let toggled = harness
        .query_by_label("Row 2")
        .ok_or("row 2 is shown")?
        .accesskit_node()
        .toggled();
    assert_eq!(
        toggled,
        Some(Toggled::True),
        "the selected row reports its selection"
    );
    Ok(())
}

#[test]
fn virtual_list_roves_past_the_viewport_and_scrolls_with_focus() -> TestResult {
    let rows = Rows::default();
    rows.disabled.set(Some(3));
    let mut harness = rows_harness(&rows);
    harness.key_press(Key::Tab);
    harness.run();
    assert!(
        harness.query_by_label("Row 0").ok_or("row 0")?.is_focused(),
        "Tab focuses the first row"
    );
    for _ in 0..3 {
        harness.key_press(Key::ArrowDown);
        harness.run();
    }
    assert!(
        harness.query_by_label("Row 4").ok_or("row 4")?.is_focused(),
        "Down skips the disabled row 3"
    );
    for _ in 0..8 {
        harness.key_press(Key::ArrowDown);
        harness.run();
    }
    let focused = harness.query_by_label("Row 12").ok_or("row 12 is built")?;
    assert!(
        focused.is_focused(),
        "focus reaches a row below the viewport"
    );
    assert!(
        rows.offset.get() > 0.0,
        "the list scrolled to the focused row"
    );
    let view = rows.view.get().ok_or("the scroll area was shown")?;
    assert!(
        view.contains_rect(focused.rect().shrink(0.5)),
        "the focused row is in view: {:?} in {view:?}",
        focused.rect()
    );
    harness.key_press(Key::ArrowUp);
    harness.run();
    assert!(
        harness
            .query_by_label("Row 11")
            .ok_or("row 11")?
            .is_focused(),
        "Up moves back"
    );
    Ok(())
}

#[test]
fn virtual_list_keeps_focus_on_a_row_scrolled_out_of_view() -> TestResult {
    let rows = Rows::default();
    let mut harness = rows_harness(&rows);
    harness.key_press(Key::Tab);
    harness.run();
    // Scroll far away with the wheel; the focused row is no longer visible.
    rows.start_offset.set(5000.0 * ROW_HEIGHT);
    harness.run();
    harness.run();
    assert!(
        !rows.visible.borrow().contains(&0),
        "row 0 is out of the viewport"
    );
    assert!(
        rows.reported.borrow().contains(&0),
        "the focused row is still built"
    );
    assert!(
        harness
            .query_by_label("Row 0")
            .ok_or("row 0 keeps its node")?
            .is_focused(),
        "and keeps focus"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// ListItem::row_height

#[test]
fn row_height_makes_rows_shorter_than_the_text_default() -> TestResult {
    for (name, theme) in support::themes() {
        let harness = support::harness_for(theme, Vec2::new(400.0, 300.0), |ui| {
            ui.add(
                MaterialList::new()
                    .dividers(false)
                    .item(ListItem::new("Short").dense(true).row_height(28.0))
                    .item(ListItem::new("Default dense").dense(true))
                    .item(ListItem::new("Short body").row_height(28.0)),
            );
        });
        let height = |label: &str| -> Result<f32, String> {
            harness
                .query_by_label(label)
                .map(|n| n.rect().height())
                .ok_or_else(|| format!("{label} is a node"))
        };
        assert_eq!(height("Short")?, 28.0, "{name}: a 28 dp row");
        assert_eq!(
            height("Default dense")?,
            48.0,
            "{name}: without it a dense one-line item keeps 48 dp"
        );
        assert_eq!(
            height("Short body")?,
            28.0,
            "{name}: the override wins over the BodyLarge text floor (44 dp)"
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// MaterialInput

#[test]
fn desktop_densities_are_28_and_24_dp() {
    assert_eq!(container_height_for_density(0), 56.0, "the default");
    assert_eq!(container_height_for_density(-3), 32.0, "compact");
    assert_eq!(container_height_for_density(-4), 28.0, "desktop -4");
    assert_eq!(container_height_for_density(-5), 24.0, "desktop -5");
    assert_eq!(
        container_height_for_density(-100),
        24.0,
        "clamped to the densest desktop step"
    );
    assert_eq!(MIN_DESKTOP_DENSITY, -5, "the densest step");
}

/// The container rect of the field `build` makes, if a node is named
/// `label` (the field keeps its label as its accessible name).
fn field_rect(build: impl Fn(&mut String) -> MaterialInput<'_> + 'static, label: &str) -> Rect {
    let rect = std::rc::Rc::new(Cell::new(Rect::NOTHING));
    let seen = rect.clone();
    let mut text = String::new();
    let harness = support::harness_for(light(), Vec2::new(400.0, 200.0), move |ui| {
        seen.set(build(&mut text).show(ui).response.rect);
    });
    if harness.query_by_label(label).is_none() {
        return Rect::NOTHING;
    }
    rect.get()
}

#[test]
fn dense_fields_take_the_desktop_heights_and_keep_their_name() {
    let rect = field_rect(
        |t| MaterialInput::filled(t).label("Gain").density(-4),
        "Gain",
    );
    assert!(
        (rect.height() - 28.0).abs() < 0.5,
        "density -4 is 28 dp, keeping the label as its name: {rect:?}"
    );
    let rect = field_rect(
        |t| MaterialInput::outlined(t).label("Pan").density(-5),
        "Pan",
    );
    assert!(
        (rect.height() - 24.0).abs() < 0.5,
        "density -5 is 24 dp: {rect:?}"
    );
    let rect = field_rect(
        |t| {
            MaterialInput::filled(t)
                .label("Track")
                .height(22.0)
                .font(egui::FontId::proportional(12.0))
        },
        "Track",
    );
    assert!(
        (rect.height() - 22.0).abs() < 0.5,
        "an explicit height is exact, even below the BodyLarge line: {rect:?}"
    );
    let rect = field_rect(|t| MaterialInput::filled(t).label("Normal"), "Normal");
    assert!(
        (rect.height() - 56.0).abs() < 0.5,
        "the default stays 56 dp: {rect:?}"
    );
}

#[test]
fn a_font_override_sets_the_text_font() -> TestResult {
    let mut text = String::from("Kick");
    let harness = support::harness_for(light(), Vec2::new(400.0, 200.0), move |ui| {
        let _ = MaterialInput::filled(&mut text)
            .label("Instrument")
            .density(-5)
            .font(egui::FontId::monospace(11.0))
            .show(ui);
    });
    let shape = text_shape(&harness.output().shapes, "Kick").ok_or("the text is painted")?;
    let font = shape
        .galley
        .job
        .sections
        .first()
        .map(|s| s.format.font_id.clone())
        .ok_or("the text has a section")?;
    assert_eq!(
        font,
        egui::FontId::monospace(11.0),
        "the text uses the override font"
    );
    Ok(())
}

/// A context with an `app-icons` family (egui's Hack font under another
/// name) and the baseline theme.
fn app_font_context() -> Result<egui::Context, String> {
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
    fonts.families.insert(
        FontFamily::Name("app-icons".into()),
        vec!["app-icons-font".to_owned()],
    );
    ctx.set_fonts(fonts);
    MaterialTheme::set(&ctx, light());
    Ok(ctx)
}

fn frame_shapes(ctx: &egui::Context, app: &mut dyn FnMut(&mut egui::Ui)) -> Vec<ClippedShape> {
    let input = || egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(600.0, 300.0))),
        ..Default::default()
    };
    // The first frame only loads the fonts.
    ctx.run_ui(input(), |ui| app(ui)).textures_delta.clear();
    let mut output = ctx.run_ui(input(), |ui| app(ui));
    output.textures_delta.clear();
    output.shapes
}

#[test]
fn field_icons_take_app_font_glyphs_and_shrink_in_dense_fields() -> TestResult {
    let ctx = app_font_context()?;
    let family = FontFamily::Name("app-icons".into());
    let mut text = String::new();
    let mut app = |ui: &mut egui::Ui| {
        let _ = MaterialInput::outlined(&mut text)
            .density(-4)
            .leading_icon(Icon::glyph('F', family.clone()))
            .trailing_icon_button(Icon::glyph('X', family.clone()), "Clear")
            .show(ui);
    };
    let shapes = frame_shapes(&ctx, &mut app);
    for glyph in ["F", "X"] {
        let shape = text_shape(&shapes, glyph).ok_or("the glyph is painted")?;
        let font = shape
            .galley
            .job
            .sections
            .first()
            .map(|s| s.format.font_id.clone())
            .ok_or("the glyph has a section")?;
        assert_eq!(
            font.family, family,
            "{glyph} is drawn in the app's icon family"
        );
        assert_eq!(
            font.size, 20.0,
            "{glyph} shrinks to 20 dp in a 28 dp field (4 dp above and below)"
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// MaterialIconButton

fn icon_button_rect(button: impl Fn() -> MaterialIconButton<'static> + 'static) -> Rect {
    let rect = std::rc::Rc::new(Cell::new(Rect::NOTHING));
    let seen = rect.clone();
    let _harness = support::harness_for(light(), Vec2::new(200.0, 200.0), move |ui| {
        seen.set(ui.add(button()).rect);
    });
    rect.get()
}

#[test]
fn touch_target_is_on_by_default_and_can_be_turned_off() {
    let default = icon_button_rect(|| MaterialIconButton::standard("play_arrow").size(24.0));
    assert_eq!(
        default.size(),
        Vec2::splat(48.0),
        "a 24 dp button keeps the 48 dp touch target by default"
    );
    let dense = icon_button_rect(|| {
        MaterialIconButton::standard("play_arrow")
            .size(24.0)
            .min_touch_target(false)
    });
    assert_eq!(
        dense.size(),
        Vec2::splat(24.0),
        "without it the button takes exactly its container"
    );
    let large = icon_button_rect(|| MaterialIconButton::filled("add").min_touch_target(false));
    assert_eq!(
        large.size(),
        Vec2::splat(40.0),
        "the baseline 40 dp button is its container size either way"
    );
}

#[test]
fn icon_size_overrides_the_token_glyph_size() -> TestResult {
    let ctx = app_font_context()?;
    let family = FontFamily::Name("app-icons".into());
    let mut app = |ui: &mut egui::Ui| {
        ui.add(
            MaterialIconButton::standard(Icon::glyph('P', family.clone()))
                .size(24.0)
                .icon_size(16.0)
                .min_touch_target(false)
                .label("Play"),
        );
        ui.add(MaterialIconButton::standard(Icon::glyph('Q', family.clone())).label("Queue"));
    };
    let shapes = frame_shapes(&ctx, &mut app);
    let size = |glyph: &str| -> Result<f32, String> {
        text_shape(&shapes, glyph)
            .and_then(|s| s.galley.job.sections.first().map(|s| s.format.font_id.size))
            .ok_or_else(|| format!("{glyph} is painted"))
    };
    assert_eq!(size("P")?, 16.0, "icon_size(16) draws a 16 dp glyph");
    assert_eq!(
        size("Q")?,
        24.0,
        "the default is the size's IconSize token (24 dp)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// MaterialSearch sections and empty text

const SEARCH_ID: &str = "desktop_search";

#[derive(Default)]
struct Palette {
    query: RefCell<String>,
    selected: Cell<Option<usize>>,
    empty: Cell<bool>,
}

impl Palette {
    fn draw(&self, ui: &mut egui::Ui) {
        let mut query = self.query.borrow_mut();
        let search = MaterialSearch::new(SEARCH_ID, &mut query)
            .placeholder("Command")
            .empty_text("No matching commands");
        let search = if self.empty.get() {
            search.section("Recent", Vec::<&str>::new())
        } else {
            search
                .section("Recent", ["Open project", "Export"])
                .section("Nothing here", Vec::<&str>::new())
                .section("Commands", ["New track", "Split clip"])
        };
        let out = search.show(ui);
        if out.selected.is_some() {
            self.selected.set(out.selected);
        }
    }
}

fn open_palette(palette: &Palette) -> Result<Harness<'_>, String> {
    let mut harness =
        support::harness_for(light(), Vec2::new(480.0, 480.0), move |ui| palette.draw(ui));
    harness
        .query_by_role(Role::SearchInput)
        .ok_or("the search field")?
        .focus();
    harness.run();
    Ok(harness)
}

#[test]
fn sections_show_headings_above_their_suggestions() -> TestResult {
    let palette = Palette::default();
    let harness = open_palette(&palette)?;
    let headings: Vec<String> = harness
        .query_all_by_role(Role::Heading)
        .filter_map(|n| n.accesskit_node().label())
        .collect();
    assert_eq!(
        headings,
        vec!["Recent".to_owned(), "Commands".to_owned()],
        "non-empty sections are headings; the empty one has none"
    );
    let options = harness.query_all_by_role(Role::ListBoxOption).count();
    assert_eq!(options, 4, "headings are not options");
    let top = |role, label: &str| -> Result<f32, String> {
        harness
            .query_by_role_and_label(role, label)
            .map(|n| n.rect().top())
            .ok_or_else(|| format!("{label} is shown"))
    };
    assert!(
        top(Role::Heading, "Commands")? < top(Role::ListBoxOption, "New track")?,
        "a heading sits above its first suggestion"
    );
    assert!(
        top(Role::ListBoxOption, "Export")? < top(Role::Heading, "Commands")?,
        "and below the previous section"
    );
    let heading = harness
        .query_by_role_and_label(Role::Heading, "Recent")
        .ok_or("Recent heading")?;
    assert_eq!(heading.rect().height(), 40.0, "heading rows are 40 dp");
    assert!(
        harness.query_by_label("No matching commands").is_none(),
        "the empty line only shows without suggestions"
    );
    Ok(())
}

#[test]
fn the_highlight_skips_headings_and_indices_count_suggestions() -> TestResult {
    let palette = Palette::default();
    let mut harness = open_palette(&palette)?;
    for _ in 0..3 {
        harness.key_press(Key::ArrowDown);
        harness.run();
    }
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        palette.selected.get(),
        Some(2),
        "the third suggestion is index 2 whatever headings come before it"
    );
    assert_eq!(
        palette.query.borrow().as_str(),
        "New track",
        "picking puts its text in the query"
    );
    assert!(
        !search::is_expanded(&harness.ctx, SEARCH_ID),
        "picking collapses"
    );
    Ok(())
}

#[test]
fn empty_text_shows_when_nothing_matches() -> TestResult {
    let palette = Palette::default();
    palette.empty.set(true);
    let harness = open_palette(&palette)?;
    let line = harness
        .query_by_role_and_label(Role::Label, "No matching commands")
        .ok_or("the empty line is a label node")?;
    assert_eq!(line.rect().height(), 56.0, "a one-line 56 dp row");
    assert_eq!(
        harness.query_all_by_role(Role::ListBoxOption).count(),
        0,
        "no options"
    );
    assert_eq!(
        harness.query_all_by_role(Role::Heading).count(),
        0,
        "an empty section shows no heading"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Pointer-anchored tooltip

const SCREEN: Vec2 = egui::vec2(1000.0, 600.0);

struct Driver {
    ctx: egui::Context,
    time: f64,
}

impl Driver {
    fn new() -> Self {
        let ctx = egui::Context::default();
        MaterialTheme::set(&ctx, light());
        Self { ctx, time: 0.0 }
    }

    fn frame(
        &mut self,
        dt: f64,
        events: Vec<Event>,
        app: &mut dyn FnMut(&mut egui::Ui),
    ) -> Vec<ClippedShape> {
        self.time += dt;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(input, |ui| app(ui));
        output.textures_delta.clear();
        output.shapes
    }
}

/// A 800 x 40 timeline at (100, 200) with a tooltip anchored to `anchor`.
fn timeline(anchor: TooltipAnchor, timeline_rect: &Cell<Rect>) -> impl FnMut(&mut egui::Ui) + '_ {
    move |ui: &mut egui::Ui| {
        let rect = Rect::from_min_size(Pos2::new(100.0, 200.0), Vec2::new(800.0, 40.0));
        let response = ui.allocate_rect(rect, egui::Sense::hover());
        timeline_rect.set(response.rect);
        MaterialTooltip::new("Playhead")
            .anchor(anchor)
            .show_for(&response);
    }
}

/// Hovers `pos` and runs frames until the tooltip text shows (or two
/// seconds pass); returns how long it took and the text's rect.
fn hover_until_shown(
    driver: &mut Driver,
    pos: Pos2,
    app: &mut dyn FnMut(&mut egui::Ui),
) -> Result<(f64, Rect), String> {
    driver.frame(0.0, vec![Event::PointerMoved(pos)], app);
    let hovered_at = driver.time;
    while driver.time - hovered_at < 2.0 {
        let shapes = driver.frame(0.05, Vec::new(), app);
        if let Some(shape) = text_shape(&shapes, "Playhead") {
            return Ok((driver.time - hovered_at, shape.visual_bounding_rect()));
        }
    }
    Err("the tooltip never appeared".to_owned())
}

#[test]
fn pointer_tooltip_appears_at_the_pointer_after_the_delay() -> TestResult {
    let mut driver = Driver::new();
    let rect = Cell::new(Rect::NOTHING);
    let mut app = timeline(TooltipAnchor::Pointer, &rect);
    driver.frame(0.1, Vec::new(), &mut app);
    let pointer = Pos2::new(300.0, 220.0);
    let (after, text) = hover_until_shown(&mut driver, pointer, &mut app)?;
    assert!(
        after >= 0.5,
        "appeared after {after} s, before egui's delay"
    );
    assert!(
        after < 0.8,
        "appeared after {after} s, long after the delay"
    );
    assert!(
        (text.left() - pointer.x).abs() < 60.0 && text.top() > pointer.y,
        "the tooltip sits just under the pointer, not under the widget: {text:?}"
    );
    assert!(
        text.top() - pointer.y >= 12.0,
        "with room for the cursor (12 points): {text:?}"
    );

    // Moving along the timeline moves the tooltip with the pointer.
    let moved = Pos2::new(700.0, 220.0);
    let mut shapes = Vec::new();
    for step in 1..=8 {
        let x = pointer.x + (moved.x - pointer.x) * step as f32 / 8.0;
        shapes = driver.frame(
            0.02,
            vec![Event::PointerMoved(Pos2::new(x, moved.y))],
            &mut app,
        );
    }
    let text = text_shape(&shapes, "Playhead")
        .ok_or("the tooltip stays open while the pointer moves over the widget")?
        .visual_bounding_rect();
    assert!(
        (text.left() - moved.x).abs() < 60.0,
        "the tooltip followed the pointer to x = {}: {text:?}",
        moved.x
    );
    Ok(())
}

#[test]
fn widget_tooltip_stays_beside_the_widget() -> TestResult {
    let mut driver = Driver::new();
    let rect = Cell::new(Rect::NOTHING);
    let mut app = timeline(TooltipAnchor::Widget, &rect);
    driver.frame(0.1, Vec::new(), &mut app);
    let (_, text) = hover_until_shown(&mut driver, Pos2::new(700.0, 220.0), &mut app)?;
    let widget = rect.get();
    assert!(
        text.top() >= widget.bottom() || text.bottom() <= widget.top(),
        "a widget-anchored tooltip sits above or below the widget: {text:?}"
    );
    assert!(
        (text.center().x - 700.0).abs() > 100.0,
        "and does not follow the pointer: {text:?}"
    );
    Ok(())
}

#[test]
fn pointer_tooltip_keeps_its_accessible_name() -> TestResult {
    let rect = Cell::new(Rect::NOTHING);
    let mut harness = Harness::builder()
        .with_size(SCREEN)
        .with_step_dt(0.05)
        .build_ui(timeline(TooltipAnchor::Pointer, &rect));
    MaterialTheme::set(&harness.ctx, light());
    harness.run();
    harness.hover_at(Pos2::new(400.0, 220.0));
    for _ in 0..30 {
        harness.step();
    }
    assert!(
        harness.query_all_by_label("Playhead").next().is_some(),
        "the tooltip is named for screen readers"
    );
    Ok(())
}
