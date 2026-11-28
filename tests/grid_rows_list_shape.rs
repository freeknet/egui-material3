// ============================================================================
// Module:       grid_rows_list_shape (integration test)
// Description:  Virtualised layout-grid rows, hover-only list rows, list-item
//               corner radius and list row gaps.
//
// Dependencies: egui, egui_kittest, egui_material3
// ============================================================================

//! `MaterialLayoutGrid::show_rows` builds only the rows in view, gives each
//! tile a response, an AccessKit grid-cell node and stable ids, and roves
//! between focusable tiles with the arrow keys; `MaterialList::row_sense_exact`
//! makes rows that show hover but are neither clickable nor focusable;
//! `ListItem::corner_radius` and `MaterialList::row_gap` shape the rows.

mod support;

use std::cell::{Cell, RefCell};

use egui::accesskit::{Action, Role};
use egui::epaint::{ClippedShape, Shape as PaintShape};
use egui::{CornerRadius, Id, Key, Pos2, Rect, ScrollArea, Sense, Vec2};
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::Harness;
use egui_material3::tokens::ColorRole;
use egui_material3::{ListItem, MaterialLayoutGrid, MaterialList, MaterialTheme};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn light() -> MaterialTheme {
    MaterialTheme::baseline(false)
}

// ---------------------------------------------------------------------------
// MaterialLayoutGrid::show_rows

const ROWS: usize = 50_000;
const COLUMNS: usize = 4;
const ROW_HEIGHT: f32 = 50.0;
const GUTTER: f32 = 10.0;
const VIEW_HEIGHT: f32 = 200.0;

/// What the virtualised grid did, across frames.
#[derive(Default)]
struct Grid {
    /// Rows the row closure ran for in the last frame.
    built: RefCell<Vec<usize>>,
    /// The visible range of the last frame.
    visible: RefCell<std::ops::Range<usize>>,
    /// `(row, tile)` of every tile reported in the last frame.
    reported: RefCell<Vec<(usize, usize)>>,
    /// The grid's height.
    height: Cell<f32>,
    /// The scroll offset after the last frame.
    offset: Cell<f32>,
    /// The scroll area's visible rect.
    view: Cell<Option<Rect>>,
    /// The offset the scroll area starts at.
    start_offset: Cell<f32>,
    /// The last tile clicked.
    clicked: Cell<Option<(usize, usize)>>,
    /// Whether tiles sense clicks (and so take focus).
    clickable: Cell<bool>,
    /// The scope id of the content `Ui` of row 1000, tile 0.
    content_id: Cell<Option<Id>>,
    /// What a fifth cell of span 1 in a full row returned.
    overflow_fits: Cell<Option<bool>>,
}

impl Grid {
    fn draw(&self, ui: &mut egui::Ui) {
        self.built.borrow_mut().clear();
        let out = ScrollArea::vertical()
            .id_salt("grid")
            .max_height(VIEW_HEIGHT)
            .auto_shrink(false)
            .vertical_scroll_offset(self.start_offset.get())
            .show_viewport(ui, |ui, viewport| {
                let sense = if self.clickable.get() {
                    Sense::click()
                } else {
                    Sense::hover()
                };
                let shown = MaterialLayoutGrid::new()
                    .id(Id::unique("virtual grid"))
                    .columns(COLUMNS)
                    .gutter(GUTTER)
                    .margin(0.0)
                    .tile_sense(sense)
                    .show_rows(ui, viewport, ROWS, ROW_HEIGHT, |row, index| {
                        self.built.borrow_mut().push(index);
                        for column in 0..COLUMNS {
                            row.labeled_cell(1, &format!("Tile {index}.{column}"), |ui| {
                                if index == 1000 && column == 0 {
                                    self.content_id.set(Some(ui.scope_id()));
                                }
                            });
                        }
                        let overflow = row.cell(1, |_| ());
                        self.overflow_fits.set(Some(overflow.is_some()));
                    });
                for tile in &shown.tiles {
                    if tile.response.clicked() {
                        self.clicked.set(Some((tile.row, tile.tile)));
                    }
                }
                *self.visible.borrow_mut() = shown.visible.clone();
                *self.reported.borrow_mut() = shown.tiles.iter().map(|t| (t.row, t.tile)).collect();
                self.height.set(shown.response.rect.height());
            });
        self.offset.set(out.state.clamped_offset().y);
        self.view.set(Some(out.inner_rect));
        // The start offset only applies once.
        self.start_offset.set(out.state.clamped_offset().y);
    }
}

