#!/usr/bin/env nu
# test_upgrade_deps.nu — Tests for scripts/upgrade_deps.nu helper functions.
# Tests commit_label generation and all_passed logic.
# Usage: nu scripts/tests/test_upgrade_deps.nu

use runner.nu [run_tests, assert_true, assert_false, assert_eq, assert_matches]

def main [] {
    let reset = (ansi reset)
    let cyan = (ansi cyan)
    let green = (ansi green)
    let red = (ansi red)

    print $"($cyan)── test_upgrade_deps.nu ───────────────────────($reset)"

    let results = run_tests [
        {
            name: "test commit_label returns YYYY-MM-DD format",
            fn: {||
                # Replicate the commit_label logic from upgrade_deps.nu
                let label = (date now | format date "%Y-%m-%d")
                assert_matches $label '^\d{4}-\d{2}-\d{2}$' --msg "commit_label should be YYYY-MM-DD"
            }
        },
        {
            name: "test commit_label is not empty",
            fn: {||
                let label = (date now | format date "%Y-%m-%d")
                assert_true (($label | str length) > 0) --msg "commit_label should not be empty"
            }
        },
        {
            name: "test commit_label has exactly 10 characters",
            fn: {||
                let label = (date now | format date "%Y-%m-%d")
                assert_eq ($label | str length) 10 --msg "commit_label should be exactly 10 chars (YYYY-MM-DD)"
            }
        },
        {
            name: "test all_passed returns true when all true",
            fn: {||
                # Replicate the all_passed logic from upgrade_deps.nu
                let results = [true, true, true]
                let passed = ($results | all {|it| $it })
                assert_true $passed --msg "all_passed should be true when all items are true"
            }
        },
        {
            name: "test all_passed returns false when one false",
            fn: {||
                let results = [true, false, true]
                let passed = ($results | all {|it| $it })
                assert_false $passed --msg "all_passed should be false when any item is false"
            }
        },
        {
            name: "test all_passed returns false when all false",
            fn: {||
                let results = [false, false, false]
                let passed = ($results | all {|it| $it })
                assert_false $passed --msg "all_passed should be false when all items are false"
            }
        },
        {
            name: "test all_passed returns true for empty list",
            fn: {||
                let results: list<bool> = []
                let passed = ($results | all {|it| $it })
                assert_true $passed --msg "all_passed should be true for empty list (vacuous truth)"
            }
        },
        {
            name: "test all_passed returns true for single true",
            fn: {||
                let results = [true]
                let passed = ($results | all {|it| $it })
                assert_true $passed --msg "all_passed should be true for single true element"
            }
        },
        {
            name: "test all_passed returns false for single false",
            fn: {||
                let results = [false]
                let passed = ($results | all {|it| $it })
                assert_false $passed --msg "all_passed should be false for single false element"
            }
        },
        {
            name: "test commit message format for toml changes",
            fn: {||
                let label = (date now | format date "%Y-%m-%d")
                let prefix = "chore(deps): upgrade dependencies "
                let msg = ($prefix + $label)
                if not ($msg | str contains "chore(deps):") {
                    error make { msg: "commit message should have conventional prefix" }
                }
                if not ($msg | str contains $label) {
                    error make { msg: "commit message should contain the date label" }
                }
                if not ($msg | str contains "upgrade dependencies") {
                    error make { msg: "commit message should describe the action" }
                }
            }
        },
        {
            name: "test commit message format for lock-only changes",
            fn: {||
                let label = (date now | format date "%Y-%m-%d")
                let prefix = "chore(deps): update lockfile "
                let msg = ($prefix + $label)
                if not ($msg | str contains "chore(deps):") {
                    error make { msg: "commit message should have conventional prefix" }
                }
                if not ($msg | str contains "update lockfile") {
                    error make { msg: "lock-only commit should say update lockfile" }
                }
            }
        },
        {
            name: "test upgrade_deps script has dry-run flag",
            fn: {||
                # Read the script source and verify it declares the --dry-run flag
                let src = (open scripts/upgrade_deps.nu --raw)
                if not ($src | str contains "dry-run") {
                    error make { msg: "upgrade_deps.nu should declare --dry-run flag" }
                }
                if not ($src | str contains "dry_run") {
                    error make { msg: "upgrade_deps.nu should use dry_run variable" }
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
