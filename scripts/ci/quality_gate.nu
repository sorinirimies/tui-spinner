#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# tui-spinner — CI Quality Gate
# ──────────────────────────────────────────────────────────────────────────────
# Runs the full quality-gate sequence used by both CI and release workflows:
#   1. cargo fmt --check
#   2. cargo clippy (deny warnings)
#   3. cargo test
#   4. nu script tests
#
# Usage:
#   nu scripts/ci/quality_gate.nu              # run all four checks
#   nu scripts/ci/quality_gate.nu --skip-fmt   # skip formatting check
#   nu scripts/ci/quality_gate.nu --skip-test  # skip test suite
#   nu scripts/ci/quality_gate.nu --skip-nu    # skip Nushell script tests
#
# Exit codes:
#   0  — all checks passed
#   1  — one or more checks failed
# ──────────────────────────────────────────────────────────────────────────────

def green [msg: string] { $"(ansi green)($msg)(ansi reset)" }
def red   [msg: string] { $"(ansi red)($msg)(ansi reset)" }
def cyan  [msg: string] { $"(ansi cyan)($msg)(ansi reset)" }

def step [label: string] {
    print $"(cyan '▶') ($label)"
}

def main [
    --skip-fmt   # Skip the cargo fmt check
    --skip-test  # Skip the cargo test suite
    --skip-nu    # Skip the Nushell script tests
] {
    print ""
    print (cyan "══════════════════════════════════════════════════════════")
    print (cyan "  tui-spinner — Quality Gate")
    print (cyan "══════════════════════════════════════════════════════════")
    print ""

    mut failed = false

    # ── 1. Formatting ─────────────────────────────────────────────────────────
    if not $skip_fmt {
        step "cargo fmt --check"
        let result = (do { cargo fmt --check } | complete)
        if $result.exit_code != 0 {
            print (red "  ✗ Formatting check failed.")
            if ($result.stderr | str trim | is-not-empty) {
                print $result.stderr
            }
            if ($result.stdout | str trim | is-not-empty) {
                print $result.stdout
            }
            $failed = true
        } else {
            print (green "  ✔ Formatting OK")
        }
        print ""
    } else {
        print "  ⏭ Skipping cargo fmt"
        print ""
    }

    # ── 2. Clippy ─────────────────────────────────────────────────────────────
    step "cargo clippy --all-features -- -D warnings"
    let clippy = (do {
        cargo clippy --all-features -- -D warnings
    } | complete)
    if $clippy.exit_code != 0 {
        print (red "  ✗ Clippy found warnings or errors.")
        if ($clippy.stderr | str trim | is-not-empty) {
            print $clippy.stderr
        }
        $failed = true
    } else {
        print (green "  ✔ Clippy passed")
    }
    print ""

    # ── 3. Tests ──────────────────────────────────────────────────────────────
    if not $skip_test {
        step "cargo test --all-features"
        let test_result = (do {
            cargo test --all-features
        } | complete)
        if $test_result.exit_code != 0 {
            print (red "  ✗ Tests failed.")
            if ($test_result.stderr | str trim | is-not-empty) {
                print $test_result.stderr
            }
            if ($test_result.stdout | str trim | is-not-empty) {
                print $test_result.stdout
            }
            $failed = true
        } else {
            print (green "  ✔ All tests passed")
        }
        print ""
    } else {
        print "  ⏭ Skipping cargo test"
        print ""
    }

    # ── 4. Nushell script tests ───────────────────────────────────────────────
    if not $skip_nu {
        step "nu scripts/tests/run_all.nu"
        let nu_result = (do {
            nu scripts/tests/run_all.nu
        } | complete)
        if $nu_result.exit_code != 0 {
            print (red "  ✗ Nushell script tests failed.")
            if ($nu_result.stderr | str trim | is-not-empty) {
                print $nu_result.stderr
            }
            if ($nu_result.stdout | str trim | is-not-empty) {
                print $nu_result.stdout
            }
            $failed = true
        } else {
            print (green "  ✔ Nushell script tests passed")
        }
        print ""
    } else {
        print "  ⏭ Skipping Nushell script tests"
        print ""
    }

    # ── Summary ───────────────────────────────────────────────────────────────
    if $failed {
        print (red "══════════════════════════════════════════════════════════")
        print (red "  ✗ Quality gate FAILED")
        print (red "══════════════════════════════════════════════════════════")
        exit 1
    }

    print (green "══════════════════════════════════════════════════════════")
    print (green "  ✔ Quality gate passed")
    print (green "══════════════════════════════════════════════════════════")
}