fn grid_harness(grid: &Grid) -> Harness<'_> {
    support::harness_for(light(), Vec2::new(420.0, 300.0), move |ui| grid.draw(ui))
}

fn is_focused(harness: &Harness<'_>, label: &str) -> Result<bool, String> {
    Ok(harness
        .query_by_label(label)
        .ok_or_else(|| format!("{label} is built"))?
        .is_focused())
}

#[test]
fn virtual_grid_builds_only_the_rows_in_view() -> TestResult {
    let grid = Grid::default();
    let harness = grid_harness(&grid);
    assert_eq!(
        grid.height.get(),
        ROWS as f32 * (ROW_HEIGHT + GUTTER) - GUTTER,
        "the grid is as tall as all its rows and the gutters between them"
    );
    // 200 / 60 = 3.3: rows 0..=3 are (partly) on screen.
    assert_eq!(*grid.visible.borrow(), 0..4, "rows 0 to 3 are visible");
    assert_eq!(
        *grid.built.borrow(),
        vec![0, 1, 2, 3],
        "only the visible rows are built, not {ROWS}"
    );
    assert_eq!(
        grid.reported.borrow().len(),
        4 * COLUMNS,
        "every built tile has a response"
    );
    assert_eq!(
        grid.overflow_fits.get(),
        Some(false),
        "a cell past the last column is not shown"
    );
    let tile = harness
        .query_by_label("Tile 2.3")
        .ok_or("a visible tile is an AccessKit node")?;
    let node = tile.accesskit_node();
    assert_eq!(node.role(), Role::GridCell, "a tile is a grid cell");
    assert_eq!(node.data().row_index(), Some(2), "the tile knows its row");
    assert_eq!(node.data().column_index(), Some(3), "and its column");
    assert!(
        (tile.rect().height() - ROW_HEIGHT).abs() < 0.01,
        "tiles are exactly the row height"
    );
    let grid_node = harness
        .query_by_role(Role::Grid)
        .ok_or("the grid is an AccessKit grid")?;
    assert_eq!(
        grid_node.accesskit_node().data().row_count(),
        Some(ROWS),
        "the grid reports every row, built or not"
    );
    assert_eq!(
        grid_node.accesskit_node().data().column_count(),
        Some(COLUMNS),
        "and its columns"
    );
    assert!(
        harness.query_by_label("Tile 50.0").is_none(),
        "a tile out of view has no node"
    );
    Ok(())
}

#[test]
fn virtual_grid_follows_the_scroll_offset_with_stable_ids() -> TestResult {
    let grid = Grid::default();
    grid.start_offset.set(1000.0 * (ROW_HEIGHT + GUTTER));
    let mut harness = grid_harness(&grid);
    assert_eq!(
        *grid.visible.borrow(),
        1000..1004,
        "scrolled 1000 rows down, rows 1000 to 1003 show"
    );
    let tile = harness
        .query_by_label("Tile 1000.0")
        .ok_or("the first tile in view has a node")?;
    assert!(
        grid.view
            .get()
            .is_some_and(|view| (tile.rect().top() - view.top()).abs() < 0.5),
        "the first row in view sits at the top of the scroll area"
    );
    let first = grid.content_id.get().ok_or("row 1000 was built")?;
    grid.start_offset.set(999.0 * (ROW_HEIGHT + GUTTER));
    harness.run();
    assert!(
        grid.visible.borrow().contains(&999),
        "the grid scrolled up a row"
    );
    assert_eq!(
        grid.content_id.get(),
        Some(first),
        "a tile's content keeps its id while the grid scrolls"
    );
    Ok(())
}

#[test]
fn visible_row_range_counts_margin_and_gutter() {
    let grid = MaterialLayoutGrid::new().margin(20.0).gutter(10.0);
    assert_eq!(
        grid.visible_row_range(egui::Rangef::new(0.0, 15.0), 100, 50.0),
        0..1,
        "the top margin shows row 0 only"
    );
    assert_eq!(
        grid.visible_row_range(egui::Rangef::new(140.0, 150.0), 100, 50.0),
        2..3,
        "60 dp stride after the 20 dp margin"
    );
    assert_eq!(
        grid.visible_row_range(egui::Rangef::new(-50.0, 1e9), 5, 50.0),
        0..5,
        "the range clamps to the row count"
    );
    assert_eq!(
        grid.visible_row_range(egui::Rangef::new(10.0, 10.0), 5, 50.0),
        0..0,
        "an empty viewport shows no rows"
    );
}

