// ============================================================================
// Module:       select
// Description:  Material Design 3 exposed dropdown menu and autocomplete.
//
// Dependencies: egui; crate::input, crate::tokens, crate::MaterialTheme
// ============================================================================

//! Material Design 3 exposed dropdown menu (select and autocomplete).
//!
//! The field is a text field ([`crate::input`] draws both, from the same
//! tokens); the menu follows Compose's `MenuTokens`:
//!
//! - **Menu**: `surfaceContainer`, extra-small corners (4 dp), elevation
//!   level 2, 8 dp vertical padding, 48 dp items with 12 dp horizontal
//!   padding and LabelLarge text in `onSurface`.
//! - **Items**: hover and keyboard highlight paint `onSurface` state layers
//!   (8% and 10%); the selected item sits on `secondaryContainer`.
//! - **Trailing icon**: a 24 dp drop-down arrow that turns half a turn as
//!   the menu opens.
//!
//! The menu opens on click, Space, Enter or the arrow keys, moves with
//! Up/Down/Home/End, selects with Enter and closes with Escape or a click
//! outside it. Typing jumps to the first option starting with the typed
//! text (typeahead). With [`MaterialSelect::enable_filter`] the field is an
//! editable text field and the menu shows only the options that contain the
//! typed text (autocomplete). [`MaterialSelect::editable`] binds the field
//! to a `String` instead: the options are suggestions and any text is
//! kept. An option's description ([`SelectOption::description`]) is its
//! row's tooltip and accessible description, and with
//! [`MaterialSelect::inline_descriptions`] a second line under its text.
//! Screen readers see a combo box with its
//! value, its expanded state and list-box options.

use crate::input::{
    container_shape, field_colors, floated_label_pos, label_progress, notch_for, paint_icon,
    paint_label, paint_supporting, FieldState, InputVariant, LabelGeometry, HORIZONTAL_PADDING,
    ICON_PADDING, ICON_SIZE, TEXT_PADDING_WITH_ICON,
};
use crate::paint::{paint_elevation, paint_state_layer};
use crate::tokens::state::{FOCUS_OPACITY, HOVER_OPACITY};
use crate::tokens::{ColorRole, Elevation, TypeRole};
use crate::MaterialTheme;
use egui::{
    self, Align2, Color32, EventFilter, Id, Key, Modifiers, Pos2, Rect, Response, RichText, Sense,
    Ui, Vec2, Widget,
};

/// `MenuTokens.ContainerElevation`.
pub const MENU_ELEVATION: Elevation = Elevation::Level2;
/// `MenuTokens.ContainerShape`: extra-small corners.
pub const MENU_CORNER_RADIUS: f32 = 4.0;
/// `DropdownMenuVerticalPadding`: space above the first and below the last
/// item.
pub const MENU_VERTICAL_PADDING: f32 = 8.0;
/// `MenuListItemContainerHeight`.
pub const MENU_ITEM_HEIGHT: f32 = 48.0;
/// `DropdownMenuItemHorizontalPadding`.
pub const MENU_ITEM_HORIZONTAL_PADDING: f32 = 12.0;
/// Typed characters within this many seconds of each other build one
/// typeahead search.
const TYPEAHEAD_RESET_SECONDS: f64 = 1.0;
/// The menu keeps this far from the window edges.
const MENU_SCREEN_MARGIN: f32 = 8.0;

/// Material Design select/dropdown component.
///
/// Select components allow users to choose one option from a list.
/// They display the currently selected option in a text field-style input
/// and show all options in a dropdown menu when activated.
///
/// Supports Material Design 3 variants (filled and outlined), filtering,
/// validation, and keyboard navigation.
///
/// # Example
/// ```rust
/// # use egui_material3::{MaterialSelect, SelectVariant};
/// # egui::__run_test_ui(|ui| {
/// let mut selected = Some(1);
///
/// ui.add(MaterialSelect::new(&mut selected)
///     .variant(SelectVariant::Outlined)
///     .label("Choose an option")
///     .option(0, "Option 1")
///     .option(1, "Option 2")
///     .option(2, "Option 3")
///     .helper_text("Select your preferred option"));
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct MaterialSelect<'a> {
    binding: Binding<'a>,
    options: Vec<SelectOption>,
    placeholder: String,
    label: Option<String>,
    variant: SelectVariant,
    enabled: bool,
    width: Option<f32>,
    error_text: Option<String>,
    helper_text: Option<String>,
    leading_icon: Option<String>,
    trailing_icon: Option<String>,
    keep_open_on_select: bool,
    enable_filter: bool,
    enable_search: bool,
    required: bool,
    menu_width: Option<f32>,
    menu_max_height: Option<f32>,
    border_radius: Option<f32>,
    menu_alignment: MenuAlignment,
    compact: bool,
    fill_width: bool,
    inline_descriptions: bool,
    id_salt: Option<egui::IdSalt>,
}

/// What a select edits: the chosen option's value, or (editable) free text.
enum Binding<'a> {
    Value(&'a mut Option<usize>),
    Text(&'a mut String),
}

/// Visual variant of the select component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectVariant {
    /// Filled variant with background color
    #[default]
    Filled,
    /// Outlined variant with border
    Outlined,
}

