// ============================================================================
// Module:       layoutgrid
// Description:  Material Design 3 Layout Grid Components.
//
// Dependencies: egui; crate::theme
// ============================================================================

//! Material Design 3 Layout Grid Components
//!
//! # M3 Color Role Usage
//!
//! Colours come from the context's [`MaterialTheme`]:
//!
//! - **primary**: Cell fill and grid outline (debug mode)
//! - **outlineVariant**: Column guides (debug mode)
//!
//! [`GridTileBar`] text is white whatever the theme: it sits on its own
//! translucent bar over arbitrary image content, not on a theme surface.
//!
//! [`MaterialLayoutGrid::show_rows`] is the virtualised form: uniform rows
//! of tiles inside `ScrollArea::show_viewport`, building only the rows in
//! view (plus the row holding keyboard focus and the row focus moves to).

use std::ops::Range;

use crate::tokens::{ColorRole, Shape, TypeRole};
use crate::MaterialTheme;
use egui::{
    epaint::CornerRadius, Color32, EventFilter, Id, InnerResponse, Key, Rangef, Rect, Response,
    Role, Sense, Ui, Vec2, Widget,
};

/// Material Design layout grid component.
///
/// Layout grids provide structure and organize content across multiple screen sizes.
/// They help create consistent layouts following Material Design principles.
///
/// ```
/// # use egui_material3::*;
/// # egui::__run_test_ui(|ui| {
/// let grid = MaterialLayoutGrid::new()
///     .columns(12)
///     .gutter(16.0)
///     .margin(24.0)
///     .cell(4, |ui| { ui.label("Column 1-4"); })
///     .cell(4, |ui| { ui.label("Column 5-8"); })
///     .cell(4, |ui| { ui.label("Column 9-12"); });
///
/// ui.add(grid);
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct MaterialLayoutGrid<'a> {
    cells: Vec<GridCell<'a>>,
    columns: usize,
    gutter: f32,
    margin: f32,
    max_width: Option<f32>,
    debug_mode: bool,
    id: Option<Id>,
    tile_sense: Sense,
}

struct GridCell<'a> {
    span: usize,
    content: Box<dyn FnOnce(&mut Ui) -> Response + 'a>,
    offset: Option<usize>,
}