#[test]
fn virtual_grid_tiles_report_clicks() -> TestResult {
    let grid = Grid::default();
    grid.clickable.set(true);
    let mut harness = grid_harness(&grid);
    harness
        .query_by_label("Tile 1.2")
        .ok_or("tile 1.2 is shown")?
        .click();
    harness.run();
    assert_eq!(grid.clicked.get(), Some((1, 2)), "a click reports its tile");
    let node = harness
        .query_by_label("Tile 1.2")
        .ok_or("tile 1.2 is shown")?;
    assert!(
        node.accesskit_node().data().supports_action(Action::Click),
        "a clickable tile offers the click action"
    );
    Ok(())
}

#[test]
fn hover_tiles_are_not_focusable() -> TestResult {
    let grid = Grid::default();
    let mut harness = grid_harness(&grid);
    harness.key_press(Key::Tab);
    harness.run();
    assert!(
        !is_focused(&harness, "Tile 0.0")?,
        "a hover-only tile is not a Tab stop"
    );
    Ok(())
}

#[test]
fn virtual_grid_roves_with_the_arrow_keys_and_scrolls_with_focus() -> TestResult {
    let grid = Grid::default();
    grid.clickable.set(true);
    let mut harness = grid_harness(&grid);
    harness.key_press(Key::Tab);
    harness.run();
    assert!(
        is_focused(&harness, "Tile 0.0")?,
        "Tab focuses the first tile"
    );
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert!(
        is_focused(&harness, "Tile 0.1")?,
        "Right moves along the row"
    );
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert!(
        is_focused(&harness, "Tile 1.1")?,
        "Down keeps the tile's place in the row"
    );
    harness.key_press(Key::ArrowLeft);
    harness.run();
    harness.key_press(Key::ArrowLeft);
    harness.run();
    assert!(
        is_focused(&harness, "Tile 0.3")?,
        "Left from a row's first tile goes to the previous row's last"
    );
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert!(
        is_focused(&harness, "Tile 1.0")?,
        "Right from a row's last tile goes to the next row's first"
    );
    for _ in 0..8 {
        harness.key_press(Key::ArrowDown);
        harness.run();
    }
    let focused = harness
        .query_by_label("Tile 9.0")
        .ok_or("tile 9.0 is built")?;
    assert!(
        focused.is_focused(),
        "focus reaches a row below the viewport"
    );
    assert!(
        grid.offset.get() > 0.0,
        "the grid scrolled to the focused row"
    );
    let view = grid.view.get().ok_or("the scroll area was shown")?;
    assert!(
        view.contains_rect(focused.rect().shrink(0.5)),
        "the focused tile is in view: {:?} in {view:?}",
        focused.rect()
    );
    Ok(())
}

