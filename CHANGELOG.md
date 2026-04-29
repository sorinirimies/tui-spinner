# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]
### ✨ Features
- feat: BarStyle enum (Block/Shade/Dot/Diamond/Square); example sections
- feat: 4 more BarStyles (Progress/Thick/Wave/Pip) from tui-slider symbols; 4x4 grid
- feat: BarMotion::Loop — continuous sweep mode for BarSpinner
- feat: trio helper shows → ← ⟳ per style; BarSpinner row in spinner.rs
### 🐛 Bug Fixes
- fix: bar_spinner example — 1 CW + 1 CCW per concept, distinct patterns
- fix: BarMotion::Loop — modular phase-in/out at both edges simultaneously
- fix: Bounce vs Loop shown as single CW bars so motion difference is obvious
## 0.1.2 - 2026-04-28
### ✨ Features
- feat: BarSpinner — BarTrack + fade_width; Hide compile in all VHS tapes
- feat: BarSpinner arc_char + presets; 3-page example with ← → navigation
### 🐛 Bug Fixes
- fix: open CHANGELOG.md with --raw in release_prepare.nu
### 🔧 Chores
- chore: bump version to 0.1.2
**Full Changelog**: https://github.com/sorinirimies/tui-spinner/compare/v0.1.1...v0.1.2
## 0.1.1 - 2026-04-28
### ♻️ Refactor
- Refactor Square spinner: simplify, remove Narrow, update styles
- refactor: rename ZedSpinner -> DotSpinner; add Spin direction
- refactor: rename DotSpinner -> FluxSpinner; simplify example
- refactor: clean example — macros, unified Tile, no glyph row, Length(4) cells
### ✨ Features
- feat: add RectangularSpinner — Zed/Claude-style bouncing braille arc
- feat: add ZedSpinner; rename RectangularSpinner → BarSpinner
- feat: FluxFrames presets + .frames() builder; reorganise example
- feat: add 5 new FluxFrames presets; rebuild example around preset/direction story
- feat: replace ARROWS with BOUNCE/HALF/SQUARE/DICE; 4×4 grid with live custom tiles
- feat: 5×4 compact grid; add BAR/CORNERS presets + SHADE/MUSIC custom tiles
- feat: split spinner.rs into 3 focused examples; add 5 FluxFrames presets; 4-row grid
### ➕ Added
- Add Gitea workflows, scripts, and Nushell test suite
### 🐛 Bug Fixes
- fix: resolve all clippy warnings; add gitea/gitea_starscream remotes; update nightly dep workflow
- fix: redesign RectangularSpinner as solid bouncing bar (true Zed/Claude style)
- fix: replace row list with 4×3 grid in flux_spinner example
- fix: vertically centre circle_spinner rows; sync justfile with gitkraft
- fix: release-* recipes must call (bump version) not bare bump
### 📚 Documentation
- docs: add VHS tapes + GIFs for all 5 examples; rewrite README with previews
### 📦 Other Changes
- Initial commit
- square, narrow, circle and linear spinner implementation
### 🔧 Chores
- chore: remove stray disktest.txt
- chore: bump version to 0.1.1
