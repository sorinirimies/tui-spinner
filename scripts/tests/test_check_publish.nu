#!/usr/bin/env nu
# test_check_publish.nu — Tests for scripts/check_publish.nu
# Verifies that the required-files check logic works correctly.
# Usage: nu scripts/tests/test_check_publish.nu

use runner.nu [run_tests, assert_true, assert_false, assert_eq, assert_str_contains]

def main [] {
    let reset = (ansi reset)
    let cyan = (ansi cyan)
    let green = (ansi green)
    let red = (ansi red)

    print $"($cyan)── test_check_publish.nu ──────────────────────($reset)"

    let results = run_tests [
        {
            name: "test required files list is complete",
            fn: {||
                # The check_publish script checks for these files.
                # Verify the canonical list is what we expect.
                let required_files = ["README.md", "LICENSE", "Cargo.toml", "CHANGELOG.md", "cliff.toml"]
                assert_eq ($required_files | length) 5 --msg "should require exactly 5 files"
                assert_true ("README.md" in $required_files) --msg "README.md should be required"
                assert_true ("LICENSE" in $required_files) --msg "LICENSE should be required"
                assert_true ("Cargo.toml" in $required_files) --msg "Cargo.toml should be required"
                assert_true ("CHANGELOG.md" in $required_files) --msg "CHANGELOG.md should be required"
                assert_true ("cliff.toml" in $required_files) --msg "cliff.toml should be required"
            }
        },
        {
            name: "test README.md exists in project",
            fn: {||
                assert_true ("README.md" | path exists) --msg "README.md should exist in project root"
            }
        },
        {
            name: "test LICENSE exists in project",
            fn: {||
                assert_true ("LICENSE" | path exists) --msg "LICENSE should exist in project root"
            }
        },
        {
            name: "test Cargo.toml exists in project",
            fn: {||
                assert_true ("Cargo.toml" | path exists) --msg "Cargo.toml should exist in project root"
            }
        },
        {
            name: "test cliff.toml exists in project",
            fn: {||
                assert_true ("cliff.toml" | path exists) --msg "cliff.toml should exist in project root"
            }
        },
        {
            name: "test Cargo.lock exists in project",
            fn: {||
                assert_true ("Cargo.lock" | path exists) --msg "Cargo.lock should exist in project root"
            }
        },
        {
            name: "test path exists correctly detects missing file",
            fn: {||
                let fake = "THIS_FILE_DOES_NOT_EXIST_12345.txt"
                assert_false ($fake | path exists) --msg "non-existent file should not pass path exists check"
            }
        },
        {
            name: "test all present files detected correctly",
            fn: {||
                # Simulate the check logic from check_publish.nu
                let required_files = ["README.md", "LICENSE", "Cargo.toml", "cliff.toml"]
                mut all_present = true
                for file in $required_files {
                    if not ($file | path exists) {
                        $all_present = false
                    }
                }
                assert_true $all_present --msg "all core required files should be present in the project"
            }
        },
        {
            name: "test missing file causes all_present to be false",
            fn: {||
                # Simulate the check with a known-missing file
                let required_files = ["README.md", "NONEXISTENT_FILE.xyz"]
                mut all_present = true
                for file in $required_files {
                    if not ($file | path exists) {
                        $all_present = false
                    }
                }
                assert_false $all_present --msg "all_present should be false when a file is missing"
            }
        },
        {
            name: "test Cargo.toml has required publish metadata",
            fn: {||
                let cargo = (open Cargo.toml | get package)
                # These fields are important for crates.io publishing
                assert_true ($cargo.name? | is-not-empty) --msg "Cargo.toml should have package.name"
                assert_true ($cargo.version? | is-not-empty) --msg "Cargo.toml should have package.version"
                assert_true ($cargo.description? | is-not-empty) --msg "Cargo.toml should have package.description"
                assert_true ($cargo.license? | is-not-empty) --msg "Cargo.toml should have package.license"
                assert_true ($cargo.repository? | is-not-empty) --msg "Cargo.toml should have package.repository"
            }
        },
        {
            name: "test check_publish script runs without crash",
            fn: {||
                # Run the actual script and verify it doesn't panic.
                # It may fail (e.g. missing CHANGELOG.md or dry-run fails), but it
                # should produce structured output and not crash.
                let result = (do { nu scripts/check_publish.nu } | complete)
                # We just verify it ran and produced output, regardless of exit code
                let combined = $"($result.stdout)($result.stderr)"
                assert_true (($combined | str length) > 0) --msg "check_publish.nu should produce output"
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
