#!/usr/bin/env python3
"""[#4338 R38] 설치 채널 매니페스트(scoop·homebrew·winget)의 버전·해시 일괄 갱신.

사용법::

    # 릴리스 자산의 SHA256SUMS.txt 를 받아서:
    #   gh release download vX.Y.Z -p SHA256SUMS.txt
    python tools/update_channel_manifests.py X.Y.Z SHA256SUMS.txt
    python tools/update_channel_manifests.py X.Y.Z SHA256SUMS.txt --check  # 멱등 검증

``--check`` 는 파일을 바꾸지 않고, 현재 커밋본이 주어진 버전·해시로 재생성한
결과와 다르면 exit 1 — 릴리스 후 매니페스트 갱신을 잊은 상태를 잡는다.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PKG = ROOT / "contrib" / "packaging"

ASSETS = {
    "linux": "rhwp-v{v}-linux-x86_64.tar.gz",
    "mac_arm": "rhwp-v{v}-macos-aarch64.tar.gz",
    "mac_x64": "rhwp-v{v}-macos-x86_64.tar.gz",
    "windows": "rhwp-v{v}-windows-x86_64.zip",
}


def parse_sums(path: Path, version: str) -> dict:
    """SHA256SUMS.txt → {키: 해시}. 네 자산이 전부 있어야 한다."""
    table = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        parts = line.split()
        if len(parts) == 2:
            table[parts[1]] = parts[0]
    out = {}
    for key, pattern in ASSETS.items():
        name = pattern.format(v=version)
        if name not in table:
            sys.exit(f"SHA256SUMS 에 {name} 이 없습니다 — 태그와 파일이 맞는지 확인하세요")
        out[key] = table[name]
    return out


def sub_all(text: str, pairs: list, path: str) -> str:
    for pattern, repl in pairs:
        text, n = re.subn(pattern, repl, text)
        if n == 0:
            sys.exit(f"{path}: 패턴 미발견 — 매니페스트 구조가 바뀌었으면 이 스크립트를 함께 고치세요\n  {pattern}")
    return text


def render(version: str, sums: dict) -> dict:
    """경로 → 갱신된 본문."""
    v = version
    out = {}

    scoop = PKG / "scoop" / "rhwp.json"
    out[scoop] = sub_all(
        scoop.read_text(encoding="utf-8"),
        [
            (r'"version": "[^"]+"', f'"version": "{v}"'),
            (r'download/v[0-9][^/]*/rhwp-v[0-9][^-]*-windows', f"download/v{v}/rhwp-v{v}-windows"),
            (r'"hash": "[0-9a-f]{64}"', f'"hash": "{sums["windows"]}"'),
        ],
        str(scoop),
    )

    brew = PKG / "homebrew" / "rhwp.rb"
    text = brew.read_text(encoding="utf-8")
    text = sub_all(
        text,
        [(r'version "[^"]+"', f'version "{v}"'),
         (r"download/v[0-9][^/]*/", f"download/v{v}/"),
         (r"rhwp-v[0-9][^-]*-", f"rhwp-v{v}-")],
        str(brew),
    )
    for key, marker in (("mac_arm", "macos-aarch64"), ("mac_x64", "macos-x86_64"), ("linux", "linux-x86_64")):
        # url 줄 다음의 sha256 줄을 자산별로 짝지어 치환한다.
        pattern = rf'({re.escape(marker)}\.tar\.gz"\s*\n\s*sha256 ")[0-9a-f]{{64}}'
        text, n = re.subn(pattern, rf"\g<1>{sums[key]}", text)
        if n != 1:
            sys.exit(f"{brew}: {marker} sha256 치환 실패 (매치 {n}건)")
    out[brew] = text

    for name in ("Edwardkim.rhwp.yaml", "Edwardkim.rhwp.installer.yaml", "Edwardkim.rhwp.locale.ko-KR.yaml"):
        p = PKG / "winget" / name
        pairs = [(r"PackageVersion: [0-9][^\s]*", f"PackageVersion: {v}")]
        if "installer" in name:
            pairs += [
                (r"download/v[0-9][^/]*/rhwp-v[0-9][^-]*-windows", f"download/v{v}/rhwp-v{v}-windows"),
                (r"InstallerSha256: [0-9A-F]{64}", f"InstallerSha256: {sums['windows'].upper()}"),
            ]
        out[p] = sub_all(p.read_text(encoding="utf-8"), pairs, str(p))

    return out


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("version", help="릴리스 버전 (v 없이)")
    ap.add_argument("sums", type=Path, help="해당 릴리스의 SHA256SUMS.txt 경로")
    ap.add_argument("--check", action="store_true", help="변경 없이 멱등 검증만")
    args = ap.parse_args()

    sums = parse_sums(args.sums, args.version)
    rendered = render(args.version, sums)

    drift = [str(p) for p, text in rendered.items() if p.read_text(encoding="utf-8") != text]
    if args.check:
        if drift:
            sys.exit("매니페스트가 낡았습니다 (재생성과 다름):\n  " + "\n  ".join(drift))
        print(f"멱등 확인: {len(rendered)}개 매니페스트가 v{args.version} 기준 최신")
        return

    for p, text in rendered.items():
        p.write_text(text, encoding="utf-8")
    print(f"갱신 완료: {len(rendered)}개 매니페스트 → v{args.version}" + (f" (변경 {len(drift)}건)" if drift else " (변경 없음)"))


if __name__ == "__main__":
    main()
