// ============================================================================
// Module:       list::types
// Description:  Type definitions for Material Design 3 List Components.
//
// Dependencies: egui
// ============================================================================

//! Type definitions for Material Design 3 List Components

use egui::{ecolor::Color32, Rect, Response, Sense, Vec2};

/// Which title font a list tile uses.
pub enum ListTileStyle {
    /// Use a title font appropriate for a list tile in a list.
    List,
    /// Use a title font appropriate for a list tile in a drawer.
    Drawer,
}

/// Defines how leading and trailing widgets are vertically aligned
/// relative to the list tile's titles (title and subtitle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListTileTitleAlignment {
    /// The top of leading/trailing widgets are placed below the title top
    /// if three-line, otherwise centered relative to title and subtitle.
    /// This is the default for Material 3.
    ThreeLine,
    /// Leading/trailing are placed 16px below title top if tile height > 72,
    /// otherwise centered. This is the default for Material 2.
    TitleHeight,
    /// Leading/trailing tops are placed at min vertical padding below title top.
    Top,
    /// Leading/trailing are centered relative to the titles.
    Center,
    /// Leading/trailing bottoms are placed at min vertical padding above title bottom.
    Bottom,
}

/// Defines the visual density for the list tile layout.
///
/// Visual density allows for compact, comfortable, or spacious layouts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualDensity {
    /// Horizontal density adjustment (-4.0 to 4.0)
    pub horizontal: f32,
    /// Vertical density adjustment (-4.0 to 4.0)
    pub vertical: f32,
}

impl VisualDensity {
    /// Standard density (no adjustment)
    pub const STANDARD: Self = Self {
        horizontal: 0.0,
        vertical: 0.0,
    };

    /// Comfortable density (slightly more spacious)
    pub const COMFORTABLE: Self = Self {
        horizontal: -1.0,
        vertical: -1.0,
    };

    /// Compact density (space-efficient)
    pub const COMPACT: Self = Self {
        horizontal: -2.0,
        vertical: -2.0,
    };

    /// Create a custom visual density
    pub fn new(horizontal: f32, vertical: f32) -> Self {
        Self {
            horizontal: horizontal.clamp(-4.0, 4.0),
            vertical: vertical.clamp(-4.0, 4.0),
        }
    }

    /// Get the base size adjustment as a Vec2
    pub fn base_size_adjustment(&self) -> Vec2 {
        Vec2::new(self.horizontal * 4.0, self.vertical * 4.0)
    }
}

impl Default for VisualDensity {
    fn default() -> Self {
        Self::STANDARD
    }
}

/// Material Design list component.
///
/// Lists are continuous, vertical indexes of text or images.
/// They are composed of items containing primary and related actions.
///
/// # Example
/// ```rust
/// # use egui_material3::*;
/// # egui::__run_test_ui(|ui| {
/// let list = MaterialList::new()
///     .item(ListItem::new("Inbox")
///         .leading_icon("inbox")
///         .trailing_text("12"))
///     .item(ListItem::new("Starred")
///         .leading_icon("star")
///         .trailing_text("3"))
///     .dividers(true);
///
/// ui.add(list);
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct MaterialList<'a> {
    /// List of items to display
    pub(crate) items: Vec<ListItem<'a>>,
    /// Whether to show dividers between items
    pub(crate) dividers: bool,
    /// Optional unique ID for this list to avoid widget ID collisions
    pub(crate) id: Option<egui::Id>,
    /// Whether to enable drawer functionality
    pub(crate) allow_drawer: bool,
    /// Whether to fill available width instead of using content width
    pub(crate) fill_width: bool,
    /// M3 Expressive segmented list: items in separate containers, 2 dp apart
    pub(crate) segmented: bool,
    /// What each row senses besides clicks (e.g. drags)
    pub(crate) row_sense: Sense,
    /// Whether `row_sense` replaces the default click sense instead of
    /// adding to it
    pub(crate) row_sense_exact: bool,
    /// Space between rows of a non-segmented list, if not the default
    pub(crate) row_gap: Option<f32>,
}

