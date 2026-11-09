# Changelog

All notable changes to this project are documented here. It is generated from
[Conventional Commits](https://www.conventionalcommits.org) by
[git-cliff](https://git-cliff.org) — do not edit it by hand.

## [0.5.2](https://github.com/sorinirimies/tui-spinner/compare/v0.5.1...v0.5.2) — 2026-11-09

### ♻️ Refactor

- Remove dead_code allows the build doesn't need; gate on unused dependencies ([`4d8f52f`](https://github.com/sorinirimies/tui-spinner/commit/4d8f52f2c540c2bab374178258854fdb9731d906))

### 📚 Documentation

- Drop the per-repo agent rules skill; the shared skills cover it ([`e6771bf`](https://github.com/sorinirimies/tui-spinner/commit/e6771bf383c670f0cff67254929a74f24f7967d2))
- This repo's agent rules as a project skill in .agents/skills ([`3ae7f99`](https://github.com/sorinirimies/tui-spinner/commit/3ae7f999cf81076b64be1d0fe6f4d90457676c8d))
- **readme:** Add the Audit and Fuzz badges ([`550b439`](https://github.com/sorinirimies/tui-spinner/commit/550b43990099dae92ad217f3772c33e3fd92957a))
- Move agent rules from .zed/rules.md to AGENTS.md ([`6cb332d`](https://github.com/sorinirimies/tui-spinner/commit/6cb332da16a036054154d581293f20e208daf21c))

### 📦 Dependencies

- Nightly cargo update 2026-11-09 ([`a2b75b7`](https://github.com/sorinirimies/tui-spinner/commit/a2b75b7f2dda00b037991ab573b68cbd409493fc))
- Nightly cargo update 2026-11-07 ([`f5e93e7`](https://github.com/sorinirimies/tui-spinner/commit/f5e93e78991d53a963f69b5d52d0b6b94b8b693d))
- Nightly cargo update 2026-11-06 ([`ea7f5c5`](https://github.com/sorinirimies/tui-spinner/commit/ea7f5c5ca62579155ad5a7feab0b3a7b209367c1))
- Update Cargo.lock ([`929aaa2`](https://github.com/sorinirimies/tui-spinner/commit/929aaa2edc466acb11c3ee2645713d7bd78a0102))
- Update Cargo.lock ([`f335946`](https://github.com/sorinirimies/tui-spinner/commit/f335946cba94e95dfcac88f9ffbfec567bfc5bfe))
- Update Cargo.lock ([`45316d4`](https://github.com/sorinirimies/tui-spinner/commit/45316d4c7ff39b34c1afde1abc7f95fd01d3d485))

### 📦 Build

- Align tooling with the library template ([`90e86c7`](https://github.com/sorinirimies/tui-spinner/commit/90e86c779bf4fa7ccdae20bce2ab25f3467bfc89))

### 🔄 CI and release

- Cross-check macOS targets from Linux, tolerate macOS runner starvation ([`ab496d0`](https://github.com/sorinirimies/tui-spinner/commit/ab496d03528f8e156d141b6f3fdbcc60d4f84173))
- **gitea:** Check the MSRV on Gitea too, with the same command as GitHub ([`29121d0`](https://github.com/sorinirimies/tui-spinner/commit/29121d09d68b06c9770240c2bfdb665f9e5f2873))
- Download archives to a file so curl retries a dropped transfer, instead of piping into tar ([`d119369`](https://github.com/sorinirimies/tui-spinner/commit/d1193699410c161673c7920edae16637399cc087))
- Bump softprops/action-gh-release from 2 to 3 (#5) ([`c7f6723`](https://github.com/sorinirimies/tui-spinner/commit/c7f6723c24a427829ca06207fa8ad2c6fb8fd201))
- The Dependabot sweep asks for a rebase on conflicts and keeps going after a failed merge ([`4722a54`](https://github.com/sorinirimies/tui-spinner/commit/4722a54a404e09f631da68d0c60fed1c156fe160))
- Merge green Dependabot PRs, majors included, with a nightly sweep; nightly commits skip ignored lockfiles ([`73b66c7`](https://github.com/sorinirimies/tui-spinner/commit/73b66c7a04e76a447fd9858acf885cd03a842b9f))
- Scan the git history for secrets with gitleaks ([`b7f621c`](https://github.com/sorinirimies/tui-spinner/commit/b7f621cdb981118f820c53639afdd7063a991a92))

### 🔧 Chores

- Ignore .codegraph index ([`5410089`](https://github.com/sorinirimies/tui-spinner/commit/54100891ddc0d6bb169a4a6f3fc3abc39bec4646))
- Remove AGENTS.md ([`35df278`](https://github.com/sorinirimies/tui-spinner/commit/35df2783fe6e451983b26e0f4a00390caccfd689))

## [0.5.1](https://github.com/sorinirimies/tui-spinner/compare/v0.5.0...v0.5.1) — 2026-10-28

### ✨ Features

- Enable the tui-nuances theme feature by default ([`978d772`](https://github.com/sorinirimies/tui-spinner/commit/978d7720fd80283110972a08cd7588ba24564945))

### 📦 Dependencies

- Update Cargo.lock ([`0daa67e`](https://github.com/sorinirimies/tui-spinner/commit/0daa67e4c0e1e939a2abcda5842ff94dc022c8ba))
- Update Cargo.lock ([`4b98fdc`](https://github.com/sorinirimies/tui-spinner/commit/4b98fdc3f393a5b225bb26fec10806bc481862d8))
- Bump tui-nuances to 0.1.10 ([`e0a38a6`](https://github.com/sorinirimies/tui-spinner/commit/e0a38a65037a76ca9339d94c25f757e2eef804aa))
- Update Cargo.lock ([`cebc099`](https://github.com/sorinirimies/tui-spinner/commit/cebc099da0b40da1a9cad07cf16ba07126e560c0))
- Update Cargo.lock ([`8d82feb`](https://github.com/sorinirimies/tui-spinner/commit/8d82febbee17f6c5399345a0749007c79fb59536))
- Update Cargo.lock ([`4721e02`](https://github.com/sorinirimies/tui-spinner/commit/4721e0249240ea76dfde240c1801c546d50a5e18))
- Update Cargo.lock ([`f062d08`](https://github.com/sorinirimies/tui-spinner/commit/f062d080016e73a5c2553b86e66867f12707aed7))
- Update Cargo.lock ([`865dd13`](https://github.com/sorinirimies/tui-spinner/commit/865dd132898097d9d97a786c49b967a9b400f4be))
- Update Cargo.lock ([`54826ef`](https://github.com/sorinirimies/tui-spinner/commit/54826effbcc626103e05f79c3e34447e76022211))

### 🔄 CI and release

- Bump actions/checkout from 6 to 7 (#4) ([`479d97a`](https://github.com/sorinirimies/tui-spinner/commit/479d97a77055b200a30ed1d4f84ab33411ac09e4))
- Bump softprops/action-gh-release from 2 to 3 (#3) ([`3126491`](https://github.com/sorinirimies/tui-spinner/commit/31264910ae40ae9fedde233b0cf6cee24ee549b5))
- **gitea:** Nightly workflow-action bumps ([`a28069a`](https://github.com/sorinirimies/tui-spinner/commit/a28069ab2aa5c0f430a230fb102f6f28cdcaae26))
- Automerge major dependabot bumps too ([`6097849`](https://github.com/sorinirimies/tui-spinner/commit/6097849f39ad5c0bc47133e037b4720437ca9157))
- Automerge green PRs (workflow_run) + dependabot for actions ([`99cb596`](https://github.com/sorinirimies/tui-spinner/commit/99cb5962a7435af7792cc603e4c24f750140d868))

### 🔧 Chores

- Bump tui-nuances to 0.1.7 ([`ac08ab7`](https://github.com/sorinirimies/tui-spinner/commit/ac08ab7ac92d173503d04e5f3c62f4a7b35a409e))
- **justfile:** Push/pull current branch instead of hardcoded main ([`e699acf`](https://github.com/sorinirimies/tui-spinner/commit/e699acf723d7af1f9938475ffa7804c26f49bb48))
- **justfile:** Skip unconfigured remotes in *-all recipes ([`901abd4`](https://github.com/sorinirimies/tui-spinner/commit/901abd4bb3ad9d6eec11b4f18e49b131d7cd534f))

## [0.5.0](https://github.com/sorinirimies/tui-spinner/compare/v0.4.34...v0.5.0) — 2026-10-14

### ✨ Features

- Make theming an optional "theme" feature ([`0620962`](https://github.com/sorinirimies/tui-spinner/commit/0620962af014e8a2e8a8681520d916baad1ff456))
- Shared theming via tui-nuances ([`198c1fd`](https://github.com/sorinirimies/tui-spinner/commit/198c1fde0fc9412f2846a6cbea2806b6e56b6f3f))

### 🐛 Bug fixes

- Bound sizes and tick walks so huge values cannot hang or exhaust memory ([`f9ce31a`](https://github.com/sorinirimies/tui-spinner/commit/f9ce31ad31cd4824506434429cfe74ec46f561f2))
- **scripts:** Drop deprecated str downcase; exit non-zero when bump is aborted ([`c176e5c`](https://github.com/sorinirimies/tui-spinner/commit/c176e5cccb5dbf68da8a89850a3c4add9e42300c))

### 📚 Documentation

- Re-record demos showing results only; reference tui-nuances version ([`6f1f96c`](https://github.com/sorinirimies/tui-spinner/commit/6f1f96ccd9531d75979833bd5f111bdafe8267b9))

### 📦 Dependencies

- Update Cargo.lock ([`44a7924`](https://github.com/sorinirimies/tui-spinner/commit/44a7924694da67b440b5455cf167975e9d891959))
- Update Cargo.lock ([`7a5e9bf`](https://github.com/sorinirimies/tui-spinner/commit/7a5e9bff73f4059c12e2c8ed2ae3e7dd32556be3))
- Update Cargo.lock ([`3ca18c4`](https://github.com/sorinirimies/tui-spinner/commit/3ca18c41563d054ffe516c46c2caa494f50116d7))
- Update Cargo.lock ([`b481ff3`](https://github.com/sorinirimies/tui-spinner/commit/b481ff33864c9f63603076bc9559cd46ee1be78f))

### 🔄 CI and release

- Prebuilt Nushell, cargo-deny + cargo-audit security workflow; add cargo-fuzz; tui-nuances 0.1.6 ([`dc49d5f`](https://github.com/sorinirimies/tui-spinner/commit/dc49d5f5637e1df0515e70b0e79f56ad2d9e7f35))
- **security:** Add cargo-audit workflow and just audit recipe ([`1f0610c`](https://github.com/sorinirimies/tui-spinner/commit/1f0610c58bdd627cc7f41836ebacffc68d4d19c4))
- Release only on Cargo.toml dep changes; commit lock-only updates without release ([`e388fe4`](https://github.com/sorinirimies/tui-spinner/commit/e388fe4e2aa8c14b044ffd3fa02284c57563aded))

### 🔧 Chores

- Depend on tui-nuances 0.1.6 from crates.io ([`ca00d22`](https://github.com/sorinirimies/tui-spinner/commit/ca00d22703af6f8062162916b07c84319c85c6b8))
- **just:** Add force-push recipes (per-remote, tags, push-all-force-with-tags) ([`14e4198`](https://github.com/sorinirimies/tui-spinner/commit/14e4198d43b280be61d1f31a7b892b3577616029))

## [0.4.21](https://github.com/sorinirimies/tui-spinner/compare/v0.4.20...v0.4.21) — 2026-09-17

### 🐛 Bug fixes

- **ci:** Gate deps-update has_changes on Cargo.lock only ([`8c19449`](https://github.com/sorinirimies/tui-spinner/commit/8c194496bcc59d0f00e1ea32eb3d3fb4e5b8ca02))

## [0.4.9](https://github.com/sorinirimies/tui-spinner/compare/v0.4.7...v0.4.9) — 2026-08-22

### 📚 Documentation

- Bump README install version pin to 0.4 ([`2133365`](https://github.com/sorinirimies/tui-spinner/commit/213336573157c8dcf567c30a1eae8d5ea9b9eeb1))

## [0.4.2](https://github.com/sorinirimies/tui-spinner/compare/v0.4.1...v0.4.2) — 2026-08-08

### 🔄 CI and release

- **deps-update:** Explicitly dispatch release.yml (GITHUB_TOKEN pushes don't trigger workflows) ([`f677803`](https://github.com/sorinirimies/tui-spinner/commit/f6778035bf33860394aba927fe0485125567278c))

## [0.4.1](https://github.com/sorinirimies/tui-spinner/compare/v0.4.0...v0.4.1) — 2026-08-05

### 📚 Documentation

- Apply consistent flat-square styling to README badges ([`75fe596`](https://github.com/sorinirimies/tui-spinner/commit/75fe596bd73bffc89241104528831c1ebf7bae5c))

### 🔄 CI and release

- **deps-update:** Auto-bump patch version + tag to trigger release ([`a1021c1`](https://github.com/sorinirimies/tui-spinner/commit/a1021c1efe10701230ff02bd66db0dbca25568c6))
- Fail the release loudly when crates.io publish is skipped ([`a9dedb8`](https://github.com/sorinirimies/tui-spinner/commit/a9dedb8b91c29a88bede0c84dfaef36465c50eba))

## [0.4.0](https://github.com/sorinirimies/tui-spinner/compare/v0.3.0...v0.4.0) — 2026-07-10

### ✨ Features

- **spinner:** Implement Into<Text> so spinners embed with no special method ([`6c36f51`](https://github.com/sorinirimies/tui-spinner/commit/6c36f5190f5859c4c6173bcfcbaac3dc97ac3add))

### 📚 Documentation

- **examples:** Add table_embed demo for spinners in Table cells ([`b369114`](https://github.com/sorinirimies/tui-spinner/commit/b3691143474ff35b3dba1b9f1055a75a581f6525))

### 🔄 CI and release

- Install git-cliff via prebuilt taiki-e/install-action on GitHub ([`8b1ee94`](https://github.com/sorinirimies/tui-spinner/commit/8b1ee94d08e24083e194a31ba05c95a2f8061453))
- Install Nushell via prebuilt hustcer/setup-nu instead of building from source ([`7e534dd`](https://github.com/sorinirimies/tui-spinner/commit/7e534dd3e554b96955c999cb83ce22282ebb2ba7))

## [0.3.0](https://github.com/sorinirimies/tui-spinner/compare/v0.2.3...v0.3.0) — 2026-07-10

### ✨ Features

- **spinner:** Add to_lines()/to_text() to embed spinners in other widgets ([`a35220b`](https://github.com/sorinirimies/tui-spinner/commit/a35220b1bcadce49b82ad8138f6b79950d8e438d))
- Add Squeeze, Radiate motions, thickness API, and per-spinner labels ([`1ab2aaf`](https://github.com/sorinirimies/tui-spinner/commit/1ab2aaf7a92e7abf0856c37e43de4a43ea97362d))
- Vertical page uses same 4x4 cell grid as horizontal; regenerate all GIFs ([`58dc647`](https://github.com/sorinirimies/tui-spinner/commit/58dc647564474b77aef9cd82f7fc29233ad0aa38))

### 🐛 Bug fixes

- Merge identical match arms to satisfy clippy::match_same_arms ([`37fb825`](https://github.com/sorinirimies/tui-spinner/commit/37fb825d49a03a57e039fcc1e0563e7572774a87))
- Upgrade ratatui to 0.30, add downgrade rejection to dep-update workflows ([`6e469ec`](https://github.com/sorinirimies/tui-spinner/commit/6e469ecf364de2bbf92d49b470c2621dfbd2a498))

### 📦 Dependencies

- Upgrade ratatui to 0.30 and refresh Cargo.lock ([`08dff42`](https://github.com/sorinirimies/tui-spinner/commit/08dff4234c49047829e8a4450f4dac003b9c777c))
- Nightly dependency upgrade 2026-04-30 ([`711d6b3`](https://github.com/sorinirimies/tui-spinner/commit/711d6b32eaaa8be7a63d7ab9979369050b8dc7df))

### 🔧 Chores

- Use compatible-only cargo upgrade to prevent downgrades ([`36ee44b`](https://github.com/sorinirimies/tui-spinner/commit/36ee44b5f633f083de5ddad6f3919764c783e237))
- Restore all 3 remotes (origin, gitea, gitea_starscream) ([`b96460e`](https://github.com/sorinirimies/tui-spinner/commit/b96460ed8abaccc581ec9b7a9c549e0a756b99c5))
- Restore all 3 remotes (origin, gitea, gitea_starscream) ([`e43c6af`](https://github.com/sorinirimies/tui-spinner/commit/e43c6afcc431ad78e32d8b19d65aeae78ce955fc))
- Remove stale gitea remote references, keep only origin + gitea_starscream ([`2e0a959`](https://github.com/sorinirimies/tui-spinner/commit/2e0a9590572484e55fdc7b1a419e4038f13b3a39))

## [0.2.3](https://github.com/sorinirimies/tui-spinner/compare/v0.2.2...v0.2.3) — 2026-04-29

### 🐛 Bug fixes

- Bump_version.nu passes --tag to git-cliff so commits land under the release version ([`40a67d9`](https://github.com/sorinirimies/tui-spinner/commit/40a67d9a90bd1fe517057e27e8ada6038cd0ca04))

### 📚 Documentation

- Move combined overview GIF to first preview; fix BarSpinner heading ([`e78080d`](https://github.com/sorinirimies/tui-spinner/commit/e78080ded28b1d2b04a678a03dab56b8a1e2227f))
- Add crates.io downloads badge to README ([`696b147`](https://github.com/sorinirimies/tui-spinner/commit/696b147ea033c37fa3f540ad6631ee71f05014fc))

## [0.2.2](https://github.com/sorinirimies/tui-spinner/compare/v0.2.1...v0.2.2) — 2026-04-29

### 🐛 Bug fixes

- Header and footer same width as grid (8% margins); fix unicode escapes ([`d8d38fb`](https://github.com/sorinirimies/tui-spinner/commit/d8d38fba5a47e3c5bb91704c62db12065671e695))
- Add 8% horizontal margins so cells are ~21% wide instead of 25% ([`6e9ab3d`](https://github.com/sorinirimies/tui-spinner/commit/6e9ab3d840300a50ead7b5e89c7dfcdab94b016f))
- Bar_spinner — 4 individual rows with equal gaps, last row pushed down ([`4019e9d`](https://github.com/sorinirimies/tui-spinner/commit/4019e9da7a46687709f0164753eb3c739599a013))
- Bar_spinner pages 1+2 — two 2-row banks with 1-row gap ([`70d0010`](https://github.com/sorinirimies/tui-spinner/commit/70d0010708e5b1efba78e36003416bda0b02c318))
- Bar_spinner page 1 — 4×3 grid (was 3×2), 12 mixed styles ([`efc2686`](https://github.com/sorinirimies/tui-spinner/commit/efc268684e32547bff2bce34241d8ff1aa355302))
- Bar_spinner page 1 — tight 3×2 mixed grid; tape shows all 3 pages ([`d074f5f`](https://github.com/sorinirimies/tui-spinner/commit/d074f5f55f75faaa2bc9cc8136180ca442d8a602))

### ♻️ Refactor

- Collapse bar_spinner to single page — 16 styles, no duplicates ([`712e7e4`](https://github.com/sorinirimies/tui-spinner/commit/712e7e4431d478f5d05f7b0ccafebc1b167d3a10))
- Shared macros + BarSpinner::with_colors + new edge-case tests ([`ba296d1`](https://github.com/sorinirimies/tui-spinner/commit/ba296d1c15ab9dfc64806eb4258ee64b999383bb))

### 📚 Documentation

- Remove CI badge; add .zed/rules.md with agent best practices ([`d5c1893`](https://github.com/sorinirimies/tui-spinner/commit/d5c18931d1a08709e917d435b479a7b0a0d0fc71))
- Update README for 0.2.x ([`09337de`](https://github.com/sorinirimies/tui-spinner/commit/09337de22585e1e8acc65f54dfb06bf718244571))
- Regenerate bar-spinner-demo.gif and spinner-demo.gif ([`b4194e1`](https://github.com/sorinirimies/tui-spinner/commit/b4194e1d6256be0524d8022f67b6fee7f8edf1da))

### 📦 Dependencies

- Update ratatui 0.29 → 0.30; refresh Cargo.lock ([`6db490c`](https://github.com/sorinirimies/tui-spinner/commit/6db490cd80eff8871f741190a0b78b1d1438706d))
- Nightly dependency upgrade 2026-04-29 ([`2ad6ec5`](https://github.com/sorinirimies/tui-spinner/commit/2ad6ec535d54ec23381d11c18f2b68a524068fe6))

## [0.2.1](https://github.com/sorinirimies/tui-spinner/compare/v0.1.2...v0.2.1) — 2026-04-29

### ✨ Features

- Trio helper shows → ← ⟳ per style; BarSpinner row in spinner.rs ([`644b1c5`](https://github.com/sorinirimies/tui-spinner/commit/644b1c53f01026ec4bf35531d44b3a1cfea01f45))
- BarMotion::Loop — continuous sweep mode for BarSpinner ([`47d4ea6`](https://github.com/sorinirimies/tui-spinner/commit/47d4ea6c09d8c274022a2b902d3be94978ecc75d))
- 4 more BarStyles (Progress/Thick/Wave/Pip) from tui-slider symbols; 4x4 grid ([`1ed2c4b`](https://github.com/sorinirimies/tui-spinner/commit/1ed2c4b07e2cc42b1a53f2856ffec820b2a09641))
- BarStyle enum (Block/Shade/Dot/Diamond/Square); example sections ([`8b092c6`](https://github.com/sorinirimies/tui-spinner/commit/8b092c6bc68a00fc1ed7812441dcbbe9440672c7))

### 🐛 Bug fixes

- Bounce vs Loop shown as single CW bars so motion difference is obvious ([`f57485e`](https://github.com/sorinirimies/tui-spinner/commit/f57485e1f7f37163a3736ae73c68dbf7ca91e112))
- BarMotion::Loop — modular phase-in/out at both edges simultaneously ([`30e4424`](https://github.com/sorinirimies/tui-spinner/commit/30e442438e263dd09551e3a5d113323ec5dcd10c))
- Bar_spinner example — 1 CW + 1 CCW per concept, distinct patterns ([`75de44e`](https://github.com/sorinirimies/tui-spinner/commit/75de44e05291f685cb8278949591d4955377f629))

### 📦 Dependencies

- Update ratatui 0.29 → 0.30; update unicode-width ([`24ac5f6`](https://github.com/sorinirimies/tui-spinner/commit/24ac5f6de678f4d7ea6de7c9bb6aaeb49eba0020))
- Nightly dependency upgrade 2026-04-28 ([`cf814e0`](https://github.com/sorinirimies/tui-spinner/commit/cf814e054efc49069dac7417edc57d7828126689))

### 🔧 Chores

- Fix example warnings for 0.2.0 release ([`ec0d7e4`](https://github.com/sorinirimies/tui-spinner/commit/ec0d7e45a1f12405179e97daa065a92f9d007925))

## [0.1.2](https://github.com/sorinirimies/tui-spinner/compare/v0.1.1...v0.1.2) — 2026-04-28

### ✨ Features

- BarSpinner arc_char + presets; 3-page example with ← → navigation ([`826d693`](https://github.com/sorinirimies/tui-spinner/commit/826d693cbd2214e9910bf9d31966976f48f8db16))
- BarSpinner — BarTrack + fade_width; Hide compile in all VHS tapes ([`6ed2bb5`](https://github.com/sorinirimies/tui-spinner/commit/6ed2bb5163afe27852c70f4c6d463e4650be3324))

### 🐛 Bug fixes

- Open CHANGELOG.md with --raw in release_prepare.nu ([`b5e89d7`](https://github.com/sorinirimies/tui-spinner/commit/b5e89d7a13ec2de2ec1a64b2dedd5d74a396abd0))

## [0.1.1](https://github.com/sorinirimies/tui-spinner/releases/tag/v0.1.1) — 2026-04-28

### ✨ Features

- Split spinner.rs into 3 focused examples; add 5 FluxFrames presets; 4-row grid ([`139ba84`](https://github.com/sorinirimies/tui-spinner/commit/139ba84d2e76c5511aca95659cf1d022757eb3e7))
- 5×4 compact grid; add BAR/CORNERS presets + SHADE/MUSIC custom tiles ([`deeaf52`](https://github.com/sorinirimies/tui-spinner/commit/deeaf52cfa0d0b30a464f148436e15af58a73348))
- Replace ARROWS with BOUNCE/HALF/SQUARE/DICE; 4×4 grid with live custom tiles ([`ab7a9af`](https://github.com/sorinirimies/tui-spinner/commit/ab7a9afc54fbdf4d6fc12952e21fa49f9693737d))
- Add 5 new FluxFrames presets; rebuild example around preset/direction story ([`344bb15`](https://github.com/sorinirimies/tui-spinner/commit/344bb15e000dbf418be342fd004287c9adce34a2))
- FluxFrames presets + .frames() builder; reorganise example ([`aa1e4b9`](https://github.com/sorinirimies/tui-spinner/commit/aa1e4b90e8c833cdfbf409790eed020e571196c2))
- Add ZedSpinner; rename RectangularSpinner → BarSpinner ([`86b570b`](https://github.com/sorinirimies/tui-spinner/commit/86b570be73afea5adc3b4c25bcad445b9a291a15))
- Add RectangularSpinner — Zed/Claude-style bouncing braille arc ([`ed05a5e`](https://github.com/sorinirimies/tui-spinner/commit/ed05a5ecd19ddb7d7c21f97694840f0f381b5ed2))

### 🐛 Bug fixes

- Release-* recipes must call (bump version) not bare bump ([`4d622f6`](https://github.com/sorinirimies/tui-spinner/commit/4d622f6bd87c33aa68d9a7d79bf09f996b1ed3d3))
- Vertically centre circle_spinner rows; sync justfile with gitkraft ([`e06af4f`](https://github.com/sorinirimies/tui-spinner/commit/e06af4fb8aee1d223173c999c20ddabf998704d7))
- Replace row list with 4×3 grid in flux_spinner example ([`a737c34`](https://github.com/sorinirimies/tui-spinner/commit/a737c3460266b2ed0bd279adf6fca4319f6deffa))
- Redesign RectangularSpinner as solid bouncing bar (true Zed/Claude style) ([`05e67d0`](https://github.com/sorinirimies/tui-spinner/commit/05e67d0c215b71037194e3d9bf84591720499600))
- Resolve all clippy warnings; add gitea/gitea_starscream remotes; update nightly dep workflow ([`97d70fe`](https://github.com/sorinirimies/tui-spinner/commit/97d70fea6da164f2e305ffb47b46b6c782569027))

### ♻️ Refactor

- Clean example — macros, unified Tile, no glyph row, Length(4) cells ([`690a1b2`](https://github.com/sorinirimies/tui-spinner/commit/690a1b289d4ede2cf1d17fa37bd285520c7222dc))
- Rename DotSpinner -> FluxSpinner; simplify example ([`2fc9b3e`](https://github.com/sorinirimies/tui-spinner/commit/2fc9b3ea4da2d5d88931859f164a00967f117a63))
- Rename ZedSpinner -> DotSpinner; add Spin direction ([`0081382`](https://github.com/sorinirimies/tui-spinner/commit/0081382b72a5c6f67781f3706a2f312a8aada938))

### 📚 Documentation

- Add VHS tapes + GIFs for all 5 examples; rewrite README with previews ([`52b32a0`](https://github.com/sorinirimies/tui-spinner/commit/52b32a0be99699f70e3153e8b8151a1a89c8535b))

### 🔧 Chores

- Remove stray disktest.txt ([`e2491a4`](https://github.com/sorinirimies/tui-spinner/commit/e2491a4b18ffa44200f377e762a4f5e53a38f492))


