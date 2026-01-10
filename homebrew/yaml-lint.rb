# typed: false
# frozen_string_literal: true

# Homebrew formula for yaml-lint
# Repository: https://github.com/hiromaily/homebrew-tap
class YamlLint < Formula
  desc "A fast YAML linter written in Rust"
  homepage "https://github.com/hiromaily/yaml-lint-rs"
  version "0.2.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/hiromaily/yaml-lint-rs/releases/download/v#{version}/yaml-lint-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_X86_64_DARWIN"
    end

    on_arm do
      url "https://github.com/hiromaily/yaml-lint-rs/releases/download/v#{version}/yaml-lint-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_AARCH64_DARWIN"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/hiromaily/yaml-lint-rs/releases/download/v#{version}/yaml-lint-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_X86_64_LINUX"
    end

    on_arm do
      url "https://github.com/hiromaily/yaml-lint-rs/releases/download/v#{version}/yaml-lint-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_AARCH64_LINUX"
    end
  end

  def install
    bin.install "yaml-lint"
  end

  test do
    # Create a test YAML file
    (testpath/"test.yaml").write <<~EOS
      key: value
      list:
        - item1
        - item2
    EOS

    # Run yaml-lint and check it exits successfully
    system "#{bin}/yaml-lint", testpath/"test.yaml"
  end
end
