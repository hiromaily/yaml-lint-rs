---
name: yaml-lint-release
description: Prepare, publish, or finish a version release of hiromaily/yaml-lint-rs on GitHub, Homebrew, and crates.io. Use for release version bumps and repairs to an existing release. Skip ordinary PRs and consumer-only upgrades.
---

# Release yaml-lint-rs

Work from the repository root. Read `make/release.mk`, `.github/workflows/release.yml`, and the installation section of `README.md` to confirm the current release flow. A version bump or release preparation request ends after the requested preparation and checks. A publication request includes monitoring and verification of the requested distribution channels.

## Prepare the version

1. Check the working tree, remote default branch, tags, and GitHub releases. Confirm that any requested PRs are included in the intended release commit. Use the requested version or bump level. For a routine fix without a specified version, choose the next patch after checking whether an existing version is already prepared or partially published.
2. Set the version with `make set-version V=X.Y.Z` if cargo-edit is available. Verify the root workspace version, the `yaml-lint-core` dependency version in `cli/Cargo.toml`, and both workspace package versions in `Cargo.lock`. If editing manually, update the manifests and refresh the lockfile with Cargo while preserving unrelated dependency versions.
3. Add the dated version section to `CHANGELOG.md` from the changes included since the previous tag. Leave future plans under Unreleased. Run `make release-prepare` and `cargo metadata --locked --format-version 1`. Keep unrelated workspace edits out of the release. Commit and land the preparation on the intended branch using the repository's normal workflow when publication is requested.

## Publish GitHub and Homebrew

1. Verify the committed version and changelog at the intended remote commit. Check both local and remote tag targets, peeling annotated tags to their commits. Create an annotated `vX.Y.Z` tag at that commit and push only that tag to `origin`. Reuse a matching tag and resume from the existing state. If a tag points elsewhere, report the conflict and leave it in place.
2. Find the tag's run with `gh run list -R hiromaily/yaml-lint-rs --workflow release.yml --branch vX.Y.Z`, confirm its commit, and monitor through completion. Verify the GitHub release contains `checksums.txt` and all four `yaml-lint-<target>.tar.gz` assets from the workflow matrix. Download them into a temporary directory and check their SHA-256 values against the manifest.
3. Verify the version, asset URLs, and all four checksums in `Formula/yaml-lint.rb` on the default branch of `hiromaily/homebrew-tap`. If the formula is behind, inspect the Homebrew job before retrying. Check the current release and formula immediately before any retry that can write to the tap. If the version being repaired is still the intended current release and the workflow cannot update the tap, use its formula template and the verified asset checksums for an authorized scoped repair. Use a guarded update against the current file or branch revision and reread both repositories after a conflict. Leave newer formula versions in place.

## Publish crates.io

The tag workflow currently handles GitHub and Homebrew. For crates.io publication, check the exact target versions of both crates and work from the release commit. Publish missing versions with `cargo publish --locked -p yaml-lint-core` first. Wait until that version is available to Cargo before running `cargo publish --locked -p yaml-lint`. Recheck registry state before retrying an ambiguous failure. For a preparation-only request, account for the CLI package verification depending on the core version being available in the registry.

Report the release commit and tag, the verified publication links, and any channel still pending with its specific blocker.
