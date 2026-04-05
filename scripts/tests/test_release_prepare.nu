#!/usr/bin/env nu
# test_release_prepare.nu — Tests for scripts/release_prepare.nu
# Tests version extraction from tags and tag format validation.
# Usage: nu scripts/tests/test_release_prepare.nu

use runner.nu [run_tests, assert_true, assert_false, assert_eq, assert_str_contains, assert_matches]

def main [] {
    let reset = (ansi reset)
    let cyan = (ansi cyan)
    let green = (ansi green)
    let red = (ansi red)

    print $"($cyan)── test_release_prepare.nu ────────────────────($reset)"

    let results = run_tests [
        {
            name: "test version extraction from v-prefixed tag",
            fn: {||
                let tag = "v0.2.0"
                let version = ($tag | str substring 1..)
                assert_eq $version "0.2.0" --msg "stripping 'v' from 'v0.2.0' should yield '0.2.0'"
            }
        },
        {
            name: "test version extraction from v-prefixed tag with pre-release",
            fn: {||
                let tag = "v1.0.0-alpha.1"
                let version = ($tag | str substring 1..)
                assert_eq $version "1.0.0-alpha.1" --msg "stripping 'v' should preserve pre-release suffix"
            }
        },
        {
            name: "test version extraction from various tags",
            fn: {||
                let cases = [
                    { tag: "v0.1.0", expected: "0.1.0" },
                    { tag: "v1.2.3", expected: "1.2.3" },
                    { tag: "v10.20.30", expected: "10.20.30" },
                    { tag: "v0.0.1-beta", expected: "0.0.1-beta" },
                    { tag: "v3.2.1-rc.5", expected: "3.2.1-rc.5" },
                ]
                for c in $cases {
                    let version = ($c.tag | str substring 1..)
                    assert_eq $version $c.expected --msg $"tag '($c.tag)' should yield version '($c.expected)'"
                }
            }
        },
        {
            name: "test tag must start with v",
            fn: {||
                let valid_tags = ["v0.1.0", "v1.0.0", "v0.0.1-alpha"]
                for tag in $valid_tags {
                    assert_true ($tag | str starts-with "v") --msg $"tag '($tag)' should start with 'v'"
                }

                let invalid_tags = ["0.1.0", "1.0.0", "release-1.0.0", "V1.0.0"]
                for tag in $invalid_tags {
                    assert_false ($tag | str starts-with "v") --msg $"tag '($tag)' should not start with lowercase 'v'"
                }
            }
        },
        {
            name: "test extracted version is valid semver",
            fn: {||
                let tags = ["v0.1.0", "v1.0.0", "v12.34.56", "v0.1.0-alpha", "v2.0.0-rc.1"]
                for tag in $tags {
                    let version = ($tag | str substring 1..)
                    let valid = ($version | parse --regex '^(\d+)\.(\d+)\.(\d+)(-[\w.]+)?$' | length) > 0
                    assert_true $valid --msg $"extracted version '($version)' from tag '($tag)' should be valid semver"
                }
            }
        },
        {
            name: "test invalid tag formats fail validation",
            fn: {||
                let bad_tags = ["v", "v1", "v1.0", "vabc", "v1.0.0.0", "v.1.0.0"]
                for tag in $bad_tags {
                    let version = ($tag | str substring 1..)
                    let valid = ($version | parse --regex '^(\d+)\.(\d+)\.(\d+)(-[\w.]+)?$' | length) > 0
                    assert_false $valid --msg $"version from bad tag '($tag)' should be invalid"
                }
            }
        },
        {
            name: "test version has exactly three numeric segments",
            fn: {||
                let tag = "v4.5.6"
                let version = ($tag | str substring 1..)
                let base = ($version | split row "-" | first)
                let parts = ($base | split row ".")
                assert_eq ($parts | length) 3 --msg "version should have exactly 3 dot-separated segments"
                assert_eq ($parts | get 0) "4" --msg "major should be 4"
                assert_eq ($parts | get 1) "5" --msg "minor should be 5"
                assert_eq ($parts | get 2) "6" --msg "patch should be 6"
            }
        },
        {
            name: "test install instructions contain correct version",
            fn: {||
                # Simulate the RELEASE_NOTES.md install-instructions block
                let version = "0.3.0"
                let toml_block = $"[dependencies]\ntui-spinner = \"($version)\""
                assert_str_contains $toml_block "tui-spinner" --msg "install instructions should reference tui-spinner"
                assert_str_contains $toml_block $version --msg "install instructions should contain the version"
            }
        },
        {
            name: "test cargo add instructions contain correct version",
            fn: {||
                let version = "0.3.0"
                let cmd = $"cargo add tui-spinner@($version)"
                assert_str_contains $cmd "tui-spinner@0.3.0" --msg "cargo add command should include package@version"
            }
        },
        {
            name: "test release_prepare rejects tag without v prefix",
            fn: {||
                let result = (do { nu scripts/release_prepare.nu "0.2.0" } | complete)
                assert_true ($result.exit_code != 0) --msg "release_prepare should reject tags without 'v' prefix"
            }
        },
        {
            name: "test release_prepare rejects malformed tag",
            fn: {||
                let result = (do { nu scripts/release_prepare.nu "vabc" } | complete)
                assert_true ($result.exit_code != 0) --msg "release_prepare should reject malformed tag 'vabc'"
            }
        },
        {
            name: "test Cargo.toml version update simulation",
            fn: {||
                # Simulate what release_prepare does to update the version
                let sample_toml = "[package]\nname = \"tui-spinner\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"
                let new_version = "0.5.0"
                let updated = ($sample_toml | str replace --regex 'version\s*=\s*"[^"]+"' $'version = "($new_version)"')
                assert_str_contains $updated $'version = "0.5.0"' --msg "Cargo.toml version should be updated"
                assert_str_contains $updated 'name = "tui-spinner"' --msg "package name should be preserved"
                assert_str_contains $updated 'edition = "2021"' --msg "edition should be preserved"
            }
        },
    ]

    # ── summary ──────────────────────────────────────────────────────
    if $results.failed > 0 {
        print $"($red)  ($results.failed) test\(s\) failed($reset)"
        exit 1
    } else {
        print $"($green)  all ($results.passed) test\(s\) passed($reset)"
    }
}