impl<'a> MaterialLayoutGrid<'a> {
    /// Create a new layout grid.
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            columns: 12,  // Standard 12-column grid
            gutter: 16.0, // Standard gutter size
            margin: 24.0, // Standard margin
            max_width: None,
            debug_mode: false,
            id: None,
            tile_sense: Sense::hover(),
        }
    }

    /// A stable id for the grid, which [`show_rows`](Self::show_rows)
    /// derives its tile ids from. Without one, the id comes from the
    /// parent `Ui`.
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// What every tile of [`show_rows`](Self::show_rows) senses: hover by
    /// default (widgets inside the tile do their own interaction, as in the
    /// eager grid), or e.g. `Sense::click()` to make whole tiles clickable,
    /// Tab stops and arrow-key roving targets.
    pub fn tile_sense(mut self, sense: Sense) -> Self {
        self.tile_sense = sense;
        self
    }

    /// Set the number of columns.
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }

    /// Set the gutter size (space between columns).
    pub fn gutter(mut self, gutter: f32) -> Self {
        self.gutter = gutter;
        self
    }

    /// Set the margin (space around the grid).
    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    /// Set maximum width of the grid.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// Enable debug mode to visualize grid structure.
    pub fn debug_mode(mut self, debug: bool) -> Self {
        self.debug_mode = debug;
        self
    }

    /// Add a cell that spans the specified number of columns.
    pub fn cell<F>(mut self, span: usize, content: F) -> Self
    where
        F: FnOnce(&mut Ui) + 'a,
    {
        self.cells.push(GridCell {
            span: span.clamp(1, self.columns),
            content: Box::new(move |ui| {
                content(ui);
                ui.allocate_response(Vec2::ZERO, Sense::hover())
            }),
            offset: None,
        });
        self
    }

    /// Add a cell with an offset (empty columns before this cell).
    pub fn cell_with_offset<F>(mut self, span: usize, offset: usize, content: F) -> Self
    where
        F: FnOnce(&mut Ui) + 'a,
    {
        self.cells.push(GridCell {
            span: span.clamp(1, self.columns),
            content: Box::new(move |ui| {
                content(ui);
                ui.allocate_response(Vec2::ZERO, Sense::hover())
            }),
            offset: Some(offset),
        });
        self
    }

    /// Add an empty cell (spacer).
    pub fn spacer(mut self, span: usize) -> Self {
        self.cells.push(GridCell {
            span: span.clamp(1, self.columns),
            content: Box::new(|ui| ui.allocate_response(Vec2::ZERO, Sense::hover())),
            offset: None,
        });
        self
    }

    fn calculate_column_width(&self, available_width: f32) -> f32 {
        let effective_width = if let Some(max_width) = self.max_width {
            available_width.min(max_width)
        } else {
            available_width
        };

        let total_gutter_width = (self.columns - 1) as f32 * self.gutter;
        let content_width = effective_width - 2.0 * self.margin - total_gutter_width;
        content_width / self.columns as f32
    }

    /// Space between rows of [`show_rows`](Self::show_rows): the gutter,
    /// never negative.
    fn row_gutter(&self) -> f32 {
        if self.gutter.is_finite() {
            self.gutter.max(0.0)
        } else {
            0.0
        }
    }

    /// The margin, or none when it is not finite.
    fn finite_margin(&self) -> f32 {
        if self.margin.is_finite() {
            self.margin
        } else {
            0.0
        }
    }

    /// The rows of a [`show_rows`](Self::show_rows) grid that fall inside
    /// `viewport`: the vertical span shown, in points from the grid's top
    /// (its margin included). Rows are `row_height` tall and one gutter
    /// apart.
    ///
    /// ```rust
    /// use egui_material3::MaterialLayoutGrid;
    ///
    /// let grid = MaterialLayoutGrid::new().margin(0.0).gutter(8.0);
    /// // 100 + 8 = 108 dp per row.
    /// let rows = grid.visible_row_range(egui::Rangef::new(216.0, 400.0), 1_000, 100.0);
    /// assert_eq!(rows, 2..4);
    /// ```
    pub fn visible_row_range(
        &self,
        viewport: Rangef,
        row_count: usize,
        row_height: f32,
    ) -> Range<usize> {
        let stride = sanitize_row_height(row_height) + self.row_gutter();
        let margin = self.finite_margin();
        let row_at = |y: f32| {
            let y = y - margin;
            if y.is_finite() {
                // Truncation is the point; negative and huge values clamp.
                ((y / stride).max(0.0) as usize).min(row_count)
            } else if y > 0.0 {
                row_count
            } else {
                0
            }
        };
        let first = row_at(viewport.min);
        if viewport.max <= viewport.min {
            return first..first;
        }
        let end = row_at(viewport.max).saturating_add(1).min(row_count);
        first..end.max(first)
    }

    /// Shows `row_count` rows of tiles, each row `row_height` points tall,
    /// but builds only the rows inside `viewport`: pass the `ui` and
    /// viewport `ScrollArea::show_viewport` gives its closure.
    /// `add_row(row, index)` adds row `index`'s tiles with
    /// [`GridRow::cell`] and friends; it is called for the visible rows
    /// only (plus the row holding keyboard focus and the row focus moves
    /// to), so tile contents come from the caller's data each frame.
    ///
    /// Columns, gutter (between columns and between rows), margin and
    /// `max_width` work as in the eager grid; cells added with
    /// [`cell`](Self::cell) are not shown. Every tile gets a response with
    /// a stable id (from [`id`](Self::id), the row and the tile), sensing
    /// [`tile_sense`](Self::tile_sense), and an AccessKit `GridCell` node
    /// with its row and column index inside a `Grid` node that reports the
    /// full `row_count`. Widgets inside a tile get ids that stay put while
    /// the grid scrolls. When tiles are focusable, Tab reaches them and the
    /// arrow keys move between tiles (Left/Right in reading order, Up/Down
    /// to the same tile of the neighbouring row), scrolling to the row
    /// focus moves to; a focused tile scrolled out of view stays built.
    ///
    /// ```rust
    /// # egui::__run_test_ui(|ui| {
    /// use egui_material3::MaterialLayoutGrid;
    ///
    /// let mut opened = None;
    /// egui::ScrollArea::vertical().max_height(300.0).show_viewport(ui, |ui, viewport| {
    ///     let shown = MaterialLayoutGrid::new()
    ///         .id(egui::Id::new("thumbnails"))
    ///         .columns(4)
    ///         .tile_sense(egui::Sense::click())
    ///         .show_rows(ui, viewport, 25_000, 96.0, |row, index| {
    ///             for column in 0..4 {
    ///                 let clip = index * 4 + column;
    ///                 row.labeled_cell(1, &format!("clip {clip}"), |ui| {
    ///                     ui.label(format!("clip {clip}"));
    ///                 });
    ///             }
    ///         });
    ///     for tile in &shown.tiles {
    ///         if tile.response.double_clicked() {
    ///             opened = Some(tile.row * 4 + tile.tile);
    ///         }
    ///     }
    /// });
    /// # let _ = opened;
    /// # });
    /// ```
    pub fn show_rows(
        self,
        ui: &mut Ui,
        viewport: Rect,
        row_count: usize,
        row_height: f32,
        mut add_row: impl FnMut(&mut GridRow<'_>, usize),
    ) -> GridRowsResponse {
        let theme = MaterialTheme::of(ui.ctx());
        let row_height = sanitize_row_height(row_height);
        let row_gutter = self.row_gutter();
        let stride = row_height + row_gutter;
        let margin = self.finite_margin();
        let grid_id = self
            .id
            .unwrap_or_else(|| ui.scope_id().with("material_layout_grid"));

        // The viewport is in the scroll content's coordinates, whose origin
        // is the top of the content ui; the grid may start below it.
        let origin = ui.max_rect().min;
        let grid_top = ui.cursor().min.y - origin.y;
        let visible = self.visible_row_range(
            Rangef::new(viewport.min.y - grid_top, viewport.max.y - grid_top),
            row_count,
            row_height,
        );

        let available_width = ui.available_width();
        let effective_width = match self.max_width {
            Some(max_width) => available_width.min(max_width),
            None => available_width,
        };
        let column_width = self.calculate_column_width(available_width);
        let rows_height = if row_count == 0 {
            0.0
        } else {
            row_count as f32 * stride - row_gutter
        };
        let total_height = rows_height + 2.0 * margin;
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(effective_width, total_height), Sense::hover());
        let response = ui.interact(rect, grid_id, Sense::hover());
        crate::a11y::describe(&response, Role::Grid, true, "", None);
        let columns = self.columns;
        ui.ctx().accesskit_node_builder(grid_id, |node| {
            node.set_row_count(row_count);
            node.set_column_count(columns);
        });

        // Keyboard roving between focusable tiles. The focused tile is
        // remembered by (row, tile, tiles in its row), so it is found
        // without walking every row and is always built: egui drops focus
        // from a widget that is not shown.
        let focusable = self.tile_sense.is_focusable();
        let focus_key = grid_id.with("focused_tile");
        let focused = if focusable {
            ui.data(|d| d.get_temp::<(usize, usize, usize)>(focus_key))
                .filter(|&(row, tile, _)| {
                    row < row_count && ui.memory(|m| m.has_focus(tile_id(grid_id, row, tile)))
                })
        } else {
            None
        };
        let target = focused.and_then(|focus| rove(ui, grid_id, focus, row_count));

        let mut indices: Vec<usize> = visible.clone().collect();
        indices.extend(focused.map(|(row, _, _)| row));
        indices.extend(target.map(|(row, _)| row));
        indices.sort_unstable();
        indices.dedup();

        let mut tiles = Vec::new();
        for index in indices {
            let row_rect = Rect::from_min_size(
                egui::pos2(rect.min.x, rect.min.y + margin + index as f32 * stride),
                Vec2::new(effective_width, row_height),
            );
            let first_tile = tiles.len();
            let mut row = GridRow {
                ui: &mut *ui,
                theme: &theme,
                layout: RowLayout {
                    grid_id,
                    row: index,
                    top: row_rect.min.y,
                    left: rect.min.x + margin,
                    height: row_height,
                    column_width,
                    gutter: self.gutter,
                    columns,
                    sense: self.tile_sense,
                    debug: self.debug_mode,
                },
                column: 0,
                tiles: &mut tiles,
            };
            add_row(&mut row, index);
            let built = tiles.len().saturating_sub(first_tile);
            if let Some((_, wanted)) = target.filter(|&(row, _)| row == index) {
                if let Some(last) = built.checked_sub(1) {
                    let tile = wanted.min(last);
                    ui.memory_mut(|m| m.request_focus(tile_id(grid_id, index, tile)));
                }
                ui.scroll_to_rect(row_rect, None);
            }
            if focusable {
                for tile in 0..built {
                    if ui.memory(|m| m.has_focus(tile_id(grid_id, index, tile))) {
                        ui.data_mut(|d| d.insert_temp(focus_key, (index, tile, built)));
                    }
                }
            }
        }

        GridRowsResponse {
            response,
            visible,
            tiles,
        }
    }
}

