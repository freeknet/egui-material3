// ============================================================================
// Module:       compact_fields (integration test)
// Description:  Dense text fields, the number field, select extras, dense search.
//
// Dependencies: egui, egui_kittest, egui_material3
// ============================================================================

//! Gaps found adopting the text inputs in a desktop app: compact and
//! fill-width `MaterialInput`, `MaterialNumberField` (drag, step keys,
//! typing, AccessKit), select option descriptions and the editable
//! dropdown, and the 36 dp search bar with trailing content.

#![expect(
    clippy::wildcard_enum_match_arm,
    reason = "the tests pick one kind of painted shape out of all of them"
)]

mod support;

use std::cell::{Cell, RefCell};

use egui::accesskit::Role;
use egui::epaint::Shape as PaintShape;
use egui::{Event, Key, Modifiers, PointerButton, Rect};
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::Harness;
use egui_material3::search::{self, MaterialSearch};
use egui_material3::tokens::ColorRole;
use egui_material3::{
    MaterialInput, MaterialNumberField, MaterialSelect, MaterialTheme, SelectOption,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn light() -> MaterialTheme {
    MaterialTheme::baseline(false)
}

/// Every text shape painted last frame: (text, rect).
fn painted_texts(harness: &Harness<'_>) -> Vec<(String, Rect)> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            PaintShape::Text(shape) => Some((
                shape.galley.text().to_owned(),
                shape.galley.rect.translate(shape.pos.to_vec2()),
            )),
            _ => None,
        })
        .collect()
}

fn painted(harness: &Harness<'_>, text: &str) -> Option<Rect> {
    painted_texts(harness)
        .into_iter()
        .find(|(t, _)| t == text)
        .map(|(_, r)| r)
}

// ---------------------------------------------------------------------------
// MaterialInput density
// ---------------------------------------------------------------------------

