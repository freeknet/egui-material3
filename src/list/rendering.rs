// ============================================================================
// Module:       list::rendering
// Description:  Rendering implementation for MaterialList.
//
// Dependencies: egui; super
// ============================================================================

//! Rendering implementation for MaterialList
//!
//! Values from Compose's `ListTokens.kt`: `ItemContainerColor` `surface`,
//! label `BodyLarge` in `onSurface`, supporting text `BodyMedium` and
//! overline / trailing supporting text `LabelSmall` in `onSurfaceVariant`,
//! icons 24 dp in `onSurfaceVariant`, `ItemLeadingSpace` and
//! `ItemTrailingSpace` 16 dp, `ItemBetweenSpace` 12 dp, `ItemTopSpace` and
//! `ItemBottomSpace` 10 dp, one/two/three-line heights 56/72/88 dp,
//! selected items in `secondaryContainer` / `onSecondaryContainer` with
//! `ItemSelectedContainerShape` `CornerLarge`, disabled content at 0.38,
//! dragged items at `Level4`, and for segmented (Expressive) lists items in
//! separate `surface` containers `SegmentedGap` 2 dp apart, inner corners
//! `ItemContainerExpressiveShape` `CornerExtraSmall`, outer corners
//! `ContainerShape` `CornerLarge`. Expandable ("drawer") items show their
//! chevron in a round container (`ExpandedListTokens.TrailingIconShape`)
//! that is `surfaceContainer` when expanded and `surface` when collapsed.
//!
//! Keyboard: every enabled item is a Tab stop, Up and Down move between
//! items, Space or Enter activates the focused one.
//!
//! [`MaterialList::show`] returns every row's response, so apps can attach
//! context menus, double-click handlers and drag sources to single rows;
//! `ui.add(list)` returns only the list's.

use std::sync::Arc;

use super::{types::*, MaterialList};
use crate::material_symbol::material_symbol_text;
use crate::material_theme::MaterialTheme;
use crate::tokens::state::{DISABLED_CONTENT_OPACITY, DRAGGED_OPACITY};
use crate::tokens::{ColorRole, Elevation, InteractionState, LayoutDirection, TypeRole, TypeStyle};
use egui::{
    ecolor::Color32, CornerRadius, EventFilter, Galley, Id, Key, Pos2, Rect, Response, Sense, Ui,
    Vec2, Widget,
};

/// `ListTokens.ItemLeadingSpace` (`MeasurementTokens.Space200`).
const LEADING_SPACE: f32 = 16.0;
/// `ListTokens.ItemTrailingSpace` (`MeasurementTokens.Space200`).
const TRAILING_SPACE: f32 = 16.0;
/// `ListTokens.ItemBetweenSpace` (`MeasurementTokens.Space150`).
const BETWEEN_SPACE: f32 = 12.0;
/// `ListTokens.ItemTopSpace` / `ItemBottomSpace` (`MeasurementTokens.Space125`).
const VERTICAL_SPACE: f32 = 10.0;
/// Space above and below a leading avatar, image or video: Compose
/// `ListItem.kt` `ListItemVerticalPadding` (8 dp), so a 40 dp avatar fits a
/// 56 dp one-line item and a 56 dp image makes a 72 dp item.
const MEDIA_VERTICAL_PADDING: f32 = 8.0;
/// `ListTokens.ItemLeadingIconSize` / `ItemTrailingIconSize`.
const ICON_SIZE: f32 = 24.0;
/// `ListTokens.ItemLeadingAvatarSize`.
const AVATAR_SIZE: f32 = 40.0;
/// `ListTokens.SegmentedGap` (`MeasurementTokens.Space25`).
const SEGMENTED_GAP: f32 = 2.0;
/// `ShapeKeyTokens.CornerLarge`.
const CORNER_LARGE: u8 = 16;
/// `ShapeKeyTokens.CornerExtraSmall`.
const CORNER_EXTRA_SMALL: u8 = 4;
/// `ShapeKeyTokens.CornerSmall`.
const CORNER_SMALL: u8 = 8;
/// Width reserved for a trailing widget.
const TRAILING_WIDGET_WIDTH: f32 = 150.0;
/// Narrowest content-sized list (as before the M3 restyle).
const MIN_CONTENT_WIDTH: f32 = 200.0;
/// Widest content-sized list: longer text wraps (as before the M3 restyle).
const MAX_CONTENT_WIDTH: f32 = 800.0;
/// Diameter of the expand chevron's round container. Not tokenised in
/// Compose; it holds the 24 dp icon with 4 dp around it.
const EXPAND_CONTAINER: f32 = 32.0;

/// Helper function to measure drawer content height
fn measure_drawer_height(drawer_fn: &dyn Fn(&mut Ui), ui: &mut Ui, width: f32) -> f32 {
    // Create a temporary off-screen rect to measure content
    let temp_rect =
        egui::Rect::from_min_size(egui::pos2(-10000.0, -10000.0), egui::Vec2::new(width, 0.0));

    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(temp_rect)
            .layout(egui::Layout::top_down(egui::Align::LEFT)),
    );

    // Add the same padding as in actual rendering
    child_ui.add_space(8.0);
    child_ui.horizontal(|ui| {
        ui.add_space(20.0); // Left indent
        ui.vertical(|ui| {
            drawer_fn(ui);
        });
    });
    child_ui.add_space(8.0);

    child_ui.min_rect().height()
}

/// The type styles of one item.
struct ItemStyles {
    overline: TypeStyle,
    primary: TypeStyle,
    secondary: TypeStyle,
    trailing: TypeStyle,
}