#[test]
fn virtual_grid_keeps_focus_on_a_tile_scrolled_out_of_view() -> TestResult {
    let grid = Grid::default();
    grid.clickable.set(true);
    let mut harness = grid_harness(&grid);
    harness.key_press(Key::Tab);
    harness.run();
    grid.start_offset.set(5000.0 * (ROW_HEIGHT + GUTTER));
    harness.run();
    harness.run();
    assert!(
        !grid.visible.borrow().contains(&0),
        "row 0 is out of the viewport"
    );
    assert!(
        grid.reported.borrow().contains(&(0, 0)),
        "the focused tile's row is still built"
    );
    assert!(
        is_focused(&harness, "Tile 0.0")?,
        "and the tile keeps focus"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// MaterialList::row_sense_exact

/// A status list whose rows sense hover only, recording the first row.
#[derive(Default)]
struct Status {
    clicks: Cell<usize>,
    hovered: Cell<bool>,
    senses_click: Cell<bool>,
}

impl Status {
    fn draw(&self, ui: &mut egui::Ui) {
        let shown = MaterialList::new()
            .id(Id::unique("status"))
            .fill_width(true)
            .row_sense_exact(Sense::hover())
            .item(ListItem::new("Indexing").on_click(|| self.clicks.set(self.clicks.get() + 1)))
            .item(ListItem::new("Captions"))
            .show(ui);
        if let Some(row) = shown.item(0) {
            self.hovered.set(row.hovered());
            self.senses_click.set(row.sense.senses_click());
        }
    }
}

#[test]
fn hover_only_rows_show_hover_but_cannot_be_clicked_or_focused() -> TestResult {
    let status = Status::default();
    let mut harness = support::harness_for(light(), Vec2::new(400.0, 200.0), |ui| status.draw(ui));
    assert!(!status.senses_click.get(), "the row does not sense clicks");
    let row = harness
        .query_by_label("Indexing")
        .ok_or("the row is shown")?;
    let node = row.accesskit_node();
    assert_eq!(node.role(), Role::ListItem, "the row is a plain list item");
    assert!(
        !node.data().supports_action(Action::Click),
        "the row offers no click action"
    );
    assert!(
        !node.data().supports_action(Action::Focus),
        "the row is not focusable"
    );
    let center = row.rect().center();
    harness.hover_at(center);
    harness.run();
    assert!(status.hovered.get(), "the row still reports hover");
    harness.query_by_label("Indexing").ok_or("the row")?.click();
    harness.run();
    assert_eq!(status.clicks.get(), 0, "a click does not run on_click");
    harness.key_press(Key::Tab);
    harness.run();
    assert!(
        !harness
            .query_by_label("Indexing")
            .ok_or("the row")?
            .is_focused(),
        "Tab does not stop on the row"
    );
    Ok(())
}

#[test]
fn row_sense_still_adds_to_clicks() {
    let senses = Cell::new((false, false));
    let _harness = support::harness_for(light(), Vec2::new(400.0, 200.0), |ui| {
        let shown = MaterialList::new()
            .row_sense(Sense::drag())
            .item(ListItem::new("Clip"))
            .show(ui);
        if let Some(row) = shown.item(0) {
            senses.set((row.sense.senses_click(), row.sense.senses_drag()));
        }
    });
    assert_eq!(senses.get(), (true, true), "row_sense adds drag to click");
}

// ---------------------------------------------------------------------------
// ListItem::corner_radius and MaterialList::row_gap

fn frame_shapes(app: &mut dyn FnMut(&mut egui::Ui)) -> Vec<ClippedShape> {
    themed_frame_shapes(light(), app)
}

fn themed_frame_shapes(
    theme: MaterialTheme,
    app: &mut dyn FnMut(&mut egui::Ui),
) -> Vec<ClippedShape> {
    let ctx = egui::Context::default();
    MaterialTheme::set(&ctx, theme);
    let input = || egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 400.0))),
        ..Default::default()
    };
    // The first frame only loads the fonts.
    ctx.run_ui(input(), |ui| app(ui)).textures_delta.clear();
    let mut output = ctx.run_ui(input(), |ui| app(ui));
    output.textures_delta.clear();
    output.shapes
}

