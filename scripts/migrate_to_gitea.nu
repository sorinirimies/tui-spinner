#!/usr/bin/env nu
# Migrate tui-spinner to dual GitHub + Gitea hosting.
# Adds a gitea remote, tests connectivity, optionally pushes, and copies workflows.
# Usage: nu scripts/migrate_to_gitea.nu --gitea-url <url> [--project-dir <dir>] [--push] [--copy-workflows]
#
# Example:
#   nu scripts/migrate_to_gitea.nu --gitea-url https://gitea.example.com/sorin/tui-spinner.git
#   nu scripts/migrate_to_gitea.nu --gitea-url https://gitea.example.com/sorin/tui-spinner.git --push --copy-workflows

def main [
    --project-dir: string = "."  # Path to the project root (defaults to current directory)
    --gitea-url: string          # Full Gitea repository URL (e.g. https://gitea.example.com/user/repo.git)
    --push                       # Push all branches and tags after setup
    --copy-workflows             # Copy .github/workflows to .gitea/workflows
] {
    let reset = (ansi reset)
    let red = (ansi red)
    let green = (ansi green)
    let yellow = (ansi yellow)
    let cyan = (ansi cyan)

    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print $"($cyan)  tui-spinner — migrate to dual hosting($reset)"
    print $"($cyan)  GitHub + Gitea($reset)"
    print $"($cyan)═══════════════════════════════════════════════($reset)"
    print ""

    if ($gitea_url | is-empty) {
        print $"($red)error:($reset) --gitea-url is required"
        print "  Usage: nu scripts/migrate_to_gitea.nu --gitea-url <url>"
        exit 1
    }

    # ── enter project directory ──────────────────────────────────────
    let dir = ($project_dir | path expand)
    if not ($dir | path exists) {
        print $"($red)error:($reset) project directory does not exist: ($dir)"
        exit 1
    }
    cd $dir

    print $"($cyan)project dir:($reset) ($dir)"
    print $"($cyan)gitea url:  ($reset) ($gitea_url)"
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

    # ── check for existing origin remote (GitHub) ────────────────────
    let existing_remotes = (do { git remote } | complete)
    let remote_list = ($existing_remotes.stdout | str trim | lines)
    let has_origin = ($remote_list | any {|r| $r == "origin"})

    if $has_origin {
        let origin_url = (do { git remote get-url origin } | complete)
        print $"($green)  ✓ origin remote found: ($origin_url.stdout | str trim)($reset)"
    } else {
        print $"($yellow)  ⚠ no 'origin' remote found — you may want to add one for GitHub($reset)"
    }
    print ""

    # ── add or update the gitea remote ───────────────────────────────
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
        print $"($yellow)  ⚠ remote is reachable but has no refs \(empty repository\)($reset)"
    } else {
        print $"($yellow)  ⚠ could not reach gitea remote — check URL and credentials($reset)"
        print $"($yellow)    ($ls_result.stderr | str trim)($reset)"
        print $"($yellow)    migration will continue — you can push once access is configured($reset)"
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

    # ── optionally copy workflows ────────────────────────────────────
    if $copy_workflows {
        let github_workflows = ".github/workflows"
        let gitea_workflows = ".gitea/workflows"

        if ($github_workflows | path exists) {
            let workflow_files = (ls $github_workflows | where type == "file" | get name)
            if ($workflow_files | length) > 0 {
                print $"($cyan)▸ copying GitHub workflows to Gitea ...($reset)"

                mkdir $gitea_workflows
                print $"($green)  ✓ created ($gitea_workflows)($reset)"

                for file in $workflow_files {
                    let filename = ($file | path basename)
                    let dest = ($gitea_workflows | path join $filename)
                    cp $file $dest
                    print $"($green)  ✓ copied ($filename)($reset)"
                }

                print $"($yellow)  ⚠ review copied workflows — Gitea Actions may need adjustments($reset)"
                print $"($yellow)    • Gitea uses 'runs-on: ubuntu-latest' or custom labels($reset)"
                print $"($yellow)    • Some GitHub-specific actions may need alternatives($reset)"
                print ""
            } else {
                print $"($yellow)  ⚠ ($github_workflows) exists but contains no files($reset)"
                print ""
            }
        } else {
            print $"($yellow)  ⚠ ($github_workflows) does not exist — nothing to copy($reset)"
            print ""
        }
    }

    # ── summary ──────────────────────────────────────────────────────
    let current_remotes = (do { git remote -v } | complete)

    print $"($green)═══════════════════════════════════════════════($reset)"
    print $"($green)  migration to dual hosting complete!($reset)"
    print $"($green)═══════════════════════════════════════════════($reset)"
    print ""
    print $"($cyan)current remotes:($reset)"
    print $"($current_remotes.stdout | str trim)"
    print ""
    print $"($cyan)dual-push workflow:($reset)"
    print $"  git push origin main --tags && git push gitea main --tags"
    print ""
    print $"($cyan)quick commands:($reset)"
    print $"  git push gitea main              # push main branch"
    print $"  git push gitea main --tags       # push main + tags"
    print $"  git push gitea --all             # push all branches"
    print $"  git push gitea --mirror          # full mirror"
    print ""
    print $"($cyan)tip:($reset) add a git alias for dual push:"
    print $"  git config alias.push-all '!git push origin main --tags && git push gitea main --tags'"
    print $"  then use: git push-all"
    print ""
}