#[test]
fn compact_input_is_32_high_and_keeps_its_accessible_name() -> TestResult {
    let text = RefCell::new(String::new());
    let rect = Cell::new(Rect::NOTHING);
    let harness = support::harness_for(light(), egui::vec2(320.0, 120.0), |ui| {
        let response = ui.add(
            MaterialInput::outlined(&mut text.borrow_mut())
                .label("Clip name")
                .compact(true),
        );
        rect.set(response.rect);
    });
    assert_eq!(rect.get().height(), 32.0, "compact field is 32 dp");
    let node = harness.get_by_label("Clip name");
    assert_eq!(
        node.accesskit_node().role(),
        Role::TextInput,
        "the named node is the text"
    );
    // The label is the placeholder, at the input size, not a floating label.
    let sizes: Vec<f32> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|c| match &c.shape {
            PaintShape::Text(t) if t.galley.text() == "Clip name" => {
                t.galley.job.sections.first().map(|s| s.format.font_id.size)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        sizes,
        vec![16.0],
        "the label is painted once, as the BodyLarge placeholder"
    );
    Ok(())
}

#[test]
fn density_steps_are_8_dp_and_states_still_paint() -> TestResult {
    // 8 dp steps to -3, then the desktop steps -4 (28) and -5 (24), the
    // densest; anything denser clamps to -5.
    for (density, height) in [
        (0, 56.0),
        (-1, 48.0),
        (-2, 40.0),
        (-3, 32.0),
        (-4, 28.0),
        (-5, 24.0),
        (-9, 24.0),
    ] {
        let text = RefCell::new(String::from("x"));
        let rect = Cell::new(Rect::NOTHING);
        let _harness = support::harness_for(light(), egui::vec2(320.0, 120.0), |ui| {
            let response = ui.add(
                MaterialInput::filled(&mut text.borrow_mut())
                    .label("Name")
                    .density(density),
            );
            rect.set(response.rect);
        });
        assert_eq!(rect.get().height(), height, "density {density}");
    }
    // Error and disabled colours survive density.
    let theme = light();
    let error = theme.color(ColorRole::Error);
    let text = RefCell::new(String::from("x"));
    let harness = support::harness_for(theme, egui::vec2(320.0, 120.0), |ui| {
        ui.add(
            MaterialInput::filled(&mut text.borrow_mut())
                .label("Name")
                .compact(true)
                .is_error(true),
        );
        ui.add(
            MaterialInput::outlined(&mut String::new())
                .label("Off")
                .compact(true)
                .enabled(false),
        );
    });
    let paints_error = harness.output().shapes.iter().any(|c| match &c.shape {
        PaintShape::Vec(shapes) => shapes.iter().any(|s| match s {
            PaintShape::Rect(r) => r.fill == error,
            _ => false,
        }),
        _ => false,
    });
    assert!(
        paints_error,
        "a compact field in error paints the error indicator"
    );
    assert!(
        harness.get_by_label("Off").accesskit_node().is_disabled(),
        "a compact disabled field is disabled"
    );
    Ok(())
}

#[test]
fn compact_input_focus_indicator_is_primary() -> TestResult {
    let theme = light();
    let primary = theme.color(ColorRole::Primary);
    let text = RefCell::new(String::new());
    let mut harness = support::harness_for(theme, egui::vec2(320.0, 120.0), |ui| {
        ui.add(
            MaterialInput::filled(&mut text.borrow_mut())
                .label("Name")
                .compact(true),
        );
    });
    harness.get_by_label("Name").focus();
    harness.run();
    let indicator = harness.output().shapes.iter().find_map(|c| match &c.shape {
        PaintShape::Vec(shapes) => shapes.iter().find_map(|s| match s {
            PaintShape::Rect(r) if r.fill == primary => Some(r.rect.height()),
            _ => None,
        }),
        _ => None,
    });
    assert_eq!(
        indicator,
        Some(2.0),
        "a focused compact field has the 2 dp primary line"
    );
    Ok(())
}

#[test]
fn fill_width_overrides_a_fixed_width() -> TestResult {
    let fixed = Cell::new(Rect::NOTHING);
    let filled = Cell::new(Rect::NOTHING);
    let available = Cell::new(0.0);
    let _harness = support::harness_for(light(), egui::vec2(320.0, 200.0), |ui| {
        available.set(ui.available_width());
        fixed.set(
            ui.add(
                MaterialInput::outlined(&mut String::new())
                    .label("Fixed")
                    .width(100.0),
            )
            .rect,
        );
        filled.set(
            ui.add(
                MaterialInput::outlined(&mut String::new())
                    .label("Filled")
                    .width(100.0)
                    .fill_width(true)
                    .compact(true),
            )
            .rect,
        );
    });
    assert_eq!(fixed.get().width(), 100.0, "a fixed width");
    assert_eq!(
        filled.get().width(),
        available.get(),
        "fill_width takes the available width"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// MaterialNumberField
// ---------------------------------------------------------------------------

fn number_harness<'a>(value: &'a RefCell<f64>, compact: bool) -> Harness<'a> {
    support::harness_for(light(), egui::vec2(320.0, 120.0), move |ui| {
        let mut value = value.borrow_mut();
        ui.add(
            MaterialNumberField::new(&mut *value)
                .label("Opacity")
                .range(0.0..=100.0)
                .step(0.5)
                .suffix("%")
                .compact(compact),
        );
    })
}

#[test]
fn number_field_reports_a_spin_button() -> TestResult {
    let value = RefCell::new(25.0);
    let harness = number_harness(&value, false);
    let node = harness.get_by_label("Opacity");
    let ak = node.accesskit_node();
    assert_eq!(ak.role(), Role::SpinButton, "role");
    assert_eq!(ak.numeric_value(), Some(25.0), "value");
    assert_eq!(ak.min_numeric_value(), Some(0.0), "min");
    assert_eq!(ak.max_numeric_value(), Some(100.0), "max");
    assert_eq!(ak.numeric_value_step(), Some(0.5), "step");
    assert_eq!(node.value().as_deref(), Some("25%"), "the shown text");
    Ok(())
}

#[test]
fn number_field_compact_and_default_heights() -> TestResult {
    for (name, theme) in support::themes() {
        for (compact, height) in [(false, 56.0), (true, 32.0)] {
            let value = RefCell::new(1.0);
            let harness = support::harness_for(theme.clone(), egui::vec2(320.0, 120.0), |ui| {
                ui.add(
                    MaterialNumberField::filled(&mut *value.borrow_mut())
                        .label("Opacity")
                        .compact(compact),
                );
            });
            let rect = harness.get_by_label("Opacity").rect();
            assert_eq!(rect.height(), height, "{name}: compact {compact}");
            let outline = theme.color(ColorRole::SurfaceContainerHighest);
            let paints_container = harness.output().shapes.iter().any(|c| match &c.shape {
                PaintShape::Vec(shapes) => shapes.iter().any(|s| match s {
                    PaintShape::Rect(r) => r.fill == outline,
                    _ => false,
                }),
                _ => false,
            });
            assert!(paints_container, "{name}: the filled container");
        }
    }
    Ok(())
}

#[test]
fn dragging_changes_the_value_by_steps() -> TestResult {
    let value = RefCell::new(10.0);
    let mut harness = number_harness(&value, true);
    let start = harness.get_by_label("Opacity").rect().center();
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    harness.event(Event::PointerMoved(start));
    harness.event(button(start, true));
    // 4 points per step: 6 moves of 4 points to the right is 6 steps.
    for i in 1..=6 {
        harness.event(Event::PointerMoved(start + egui::vec2(4.0 * i as f32, 0.0)));
    }
    let end = start + egui::vec2(24.0, 0.0);
    harness.event(button(end, false));
    harness.run();
    let after = *value.borrow();
    assert!(
        (after * 2.0).fract() == 0.0 && after > 10.0,
        "a drag moves by whole steps: {after}"
    );
    assert!(
        (12.0..=13.0).contains(&after),
        "24 points at 4 points per step is about 6 steps of 0.5: {after}"
    );
    assert!(
        !harness.get_by_label("Opacity").is_focused(),
        "a drag does not start typing"
    );
    Ok(())
}

#[test]
fn up_down_and_page_keys_step() -> TestResult {
    let value = RefCell::new(10.0);
    let mut harness = number_harness(&value, false);
    harness.get_by_label("Opacity").focus();
    harness.run();
    assert!(harness.get_by_label("Opacity").is_focused(), "focused");
    harness.key_press(Key::ArrowUp);
    harness.run();
    assert_eq!(*value.borrow(), 10.5, "Up adds a step");
    harness.key_press(Key::ArrowDown);
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert_eq!(*value.borrow(), 9.5, "Down takes a step");
    harness.key_press(Key::PageUp);
    harness.run();
    assert_eq!(*value.borrow(), 14.5, "Page Up adds ten steps");
    harness.key_press(Key::PageDown);
    harness.key_press(Key::PageDown);
    harness.run();
    assert_eq!(*value.borrow(), 4.5, "Page Down takes ten steps");
    for _ in 0..3 {
        harness.key_press(Key::PageDown);
    }
    harness.run();
    assert_eq!(*value.borrow(), 0.0, "stepping clamps at the minimum");
    Ok(())
}

#[test]
fn typing_commits_clamped_on_enter_and_rejects_junk() -> TestResult {
    let value = RefCell::new(10.0);
    let mut harness = number_harness(&value, false);
    harness.get_by_label("Opacity").focus();
    harness.run();
    // Focus selects the value, so typing replaces it.
    harness.get_by_label("Opacity").type_text("250");
    harness.run();
    assert_eq!(
        *value.borrow(),
        10.0,
        "out-of-range typing waits for commit"
    );
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        *value.borrow(),
        100.0,
        "Enter commits, clamped to the maximum"
    );

    harness.get_by_label("Opacity").focus();
    harness.run();
    harness.get_by_label("Opacity").type_text("abc");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        *value.borrow(),
        100.0,
        "text that is not a number is dropped"
    );

    harness.get_by_label("Opacity").focus();
    harness.run();
    harness.get_by_label("Opacity").type_text("42");
    harness.run();
    assert_eq!(
        *value.borrow(),
        42.0,
        "an in-range number applies while typing"
    );
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(
        *value.borrow(),
        100.0,
        "Escape restores the value from before"
    );
    Ok(())
}

