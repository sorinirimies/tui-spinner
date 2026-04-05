#!/usr/bin/env nu
# Prepare a release for tui-spinner.
# Updates Cargo.toml, generates changelog, release notes with install instructions.
# Usage: nu scripts/release_prepare.nu v0.2.0

def main [
    tag: string  # Release tag (e.g. "v0.2.0")
] {
    let reset = (ansi reset)
    let red = (ansi red)
    let green = (ansi green)
    let yellow = (ansi yellow)
    let cyan = (ansi cyan)

    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print $"($cyan)  tui-spinner release preparation($reset)"
    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print ""

    # ── validate tag format ──────────────────────────────────────────
    if not ($tag | str starts-with "v") {
        print $"($red)error:($reset) tag must start with 'v' \(got: ($tag)\)"
        exit 1
    }

    let version = ($tag | str substring 1..)
    let valid = ($version | parse --regex '^(\d+)\.(\d+)\.(\d+)(-[\w.]+)?$' | length) > 0
    if not $valid {
        print $"($red)error:($reset) tag must be in vX.Y.Z format \(got: ($tag)\)"
        exit 1
    }

    let current_version = (open Cargo.toml | get package.version)
    print $"($cyan)tag:             ($reset) ($tag)"
    print $"($cyan)version:         ($reset) ($version)"
    print $"($cyan)current version: ($reset) ($current_version)"
    print ""

    # ── update Cargo.toml version ────────────────────────────────────
    print $"($cyan)▸ updating Cargo.toml to ($version) ...($reset)"
    let cargo_toml = (open Cargo.toml --raw)
    let updated = ($cargo_toml | str replace --regex 'version\s*=\s*"[^"]+"' $'version = "($version)"')
    $updated | save --force Cargo.toml
    print $"($green)  ✓ Cargo.toml updated($reset)"

    # ── sync Cargo.lock ──────────────────────────────────────────────
    print $"($cyan)▸ syncing Cargo.lock ...($reset)"
    let update_result = (do { cargo update --workspace } | complete)
    if $update_result.exit_code == 0 {
        print $"($green)  ✓ Cargo.lock synced($reset)"
    } else {
        print $"($yellow)  ⚠ cargo update returned non-zero, continuing ...($reset)"
    }

    # ── regenerate full CHANGELOG.md via git-cliff ───────────────────
    let has_cliff = (which git-cliff | length) > 0
    if $has_cliff {
        print $"($cyan)▸ generating CHANGELOG.md via git-cliff ...($reset)"
        let cliff_result = (do { git-cliff --output CHANGELOG.md } | complete)
        if $cliff_result.exit_code == 0 {
            let cl_size = (open CHANGELOG.md | str length)
            print $"($green)  ✓ CHANGELOG.md generated \(($cl_size) chars\)($reset)"
        } else {
            print $"($yellow)  ⚠ git-cliff failed to generate CHANGELOG.md($reset)"
        }

        # ── generate per-release diff ────────────────────────────────
        print $"($cyan)▸ generating release diff ...($reset)"
        let diff_file = $"_release_diff_($tag).md"
        let diff_result = (do { git-cliff --latest --strip header --output $diff_file } | complete)
        if $diff_result.exit_code == 0 and ($diff_file | path exists) {
            let diff_size = (open $diff_file | str length)
            if $diff_size > 0 {
                print $"($green)  ✓ ($diff_file) generated \(($diff_size) chars\)($reset)"
            } else {
                print $"($yellow)  ⚠ release diff was empty, trying --unreleased ...($reset)"
                let fallback = (do { git-cliff --unreleased --strip header --output $diff_file } | complete)
                if $fallback.exit_code != 0 {
                    $"## ($version)\n\nRelease ($tag)\n" | save --force $diff_file
                    print $"($yellow)  ⚠ wrote fallback diff($reset)"
                }
            }
        } else {
            print $"($yellow)  ⚠ git-cliff --latest failed, trying --unreleased ...($reset)"
            let fallback = (do { git-cliff --unreleased --strip header --output $diff_file } | complete)
            if $fallback.exit_code != 0 {
                $"## ($version)\n\nRelease ($tag)\n" | save --force $diff_file
                print $"($yellow)  ⚠ wrote fallback diff($reset)"
            }
        }
    } else {
        print $"($yellow)  ⚠ git-cliff not found — skipping changelog generation($reset)"
    }

    # ── build RELEASE_NOTES.md ───────────────────────────────────────
    print $"($cyan)▸ building RELEASE_NOTES.md ...($reset)"

    mut notes = $"# tui-spinner ($tag)\n\n"

    # include diff content if available
    let diff_file = $"_release_diff_($tag).md"
    if ($diff_file | path exists) {
        let diff_content = (open $diff_file --raw | str trim)
        if ($diff_content | str length) > 0 {
            $notes = $notes + $diff_content + "\n\n"
        }
    }

    # install instructions
    $notes = $notes + "## Installation\n\n"
    $notes = $notes + "Add `tui-spinner` to your `Cargo.toml`:\n\n"
    $notes = $notes + "```toml\n"
    $notes = $notes + "[dependencies]\n"
    $notes = $notes + $"tui-spinner = \"($version)\"\n"
    $notes = $notes + "```\n\n"
    $notes = $notes + "Or install via cargo:\n\n"
    $notes = $notes + "```sh\n"
    $notes = $notes + $"cargo add tui-spinner@($version)\n"
    $notes = $notes + "```\n"

    $notes | save --force RELEASE_NOTES.md
    let notes_size = ($notes | str length)
    print $"($green)  ✓ RELEASE_NOTES.md generated \(($notes_size) chars\)($reset)"

    # ── clean up temp files ──────────────────────────────────────────
    print $"($cyan)▸ cleaning up temp files ...($reset)"
    if ($diff_file | path exists) {
        rm $diff_file
        print $"($green)  ✓ removed ($diff_file)($reset)"
    }
    print $"($green)  ✓ cleanup complete($reset)"

    # ── summary ──────────────────────────────────────────────────────
    print ""
    print $"($green)═══════════════════════════════════════════════($reset)"
    print $"($green)  release ($tag) prepared!($reset)"
    print $"($green)═══════════════════════════════════════════════($reset)"
    print ""
    print $"($cyan)artifacts:($reset)"
    print $"  • Cargo.toml       — version set to ($version)"
    print $"  • Cargo.lock       — synced"
    if $has_cliff {
        print $"  • CHANGELOG.md     — regenerated"
    }
    print $"  • RELEASE_NOTES.md — with install instructions"
    print ""
    print $"($cyan)next steps:($reset)"
    print $"  git add Cargo.toml Cargo.lock CHANGELOG.md RELEASE_NOTES.md"
    print $"  git commit -m \"chore: prepare release ($tag)\""
    print $"  git tag -a ($tag) -m \"Release ($tag)\""
    print $"  git push origin main --tags"
    print $"  git push gitea main --tags    # if gitea remote exists"
    print $"  cargo publish"
    print ""
}
