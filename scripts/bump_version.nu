#!/usr/bin/env nu
# Bump the crate version in Cargo.toml, run checks, commit, and tag.
# Usage: nu scripts/bump_version.nu 0.2.0 [--yes]

def main [
    new_version: string  # The new version string (X.Y.Z or X.Y.Z-suffix)
    --yes (-y)           # Skip confirmation prompts
] {
    let reset = (ansi reset)
    let red = (ansi red)
    let green = (ansi green)
    let yellow = (ansi yellow)
    let cyan = (ansi cyan)

    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print $"($cyan)  tui-spinner version bump($reset)"
    print $"($cyan)═══════════════════════════════════════════════($reset)"

    # ── validate version format ──────────────────────────────────────
    let valid = ($new_version | parse --regex '^(\d+)\.(\d+)\.(\d+)(-[\w.]+)?$' | length) > 0
    if not $valid {
        print $"($red)error:($reset) invalid version format: ($new_version)"
        print $"       expected X.Y.Z or X.Y.Z-suffix"
        exit 1
    }

    # ── read current version ─────────────────────────────────────────
    let current_version = (open Cargo.toml | get package.version)
    print $"($cyan)current version:($reset) ($current_version)"
    print $"($cyan)new version:    ($reset) ($new_version)"

    if $current_version == $new_version {
        print $"($yellow)warning:($reset) new version is the same as current version"
    }

    # ── confirmation ─────────────────────────────────────────────────
    if not $yes {
        let answer = (input $"($yellow)proceed with bump? \(y/N\):($reset) ")
        if ($answer | str downcase) != "y" {
            print $"($yellow)aborted.($reset)"
            exit 0
        }
    }

    # ── update Cargo.toml ────────────────────────────────────────────
    print $"($cyan)▸ updating Cargo.toml ...($reset)"
    let cargo_toml = (open Cargo.toml --raw)
    let updated = ($cargo_toml | str replace --regex 'version\s*=\s*"[^"]+"' $'version = "($new_version)"')
    $updated | save --force Cargo.toml
    print $"($green)  ✓ Cargo.toml updated($reset)"

    # ── sync Cargo.lock ──────────────────────────────────────────────
    print $"($cyan)▸ syncing Cargo.lock ...($reset)"
    let update_result = (do { cargo update --workspace } | complete)
    if $update_result.exit_code != 0 {
        print $"($red)  ✗ cargo update failed($reset)"
        print $"($red)    ($update_result.stderr | str trim)($reset)"
        exit 1
    }
    print $"($green)  ✓ Cargo.lock synced($reset)"

    # ── cargo fmt ────────────────────────────────────────────────────
    print $"($cyan)▸ running cargo fmt ...($reset)"
    let fmt_result = (do { cargo fmt } | complete)
    if $fmt_result.exit_code != 0 {
        print $"($red)  ✗ cargo fmt failed($reset)"
        print $"($red)    ($fmt_result.stderr | str trim)($reset)"
        exit 1
    }
    print $"($green)  ✓ formatted($reset)"

    # ── cargo clippy ─────────────────────────────────────────────────
    print $"($cyan)▸ running cargo clippy ...($reset)"
    let clippy_result = (do { cargo clippy -- -D warnings } | complete)
    if $clippy_result.exit_code != 0 {
        print $"($red)  ✗ clippy found issues($reset)"
        print $"($red)    ($clippy_result.stderr | str trim)($reset)"
        exit 1
    }
    print $"($green)  ✓ clippy clean($reset)"

    # ── cargo test ───────────────────────────────────────────────────
    print $"($cyan)▸ running cargo test ...($reset)"
    let test_result = (do { cargo test } | complete)
    if $test_result.exit_code != 0 {
        print $"($red)  ✗ tests failed($reset)"
        print $"($red)    ($test_result.stderr | str trim)($reset)"
        exit 1
    }
    print $"($green)  ✓ tests passed($reset)"

    # ── changelog (optional) ─────────────────────────────────────────
    let has_cliff = (which git-cliff | length) > 0
    if $has_cliff {
        print $"($cyan)▸ generating CHANGELOG.md via git-cliff ...($reset)"
        let cliff_result = (do { git-cliff --output CHANGELOG.md } | complete)
        if $cliff_result.exit_code != 0 {
            print $"($yellow)  ⚠ git-cliff failed, skipping changelog($reset)"
        } else {
            print $"($green)  ✓ CHANGELOG.md updated($reset)"
        }
    } else {
        print $"($yellow)  ⚠ git-cliff not found, skipping changelog($reset)"
    }

    # ── git commit ───────────────────────────────────────────────────
    print $"($cyan)▸ committing changes ...($reset)"
    let add_result = (do { git add Cargo.toml Cargo.lock } | complete)
    if $add_result.exit_code != 0 {
        print $"($red)  ✗ git add failed($reset)"
        print $"($red)    ($add_result.stderr | str trim)($reset)"
        exit 1
    }
    if ($has_cliff and ("CHANGELOG.md" | path exists)) {
        do { git add CHANGELOG.md } | complete
    }
    let commit_result = (do { git commit -m $"chore: bump version to ($new_version)" } | complete)
    if $commit_result.exit_code != 0 {
        print $"($red)  ✗ git commit failed($reset)"
        print $"($red)    ($commit_result.stderr | str trim)($reset)"
        exit 1
    }
    print $"($green)  ✓ committed($reset)"

    # ── git tag ──────────────────────────────────────────────────────
    let tag = $"v($new_version)"
    print $"($cyan)▸ creating tag ($tag) ...($reset)"
    let tag_result = (do { git tag -a $tag -m $"Release ($tag)" } | complete)
    if $tag_result.exit_code != 0 {
        print $"($red)  ✗ git tag failed($reset)"
        print $"($red)    ($tag_result.stderr | str trim)($reset)"
        exit 1
    }
    print $"($green)  ✓ tagged ($tag)($reset)"

    # ── summary ──────────────────────────────────────────────────────
    print ""
    print $"($green)═══════════════════════════════════════════════($reset)"
    print $"($green)  version bumped: ($current_version) → ($new_version)($reset)"
    print $"($green)═══════════════════════════════════════════════($reset)"
    print ""
    print $"($cyan)next steps:($reset)"
    print $"  git push origin main --tags"
    print $"  git push gitea main --tags        # if gitea remote exists"
    print $"  cargo publish"
    print ""
}
