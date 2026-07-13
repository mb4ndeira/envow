class Envolve < Formula
  desc "Env schema validator and .env.example generator"
  homepage "https://github.com/mb4ndeira/envolve"
  url "https://github.com/mb4ndeira/envolve/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "" # update after release: curl -sL <url> | shasum -a 256
  license "MIT"
  head "https://github.com/mb4ndeira/envolve.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/envolve", "--version"
  end
end