impl From<SelectVariant> for InputVariant {
    fn from(variant: SelectVariant) -> Self {
        match variant {
            SelectVariant::Filled => InputVariant::Filled,
            SelectVariant::Outlined => InputVariant::Outlined,
        }
    }
}

/// Menu alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuAlignment {
    /// Align menu to start edge
    #[default]
    Start,
    /// Align menu to end edge
    End,
}

/// Individual option in a select component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    value: usize,
    text: String,
    description: Option<String>,
}

impl SelectOption {
    /// An option: `value` is what the select's binding is set to, `text` is
    /// shown.
    pub fn new(value: usize, text: impl Into<String>) -> Self {
        Self {
            value,
            text: text.into(),
            description: None,
        }
    }

    /// A description: shown as a tooltip when the pointer rests on the
    /// option, as a second line with [`MaterialSelect::inline_descriptions`],
    /// and as the option's accessible description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The value the select is set to when this option is picked.
    pub fn value(&self) -> usize {
        self.value
    }

    /// The text shown for this option.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The description, if any.
    pub fn description_text(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// What a select remembers between frames.
#[derive(Clone, Debug, Default)]
struct SelectState {
    open: bool,
    /// The keyboard-highlighted option, as an index into `options`.
    highlighted: Option<usize>,
    /// The text in the editable field (filter mode).
    filter: String,
    /// The user has typed since the menu opened, so the menu filters.
    filtering: bool,
    typeahead: String,
    typeahead_time: f64,
    /// Where the menu was drawn last frame, for click-outside.
    menu_rect: Option<Rect>,
    /// The highlight moved by keyboard: scroll it into view.
    scroll_to_highlight: bool,
}

impl<'a> MaterialSelect<'a> {
    /// Create a new select component.
    ///
    /// # Arguments
    /// * `selected` - Mutable reference to the currently selected option value
    pub fn new(selected: &'a mut Option<usize>) -> Self {
        Self::with_binding(Binding::Value(selected))
    }

    /// An editable exposed dropdown bound to free text: the field is a text
    /// field, the menu suggests the options containing the typed text, and
    /// picking one puts its text in `text`. Any text is allowed; it does not
    /// have to match an option.
    ///
    /// ```rust
    /// # use egui_material3::MaterialSelect;
    /// # egui::__run_test_ui(|ui| {
    /// let mut codec = String::from("h264");
    /// ui.add(MaterialSelect::editable(&mut codec)
    ///     .label("Codec")
    ///     .option(0, "h264")
    ///     .option(1, "hevc")
    ///     .option(2, "prores"));
    /// # });
    /// ```
    pub fn editable(text: &'a mut String) -> Self {
        let mut select = Self::with_binding(Binding::Text(text));
        select.enable_filter = true;
        select.placeholder = String::new();
        select
    }

    fn with_binding(binding: Binding<'a>) -> Self {
        Self {
            binding,
            options: Vec::new(),
            placeholder: "Select an option".to_string(),
            label: None,
            variant: SelectVariant::default(),
            enabled: true,
            width: None,
            error_text: None,
            helper_text: None,
            leading_icon: None,
            trailing_icon: None,
            keep_open_on_select: false,
            enable_filter: false,
            enable_search: true,
            required: false,
            menu_width: None,
            menu_max_height: None,
            border_radius: None,
            menu_alignment: MenuAlignment::default(),
            compact: false,
            fill_width: false,
            inline_descriptions: false,
            id_salt: None,
        }
    }

    /// Add an option: `value` is what `selected` is set to, `text` is shown.
    pub fn option(mut self, value: usize, text: impl Into<String>) -> Self {
        self.options.push(SelectOption::new(value, text));
        self
    }

    /// Add an option with a description, shown as a tooltip on its menu row
    /// (and as a second line with [`Self::inline_descriptions`]).
    pub fn option_with_description(
        mut self,
        value: usize,
        text: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.options
            .push(SelectOption::new(value, text).description(description));
        self
    }

    /// Add a prepared [`SelectOption`].
    pub fn option_item(mut self, option: SelectOption) -> Self {
        self.options.push(option);
        self
    }

    /// Paint option descriptions as a second line under the option text
    /// (rows grow from 48 dp by one BodySmall line), not only as tooltips.
    /// Options without a description keep one-line rows.
    pub fn inline_descriptions(mut self, inline: bool) -> Self {
        self.inline_descriptions = inline;
        self
    }

    /// Fill the available width instead of the fixed width (default 200).
    pub fn fill_width(mut self, fill: bool) -> Self {
        self.fill_width = fill;
        self
    }

    /// Set placeholder text shown when no option is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set label text that floats above the field when focused or has content.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the visual variant of the select component.
    pub fn variant(mut self, variant: SelectVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Enable or disable the select component.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set a fixed width for the select component (default 200).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set error text to display below the select; the field shows its
    /// error colours while it is set.
    pub fn error_text(mut self, text: impl Into<String>) -> Self {
        self.error_text = Some(text.into());
        self
    }

    /// Set helper text to display below the select component.
    pub fn helper_text(mut self, text: impl Into<String>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// Set an icon to display at the start of the select field.
    pub fn leading_icon(mut self, icon: impl Into<String>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// Set an icon to display at the end of the select field, in place of
    /// the drop-down arrow.
    pub fn trailing_icon(mut self, icon: impl Into<String>) -> Self {
        self.trailing_icon = Some(icon.into());
        self
    }

    /// Keep the menu open after an option is selected.
    pub fn keep_open_on_select(mut self, keep_open: bool) -> Self {
        self.keep_open_on_select = keep_open;
        self
    }

    /// Make the field editable: the menu shows only the options containing
    /// the typed text, and Enter picks the highlighted (first) match. The
    /// text reverts to the selected option when the field loses focus; for
    /// free text use [`Self::editable`].
    pub fn enable_filter(mut self, enable: bool) -> Self {
        self.enable_filter = enable;
        self
    }

    /// Typeahead: typing while the (non-editable) field is focused jumps to
    /// the first option starting with the typed text. On by default.
    pub fn enable_search(mut self, enable: bool) -> Self {
        self.enable_search = enable;
        self
    }

    /// Mark the field as required (reported to screen readers).
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set independent menu width.
    pub fn menu_width(mut self, width: f32) -> Self {
        self.menu_width = Some(width);
        self
    }

    /// Set maximum menu height.
    pub fn menu_max_height(mut self, height: f32) -> Self {
        self.menu_max_height = Some(height);
        self
    }

    /// Override the menu corner radius (default 4, `MenuTokens.ContainerShape`).
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }

    /// Set menu alignment.
    pub fn menu_alignment(mut self, alignment: MenuAlignment) -> Self {
        self.menu_alignment = alignment;
        self
    }

    /// Compact mode: a 32 dp field without a floating label, for tables and
    /// toolbars.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Derive the select's ids from `salt` instead of its position in the
    /// layout.
    pub fn id_salt(mut self, salt: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(egui::IdSalt::new(salt));
        self
    }
}

/// Indices into `options` of the options to show for `state`.
fn visible_options(options: &[SelectOption], state: &SelectState) -> Vec<usize> {
    if !state.filtering || state.filter.is_empty() {
        return (0..options.len()).collect();
    }
    let needle = state.filter.to_lowercase();
    options
        .iter()
        .enumerate()
        .filter(|(_, option)| option.text.to_lowercase().contains(&needle))
        .map(|(i, _)| i)
        .collect()
}

/// The first option whose text starts with `prefix`, ignoring case.
fn typeahead_match(options: &[SelectOption], prefix: &str) -> Option<usize> {
    let prefix = prefix.to_lowercase();
    if prefix.is_empty() {
        return None;
    }
    options
        .iter()
        .position(|option| option.text.to_lowercase().starts_with(&prefix))
}

/// Moves `highlighted` by `step` within `visible` (clamped at the ends).
fn step_highlight(visible: &[usize], highlighted: Option<usize>, step: isize) -> Option<usize> {
    let last = visible.len().checked_sub(1)?;
    let current = highlighted.and_then(|h| visible.iter().position(|&v| v == h));
    let next = match current {
        None if step >= 0 => 0,
        None => last,
        Some(i) => i.saturating_add_signed(step).min(last),
    };
    visible.get(next).copied()
}

/// The height of an option's menu row: 48 dp (`MenuListItemContainerHeight`),
/// plus a BodySmall line when its description is painted inline.
fn item_height(option: &SelectOption, inline_descriptions: bool) -> f32 {
    if inline_descriptions && option.description.is_some() {
        MENU_ITEM_HEIGHT + TypeRole::BodySmall.style().line_height
    } else {
        MENU_ITEM_HEIGHT
    }
}

/// The id remembering which select is open, so opening one closes others.
fn open_select_id() -> Id {
    Id::unique("egui_material3::open_select")
}

impl Widget for MaterialSelect<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let ctx = ui.ctx().clone();
        let theme = MaterialTheme::of(&ctx);
        let enabled = self.enabled && ui.is_enabled();
        let variant = InputVariant::from(self.variant);
        let is_error = self.error_text.is_some();
        let body = TypeRole::BodyLarge.style();
        let font = body.font_id(&ctx);

        let text_mode = matches!(self.binding, Binding::Text(_));
        let enable_filter = self.enable_filter || text_mode;
        // The chosen option's value; editable text chooses the option it
        // spells exactly.
        let selected_value: Option<usize> = match &self.binding {
            Binding::Value(selected) => **selected,
            Binding::Text(text) => self
                .options
                .iter()
                .find(|o| o.text == **text)
                .map(|o| o.value),
        };
        let width = if self.fill_width {
            ui.available_width()
        } else {
            self.width.unwrap_or(200.0)
        };
        let height = if self.compact { 32.0 } else { 56.0 };
        let supporting = self.error_text.as_deref().or(self.helper_text.as_deref());
        let supporting_height = if supporting.is_some() {
            4.0 + TypeRole::BodySmall.style().line_height
        } else {
            0.0
        };
        let (total_rect, allocated) =
            ui.allocate_exact_size(Vec2::new(width, height + supporting_height), Sense::hover());
        let id = self
            .id_salt
            .map_or(allocated.id, |salt| ui.make_persistent_id(salt))
            .with("select_widget");
        let edit_id = id.with("filter");
        let rect = Rect::from_min_size(total_rect.min, Vec2::new(width, height));
        let mut response = ui.interact(
            rect,
            id,
            crate::a11y::sense(
                enabled,
                if enable_filter {
                    Sense::CLICK
                } else {
                    Sense::click()
                },
            ),
        );

        let mut state = ctx
            .data(|d| d.get_temp::<SelectState>(id))
            .unwrap_or_default();
        let selected_index = |selected: &Option<usize>| {
            selected.and_then(|value| self.options.iter().position(|o| o.value == value))
        };
        let selected_text = |selected: &Option<usize>| {
            selected_index(selected)
                .and_then(|i| self.options.get(i))
                .map(|o| o.text.clone())
        };

        // Only one select is open at a time.
        let open_elsewhere = ctx
            .data(|d| d.get_temp::<Id>(open_select_id()))
            .is_some_and(|open| open != id);
        if state.open && open_elsewhere {
            state.open = false;
        }
        if !enabled {
            state.open = false;
        }

        let focus_id = if enable_filter { edit_id } else { id };
        let focused = ctx.memory(|m| m.has_focus(focus_id));
        let open_menu = |state: &mut SelectState, selected: &Option<usize>| {
            if !state.open {
                state.open = true;
                state.filtering = false;
                state.highlighted = selected_index(selected)
                    .or(Some(0))
                    .filter(|_| !self.options.is_empty());
                state.scroll_to_highlight = true;
            }
        };

        // Pointer clicks only: Enter and Space (which egui also reports as
        // clicks on a focused widget) are handled with the other keys below.
        if response.clicked_by(egui::PointerButton::Primary) {
            if state.open {
                state.open = false;
            } else {
                open_menu(&mut state, &selected_value);
            }
            ctx.memory_mut(|m| m.request_focus(focus_id));
        }

        // Keyboard, while the field has focus. Arrow keys and Escape are
        // taken before egui's focus navigation and the text edit see them.
        let mut picked: Option<usize> = None;
        if focused && enabled {
            let filter = EventFilter {
                tab: false,
                horizontal_arrows: enable_filter,
                vertical_arrows: true,
                escape: true,
            };
            ctx.memory_mut(|m| m.set_focus_lock_filter(focus_id, filter));
            let pressed = |key| ui.input_mut(|i| i.consume_key(Modifiers::NONE, key));
            let visible = visible_options(&self.options, &state);
            if pressed(Key::ArrowDown) {
                if state.open {
                    state.highlighted = step_highlight(&visible, state.highlighted, 1);
                } else {
                    open_menu(&mut state, &selected_value);
                }
                state.scroll_to_highlight = true;
            }
            if pressed(Key::ArrowUp) {
                if state.open {
                    state.highlighted = step_highlight(&visible, state.highlighted, -1);
                } else {
                    open_menu(&mut state, &selected_value);
                }
                state.scroll_to_highlight = true;
            }
            if state.open && pressed(Key::Home) {
                state.highlighted = visible.first().copied();
                state.scroll_to_highlight = true;
            }
            if state.open && pressed(Key::End) {
                state.highlighted = visible.last().copied();
                state.scroll_to_highlight = true;
            }
            if state.open && pressed(Key::Escape) {
                state.open = false;
            }
            let activate = pressed(Key::Enter) || (!enable_filter && pressed(Key::Space));
            if activate {
                if state.open {
                    picked = state.highlighted.filter(|h| visible.contains(h));
                    if picked.is_none() {
                        state.open = false;
                    }
                } else if !enable_filter {
                    open_menu(&mut state, &selected_value);
                }
            }
            if !enable_filter && self.enable_search {
                #[expect(
                    clippy::wildcard_enum_match_arm,
                    reason = "typeahead reads typed text only; every other event is ignored"
                )]
                let typed: String = ui.input(|i| {
                    i.events
                        .iter()
                        .filter_map(|e| match e {
                            egui::Event::Text(t) => Some(t.as_str()),
                            _ => None,
                        })
                        .collect()
                });
                if !typed.trim().is_empty() {
                    let now = ui.input(|i| i.time);
                    if now - state.typeahead_time > TYPEAHEAD_RESET_SECONDS {
                        state.typeahead.clear();
                    }
                    state.typeahead.push_str(&typed);
                    state.typeahead_time = now;
                    if let Some(i) = typeahead_match(&self.options, &state.typeahead) {
                        open_menu(&mut state, &selected_value);
                        state.highlighted = Some(i);
                        state.scroll_to_highlight = true;
                    }
                }
            }
        }

        // Focus moved to another widget (Tab): close.
        let focus_now = ctx.memory(|m| m.focused());
        if state.open && focus_now.is_some_and(|f| f != focus_id) {
            state.open = false;
        }

        // A press outside the field and the menu closes it.
        if state.open {
            let press = ui.input(|i| {
                if i.pointer.any_pressed() {
                    i.pointer.press_origin()
                } else {
                    None
                }
            });
            if let Some(pos) = press {
                let in_menu = state.menu_rect.is_some_and(|r| r.contains(pos));
                if !rect.contains(pos) && !in_menu {
                    state.open = false;
                }
            }
        }

        // Colours and layout of the field.
        let hovered = enabled && response.hovered();
        let colors = field_colors(
            &theme,
            variant,
            FieldState {
                enabled,
                error: is_error,
                focused: focused || state.open,
                hovered,
            },
        );
        let text_start = if self.leading_icon.is_some() {
            TEXT_PADDING_WITH_ICON
        } else {
            HORIZONTAL_PADDING
        };
        let text_end = TEXT_PADDING_WITH_ICON;
        let container_slot = ui.painter().add(egui::Shape::Noop);
        let has_label = self.label.is_some() && !self.compact;
        let input_top = if variant == InputVariant::Filled && has_label {
            rect.top() + 24.0
        } else {
            rect.center().y - body.line_height / 2.0
        };
        let input_rect = Rect::from_min_size(
            Pos2::new(rect.left() + text_start, input_top),
            Vec2::new((width - text_start - text_end).max(1.0), body.line_height),
        );

        // The field's text: an editable filter, or the selected option.
        let mut edit_response = None;
        if enable_filter {
            if !(focused || state.open) {
                state.filter = selected_text(&selected_value).unwrap_or_default();
                state.filtering = false;
            }
            if let Binding::Text(text) = &self.binding {
                // Editable: the bound text is the field's text.
                state.filter.clone_from(text);
            }
            let show_placeholder = !has_label || focused;
            let mut filter = std::mem::take(&mut state.filter);
            let edit = {
                // A child that does not allocate: the field already holds its space.
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(input_rect));
                let ui = &mut child;
                ui.visuals_mut().text_cursor.stroke.color = colors.caret;
                let mut edit = egui::TextEdit::singleline(&mut filter)
                    .id(edit_id)
                    .font(font.clone())
                    .text_color(colors.input)
                    .frame(egui::Frame::NONE)
                    .margin(egui::Margin::ZERO)
                    .desired_width(input_rect.width())
                    .min_size(input_rect.size())
                    .align(Align2::LEFT_CENTER)
                    .interactive(enabled);
                if show_placeholder {
                    edit = edit.hint_text(
                        RichText::new(&self.placeholder)
                            .font(font.clone())
                            .color(colors.placeholder),
                    );
                }
                ui.add(edit)
            };
            state.filter = filter;
            if let (true, Binding::Text(text)) = (edit.changed(), &mut self.binding) {
                text.clone_from(&state.filter);
            }
            if edit.changed() {
                open_menu(&mut state, &selected_value);
                state.filtering = true;
                let visible = visible_options(&self.options, &state);
                state.highlighted = visible.first().copied();
                state.scroll_to_highlight = true;
            }
            if edit.clicked() && !state.open {
                open_menu(&mut state, &selected_value);
            }
            edit_response = Some(edit);
        } else {
            let display = selected_text(&selected_value);
            let show_placeholder = !has_label || state.open || focused;
            let (text, color) = match &display {
                Some(text) => (Some(text.as_str()), colors.input),
                None if show_placeholder => (Some(self.placeholder.as_str()), colors.placeholder),
                None => (None, colors.input),
            };
            if let Some(text) = text {
                ui.painter()
                    .with_clip_rect(input_rect.intersect(ui.clip_rect()))
                    .text(
                        Pos2::new(input_rect.left(), input_rect.center().y),
                        Align2::LEFT_CENTER,
                        text,
                        font.clone(),
                        color,
                    );
            }
        }

