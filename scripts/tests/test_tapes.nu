#!/usr/bin/env nu
# Every tape must only run `cargo run` between Hide and Show.
use std assert
def main [] {
    for t in (glob examples/vhs/*.tape) {
        mut hidden = false
        for line in (open $t | lines) {
            if $line starts-with "Hide" { $hidden = true }
            if $line starts-with "Show" { $hidden = false }
            if ($line | str contains "cargo run") { assert $hidden $"($t): cargo run visible" }
        }
    }
    print "✓ tapes hide compilation"
}