#[test]
fn integer_fields_round_and_accesskit_steps() -> TestResult {
    let value = RefCell::new(30_u32);
    let mut harness = support::harness_for(light(), egui::vec2(320.0, 120.0), |ui| {
        let mut value = value.borrow_mut();
        ui.add(
            MaterialNumberField::filled(&mut *value)
                .label("FPS")
                .range(1..=240),
        );
    });
    harness.get_by_label("FPS").increment_accesskit();
    harness.run();
    assert_eq!(*value.borrow(), 31, "AccessKit increment");
    harness.get_by_label("FPS").set_value_accesskit(59.6);
    harness.run();
    assert_eq!(*value.borrow(), 60, "AccessKit set value, rounded");
    harness.get_by_label("FPS").focus();
    harness.run();
    harness.get_by_label("FPS").type_text("0");
    harness.key_press(Key::Tab);
    harness.run();
    assert_eq!(*value.borrow(), 1, "leaving the field commits, clamped");
    Ok(())
}

// ---------------------------------------------------------------------------
// MaterialSelect
// ---------------------------------------------------------------------------

#[test]
fn option_descriptions_are_tooltips_accessible_and_inline() -> TestResult {
    for inline in [false, true] {
        let selected = RefCell::new(None);
        let mut harness = support::harness_for(light(), egui::vec2(400.0, 400.0), |ui| {
            ui.add(
                MaterialSelect::new(&mut selected.borrow_mut())
                    .label("Preset")
                    .width(300.0)
                    .option_with_description(0, "Fast", "Lower quality, quick render")
                    .option_item(SelectOption::new(1, "Best").description("Slow, highest quality"))
                    .option(2, "Custom")
                    .inline_descriptions(inline),
            );
        });
        harness
            .ctx
            .all_styles_mut(|s| s.interaction.tooltip_delay = 0.0);
        harness.get_by_label("Preset").click();
        harness.run();
        let fast = harness.get_by_label("Fast");
        assert_eq!(
            fast.accesskit_node().description().as_deref(),
            Some("Lower quality, quick render"),
            "the option's accessible description"
        );
        let fast_rect = fast.rect();
        let custom_rect = harness.get_by_label("Custom").rect();
        let painted_inline = painted(&harness, "Lower quality, quick render").is_some();
        assert_eq!(
            painted_inline, inline,
            "inline {inline}: second line painted"
        );
        let expected = if inline { 64.0 } else { 48.0 };
        assert_eq!(fast_rect.height(), expected, "inline {inline}: row height");
        assert_eq!(custom_rect.height(), 48.0, "no description: one line");

        harness.hover_at(harness.get_by_label("Best").rect().center());
        harness.run_steps(4);
        assert!(
            painted(&harness, "Slow, highest quality").is_some(),
            "inline {inline}: hovering an option shows its description as a tooltip"
        );
    }
    Ok(())
}

