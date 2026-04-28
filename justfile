# tui-spinner - task runner
# Install just:      cargo install just
# Install git-cliff: cargo install git-cliff
# Install vhs:       brew install vhs  OR  go install github.com/charmbracelet/vhs@latest
# Usage: just <task>
# -- Default ---------------------------------------------------------------

default:
    @just --list

# -- Tool checks -----------------------------------------------------------

_check-git-cliff:
    @command -v git-cliff >/dev/null 2>&1 || { \
        echo "git-cliff not found. Install with: cargo install git-cliff"; exit 1; \
    }

# Check nu (nushell) is available
_check-nu:
    @command -v nu >/dev/null 2>&1 || { \
        echo "nu (nushell) not found. Install: https://www.nushell.sh"; exit 1; \
    }

_check-vhs:
    @command -v vhs >/dev/null 2>&1 || { \
        echo "vhs not found."; \
        echo "   macOS:      brew install vhs"; \
        echo "   Any:        go install github.com/charmbracelet/vhs@latest"; \
        exit 1; \
    }

# Install all recommended development tools
install-tools:
    @echo "Installing development tools..."
    @command -v git-cliff >/dev/null 2>&1 || cargo install git-cliff
    @command -v nu >/dev/null 2>&1 && echo "nu found" || echo "nu (nushell) not found. Install: https://www.nushell.sh"
    @echo "All tools installed!"

# -- Build -----------------------------------------------------------------

# Build the library (dev)
build:
    cargo build

# Build release version
build-release:
    cargo build --release

# -- Run -------------------------------------------------------------------

# Run the main spinner demo example
run:
    cargo run --example spinner

# -- Test ------------------------------------------------------------------

# Run the Rust test suite
test:
    cargo test

# Run Nu script tests
test-nu: _check-nu
    nu scripts/tests/run_all.nu

# Run both Rust and Nu tests
test-all: test test-nu
    @echo "All Rust and Nu tests passed!"

# -- Code quality ----------------------------------------------------------

# Check without building
check:
    cargo check

# Format all code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt --check

# Run clippy
clippy:
    cargo clippy -- -D warnings

# Run all quality checks (fmt, clippy, test) - must pass before a release
check-all: fmt-check clippy test
    @echo "All checks passed!"

# -- VHS Demo GIFs --------------------------------------------------------

VHS_DIR := "examples/vhs"
VHS_GENERATED := "examples/vhs/generated"

# Generate all VHS demo GIFs
vhs-all: _check-vhs
    @mkdir -p {{ VHS_GENERATED }}
    @echo "=== tui-spinner VHS Tapes ==="
    @for tape in {{ VHS_DIR }}/*.tape; do \
        echo ">>  $$tape"; \
        vhs "$$tape" || echo "Failed: $$tape"; \
    done
    @echo "Demo GIFs generated -> {{ VHS_GENERATED }}/"

# Render a single tape by name, e.g.: just vhs-tape spinner-demo
vhs-tape name: _check-vhs
    @if [ -f "{{ VHS_DIR }}/{{ name }}.tape" ]; then \
        echo ">>  {{ VHS_DIR }}/{{ name }}.tape"; \
        vhs "{{ VHS_DIR }}/{{ name }}.tape" && echo "Done."; \
    else \
        echo "Tape not found: {{ name }}.tape"; \
        echo ""; \
        just vhs-list; \
        exit 1; \
    fi

# List all available VHS tapes and any already-generated GIFs
vhs-list:
    @echo "Tapes  ->  {{ VHS_DIR }}/"
    @ls {{ VHS_DIR }}/*.tape 2>/dev/null | sed 's|.*/||; s|\.tape||' | sed 's/^/  /' || echo "  (none)"
    @echo ""
    @echo "Generated  ->  {{ VHS_GENERATED }}/"
    @ls {{ VHS_GENERATED }}/*.gif 2>/dev/null | sed 's|.*/||' | sed 's/^/  /' || echo "  (none yet)"

# Pull GIF files from Git LFS (run once after a fresh clone)
lfs-pull:
    @command -v git-lfs >/dev/null 2>&1 || { \
        echo "git-lfs not found. Install with: brew install git-lfs"; exit 1; \
    }
    git lfs pull
    @echo "LFS objects pulled."

# -- Documentation ---------------------------------------------------------

# Generate and open docs in browser
doc:
    cargo doc --no-deps --open