impl<'a> Default for MaterialLayoutGrid<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MaterialLayoutGrid<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let m3 = MaterialTheme::of(ui.ctx());
        let available_width = ui.available_width();
        let column_width = self.calculate_column_width(available_width);

        let MaterialLayoutGrid {
            cells,
            columns,
            gutter,
            margin,
            max_width,
            debug_mode,
            id: _,
            tile_sense: _,
        } = self;

        if cells.is_empty() {
            return ui.allocate_response(Vec2::ZERO, Sense::hover());
        }

        let effective_width = if let Some(max_width) = max_width {
            available_width.min(max_width)
        } else {
            available_width
        };

        // Start layout
        let start_pos = ui.next_widget_position();
        let mut current_row_y = start_pos.y + margin;
        let mut current_column = 0;
        let mut row_height: f32 = 0.0;
        let mut max_y = current_row_y;

        let mut responses = Vec::new();

        // Process each cell
        for cell in cells {
            // Handle offset
            if let Some(offset) = cell.offset {
                current_column += offset;
            }

            // Check if we need to wrap to next row
            if current_column + cell.span > columns {
                current_row_y = max_y + gutter;
                current_column = 0;
                row_height = 0.0;
            }

            // Calculate cell position and size
            let cell_x = start_pos.x + margin + current_column as f32 * (column_width + gutter);
            let cell_width = cell.span as f32 * column_width + (cell.span - 1) as f32 * gutter;

            // Create a constrained UI for this cell
            let cell_rect = Rect::from_min_size(
                egui::pos2(cell_x, current_row_y),
                Vec2::new(cell_width, ui.available_height()),
            );

            let cell_response =
                ui.scope_builder(egui::UiBuilder::new().max_rect(cell_rect), |ui| {
                    // Debug visualization
                    if debug_mode {
                        let debug_color = m3.color(ColorRole::Primary).gamma_multiply(0.12);
                        ui.painter().rect_filled(
                            cell_rect,
                            Shape::EXTRA_SMALL.corner_radius(cell_rect, m3.layout_direction),
                            debug_color,
                        );
                    }

                    (cell.content)(ui)
                });

            let cell_height = cell_response.response.rect.height();
            row_height = row_height.max(cell_height);
            max_y = max_y.max(current_row_y + row_height);

            responses.push(cell_response.response);
            current_column += cell.span;
        }

        // Calculate total grid size
        let total_height = max_y - start_pos.y + margin;
        let grid_rect = Rect::from_min_size(start_pos, Vec2::new(effective_width, total_height));

        // Debug: Draw grid outline
        if debug_mode {
            let outline_color = m3.color(ColorRole::Primary);
            ui.painter().rect_stroke(
                grid_rect,
                Shape::EXTRA_SMALL.corner_radius(grid_rect, m3.layout_direction),
                egui::epaint::Stroke::new(1.0, outline_color),
                egui::epaint::StrokeKind::Inside,
            );

            // Draw column guides
            for i in 0..=columns {
                let x = start_pos.x + margin + i as f32 * (column_width + gutter) - gutter / 2.0;
                if i > 0 && i < columns {
                    ui.painter().line_segment(
                        [egui::pos2(x, start_pos.y + margin), egui::pos2(x, max_y)],
                        egui::epaint::Stroke::new(0.5, m3.color(ColorRole::OutlineVariant)),
                    );
                }
            }
        }

        // Allocate the full grid space
        let (_grid_response_rect, mut grid_response) =
            ui.allocate_at_least(Vec2::new(effective_width, total_height), Sense::hover());

        // Union all cell responses
        for response in responses {
            grid_response = grid_response.union(response);
        }

        grid_response
    }
}

