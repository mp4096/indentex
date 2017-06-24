class IndentexBin < Formula
  version '0.4.0'
  desc "A transpiler for an indentation-based superset of LaTeX."
  homepage "https://github.com/mp4096/indentex"
  url "https://github.com/mp4096/indentex/releases/download/#{version}/indentex_#{version}_x86_64-apple-darwin.tar.gz"
  sha256 "TODO"

  def install
    bin.install "indentex"
  end
end
