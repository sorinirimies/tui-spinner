#!/usr/bin/env nu
# test_version.nu — Tests for scripts/version.nu
# Usage: nu scripts/tests/test_version.nu

use runner.nu [run_tests, assert_true, assert_matches, assert_eq]

def main [] {
    let reset = (ansi reset)
    let cyan = (ansi cyan)
    let green = (ansi green)
    let red = (ansi red)

    print $"($cyan)── test_version.nu ────────────────────────────($reset)"

    let results = run_tests [
        {
            name: "test version script returns output",
            fn: {||
                let result = (do { nu scripts/version.nu } | complete)
                assert_eq $result.exit_code 0 --msg "version.nu should exit with code 0"
                let output = ($result.stdout | str trim)
                assert_true (($output | str length) > 0) --msg "version.nu should produce non-empty output"
            }
        },
        {
            name: "test version is valid semver format",
            fn: {||
                let result = (do { nu scripts/version.nu } | complete)
                let version = ($result.stdout | str trim)
                assert_matches $version '^\d+\.\d+\.\d+' --msg "version should match X.Y.Z semver format"
            }
        },
        {
            name: "test version matches Cargo.toml",
            fn: {||
                let script_version = (do { nu scripts/version.nu } | complete).stdout | str trim
                let toml_version = (open Cargo.toml | get package.version)
                assert_eq $script_version $toml_version --msg "version.nu output should match Cargo.toml package.version"
            }
        },
        {
            name: "test version has exactly three numeric segments",
            fn: {||
                let version = (do { nu scripts/version.nu } | complete).stdout | str trim
                # Strip any pre-release suffix (e.g. -alpha.1)
                let base = ($version | split row "-" | first)
                let parts = ($base | split row ".")
                assert_eq ($parts | length) 3 --msg "version should have exactly 3 dot-separated segments"
                # Each segment should be a non-negative integer
                for part in $parts {
                    let is_numeric = ($part | parse --regex '^\d+$' | length) > 0
                    assert_true $is_numeric --msg $"version segment '($part)' should be numeric"
                }
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
