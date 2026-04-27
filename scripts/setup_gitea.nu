#!/usr/bin/env nu
# Set up a Gitea remote for tui-spinner.
# Usage:
#   nu scripts/setup_gitea.nu <url>                                 # adds/updates 'gitea'
#   nu scripts/setup_gitea.nu <url> --remote gitea_starscream       # custom name
#   nu scripts/setup_gitea.nu <url> --push                          # push after setup
#
# Examples:
#   nu scripts/setup_gitea.nu ssh://git@192.168.1.204:30009/sorin/tui-spinner.git
#   nu scripts/setup_gitea.nu gitea@192.168.1.44:sorin/tui-spinner.git --remote gitea_starscream

def main [
    gitea_url: string          # Full Gitea repository URL
    --remote: string = "gitea" # Remote name (default: gitea)
    --push                     # Push all branches and tags after setup
] {
    let reset = (ansi reset)
    let red = (ansi red)
    let green = (ansi green)
    let yellow = (ansi yellow)
    let cyan = (ansi cyan)

    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print $"($cyan)  tui-spinner — Gitea remote setup($reset)"
    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print ""

    # ── check prerequisites ──────────────────────────────────────────
    print $"($cyan)▸ checking prerequisites ...($reset)"

    let has_git = (which git | length) > 0
    if not $has_git {
        print $"($red)  ✗ git is not installed($reset)"
        exit 1
    }
    print $"($green)  ✓ git is available($reset)"

    let in_repo = (do { git rev-parse --is-inside-work-tree } | complete)
    if $in_repo.exit_code != 0 {
        print $"($red)  ✗ not inside a git repository($reset)"
        exit 1
    }
    print $"($green)  ✓ inside a git repository($reset)"
    print ""

    # ── add or update the remote ─────────────────────────────────────
    let existing_remotes = (do { git remote } | complete)
    let remote_list = ($existing_remotes.stdout | str trim | lines)
    let remote_exists = ($remote_list | any {|r| $r == $remote})

    if $remote_exists {
        print $"($yellow)▸ remote '($remote)' already exists — updating URL ...($reset)"
        let set_result = (do { git remote set-url $remote $gitea_url } | complete)
        if $set_result.exit_code != 0 {
            print $"($red)  ✗ failed to update remote URL($reset)"
            print $"($red)    ($set_result.stderr | str trim)($reset)"
            exit 1
        }
        print $"($green)  ✓ remote '($remote)' URL updated to ($gitea_url)($reset)"
    } else {
        print $"($cyan)▸ adding remote '($remote)' ...($reset)"
        let add_result = (do { git remote add $remote $gitea_url } | complete)
        if $add_result.exit_code != 0 {
            print $"($red)  ✗ failed to add remote($reset)"
            print $"($red)    ($add_result.stderr | str trim)($reset)"
            exit 1
        }
        print $"($green)  ✓ remote '($remote)' added: ($gitea_url)($reset)"
    }
    print ""

    # ── test connection ──────────────────────────────────────────────
    print $"($cyan)▸ testing connection to '($remote)' ...($reset)"
    let ls_result = (do { git ls-remote --exit-code $remote } | complete)
    if $ls_result.exit_code == 0 {
        print $"($green)  ✓ connection successful($reset)"
    } else if $ls_result.exit_code == 2 {
        print $"($yellow)  ⚠ remote is reachable but has no refs (empty repository)($reset)"
    } else {
        print $"($yellow)  ⚠ could not reach remote — check URL and credentials($reset)"
        print $"($yellow)    ($ls_result.stderr | str trim)($reset)"
        print $"($yellow)    setup will continue — you can push once access is configured($reset)"
    }
    print ""

    # ── optionally push all branches and tags ────────────────────────
    if $push {
        print $"($cyan)▸ pushing all branches to '($remote)' ...($reset)"
        let push_result = (do { git push $remote --all } | complete)
        if $push_result.exit_code == 0 {
            print $"($green)  ✓ all branches pushed($reset)"
        } else {
            print $"($red)  ✗ branch push failed($reset)"
            print $"($red)    ($push_result.stderr | str trim)($reset)"
        }

        print $"($cyan)▸ pushing all tags to '($remote)' ...($reset)"
        let tags_result = (do { git push $remote --tags } | complete)
        if $tags_result.exit_code == 0 {
            print $"($green)  ✓ all tags pushed($reset)"
        } else {
            print $"($red)  ✗ tag push failed($reset)"
            print $"($red)    ($tags_result.stderr | str trim)($reset)"
        }
        print ""
    }

    # ── summary ──────────────────────────────────────────────────────
    let current_remotes = (do { git remote -v } | complete)

    print $"($green)═══════════════════════════════════════════════($reset)"
    print $"($green)  Remote '($remote)' setup complete!($reset)"
    print $"($green)═══════════════════════════════════════════════($reset)"
    print ""
    print $"($cyan)current remotes:($reset)"
    print $"($current_remotes.stdout | str trim)"
    print ""
    print $"($cyan)quick commands:($reset)"
    print $"  git push ($remote) main              # push main branch"
    print $"  git push ($remote) main --tags       # push main + tags"
    print $"  git push ($remote) --all             # push all branches"
    print ""
    print $"($cyan)push to all remotes at once:($reset)"
    print $"  just push-all"
    print ""
}
