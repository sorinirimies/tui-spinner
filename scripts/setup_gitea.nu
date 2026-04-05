#!/usr/bin/env nu
# Set up Gitea as a second remote for tui-spinner.
# Usage: nu scripts/setup_gitea.nu <gitea_url> [--push]
#
# Example:
#   nu scripts/setup_gitea.nu https://gitea.example.com/sorin/tui-spinner.git
#   nu scripts/setup_gitea.nu https://gitea.example.com/sorin/tui-spinner.git --push

def main [
    gitea_url: string  # Full Gitea repository URL (e.g. https://gitea.example.com/user/repo.git)
    --push             # Push all branches and tags after setup
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

    # ── add or update the gitea remote ───────────────────────────────
    let existing_remotes = (do { git remote } | complete)
    let remote_list = ($existing_remotes.stdout | str trim | lines)
    let gitea_exists = ($remote_list | any {|r| $r == "gitea"})

    if $gitea_exists {
        print $"($yellow)▸ remote 'gitea' already exists — updating URL ...($reset)"
        let set_result = (do { git remote set-url gitea $gitea_url } | complete)
        if $set_result.exit_code != 0 {
            print $"($red)  ✗ failed to update gitea remote URL($reset)"
            print $"($red)    ($set_result.stderr | str trim)($reset)"
            exit 1
        }
        print $"($green)  ✓ gitea remote URL updated to ($gitea_url)($reset)"
    } else {
        print $"($cyan)▸ adding remote 'gitea' ...($reset)"
        let add_result = (do { git remote add gitea $gitea_url } | complete)
        if $add_result.exit_code != 0 {
            print $"($red)  ✗ failed to add gitea remote($reset)"
            print $"($red)    ($add_result.stderr | str trim)($reset)"
            exit 1
        }
        print $"($green)  ✓ gitea remote added: ($gitea_url)($reset)"
    }
    print ""

    # ── test connection ──────────────────────────────────────────────
    print $"($cyan)▸ testing connection to gitea remote ...($reset)"
    let ls_result = (do { git ls-remote --exit-code gitea } | complete)
    if $ls_result.exit_code == 0 {
        print $"($green)  ✓ connection successful($reset)"
    } else if $ls_result.exit_code == 2 {
        # exit code 2 means the remote exists but is empty (no refs)
        print $"($yellow)  ⚠ remote is reachable but has no refs (empty repository)($reset)"
    } else {
        print $"($yellow)  ⚠ could not reach gitea remote — check URL and credentials($reset)"
        print $"($yellow)    ($ls_result.stderr | str trim)($reset)"
        print $"($yellow)    setup will continue — you can push once access is configured($reset)"
    }
    print ""

    # ── optionally push all branches and tags ────────────────────────
    if $push {
        print $"($cyan)▸ pushing all branches to gitea ...($reset)"
        let push_result = (do { git push gitea --all } | complete)
        if $push_result.exit_code == 0 {
            print $"($green)  ✓ all branches pushed($reset)"
        } else {
            print $"($red)  ✗ branch push failed($reset)"
            print $"($red)    ($push_result.stderr | str trim)($reset)"
        }

        print $"($cyan)▸ pushing all tags to gitea ...($reset)"
        let tags_result = (do { git push gitea --tags } | complete)
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
    print $"($green)  Gitea remote setup complete!($reset)"
    print $"($green)═══════════════════════════════════════════════($reset)"
    print ""
    print $"($cyan)current remotes:($reset)"
    print $"($current_remotes.stdout | str trim)"
    print ""
    print $"($cyan)quick commands:($reset)"
    print $"  git push gitea main              # push main branch"
    print $"  git push gitea main --tags       # push main + tags"
    print $"  git push gitea --all             # push all branches"
    print $"  git push gitea --mirror          # full mirror"
    print ""
    print $"($cyan)push to both remotes at once:($reset)"
    print $"  git push origin main --tags && git push gitea main --tags"
    print ""
}
