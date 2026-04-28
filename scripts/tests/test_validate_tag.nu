#!/usr/bin/env nu
# ── tui-spinner · test_validate_tag.nu ──────────────────────────────────────
# Tests for scripts/ci/validate_tag.nu — tag validation and version extraction.

use runner.nu [run_tests, assert_eq, assert_true, assert_false]

def main [] {
    let reset = (ansi reset)
    let cyan = (ansi cyan)
    let green = (ansi green)
    let red = (ansi red)

    print $"($cyan)── test_validate_tag.nu ───────────────────────($reset)"

    let results = run_tests [
        {
            name: "test validate_tag: simple release tag",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "v1.0.0" } | complete)
                assert_eq $result.exit_code 0 --msg "v1.0.0 should be valid"
                let output = ($result.stdout | str trim)
                assert_true ($output | str contains "tag=v1.0.0") --msg "output should contain tag=v1.0.0"
                assert_true ($output | str contains "version=1.0.0") --msg "output should contain version=1.0.0"
            }
        },
        {
            name: "test validate_tag: patch release",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "v0.1.6" } | complete)
                assert_eq $result.exit_code 0 --msg "v0.1.6 should be valid"
            }
        },
        {
            name: "test validate_tag: zero version",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "v0.0.0" } | complete)
                assert_eq $result.exit_code 0 --msg "v0.0.0 should be valid"
            }
        },
        {
            name: "test validate_tag: large version numbers",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "v12.345.6789" } | complete)
                assert_eq $result.exit_code 0 --msg "v12.345.6789 should be valid"
            }
        },
        {
            name: "test validate_tag: rejects missing v prefix",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "1.0.0" } | complete)
                assert_true ($result.exit_code != 0) --msg "1.0.0 should be rejected (no v prefix)"
            }
        },
        {
            name: "test validate_tag: rejects two-segment version",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "v1.0" } | complete)
                assert_true ($result.exit_code != 0) --msg "v1.0 should be rejected (two segments)"
            }
        },
        {
            name: "test validate_tag: rejects four-segment version",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "v1.0.0.0" } | complete)
                assert_true ($result.exit_code != 0) --msg "v1.0.0.0 should be rejected (four segments)"
            }
        },
        {
            name: "test validate_tag: rejects empty string",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "" } | complete)
                assert_true ($result.exit_code != 0) --msg "empty string should be rejected"
            }
        },
        {
            name: "test validate_tag: rejects bare text",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "release" } | complete)
                assert_true ($result.exit_code != 0) --msg "'release' should be rejected"
            }
        },
        {
            name: "test validate_tag: rejects pre-release suffix",
            fn: {||
                # The CI tag validator only accepts strict vX.Y.Z — no pre-release.
                let result = (do { nu scripts/ci/validate_tag.nu "v1.0.0-rc.1" } | complete)
                assert_true ($result.exit_code != 0) --msg "v1.0.0-rc.1 should be rejected"
            }
        },
        {
            name: "test validate_tag: rejects alpha suffix",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "v1.0.0-alpha" } | complete)
                assert_true ($result.exit_code != 0) --msg "v1.0.0-alpha should be rejected"
            }
        },
        {
            name: "test validate_tag: output format is key=value",
            fn: {||
                let result = (do { nu scripts/ci/validate_tag.nu "v2.3.4" } | complete)
                assert_eq $result.exit_code 0 --msg "v2.3.4 should be valid"
                let lines = ($result.stdout | str trim | lines)
                assert_eq ($lines | length) 2 --msg "should output exactly 2 lines"
                assert_true (($lines | get 0) | str starts-with "tag=") --msg "first line should be tag=..."
                assert_true (($lines | get 1) | str starts-with "version=") --msg "second line should be version=..."
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
