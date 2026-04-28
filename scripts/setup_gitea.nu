#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# tui-spinner — Gitea remote setup
# ──────────────────────────────────────────────────────────────────────────────
# Adds or updates a named git remote for a Gitea instance, tests connectivity,
# and optionally pushes all branches and tags.
#
# Usage:
#   nu scripts/setup_gitea.nu <url>                                 # adds/updates 'gitea'
#   nu scripts/setup_gitea.nu <url> --remote gitea_starscream       # custom name
#   nu scripts/setup_gitea.nu <url> --push                          # push after setup
#
# Examples:
#   nu scripts/setup_gitea.nu ssh://git@192.168.1.204:30009/sorin/tui-spinner.git
#   nu scripts/setup_gitea.nu gitea@192.168.1.44:sorin/tui-spinner.git --remote gitea_starscream
# ──────────────────────────────────────────────────────────────────────────────

def main [
    gitea_url: string          # Full Gitea repository URL
    --remote: string = "gitea" # Remote name (default: gitea)
    --push                     # Push all branches and tags after setup
] {
    print ""
    print $"(ansi cyan)══════════════════════════════════════════════════════════(ansi reset)"
    print $"(ansi cyan)  tui-spinner — Gitea remote setup(ansi reset)"
    print $"(ansi cyan)══════════════════════════════════════════════════════════(ansi reset)"
    print ""

    # ── check prerequisites ──────────────────────────────────────
    print $"(ansi cyan)▸ checking prerequisites …(ansi reset)"

    let has_git = (which git | length) > 0
    if not $has_git {
        print $"(ansi red)  ✗ git is not installed(ansi reset)"
        exit 1
    }
    print $"(ansi green)  ✓ git is available(ansi reset)"

    let in_repo = (do { git rev-parse --is-inside-work-tree } | complete)
    if $in_repo.exit_code != 0 {
        print $"(ansi red)  ✗ not inside a git repository(ansi reset)"
        exit 1
    }
    print $"(ansi green)  ✓ inside a git repository(ansi reset)"
    print ""

    # ── add or update the remote ─────────────────────────────────
    let existing_remotes = (do { git remote } | complete)
    let remote_list = ($existing_remotes.stdout | str trim | lines)
    let remote_exists = ($remote_list | any {|r| $r == $remote})

    if $remote_exists {
        print $"(ansi yellow)▸ remote '($remote)' already exists — updating URL …(ansi reset)"
        let set_result = (do { git remote set-url $remote $gitea_url } | complete)
        if $set_result.exit_code != 0 {
            print $"(ansi red)  ✗ failed to update remote URL(ansi reset)"
            print $"(ansi red)    ($set_result.stderr | str trim)(ansi reset)"
            exit 1
        }
        print $"(ansi green)  ✓ remote '($remote)' URL updated to ($gitea_url)(ansi reset)"
    } else {
        print $"(ansi cyan)▸ adding remote '($remote)' …(ansi reset)"
        let add_result = (do { git remote add $remote $gitea_url } | complete)
        if $add_result.exit_code != 0 {
            print $"(ansi red)  ✗ failed to add remote(ansi reset)"
            print $"(ansi red)    ($add_result.stderr | str trim)(ansi reset)"
            exit 1
        }
        print $"(ansi green)  ✓ remote '($remote)' added: ($gitea_url)(ansi reset)"
    }
    print ""

    # ── test connection ──────────────────────────────────────────
    print $"(ansi cyan)▸ testing connection to '($remote)' …(ansi reset)"
    let ls_result = (do { git ls-remote --exit-code $remote } | complete)
    if $ls_result.exit_code == 0 {
        print $"(ansi green)  ✓ connection successful(ansi reset)"
    } else if $ls_result.exit_code == 2 {
        print $"(ansi yellow)  ⚠ remote is reachable but has no refs (empty repository)(ansi reset)"
    } else {
        print $"(ansi yellow)  ⚠ could not reach remote — check URL and credentials(ansi reset)"
        print $"(ansi yellow)    ($ls_result.stderr | str trim)(ansi reset)"
        print $"(ansi yellow)    setup will continue — you can push once access is configured(ansi reset)"
    }
    print ""

    # ── optionally push all branches and tags ────────────────────
    if $push {
        print $"(ansi cyan)▸ pushing all branches to '($remote)' …(ansi reset)"
        let push_result = (do { git push $remote --all } | complete)
        if $push_result.exit_code == 0 {
            print $"(ansi green)  ✓ all branches pushed(ansi reset)"
        } else {
            print $"(ansi red)  ✗ branch push failed(ansi reset)"
            print $"(ansi red)    ($push_result.stderr | str trim)(ansi reset)"
        }

        print $"(ansi cyan)▸ pushing all tags to '($remote)' …(ansi reset)"
        let tags_result = (do { git push $remote --tags } | complete)
        if $tags_result.exit_code == 0 {
            print $"(ansi green)  ✓ all tags pushed(ansi reset)"
        } else {
            print $"(ansi red)  ✗ tag push failed(ansi reset)"
            print $"(ansi red)    ($tags_result.stderr | str trim)(ansi reset)"
        }
        print ""
    }

    # ── summary ──────────────────────────────────────────────────
    let current_remotes = (do { git remote -v } | complete)

    print $"(ansi green)══════════════════════════════════════════════════════════(ansi reset)"
    print $"(ansi green)  Remote '($remote)' setup complete!(ansi reset)"
    print $"(ansi green)══════════════════════════════════════════════════════════(ansi reset)"
    print ""
    print $"(ansi cyan)current remotes:(ansi reset)"
    print $"($current_remotes.stdout | str trim)"
    print ""
    print $"(ansi cyan)quick commands:(ansi reset)"
    print $"  git push ($remote) main              # push main branch"
    print $"  git push ($remote) main --tags       # push main + tags"
    print $"  git push ($remote) --all             # push all branches"
    print ""
    print $"(ansi cyan)push to all remotes at once:(ansi reset)"
    print $"  just push-all"
    print ""
}
