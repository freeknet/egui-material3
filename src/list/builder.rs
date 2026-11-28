// ============================================================================
// Module:       list::builder
// Description:  Builder methods for MaterialList.
//
// Dependencies: egui; super
// ============================================================================

//! Builder methods for MaterialList

use super::{types::*, MaterialList};
use egui::ecolor::Color32;

impl<'a> Default for MaterialList<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MaterialList<'a> {
    /// Create a new empty list.
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let list = MaterialList::new();
    /// ```
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            dividers: true,
            id: None,
            allow_drawer: false,
            fill_width: false,
            segmented: false,
            row_sense: egui::Sense::hover(),
        }
    }

    /// What every row senses in addition to clicks, e.g.
    /// `Sense::drag()` to make rows drag sources. Rows always sense clicks
    /// (and so secondary and double clicks); read them from
    /// [`MaterialList::show`]. Adding drag makes a drag that starts on a
    /// row belong to the row rather than to a surrounding `ScrollArea`.
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let list = MaterialList::new()
    ///     .row_sense(egui::Sense::drag())
    ///     .item(ListItem::new("Drag me"));
    /// ```
    pub fn row_sense(mut self, sense: egui::Sense) -> Self {
        self.row_sense = sense;
        self
    }

    /// M3 Expressive segmented list: every item sits in its own `surface`
    /// container, 2 dp apart (`ListTokens.SegmentedGap`), with large outer
    /// corners and extra-small inner ones. Dividers are not drawn.
    pub fn segmented(mut self, segmented: bool) -> Self {
        self.segmented = segmented;
        self
    }

    /// Add an item to the list.
    ///
    /// # Arguments
    /// * `item` - The list item to add
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// # egui::__run_test_ui(|ui| {
    /// let item = ListItem::new("Sample Item");
    /// let list = MaterialList::new().item(item);
    /// # });
    /// ```
    pub fn item(mut self, item: ListItem<'a>) -> Self {
        self.items.push(item);
        self
    }

    /// Set whether to show dividers between items.
    ///
    /// # Arguments
    /// * `dividers` - Whether to show divider lines between items
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let list = MaterialList::new().dividers(false); // No dividers
    /// ```
    pub fn dividers(mut self, dividers: bool) -> Self {
        self.dividers = dividers;
        self
    }

    /// Set a custom ID for this list to avoid widget ID collisions.
    ///
    /// Use this when you have multiple lists with similar content in the same UI.
    ///
    /// # Arguments
    /// * `id` - A unique identifier for this list
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let list = MaterialList::new()
    ///     .id(egui::Id::unique("my_list"))
    ///     .dividers(false);
    /// ```
    pub fn id(mut self, id: impl Into<egui::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Enable drawer functionality for list items.
    ///
    /// When enabled, items with drawers will show an expand/collapse arrow.
    /// Clicking anywhere on the item toggles the drawer visibility.
    ///
    /// **Note**: For items with both a drawer and an action callback,
    /// the drawer toggle takes priority and the action callback is ignored.
    ///
    /// # Arguments
    /// * `allow` - Whether to enable drawer functionality
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let list = MaterialList::new()
    ///     .allow_drawer(true)
    ///     .item(ListItem::new("Item").drawer(|ui| {
    ///         ui.label("Drawer content");
    ///     }));
    /// ```
    pub fn allow_drawer(mut self, allow: bool) -> Self {
        self.allow_drawer = allow;
        self
    }

    /// Set whether the list should fill the available width.
    ///
    /// When enabled, the list expands to fill the full available width
    /// instead of sizing to fit its content.
    ///
    /// # Arguments
    /// * `fill` - Whether to fill available width
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let list = MaterialList::new()
    ///     .fill_width(true)
    ///     .item(ListItem::new("Item"));
    /// ```
    pub fn fill_width(mut self, fill: bool) -> Self {
        self.fill_width = fill;
        self
    }
}

