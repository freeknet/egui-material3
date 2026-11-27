// ============================================================================
// Module:       egui-material3 (library crate root)
// Description:  Crate root: module tree and the public re-exports of every component.
//
// Dependencies: eframe, egui; self::spreadsheet
// ============================================================================

//! # egui-material3
//!
//! A comprehensive Material Design 3 component library for [egui](https://github.com/emilk/egui),
//! providing a complete set of Material Design components with advanced theming support.
//!
//! ## Features
//!
//! - **Complete Material Design 3 Components**: Buttons, checkboxes, sliders, dialogs, data tables, and more
//! - **Advanced Theming System**: Support for light/dark modes, contrast levels, and custom Material Design themes
//! - **Build-time Theme Inclusion**: Automatically include theme JSON files at compile time for optimal performance
//! - **Runtime Theme Loading**: Load and switch themes dynamically at runtime
//! - **Material Design Icons**: Full support for Material Symbols with built-in icon font loading
//! - **Responsive Design**: Components adapt to different screen sizes and orientations
//!
//! ## Quick Start
//!
//! Add this to your `Cargo.toml`:
//! ```bash
//! $ cargo add egui-material3
//! ```
//!
//! ### Basic Usage
//!
//! ```rust,no_run
//! use eframe::egui;
//! use egui_material3::{
//!     MaterialButton, MaterialCheckbox, MaterialSlider,
//!     theme::{setup_google_fonts, setup_local_fonts, setup_local_theme,
//!            load_fonts, load_themes, update_window_background}
//! };
//!
//! fn main() -> Result<(), eframe::Error> {
//!     let options = eframe::NativeOptions {
//!         viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
//!         ..Default::default()
//!     };
//!     
//!     eframe::run_native(
//!         "Material Design App",
//!         options,
//!         Box::new(|cc| {
//!             // Setup Material Design fonts and themes
//!             setup_google_fonts(Some("Roboto"));
//!             setup_local_fonts(Some("resources/MaterialSymbolsOutlined.ttf"));
//!             setup_local_theme(None); // Use default theme
//!             
//!             // Load fonts and themes
//!             load_fonts(&cc.egui_ctx);
//!             load_themes();
//!             
//!             // Apply theme background
//!             update_window_background(&cc.egui_ctx);
//!             
//!             Ok(Box::<MyApp>::default())
//!         }),
//!     )
//! }
//!
//! #[derive(Default)]
//! struct MyApp {
//!     checked: bool,
//!     slider_value: f32,
//! }
//!
//! impl eframe::App for MyApp {
//!     fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
//!         egui::CentralPanel::default().show(ui, |ui| {
//!             ui.heading("Material Design Components");
//!             
//!             // Use Material Design components
//!             ui.add(MaterialButton::new("Click me"));
//!             ui.add(MaterialCheckbox::new(&mut self.checked, "Check me"));
//!             ui.add(MaterialSlider::new(&mut self.slider_value, 0.0..=100.0));
//!         });
//!     }
//! }
//! ```
//!
//! ## Theme System
//!
//! The theme system supports Material Design 3 with comprehensive theming capabilities:
//!
//! ### Build-time Theme Inclusion
//!
//! Themes can be automatically included at build time from JSON files:
//!
//! ```rust,no_run
//! use egui_material3::theme::{setup_local_theme, load_themes};
//!
//! // Uses themes from resources/ and examples/ directories automatically
//! setup_local_theme(None);
//! load_themes();
//! ```
//!
//! ### Runtime Theme Loading
//!
//! Load custom themes dynamically:
//!
//! ```rust,no_run
//! use egui_material3::theme::{setup_local_theme, load_themes};
//!
//! // Load specific theme file
//! setup_local_theme(Some("path/to/my-theme.json"));
//! load_themes();
//! ```
//!
//! ### Theme Modes and Contrast
//!
//! Each egui context carries its own theme, so two viewports (or two tests
//! running in parallel) can differ. Change a context's mode and contrast at
//! runtime with [`theme::apply_theme_context`]:
//!
//! ```rust
//! use egui_material3::theme::{
//!     apply_theme_context, theme_context_of, update_window_background, ContrastLevel, ThemeMode,
//! };
//!
//! # egui::__run_test_ctx(|ctx| {
//! let mut settings = theme_context_of(ctx);
//! settings.theme_mode = ThemeMode::Dark;
//! settings.contrast_level = ContrastLevel::High;
//! apply_theme_context(ctx, &settings, None::<fn() -> ThemeMode>);
//! update_window_background(ctx);
//! # });
//! ```
//!
//! ## Available Components
//!
//! ### Basic Components
//! - [`MaterialButton`] - Material Design buttons with multiple variants
//! - [`MaterialCheckbox`] - Checkboxes following Material Design guidelines  
//! - [`MaterialSlider`] - Sliders with Material Design styling
//! - [`MaterialSwitch`] - Toggle switches
//! - [`MaterialRadio`] - Radio button groups
//! - [`MaterialSelect`] - Dropdown selection components
//!
//! ### Advanced Components
//! - [`MaterialChip`] - Filter and action chips
//! - [`MaterialCard2`] - Material Design cards
//! - [`MaterialDialog`] - Modal dialogs and alerts
//! - [`MaterialFab`] - Floating Action Buttons
//! - [`MaterialProgress`] - Progress indicators and loading states
//! - [`MaterialDataTable`] - Data tables with sorting and selection
//!
//! ### Navigation Components
//! - [`MaterialTabs`] - Tab navigation
//! - [`MaterialDrawer`] - Navigation drawers
//! - [`MaterialTopAppBar`] - Top app bars
//! - [`MaterialBottomAppBar`] - Bottom app bar with a FAB slot
//! - [`MaterialToolbar`] - Docked toolbar
//! - [`MaterialFloatingToolbar`] - Floating toolbar
//!
//! ### Icons and Visual Elements
//! - [`MaterialIcon`] - Material Design icons with font support
//! - [`MaterialList`] - Lists following Material Design patterns
//! - [`MaterialImageList`] - Image lists with online/offline support and smart caching
//! - [`MaterialTimeline`] - Timeline component for displaying chronological events
//!
//! ## Image Lists and OnDemand Feature
//!
//! The [`MaterialImageList`] component provides comprehensive image display capabilities:
//!
//! ```rust,no_run
//! use egui_material3::image_list;
//!
//! # fn demo(ui: &mut egui::Ui) -> Result<(), Box<dyn std::error::Error>> {
//! // Local image files
//! ui.add(image_list()
//!     .columns(3)
//!     .item_spacing(8.0)
//!     .items_from_paths(glob::glob("resources/*.png")?.flatten()));
//!
//! // Online images (requires 'ondemand' feature)
//! ui.add(image_list()
//!     .columns(4)
//!     .item_spacing(8.0)
//!     .items_from_urls(vec![
//!         "https://example.com/image1.jpg".to_string(),
//!         "https://example.com/image2.png".to_string(),
//!     ]));
//!
//! // Embedded images from byte arrays
//! ui.add(image_list()
//!     .columns(2)
//!     .item_spacing(8.0)
//!     .items_from_bytes(vec![
//!         include_bytes!("../resources/320x240.png").to_vec(),
//!         include_bytes!("../resources/imgur_image.png").to_vec(),
//!     ]));
//! # Ok(())
//! # }
//! ```
//!
//! ### SVG and OS theme features
//!
//! No feature is on by default. `svg` rasterises SVG icons with resvg
//! (without it an SVG icon paints nothing and the `image_utils` SVG
//! functions return an error); `system-theme` lets `detect_os_theme`
//! follow the OS light/dark setting (without it, it returns
//! `ThemeMode::Light`). Enable both to keep the behaviour of releases
//! where they were always on:
//!
//! ```toml
//! [dependencies]
//! egui-material3 = { version = "...", features = ["svg", "system-theme"] }
//! ```
//!
//! ### OnDemand Feature
//!
//! Enable the `ondemand` feature for online image support:
//!
//! ```toml
//! [dependencies]
//! egui-material3 = { version = "0.0.6", features = ["ondemand"] }
//! ```
//!
//! Key capabilities:
//! - **Smart caching**: Downloaded images are cached locally with correct file extensions
//! - **Format detection**: Automatically detects PNG, JPEG, GIF, and WebP formats
//! - **Efficient loading**: Images are downloaded once and reused from cache
//! - **Performance optimized**: UI repaints only when new images are available
//! - **Error handling**: Graceful fallback with visual indicators for failed loads
//!
//! ## Examples
//!
//! The crate includes comprehensive examples:
//!
//! - `widget_gallery_example` - Showcase of all Material components with theme switching
//! - `nobel_prizes_example` - Real-world data table implementation
//! - `stories` - Individual component showcase windows for detailed exploration
//! - `package` - Standalone example with bundled resources and themes
//! - `ondemand` - Image list demonstration with online/offline images and smart caching
//!
//! Run examples with:
//! ```bash
//! cargo run --example widget_gallery_example
//! cargo run --example nobel_prizes_example
//! cargo run --example stories
//!
//! # OnDemand example with online image support
//! cd examples/ondemand && cargo run
//!
//! # Package example runs independently with its own Cargo.toml
//! cd examples/package && cargo run
//! ```
//!
//! ## Material Design Resources
//!
//! - [Material Design 3](https://m3.material.io/)
//! - [Material Theme Builder](https://m3.material.io/theme-builder)
//! - [Material Design Icons](https://fonts.google.com/icons)
//!
//! This crate follows the Material Design 3 specifications and guidelines for consistent,
//! accessible, and beautiful user interfaces.