        let painter = ui.painter().clone();
        let has_content = if enable_filter {
            !state.filter.is_empty()
        } else {
            selected_value.is_some()
        };
        let floated = has_content || focused || state.open;
        let progress = if has_label {
            label_progress(&ctx, id.with("label"), floated)
        } else {
            0.0
        };
        let label_geometry = LabelGeometry {
            rest: Pos2::new(rect.left() + text_start, rect.center().y),
            floated: floated_label_pos(variant, rect, text_start),
        };
        let notch = match (&self.label, variant) {
            (Some(label), InputVariant::Outlined) if has_label => {
                notch_for(&painter, &ctx, label, label_geometry.floated, progress)
            }
            _ => None,
        };
        painter.set(
            container_slot,
            container_shape(rect, variant, &colors, notch),
        );
        if let Some(label) = self.label.as_ref().filter(|_| has_label) {
            paint_label(
                &painter,
                &ctx,
                label,
                label_geometry,
                progress,
                colors.label,
            );
        }
        if let Some(icon) = &self.leading_icon {
            let center = Pos2::new(
                rect.left() + ICON_PADDING + ICON_SIZE / 2.0,
                rect.center().y,
            );
            paint_icon(&painter, center, icon, colors.leading_icon);
        }
        let trailing_center = Pos2::new(
            rect.right() - ICON_PADDING - ICON_SIZE / 2.0,
            rect.center().y,
        );
        // In filter mode the arrow is the way to open the full list.
        if enable_filter && enabled {
            let arrow = ui.interact(
                Rect::from_center_size(trailing_center, Vec2::splat(40.0)),
                id.with("arrow"),
                Sense::CLICK,
            );
            if arrow.clicked() {
                if state.open {
                    state.open = false;
                } else {
                    open_menu(&mut state, &selected_value);
                    state.filtering = false;
                }
                ctx.memory_mut(|m| m.request_focus(edit_id));
            }
        }
        match &self.trailing_icon {
            Some(icon) => paint_icon(&painter, trailing_center, icon, colors.trailing_icon),
            None => {
                let turn = ctx.animate_bool_with_time(
                    id.with("arrow_turn"),
                    state.open,
                    crate::input::LABEL_ANIMATION_SECONDS,
                );
                paint_drop_down_arrow(&painter, trailing_center, turn, colors.trailing_icon);
            }
        }
        if let Some(text) = supporting {
            let row = Rect::from_min_max(
                Pos2::new(total_rect.left(), rect.bottom() + 4.0),
                total_rect.max,
            );
            paint_supporting(&painter, &ctx, row, Some(text), None, colors.supporting);
        }