fn item_styles(item: &ListItem<'_>) -> ItemStyles {
    let dense = item.dense.unwrap_or(false);
    let primary = match (&item.style, dense) {
        (Some(ListTileStyle::Drawer), _) => TypeRole::LabelLarge,
        (Some(ListTileStyle::List) | None, true) => TypeRole::BodyMedium,
        (Some(ListTileStyle::List) | None, false) => TypeRole::BodyLarge,
    };
    let secondary = if dense {
        TypeRole::BodySmall
    } else {
        TypeRole::BodyMedium
    };
    ItemStyles {
        overline: TypeRole::LabelSmall.style(),
        primary: primary.style(),
        secondary: secondary.style(),
        trailing: TypeRole::LabelSmall.style(),
    }
}

fn layout(ui: &Ui, style: &TypeStyle, text: &str, wrap_width: f32) -> Arc<Galley> {
    let mut job = style.layout_job(ui.ctx(), text, Color32::PLACEHOLDER);
    job.wrap.max_width = wrap_width.max(1.0);
    job.wrap.break_anywhere = false;
    ui.painter().layout_job(job)
}

/// Size of the leading element (width and height), if any.
fn leading_size(item: &ListItem<'_>) -> Option<Vec2> {
    if let Some((size, _)) = &item.leading_ui {
        return Some(*size);
    }
    let icon_width = item.min_leading_width.unwrap_or(ICON_SIZE);
    match &item.leading {
        Some(ListLeading::Avatar(_)) => Some(Vec2::splat(AVATAR_SIZE)),
        // ListTokens.ItemLeadingImageWidth / Height.
        Some(ListLeading::Image(_)) => Some(Vec2::splat(56.0)),
        // ListTokens.ItemLargeLeadingVideo* and ItemSmallLeadingVideo*.
        Some(ListLeading::Video { large: true, .. }) => Some(Vec2::new(114.0, 64.0)),
        Some(ListLeading::Video { large: false, .. }) => Some(Vec2::new(100.0, 56.0)),
        Some(ListLeading::Checkbox(_) | ListLeading::Radio(_)) => {
            Some(Vec2::new(icon_width.max(ICON_SIZE), ICON_SIZE))
        }
        None => item
            .leading_icon
            .as_ref()
            .map(|_| Vec2::new(icon_width, ICON_SIZE)),
    }
}

/// Horizontal space the trailing elements take, gap included.
fn trailing_width(ui: &Ui, item: &ListItem<'_>, styles: &ItemStyles, drawer: bool) -> f32 {
    let mut width = 0.0;
    let mut add = |w: f32| {
        width += w + BETWEEN_SPACE;
    };
    if item.trailing_icon.is_some() {
        add(ICON_SIZE);
    } else if drawer && item.trailing_text.is_none() && item.trailing_widget.is_none() {
        add(EXPAND_CONTAINER);
    }
    if item.trailing_widget.is_some() {
        add(TRAILING_WIDGET_WIDTH);
    }
    if let Some(text) = &item.trailing_text {
        add(layout(ui, &styles.trailing, text, f32::INFINITY).size().x);
    }
    width
}

/// The horizontal gap between leading element and text.
fn title_gap(item: &ListItem<'_>) -> f32 {
    let density = item.visual_density.unwrap_or_default();
    (item.horizontal_title_gap.unwrap_or(BETWEEN_SPACE) + density.horizontal * 2.0).max(0.0)
}

/// Layout of one item at a given width.
struct ItemLayout {
    height: f32,
    text_width: f32,
    overline: Option<Arc<Galley>>,
    primary: Arc<Galley>,
    secondary: Option<Arc<Galley>>,
}

fn item_layout(ui: &Ui, item: &ListItem<'_>, width: f32, drawer: bool) -> ItemLayout {
    let styles = item_styles(item);
    let dense = item.dense.unwrap_or(false);
    let density_adjustment = item
        .visual_density
        .unwrap_or_default()
        .base_size_adjustment()
        .y;
    let leading = leading_size(item);
    let leading_width = leading.map_or(0.0, |s| s.x + title_gap(item));
    let text_width = (width
        - LEADING_SPACE
        - leading_width
        - trailing_width(ui, item, &styles, drawer)
        - TRAILING_SPACE)
        .max(1.0);

    let overline = item
        .overline_text
        .as_deref()
        .map(|t| layout(ui, &styles.overline, t, text_width));
    let primary = layout(ui, &styles.primary, &item.primary_text, text_width);
    let secondary = item
        .secondary_text
        .as_deref()
        .map(|t| layout(ui, &styles.secondary, t, text_width));

    // ListTokens.ItemOneLine/TwoLine/ThreeLineContainerHeight (dense: 8 dp
    // less, as in Compose's dense ListItem).
    let secondary_lines = secondary.as_ref().map_or(0, |g| g.rows.len().max(1));
    let text_lines = 1 + usize::from(overline.is_some()) + secondary_lines;
    let three_line = item.is_three_line.unwrap_or(false) || text_lines >= 3;
    let base = match (three_line, text_lines >= 2, dense) {
        (true, _, false) => 88.0,
        (true, _, true) => 76.0,
        (false, true, false) => 72.0,
        (false, true, true) => 64.0,
        (false, false, false) => 56.0,
        (false, false, true) => 48.0,
    };
    let block = overline.as_ref().map_or(0.0, |g| g.size().y)
        + primary.size().y
        + secondary.as_ref().map_or(0.0, |g| g.size().y);
    // Each text line beyond three adds one supporting line
    // (`TypeScaleTokens.BodyMediumLineHeight`, 20 dp) to the three-line height.
    let extra_lines = text_lines.saturating_sub(3) as f32 * 20.0;
    let base = base + extra_lines;
    let media = leading.map_or(0.0, |s| s.y) + 2.0 * MEDIA_VERTICAL_PADDING;
    let natural = (block + 2.0 * VERTICAL_SPACE).max(media);
    let height = match item.row_height {
        Some(exact) => exact,
        None => item
            .min_tile_height
            .unwrap_or(base + density_adjustment)
            .max(natural),
    };
    ItemLayout {
        height,
        text_width,
        overline,
        primary,
        secondary,
    }
}