pub mod a11y;
pub mod actionsheet;
pub mod badge;
pub mod banner;
pub mod bottom_app_bar;
pub mod bottom_sheet;
pub mod breadcrumbs;
pub mod button;
pub mod button_group;
pub mod card2;
pub mod carousel;
pub mod checkbox;
pub mod chips;
pub mod color;
pub mod dashcounter;
pub mod datatable;
pub mod date_picker;
pub mod dialog;
pub mod divider;
pub mod drawer;
#[cfg(feature = "spreadsheet")]
pub mod egui_smol;
pub mod fab;
pub mod fab_menu;
pub mod floating_toolbar;
pub mod fonts;
pub mod icon;
pub mod iconbutton;
pub mod image_utils;
pub mod imagelist;
pub mod input;
pub mod layoutgrid;
pub mod list;
pub mod loading_indicator;
pub mod material_symbol;
pub mod material_theme;
pub mod menu;
pub mod navigation;
pub mod navigation_bar;
pub mod navigation_rail;
pub mod notification;
pub mod noto_emoji;
pub mod paint;
pub mod progress;
pub mod radio;
pub mod scrim;
pub mod search;
pub mod segmented_button;
pub mod select;
pub mod shapes;
pub mod side_sheet;
pub mod slider;
pub mod snackbar;
pub mod split_button;
pub mod spreadsheet;
#[cfg(feature = "svg_emoji")]
pub mod svg_emoji;
pub mod switch;
pub mod tabs;
pub mod theme;
pub mod time_picker;
pub mod timeline;
pub mod tokens;
pub mod toolbar;
pub mod tooltip;
pub mod topappbar;
pub mod treeview;