        // The menu.
        let visible = visible_options(&self.options, &state);
        state.menu_rect = None;
        if state.open && !visible.is_empty() {
            let menu = MenuLayout::new(
                &ctx,
                rect,
                visible
                    .iter()
                    .filter_map(|&i| self.options.get(i))
                    .map(|o| item_height(o, self.inline_descriptions))
                    .sum(),
                self.menu_width.unwrap_or(width),
                self.menu_max_height,
                self.menu_alignment,
            );
            state.menu_rect = Some(menu.rect);
            let radius = self.border_radius.unwrap_or(MENU_CORNER_RADIUS);
            let label_font = TypeRole::LabelLarge.style().font_id(&ctx);
            let description_font = TypeRole::BodySmall.style().font_id(&ctx);
            let on_surface_variant = theme.color(ColorRole::OnSurfaceVariant);
            let on_surface = theme.color(ColorRole::OnSurface);
            let scroll_to_highlight = std::mem::take(&mut state.scroll_to_highlight);
            let selected_now = selected_index(&selected_value);
            egui::Area::new(id.with("menu"))
                .order(egui::Order::Foreground)
                .fixed_pos(menu.rect.min)
                .constrain(false)
                .show(&ctx, |ui| {
                    let painter = ui.painter();
                    paint_elevation(
                        painter,
                        menu.rect,
                        radius,
                        MENU_ELEVATION,
                        theme.color(ColorRole::Shadow),
                    );
                    painter.rect_filled(
                        menu.rect,
                        radius,
                        theme.color(ColorRole::SurfaceContainer),
                    );
                    let list_rect = menu.rect.shrink2(Vec2::new(0.0, MENU_VERTICAL_PADDING));
                    ui.scope_builder(egui::UiBuilder::new().max_rect(list_rect), |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        egui::ScrollArea::vertical()
                            .id_salt(id.with("menu_scroll"))
                            .max_height(list_rect.height())
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                for &index in &visible {
                                    let Some(option) = self.options.get(index) else {
                                        continue;
                                    };
                                    let (item_rect, item) = ui.allocate_exact_size(
                                        Vec2::new(
                                            list_rect.width(),
                                            item_height(option, self.inline_descriptions),
                                        ),
                                        Sense::CLICK,
                                    );
                                    let item = match &option.description {
                                        Some(description) => {
                                            let text = description.clone();
                                            ctx.accesskit_node_builder(item.id, |node| {
                                                node.set_description(text);
                                            });
                                            item.on_hover_text(description)
                                        }
                                        None => item,
                                    };
                                    let is_selected = selected_now == Some(index);
                                    let is_highlighted = state.highlighted == Some(index);
                                    crate::a11y::toggle(
                                        &item,
                                        egui::Role::ListBoxOption,
                                        true,
                                        &option.text,
                                        is_selected,
                                    );
                                    let text_color = if is_selected {
                                        ui.painter().rect_filled(
                                            item_rect,
                                            0.0,
                                            theme.color(ColorRole::SecondaryContainer),
                                        );
                                        theme.color(ColorRole::OnSecondaryContainer)
                                    } else {
                                        on_surface
                                    };
                                    let opacity = if is_highlighted {
                                        FOCUS_OPACITY
                                    } else if item.hovered() {
                                        HOVER_OPACITY
                                    } else {
                                        0.0
                                    };
                                    if opacity > 0.0 {
                                        let layer = crate::tokens::InteractionState {
                                            focused: is_highlighted,
                                            hovered: item.hovered(),
                                            ..Default::default()
                                        };
                                        paint_state_layer(
                                            ui.painter(),
                                            item_rect,
                                            0,
                                            text_color,
                                            layer,
                                        );
                                    }
                                    let text_rect = item_rect
                                        .shrink2(Vec2::new(MENU_ITEM_HORIZONTAL_PADDING, 0.0));
                                    let clipped = ui
                                        .painter()
                                        .with_clip_rect(text_rect.intersect(ui.clip_rect()));
                                    let inline = option
                                        .description
                                        .as_deref()
                                        .filter(|_| self.inline_descriptions);
                                    match inline {
                                        Some(description) => {
                                            // Two lines centred as a block:
                                            // LabelLarge over BodySmall.
                                            let label_line =
                                                TypeRole::LabelLarge.style().line_height;
                                            let small_line =
                                                TypeRole::BodySmall.style().line_height;
                                            let top = text_rect.center().y
                                                - (label_line + small_line) / 2.0;
                                            clipped.text(
                                                Pos2::new(text_rect.left(), top + label_line / 2.0),
                                                Align2::LEFT_CENTER,
                                                &option.text,
                                                label_font.clone(),
                                                text_color,
                                            );
                                            let description_color = if is_selected {
                                                text_color
                                            } else {
                                                on_surface_variant
                                            };
                                            clipped.text(
                                                Pos2::new(
                                                    text_rect.left(),
                                                    top + label_line + small_line / 2.0,
                                                ),
                                                Align2::LEFT_CENTER,
                                                description,
                                                description_font.clone(),
                                                description_color,
                                            );
                                        }
                                        None => {
                                            clipped.text(
                                                Pos2::new(text_rect.left(), text_rect.center().y),
                                                Align2::LEFT_CENTER,
                                                &option.text,
                                                label_font.clone(),
                                                text_color,
                                            );
                                        }
                                    }
                                    if is_highlighted && scroll_to_highlight {
                                        ui.scroll_to_rect(item_rect, None);
                                    }
                                    if item.clicked() {
                                        picked = Some(index);
                                    }
                                }
                            });
                    });
                });
        }

        if let Some(index) = picked {
            if let Some(option) = self.options.get(index) {
                match &mut self.binding {
                    Binding::Value(selected) => {
                        if **selected != Some(option.value) {
                            response.mark_changed();
                        }
                        **selected = Some(option.value);
                    }
                    Binding::Text(text) => {
                        if **text != option.text {
                            response.mark_changed();
                        }
                        text.clone_from(&option.text);
                    }
                }
                state.filter = option.text.clone();
                state.filtering = false;
                state.highlighted = Some(index);
            }
            if !self.keep_open_on_select {
                state.open = false;
            }
            ctx.memory_mut(|m| m.request_focus(focus_id));
        }

        // Publish which select is open.
        ctx.data_mut(|d| {
            let current = d.get_temp::<Id>(open_select_id());
            if state.open {
                d.insert_temp(open_select_id(), id);
            } else if current == Some(id) {
                d.remove::<Id>(open_select_id());
            }
        });
        let open = state.open;
        ctx.data_mut(|d| d.insert_temp(id, state));

        // Accessibility: a combo box named by its label, with its value and
        // whether the list is expanded.
        let name = self
            .label
            .clone()
            .unwrap_or_else(|| self.placeholder.clone());
        let value = match &self.binding {
            Binding::Value(_) => selected_text(&selected_value).unwrap_or_default(),
            Binding::Text(text) => (*text).clone(),
        };
        let a11y_id = match &edit_response {
            Some(edit) => edit.id,
            None => {
                let value = value.clone();
                let name = name.clone();
                response.widget_info(move || {
                    let mut info =
                        egui::WidgetInfo::labeled(egui::Role::ComboBox, enabled, name.clone());
                    info.current_text_value = Some(value.clone());
                    info
                });
                id
            }
        };
        let required = self.required;
        ctx.accesskit_node_builder(a11y_id, |node| {
            node.set_role(egui::accesskit::Role::ComboBox);
            node.set_label(name);
            node.set_expanded(open);
            if required {
                node.set_required();
            }
        });

        match edit_response {
            Some(edit) => edit.union(response),
            None => response,
        }
    }
}