/// Every filled rect shape, flattened.
fn rects(shapes: &[ClippedShape]) -> Vec<egui::epaint::RectShape> {
    fn collect(shape: &PaintShape, out: &mut Vec<egui::epaint::RectShape>) {
        match shape {
            PaintShape::Rect(r) => out.push(r.clone()),
            PaintShape::Vec(shapes) => shapes.iter().for_each(|s| collect(s, out)),
            PaintShape::Noop
            | PaintShape::Circle(_)
            | PaintShape::Ellipse(_)
            | PaintShape::LineSegment { .. }
            | PaintShape::Path(_)
            | PaintShape::Band(_)
            | PaintShape::Text(_)
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

fn selected_container_corners(
    theme: &MaterialTheme,
    item: impl Fn() -> ListItem<'static>,
) -> Option<CornerRadius> {
    let container = theme.color(ColorRole::SecondaryContainer);
    let shapes = themed_frame_shapes(theme.clone(), &mut |ui| {
        ui.add(MaterialList::new().item(item()));
    });
    rects(&shapes)
        .into_iter()
        .find(|r| r.fill == container)
        .map(|r| r.corner_radius)
}

#[test]
fn corner_radius_overrides_the_selected_shape() -> TestResult {
    for (name, theme) in support::themes() {
        let default = selected_container_corners(&theme, || ListItem::new("Clip").selected(true))
            .ok_or("the selected item paints its container")?;
        assert_eq!(
            default,
            CornerRadius::same(16),
            "{name}: selected items default to CornerLarge"
        );
        let custom = selected_container_corners(&theme, || {
            ListItem::new("Clip").selected(true).corner_radius(6.0)
        })
        .ok_or("the selected item paints its container")?;
        assert_eq!(
            custom,
            CornerRadius::same(6),
            "{name}: corner_radius replaces it"
        );
    }
    Ok(())
}

#[test]
fn corner_radius_shapes_an_unselected_tile_colour() -> TestResult {
    let tile = egui::Color32::from_rgb(1, 2, 3);
    let shapes = frame_shapes(&mut |ui| {
        ui.add(
            MaterialList::new().item(ListItem::new("Clip").tile_color(tile).corner_radius(10.0)),
        );
    });
    let rect = rects(&shapes)
        .into_iter()
        .find(|r| r.fill == tile)
        .ok_or("the tile colour is painted")?;
    assert_eq!(
        rect.corner_radius,
        CornerRadius::same(10),
        "the unselected container takes the corners too"
    );
    Ok(())
}

/// The rects of rows "A", "B" and "C" of a list built by `list`.
fn row_rects(list: impl Fn() -> MaterialList<'static>) -> Vec<Rect> {
    let out = RefCell::new(Vec::new());
    let harness = support::harness_for(light(), Vec2::new(400.0, 400.0), |ui| {
        let shown = list()
            .fill_width(true)
            .item(ListItem::new("A"))
            .item(ListItem::new("B"))
            .item(ListItem::new("C"))
            .show(ui);
        *out.borrow_mut() = shown.items.iter().map(|r| r.rect).collect();
    });
    drop(harness);
    out.into_inner()
}

fn gaps(rows: &[Rect]) -> Vec<f32> {
    rows.windows(2)
        .filter_map(|pair| match pair {
            [a, b] => Some(b.top() - a.bottom()),
            _ => None,
        })
        .collect()
}

#[test]
fn row_gap_separates_rows_of_plain_lists() {
    assert_eq!(
        gaps(&row_rects(|| MaterialList::new().dividers(false))),
        vec![0.0, 0.0],
        "plain rows touch by default"
    );
    assert_eq!(
        gaps(&row_rects(|| MaterialList::new()
            .dividers(false)
            .row_gap(6.0))),
        vec![6.0, 6.0],
        "row_gap puts space between rows"
    );
    assert_eq!(
        gaps(&row_rects(|| MaterialList::new().row_gap(0.0))),
        vec![1.0, 1.0],
        "with dividers the gap is at least the divider"
    );
    assert_eq!(
        gaps(&row_rects(|| MaterialList::new().row_gap(9.0))),
        vec![9.0, 9.0],
        "a wider gap holds the divider"
    );
    assert_eq!(
        gaps(&row_rects(|| MaterialList::new()
            .segmented(true)
            .row_gap(9.0))),
        vec![2.0, 2.0],
        "segmented lists keep their 2 dp gap"
    );
    assert_eq!(
        gaps(&row_rects(|| MaterialList::new()
            .dividers(false)
            .row_gap(f32::NAN))),
        vec![0.0, 0.0],
        "a gap that is not a number is none"
    );
}

#[test]
fn row_gap_counts_in_the_virtual_row_range() {
    let list = MaterialList::new().dividers(false).row_gap(4.0);
    assert_eq!(
        list.visible_row_range(egui::Rangef::new(320.0, 330.0), 100, 28.0),
        10..11,
        "32 dp stride with a 4 dp gap"
    );
}

#[test]
fn the_divider_sits_in_the_middle_of_the_gap() -> TestResult {
    let divider = light().color(ColorRole::OutlineVariant);
    let first_row = Cell::new(None);
    let shapes = frame_shapes(&mut |ui| {
        let shown = MaterialList::new()
            .fill_width(true)
            .row_gap(9.0)
            .item(ListItem::new("A"))
            .item(ListItem::new("B"))
            .item(ListItem::new("C"))
            .show(ui);
        first_row.set(shown.item(0).map(|r| r.rect));
    });
    let first = first_row.get().ok_or("row A")?;
    let line = rects(&shapes)
        .into_iter()
        .find(|r| r.fill == divider)
        .ok_or("a divider is painted")?;
    let offset = line.rect.top() - first.bottom();
    assert!(
        (offset - 4.0).abs() < 0.01,
        "the 1 dp divider is 4 dp below the row, in a 9 dp gap: {offset}"
    );
    Ok(())
}