/// The content width an item would like (for lists that do not fill the
/// available width).
fn item_natural_width(ui: &Ui, item: &ListItem<'_>, drawer: bool) -> f32 {
    let styles = item_styles(item);
    let text = |style: &TypeStyle, t: &str| layout(ui, style, t, f32::INFINITY).size().x;
    let text_width = text(&styles.primary, &item.primary_text)
        .max(item.secondary_text.as_deref().map_or(0.0, |t| {
            t.lines()
                .map(|line| text(&styles.secondary, line))
                .fold(0.0, f32::max)
        }))
        .max(
            item.overline_text
                .as_deref()
                .map_or(0.0, |t| text(&styles.overline, t)),
        );
    LEADING_SPACE
        + leading_size(item).map_or(0.0, |s| s.x + title_gap(item))
        + text_width
        + trailing_width(ui, item, &styles, drawer)
        + TRAILING_SPACE
}

/// Mirrors `rect` within `row` for right-to-left layouts.
fn mirror(rect: Rect, row: Rect, direction: LayoutDirection) -> Rect {
    match direction {
        LayoutDirection::Ltr => rect,
        LayoutDirection::Rtl => {
            let min_x = row.min.x + row.max.x - rect.max.x;
            Rect::from_min_size(Pos2::new(min_x, rect.min.y), rect.size())
        }
    }
}

/// The colours of one item.
struct ItemColors {
    container: Color32,
    label: Color32,
    supporting: Color32,
    icon: Color32,
}

fn item_colors(theme: &MaterialTheme, item: &ListItem<'_>, segmented: bool) -> ItemColors {
    let on_surface = theme.color(ColorRole::OnSurface);
    if !item.enabled {
        let disabled = on_surface.gamma_multiply(DISABLED_CONTENT_OPACITY);
        let container = if item.selected {
            // ListTokens.ItemSelectedDisabledContainerColor / Opacity.
            on_surface.gamma_multiply(DISABLED_CONTENT_OPACITY * 0.32)
        } else if segmented {
            theme.color(ColorRole::Surface)
        } else {
            item.tile_color.unwrap_or(Color32::TRANSPARENT)
        };
        return ItemColors {
            container,
            label: disabled,
            supporting: disabled,
            icon: disabled,
        };
    }
    if item.selected {
        let content = item
            .selected_color
            .unwrap_or_else(|| theme.color(ColorRole::OnSecondaryContainer));
        return ItemColors {
            container: item
                .selected_tile_color
                .unwrap_or_else(|| theme.color(ColorRole::SecondaryContainer)),
            label: content,
            supporting: content,
            icon: content,
        };
    }
    let container = match item.tile_color {
        Some(color) => color,
        None if segmented || item.dragged => theme.color(ColorRole::Surface),
        None => Color32::TRANSPARENT,
    };
    ItemColors {
        container,
        label: item.text_color.unwrap_or(on_surface),
        supporting: theme.color(ColorRole::OnSurfaceVariant),
        icon: item
            .icon_color
            .unwrap_or_else(|| theme.color(ColorRole::OnSurfaceVariant)),
    }
}

/// Corner radii of item `index` of `count`.
fn item_corners(selected: bool, segmented: bool, index: usize, count: usize) -> CornerRadius {
    if selected {
        return CornerRadius::same(CORNER_LARGE);
    }
    if !segmented {
        return CornerRadius::ZERO;
    }
    let top = if index == 0 {
        CORNER_LARGE
    } else {
        CORNER_EXTRA_SMALL
    };
    let bottom = if index + 1 == count {
        CORNER_LARGE
    } else {
        CORNER_EXTRA_SMALL
    };
    CornerRadius {
        nw: top,
        ne: top,
        sw: bottom,
        se: bottom,
    }
}

fn paint_checkbox(
    painter: &egui::Painter,
    rect: Rect,
    checked: bool,
    theme: &MaterialTheme,
    enabled: bool,
) {
    // CheckboxTokens: 18 dp container, 2 dp outline, CornerExtraSmall (2 dp).
    let box_rect = Rect::from_center_size(rect.center(), Vec2::splat(18.0));
    let fade = |c: Color32| {
        if enabled {
            c
        } else {
            c.gamma_multiply(DISABLED_CONTENT_OPACITY)
        }
    };
    if checked {
        painter.rect_filled(box_rect, 2.0, fade(theme.color(ColorRole::Primary)));
        crate::segmented_button::paint_check(
            painter,
            box_rect.shrink(1.0),
            fade(theme.color(ColorRole::OnPrimary)),
        );
    } else {
        painter.rect_stroke(
            box_rect,
            2.0,
            egui::Stroke::new(2.0, fade(theme.color(ColorRole::OnSurfaceVariant))),
            egui::StrokeKind::Inside,
        );
    }
}

fn paint_radio(
    painter: &egui::Painter,
    rect: Rect,
    selected: bool,
    theme: &MaterialTheme,
    enabled: bool,
) {
    // RadioButtonTokens: 20 dp icon, 2 dp ring, 10 dp dot.
    let color = if selected {
        theme.color(ColorRole::Primary)
    } else {
        theme.color(ColorRole::OnSurfaceVariant)
    };
    let color = if enabled {
        color
    } else {
        color.gamma_multiply(DISABLED_CONTENT_OPACITY)
    };
    painter.circle_stroke(rect.center(), 9.0, egui::Stroke::new(2.0, color));
    if selected {
        painter.circle_filled(rect.center(), 5.0, color);
    }
}