impl<'a> ListItem<'a> {
    /// Create a new list item with primary text.
    ///
    /// # Arguments
    /// * `primary_text` - The main text to display
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("My List Item");
    /// ```
    pub fn new(primary_text: impl Into<String>) -> Self {
        Self {
            primary_text: primary_text.into(),
            secondary_text: None,
            overline_text: None,
            leading_icon: None,
            leading_icon_label: None,
            trailing_icon: None,
            trailing_text: None,
            trailing_widget: None,
            enabled: true,
            selected: false,
            dense: None,
            is_three_line: None,
            visual_density: None,
            style: None,
            title_alignment: None,
            horizontal_title_gap: None,
            min_vertical_padding: None,
            min_leading_width: None,
            min_tile_height: None,
            row_height: None,
            tile_color: None,
            selected_tile_color: None,
            selected_color: None,
            icon_color: None,
            text_color: None,
            action: None,
            leading: None,
            dragged: false,
            drawer: None,
            leading_ui: None,
            content_ui: None,
        }
    }

    /// A round avatar with a short text (initials) at the start.
    pub fn leading_avatar(mut self, text: impl Into<String>) -> Self {
        self.leading = Some(ListLeading::Avatar(text.into()));
        self
    }

    /// A 56 dp square image at the start.
    pub fn leading_image(mut self, source: impl Into<egui::ImageSource<'static>>) -> Self {
        self.leading = Some(ListLeading::Image(source.into()));
        self
    }

    /// A video thumbnail at the start: 100 x 56 dp, or 114 x 64 dp when
    /// `large`.
    pub fn leading_video(
        mut self,
        source: impl Into<egui::ImageSource<'static>>,
        large: bool,
    ) -> Self {
        self.leading = Some(ListLeading::Video {
            source: source.into(),
            large,
        });
        self
    }

    /// A checkbox at the start showing `checked`. Toggle your state in
    /// [`ListItem::on_click`].
    pub fn leading_checkbox(mut self, checked: bool) -> Self {
        self.leading = Some(ListLeading::Checkbox(checked));
        self
    }

    /// A radio button at the start showing `selected`. Update your state in
    /// [`ListItem::on_click`].
    pub fn leading_radio(mut self, selected: bool) -> Self {
        self.leading = Some(ListLeading::Radio(selected));
        self
    }

    /// Custom UI in the leading slot, `size` points large. It replaces any
    /// other leading element. The callback gets a `Ui` whose `max_rect` is
    /// the slot (placed and mirrored like the built-in leading elements)
    /// and the item's [`ListItemState`]. The item keeps its container,
    /// state layer, selection, focus ring, keyboard handling and AccessKit
    /// node (labelled with the primary text).
    ///
    /// ```rust
    /// # use egui_material3::*;
    /// # egui::__run_test_ui(|ui| {
    /// ui.add(MaterialList::new().item(
    ///     ListItem::new("Clip 3").leading_ui(egui::vec2(48.0, 27.0), |ui, state| {
    ///         ui.painter().rect_filled(ui.max_rect(), 4.0, state.icon_color);
    ///     }),
    /// ));
    /// # });
    /// ```
    pub fn leading_ui(
        mut self,
        size: egui::Vec2,
        add_contents: impl FnOnce(&mut egui::Ui, &ListItemState) + 'a,
    ) -> Self {
        let size = egui::Vec2::new(
            if size.x.is_finite() {
                size.x.max(0.0)
            } else {
                0.0
            },
            if size.y.is_finite() {
                size.y.max(0.0)
            } else {
                0.0
            },
        );
        self.leading_ui = Some((size, Box::new(add_contents)));
        self
    }

    /// Custom UI for the whole row: it replaces the leading element, the
    /// text and the trailing elements, while the item keeps its container,
    /// state layer, selection, focus ring, click handling, keyboard roving
    /// and AccessKit node (still labelled with the primary text, which is
    /// not painted). The callback gets a left-to-right `Ui` over the item
    /// inset by the 16 dp leading and trailing space, and the item's
    /// [`ListItemState`] (whose `rect` is the whole item).
    ///
    /// The row is as tall as the built-in one would be for the primary
    /// text (56 dp for one line); set [`min_tile_height`](Self::min_tile_height)
    /// for taller content. Widgets added by the callback sit above the item
    /// and take their own clicks.
    pub fn content_ui(
        mut self,
        add_contents: impl FnOnce(&mut egui::Ui, &ListItemState) + 'a,
    ) -> Self {
        self.content_ui = Some(Box::new(add_contents));
        self
    }