/// A usable row height: finite and at least one point (else 100 dp, the
/// [`GridTile`] default height).
fn sanitize_row_height(height: f32) -> f32 {
    if height.is_finite() {
        height.max(1.0)
    } else {
        100.0
    }
}

/// The id of tile `tile` of row `row` in the grid `grid_id`.
fn tile_id(grid_id: Id, row: usize, tile: usize) -> Id {
    grid_id.with(("tile", row, tile))
}

/// The arrow keys move focus from the focused tile `(row, tile, tiles in
/// its row)`; returns the `(row, tile)` focus should move to, where
/// `usize::MAX` means the row's last tile.
fn rove(
    ui: &Ui,
    grid_id: Id,
    (row, tile, tiles): (usize, usize, usize),
    row_count: usize,
) -> Option<(usize, usize)> {
    ui.memory_mut(|m| {
        m.set_focus_lock_filter(
            tile_id(grid_id, row, tile),
            EventFilter {
                horizontal_arrows: true,
                vertical_arrows: true,
                ..Default::default()
            },
        );
    });
    let none = egui::Modifiers::NONE;
    let (left, right, up, down) = ui.input_mut(|i| {
        (
            i.consume_key(none, Key::ArrowLeft),
            i.consume_key(none, Key::ArrowRight),
            i.consume_key(none, Key::ArrowUp),
            i.consume_key(none, Key::ArrowDown),
        )
    });
    let has_next_row = row.saturating_add(1) < row_count;
    if right {
        if tile.saturating_add(1) < tiles {
            Some((row, tile + 1))
        } else {
            has_next_row.then(|| (row + 1, 0))
        }
    } else if left {
        if tile > 0 {
            Some((row, tile - 1))
        } else {
            row.checked_sub(1).map(|previous| (previous, usize::MAX))
        }
    } else if down {
        has_next_row.then(|| (row + 1, tile))
    } else if up {
        row.checked_sub(1).map(|previous| (previous, tile))
    } else {
        None
    }
}

