#!/usr/bin/env nu
# Pre-publish readiness check for tui-spinner.
# Verifies formatting, lints, tests, docs, required files, and dry-run publish.
# Usage: nu scripts/check_publish.nu

def main [] {
    let reset = (ansi reset)
    let red = (ansi red)
    let green = (ansi green)
    let yellow = (ansi yellow)
    let cyan = (ansi cyan)

    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print $"($cyan)  tui-spinner publish readiness check($reset)"
    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print ""

    mut passed = 0
    mut failed = 0
    mut warnings = 0
    mut results: list<string> = []

    # ── 1. cargo fmt --check ─────────────────────────────────────────
    print $"($cyan)▸ checking formatting ...($reset)"
    let fmt_result = do { cargo fmt --check } | complete
    if $fmt_result.exit_code == 0 {
        print $"($green)  ✓ formatting ok($reset)"
        $passed = $passed + 1
        $results = ($results | append $"($green)✓($reset) formatting")
    } else {
        print $"($red)  ✗ formatting issues found($reset)"
        $failed = $failed + 1
        $results = ($results | append $"($red)✗($reset) formatting")
    }

    # ── 2. cargo clippy ──────────────────────────────────────────────
    print $"($cyan)▸ running clippy ...($reset)"
    let clippy_result = do { cargo clippy -- -D warnings } | complete
    if $clippy_result.exit_code == 0 {
        print $"($green)  ✓ clippy clean($reset)"
        $passed = $passed + 1
        $results = ($results | append $"($green)✓($reset) clippy")
    } else {
        print $"($red)  ✗ clippy found warnings/errors($reset)"
        $failed = $failed + 1
        $results = ($results | append $"($red)✗($reset) clippy")
    }

    # ── 3. cargo test ────────────────────────────────────────────────
    print $"($cyan)▸ running tests ...($reset)"
    let test_result = do { cargo test } | complete
    if $test_result.exit_code == 0 {
        print $"($green)  ✓ tests passed($reset)"
        $passed = $passed + 1
        $results = ($results | append $"($green)✓($reset) tests")
    } else {
        print $"($red)  ✗ tests failed($reset)"
        $failed = $failed + 1
        $results = ($results | append $"($red)✗($reset) tests")
    }

    # ── 4. cargo doc ─────────────────────────────────────────────────
    print $"($cyan)▸ building documentation ...($reset)"
    let doc_result = do { cargo doc --no-deps } | complete
    if $doc_result.exit_code == 0 {
        print $"($green)  ✓ documentation builds($reset)"
        $passed = $passed + 1
        $results = ($results | append $"($green)✓($reset) documentation")
    } else {
        print $"($red)  ✗ documentation build failed($reset)"
        $failed = $failed + 1
        $results = ($results | append $"($red)✗($reset) documentation")
    }

    # ── 5. required files ────────────────────────────────────────────
    print $"($cyan)▸ checking required files ...($reset)"
    let required_files = ["README.md", "LICENSE", "Cargo.toml", "CHANGELOG.md", "cliff.toml"]
    mut all_files_present = true
    for file in $required_files {
        if ($file | path exists) {
            print $"($green)  ✓ ($file)($reset)"
        } else {
            print $"($red)  ✗ ($file) missing($reset)"
            $all_files_present = false
        }
    }
    if $all_files_present {
        $passed = $passed + 1
        $results = ($results | append $"($green)✓($reset) required files")
    } else {
        $failed = $failed + 1
        $results = ($results | append $"($red)✗($reset) required files")
    }

    # ── 6. Cargo.lock present ────────────────────────────────────────
    print $"($cyan)▸ checking Cargo.lock ...($reset)"
    if ("Cargo.lock" | path exists) {
        print $"($green)  ✓ Cargo.lock present($reset)"
        $passed = $passed + 1
        $results = ($results | append $"($green)✓($reset) Cargo.lock")
    } else {
        print $"($yellow)  ⚠ Cargo.lock not found (optional for libraries)($reset)"
        $warnings = $warnings + 1
        $results = ($results | append $"($yellow)⚠($reset) Cargo.lock")
    }

    # ── 7. cargo publish --dry-run ───────────────────────────────────
    print $"($cyan)▸ running publish dry-run ...($reset)"
    let publish_result = do { cargo publish --dry-run --allow-dirty } | complete
    if $publish_result.exit_code == 0 {
        print $"($green)  ✓ publish dry-run passed($reset)"
        $passed = $passed + 1
        $results = ($results | append $"($green)✓($reset) publish dry-run")
    } else {
        print $"($red)  ✗ publish dry-run failed($reset)"
        print $"($red)    ($publish_result.stderr | str trim)($reset)"
        $failed = $failed + 1
        $results = ($results | append $"($red)✗($reset) publish dry-run")
    }

    # ── summary ──────────────────────────────────────────────────────
    print ""
    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print $"($cyan)  summary($reset)"
    print $"($cyan)═══════════════════════════════════════════════($reset)"
    for r in $results {
        print $"  ($r)"
    }
    print ""
    print $"  ($green)passed:($reset)   ($passed)"
    print $"  ($red)failed:($reset)   ($failed)"
    print $"  ($yellow)warnings:($reset) ($warnings)"
    print ""

    if $failed == 0 {
        print $"($green)🚀 ready to publish!($reset)"
        print $"   cargo publish"
        print ""
    } else {
        print $"($red)⛔ not ready — fix the failures above before publishing.($reset)"
        print ""
        exit 1
    }
}