const CODECS: [&str; 3] = ["h264", "hevc", "prores"];

fn editable_harness(text: &RefCell<String>) -> Harness<'_> {
    support::harness_for(light(), egui::vec2(400.0, 400.0), |ui| {
        let mut text = text.borrow_mut();
        let select = CODECS.iter().enumerate().fold(
            MaterialSelect::editable(&mut text)
                .label("Codec")
                .width(240.0),
            |s, (i, c)| s.option(i, *c),
        );
        ui.add(select);
        ui.label("elsewhere");
    })
}

#[test]
fn editable_select_filters_suggestions_and_keeps_free_text() -> TestResult {
    let text = RefCell::new(String::new());
    let mut harness = editable_harness(&text);
    harness.get_by_label("Codec").focus();
    harness.run();
    harness.get_by_label("Codec").type_text("h");
    harness.run();
    assert_eq!(text.borrow().as_str(), "h", "typing edits the bound text");
    assert!(harness.query_by_label("h264").is_some(), "h264 suggested");
    assert!(harness.query_by_label("hevc").is_some(), "hevc suggested");
    assert!(
        harness.query_by_label("prores").is_none(),
        "prores filtered out"
    );

    harness.get_by_label("Codec").type_text("x");
    harness.run();
    assert!(
        harness.query_by_label("h264").is_none(),
        "nothing matches hx"
    );
    harness.key_press(Key::Tab);
    harness.run();
    assert_eq!(
        text.borrow().as_str(),
        "hx",
        "free text that matches no option is kept after leaving the field"
    );
    Ok(())
}

