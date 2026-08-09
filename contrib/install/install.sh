#!/usr/bin/env bash
# [#4375] rhwp Linux/macOS 설치 스크립트 — 릴리스 tar.gz 를 받아 검증·배치한다.
#
#   curl -fsSL https://raw.githubusercontent.com/edwardkim/rhwp/devel/contrib/install/install.sh | bash
#   # 또는: ./install.sh v0.8.2 ~/.local/bin
#
# ① 버전 해석(latest→GitHub API) ② tar.gz+SHA256SUMS 다운로드 ③ sha256sum -c
# ④ 해체(rhwp/rhwp) ⑤ 설치 디렉터리 배치(기본 ~/.local/bin, PATH 는 안내만).
set -euo pipefail

REPO="edwardkim/rhwp"
VERSION="${1:-latest}"
INSTALL_DIR="${2:-$HOME/.local/bin}"

if [[ "$VERSION" == "latest" ]]; then
    VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep -m1 '"tag_name"' | cut -d '"' -f 4)
fi

case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)  SUFFIX="linux-x86_64"  ;;
    Darwin-x86_64) SUFFIX="macos-x86_64"  ;;
    Darwin-arm64)  SUFFIX="macos-aarch64" ;;
    *) echo "지원하지 않는 플랫폼: $(uname -s)-$(uname -m)" >&2; exit 1 ;;
esac
ASSET="rhwp-$VERSION-$SUFFIX.tar.gz"
BASE="https://github.com/$REPO/releases/download/$VERSION"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
echo "다운로드: $BASE/$ASSET"
curl -fsSL -o "$TMP/$ASSET" "$BASE/$ASSET"
curl -fsSL -o "$TMP/SHA256SUMS.txt" "$BASE/SHA256SUMS.txt"
( cd "$TMP" && sha256sum -c SHA256SUMS.txt --ignore-missing )
echo "무결성 확인: SHA-256 일치"

tar -xzf "$TMP/$ASSET" -C "$TMP"
mkdir -p "$INSTALL_DIR"
install -m 755 "$TMP/rhwp/rhwp" "$INSTALL_DIR/rhwp"

"$INSTALL_DIR/rhwp" --version
echo "설치 완료: $INSTALL_DIR/rhwp — 첫 확인은 'rhwp capabilities'"
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "주의: $INSTALL_DIR 이 PATH 에 없습니다 — 셸 프로필에 추가하세요." ;;
esac