// Deprecated FAB items, kept so existing callers compile.
#[expect(
    deprecated,
    reason = "re-exports deprecated items for existing callers"
)]
pub use fab::{fab_branded, google_branded_icon, SvgIcon, SvgPath};

// MVVM support module
pub mod mvvm;

#[expect(
    deprecated,
    reason = "kept so existing callers of the old badge sizes still compile"
)]
pub use badge::BadgeSize;

// The process-wide theme shims, kept so existing callers compile; widgets
// read `MaterialTheme::of(ctx)`.
#[expect(
    deprecated,
    reason = "re-exports the deprecated global-theme shims for existing callers"
)]
pub use theme::{get_global_color, get_global_color_role, get_global_theme, update_global_theme};

pub use {
    actionsheet::{action_sheet, ActionButton, ActionGroup, MaterialActionSheet},
    badge::{badge, badge_dot, BadgeColor, BadgePosition, MaterialBadge},
    banner::{BannerExt, BannerResponse, BannerTone, MaterialBanner},
    bottom_app_bar::{bottom_app_bar, MaterialBottomAppBar, BOTTOM_APP_BAR_HEIGHT},
    bottom_sheet::{BottomSheet, BottomSheetKind, BottomSheetResponse, SheetCloseReason},
    breadcrumbs::{breadcrumbs, MaterialBreadcrumbs},
    button::{ButtonShape, ButtonSize, MaterialButton, MaterialButtonVariant},
    button_group::{ButtonGroup, ButtonGroupResponse},
    card2::{elevated_card2, filled_card2, outlined_card2, Card2Variant, MaterialCard2},
    carousel::{
        carousel, CarouselItem, CarouselItemLayout, CarouselMetrics, CarouselResponse,
        CarouselStrategy, MaterialCarousel,
    },
    checkbox::{checkbox, MaterialCheckbox},
    chips::{
        assist_chip, chip_set, filter_chip, input_chip, suggestion_chip, ChipResponse, ChipVariant,
        MaterialChip,
    },
    dashcounter::{dashcounter, CounterCard, MaterialDashCounter},
    datatable::{
        data_table, CellContent, ColumnWidth, DataTableCell, DataTableColumn, DataTableRow,
        DataTableSource, DataTableState, DataTableTheme, HAlign, MaterialDataTable, RowAction,
        SortDirection, VAlign,
    },
    dialog::{
        dialog, DialogCloseReason, DialogContentStyle, DialogResponse, FullScreenDialog,
        MaterialDialog,
    },
    divider::{divider, vertical_divider, DividerOrientation, MaterialDivider},
    drawer::{
        dismissible_drawer, modal_drawer, permanent_drawer, standard_drawer, DrawerAlignment,
        DrawerHeader, DrawerItem, DrawerSection, DrawerThemeData, DrawerVariant, MaterialDrawer,
    },
    egui::TextEdit, // Re-export egui's TextEdit
    fab::{
        fab_primary, fab_primary_container, fab_secondary, fab_secondary_container, fab_surface,
        fab_tertiary, fab_tertiary_container, ExtendedFabMetrics, FabMetrics, FabSize, FabVariant,
        MaterialFab,
    },
    fab_menu::{FabMenu, FabMenuColors, FabMenuResponse},
    floating_toolbar::{
        floating_toolbar, FloatingToolbarOrientation, FloatingToolbarStyle,
        MaterialFloatingToolbar, FLOATING_TOOLBAR_SIZE,
    },
    icon::{icon, paint_icon, Icon, MaterialIcon},
    iconbutton::{
        icon_button_filled, icon_button_filled_tonal, icon_button_outlined, icon_button_standard,
        icon_button_toggle, IconButtonColors, IconButtonMetrics, IconButtonShape, IconButtonSize,
        IconButtonVariant, IconButtonWidth, MaterialIconButton,
    },
    imagelist::{
        image_list, masonry_image_list, woven_image_list, ImageListItem, ImageListVariant,
        MaterialImageList,
    },
    input::{InputVariant, MaterialInput},
    layoutgrid::{debug_layout_grid, layout_grid, GridTile, GridTileBar, MaterialLayoutGrid},
    list::{
        list, list_item, ListItem, ListLeading, ListResponse, ListTileStyle,
        ListTileTitleAlignment, MaterialList, MaterialListState, VisualDensity,
    },
    loading_indicator::LoadingIndicator,
    material_theme::{MaterialTheme, SUCCESS_SEED, WARNING_SEED},
    menu::{
        context_menu, menu, menu_item, Corner, FocusState, MaterialMenu, MenuBarThemeData,
        MenuButtonThemeData, MenuEntry, MenuItem, MenuResponse, MenuStyle, MenuThemeData,
        MenuVariant, Positioning,
    },
    navigation::{NavigationBadge, NavigationItem},
    navigation_bar::{NavigationBar, NavigationBarVariant},
    navigation_rail::{NavigationRail, RailHeader},
    notification::{
        notification, MaterialNotification, MaterialNotificationWithOffset, NotificationAlign,
    },
    progress::{circular_progress, linear_progress, MaterialProgress, ProgressVariant},
    radio::{
        radio, radio_group, radio_list_tile, ListTileControlAffinity, MaterialRadio,
        MaterialRadioGroup, RadioListTile,
    },
    segmented_button::{Segment, SegmentedButton, SegmentedButtonResponse},
    select::{select, MaterialSelect, MenuAlignment, SelectVariant},
    side_sheet::{SideSheet, SideSheetKind, SideSheetResponse},
    slider::{
        range_slider, slider, MaterialRangeSlider, MaterialSlider, RangeValues, SliderInteraction,
        SliderSize, ThumbShape,
    },
    snackbar::{
        snackbar, snackbar_with_action, MaterialSnackbar, MaterialSnackbarWithOffset,
        SnackBarBehavior, SnackbarDuration, SnackbarHost, SnackbarId, SnackbarOutcome,
        SnackbarPosition, SnackbarResult, SnackbarVisuals,
    },
    split_button::{SplitButton, SplitButtonResponse},
    switch::{switch, MaterialSwitch},
    tabs::{tabs_primary, tabs_secondary, MaterialTabs, TabItem, TabLayout, TabVariant},
    theme::{
        apply_theme_context, theme_context_of, ContrastLevel, MaterialThemeContext,
        MaterialThemeFile, ThemeMode,
    },
    timeline::{
        timeline, MaterialTimeline, TimelineDot, TimelineDotColor, TimelineDotVariant,
        TimelineItem, TimelinePosition,
    },
    tokens::{ColorRole, ColorScheme},
    toolbar::{
        docked_toolbar, scroll_area_id, scroll_area_offset, toolbar, MaterialToolbar,
        ToolbarElevation,
    },
    tooltip::{
        show_tooltip_on_hover, show_tooltip_on_hover_custom, tooltip, with_tooltip,
        MaterialTooltip, RichTooltip, RichTooltipResponse, TooltipPosition,
    },
    topappbar::{
        center_aligned_top_app_bar, large_flexible_top_app_bar, large_top_app_bar,
        medium_flexible_top_app_bar, medium_top_app_bar, top_app_bar, MaterialTopAppBar,
        TopAppBarScrollBehavior, TopAppBarVariant,
    },
    treeview::{tree_view, MaterialTreeView, TreeViewItem, TreeViewState},
};

#[cfg(feature = "spreadsheet")]
pub use spreadsheet::{
    column, integer_column, number_column, text_column, ColumnDef, ColumnType, FileFormat,
    MaterialSpreadsheet, RowData, SpreadsheetAction, SpreadsheetDataModel,
};
