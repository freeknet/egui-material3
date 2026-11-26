// ============================================================================
// Module:       header_check (integration test)
// Description:  Enforces the file-header convention: every source file opens
//               with a banner naming its module, purpose, and dependencies.
//
// Dependencies: anyhow, std::fs (walks build.rs, src/, tests/ and examples/)
// ============================================================================

//! Enforces the file-header convention across every source file.
//!
//! The banner at the top of a file is the one piece of orientation nothing
//! else can imply: a reader opening a module cold gets its path in the
//! crate, what it is for, and what it leans on, before reading a line of
//! code. A convention only holds if something checks it, so this walks
//! `build.rs`, `src/`, `tests/` and `examples/` and fails for any file
//! whose header is missing, malformed, or naming the wrong module.
//!
//! The check is structural, not editorial. It verifies that the fields are
//! present and filled in, and that `Module:` matches the path the file sits
//! at, which is what catches a header copied from a neighbour and never
//! re-read. Whether a description is any good is a question for review.
//!
//! The banner is plain `//`, not `//!`, and is deliberately not rustdoc.
//! The `//!` block below it is the documentation.
//!
//! `Module:` follows the path, adapted to a library crate with examples:
//!
//! | File                              | `Module:`                              |
//! |-----------------------------------|----------------------------------------|
//! | `build.rs`                        | `build (build script)`                 |
//! | `src/lib.rs`                      | `egui-material3 (library crate root)`  |
//! | `src/foo.rs`, `src/foo/mod.rs`    | `foo`                                  |
//! | `src/foo/bar.rs`                  | `foo::bar`                             |
//! | `tests/x.rs`                      | `x (integration test)`                 |
//! | `examples/x.rs`                   | `x (example)`                          |
//! | `examples/x/main.rs`              | `x (example crate root)`               |
//! | `examples/x/y.rs`                 | `x::y (example)`                       |
//! | `examples/x/src/main.rs`          | `x (standalone example crate)`         |

use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// The rule above and below every header. Both edges are checked, so a
/// header cannot silently run on into the code below it.
const RULE: &str =
    "// ============================================================================";

/// Width of the label column, so `Description:` and its continuation lines
/// share one left edge.
const LABEL_WIDTH: usize = 14;

/// Fields every header must carry.
const REQUIRED: [&str; 3] = ["Module", "Description", "Dependencies"];

/// rustfmt's default `max_width`. A header that runs past it wraps in an
/// editor and stops lining up.
const MAX_WIDTH: usize = 100;

/// The library crate's name as the root header spells it.
const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// Every `.rs` file under `root`, found iteratively, skipping build output.
fn rust_sources(root: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name == "target") {
                    continue;
                }
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

/// The `Module:` value a file at `relative` (forward slashes, relative to
/// the crate root) is required to declare.
fn module_for(relative: &str) -> Result<String> {
    if relative == "build.rs" {
        return Ok("build (build script)".to_string());
    }
    let stem = relative
        .strip_suffix(".rs")
        .ok_or_else(|| anyhow!("{relative} is not a Rust source file"))?;
    let (area, rest) = stem
        .split_once('/')
        .ok_or_else(|| anyhow!("{relative} is not under a checked directory"))?;
    let parts: Vec<&str> = rest.split('/').collect();

    match area {
        "tests" => Ok(format!("{rest} (integration test)")),
        "examples" => Ok(match parts.as_slice() {
            [name] => format!("{name} (example)"),
            [name, "src", "main"] => format!("{name} (standalone example crate)"),
            [name, "main"] => format!("{name} (example crate root)"),
            _ => format!("{} (example)", parts.join("::")),
        }),
        "src" => Ok(match parts.as_slice() {
            ["lib"] => format!("{CRATE_NAME} (library crate root)"),
            [modules @ .., "mod"] => modules.join("::"),
            _ => parts.join("::"),
        }),
        _ => Err(anyhow!("{relative} is not under a checked directory")),
    }
}

/// One field's value, with any aligned continuation lines folded in.
fn field(body: &[&str], label: &str) -> Option<String> {
    let prefix = format!("// {label}:");
    let start = body.iter().position(|line| line.starts_with(&prefix))?;
    let indent = format!("// {}", " ".repeat(LABEL_WIDTH));

    let mut value = body.get(start)?.get(prefix.len()..)?.trim().to_string();
    for line in body.get(start + 1..)? {
        let Some(rest) = line.strip_prefix(&indent) else {
            break;
        };
        value.push(' ');
        value.push_str(rest.trim());
    }
    Some(value)
}

/// Checks one file, returning a description of each problem found.
fn problems(relative: &str, text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    if lines.first() != Some(&RULE) {
        found.push(format!("{relative}: does not open with the header rule"));
        return found;
    }
    let Some(close) = lines.iter().skip(1).position(|line| *line == RULE) else {
        found.push(format!("{relative}: header rule is never closed"));
        return found;
    };
    let Some(body) = lines.get(1..=close) else {
        found.push(format!("{relative}: header body could not be read"));
        return found;
    };

    for line in body {
        if !line.starts_with("// ") && *line != "//" {
            found.push(format!(
                "{relative}: non-comment line inside header: {line}"
            ));
        }
        if line.chars().count() > MAX_WIDTH {
            found.push(format!(
                "{relative}: header line exceeds {MAX_WIDTH} columns"
            ));
        }
    }

    for label in REQUIRED {
        let Some(value) = field(body, label) else {
            found.push(format!("{relative}: header is missing `{label}:`"));
            continue;
        };
        if value.is_empty() {
            found.push(format!("{relative}: `{label}:` is empty"));
        } else if value.starts_with('[') {
            found.push(format!(
                "{relative}: `{label}:` still holds the template placeholder"
            ));
        }
    }

    match (field(body, "Module"), module_for(relative)) {
        (Some(declared), Ok(expected)) if declared != expected => found.push(format!(
            "{relative}: declares `Module: {declared}` but sits at `{expected}`"
        )),
        (_, Err(error)) => found.push(format!("{relative}: {error}")),
        (Some(_) | None, Ok(_)) => {}
    }

    found
}

