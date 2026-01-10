# Homebrew Formula for yaml-lint

This directory contains the Homebrew formula template for yaml-lint.

## Setup Instructions

### 1. Create Homebrew Tap Repository

Create a new GitHub repository named `homebrew-tap`:

```bash
# On GitHub, create: https://github.com/hiromaily/homebrew-tap
```

### 2. Add Formula to Tap

Copy `yaml-lint.rb` to your tap repository:

```bash
# Clone your tap repository
git clone https://github.com/hiromaily/homebrew-tap.git
cd homebrew-tap

# Create Formula directory
mkdir -p Formula

# Copy formula (update SHA256 hashes first!)
cp /path/to/yaml-lint-rs/homebrew/yaml-lint.rb Formula/
```

### 3. Update SHA256 Hashes

After creating a release, calculate SHA256 for each binary:

```bash
# Download release assets and calculate SHA256
curl -sL https://github.com/hiromaily/yaml-lint-rs/releases/download/v0.1.0/yaml-lint-x86_64-apple-darwin.tar.gz | shasum -a 256
curl -sL https://github.com/hiromaily/yaml-lint-rs/releases/download/v0.1.0/yaml-lint-aarch64-apple-darwin.tar.gz | shasum -a 256
curl -sL https://github.com/hiromaily/yaml-lint-rs/releases/download/v0.1.0/yaml-lint-x86_64-unknown-linux-gnu.tar.gz | shasum -a 256
curl -sL https://github.com/hiromaily/yaml-lint-rs/releases/download/v0.1.0/yaml-lint-aarch64-unknown-linux-gnu.tar.gz | shasum -a 256
```

Replace `REPLACE_WITH_ACTUAL_SHA256_*` placeholders in the formula.

### 4. Create GitHub Token for Auto-Update (Optional)

To enable automatic formula updates on release:

1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Create a token with `repo` scope for `homebrew-tap` repository
3. Add it as a secret `HOMEBREW_TAP_TOKEN` in yaml-lint-rs repository

### 5. Installation

Users can then install with:

```bash
# Add tap
brew tap hiromaily/tap

# Install
brew install yaml-lint

# Or in one command
brew install hiromaily/tap/yaml-lint
```

### 6. Creating a Release

```bash
# Tag the release
git tag v0.1.0
git push origin v0.1.0

# GitHub Actions will automatically:
# 1. Build binaries for all platforms
# 2. Create GitHub release with assets
# 3. Update Homebrew formula (if HOMEBREW_TAP_TOKEN is set)
```

## Manual Formula Update

If automatic update fails, manually update the formula:

```bash
cd homebrew-tap

# Edit Formula/yaml-lint.rb
# - Update version
# - Update SHA256 hashes

git add Formula/yaml-lint.rb
git commit -m "Update yaml-lint to v0.1.0"
git push
```

## Testing the Formula

```bash
# Test locally before publishing
brew install --build-from-source ./Formula/yaml-lint.rb

# Or test with audit
brew audit --strict --new-formula Formula/yaml-lint.rb
```