/// Where one row of a [`MaterialLayoutGrid::show_rows`] grid goes.
#[derive(Clone, Copy)]
struct RowLayout {
    grid_id: Id,
    row: usize,
    top: f32,
    left: f32,
    height: f32,
    column_width: f32,
    gutter: f32,
    columns: usize,
    sense: Sense,
    debug: bool,
}

/// One row of a [`MaterialLayoutGrid::show_rows`] grid, which the row
/// closure adds tiles to, left to right, as with the eager grid's
/// [`cell`](MaterialLayoutGrid::cell), [`cell_with_offset`](MaterialLayoutGrid::cell_with_offset)
/// and [`spacer`](MaterialLayoutGrid::spacer). Rows do not wrap: a cell
/// that does not fit in the columns left is not shown.
pub struct GridRow<'g> {
    ui: &'g mut Ui,
    theme: &'g MaterialTheme,
    layout: RowLayout,
    column: usize,
    tiles: &'g mut Vec<GridTileResponse>,
}

impl GridRow<'_> {
    /// The row's index.
    pub fn index(&self) -> usize {
        self.layout.row
    }

    /// The first column not yet taken.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Adds a tile spanning `span` columns; `content` fills it. Returns the
    /// content's value and the tile's response, or `None` when the tile
    /// does not fit in the columns left.
    pub fn cell<R>(
        &mut self,
        span: usize,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        self.add(span, 0, None, content)
    }

    /// [`cell`](Self::cell) after `offset` empty columns.
    pub fn cell_with_offset<R>(
        &mut self,
        span: usize,
        offset: usize,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        self.add(span, offset, None, content)
    }

    /// [`cell`](Self::cell) whose AccessKit node is named `label`, for
    /// tiles a screen reader or test should find by name (a thumbnail with
    /// no text, say).
    pub fn labeled_cell<R>(
        &mut self,
        span: usize,
        label: &str,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        self.add(span, 0, Some(label), content)
    }

    /// Leaves `span` columns empty.
    pub fn spacer(&mut self, span: usize) {
        self.column = self.column.saturating_add(span.max(1));
    }

    fn add<R>(
        &mut self,
        span: usize,
        offset: usize,
        label: Option<&str>,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let layout = self.layout;
        let span = span.clamp(1, layout.columns);
        let start = self.column.saturating_add(offset);
        if start.saturating_add(span) > layout.columns {
            return None;
        }
        let tile = self.tiles.iter().filter(|t| t.row == layout.row).count();
        let rect = Rect::from_min_size(
            egui::pos2(
                layout.left + start as f32 * (layout.column_width + layout.gutter),
                layout.top,
            ),
            Vec2::new(
                span as f32 * layout.column_width + (span - 1) as f32 * layout.gutter,
                layout.height,
            ),
        );
        let id = tile_id(layout.grid_id, layout.row, tile);
        let response = self.ui.interact(rect, id, layout.sense);
        crate::a11y::describe(&response, Role::GridCell, true, label.unwrap_or(""), None);
        let row = layout.row;
        self.ui.ctx().accesskit_node_builder(id, |node| {
            node.set_row_index(row);
            node.set_column_index(start);
        });

        let corners = Shape::EXTRA_SMALL.corner_radius(rect, self.theme.layout_direction);
        if layout.debug {
            let debug_color = self.theme.color(ColorRole::Primary).gamma_multiply(0.12);
            self.ui.painter().rect_filled(rect, corners, debug_color);
        }
        if response.has_focus() {
            // Tiles may touch their neighbours, so the ring goes inside.
            let ring =
                rect.shrink(crate::paint::FOCUS_RING_OFFSET + crate::paint::FOCUS_RING_WIDTH);
            crate::paint::paint_focus_ring(
                self.ui.painter(),
                ring,
                corners,
                self.theme.color(ColorRole::Secondary),
            );
        }

        let mut child = self.ui.new_child(
            egui::UiBuilder::new()
                .scope_id(id.with("content"))
                .max_rect(rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        child.set_clip_rect(rect.intersect(self.ui.clip_rect()));
        let inner = content(&mut child);

        self.tiles.push(GridTileResponse {
            row,
            tile,
            column: start,
            response: response.clone(),
        });
        self.column = start + span;
        Some(InnerResponse::new(inner, response))
    }
}

/// One tile of a [`MaterialLayoutGrid::show_rows`] grid.
#[derive(Debug, Clone)]
pub struct GridTileResponse {
    /// The row the tile is in.
    pub row: usize,
    /// The tile's place in its row: 0 for the first tile added.
    pub tile: usize,
    /// The first column the tile spans.
    pub column: usize,
    /// The whole tile, sensing [`MaterialLayoutGrid::tile_sense`].
    pub response: Response,
}

/// What [`MaterialLayoutGrid::show_rows`] reports: the grid's own response,
/// the rows the viewport shows, and one response per tile built.
#[derive(Debug, Clone)]
pub struct GridRowsResponse {
    /// The whole grid, `row_count` rows tall.
    pub response: Response,
    /// The rows inside the viewport.
    pub visible: Range<usize>,
    /// Every tile built this frame, by row and then by place in the row.
    pub tiles: Vec<GridTileResponse>,
}

impl GridRowsResponse {
    /// The response of tile `tile` of row `row`, if it was built this frame.
    pub fn tile(&self, row: usize, tile: usize) -> Option<&Response> {
        self.tiles
            .iter()
            .find(|t| t.row == row && t.tile == tile)
            .map(|t| &t.response)
    }
}

/// Convenience function to create a new layout grid.
pub fn layout_grid() -> MaterialLayoutGrid<'static> {
    MaterialLayoutGrid::new()
}