/// Where the menu goes: below the field if it fits, else above, else on
/// the side with more room, scrolling.
struct MenuLayout {
    rect: Rect,
}

impl MenuLayout {
    fn new(
        ctx: &egui::Context,
        field: Rect,
        content_height: f32,
        width: f32,
        max_height: Option<f32>,
        alignment: MenuAlignment,
    ) -> Self {
        let screen = ctx.content_rect().shrink(MENU_SCREEN_MARGIN);
        let full = content_height + 2.0 * MENU_VERTICAL_PADDING;
        let wanted = max_height.map_or(full, |max| full.min(max.max(MENU_ITEM_HEIGHT)));
        let below = (screen.bottom() - field.bottom()).max(0.0);
        let above = (field.top() - screen.top()).max(0.0);
        let (top, height) = if wanted <= below || below >= above {
            let height = wanted.min(below).max(MENU_ITEM_HEIGHT.min(wanted));
            (field.bottom(), height)
        } else {
            let height = wanted.min(above);
            (field.top() - height, height)
        };
        let left = match alignment {
            MenuAlignment::Start => field.left(),
            MenuAlignment::End => field.right() - width,
        };
        Self {
            rect: Rect::from_min_size(Pos2::new(left, top), Vec2::new(width, height)),
        }
    }
}