/// What [`MaterialList::show`] reports: the list's own response and one
/// response per row.
///
/// Each row's response is the whole item container, with the row's id,
/// and senses clicks (so secondary and double clicks too) plus whatever
/// [`MaterialList::row_sense`] adds, or exactly what
/// [`MaterialList::row_sense_exact`] sets; a disabled row senses hover only.
/// Attach context menus, double-click handlers or drag sources to it:
///
/// ```rust
/// # egui::__run_test_ui(|ui| {
/// use egui_material3::{list_item, MaterialList, MaterialMenu, menu_item};
///
/// let shown = MaterialList::new()
///     .id(egui::Id::unique("files"))
///     .row_sense(egui::Sense::drag())
///     .item(list_item("clip-01.mp4"))
///     .item(list_item("clip-02.mp4"))
///     .show(ui);
/// for (index, row) in shown.items.iter().enumerate() {
///     if row.double_clicked() {
///         // open item `index`
///     }
///     if row.drag_started() {
///         // start dragging item `index`
///     }
///     MaterialMenu::context(row.id.with("menu"))
///         .item(menu_item("Rename"))
///         .show_context(row);
/// }
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct ListResponse {
    /// The whole list.
    pub response: Response,
    /// One response per row, index-aligned with the items added.
    pub items: Vec<Response>,
}

impl ListResponse {
    /// The response of row `index`, if there is one.
    pub fn item(&self, index: usize) -> Option<&Response> {
        self.items.get(index)
    }
}

/// What [`MaterialList::show_rows`] reports: the list's own response, the
/// rows the viewport shows, and one response per row that was built.
///
/// `rows` holds the visible rows in order, plus the focused row and the
/// row keyboard roving moved to when they are outside the viewport (they
/// are laid out so they keep focus, and scrolled to). Each response is the
/// whole row, with the row's id, as in [`ListResponse`].
#[derive(Debug, Clone)]
pub struct ListRowsResponse {
    /// The whole list, `row_count` rows tall.
    pub response: Response,
    /// The rows inside the viewport.
    pub visible: std::ops::Range<usize>,
    /// `(row index, response)` for every row built this frame, by index.
    pub rows: Vec<(usize, Response)>,
}

impl ListRowsResponse {
    /// The response of row `index`, if it was built this frame.
    pub fn row(&self, index: usize) -> Option<&Response> {
        self.rows
            .binary_search_by_key(&index, |(i, _)| *i)
            .ok()
            .and_then(|position| self.rows.get(position))
            .map(|(_, response)| response)
    }
}

/// What a list item shows at its start, besides a plain icon.
///
/// Sizes and shapes from Compose's `ListTokens.kt`.
pub enum ListLeading {
    /// A round avatar with a short text (initials):
    /// `ItemLeadingAvatarSize` 40 dp, `ItemLeadingAvatarColor`
    /// `primaryContainer`, label `TitleMedium` in `onPrimaryContainer`.
    Avatar(String),
    /// A square image: `ItemLeadingImageWidth`/`Height` 56 dp,
    /// `ItemLeadingImageShape` `CornerNone`.
    Image(egui::ImageSource<'static>),
    /// A video thumbnail: small `ItemSmallLeadingVideoWidth` 100 x 56 dp, or
    /// large `ItemLargeLeadingVideoWidth` 114 x 64 dp;
    /// `ItemLeadingVideoShape` `CornerSmall`.
    Video {
        /// The thumbnail image.
        source: egui::ImageSource<'static>,
        /// The large (114 x 64) size instead of the small (100 x 56) one.
        large: bool,
    },
    /// A checkbox showing the given state. The item reports itself as a
    /// checkbox; toggle your state in `on_click`.
    Checkbox(bool),
    /// A radio button showing the given state. The item reports itself as a
    /// radio button; update your state in `on_click`.
    Radio(bool),
}

/// What a custom list-item callback ([`ListItem::leading_ui`],
/// [`ListItem::content_ui`]) sees: the item's geometry, state and the
/// colours the built-in content would use in that state, so custom content
/// can match selection, disabled and hover styling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListItemState {
    /// The whole item (container) rect.
    pub rect: Rect,
    /// Whether the item is enabled.
    pub enabled: bool,
    /// Whether the item is selected.
    pub selected: bool,
    /// Whether the pointer is over the item.
    pub hovered: bool,
    /// Whether the item is being pressed.
    pub pressed: bool,
    /// Whether the item has keyboard focus.
    pub focused: bool,
    /// The label colour (`onSurface`, `onSecondaryContainer` when selected,
    /// 38 % `onSurface` when disabled).
    pub label_color: Color32,
    /// The supporting-text colour.
    pub supporting_color: Color32,
    /// The icon colour.
    pub icon_color: Color32,
}

/// A custom list-item callback: a `Ui` over its slot, and the item state.
pub(crate) type ListItemUi<'a> = Box<dyn FnOnce(&mut egui::Ui, &ListItemState) + 'a>;

