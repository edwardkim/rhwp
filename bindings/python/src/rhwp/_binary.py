"""rhwp 실행 파일 탐색.

탐색 순서는 `mydocs/tech/bindings_foundation.md` §3 이 고정한 그대로다:

1. 환경변수 ``RHWP_BIN``
2. 패키지 동봉 (``rhwp/_bin/``)
3. ``PATH``

순서 자체가 계약이다 — 개발자가 로컬 빌드를 가리키고 싶을 때(1) 패키지 동봉본(2)이
가로채면 "왜 내 수정이 반영 안 되지"라는 진단 불가 상황이 생긴다.
"""

from __future__ import annotations

import os
import shutil
import stat
import sys
from pathlib import Path
from typing import List, Optional

from .errors import BinaryNotFoundError

__all__ = ["find_binary", "clear_cache", "binary_name", "BUNDLED_DIR"]

#: 환경변수 이름 — 문서 §3 고정.
ENV_VAR = "RHWP_BIN"

#: 패키지 동봉 바이너리 위치 (휠에 포함될 때).
BUNDLED_DIR = Path(__file__).resolve().parent / "_bin"

# 탐색은 프로세스 수명 동안 캐시한다 — 명령 하나마다 PATH 를 훑을 이유가 없다.
_cached: Optional[Path] = None


def binary_name() -> str:
    """플랫폼별 실행 파일 이름."""
    return "rhwp.exe" if sys.platform == "win32" else "rhwp"


def clear_cache() -> None:
    """탐색 캐시를 비운다 (테스트에서 환경변수를 바꿔 가며 검사할 때 필요)."""
    global _cached
    _cached = None


def _is_executable(path: Path) -> bool:
    """실행 가능한 **파일**인지. 디렉터리·심볼릭 깨짐·권한 없음을 모두 건다."""
    try:
        if not path.is_file():
            return False
    except OSError:
        # 경로가 너무 길거나 권한이 없어 stat 자체가 실패하는 경우.
        return False
    if sys.platform == "win32":
        # 윈도우는 실행 비트가 없다 — 확장자로 판단한다.
        return path.suffix.lower() in (".exe", ".bat", ".cmd")
    try:
        mode = path.stat().st_mode
    except OSError:
        return False
    return bool(mode & (stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH))


def _from_env() -> Optional[Path]:
    """``RHWP_BIN`` 이 가리키는 경로. 디렉터리를 줬으면 그 안의 실행 파일도 본다."""
    raw = os.environ.get(ENV_VAR, "").strip()
    if not raw:
        return None
    candidate = Path(raw).expanduser()
    if candidate.is_dir():
        candidate = candidate / binary_name()
    if _is_executable(candidate):
        return candidate.resolve()
    # 환경변수를 **줬는데 못 쓰는** 것은 조용히 넘길 일이 아니다. 사용자는 그걸
    # 쓰고 있다고 믿는데 다른 바이너리가 실행되면 디버깅이 불가능해진다.
    raise BinaryNotFoundError(
        f"{ENV_VAR} 가 가리키는 실행 파일을 쓸 수 없습니다: {raw}\n"
        f"  (존재하지 않거나, 파일이 아니거나, 실행 권한이 없습니다)"
    )


def _from_bundle() -> Optional[Path]:
    """패키지에 동봉된 바이너리."""
    candidate = BUNDLED_DIR / binary_name()
    return candidate.resolve() if _is_executable(candidate) else None


def _from_path() -> Optional[Path]:
    """``PATH`` 에서 찾기."""
    found = shutil.which(binary_name())
    return Path(found).resolve() if found else None


def find_binary(*, refresh: bool = False) -> Path:
    """rhwp 실행 파일 경로를 돌려준다.

    Args:
        refresh: 참이면 캐시를 무시하고 다시 탐색한다.

    Returns:
        실행 파일의 절대 경로.

    Raises:
        BinaryNotFoundError: 세 경로 모두에서 찾지 못했을 때. 메시지에 시도한
            위치를 전부 담는다 — "없다"만 알려주면 사용자가 어디에 둬야 할지
            모른다.
    """
    global _cached
    if _cached is not None and not refresh:
        return _cached

    tried: List[str] = []

    found = _from_env()  # 환경변수가 잘못되면 여기서 바로 예외.
    if found is not None:
        _cached = found
        return found
    tried.append(f"{ENV_VAR} (미설정)")

    found = _from_bundle()
    if found is not None:
        _cached = found
        return found
    tried.append(f"패키지 동봉 ({BUNDLED_DIR / binary_name()})")

    found = _from_path()
    if found is not None:
        _cached = found
        return found
    tried.append(f"PATH ({binary_name()} 없음)")

    raise BinaryNotFoundError(
        "rhwp 실행 파일을 찾지 못했습니다. 다음 순서로 탐색했습니다:\n"
        + "\n".join(f"  {i}. {t}" for i, t in enumerate(tried, 1))
        + f"\n\n해결: rhwp 를 설치해 PATH 에 두거나, {ENV_VAR} 로 경로를 지정하세요."
    )