impl<'a> Widget for MaterialList<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

/// What every row of one list shares.
struct RowEnv<'t> {
    theme: &'t MaterialTheme,
    direction: LayoutDirection,
    segmented: bool,
    allow_drawer: bool,
    row_sense: Sense,
    list_id: Id,
}

/// The id of row `index` of the list `list_id`.
fn row_id(list_id: Id, index: usize) -> Id {
    list_id.with(("item", index))
}

/// Up and Down move focus from row `current` to the next enabled row of
/// `count`; returns the row focus moved to, if it moved.
fn rove(
    ui: &Ui,
    list_id: Id,
    current: usize,
    count: usize,
    mut enabled: impl FnMut(usize) -> bool,
) -> Option<usize> {
    ui.memory_mut(|m| {
        m.set_focus_lock_filter(
            row_id(list_id, current),
            EventFilter {
                vertical_arrows: true,
                ..Default::default()
            },
        );
    });
    let none = egui::Modifiers::NONE;
    let (down, up) = ui.input_mut(|i| {
        (
            i.consume_key(none, Key::ArrowDown),
            i.consume_key(none, Key::ArrowUp),
        )
    });
    let target = if down {
        (current.saturating_add(1)..count).find(|&i| enabled(i))
    } else if up {
        (0..current).rev().find(|&i| enabled(i))
    } else {
        None
    };
    if let Some(target) = target {
        ui.memory_mut(|m| m.request_focus(row_id(list_id, target)));
    }
    target
}

/// Where one row goes: its rect and its place among the list's rows.
#[derive(Clone, Copy)]
struct RowSlot {
    rect: Rect,
    index: usize,
    count: usize,
}