/// Paints the Material `arrow_drop_down` glyph (a 10 x 5 dp triangle in a
/// 24 dp box) centred on `center`, turned `turn` of a half turn.
fn paint_drop_down_arrow(painter: &egui::Painter, center: Pos2, turn: f32, color: Color32) {
    let angle = std::f32::consts::PI * turn;
    let (sin, cos) = angle.sin_cos();
    let rotate = |v: Vec2| center + Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos);
    let points = vec![
        rotate(Vec2::new(-5.0, -2.5)),
        rotate(Vec2::new(5.0, -2.5)),
        rotate(Vec2::new(0.0, 2.5)),
    ];
    painter.add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
}

/// Convenience function to create a select component.
///
/// Shorthand for `MaterialSelect::new()`.
///
/// # Example
/// ```rust
/// # use egui_material3::select;
/// # egui::__run_test_ui(|ui| {
/// let mut selection = Some(1);
/// ui.add(select(&mut selection)
///     .option(0, "Option 1")
///     .option(1, "Option 2"));
/// # });
/// ```
pub fn select<'a>(selected: &'a mut Option<usize>) -> MaterialSelect<'a> {
    MaterialSelect::new(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(names: &[&str]) -> Vec<SelectOption> {
        names
            .iter()
            .enumerate()
            .map(|(value, text)| SelectOption::new(value, *text))
            .collect()
    }

    #[test]
    fn filter_keeps_options_containing_the_text() {
        let options = options(&["Apple", "Banana", "Grape", "Pineapple"]);
        let state = SelectState {
            filter: "APP".to_owned(),
            filtering: true,
            ..Default::default()
        };
        assert_eq!(
            visible_options(&options, &state),
            vec![0, 3],
            "case-insensitive contains"
        );
        let unfiltered = SelectState {
            filter: "APP".to_owned(),
            ..Default::default()
        };
        assert_eq!(
            visible_options(&options, &unfiltered).len(),
            4,
            "before typing, every option shows"
        );
    }

    #[test]
    fn typeahead_finds_the_first_prefix_match() {
        let options = options(&["Apple", "Banana", "Blueberry"]);
        assert_eq!(typeahead_match(&options, "b"), Some(1), "first b");
        assert_eq!(typeahead_match(&options, "BL"), Some(2), "bl");
        assert_eq!(typeahead_match(&options, "x"), None, "no match");
        assert_eq!(typeahead_match(&options, ""), None, "empty");
    }

    #[test]
    fn highlight_steps_and_clamps() {
        let visible = [2, 5, 7];
        assert_eq!(
            step_highlight(&visible, None, 1),
            Some(2),
            "down from nothing"
        );
        assert_eq!(
            step_highlight(&visible, None, -1),
            Some(7),
            "up from nothing"
        );
        assert_eq!(step_highlight(&visible, Some(2), 1), Some(5), "down");
        assert_eq!(
            step_highlight(&visible, Some(7), 1),
            Some(7),
            "clamped at end"
        );
        assert_eq!(
            step_highlight(&visible, Some(2), -1),
            Some(2),
            "clamped at start"
        );
        assert_eq!(step_highlight(&[], Some(2), 1), None, "no options");
    }

    #[test]
    fn menu_tokens() {
        assert_eq!(MENU_CORNER_RADIUS, 4.0, "extra-small corners");
        assert_eq!(MENU_ELEVATION, Elevation::Level2, "level 2");
        assert_eq!(MENU_ITEM_HEIGHT, 48.0, "48 dp items");
    }

    /// Regression test for nikescar/egui-material3#16: an open dropdown stays
    /// open when the select moves on screen.
    #[test]
    fn test_dropdown_stays_open_when_select_moves() {
        let ctx = egui::Context::default();
        let mut selected = None;
        let frame = |events: Vec<egui::Event>, offset: f32, selected: &mut Option<usize>| {
            let mut rect = Rect::NOTHING;
            let input = egui::RawInput {
                events,
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| {
                ui.add_space(offset);
                let select = MaterialSelect::new(selected)
                    .placeholder("Pick")
                    .option(0, "Alpha")
                    .option(1, "Beta");
                rect = ui.add(select).rect;
            });
            output.textures_delta.clear();
            let shows_options = output.shapes.iter().any(|clipped| {
                if let egui::Shape::Text(text) = &clipped.shape {
                    text.galley.text() == "Beta"
                } else {
                    false
                }
            });
            (rect, shows_options)
        };
        let (rect, _) = frame(Vec::new(), 0.0, &mut selected);
        let pos = rect.center();
        let button = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(
            vec![egui::Event::PointerMoved(pos), button(true)],
            0.0,
            &mut selected,
        );
        let (_, open) = frame(vec![button(false)], 0.0, &mut selected);
        let (_, open_after) = frame(Vec::new(), 0.0, &mut selected);
        assert!(
            open || open_after,
            "clicking the select should open its dropdown"
        );
        frame(Vec::new(), 30.0, &mut selected);
        let (_, still_open) = frame(Vec::new(), 30.0, &mut selected);
        assert!(
            still_open,
            "the dropdown should stay open after the select moves"
        );
    }
}
