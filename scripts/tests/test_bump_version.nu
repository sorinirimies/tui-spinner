#!/usr/bin/env nu
# test_bump_version.nu — Tests for scripts/bump_version.nu
# Tests version format validation and Cargo.toml version reading/updating.
# Adapted for a single crate (no workspace).
# Usage: nu scripts/tests/test_bump_version.nu

use runner.nu [run_tests, assert_true, assert_false, assert_eq, assert_str_contains]

def main [] {
    let reset = (ansi reset)
    let cyan = (ansi cyan)
    let green = (ansi green)
    let red = (ansi red)

    print $"($cyan)── test_bump_version.nu ───────────────────────($reset)"

    let results = run_tests [
        {
            name: "test valid semver X.Y.Z is accepted",
            fn: {||
                let versions = ["0.1.0", "1.0.0", "12.34.56", "0.0.1", "999.999.999"]
                for v in $versions {
                    let matched = ($v | parse --regex '^(\d+)\.(\d+)\.(\d+)(-[\w.]+)?$' | length) > 0
                    assert_true $matched --msg $"version '($v)' should be valid semver"
                }
            }
        },
        {
            name: "test valid semver X.Y.Z-suffix is accepted",
            fn: {||
                let versions = ["0.1.0-alpha", "1.0.0-beta.1", "2.3.4-rc.2", "0.1.0-pre.release.3"]
                for v in $versions {
                    let matched = ($v | parse --regex '^(\d+)\.(\d+)\.(\d+)(-[\w.]+)?$' | length) > 0
                    assert_true $matched --msg $"version '($v)' should be valid semver with suffix"
                }
            }
        },
        {
            name: "test invalid version formats are rejected",
            fn: {||
                let bad_versions = ["1.0", "1", "abc", "1.0.0.0", "v1.0.0", "1.0.0-", ".1.0.0", "1..0.0"]
                for v in $bad_versions {
                    let matched = ($v | parse --regex '^(\d+)\.(\d+)\.(\d+)(-[\w.]+)?$' | length) > 0
                    assert_false $matched --msg $"version '($v)' should be rejected as invalid"
                }
            }
        },
        {
            name: "test read current version from Cargo.toml",
            fn: {||
                let version = (open Cargo.toml | get package.version)
                assert_true (($version | str length) > 0) --msg "Cargo.toml should have a non-empty package.version"
                let matched = ($version | parse --regex '^(\d+)\.(\d+)\.(\d+)(-[\w.]+)?$' | length) > 0
                assert_true $matched --msg $"Cargo.toml version '($version)' should be valid semver"
            }
        },
        {
            name: "test version line replacement in Cargo.toml content",
            fn: {||
                # Simulate the replacement logic on a sample Cargo.toml snippet
                let sample = "
[package]
name = \"tui-spinner\"
version = \"0.1.0\"
edition = \"2021\"
"
                let new_version = "0.2.0"
                let updated = ($sample | str replace --regex 'version\s*=\s*"[^"]+"' $'version = "($new_version)"')
                assert_str_contains $updated $'version = "0.2.0"' --msg "version line should be updated to 0.2.0"
                # Ensure other fields are preserved
                assert_str_contains $updated 'name = "tui-spinner"' --msg "package name should be preserved"
                assert_str_contains $updated 'edition = "2021"' --msg "edition should be preserved"
            }
        },
        {
            name: "test version replacement only changes first occurrence",
            fn: {||
                # In a single-crate project, only the [package] version matters.
                # The regex replaces the first match, which is the package version.
                let sample = "
[package]
name = \"tui-spinner\"
version = \"0.1.0\"

[dependencies]
ratatui = { version = \"0.30\", default-features = false }
"
                let new_version = "0.2.0"
                let updated = ($sample | str replace --regex 'version\s*=\s*"[^"]+"' $'version = "($new_version)"')
                # The first version line should be updated
                assert_str_contains $updated 'version = "0.2.0"' --msg "package version should be updated"
                # The dependency version should remain untouched
                assert_str_contains $updated 'version = "0.30"' --msg "dependency version should be preserved"
            }
        },
        {
            name: "test version with pre-release suffix replacement",
            fn: {||
                let sample = "[package]\nversion = \"0.1.0\"\n"
                let new_version = "0.2.0-alpha.1"
                let updated = ($sample | str replace --regex 'version\s*=\s*"[^"]+"' $'version = "($new_version)"')
                assert_str_contains $updated $'version = "0.2.0-alpha.1"' --msg "pre-release version should be set"
            }
        },
        {
            name: "test bump_version rejects invalid version with error",
            fn: {||
                # Run bump_version.nu with an invalid version — it should fail
                let result = (do { nu scripts/bump_version.nu "not-a-version" --yes } | complete)
                assert_true ($result.exit_code != 0) --msg "bump_version should exit non-zero for invalid version"
            }
        },
        {
            name: "test Cargo.toml has package section (not workspace)",
            fn: {||
                # Ensure this is a single crate, not a workspace
                let cargo = (open Cargo.toml)
                let has_package = ($cargo | get package? | is-not-empty)
                assert_true $has_package --msg "Cargo.toml should have a [package] section"

                # A workspace root Cargo.toml would have [workspace] but no [package]
                let has_workspace = ($cargo | get workspace? | is-not-empty)
                assert_false $has_workspace --msg "Cargo.toml should not have a [workspace] section (single crate)"
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