# -- Changelog -------------------------------------------------------------

# Regenerate the full CHANGELOG.md from all tags
changelog: _check-git-cliff
    @echo "Generating full changelog..."
    git-cliff --output CHANGELOG.md
    @echo "CHANGELOG.md updated."

# Prepend only unreleased commits to CHANGELOG.md
changelog-unreleased: _check-git-cliff
    git-cliff --unreleased --prepend CHANGELOG.md
    @echo "Unreleased changes prepended."

# Preview changelog for the next release without writing the file
changelog-preview: _check-git-cliff
    @git-cliff --unreleased

# -- Version bump ----------------------------------------------------------
# Usage: just bump 0.2.0
#
# Runs fmt -> clippy -> test -> changelog -> commit -> tag, then shows push hints.

# Bump the version, regenerate Cargo.lock + CHANGELOG.md, commit and tag.
bump version: check-all _check-git-cliff _check-nu
    nu scripts/bump_version.nu --yes {{ version }}

# -- Publish (crates.io) --------------------------------------------------

# Run the full pre-publish readiness check (fmt, clippy, tests, docs, dry-run)
check-publish: _check-nu
    nu scripts/check_publish.nu

# Dry-run publish
publish-dry: check-all
    cargo publish --dry-run

# Publish to crates.io
publish: check-all
    cargo publish

# Show what would be released without making any changes
release-preview: _check-git-cliff
    @echo "Current version:"
    @just version
    @echo ""
    @echo "Unreleased commits:"
    @git-cliff --unreleased
    @echo ""
    @echo "Published crate:  tui-spinner"

# -- Housekeeping ----------------------------------------------------------

# Remove build artifacts
clean:
    cargo clean

# Update all dependencies (Cargo.lock only)
update:
    cargo update

# Update dependencies, run the full quality gate, then commit and push if all green.
update-deps:
    @echo "Updating dependencies..."
    cargo update
    @echo "Running quality gate..."
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo test
    @echo "All checks passed - committing dependency updates..."
    git add Cargo.lock
    git diff --cached --quiet || git commit -m "chore: update dependencies"
    git push origin main
    @echo "Dependency updates pushed to GitHub."

# Show outdated dependencies (requires cargo-outdated)
outdated:
    cargo outdated

# Show the current crate version
version: _check-nu
    @nu scripts/version.nu

# -- Git remotes -----------------------------------------------------------

# Show all configured remotes
remotes:
    @git remote -v

# Push main branch to GitHub
push:
    git push origin main

# Push main branch to Gitea
push-gitea:
    git push gitea main

# Push main branch to Gitea Starscream
push-gitea-starscream:
    git push gitea_starscream main

# Push main to all remotes (continues on failure)
push-all:
    #!/usr/bin/env sh
    failed=""
    git push origin main             || failed="$failed origin"
    git push gitea main              || failed="$failed gitea"
    git push gitea_starscream main   || failed="$failed gitea_starscream"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to push to:$failed"
    else
        echo "✅ Pushed to GitHub, Gitea, and Gitea Starscream!"
    fi

# Force-push the current branch to all remotes
push-all-force:
    #!/usr/bin/env sh
    failed=""
    git push --force origin main             || failed="$failed origin"
    git push --force gitea main              || failed="$failed gitea"
    git push --force gitea_starscream main   || failed="$failed gitea_starscream"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to force-push to:$failed"
    else
        echo "✅ Force-pushed to GitHub, Gitea, and Gitea Starscream!"
    fi

# Pull the current branch from GitHub (origin)
pull:
    git pull origin main

# Pull the current branch from Gitea
pull-gitea:
    git pull gitea main

# Pull the current branch from Gitea Starscream
pull-gitea-starscream:
    git pull gitea_starscream main

# Pull the current branch from all remotes (continues on failure)
pull-all:
    #!/usr/bin/env sh
    failed=""
    git pull origin main             || failed="$failed origin"
    git pull gitea main              || failed="$failed gitea"
    git pull gitea_starscream main   || failed="$failed gitea_starscream"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to pull from:$failed"
    else
        echo "✅ Pulled from GitHub, Gitea, and Gitea Starscream!"
    fi

# Push all tags to GitHub
push-tags:
    git push origin --tags

# Push tags to Gitea
push-tags-gitea:
    git push gitea --tags