    /// Any [`ListLeading`] element at the start.
    pub fn leading(mut self, leading: ListLeading) -> Self {
        self.leading = Some(leading);
        self
    }

    /// Show the item as being dragged (for reorderable lists): the dragged
    /// state layer (`StateLayerTokens.DraggedOpacity`) and
    /// `ListTokens.ItemDraggedContainerElevation` (`Level4`).
    pub fn dragged(mut self, dragged: bool) -> Self {
        self.dragged = dragged;
        self
    }

    /// Set the secondary text for the item.
    ///
    /// Secondary text is displayed below the primary text.
    ///
    /// # Arguments
    /// * `text` - The secondary text to display
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .secondary_text("This is some secondary text");
    /// ```
    pub fn secondary_text(mut self, text: impl Into<String>) -> Self {
        self.secondary_text = Some(text.into());
        self
    }

    /// Set the overline text for the item.
    ///
    /// Overline text is displayed above the primary text.
    ///
    /// # Arguments
    /// * `text` - The overline text to display
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .overline("Important")
    ///     .secondary_text("This is some secondary text");
    /// ```
    pub fn overline(mut self, text: impl Into<String>) -> Self {
        self.overline_text = Some(text.into());
        self
    }

    /// Set a leading icon for the item.
    ///
    /// A leading icon is displayed at the start of the item, before the text.
    ///
    /// # Arguments
    /// * `icon` - The name of the icon to display
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .leading_icon("check");
    /// ```
    pub fn leading_icon(mut self, icon: impl Into<String>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// Set a text label to display below the leading icon.
    ///
    /// The label appears below the icon in a smaller font.
    /// Only rendered when a leading icon is also present.
    ///
    /// # Arguments
    /// * `label` - The text to display below the icon
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .leading_icon("folder")
    ///     .leading_icon_label("Department");
    /// ```
    pub fn leading_icon_label(mut self, label: impl Into<String>) -> Self {
        self.leading_icon_label = Some(label.into());
        self
    }

    /// Set a trailing icon for the item.
    ///
    /// A trailing icon is displayed at the end of the item, after the text.
    ///
    /// # Arguments
    /// * `icon` - The name of the icon to display
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .trailing_icon("more_vert");
    /// ```
    pub fn trailing_icon(mut self, icon: impl Into<String>) -> Self {
        self.trailing_icon = Some(icon.into());
        self
    }

    /// Set trailing text for the item.
    ///
    /// Trailing text is displayed at the end of the item, after the icons.
    ///
    /// # Arguments
    /// * `text` - The trailing text to display
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .trailing_text("99+");
    /// ```
    pub fn trailing_text(mut self, text: impl Into<String>) -> Self {
        self.trailing_text = Some(text.into());
        self
    }

    /// Set a custom widget to render in the trailing area.
    ///
    /// The widget is rendered in the trailing area where icons/text normally appear.
    /// Useful for action buttons or custom controls.
    ///
    /// # Arguments
    /// * `widget` - A closure that renders the widget
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .trailing_widget(|ui| {
    ///         if ui.button("Edit").clicked() {
    ///             // Handle edit
    ///         }
    ///     });
    /// ```
    pub fn trailing_widget(
        mut self,
        widget: impl Fn(&mut egui::Ui) + Send + Sync + 'static,
    ) -> Self {
        self.trailing_widget = Some(std::sync::Arc::new(widget));
        self
    }

    /// Enable or disable the item.
    ///
    /// Disabled items are not interactive and are typically displayed with
    /// reduced opacity.
    ///
    /// # Arguments
    /// * `enabled` - Whether the item should be enabled
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .enabled(false); // This item is disabled
    /// ```
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the selected state of the item.
    ///
    /// Selected items are highlighted with a different background color
    /// and may use different text/icon colors.
    ///
    /// # Arguments
    /// * `selected` - Whether the item should appear selected
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .selected(true); // This item appears selected
    /// ```
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set whether this list tile is part of a vertically dense list.
    ///
    /// Dense list tiles default to a smaller height.
    ///
    /// # Arguments
    /// * `dense` - Whether to use dense layout
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .dense(true); // Compact layout
    /// ```
    pub fn dense(mut self, dense: bool) -> Self {
        self.dense = Some(dense);
        self
    }

    /// Set whether this list tile is intended to display three lines of text.
    ///
    /// # Arguments
    /// * `is_three_line` - Whether to use three-line layout
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .is_three_line(true);
    /// ```
    pub fn is_three_line(mut self, is_three_line: bool) -> Self {
        self.is_three_line = Some(is_three_line);
        self
    }

    /// Set the visual density for compact/comfortable/spacious layouts.
    ///
    /// # Arguments
    /// * `density` - The visual density to apply
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .visual_density(VisualDensity::COMPACT);
    /// ```
    pub fn visual_density(mut self, density: VisualDensity) -> Self {
        self.visual_density = Some(density);
        self
    }

    /// Set the title style (List or Drawer).
    ///
    /// # Arguments
    /// * `style` - The list tile style
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .style(ListTileStyle::Drawer);
    /// ```
    pub fn style(mut self, style: ListTileStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Set how leading and trailing widgets are vertically aligned.
    ///
    /// # Arguments
    /// * `alignment` - The title alignment mode
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .title_alignment(ListTileTitleAlignment::Center);
    /// ```
    pub fn title_alignment(mut self, alignment: ListTileTitleAlignment) -> Self {
        self.title_alignment = Some(alignment);
        self
    }

    /// Set the horizontal gap between titles and leading/trailing widgets.
    ///
    /// # Arguments
    /// * `gap` - The gap in pixels
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .horizontal_title_gap(20.0);
    /// ```
    pub fn horizontal_title_gap(mut self, gap: f32) -> Self {
        self.horizontal_title_gap = Some(gap);
        self
    }

    /// Set the minimum padding on top and bottom of title/subtitle.
    ///
    /// # Arguments
    /// * `padding` - The minimum vertical padding in pixels
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .min_vertical_padding(8.0);
    /// ```
    pub fn min_vertical_padding(mut self, padding: f32) -> Self {
        self.min_vertical_padding = Some(padding);
        self
    }

    /// Set the minimum width allocated for the leading widget.
    ///
    /// # Arguments
    /// * `width` - The minimum leading width in pixels
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .min_leading_width(48.0);
    /// ```
    pub fn min_leading_width(mut self, width: f32) -> Self {
        self.min_leading_width = Some(width);
        self
    }

    /// Set the minimum height allocated for the list tile.
    ///
    /// # Arguments
    /// * `height` - The minimum tile height in pixels
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .min_tile_height(64.0);
    /// ```
    pub fn min_tile_height(mut self, height: f32) -> Self {
        self.min_tile_height = Some(height);
        self
    }

    /// An exact row height in points, overriding both the token heights
    /// (56/72/88 dp, set with [`min_tile_height`](Self::min_tile_height))
    /// and the floor the text and leading media would otherwise impose, so
    /// rows can be shorter than the primary text's default: 28 dp desktop
    /// rows, say. This is not a Material 3 token value; pick a
    /// [`dense`](Self::dense) item (`BodyMedium`, a 20 dp line) for rows
    /// under 44 dp. The text stays vertically centred and is clipped to the
    /// row; leading and trailing elements stay centred. Non-finite values
    /// are ignored and the height is at least 1.
    ///
    /// ```rust
    /// # use egui_material3::*;
    /// # egui::__run_test_ui(|ui| {
    /// ui.add(
    ///     MaterialList::new()
    ///         .dividers(false)
    ///         .item(ListItem::new("clip-01.mp4").dense(true).row_height(28.0))
    ///         .item(ListItem::new("clip-02.mp4").dense(true).row_height(28.0)),
    /// );
    /// # });
    /// ```
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height.is_finite().then(|| height.max(1.0));
        self
    }

    /// Set the background color when not selected.
    ///
    /// Overrides the default transparent background.
    /// By default, non-selected items are transparent to show the parent surface.
    ///
    /// # Arguments
    /// * `color` - The tile background color
    ///
    /// # Example
    /// ```rust
    /// # use egui::Color32;
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .tile_color(Color32::from_rgb(240, 240, 240));
    /// ```
    pub fn tile_color(mut self, color: Color32) -> Self {
        self.tile_color = Some(color);
        self
    }

    /// Set the background color when selected.
    ///
    /// Overrides the default M3 **primaryContainer** color role.
    /// By default, selected items use primaryContainer (less emphasized fill for selected elements).
    ///
    /// # Arguments
    /// * `color` - The selected tile background color
    ///
    /// # Example
    /// ```rust
    /// # use egui::Color32;
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .selected_tile_color(Color32::from_rgb(200, 230, 255));
    /// ```
    pub fn selected_tile_color(mut self, color: Color32) -> Self {
        self.selected_tile_color = Some(color);
        self
    }

    /// Set the color for icons and text when selected.
    ///
    /// Overrides the default M3 **onPrimaryContainer** color role.
    /// By default, selected items use onPrimaryContainer (content on primaryContainer).
    ///
    /// # Arguments
    /// * `color` - The selected content color
    ///
    /// # Example
    /// ```rust
    /// # use egui::Color32;
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .selected_color(Color32::from_rgb(0, 100, 200));
    /// ```
    pub fn selected_color(mut self, color: Color32) -> Self {
        self.selected_color = Some(color);
        self
    }

    /// Set the default color for leading and trailing icons.
    ///
    /// Overrides the default M3 **onSurfaceVariant** color role.
    /// By default, icons use onSurfaceVariant (lower emphasis for icons).
    ///
    /// # Arguments
    /// * `color` - The icon color
    ///
    /// # Example
    /// ```rust
    /// # use egui::Color32;
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .icon_color(Color32::from_rgb(100, 100, 100));
    /// ```
    pub fn icon_color(mut self, color: Color32) -> Self {
        self.icon_color = Some(color);
        self
    }

    /// Set the text color for title, subtitle, leading, and trailing.
    ///
    /// Overrides the default M3 **onSurface** color role for primary text.
    /// By default, primary text uses onSurface (standard content color on surface).
    /// Note: Secondary/overline text always uses onSurfaceVariant regardless of this setting.
    ///
    /// # Arguments
    /// * `color` - The text color
    ///
    /// # Example
    /// ```rust
    /// # use egui::Color32;
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .text_color(Color32::from_rgb(0, 0, 0));
    /// ```
    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Set a click action for the item.
    ///
    /// # Arguments
    /// * `f` - A function to call when the item is clicked
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item")
    ///     .on_click(|| {
    ///         println!("Item was clicked!");
    ///     });
    /// ```
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: Fn() + 'a,
    {
        self.action = Some(Box::new(f));
        self
    }

    /// Add a drawer that expands below this item when clicked.
    ///
    /// The drawer shows additional content in an expandable panel.
    /// Click anywhere on the item to expand/collapse the drawer.
    ///
    /// **Note**: When a drawer is present, clicking the item toggles the drawer
    /// instead of executing any action callback. The drawer toggle takes priority.
    ///
    /// # Arguments
    /// * `f` - A function that renders the drawer content
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let item = ListItem::new("Item with drawer")
    ///     .drawer(|ui| {
    ///         ui.label("Additional details");
    ///         ui.label("More information here");
    ///     });
    /// ```
    pub fn drawer<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut egui::Ui) + Send + Sync + 'static,
    {
        self.drawer = Some(std::sync::Arc::new(f));
        self
    }

    /// Conditionally add a drawer based on a boolean condition.
    ///
    /// Only sets the drawer if `condition` is true.
    /// Useful for showing drawers only on items that match certain criteria.
    ///
    /// # Arguments
    /// * `condition` - Whether to add the drawer
    /// * `f` - A function that renders the drawer content
    ///
    /// # Example
    /// ```rust
    /// # use egui_material3::*;
    /// let has_details = true;
    /// let item = ListItem::new("Item")
    ///     .drawer_if(has_details, |ui| {
    ///         ui.label("Details shown only when condition is true");
    ///     });
    /// ```
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
