#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
#  tui-spinner – CI Release Notes Generator
# ──────────────────────────────────────────────────────────────────────────────
#  Called by the release workflow AFTER the tag has been pushed.
#  The version is already bumped in Cargo.toml — this script
#  only generates CHANGELOG.md and RELEASE_NOTES.md.
#
#  Usage:
#    nu scripts/ci/release_notes.nu v0.2.0
# ──────────────────────────────────────────────────────────────────────────────

def main [raw_tag: string] {
    let version = ($raw_tag | str replace --regex '^v' '')

    print $"Generating release artifacts for v($version)…"

    # ── Generate CHANGELOG.md ─────────────────────────────────
    if (which git-cliff | is-not-empty) {
        print "  Generating CHANGELOG.md…"
        try {
            git-cliff --tag $"v($version)" --output CHANGELOG.md
            print "  ✔ CHANGELOG.md updated"
        } catch {
            print "  ⚠ git-cliff failed — skipping changelog"
        }
    } else {
        print "  ⚠ git-cliff not found — skipping changelog"
    }

    # ── Generate RELEASE_NOTES.md ─────────────────────────────
    print "  Generating RELEASE_NOTES.md…"

    mut notes = $"# tui-spinner v($version)\n\n"
    $notes = $notes + "## Installation\n\n"
    $notes = $notes + "Add `tui-spinner` to your `Cargo.toml`:\n\n"
    $notes = $notes + "```toml\n"
    $notes = $notes + "[dependencies]\n"
    $notes = $notes + $"tui-spinner = \"($version)\"\n"
    $notes = $notes + "```\n\n"
    $notes = $notes + "Or install via cargo:\n\n"
    $notes = $notes + "```sh\n"
    $notes = $notes + $"cargo add tui-spinner@($version)\n"
    $notes = $notes + "```\n\n"

    # Append changelog for this version
    if (which git-cliff | is-not-empty) {
        try {
            let cliff_notes = (git-cliff --tag $"v($version)" --unreleased --strip header | str trim)
            if ($cliff_notes | is-not-empty) {
                $notes = $notes + "## What's Changed\n\n"
                $notes = $notes + $cliff_notes
                $notes = $notes + "\n"
            }
        }
    }

    $notes | save --force RELEASE_NOTES.md
    print "  ✔ RELEASE_NOTES.md generated"
    print $"✅ Release artifacts ready for v($version)"
}