#[test]
fn every_source_file_carries_a_module_header() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut failures = Vec::new();
    let mut checked = 0usize;

    for area in ["src", "tests", "examples"] {
        let dir = root.join(area);
        if !dir.is_dir() {
            continue;
        }
        for path in rust_sources(&dir)? {
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&path)?;
            failures.extend(problems(&relative, &text));
            checked += 1;
        }
    }

    let build = root.join("build.rs");
    if build.is_file() {
        failures.extend(problems("build.rs", &fs::read_to_string(build)?));
        checked += 1;
    }

    assert!(
        checked > 0,
        "found no source files to check under {}",
        root.display()
    );
    assert!(
        failures.is_empty(),
        "{} file header problem(s) across {checked} file(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
    Ok(())
}

#[test]
fn a_module_path_is_derived_from_where_the_file_sits() -> Result<()> {
    let root = format!("{CRATE_NAME} (library crate root)");
    let cases = [
        ("build.rs", "build (build script)"),
        ("src/lib.rs", root.as_str()),
        ("src/theme.rs", "theme"),
        ("src/button/mod.rs", "button"),
        ("src/button/rendering.rs", "button::rendering"),
        ("tests/dark_visuals.rs", "dark_visuals (integration test)"),
        ("examples/svg_icon_demo.rs", "svg_icon_demo (example)"),
        ("examples/stories/main.rs", "stories (example crate root)"),
        (
            "examples/stories/button_window.rs",
            "stories::button_window (example)",
        ),
        (
            "examples/ondemand/src/main.rs",
            "ondemand (standalone example crate)",
        ),
    ];
    for (path, expected) in cases {
        assert_eq!(
            module_for(path)?,
            expected,
            "wrong module path derived for {path}"
        );
    }
    Ok(())
}

#[test]
fn a_header_naming_the_wrong_module_is_rejected() -> Result<()> {
    let text = format!(
        "{RULE}\n\
         // Module:       chips\n\
         // Description:  Something.\n\
         //\n\
         // Dependencies: None.\n\
         {RULE}\n"
    );
    let found = problems("src/theme.rs", &text);
    assert!(
        found.iter().any(|problem| problem.contains("but sits at")),
        "a mismatched Module: should be reported, got {found:?}"
    );
    Ok(())
}

#[test]
fn a_missing_or_placeholder_field_is_rejected() -> Result<()> {
    let missing = format!(
        "{RULE}\n\
         // Module:       theme\n\
         // Dependencies: None.\n\
         {RULE}\n"
    );
    let found = problems("src/theme.rs", &missing);
    assert!(
        found.iter().any(|problem| problem.contains("Description:")),
        "a missing Description: should be reported, got {found:?}"
    );

    let placeholder = format!(
        "{RULE}\n\
         // Module:       theme\n\
         // Description:  [Brief description of the file's purpose]\n\
         //\n\
         // Dependencies: None.\n\
         {RULE}\n"
    );
    let found = problems("src/theme.rs", &placeholder);
    assert!(
        found.iter().any(|problem| problem.contains("placeholder")),
        "an unfilled template should be reported, got {found:?}"
    );
    Ok(())
}

#[test]
fn a_file_that_does_not_open_with_a_header_is_rejected() -> Result<()> {
    let found = problems("src/theme.rs", "//! Just docs, no banner.\n");
    assert!(
        found
            .iter()
            .any(|problem| problem.contains("does not open with")),
        "got {found:?}"
    );

    let unclosed = format!("{RULE}\n// Module:       theme\n");
    let found = problems("src/theme.rs", &unclosed);
    assert!(
        found.iter().any(|problem| problem.contains("never closed")),
        "a header that runs on into the code should be reported, got {found:?}"
    );
    Ok(())
}

#[test]
fn a_field_wrapped_onto_the_label_column_reads_as_one_value() -> Result<()> {
    let body = [
        "// Module:       theme",
        "// Description:  First half of the sentence",
        "//               and its continuation.",
        "//",
        "// Dependencies: egui",
    ];
    assert_eq!(
        field(&body, "Description"),
        Some("First half of the sentence and its continuation.".to_string()),
        "continuation lines should fold into the field they belong to"
    );
    assert_eq!(
        field(&body, "Module"),
        Some("theme".to_string()),
        "a field must not swallow the field below it"
    );
    Ok(())
}

#[test]
fn an_overlong_header_line_is_rejected() -> Result<()> {
    let long = "x".repeat(MAX_WIDTH);
    let text = format!(
        "{RULE}\n\
         // Module:       theme\n\
         // Description:  {long}\n\
         //\n\
         // Dependencies: None.\n\
         {RULE}\n"
    );
    let found = problems("src/theme.rs", &text);
    assert!(
        found.iter().any(|problem| problem.contains("columns")),
        "got {found:?}"
    );
    Ok(())
}