#[test]
fn editable_select_picks_a_suggestion() -> TestResult {
    let text = RefCell::new(String::from("old"));
    let mut harness = editable_harness(&text);
    assert_eq!(
        harness.get_by_label("Codec").value().as_deref(),
        Some("old"),
        "the field shows the bound text"
    );
    harness.get_by_label("Codec").focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_label("Codec").type_text("pro");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        text.borrow().as_str(),
        "prores",
        "Enter picks the first suggestion"
    );
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_label("Codec").type_text("hev");
    harness.run();
    harness.get_by_label("hevc").click();
    harness.run();
    assert_eq!(
        text.borrow().as_str(),
        "hevc",
        "clicking a suggestion picks it"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// MaterialSearch
// ---------------------------------------------------------------------------

const SEARCH_ID: &str = "dense_search";

fn search_harness(query: &RefCell<String>) -> Harness<'_> {
    support::harness_for(light(), egui::vec2(480.0, 400.0), |ui| {
        MaterialSearch::new(SEARCH_ID, &mut query.borrow_mut())
            .placeholder("Search commands")
            .height(36.0)
            .width(400.0)
            .suggestions(["Render", "Export"])
            .trailing_content(|ui| {
                ui.label("Ctrl+K");
            })
            .show(ui);
    })
}

#[test]
fn search_bar_can_be_36_high() -> TestResult {
    let query = RefCell::new(String::new());
    let harness = search_harness(&query);
    let container = light().color(ColorRole::SurfaceContainerHigh);
    let bar = harness
        .output()
        .shapes
        .iter()
        .find_map(|c| match &c.shape {
            PaintShape::Rect(r) if r.fill == container => Some(r.clone()),
            _ => None,
        })
        .ok_or("the bar paints its container")?;
    assert_eq!(bar.rect.height(), 36.0, "a 36 dp bar");
    assert_eq!(bar.corner_radius.nw, 18, "still fully rounded");
    assert_eq!(search::MIN_BAR_HEIGHT, 36.0, "the floor");
    Ok(())
}

#[test]
fn search_trailing_content_is_painted_in_the_bar() -> TestResult {
    let query = RefCell::new(String::new());
    let harness = search_harness(&query);
    let chip = painted(&harness, "Ctrl+K").ok_or("the trailing content is painted")?;
    let field = harness
        .query_by_role(Role::SearchInput)
        .ok_or("a search field")?
        .rect();
    assert!(
        chip.min.y >= 0.0 && chip.max.y <= 36.0,
        "the content sits inside the 36 dp bar: {chip:?}"
    );
    assert!(
        chip.left() >= field.right(),
        "the query stops before the content: field {field:?}, content {chip:?}"
    );
    assert!(chip.right() <= 400.0, "inside the bar's width");
    Ok(())
}

#[test]
fn dense_search_still_expands_on_focus() -> TestResult {
    let query = RefCell::new(String::new());
    let mut harness = search_harness(&query);
    harness
        .query_by_role(Role::SearchInput)
        .ok_or("a search field")?
        .focus();
    harness.run();
    assert!(
        search::is_expanded(&harness.ctx, SEARCH_ID),
        "focus expands the 36 dp bar"
    );
    assert!(
        harness.query_by_label("Render").is_some(),
        "the view lists the suggestions"
    );
    assert!(
        painted(&harness, "Ctrl+K").is_none(),
        "the expanded view does not show the trailing content"
    );
    harness.key_press(Key::Escape);
    harness.run();
    assert!(
        !search::is_expanded(&harness.ctx, SEARCH_ID),
        "Escape collapses"
    );
    Ok(())
}