/// Convenience function to create a layout grid with debug mode enabled.
pub fn debug_layout_grid() -> MaterialLayoutGrid<'static> {
    MaterialLayoutGrid::new().debug_mode(true)
}

/// Material Design grid tile bar.
///
/// A header or footer bar typically used with [GridTile].
/// Contains optional leading/trailing icons and title/subtitle text.
///
/// ```
/// # use egui_material3::*;
/// # egui::__run_test_ui(|ui| {
/// let tile_bar = GridTileBar::new()
///     .title("Image Title")
///     .subtitle("Subtitle text")
///     .background_color(egui::Color32::from_black_alpha(128));
///
/// ui.add(tile_bar);
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
#[expect(
    clippy::type_complexity,
    reason = "a boxed UI callback; the closure signature is clearer spelled out than behind an alias"
)]
pub struct GridTileBar<'a> {
    background_color: Option<Color32>,
    leading: Option<Box<dyn FnOnce(&mut Ui) + 'a>>,
    title: Option<String>,
    subtitle: Option<String>,
    trailing: Option<Box<dyn FnOnce(&mut Ui) + 'a>>,
}

impl<'a> GridTileBar<'a> {
    /// Create a new grid tile bar.
    pub fn new() -> Self {
        Self {
            background_color: None,
            leading: None,
            title: None,
            subtitle: None,
            trailing: None,
        }
    }

