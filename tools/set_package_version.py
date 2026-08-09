#!/usr/bin/env python3
"""[#4336 R63] 바인딩 패키지 버전을 릴리스 태그로 정렬한다.

version_policy.md §6: 태그 `vX.Y.Z` = `CARGO_PKG_VERSION` = 바인딩 패키지 버전.
릴리스 파이프라인이 빌드 직전에 호출한다 — 저장소에는 커밋하지 않는 일시 정렬이다
(저장소의 바인딩 버전은 개발 트리 표지일 뿐, 배포 버전의 원천은 태그다).

사용법::

    python tools/set_package_version.py 0.8.3            # python + node 모두
    python tools/set_package_version.py 0.8.3 --check    # Cargo.toml 과 일치 검증만

``--check`` 는 Cargo.toml 의 version 과 인자가 다르면 exit 1 — 태그를 잘못 찍은
릴리스가 패키지로 번지기 전에 여기서 멈춘다.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PYPROJECT = ROOT / "bindings" / "python" / "pyproject.toml"
NODE_PKG = ROOT / "bindings" / "node" / "package.json"
CARGO = ROOT / "Cargo.toml"

SEMVER = re.compile(r"^\d+\.\d+\.\d+(?:[-.+][0-9A-Za-z.-]+)?$")


def cargo_version() -> str:
    m = re.search(r'(?m)^version = "([^"]+)"$', CARGO.read_text(encoding="utf-8"))
    if not m:
        sys.exit("Cargo.toml 에서 version 을 찾지 못했습니다")
    return m.group(1)


def set_pyproject(version: str) -> None:
    text = PYPROJECT.read_text(encoding="utf-8")
    new, n = re.subn(r'(?m)^version = "[^"]+"$', f'version = "{version}"', text, count=1)
    if n != 1:
        sys.exit("pyproject.toml 의 version 줄을 찾지 못했습니다")
    PYPROJECT.write_text(new, encoding="utf-8")


def set_node(version: str) -> None:
    data = json.loads(NODE_PKG.read_text(encoding="utf-8"))
    data["version"] = version
    NODE_PKG.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("version", help="릴리스 버전 (태그에서 v 를 뗀 값)")
    ap.add_argument("--check", action="store_true", help="Cargo.toml 일치 검증만 수행")
    args = ap.parse_args()

    if not SEMVER.match(args.version):
        sys.exit(f"semver 형식이 아닙니다: {args.version}")

    cargo = cargo_version()
    if args.version != cargo:
        sys.exit(
            f"버전 불일치: 인자 {args.version} != Cargo.toml {cargo} — "
            f"태그와 Cargo.toml 이 갈린 릴리스는 여기서 멈춥니다 (version_policy.md §6)"
        )
    if args.check:
        print(f"일치: {cargo}")
        return

    set_pyproject(args.version)
    set_node(args.version)
    print(f"정렬 완료: pyproject.toml·package.json → {args.version}")


if __name__ == "__main__":
    main()