/// Lays out, paints and wires up one row (without its drawer content) in
/// `slot`, and returns its response. Click actions are queued in
/// `pending`; drawer toggles go to `state`.
fn show_row<'a>(
    ui: &mut Ui,
    env: &RowEnv<'_>,
    mut item: ListItem<'a>,
    layout: ItemLayout,
    slot: RowSlot,
    state: &mut MaterialListState,
    pending: &mut Vec<Box<dyn Fn() + 'a>>,
) -> Response {
    let RowSlot {
        rect: item_rect,
        index,
        count,
    } = slot;
    let theme = env.theme;
    let direction = env.direction;
    let segmented = env.segmented;
    let has_drawer = env.allow_drawer && item.drawer.is_some();
    let item_response = ui.interact(
        item_rect,
        row_id(env.list_id, index),
        crate::a11y::sense(item.enabled, env.row_sense),
    );
    match &item.leading {
        Some(ListLeading::Checkbox(checked)) => crate::a11y::toggle(
            &item_response,
            egui::Role::CheckBox,
            item.enabled,
            &item.primary_text,
            *checked,
        ),
        Some(ListLeading::Radio(selected)) => crate::a11y::toggle(
            &item_response,
            egui::Role::RadioButton,
            item.enabled,
            &item.primary_text,
            *selected,
        ),
        Some(ListLeading::Avatar(_) | ListLeading::Image(_) | ListLeading::Video { .. }) | None => {
            crate::a11y::toggle(
                &item_response,
                egui::Role::ListItem,
                item.enabled,
                &item.primary_text,
                item.selected,
            )
        }
    }

    let colors = item_colors(theme, &item, segmented);
    let corners = item
        .corner_radius
        .unwrap_or_else(|| item_corners(item.selected, segmented, index, count));
    let owned_painter = ui.painter().clone();
    let painter = &owned_painter;

    if item.dragged && item.enabled {
        crate::paint::paint_elevation(
            painter,
            item_rect,
            corners,
            Elevation::Level4,
            theme.color(ColorRole::Shadow),
        );
    }
    if colors.container != Color32::TRANSPARENT {
        painter.rect_filled(item_rect, corners, colors.container);
    }
    let mut interaction = InteractionState::from_response(&item_response);
    interaction.dragged |= item.dragged && item.enabled;
    let opacity = if interaction.dragged {
        DRAGGED_OPACITY
    } else {
        interaction.opacity()
    };
    if opacity > 0.0 {
        painter.rect_filled(item_rect, corners, colors.label.gamma_multiply(opacity));
    }
    if item_response.has_focus() {
        // Items touch their neighbours, so the ring goes inside.
        let ring =
            item_rect.shrink(crate::paint::FOCUS_RING_OFFSET + crate::paint::FOCUS_RING_WIDTH);
        crate::paint::paint_focus_ring(painter, ring, corners, theme.color(ColorRole::Secondary));
    }

    // Handle click - drawer toggle takes priority over action callback
    if item_response.clicked() && item.enabled {
        if has_drawer {
            if !state.drawer_open_items.remove(&index) {
                state.drawer_open_items.insert(index);
            }
        } else if let Some(action) = item.action.take() {
            pending.push(action);
        }
    }

    // Vertical anchor of leading/trailing elements: centred, or at
    // the top for three-line items (ListTileTitleAlignment::ThreeLine).
    let three_line = layout.height >= 88.0 - 0.5;
    let top_padding = item.min_vertical_padding.unwrap_or(VERTICAL_SPACE);
    let align_top = match item
        .title_alignment
        .unwrap_or(ListTileTitleAlignment::ThreeLine)
    {
        ListTileTitleAlignment::ThreeLine | ListTileTitleAlignment::TitleHeight => three_line,
        ListTileTitleAlignment::Top => true,
        ListTileTitleAlignment::Center | ListTileTitleAlignment::Bottom => false,
    };
    let align_bottom = item.title_alignment == Some(ListTileTitleAlignment::Bottom);
    let place_v = |height: f32| -> f32 {
        if align_top {
            item_rect.min.y + top_padding
        } else if align_bottom {
            item_rect.max.y - top_padding - height
        } else {
            item_rect.center().y - height / 2.0
        }
    };

    let custom_state = ListItemState {
        rect: item_rect,
        enabled: item.enabled,
        selected: item.selected,
        hovered: interaction.hovered,
        pressed: interaction.pressed,
        focused: item_response.has_focus(),
        label_color: colors.label,
        supporting_color: colors.supporting,
        icon_color: colors.icon,
    };
    if let Some(content_ui) = item.content_ui.take() {
        let content_rect = Rect::from_min_max(
            Pos2::new(item_rect.min.x + LEADING_SPACE, item_rect.min.y),
            Pos2::new(item_rect.max.x - TRAILING_SPACE, item_rect.max.y),
        );
        let layout_dir = match direction {
            LayoutDirection::Ltr => egui::Layout::left_to_right(egui::Align::Center),
            LayoutDirection::Rtl => egui::Layout::right_to_left(egui::Align::Center),
        };
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .id_salt(("list_item_content", index))
                .max_rect(content_rect)
                .layout(layout_dir),
        );
        child.set_clip_rect(item_rect.intersect(ui.clip_rect()));
        if !item.enabled {
            child.disable();
        }
        content_ui(&mut child, &custom_state);
    } else {
        // Leading.
        let mut content_x = item_rect.min.x + LEADING_SPACE;
        let leading_ui = item.leading_ui.take();
        if let Some((size, add_contents)) = leading_ui {
            let lead_rect = mirror(
                Rect::from_min_size(Pos2::new(content_x, place_v(size.y)), size),
                item_rect,
                direction,
            );
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt(("list_item_leading", index))
                    .max_rect(lead_rect)
                    .layout(egui::Layout::top_down(egui::Align::Center)),
            );
            child.set_clip_rect(lead_rect.intersect(ui.clip_rect()));
            if !item.enabled {
                child.disable();
            }
            add_contents(&mut child, &custom_state);
            content_x += size.x + title_gap(&item);
        } else if let Some(size) = leading_size(&item) {
            let lead_rect = mirror(
                Rect::from_min_size(Pos2::new(content_x, place_v(size.y)), size),
                item_rect,
                direction,
            );
            match &item.leading {
                Some(ListLeading::Avatar(text)) => {
                    // ListTokens.ItemLeadingAvatar*.
                    painter.circle_filled(
                        lead_rect.center(),
                        AVATAR_SIZE / 2.0,
                        theme.color(ColorRole::PrimaryContainer),
                    );
                    let galley = layout_text_ctx(ui, &TypeRole::TitleMedium.style(), text);
                    let at = lead_rect.center() - galley.size() / 2.0;
                    painter.galley(at, galley, theme.color(ColorRole::OnPrimaryContainer));
                }
                Some(ListLeading::Image(source)) => {
                    egui::Image::new(source.clone())
                        .corner_radius(0)
                        .paint_at(ui, lead_rect);
                }
                Some(ListLeading::Video { source, .. }) => {
                    egui::Image::new(source.clone())
                        .corner_radius(CORNER_SMALL)
                        .paint_at(ui, lead_rect);
                }
                Some(ListLeading::Checkbox(checked)) => {
                    paint_checkbox(painter, lead_rect, *checked, theme, item.enabled);
                }
                Some(ListLeading::Radio(selected)) => {
                    paint_radio(painter, lead_rect, *selected, theme, item.enabled);
                }
                None => {
                    if let Some(icon_name) = &item.leading_icon {
                        let (icon_center, label) = match &item.leading_icon_label {
                            Some(label) => (lead_rect.center() - Vec2::new(0.0, 12.0), Some(label)),
                            None => (lead_rect.center(), None),
                        };
                        painter.text(
                            icon_center,
                            egui::Align2::CENTER_CENTER,
                            material_symbol_text(icon_name),
                            egui::FontId::proportional(ICON_SIZE),
                            colors.icon,
                        );
                        if let Some(label) = label {
                            let galley = layout_text_ctx(ui, &TypeRole::LabelSmall.style(), label);
                            let at = Pos2::new(
                                lead_rect.center().x - galley.size().x / 2.0,
                                lead_rect.center().y + 4.0,
                            );
                            painter.galley(at, galley, colors.supporting);
                        }
                    }
                }
            }
            content_x += size.x + title_gap(&item);
        }

        // Text block, centred vertically.
        let block = layout.overline.as_ref().map_or(0.0, |g| g.size().y)
            + layout.primary.size().y
            + layout.secondary.as_ref().map_or(0.0, |g| g.size().y);
        let mut text_y = (item_rect.center().y - block / 2.0).max(item_rect.min.y);
        let text_clip = mirror(
            Rect::from_min_size(
                Pos2::new(content_x, item_rect.min.y),
                Vec2::new(layout.text_width, item_rect.height()),
            ),
            item_rect,
            direction,
        );
        let text_painter = painter.with_clip_rect(text_clip.intersect(painter.clip_rect()));
        let paint_galley = |galley: Arc<Galley>, color: Color32, y: &mut f32| {
            let size = galley.size();
            let at = mirror(
                Rect::from_min_size(Pos2::new(content_x, *y), size),
                item_rect,
                direction,
            );
            text_painter.galley(at.min, galley, color);
            *y += size.y;
        };
        if let Some(overline) = layout.overline {
            paint_galley(overline, colors.supporting, &mut text_y);
        }
        paint_galley(layout.primary, colors.label, &mut text_y);
        if let Some(secondary) = layout.secondary {
            paint_galley(secondary, colors.supporting, &mut text_y);
        }

        // Trailing elements, from the end inwards.
        let mut end_x = item_rect.max.x - TRAILING_SPACE;
        if let Some(icon_name) = &item.trailing_icon {
            let slot = mirror(
                Rect::from_min_size(
                    Pos2::new(end_x - ICON_SIZE, place_v(ICON_SIZE)),
                    Vec2::splat(ICON_SIZE),
                ),
                item_rect,
                direction,
            );
            painter.text(
                slot.center(),
                egui::Align2::CENTER_CENTER,
                material_symbol_text(icon_name),
                egui::FontId::proportional(ICON_SIZE),
                colors.icon,
            );
            end_x -= ICON_SIZE + BETWEEN_SPACE;
        } else if has_drawer && item.trailing_text.is_none() && item.trailing_widget.is_none() {
            let is_open = state.drawer_open_items.contains(&index);
            let slot = mirror(
                Rect::from_min_size(
                    Pos2::new(end_x - EXPAND_CONTAINER, place_v(EXPAND_CONTAINER)),
                    Vec2::splat(EXPAND_CONTAINER),
                ),
                item_rect,
                direction,
            );
            let chevron_response = ui.interact(
                slot,
                row_id(env.list_id, index).with("drawer_chevron"),
                Sense::click(),
            );
            crate::a11y::toggle(
                &chevron_response,
                egui::Role::Button,
                true,
                "Expand",
                is_open,
            );
            if chevron_response.clicked() && !state.drawer_open_items.remove(&index) {
                state.drawer_open_items.insert(index);
            }
            let is_open = state.drawer_open_items.contains(&index);
            // ExpandedListTokens.*ItemTrailingIconContainerColor / IconColor.
            let container = if is_open {
                theme.color(ColorRole::SurfaceContainer)
            } else {
                theme.color(ColorRole::Surface)
            };
            let painter = ui.painter();
            painter.circle_filled(slot.center(), EXPAND_CONTAINER / 2.0, container);
            painter.text(
                slot.center(),
                egui::Align2::CENTER_CENTER,
                material_symbol_text(if is_open {
                    "expand_less"
                } else {
                    "expand_more"
                }),
                egui::FontId::proportional(ICON_SIZE),
                if item.enabled {
                    theme.color(ColorRole::OnSurface)
                } else {
                    colors.icon
                },
            );
            end_x -= EXPAND_CONTAINER + BETWEEN_SPACE;
        }
        if let Some(widget_fn) = &item.trailing_widget {
            let widget_height = 32.0;
            let widget_rect = mirror(
                Rect::from_min_size(
                    Pos2::new(end_x - TRAILING_WIDGET_WIDTH, place_v(widget_height)),
                    Vec2::new(TRAILING_WIDGET_WIDTH, widget_height),
                ),
                item_rect,
                direction,
            );
            let layout_dir = match direction {
                LayoutDirection::Ltr => egui::Layout::right_to_left(egui::Align::Center),
                LayoutDirection::Rtl => egui::Layout::left_to_right(egui::Align::Center),
            };
            ui.scope_builder(egui::UiBuilder::new().max_rect(widget_rect), |ui| {
                ui.with_layout(layout_dir, |ui| {
                    widget_fn(ui);
                });
            });
            end_x -= TRAILING_WIDGET_WIDTH + BETWEEN_SPACE;
        }
        if let Some(trailing_text) = &item.trailing_text {
            // ListTokens.ItemTrailingSupportingTextFont / Color.
            let galley = layout_text_ctx(ui, &TypeRole::LabelSmall.style(), trailing_text);
            let size = galley.size();
            let at = mirror(
                Rect::from_min_size(Pos2::new(end_x - size.x, place_v(size.y)), size),
                item_rect,
                direction,
            );
            ui.painter().galley(at.min, galley, colors.supporting);
        }
    }

    item_response
}

