# tui-spinner — Agent Rules & Best Practices

This file is read by Zed AI and other agents working on this codebase.
Follow every rule here before writing or suggesting any code.

---

## 1. Project Overview

`tui-spinner` is a Rust library of animated spinner widgets for
[Ratatui](https://ratatui.rs) TUI applications.

**Public widgets (all re-exported from `src/lib.rs`):**

| Widget | File | What it does |
|--------|------|--------------|
| `LinearSpinner` | `src/linear_spinner.rs` | Horizontal scroll / vertical bounce |
| `SquareSpinner` | `src/square_spinner.rs` | Square braille-arc (legacy alias) |
| `RectSpinner` | `src/rect_spinner.rs` | Configurable-rectangle braille-arc |
| `CircleSpinner` | `src/circle_spinner.rs` | Circular braille-arc |
| `BarSpinner` | `src/bar_spinner.rs` | Solid bar with bouncing/looping glow arc |
| `FluxSpinner` | `src/flux_spinner.rs` | Single-char glyph cycling through frames |

**Shared internal macros** live in `src/macros.rs` and are loaded
via `#[macro_use] mod macros;` in `src/lib.rs`:
- `impl_styled_for!(T)` — generates `impl Styled`
- `impl_widget_via_ref!(T)` — generates `impl Widget for T { delegate to &T }`
- `render_spinner_body!(self, area, buf, lines)` — standard Widget render body

---

## 2. Quality Gate — Run Before Every Commit

```sh
cargo fmt                                # format first
cargo clippy --all-features -- -D warnings   # zero warnings required
cargo test --all-features               # all tests must pass
RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo +nightly doc --no-deps --all-features
```

The `just check-all` recipe runs fmt-check + clippy + test in sequence.

---

## 3. Rust Style & Conventions

### Builder pattern
Every widget uses an owned builder pattern — all setters return `Self`:
```rust
#[must_use]
pub fn arc_color(mut self, color: Color) -> Self {
    self.arc_color = color;
    self
}
```
- Always annotate builder methods with `#[must_use]`.
- Simple setters (one-liner `self.field = value; self`) are preferred over
  macro-generated builders when doc comments add value.

### Clippy lints
The crate enables `#![warn(clippy::pedantic)]`.  Common suppressions:
```rust
#[allow(clippy::cast_possible_truncation)]   // intentional usize→u16 in render
#[allow(clippy::cast_sign_loss)]             // geometry always non-negative
#[allow(clippy::cast_possible_wrap)]         // radii never overflow isize
#[allow(clippy::cast_precision_loss)]        // f64 sufficient for pixel math
```
Add `#[allow]` at **function level**, not crate/module level.

### Naming
- Widget structs: `PascalCase` ending in `Spinner` (e.g. `BarSpinner`).
- Enums for options: `PascalCase` ending in the concept (`BarStyle`, `BarMotion`, `BarTrack`, `Spin`, `Centre`).
- Internal engine structs: `PascalCase` without `Spinner` suffix (e.g. `RectEngine`).
- No brand names (`Zed`, `Claude`) in public API names or doc strings.

### Error handling
Widgets are **stateless renderers** — they never panic or return errors.
Guard against zero-area rendering at the top of every `Widget::render` impl:
```rust
if area.area() == 0 { return; }
```

---

## 4. Widget Architecture

Every spinner follows the same four-layer structure:

```
Public struct (BarSpinner) — builder API, stores config
  └─ build_lines() / render_lines() — computes Line<'static> from config
       └─ internal engine (RectEngine, etc.) — pure animation logic
            └─ walk() + render_lines() — step state + produce spans
```

The `Widget for &T` impl calls `build_lines` and hands the result to
`render_spinner_body!`.  The owned `Widget for T` delegates via
`impl_widget_via_ref!(T)`.

### `impl Styled`
Always use the shared macro:
```rust
impl_styled_for!(BarSpinner<'_>);
```

### `impl Widget`
For the owned delegator:
```rust
impl_widget_via_ref!(BarSpinner<'_>);
```
For the reference impl, use `render_spinner_body!` unless the widget has
unique width-resolution logic (like `BarSpinner`) or branches on a variant
(like `LinearSpinner`).

---

## 5. Testing Requirements

- Every new public builder method must have at least one test asserting the
  field was stored correctly.
- Every new rendering path must have a `widget_renders_without_panic` test
  using `ratatui::backend::TestBackend`.
- Edge cases to always cover: `width = 0`, `height = 0`, zero-area `Rect`,
  values larger than the container.
- Doc-tests on all public methods with `# Examples` sections must compile.
- Run `cargo test --all-features` — doc-tests and unit tests both must pass.

---

## 6. Examples

Each widget has a dedicated example file in `examples/`:

| File | Widget |
|------|--------|
| `examples/spinner.rs` | Combined overview (all widgets) |
| `examples/square_spinner.rs` | SquareSpinner |
| `examples/circle_spinner.rs` | CircleSpinner |
| `examples/linear_spinner.rs` | LinearSpinner |
| `examples/bar_spinner.rs` | BarSpinner |
| `examples/flux_spinner.rs` | FluxSpinner |

### Example conventions
- Every example is **self-contained**: macros (`sty!`, `sp!`, `section_block!`)
  are defined inline at the top — no shared utility module.
- All examples use the standard event loop:
  `(delta.as_millis() / 80).max(1)` steps per frame, poll at 16 ms (~60 fps).
- Examples must compile cleanly with `cargo build --examples`.
- No `unused_import` or `unused_macros` warnings.

### VHS tapes
Each example has a corresponding tape in `examples/vhs/`:
```
Set Theme "Catppuccin Mocha"
Set Width 1400 / Set Height 900 / Set FontSize 14

Hide
Type "cargo run --example foo"
Enter
Sleep 12s          # wait out compilation invisibly
Show
Sleep 8s           # record the running TUI
Type "q"
Sleep 500ms
```
`Hide`/`Show` is mandatory — never show the compilation phase in the GIF.
GIF files in `examples/vhs/generated/` are tracked via **Git LFS**.

---

## 7. Git Workflow

### Remotes
```
origin          git@github.com:sorinirimies/tui-spinner.git   (primary)
gitea           ssh://git@192.168.1.204:30009/sorin/tui-spinner.git
gitea_starscream gitea@192.168.1.44:sorin/tui-spinner.git
```

### Push helpers (justfile)
```sh
just push-all               # push main to all three remotes
just sync-all-gitea         # force-sync both Gitea instances from origin
just push-all-force         # force-push to all three remotes
```

### Release
```sh
just bump 0.2.0             # runs quality gate, bumps version, tags
just push-release-all       # push commit + tag to all remotes
```

### Commit message style
Follow Conventional Commits:
```
feat: add BarMotion::Loop — continuous sweep mode
fix:  BarSpinner Loop wraps arc at both edges simultaneously
refactor: shared macros impl_styled_for!, impl_widget_via_ref!
docs: update README for 0.2.x — BarStyle/BarMotion/BarTrack tables
chore(deps): update ratatui 0.29 → 0.30
```

---

## 8. Dependencies

| Crate | Version | Role |
|-------|---------|------|
| `ratatui` | `0.30` | TUI framework (lib + dev-dep) |
| `crossterm` | `0.29` | Terminal backend (dev-dep only) |
| `color-eyre` | `0.6` | Error reporting in examples |

- `ratatui` with `default-features = false` in `[dependencies]` (no terminal
  backend in the library itself).
- `ratatui` with `features = ["crossterm"]` in `[dev-dependencies]` (examples
  need a backend).
- When updating deps use `cargo upgrade --incompatible allow` followed by
  `cargo update` to refresh the lock file.

---

## 9. Public API Design Principles

1. **Stateless** — widgets take a `tick: u64` counter; no mutable state.
2. **Builder pattern** — every option is a fluent setter returning `Self`.
3. **Sane defaults** — `::new(tick)` produces a useful spinner with no further
   configuration.
4. **Preset constructors** — for common configurations provide named constructors
   like `BarSpinner::zed(tick)`, `BarSpinner::claude(tick)`.
5. **No brand names** — avoid `Zed`, `Claude`, `Copilot` etc. in public API.
6. **Consistent enum naming** — direction/style enums are always `PascalCase`
   and describe the concept, not the tool.
7. **`char_size()` method** — every widget that has a deterministic rendered
   size exposes `fn char_size(&self) -> (u16, u16)` or similar.
8. **`#[must_use]`** — every builder setter and every `::new` constructor.

---

## 10. Nushell Scripts

Utility scripts live in `scripts/`:

| Script | Purpose |
|--------|---------|
| `bump_version.nu` | Quality gate + version bump + tag |
| `release_prepare.nu` | CHANGELOG via git-cliff + release notes |
| `setup_gitea.nu` | Add/update a Gitea remote (`--remote <name>`) |
| `upgrade_deps.nu` | `cargo upgrade` + cross-check |
| `version.nu` | Print current crate version |

When reading files in Nushell scripts, always use `open --raw <file>` for
`.md` and other non-structured files to avoid type errors.

---

## 11. Things to Avoid

- ❌ Don't use brand names (`Zed`, `Claude`) in public API or doc strings.
- ❌ Don't add `#[macro_export]` to internal macros — use `#[macro_use] mod macros`.
- ❌ Don't use `Constraint::Percentage` for fixed-height UI elements that
  should stay compact — prefer `Constraint::Length(n)`.
- ❌ Don't add `use ratatui::style::Styled` to spinner files — the macro
  uses the fully-qualified path.
- ❌ Don't use Python-style unicode escapes (`\uXXXX`) in Rust string literals
  — use literal Unicode characters or `\u{XXXX}`.
- ❌ Don't leave unused imports after refactoring — `cargo clippy -D warnings`
  will catch them.
- ❌ Don't commit stale GIFs — regenerate after significant example changes.
