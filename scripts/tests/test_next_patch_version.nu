#!/usr/bin/env nu
# test_next_patch_version.nu — Tests for the "next patch version" logic used
# by the nightly deps-update workflows (.github/workflows/deps-update.yml and
# .gitea/workflows/deps-update.yml) to auto-bump the crate version before
# tagging a release.
#
# The workflows compute this in bash as:
#   CURRENT=$(nu scripts/version.nu)
#   IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"
#   NEXT="${MAJOR}.${MINOR}.$((PATCH + 1))"
#
# This test replicates that logic in Nushell so it can be verified in the
# regular test suite without needing bash or a live CI run.
# Usage: nu scripts/tests/test_next_patch_version.nu

use runner.nu [run_tests, assert_true, assert_eq, assert_str_contains]

# Mirrors the bash logic from the deps-update workflows.
def next_patch_version [current: string]: nothing -> string {
    let parts = ($current | split row ".")
    let major = ($parts | get 0)
    let minor = ($parts | get 1)
    # Strip any pre-release suffix (e.g. "0-alpha") before incrementing.
    let patch = ($parts | get 2 | split row "-" | first | into int)
    $"($major).($minor).($patch + 1)"
}

def main [] {
    let reset = (ansi reset)
    let cyan = (ansi cyan)
    let green = (ansi green)
    let red = (ansi red)

    print $"($cyan)── test_next_patch_version.nu ─────────────────($reset)"

    let results = run_tests [
        {
            name: "test patch increments by one",
            fn: {||
                assert_eq (next_patch_version "0.4.0") "0.4.1"
            }
        },
        {
            name: "test minor/major are preserved",
            fn: {||
                assert_eq (next_patch_version "1.2.3") "1.2.4"
                assert_eq (next_patch_version "12.34.56") "12.34.57"
            }
        },
        {
            name: "test patch rolls over past 9",
            fn: {||
                assert_eq (next_patch_version "0.4.9") "0.4.10"
            }
        },
        {
            name: "test zero version bumps correctly",
            fn: {||
                assert_eq (next_patch_version "0.0.0") "0.0.1"
            }
        },
        {
            name: "test result is valid semver",
            fn: {||
                let next = (next_patch_version "0.4.0")
                let matched = ($next | parse --regex '^\d+\.\d+\.\d+$' | length) > 0
                assert_true $matched --msg $"'($next)' should be valid semver"
            }
        },
        {
            name: "test next version differs from current (bump-guard compatible)",
            fn: {||
                let current = "0.4.0"
                let next = (next_patch_version $current)
                assert_true ($next != $current) --msg "next version must differ so _check-version-changed does not fail"
            }
        },
        {
            name: "test next_patch_version matches current Cargo.toml version shape",
            fn: {||
                let current = (open Cargo.toml | get package.version)
                let next = (next_patch_version $current)
                # Same major.minor, patch + 1
                let cur_parts = ($current | split row ".")
                let next_parts = ($next | split row ".")
                assert_eq ($next_parts | get 0) ($cur_parts | get 0) --msg "major should be unchanged"
                assert_eq ($next_parts | get 1) ($cur_parts | get 1) --msg "minor should be unchanged"
            }
        },
        {
            name: "test github deps-update workflow computes next version via scripts/version.nu",
            fn: {||
                let src = (open .github/workflows/deps-update.yml --raw)
                assert_str_contains $src "nu scripts/version.nu" --msg "should read current version via scripts/version.nu"
                assert_str_contains $src "PATCH + 1" --msg "should increment the patch segment"
                assert_str_contains $src "bump_version.nu --yes" --msg "should invoke bump_version.nu non-interactively"
                assert_str_contains $src "git push origin main --follow-tags" --msg "should push the commit and tag to trigger release.yml"
                assert_str_contains $src "gh workflow run release.yml" --msg "should explicitly dispatch release.yml since GITHUB_TOKEN pushes don't trigger workflows"
            }
        },
        {
            name: "test gitea deps-update workflow computes next version via scripts/version.nu",
            fn: {||
                let src = (open .gitea/workflows/deps-update.yml --raw)
                assert_str_contains $src "nu scripts/version.nu" --msg "should read current version via scripts/version.nu"
                assert_str_contains $src "PATCH + 1" --msg "should increment the patch segment"
                assert_str_contains $src "bump_version.nu --yes" --msg "should invoke bump_version.nu non-interactively"
                assert_str_contains $src "git push origin main --follow-tags" --msg "should push the commit and tag to trigger release.yml"
                assert_str_contains $src "actions/workflows/release.yml/dispatches" --msg "should explicitly dispatch release.yml as a fallback trigger"
            }
        },
        {
            name: "test both deps-update workflows trigger release only on a tag push",
            fn: {||
                # release.yml (both platforms) must be scoped to `push: tags: v*`
                # so that pushing the bump commit alone (without --follow-tags)
                # would NOT be sufficient — this guards against regressing the
                # `--follow-tags` flag away in a future edit.
                let gh_release = (open .github/workflows/release.yml --raw)
                let gitea_release = (open .gitea/workflows/release.yml --raw)
                assert_str_contains $gh_release "tags:" --msg "github release.yml should trigger on tag push"
                assert_str_contains $gitea_release "tags:" --msg "gitea release.yml should trigger on tag push"
            }
        },
    ]

    # ── summary ──────────────────────────────────────────────────────
    if $results.failed > 0 {
        print $"($red)  ($results.failed) test\(s\) failed($reset)"
        exit 1
    } else {
        print $"($green)  all ($results.passed) test\(s\) passed($reset)"
    }
}