# Push all tags to all remotes (continues on failure)
push-tags-all:
    #!/usr/bin/env sh
    failed=""
    git push origin --tags             || failed="$failed origin"
    git push gitea --tags              || failed="$failed gitea"
    git push gitea_starscream --tags   || failed="$failed gitea_starscream"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to push tags to:$failed"
    else
        echo "✅ Tags pushed to all remotes!"
    fi

# Push the latest commit + tags to every remote (quality-gated; continues on failure)
push-release-all: check-all
    #!/usr/bin/env sh
    failed=""
    git push --follow-tags origin main             || failed="$failed origin"
    git push --follow-tags gitea main              || failed="$failed gitea"
    git push --follow-tags gitea_starscream main   || failed="$failed gitea_starscream"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to push to:$failed"
    else
        echo "✅ Latest commit + tags pushed to all remotes."
    fi

# -- Release workflows -----------------------------------------------------

# Bump, commit, tag, then push to GitHub — triggers Release workflow
release version: bump
    @echo "Pushing release v{{ version }} to GitHub..."
    git push --follow-tags origin main
    @echo "✅ Release v{{ version }} pushed — Release workflow will trigger automatically."

# Bump, commit, tag, then push to Gitea only
release-gitea version: bump
    @echo "Pushing release v{{ version }} to Gitea..."
    git push --follow-tags gitea main
    @echo "✅ Release v{{ version }} live on Gitea."

# Bump, commit, tag, then push to Gitea Starscream only
release-gitea-starscream version: bump
    @echo "Pushing release v{{ version }} to Gitea Starscream..."
    git push --follow-tags gitea_starscream main
    @echo "✅ Release v{{ version }} live on Gitea Starscream."

# Bump, commit, tag, then push to all remotes (continues on failure)
release-all version: bump
    #!/usr/bin/env sh
    echo "Pushing release v{{ version }} to all remotes..."
    failed=""
    git push --follow-tags origin main             || failed="$failed origin"
    git push --follow-tags gitea main              || failed="$failed gitea"
    git push --follow-tags gitea_starscream main   || failed="$failed gitea_starscream"
    if [ -n "$failed" ]; then
        echo "⚠️  Release v{{ version }} failed to push to:$failed"
    else
        echo "✅ Release v{{ version }} pushed to GitHub, Gitea, and Gitea Starscream!"
    fi

# Manually re-trigger the Release workflow for an existing tag via the gh CLI
release-retrigger version:
    @command -v gh >/dev/null 2>&1 || { \
        echo "GitHub CLI (gh) not found. Install from https://cli.github.com"; exit 1; \
    }
    @echo "Manually dispatching Release workflow for tag v{{ version }}..."
    gh workflow run release.yml --field tag=v{{ version }}
    @echo "✅ Dispatched — check progress in GitHub Actions."

# -- Gitea sync ------------------------------------------------------------

# Force-sync Gitea with GitHub
sync-gitea:
    git push gitea main --force
    git push gitea --tags --force
    @echo "✅ Gitea force-synced with GitHub."

# Force-sync Gitea Starscream with GitHub
sync-gitea-starscream:
    git push gitea_starscream main --force
    git push gitea_starscream --tags --force
    @echo "✅ Gitea Starscream force-synced with GitHub."

# Force-sync all Gitea instances with GitHub (continues on failure)
sync-all-gitea:
    #!/usr/bin/env sh
    failed=""
    git push gitea main --force                  || failed="$failed gitea"
    git push gitea --tags --force                || failed="$failed gitea-tags"
    git push gitea_starscream main --force       || failed="$failed gitea_starscream"
    git push gitea_starscream --tags --force     || failed="$failed gitea_starscream-tags"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to sync:$failed"
    else
        echo "✅ All Gitea instances force-synced with GitHub."
    fi

# -- Gitea setup -----------------------------------------------------------

# Add or update the 'gitea' remote (usage: just setup-gitea <url>)
setup-gitea url: _check-nu
    nu scripts/setup_gitea.nu --remote gitea {{ url }}

# Add or update the 'gitea_starscream' remote (usage: just setup-gitea-starscream <url>)
setup-gitea-starscream url: _check-nu
    nu scripts/setup_gitea.nu --remote gitea_starscream {{ url }}

# Migrate this project to dual GitHub + Gitea hosting (interactive)
migrate-gitea: _check-nu
    nu scripts/migrate_to_gitea.nu