/// Persistent state for a Material Design list
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MaterialListState {
    /// Set of item indices that have their drawers expanded
    pub drawer_open_items: std::collections::HashSet<usize>,
}

/// Individual item in a Material Design list.
///
/// List items can contain primary text, secondary text, overline text,
/// leading and trailing icons, and custom actions.
///
/// # Example
/// ```rust
/// # use egui_material3::*;
/// let item = ListItem::new("Primary Text")
///     .secondary_text("Secondary supporting text")
///     .leading_icon("person")
///     .trailing_icon("more_vert")
///     .on_click(|| println!("Item clicked"));
/// ```
pub struct ListItem<'a> {
    /// Main text displayed for this item
    pub(crate) primary_text: String,
    /// Optional secondary text displayed below primary text
    pub(crate) secondary_text: Option<String>,
    /// Optional overline text displayed above primary text
    pub(crate) overline_text: Option<String>,
    /// Optional icon displayed at the start of the item
    pub(crate) leading_icon: Option<String>,
    /// Optional text label displayed below the leading icon
    pub(crate) leading_icon_label: Option<String>,
    /// Optional icon displayed at the end of the item
    pub(crate) trailing_icon: Option<String>,
    /// Optional text displayed at the end of the item
    pub(crate) trailing_text: Option<String>,
    /// Optional custom widget rendered in the trailing area
    #[expect(
        clippy::type_complexity,
        reason = "a boxed UI callback; the closure signature is clearer spelled out than behind an alias"
    )]
    pub(crate) trailing_widget: Option<std::sync::Arc<dyn Fn(&mut egui::Ui) + Send + Sync>>,
    /// Whether the item is enabled and interactive
    pub(crate) enabled: bool,
    /// Whether the item is selected
    pub(crate) selected: bool,
    /// Whether this list tile is part of a vertically dense list
    pub(crate) dense: Option<bool>,
    /// Whether this list tile is intended to display three lines of text
    pub(crate) is_three_line: Option<bool>,
    /// Defines how compact the list tile's layout will be
    pub(crate) visual_density: Option<VisualDensity>,
    /// Defines the font used for the title
    pub(crate) style: Option<ListTileStyle>,
    /// Defines how leading and trailing are vertically aligned
    pub(crate) title_alignment: Option<ListTileTitleAlignment>,
    /// The horizontal gap between the titles and the leading/trailing widgets
    pub(crate) horizontal_title_gap: Option<f32>,
    /// The minimum padding on the top and bottom of the title and subtitle widgets
    pub(crate) min_vertical_padding: Option<f32>,
    /// The minimum width allocated for the leading widget
    pub(crate) min_leading_width: Option<f32>,
    /// The minimum height allocated for the list tile widget
    pub(crate) min_tile_height: Option<f32>,
    /// An exact row height, below the text's natural height if need be
    pub(crate) row_height: Option<f32>,
    /// Background color when selected is false
    pub(crate) tile_color: Option<Color32>,
    /// Background color when selected is true
    pub(crate) selected_tile_color: Option<Color32>,
    /// Color for icons and text when selected
    pub(crate) selected_color: Option<Color32>,
    /// Default color for leading and trailing icons
    pub(crate) icon_color: Option<Color32>,
    /// Text color for title, subtitle, leading, and trailing
    pub(crate) text_color: Option<Color32>,
    /// Callback function to execute when the item is clicked
    pub(crate) action: Option<Box<dyn Fn() + 'a>>,
    /// Optional leading avatar, image, video, checkbox or radio
    pub(crate) leading: Option<ListLeading>,
    /// Whether the item is being dragged (shows the dragged state layer)
    pub(crate) dragged: bool,
    /// Optional drawer content that expands below the item
    #[expect(
        clippy::type_complexity,
        reason = "a boxed UI callback; the closure signature is clearer spelled out than behind an alias"
    )]
    pub(crate) drawer: Option<std::sync::Arc<dyn Fn(&mut egui::Ui) + Send + Sync>>,
    /// Custom UI in the leading slot, with the slot's size
    pub(crate) leading_ui: Option<(Vec2, ListItemUi<'a>)>,
    /// Custom UI replacing the leading, text and trailing content
    pub(crate) content_ui: Option<ListItemUi<'a>>,
    /// Corners of the container, state layer and focus ring in every state
    pub(crate) corner_radius: Option<egui::CornerRadius>,
}
