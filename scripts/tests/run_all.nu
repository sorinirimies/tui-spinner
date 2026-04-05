#!/usr/bin/env nu
# run_all.nu — Discover and run all test_*.nu files for tui-spinner.
# Usage: nu scripts/tests/run_all.nu
#
# Discovers every test_*.nu file in the same directory, executes each one,
# and prints a summary with pass/fail counts.

def main [] {
    let reset = (ansi reset)
    let red = (ansi red)
    let green = (ansi green)
    let yellow = (ansi yellow)
    let cyan = (ansi cyan)

    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print $"($cyan)  tui-spinner — nushell test suite($reset)"
    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print ""

    # ── discover test files ──────────────────────────────────────────
    let test_dir = ($nu.current-exe | path dirname | path join "" | ignore; "scripts/tests")
    # Resolve relative to the repo root (assume we run from repo root)
    let pattern = "scripts/tests/test_*.nu"
    let test_files = (glob $pattern | sort)

    if ($test_files | length) == 0 {
        print $"($yellow)  ⚠ no test files found matching ($pattern)($reset)"
        exit 0
    }

    let total_files = ($test_files | length)
    print $"($cyan)  found ($total_files) test file\(s\):($reset)"
    for f in $test_files {
        print $"    • ($f | path basename)"
    }
    print ""

    # ── run each test file ───────────────────────────────────────────
    mut passed_files = 0
    mut failed_files = 0
    mut file_results: list<string> = []

    for test_file in $test_files {
        let basename = ($test_file | path basename)
        print $"($cyan)──────────────────────────────────────────────($reset)"
        print $"($cyan)  ▸ ($basename)($reset)"
        print $"($cyan)──────────────────────────────────────────────($reset)"

        let result = (do { nu $test_file } | complete)

        # Print the test file's output
        if ($result.stdout | str trim | str length) > 0 {
            print $result.stdout
        }
        if ($result.stderr | str trim | str length) > 0 {
            print $result.stderr
        }

        if $result.exit_code == 0 {
            print $"($green)  ✓ ($basename) passed($reset)"
            $passed_files = $passed_files + 1
            $file_results = ($file_results | append $"($green)✓($reset) ($basename)")
        } else {
            print $"($red)  ✗ ($basename) failed \(exit code ($result.exit_code)\)($reset)"
            $failed_files = $failed_files + 1
            $file_results = ($file_results | append $"($red)✗($reset) ($basename)")
        }
        print ""
    }

    # ── summary ──────────────────────────────────────────────────────
    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print $"($cyan)  tui-spinner test summary($reset)"
    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print ""
    for r in $file_results {
        print $"  ($r)"
    }
    print ""
    print $"  ($green)passed:($reset) ($passed_files)"
    print $"  ($red)failed:($reset) ($failed_files)"
    print $"  total:  ($total_files)"
    print ""

    if $failed_files == 0 {
        print $"($green)🎉 all tests passed!($reset)"
        print ""
    } else {
        print $"($red)⛔ ($failed_files) test file\(s\) failed.($reset)"
        print ""
        exit 1
    }
}