    /// Set the background color.
    pub fn background_color(mut self, color: Color32) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set a leading widget (left side icon/widget).
    pub fn leading<F>(mut self, content: F) -> Self
    where
        F: FnOnce(&mut Ui) + 'a,
    {
        self.leading = Some(Box::new(content));
        self
    }

    /// Set the title text.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the subtitle text.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set a trailing widget (right side icon/widget).
    pub fn trailing<F>(mut self, content: F) -> Self
    where
        F: FnOnce(&mut Ui) + 'a,
    {
        self.trailing = Some(Box::new(content));
        self
    }
}

impl<'a> Default for GridTileBar<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for GridTileBar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let GridTileBar {
            background_color,
            leading,
            title,
            subtitle,
            trailing,
        } = self;

        // Calculate height based on content
        let height = if title.is_some() && subtitle.is_some() {
            68.0
        } else {
            48.0
        };

        // Calculate padding
        let padding_start = if leading.is_some() { 8.0 } else { 16.0 };
        let padding_end = if trailing.is_some() { 8.0 } else { 16.0 };

        let available_width = ui.available_width();
        let start_pos = ui.next_widget_position();

        // Draw background if specified
        if let Some(bg_color) = background_color {
            let bg_rect = Rect::from_min_size(start_pos, Vec2::new(available_width, height));
            ui.painter()
                .rect_filled(bg_rect, CornerRadius::ZERO, bg_color);
        }

        let _response = ui.horizontal(|ui| {
            ui.add_space(padding_start);

            // Leading widget
            if let Some(leading_fn) = leading {
                leading_fn(ui);
                ui.add_space(8.0);
            }

            // Title and subtitle
            if title.is_some() || subtitle.is_some() {
                ui.vertical(|ui| {
                    ui.set_min_height(height);
                    ui.add_space((height - if subtitle.is_some() { 40.0 } else { 20.0 }) / 2.0);

                    if let Some(title_text) = &title {
                        let job = TypeRole::TitleMedium.style().layout_job(
                            ui.ctx(),
                            title_text.as_str(),
                            Color32::WHITE,
                        );
                        ui.label(job);
                    }

                    if let Some(subtitle_text) = &subtitle {
                        let job = TypeRole::BodySmall.style().layout_job(
                            ui.ctx(),
                            subtitle_text.as_str(),
                            Color32::WHITE,
                        );
                        ui.label(job);
                    }
                });
            }

            // Trailing widget
            if let Some(trailing_fn) = trailing {
                ui.add_space(8.0);
                trailing_fn(ui);
            }

            ui.add_space(padding_end);
        });

        ui.allocate_rect(
            Rect::from_min_size(start_pos, Vec2::new(available_width, height)),
            Sense::hover(),
        )
    }
}

