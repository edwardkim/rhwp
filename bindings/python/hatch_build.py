"""[#4336 R63] 휠에 rhwp 바이너리를 동봉하는 hatchling 빌드 훅.

동작 계약:

- 환경변수 두 개가 **함께** 있을 때만 개입한다.
  - ``RHWP_WHEEL_BUNDLE`` — 동봉할 실행 파일 경로 (``rhwp`` 또는 ``rhwp.exe``)
  - ``RHWP_WHEEL_TAG`` — 휠 플랫폼 태그 (예: ``py3-none-win_amd64``)
- 개입 시: 바이너리를 ``rhwp/_bin/`` 로 강제 포함하고 휠 태그를 플랫폼 태그로
  바꾼다. 런타임 탐색(`src/rhwp/_binary.py`)의 2순위 "패키지 동봉"이 이 자리를
  이미 계약(`bindings_foundation.md` §3)으로 보고 있으므로 **런타임 코드는
  무변경**이다.
- 환경변수가 없으면 아무것도 하지 않는다 — 로컬 `pip install -e` 와 sdist 는
  지금과 완전히 동일한 순수 패키지다.
- 하나만 있으면 **즉시 실패**한다. 태그 없는 동봉(잘못된 플랫폼에 설치됨)도,
  동봉 없는 플랫폼 태그(빈 약속)도 소비자를 속이는 휠이 된다.
"""

from __future__ import annotations

import os
from pathlib import Path

from hatchling.builders.hooks.plugin.interface import BuildHookInterface

BUNDLE_ENV = "RHWP_WHEEL_BUNDLE"
TAG_ENV = "RHWP_WHEEL_TAG"
_ALLOWED_NAMES = ("rhwp", "rhwp.exe")


class CustomBuildHook(BuildHookInterface):
    """휠 대상에서만 동작하는 바이너리 동봉 훅."""

    def initialize(self, version: str, build_data: dict) -> None:  # noqa: ARG002
        if self.target_name != "wheel":
            return

        bundle = os.environ.get(BUNDLE_ENV, "").strip()
        tag = os.environ.get(TAG_ENV, "").strip()
        if not bundle and not tag:
            return  # 순수 휠 — 종전과 동일.
        if not (bundle and tag):
            raise RuntimeError(
                f"{BUNDLE_ENV} 와 {TAG_ENV} 는 함께 설정해야 합니다 "
                f"(현재: BUNDLE={'설정' if bundle else '미설정'}, "
                f"TAG={'설정' if tag else '미설정'}). 태그 없는 동봉도, 동봉 없는 "
                f"플랫폼 태그도 소비자를 속이는 휠이 됩니다."
            )

        path = Path(bundle).expanduser().resolve()
        if not path.is_file():
            raise RuntimeError(f"{BUNDLE_ENV} 가 가리키는 파일이 없습니다: {path}")
        if path.name not in _ALLOWED_NAMES:
            raise RuntimeError(
                f"동봉 파일 이름은 {_ALLOWED_NAMES} 중 하나여야 합니다: {path.name} "
                f"(런타임 탐색 `_binary.binary_name()` 과 일치해야 발견됩니다)"
            )

        build_data["force_include"][str(path)] = f"rhwp/_bin/{path.name}"
        build_data["tag"] = tag
        build_data["pure_python"] = False
