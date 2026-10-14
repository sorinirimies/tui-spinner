#!/usr/bin/env nu
# Build examples first so VHS recordings never show compilation, then record all tapes.
def main [] {
    if (cargo build --release --examples --features theme | complete).exit_code != 0 {
        error make { msg: "cargo build --release --examples --features theme failed" }
    }
    mkdir examples/vhs/generated
    for t in (glob examples/vhs/*.tape | sort) {
        print $"recording ($t)"
        vhs $t
    }
}