impl<'a> MaterialList<'a> {
    /// Shows the list and returns its response and one response per row
    /// (see [`ListResponse`]). `ui.add(list)` shows it the same way and
    /// returns only the list's response.
    pub fn show(self, ui: &mut Ui) -> ListResponse {
        let theme = MaterialTheme::of(ui.ctx());
        let segmented = self.segmented;
        let dividers = self.dividers && !segmented;
        let gap = self.gap();
        let row_sense = self.effective_row_sense();

        // Load persistent drawer state BEFORE calculating total height
        let list_id = self
            .id
            .unwrap_or_else(|| ui.scope_id().with("material_list"));
        let state_id = list_id;
        let mut state = ui
            .data_mut(|d| d.get_persisted::<MaterialListState>(state_id))
            .unwrap_or_default();
        let count = self.items.len();

        // Keyboard: Up/Down move focus between enabled items.
        let focused = ui.memory(|m| m.focused());
        if let Some(current) = (0..count).find(|&i| focused == Some(row_id(list_id, i))) {
            let items = &self.items;
            rove(ui, list_id, current, count, |i| {
                items.get(i).is_some_and(|item| item.enabled)
            });
        }

        // Width.
        let list_width = if self.fill_width {
            ui.available_width()
        } else {
            self.items
                .iter()
                .map(|item| {
                    item_natural_width(ui, item, self.allow_drawer && item.drawer.is_some())
                })
                .fold(MIN_CONTENT_WIDTH, f32::max)
                .min(MAX_CONTENT_WIDTH)
                .min(ui.available_width())
        };

        // Heights.
        let layouts: Vec<ItemLayout> = self
            .items
            .iter()
            .map(|item| {
                item_layout(
                    ui,
                    item,
                    list_width,
                    self.allow_drawer && item.drawer.is_some(),
                )
            })
            .collect();
        let mut total_height: f32 = 0.0;
        for (index, (item, layout)) in self.items.iter().zip(&layouts).enumerate() {
            total_height += layout.height;
            if let Some(drawer_fn) = item
                .drawer
                .as_ref()
                .filter(|_| self.allow_drawer && state.drawer_open_items.contains(&index))
            {
                total_height += measure_drawer_height(&**drawer_fn, ui, list_width);
            }
        }
        if count > 1 {
            total_height += (count - 1) as f32 * gap;
        }

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(list_width, total_height), Sense::hover());
        if !segmented {
            // ListTokens.ItemContainerColor.
            ui.painter()
                .rect_filled(rect, 0.0, theme.color(ColorRole::Surface));
        }

