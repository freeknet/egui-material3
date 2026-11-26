// ============================================================================
// Module:       datatable::types
// Description:  Type definitions for Material Design 3 Data Table.
//
// Dependencies: egui
// ============================================================================

//! Type definitions for Material Design 3 Data Table

use egui::{ecolor::Color32, epaint::Stroke, FontId, WidgetText};
use std::collections::{HashMap, HashSet};

/// Theme/styling configuration for MaterialDataTable
#[derive(Clone, Debug)]
pub struct DataTableTheme {
    /// Table background color; `None` uses the theme's `surface` color.
    pub decoration: Option<Color32>,
    /// Background color of the heading row.
    pub heading_row_color: Option<Color32>,
    /// Height of the heading row, in points.
    pub heading_row_height: Option<f32>,
    /// Font size of the column headers; `None` uses the responsive default.
    pub header_font_size: Option<f32>,
    /// Font and color for the column header text.
    pub heading_text_style: Option<(FontId, Color32)>,
    /// Background color of the data rows.
    pub data_row_color: Option<Color32>,
    /// Minimum height of a data row, in points.
    pub data_row_min_height: Option<f32>,
    /// Maximum height of a data row, in points; `None` leaves it unbounded.
    pub data_row_max_height: Option<f32>,
    /// Font and color for the data cell text.
    pub data_text_style: Option<(FontId, Color32)>,
    /// Horizontal padding at the left and right edges of the table, in points.
    pub horizontal_margin: Option<f32>,
    /// Horizontal gap between columns, in points.
    pub column_spacing: Option<f32>,
    /// Thickness of the dividers between rows, in points.
    pub divider_thickness: Option<f32>,
    /// Color of the dividers between rows.
    pub divider_color: Option<Color32>,
    /// Horizontal margin around the selection checkbox, in points.
    pub checkbox_horizontal_margin: Option<f32>,
    /// Outline drawn around the table; `None` uses a 1-point stroke in the `outline` color.
    pub border_stroke: Option<Stroke>,
    /// Color of the sort arrow on the column currently sorted.
    pub sort_active_color: Option<Color32>,
    /// Color of the sort arrow on sortable columns that are not sorted.
    pub sort_inactive_color: Option<Color32>,
    /// Background color of selected rows.
    pub selected_row_color: Option<Color32>,
    /// Whether a divider is drawn below the last row.
    pub show_bottom_border: bool,
    /// Whether the selection checkbox column is shown when selection is allowed.
    pub show_checkbox_column: bool,
    /// Show a tooltip with the full text on every data cell.
    pub enable_cell_tooltips: bool,
    /// Show a cell tooltip only when the text is wider than its cell.
    pub smart_cell_tooltips: bool,
}

impl Default for DataTableTheme {
    fn default() -> Self {
        Self {
            decoration: None,
            heading_row_color: None,
            heading_row_height: Some(56.0),
            header_font_size: None,
            heading_text_style: None,
            data_row_color: None,
            data_row_min_height: Some(52.0),
            data_row_max_height: None,
            data_text_style: None,
            horizontal_margin: Some(24.0),
            column_spacing: Some(56.0),
            divider_thickness: Some(1.0),
            divider_color: None,
            checkbox_horizontal_margin: Some(16.0),
            border_stroke: None,
            sort_active_color: None,
            sort_inactive_color: None,
            selected_row_color: None,
            show_bottom_border: true,
            show_checkbox_column: true,
            enable_cell_tooltips: false,
            smart_cell_tooltips: false,
        }
    }
}

/// Column width specification
#[derive(Clone, Debug, PartialEq)]
pub enum ColumnWidth {
    /// A fixed width, in points.
    Fixed(f32),
    /// A flexible width, with this value as its weight.
    Flex(f32),
}

impl Default for ColumnWidth {
    fn default() -> Self {
        ColumnWidth::Fixed(100.0)
    }
}

/// Sort direction for columns
#[derive(Clone, Debug, PartialEq, Hash, serde::Serialize, serde::Deserialize, Default)]
pub enum SortDirection {
    /// Smallest value first.
    #[default]
    Ascending,
    /// Largest value first.
    Descending,
}

/// Display mode for datatable responsive sizing
///
/// Controls how the table scales fonts, padding, and header heights.
/// By default, tables automatically detect their width and apply appropriate styling.
/// Use Force* variants to override automatic detection.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum CompactMode {
    /// Automatic sizing based on table width (default)
    ///
    /// - Width < 400px: narrow styling (12px fonts, 32px padding, 40px header)
    /// - Width < 500px: compact styling (14px fonts, 40px padding, 48px header)
    /// - Width ≥ 500px: wide styling (16px fonts, 48px padding, 56px header)
    #[default]
    Auto,
    /// Force very narrow styling regardless of actual table width
    ///
    /// Uses 12px fonts, 32px padding, 40px header height
    ForceNarrow,
    /// Force compact styling regardless of actual table width
    ///
    /// Uses 14px fonts, 40px padding, 48px header height
    ForceCompact,
    /// Force wide styling regardless of actual table width
    ///
    /// Uses 16px fonts, 48px padding, 56px header height
    ForceWide,
}

