# [#4338 R38] Homebrew formula — 탭 위치는 메인테이너 결정 사항
# (mydocs/manual/channel_manifests_guide.md §3). 버전·sha256 갱신은
# tools/update_channel_manifests.py 가 수행한다.
class Rhwp < Formula
  desc "HWP/HWPX document engine — parse, edit, render, convert (CLI + MCP server)"
  homepage "https://github.com/edwardkim/rhwp"
  version "0.8.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/edwardkim/rhwp/releases/download/v0.8.2/rhwp-v0.8.2-macos-aarch64.tar.gz"
      sha256 "2833431bed6034a0af03f7d889f1a41603e61b4bda5e16c93d2fc58efee5b5ea"
    else
      url "https://github.com/edwardkim/rhwp/releases/download/v0.8.2/rhwp-v0.8.2-macos-x86_64.tar.gz"
      sha256 "7f53cb75dc3ff2a8c3d3178caaa0d3bffb396e7a768d215a747254e79471cbbd"
    end
  end

  on_linux do
    url "https://github.com/edwardkim/rhwp/releases/download/v0.8.2/rhwp-v0.8.2-linux-x86_64.tar.gz"
    sha256 "3225246533eca2b10ec2926228aee0d1cbf0ea6de0553e053ec8d6cb79fa9570"
  end

  def install
    bin.install "rhwp"
  end

  test do
    assert_match "rhwp", shell_output("#{bin}/rhwp --version")
  end
end