        let env = RowEnv {
            theme: &theme,
            direction: theme.layout_direction,
            segmented,
            allow_drawer: self.allow_drawer,
            row_sense,
            list_id,
        };
        let mut current_y = rect.min.y;
        let mut pending_actions = Vec::new();
        let mut rows = Vec::with_capacity(count);

        for (index, (item, layout)) in self.items.into_iter().zip(layouts).enumerate() {
            let item_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, current_y),
                Vec2::new(rect.width(), layout.height),
            );
            current_y += layout.height;
            let drawer = item.drawer.clone();
            let slot = RowSlot {
                rect: item_rect,
                index,
                count,
            };
            rows.push(show_row(
                ui,
                &env,
                item,
                layout,
                slot,
                &mut state,
                &mut pending_actions,
            ));

            // Render expanded drawer content if open
            if let Some(drawer_fn) = drawer
                .as_ref()
                .filter(|_| self.allow_drawer && state.drawer_open_items.contains(&index))
            {
                let drawer_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x, current_y),
                    Vec2::new(rect.width(), 0.0),
                );
                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(drawer_rect)
                        .layout(egui::Layout::top_down(egui::Align::LEFT)),
                );
                child_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                child_ui.add_space(8.0);
                child_ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        drawer_fn(ui);
                    });
                });
                child_ui.add_space(8.0);
                current_y += child_ui.min_rect().height();
            }

            if index + 1 < count {
                if dividers {
                    paint_divider(ui, &theme, rect, current_y + (gap - 1.0) / 2.0);
                }
                current_y += gap;
            }
        }

        // Execute pending actions
        for action in pending_actions {
            action();
        }

        // Persist drawer state
        ui.data_mut(|d| d.insert_persisted(state_id, state));

        ListResponse {
            response,
            items: rows,
        }
    }

    /// The rows of a [`show_rows`](Self::show_rows) list that fall inside
    /// `viewport`: the vertical span shown, in points from the list's top.
    /// Rows are `row_height` apart, plus the 1 dp divider, the 2 dp
    /// segmented gap or the [`row_gap`](Self::row_gap) this list puts
    /// between them.
    ///
    /// ```rust
    /// use egui_material3::MaterialList;
    ///
    /// let list = MaterialList::new().dividers(false);
    /// let rows = list.visible_row_range(egui::Rangef::new(280.0, 480.0), 10_000, 28.0);
    /// assert_eq!(rows, 10..18);
    /// ```
    pub fn visible_row_range(
        &self,
        viewport: egui::Rangef,
        row_count: usize,
        row_height: f32,
    ) -> std::ops::Range<usize> {
        let stride = sanitize_row_height(row_height) + self.gap();
        let row_at = |y: f32| {
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
        let end = row_at(viewport.max).saturating_add(1).min(row_count);
        let end = if viewport.max <= viewport.min {
            first
        } else {
            end
        };
        first..end.max(first)
    }

    /// Shows `row_count` rows of `row_height` points but lays out only those
    /// inside `viewport`, for long lists in a [`egui::ScrollArea`]: pass the
    /// `ui` and viewport `ScrollArea::show_viewport` gives its closure.
    /// `build_row(index)` makes row `index`'s [`ListItem`]; it is called for
    /// the visible rows only (plus the focused row and the row the keyboard
    /// moves to, and while roving to find the next enabled row), so a row's
    /// state (selection, enabled) comes from the caller's data each frame.
    ///
    /// Each row is exactly `row_height` tall (as with
    /// [`ListItem::row_height`]), the list fills the available width, and
    /// rows keep everything [`show`](Self::show) gives them: responses (in
    /// [`ListRowsResponse::rows`]), selection colours, focus rings, Tab and
    /// Up/Down roving (the list scrolls to the row focus moves to), and an
    /// AccessKit node per built row with its position in the set of
    /// `row_count`. Items added with [`item`](Self::item) and drawers are
    /// not shown: a virtualised list needs uniform rows.
    ///
    /// ```rust
    /// # egui::__run_test_ui(|ui| {
    /// use egui_material3::{ListItem, MaterialList};
    ///
    /// let mut selected = Some(3);
    /// egui::ScrollArea::vertical().max_height(300.0).show_viewport(ui, |ui, viewport| {
    ///     let shown = MaterialList::new()
    ///         .id(egui::Id::unique("clips"))
    ///         .dividers(false)
    ///         .show_rows(ui, viewport, 100_000, 28.0, |index| {
    ///             ListItem::new(format!("clip-{index:05}.mp4"))
    ///                 .dense(true)
    ///                 .selected(selected == Some(index))
    ///         });
    ///     for (index, row) in &shown.rows {
    ///         if row.clicked() {
    ///             selected = Some(*index);
    ///         }
    ///     }
    /// });
    /// # });
    /// ```
    pub fn show_rows(
        self,
        ui: &mut Ui,
        viewport: Rect,
        row_count: usize,
        row_height: f32,
        mut build_row: impl FnMut(usize) -> ListItem<'a>,
    ) -> ListRowsResponse {
        let theme = MaterialTheme::of(ui.ctx());
        let segmented = self.segmented;
        let dividers = self.dividers && !segmented;
        let row_height = sanitize_row_height(row_height);
        let gap = self.gap();
        let row_sense = self.effective_row_sense();
        let stride = row_height + gap;
        let list_id = self
            .id
            .unwrap_or_else(|| ui.scope_id().with("material_list"));

        // The viewport is in the scroll content's coordinates, whose origin
        // is the top of the content ui; the list may start below it.
        let origin = ui.max_rect().min;
        let list_top = ui.cursor().min.y - origin.y;
        let visible = self.visible_row_range(
            egui::Rangef::new(viewport.min.y - list_top, viewport.max.y - list_top),
            row_count,
            row_height,
        );

        let total_height = if row_count == 0 {
            0.0
        } else {
            row_count as f32 * stride - gap
        };
        let width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, total_height), Sense::hover());
        let row_rect = |index: usize| {
            Rect::from_min_size(
                Pos2::new(rect.min.x, rect.min.y + index as f32 * stride),
                Vec2::new(width, row_height),
            )
        };

        // Keyboard roving. The focused row is remembered by index, so it is
        // found without walking every row, and is always built: egui drops
        // focus from a widget that is not shown.
        let focus_key = list_id.with("focused_row");
        let focused = ui
            .data(|d| d.get_temp::<usize>(focus_key))
            .filter(|&i| i < row_count && ui.memory(|m| m.has_focus(row_id(list_id, i))));
        let target = focused
            .and_then(|current| rove(ui, list_id, current, row_count, |i| build_row(i).enabled));

        let mut indices: Vec<usize> = visible.clone().collect();
        indices.extend(focused);
        indices.extend(target);
        indices.sort_unstable();
        indices.dedup();

        if !segmented {
            // ListTokens.ItemContainerColor, over the part on screen only.
            let painted = rect.intersect(ui.clip_rect());
            if painted.is_positive() {
                ui.painter()
                    .rect_filled(painted, 0.0, theme.color(ColorRole::Surface));
            }
        }

        let env = RowEnv {
            theme: &theme,
            direction: theme.layout_direction,
            segmented,
            allow_drawer: false,
            row_sense,
            list_id,
        };
        let mut state = MaterialListState::default();
        let mut pending_actions = Vec::new();
        let mut rows = Vec::with_capacity(indices.len());
        for index in indices {
            let mut item = build_row(index);
            item.row_height = Some(row_height);
            let layout = item_layout(ui, &item, width, false);
            let item_rect = row_rect(index);
            let slot = RowSlot {
                rect: item_rect,
                index,
                count: row_count,
            };
            let row = show_row(
                ui,
                &env,
                item,
                layout,
                slot,
                &mut state,
                &mut pending_actions,
            );
            ui.ctx().accesskit_node_builder(row.id, |node| {
                node.set_position_in_set(index.saturating_add(1));
                node.set_size_of_set(row_count);
            });
            if target == Some(index) {
                ui.scroll_to_rect(item_rect, None);
            }
            if row.has_focus() {
                ui.data_mut(|d| d.insert_temp(focus_key, index));
            }
            if dividers && index + 1 < row_count {
                paint_divider(ui, &theme, rect, item_rect.max.y + (gap - 1.0) / 2.0);
            }
            rows.push((index, row));
        }

        for action in pending_actions {
            action();
        }

        ListRowsResponse {
            response,
            visible,
            rows,
        }
    }
}

