#!/usr/bin/env nu
# Nightly dependency upgrade script for tui-spinner.
# Runs cargo upgrade + cargo update, then applies a quality gate.
# Usage: nu scripts/upgrade_deps.nu [--bot-name <name>] [--bot-email <email>] [--remote <remote>] [--dry-run]

# ── helper: generate a commit label from the current date ────────────
def commit_label [] {
    date now | format date "%Y-%m-%d"
}

# ── helper: check if all quality-gate steps passed ───────────────────
def all_passed [results: list<bool>] {
    ($results | all {|it| $it })
}

# ── helper: run a command and return true if exit code is 0 ──────────
def gate_step [name: string, --cmd: string] {
    let reset = (ansi reset)
    let green = (ansi green)
    let red = (ansi red)
    let cyan = (ansi cyan)

    print $"($cyan)  ▸ ($name) ...($reset)"
    let result = (do { nu -c $cmd } | complete)
    if $result.exit_code == 0 {
        print $"($green)    ✓ ($name) passed($reset)"
        true
    } else {
        print $"($red)    ✗ ($name) failed($reset)"
        false
    }
}

def main [
    --bot-name: string = "dep-bot"       # Git author name for commits
    --bot-email: string = "dep-bot@noreply" # Git author email for commits
    --remote: string = "origin"          # Remote to push to
    --dry-run                            # Show what would happen without committing/pushing
] {
    let reset = (ansi reset)
    let red = (ansi red)
    let green = (ansi green)
    let yellow = (ansi yellow)
    let cyan = (ansi cyan)

    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print $"($cyan)  tui-spinner dependency upgrade($reset)"
    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print ""

    let label = (commit_label)
    print $"($cyan)date label:($reset) ($label)"
    print $"($cyan)bot:       ($reset) ($bot_name) <($bot_email)>"
    print $"($cyan)remote:    ($reset) ($remote)"
    if $dry_run {
        print $"($yellow)dry-run mode — no commits or pushes will be made($reset)"
    }
    print ""

    # ── snapshot Cargo.toml before changes ───────────────────────────
    let toml_before = (open Cargo.toml --raw)

    # ── Phase 1: cargo upgrade (semver-incompatible allowed) ─────────
    print $"($cyan)▸ Phase 1: cargo upgrade --incompatible allow ...($reset)"
    let has_cargo_edit = (which cargo-upgrade | length) > 0
    if $has_cargo_edit {
        let upgrade_result = (do { cargo upgrade --incompatible allow } | complete)
        if $upgrade_result.exit_code == 0 {
            print $"($green)  ✓ cargo upgrade completed($reset)"
        } else {
            print $"($yellow)  ⚠ cargo upgrade returned non-zero, continuing ...($reset)"
        }
    } else {
        print $"($yellow)  ⚠ cargo-edit not installed, skipping cargo upgrade($reset)"
    }

    # ── Phase 2: cargo update ────────────────────────────────────────
    print $"($cyan)▸ Phase 2: cargo update ...($reset)"
    let update_result = (do { cargo update } | complete)
    if $update_result.exit_code == 0 {
        print $"($green)  ✓ cargo update completed($reset)"
    } else {
        print $"($red)  ✗ cargo update failed($reset)"
        exit 1
    }

    # ── detect what changed ──────────────────────────────────────────
    let toml_after = (open Cargo.toml --raw)
    let toml_changed = ($toml_before != $toml_after)

    let lock_diff = (do { git diff --name-only Cargo.lock } | complete)
    let lock_changed = ($lock_diff.stdout | str trim | str length) > 0

    if (not $toml_changed) and (not $lock_changed) {
        print $"($green)no dependency changes detected — nothing to do.($reset)"
        return
    }

    print $"($cyan)changes detected:($reset)"
    if $toml_changed { print $"  • Cargo.toml modified" }
    if $lock_changed { print $"  • Cargo.lock modified" }
    print ""

    # ── quality gate ─────────────────────────────────────────────────
    print $"($cyan)▸ Quality gate ...($reset)"
    let fmt_ok = (gate_step "cargo fmt" --cmd "cargo fmt --check")
    let clippy_ok = (gate_step "cargo clippy" --cmd "cargo clippy -- -D warnings")
    let test_ok = (gate_step "cargo test" --cmd "cargo test")

    let gate_ok = (all_passed [$fmt_ok, $clippy_ok, $test_ok])
    print ""

    if $dry_run {
        if $gate_ok {
            print $"($green)dry-run: quality gate passed — would commit changes($reset)"
        } else {
            print $"($yellow)dry-run: quality gate failed — would revert Cargo.toml if changed($reset)"
        }
        # restore original Cargo.toml in dry-run regardless
        $toml_before | save --force Cargo.toml
        do { cargo update --workspace } | complete
        print $"($cyan)dry-run: restored original state($reset)"
        return
    }

    # ── commit strategy ──────────────────────────────────────────────
    let git_env = {
        GIT_AUTHOR_NAME: $bot_name,
        GIT_AUTHOR_EMAIL: $bot_email,
        GIT_COMMITTER_NAME: $bot_name,
        GIT_COMMITTER_EMAIL: $bot_email,
    }

    if $gate_ok {
        # gate passed
        if $toml_changed {
            # commit both Cargo.toml + Cargo.lock
            print $"($cyan)▸ committing Cargo.toml + Cargo.lock ...($reset)"
            git add Cargo.toml Cargo.lock
            with-env $git_env {
                git commit -m $"chore\(deps\): upgrade dependencies ($label)"
            }
            print $"($green)  ✓ committed Cargo.toml + Cargo.lock($reset)"
        } else {
            # only lock changed
            print $"($cyan)▸ committing Cargo.lock ...($reset)"
            git add Cargo.lock
            with-env $git_env {
                git commit -m $"chore\(deps\): update lockfile ($label)"
            }
            print $"($green)  ✓ committed Cargo.lock($reset)"
        }

        print ""
        print $"($green)═══════════════════════════════════════════════($reset)"
        print $"($green)  dependency upgrade complete($reset)"
        print $"($green)═══════════════════════════════════════════════($reset)"
        print ""
        print $"($cyan)next:($reset) git push ($remote) main"
    } else {
        # gate failed
        if $toml_changed {
            # revert Cargo.toml, re-sync lockfile, commit lock only
            print $"($yellow)▸ quality gate failed — reverting Cargo.toml ...($reset)"
            $toml_before | save --force Cargo.toml
            print $"($green)  ✓ Cargo.toml reverted($reset)"

            print $"($cyan)▸ re-syncing Cargo.lock with reverted Cargo.toml ...($reset)"
            do { cargo update --workspace } | complete
            print $"($green)  ✓ Cargo.lock re-synced($reset)"

            let lock_diff2 = (do { git diff --name-only Cargo.lock } | complete)
            let lock_still_changed = ($lock_diff2.stdout | str trim | str length) > 0

            if $lock_still_changed {
                print $"($cyan)▸ committing Cargo.lock only (safe subset) ...($reset)"
                git add Cargo.lock
                with-env $git_env {
                    git commit -m $"chore\(deps\): update lockfile ($label) \(partial\)"
                }
                print $"($green)  ✓ committed Cargo.lock($reset)"
            } else {
                print $"($yellow)  no lockfile changes remain after revert($reset)"
            }
        } else {
            # only lock changed but gate failed — commit lock anyway
            print $"($yellow)▸ quality gate failed but only Cargo.lock changed — committing ...($reset)"
            git add Cargo.lock
            with-env $git_env {
                git commit -m $"chore\(deps\): update lockfile ($label)"
            }
            print $"($green)  ✓ committed Cargo.lock($reset)"
        }

        print ""
        print $"($red)═══════════════════════════════════════════════($reset)"
        print $"($red)  quality gate failed — review needed($reset)"
        print $"($red)═══════════════════════════════════════════════($reset)"
        print ""
        exit 1
    }
}