/// Vertical alignment for cells
#[derive(Clone, Debug, PartialEq, Default)]
pub enum VAlign {
    /// Align the content to the top of the cell.
    Top,
    /// Center the content vertically.
    #[default]
    Center,
    /// Align the content to the bottom of the cell.
    Bottom,
}

/// Horizontal alignment for cells
#[derive(Clone, Debug, PartialEq, Default)]
pub enum HAlign {
    /// Align the content to the left of the cell.
    #[default]
    Left,
    /// Center the content horizontally.
    Center,
    /// Align the content to the right of the cell.
    Right,
}

/// Actions that can be performed on data table rows
#[derive(Debug, Clone)]
pub enum RowAction {
    /// The row at this index was put into edit mode.
    Edit(usize),
    /// The row at this index was deleted.
    Delete(usize),
    /// The edits to the row at this index were saved.
    Save(usize),
    /// The edits to the row at this index were discarded.
    Cancel(usize),
}

/// Cell content - either text or a custom widget
pub enum CellContent {
    /// Plain or styled text.
    Text(WidgetText),
    /// A closure that draws the cell's content.
    Widget(std::sync::Arc<dyn Fn(&mut egui::Ui) + Send + Sync>),
}

/// A single cell in the data table
pub struct DataTableCell {
    /// What the cell shows.
    pub content: CellContent,
    /// Horizontal alignment; `None` uses the column's alignment.
    pub h_align: Option<HAlign>,
    /// Vertical alignment; `None` uses the column's alignment.
    pub v_align: Option<VAlign>,
    /// Whether the text is placeholder text, drawn in a muted color.
    pub placeholder: bool,
    /// Whether an edit icon is drawn next to the content.
    pub show_edit_icon: bool,
}

impl DataTableCell {
    /// A cell that shows text.
    pub fn text(text: impl Into<WidgetText>) -> Self {
        Self {
            content: CellContent::Text(text.into()),
            h_align: None,
            v_align: None,
            placeholder: false,
            show_edit_icon: false,
        }
    }

    /// A cell whose content is drawn by the given closure.
    pub fn widget<F>(f: F) -> Self
    where
        F: Fn(&mut egui::Ui) + Send + Sync + 'static,
    {
        Self {
            content: CellContent::Widget(std::sync::Arc::new(f)),
            h_align: None,
            v_align: None,
            placeholder: false,
            show_edit_icon: false,
        }
    }

    /// Sets the horizontal alignment of this cell.
    pub fn h_align(mut self, align: HAlign) -> Self {
        self.h_align = Some(align);
        self
    }

    /// Sets the vertical alignment of this cell.
    pub fn v_align(mut self, align: VAlign) -> Self {
        self.v_align = Some(align);
        self
    }

    /// Marks the text as placeholder text, drawn in a muted color.
    pub fn placeholder(mut self, is_placeholder: bool) -> Self {
        self.placeholder = is_placeholder;
        self
    }

    /// Shows an edit icon next to the content.
    pub fn show_edit_icon(mut self, show: bool) -> Self {
        self.show_edit_icon = show;
        self
    }
}

/// A column definition in the data table
#[derive(Clone)]
pub struct DataTableColumn {
    /// Header text of the column.
    pub title: String,
    /// A closure that draws the header instead of the title text.
    #[expect(
        clippy::type_complexity,
        reason = "a boxed UI callback; the closure signature is clearer spelled out than behind an alias"
    )]
    pub header_widget: Option<std::sync::Arc<dyn Fn(&mut egui::Ui) + Send + Sync>>,
    /// Width of the column, in points.
    pub width: f32,
    /// Whether the column holds numbers, which sorts it numerically.
    pub numeric: bool,
    /// Whether clicking the header sorts by this column.
    pub sortable: bool,
    /// Current sort direction of this column, if it is sorted.
    pub sort_direction: Option<SortDirection>,
    /// Horizontal alignment of the column's cells.
    pub h_align: HAlign,
    /// Vertical alignment of the column's cells.
    pub v_align: VAlign,
    /// Tooltip shown when hovering the header.
    pub tooltip: Option<String>,
    /// Horizontal alignment of the header; `None` uses the column's alignment.
    pub heading_alignment: Option<HAlign>,
    /// How the width of the column is determined.
    pub column_width: ColumnWidth,
}