/// A usable row height: finite and at least one point (else 56 dp,
/// `ListTokens.ItemOneLineContainerHeight`).
fn sanitize_row_height(height: f32) -> f32 {
    if height.is_finite() {
        height.max(1.0)
    } else {
        56.0
    }
}

impl MaterialList<'_> {
    /// The space between two rows: the 2 dp segmented gap, else the
    /// [`row_gap`](Self::row_gap) (at least the 1 dp divider when dividers
    /// show), else the divider or none.
    fn gap(&self) -> f32 {
        if self.segmented {
            return SEGMENTED_GAP;
        }
        let custom = self
            .row_gap
            .filter(|gap| gap.is_finite())
            .map_or(0.0, |gap| gap.max(0.0));
        if self.dividers {
            // DividerTokens.Thickness.
            custom.max(1.0)
        } else {
            custom
        }
    }

    /// What every row senses: the click sense plus `row_sense`, or exactly
    /// `row_sense`.
    fn effective_row_sense(&self) -> Sense {
        if self.row_sense_exact {
            self.row_sense
        } else {
            Sense::click() | self.row_sense
        }
    }
}

/// Paints the divider under the row ending at `y`, inset like
/// `ListTokens.DividerLeadingSpace` / `DividerTrailingSpace` in
/// `DividerTokens.Color`.
fn paint_divider(ui: &Ui, theme: &MaterialTheme, list: Rect, y: f32) {
    let line = Rect::from_min_max(
        Pos2::new(list.min.x + 16.0, y),
        Pos2::new(list.max.x - 16.0, y + 1.0),
    );
    ui.painter()
        .rect_filled(line, 0.0, theme.color(ColorRole::OutlineVariant));
}

fn layout_text_ctx(ui: &Ui, style: &TypeStyle, text: &str) -> Arc<Galley> {
    layout(ui, style, text, f32::INFINITY)
}

/// Create a new list item with the given primary text.
pub fn list_item(primary_text: impl Into<String>) -> ListItem<'static> {
    ListItem::new(primary_text)
}

/// Create a new, empty Material Design list.
pub fn list() -> MaterialList<'static> {
    MaterialList::new()
}
