#!/usr/bin/env nu
# runner.nu — Shared test runner for tui-spinner nushell scripts.
# Discovers and runs all `test ` prefixed commands exported from a test module.
#
# Usage (from another test file):
#   use runner.nu [run_tests, assert_eq, assert_true, assert_false, assert_str_contains, assert_matches]
#   run_tests [ { name: "test foo", fn: {|| assert_true true } } ]

# Run a list of named test closures and return a summary record.
# Each entry should be: { name: string, fn: closure }
#
# Returns a record: { passed: int, failed: int, errors: list<string> }
export def run_tests [
    test_fns: list<any>
] {
    let reset = (ansi reset)
    let red = (ansi red)
    let green = (ansi green)
    let cyan = (ansi cyan)

    let total = ($test_fns | length)
    print $"($cyan)  running ($total) test\(s\) ...($reset)"
    print ""

    # Run each test and collect results as a list of records.
    # This avoids mutable variable captures inside closures.
    let outcomes = ($test_fns | each {|t|
        let name = $t.name
        let fn = $t.fn

        let outcome = (try {
            do $fn
            { status: "pass", name: $name, error: "" }
        } catch {|err|
            let msg = ($err.msg? | default (($err | to text) | str trim))
            { status: "fail", name: $name, error: $msg }
        })

        if $outcome.status == "pass" {
            print $"  ($green)✓($reset) ($name)"
        } else {
            print $"  ($red)✗($reset) ($name)"
            print $"    ($red)($outcome.error)($reset)"
        }

        $outcome
    })

    print ""

    let passed = ($outcomes | where status == "pass" | length)
    let failed = ($outcomes | where status == "fail" | length)
    let errors = ($outcomes | where status == "fail" | each {|o| $"($o.name): ($o.error)" })

    {
        passed: $passed,
        failed: $failed,
        errors: $errors,
    }
}

# Assert that two values are equal. Raises an error if they differ.
export def assert_eq [actual: any, expected: any, --msg: string = ""] {
    if $actual != $expected {
        let label = if ($msg | is-empty) { "" } else { $"($msg): " }
        error make {
            msg: $"($label)expected ($expected | to text), got ($actual | to text)"
        }
    }
}

# Assert that a value is true.
export def assert_true [value: bool, --msg: string = ""] {
    if not $value {
        let label = if ($msg | is-empty) { "assertion failed" } else { $msg }
        error make { msg: $label }
    }
}

# Assert that a value is false.
export def assert_false [value: bool, --msg: string = ""] {
    if $value {
        let label = if ($msg | is-empty) { "expected false, got true" } else { $msg }
        error make { msg: $label }
    }
}

# Assert that a string contains a substring.
export def assert_str_contains [haystack: string, needle: string, --msg: string = ""] {
    if not ($haystack | str contains $needle) {
        let label = if ($msg | is-empty) {
            $"expected string to contain '($needle)'"
        } else {
            $msg
        }
        error make { msg: $label }
    }
}

# Assert that a string matches a regex pattern.
export def assert_matches [value: string, pattern: string, --msg: string = ""] {
    let matched = ($value | parse --regex $pattern | length) > 0
    if not $matched {
        let label = if ($msg | is-empty) {
            $"expected '($value)' to match pattern '($pattern)'"
        } else {
            $msg
        }
        error make { msg: $label }
    }
}