/// Material Design grid tile.
///
/// A tile in a grid list with optional header and footer overlays.
/// Based on Flutter's GridTile component.
///
/// ```
/// # use egui_material3::*;
/// # egui::__run_test_ui(|ui| {
/// let tile = GridTile::new(|ui| {
///     ui.label("Tile content");
/// })
/// .footer(
///     GridTileBar::new()
///         .title("Image Title")
///         .background_color(egui::Color32::from_black_alpha(128))
/// );
///
/// ui.add(tile);
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct GridTile<'a> {
    header: Option<GridTileBar<'a>>,
    footer: Option<GridTileBar<'a>>,
    child: Box<dyn FnOnce(&mut Ui) + 'a>,
    min_height: f32,
}

impl<'a> GridTile<'a> {
    /// Create a new grid tile with the given content.
    pub fn new<F>(content: F) -> Self
    where
        F: FnOnce(&mut Ui) + 'a,
    {
        Self {
            header: None,
            footer: None,
            child: Box::new(content),
            min_height: 100.0,
        }
    }

    /// Set the header bar (shown at the top).
    pub fn header(mut self, header: GridTileBar<'a>) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the footer bar (shown at the bottom).
    pub fn footer(mut self, footer: GridTileBar<'a>) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Set the minimum height of the tile.
    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = height;
        self
    }
}

impl Widget for GridTile<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let GridTile {
            header,
            footer,
            child,
            min_height,
        } = self;

        if header.is_none() && footer.is_none() {
            // Simple case: no overlays
            let response = ui.vertical(|ui| {
                ui.set_min_height(min_height);
                child(ui);
            });
            return response.response;
        }

        // Complex case: with header/footer overlays
        let available_width = ui.available_width();
        let start_pos = ui.next_widget_position();

        // First, render the child to get the content height
        let child_response = ui.vertical(|ui| {
            ui.set_min_height(min_height);
            child(ui);
        });

        let content_height = child_response.response.rect.height().max(min_height);
        let tile_rect = Rect::from_min_size(start_pos, Vec2::new(available_width, content_height));

        // Draw header overlay if present
        if let Some(header_bar) = header {
            let header_ui = &mut ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(
                        start_pos,
                        Vec2::new(available_width, 68.0),
                    ))
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
            );
            header_ui.add(header_bar);
        }

        // Draw footer overlay if present
        if let Some(footer_bar) = footer {
            let footer_height = 68.0; // Will be adjusted by the GridTileBar itself
            let footer_pos = egui::pos2(start_pos.x, start_pos.y + content_height - footer_height);
            let footer_ui = &mut ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(
                        footer_pos,
                        Vec2::new(available_width, footer_height),
                    ))
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
            );
            footer_ui.add(footer_bar);
        }

        ui.allocate_rect(tile_rect, Sense::hover())
    }
}