/// A single row in the data table
pub struct DataTableRow<'a> {
    /// The cells of the row, one per column.
    pub cells: Vec<DataTableCell>,
    /// Whether the row is selected.
    pub selected: bool,
    /// Whether `selected` was set by the caller, so it overrides the stored selection state.
    pub selection_externally_set: bool,
    /// Whether the row cannot be selected or edited.
    pub readonly: bool,
    /// Optional identifier for the row.
    pub id: Option<String>,
    /// Background color of the row; `None` uses the theme's row color.
    pub color: Option<Color32>,
    /// Whether the row highlights when hovered.
    pub on_hover: bool,
    /// A closure that draws an expandable drawer below the row.
    #[expect(
        clippy::type_complexity,
        reason = "a boxed UI callback; the closure signature is clearer spelled out than behind an alias"
    )]
    pub drawer: Option<std::sync::Arc<dyn Fn(&mut egui::Ui) + Send + Sync>>,
    /// Ties the row to the lifetime `'a`.
    pub _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Default for DataTableRow<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DataTableRow<'a> {
    /// An empty row.
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            selected: false,
            selection_externally_set: false,
            readonly: false,
            id: None,
            color: None,
            on_hover: true,
            drawer: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Appends a text cell.
    pub fn cell(mut self, text: impl Into<WidgetText>) -> Self {
        self.cells.push(DataTableCell::text(text));
        self
    }

    /// Appends a cell drawn by the given closure.
    pub fn cell_widget<F>(mut self, widget: F) -> Self
    where
        F: Fn(&mut egui::Ui) + Send + Sync + 'static,
    {
        self.cells.push(DataTableCell::widget(widget));
        self
    }

    /// Appends a fully configured cell.
    pub fn cell_custom(mut self, cell: DataTableCell) -> Self {
        self.cells.push(cell);
        self
    }

    /// Alias for cell_custom (backwards compatibility)
    pub fn custom_cell(self, cell: DataTableCell) -> Self {
        self.cell_custom(cell)
    }

    /// Sets whether the row is selected, overriding the stored selection state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self.selection_externally_set = true;
        self
    }

    /// Makes the row read-only: it cannot be selected or edited.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Sets an identifier for the row.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the background color of the row.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets whether the row highlights when hovered.
    pub fn on_hover(mut self, hover: bool) -> Self {
        self.on_hover = hover;
        self
    }

    /// Adds an expandable drawer below the row, drawn by the given closure.
    pub fn drawer<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut egui::Ui) + Send + Sync + 'static,
    {
        self.drawer = Some(std::sync::Arc::new(f));
        self
    }

    /// Conditionally add a drawer based on a boolean condition.
    /// Only sets the drawer if `condition` is true.
    pub fn drawer_if<F>(mut self, condition: bool, f: F) -> Self
    where
        F: Fn(&mut egui::Ui) + Send + Sync + 'static,
    {
        if condition {
            self.drawer = Some(std::sync::Arc::new(f));
        }
        self
    }
}

/// Persistent state for a Material Design data table
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct DataTableState {
    /// Selection state of each row, by row index.
    pub selected_rows: Vec<bool>,
    /// Whether the header's select-all checkbox is checked.
    pub header_checkbox: bool,
    /// Sort direction of each column, by column title.
    pub column_sorts: HashMap<String, SortDirection>,
    /// Index of the column the table is sorted by, if any.
    pub sorted_column: Option<usize>,
    /// Direction of the current sort.
    pub sort_direction: SortDirection,
    /// Indices of the rows in edit mode.
    pub editing_rows: HashSet<usize>,
    /// Edited cell values of each row in edit mode, by row index.
    pub edit_data: HashMap<usize, Vec<String>>,
    /// Indices of the rows whose drawer is open.
    pub drawer_open_rows: HashSet<usize>,

    // Performance cache fields - not persisted to avoid serialization overhead.
    // These are recalculated on demand when layout_cache_hash doesn't match.
    // Note: sorted_column and sort_direction ARE persisted above, so sorting
    // state is preserved. These cached values are recomputed as needed.
    /// Cached height of each row; not persisted.
    #[serde(skip)]
    pub cached_row_heights: Vec<f32>,
    /// Cached height of the header row; not persisted.
    #[serde(skip)]
    pub cached_header_height: f32,
    /// Cached row order after sorting; not persisted.
    #[serde(skip)]
    pub cached_sorted_indices: Vec<usize>,
    /// Hash of the layout inputs the caches were computed for; not persisted.
    #[serde(skip)]
    pub layout_cache_hash: u64,
    /// Time the caches were last refreshed, in seconds; not persisted.
    #[serde(skip)]
    pub last_refresh_time: f64,
}

/// Response returned by the data table widget
#[derive(Debug)]
pub struct DataTableResponse {
    /// The response of the whole table.
    pub response: egui::Response,
    /// Selection state of each row after this frame.
    pub selected_rows: Vec<bool>,
    /// Whether the header's select-all checkbox is checked.
    pub header_checkbox: bool,
    /// Index of the column whose header was clicked this frame, if any.
    pub column_clicked: Option<usize>,
    /// The sorted column, if any, and the sort direction.
    pub sort_state: (Option<usize>, SortDirection),
    /// Row actions (edit, delete, save, cancel) triggered this frame.
    pub row_actions: Vec<RowAction>,
}

/// Trait for providing data to a table lazily
pub trait DataTableSource {
    /// Number of rows the source provides.
    fn row_count(&self) -> usize;
    /// The row at `index`, or `None` if there is none.
    fn get_row(&self, index: usize) -> Option<DataTableRow<'_>>;
    /// Whether `row_count` is an estimate rather than an exact count.
    fn is_row_count_approximate(&self) -> bool {
        false
    }
    /// Number of rows currently selected.
    fn selected_row_count(&self) -> usize {
        0
    }
}
