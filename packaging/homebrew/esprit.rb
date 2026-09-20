class Esprit < Formula
  desc "AI workspace and operating layer — fully offline, no Ollama required"
  homepage "https://github.com/krtvyasingh/Esprit"
  version "0.1.0"
  license "MIT"

  # Bottles are pre-compiled by CI — no Rust, CMake, or C++ needed.
  on_macos do
    on_arm do
      url "https://github.com/krtvyasingh/Esprit/releases/download/v0.1.0/esprit-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_MACOS"
    end
    on_intel do
      url "https://github.com/krtvyasingh/Esprit/releases/download/v0.1.0/esprit-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_MACOS"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/krtvyasingh/Esprit/releases/download/v0.1.0/esprit-aarch64-unknown-linux-musl.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_LINUX"
    end
    on_intel do
      url "https://github.com/krtvyasingh/Esprit/releases/download/v0.1.0/esprit-x86_64-unknown-linux-musl.tar.gz"
      sha256 "PLACEHOLDER_X86_64_LINUX"
    end
  end

  def install
    bin.install "esprit"
  end

  def post_install
    ohai "Esprit installed! 🚀"
    ohai ""
    ohai "Download the default AI model (one-time, ~390 MB):"
    ohai "  esprit init"
    ohai ""
    ohai "Then use it:"
    ohai "  esprit ask \"what does this project do?\""
    ohai "  esprit doctor"
    ohai "  esprit --help"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/esprit --version")
  end
end
