# egui-material3 (freeknet fork): working notes for agents

Material Design 3 widgets for [egui](https://github.com/emilk/egui) 0.37,
as a library crate. This is the `freeknet/egui-material3` fork of
`nikescar/egui-material3`. Everything in this file applies to the
`freeknet` branch; upstream has none of these rules, and they must not
leak into pull requests sent there.

**Read these before changing anything:**

| Doc | When |
|---|---|
| [`README.md`](README.md) | What the crate is and how it is used. |
| [`SECURITY.md`](SECURITY.md) | Before touching anything that handles untrusted input (CSV, JSON themes, SVG, URLs, data URLs). |
| [`TROUBLESHOOT.md`](TROUBLESHOOT.md) | Build problems. |

## Commands

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --features ondemand,spreadsheet,svg,system-theme -- -D warnings
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --features ondemand,spreadsheet,svg,system-theme --lib --tests --examples
cargo test --locked --features ondemand,spreadsheet,svg,system-theme --doc
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --features ondemand,spreadsheet,svg,system-theme
cargo deny --all-features check
cargo run --example stories --features spreadsheet,svg,system-theme   # the component gallery
```

All of these are clean on `freeknet` and enforced by CI
(`.github/workflows/ci.yml`, jobs `rustfmt`, `cargo-deny`,
`clippy and test` on Linux and Windows, `rustdoc`, and the `ci-ok`
aggregate). A warning is a build failure. Set `CARGO_BUILD_JOBS=4` on
shared machines.

CI builds the `ondemand`, `spreadsheet`, `svg` and `system-theme`
features, and clippy once more with none (no feature is on by default:
`svg` and `system-theme` keep resvg and dark-light, and the unmaintained
crates under them, out of downstream builds that do not ask for them;
gate their code with `cfg(feature = ...)` and give the off path a
documented fallback). The `svg_*` features
are not built there: `build.rs` downloads thousands of SVG files from the
GitHub API for them, which an unauthenticated runner is rate-limited out
of. Build them locally when you touch `svg_emoji.rs` or `build.rs`.

The doctests run in CI on Linux (slow on Windows: one binary per
snippet). Snippets compile; use `no_run` for ones that need a window,
the network or the file system, and hidden `# egui::__run_test_ui(|ui| {`
scaffolding for ones that need a `ui`.

## Branch model

- **`freeknet`** is the long-lived integration branch, cut from
  `port/egui-0.37`. Downstream apps depend on it, and the Material 3 work
  packages (see the audit: WP-0 foundation first, then WP-1..17) branch
  from it and merge back into it.
- **Fork-only house rules live only on `freeknet`:** the toolchain pin,
  CI, `deny.toml`, the `[lints]` block, header banners, `Cargo.lock`,
  and these agent docs.
- **Changes upstream would welcome** (bug fixes, panics, unsafe, clippy,
  doctests, dependency advisories) go on a **topic branch cut from
  `port/egui-0.37`** (or stacked on another topic branch when they depend
  on it), are opened as a PR to `nikescar/egui-material3` from
  `freeknet:<branch>`, and are merged into `freeknet` with
  `git merge --no-ff`. Keep each PR focused; follow upstream's commit
  style (`fix(scope): ...`, `feat(scope): ...`, `docs: ...`, `test: ...`).
  A topic branch must build and pass on its own *without* the fork's
  rules, so it never relies on `Cargo.lock`, `anyhow` or the lints.
- When a topic branch is merged upstream, nothing needs to happen here;
  when upstream moves, merge `upstream/main` (or the port branch) into
  `freeknet`, never rebase it.

## Material 3 values come from Compose

Every Material 3 token value (colour roles, type scale, shape, elevation,
state-layer opacity, motion durations, easings and springs, component
sizes) must come from the Jetpack Compose material3 token files,
`androidx.compose.material3.tokens` on `androidx-main` (`*Tokens.kt`).
Not from the M3 website, Flutter, MDC-Web, or memory. Cite the token
file and name in a comment or test next to the value. Where the Baseline
and Expressive token sets disagree, say which one you implement.

## Editing UI without looking at it

Agents verify headlessly: render with `egui::Context::run_ui` in a test
and assert on geometry, colours or (from WP-0) the AccessKit tree and
kittest snapshots. Do not judge widgets from screenshots, and do not drive
a desktop. Whether a widget looks right is the human's call: point them
at the story in `examples/stories/`.

## Commit only what you changed

Several agents may work in this repository at once. Check `git status`
before staging. Use `git commit -- path/to/file`; never `git add -A`,
`git add .` or `git commit -a`. Commit subjects use conventional
prefixes (`feat:`, `fix:`, `perf:`, `refactor:`, `docs:`, `test:`,
`build:`, `ci:`, `style:`). No AI or tool attribution anywhere: no
`Co-authored-by` trailers, no "generated with" lines in commits or PR
bodies.

Line endings are `.gitattributes`' problem: every text file is stored and
checked out as LF. Do not convert endings by hand.

`CHANGELOG.md` is generated by git-cliff (`cliff.toml`) from commit
subjects at release time, so it is not edited by hand; a good commit
subject is the changelog entry. Do not bump the crate version.

---

## Rules the build enforces

### Lints

The `[lints]` block in `Cargo.toml` applies to the library, tests,
examples and build script. Denied, among others: `unwrap_used`,
`expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`,
`indexing_slicing`, `string_slice`, `wildcard_enum_match_arm`,
`missing_docs`, `unreachable_pub`, `unsafe_code`, `allow_attributes`,
and every rustc warning (`warnings = "deny"`, so a deprecated egui API
fails the build too).

Use `?`, `let ... else`, `.get()`, `if let`, or an explicit fallback. A
poisoned `Mutex` recovers with `.unwrap_or_else(PoisonError::into_inner)`.
Tests return `Result` (`anyhow::Result<()>` in integration tests) and
every `assert!` carries a message. Release builds use `panic = "abort"`,
so a panic in a widget aborts the host app.

Suppress a lint only with `#[expect(lint, reason = "...")]`, never
`#[allow]`: `expect` fails the build when the lint stops firing.

### Lint decisions for a public widget library

These are deliberate; do not "fix" them, and add to this list if you make
another:

- **Generated icon tables are exempt from `missing_docs`.**
  `src/material_symbol.rs` (about 2200 codepoint constants) and
  `src/noto_emoji.rs` (about 1400) carry one module-level
  `#![expect(missing_docs, reason = ...)]`. Each constant is named after
  its icon or Unicode name; a doc line per constant would only repeat it.
  The functions in those modules are documented.
- **`type_complexity` and `too_many_arguments` are expected, with a
  reason, on existing public builders** (boxed UI callbacks, flat
  constructor parameter lists). Changing those signatures would break
  the public API for a lint; new code should prefer a named callback type
  or a parameter struct instead.
- **`missing_docs` covers public fields and variants too.** Builder
  structs keep their fields private where possible; a public field gets a
  doc line saying what it controls.
- **`unreachable_pub`:** items in private modules are `pub(crate)`, so
  `pub` always means "part of the crate's API". This includes the story
  modules of the `stories` example.
- **`wildcard_enum_match_arm`:** match arms name every variant of the
  crate's own enums. A foreign `#[non_exhaustive]` enum is the one place a
  `_` arm is needed; put `#[expect(clippy::wildcard_enum_match_arm,
  reason = "...")]` on that match.
- **`indexing_slicing` and `string_slice` are denied** (beyond the usual
  house list): indexing a widget's item list or a user string is exactly
  how a library panics in someone else's app. Fixed-size array indexing
  with a constant index is allowed by the lint itself.
- **No `unsafe`.** The crate has none; `unsafe_code` is denied.

### Every source file opens with the header banner

A ruled block naming `Module:`, `Description:` and `Dependencies:`, then a
blank line, then the module's `//!` docs:

```rust
// ============================================================================
// Module:       button::rendering
// Description:  Rendering implementation for MaterialButton.
//
// Dependencies: egui; super
// ============================================================================

//! Rendering implementation for MaterialButton
```

`tests/header_check.rs` walks `build.rs`, `src/`, `tests/` and
`examples/` and fails for a file missing one, so a new file needs its
header in the same commit. `Module:` is checked against the path:
`src/lib.rs` is `egui-material3 (library crate root)`, `src/foo/mod.rs`
is `foo`, `src/foo/bar.rs` is `foo::bar`, `tests/x.rs` is
`x (integration test)`, `examples/x.rs` is `x (example)`,
`examples/stories/main.rs` is `stories (example crate root)`,
`examples/stories/x.rs` is `stories::x (example)`, and
`examples/x/src/main.rs` is `x (standalone example crate)`. The banner
is plain `//`, not `//!`.

### The toolchain is pinned, once

`rust-toolchain.toml` pins `1.99.1` with clippy and rustfmt, and every
workflow's `toolchain: "..."` line must name the same version;
`tests/toolchain_pin.rs` fails if they drift. Bump them together, in
their own commit. Never `@stable`. The pin is the development toolchain,
not the crate's MSRV.

### Workflows are pinned and least-privilege

`tests/workflow_hygiene.rs` fails unless every action is pinned to a full
commit SHA with a `# version` comment, every workflow declares top-level
`permissions:`, every job has `timeout-minutes:`, and every resolving
cargo command passes `--locked`. `Cargo.lock` is committed on this branch
for that reason.

### Every dependency passes `cargo deny`

`deny.toml` allows a fixed list of permissive licences (the crate itself
is MIT OR Apache-2.0), denies unknown registries and git sources (the one
allowed git source is the resvg fork that `.cargo/config.toml` patches
in), and fails on advisories. Adding a crate under a licence not on the
list is a decision, not a fix: add it in its own commit, saying why. An
ignored advisory carries a note saying what pulls it in and what would
clear it.

### Constants live with what they govern

There is no `constants.rs`. Use a function-local `const` first, then a
module-private one, then `pub` in the module that owns the concept. Token
values from WP-0 live in the `tokens` module that owns them.

---

## The theme is per context

Widgets read colours with `MaterialTheme::of(ctx).color(ColorRole::..)`
(and the `tokens`/`paint`/`a11y` helpers), never the process-wide theme.
A context gets its theme from `apply_theme_context` (visuals, type scale
and widget colours) or `MaterialTheme::set` (widget colours only); one
with none falls back to the process-wide default that `load_themes` and
`apply_theme` manage.

`get_global_color`, `get_global_color_role`, `get_global_theme`,
`update_global_theme`, `get_theme_mode`, `set_theme_mode`,
`get_contrast_level`, `set_contrast_level` and
`DrawerThemeData::material2_defaults` are `#[deprecated]` shims kept for
downstream callers: they read or change that default. Do not call them
from `src/`; `tests/tokens.rs` (`no_widget_reads_the_global_theme`) fails
the build if a file other than `theme.rs` does, and
`tests/theme_isolation.rs` checks that two contexts with different themes
paint only their own colours. Tests theme their own context (see
`tests/support`), so they run in parallel, examples included
(`--examples`).
