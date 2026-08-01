"""테스트 공용 픽스처.

설계: **단위 테스트는 바이너리 없이 돌아야 한다.** 탐색·변환·예외 매핑·계획
직렬화는 순수 로직이므로 rhwp 빌드 없이 검증 가능하고, 그래야 CI 가 빠르다.
실제 문서를 만지는 통합 테스트만 ``@pytest.mark.integration`` 으로 격리한다.
"""

from __future__ import annotations

import os
import shutil
import sys
from pathlib import Path
from typing import Iterator, Optional

import pytest

# src 레이아웃 — 설치 없이 테스트할 수 있게 경로를 추가한다.
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

import rhwp  # noqa: E402
from rhwp import _binary  # noqa: E402

#: 저장소 루트 (bindings/python/tests → 3단계 위).
REPO_ROOT = Path(__file__).resolve().parents[3]


def _locate_binary() -> Optional[Path]:
    """테스트용 rhwp 를 찾는다: 환경변수 → 저장소 빌드 산출물 → PATH."""
    env = os.environ.get("RHWP_BIN", "").strip()
    if env:
        candidate = Path(env)
        if candidate.is_file():
            return candidate
    name = "rhwp.exe" if sys.platform == "win32" else "rhwp"
    for profile in ("release", "debug"):
        candidate = REPO_ROOT / "target" / profile / name
        if candidate.is_file():
            return candidate
    found = shutil.which(name)
    return Path(found) if found else None


@pytest.fixture(scope="session")
def binary_path() -> Path:
    """실제 rhwp 실행 파일. 없으면 통합 테스트를 건너뛴다."""
    found = _locate_binary()
    if found is None:
        pytest.skip("rhwp 바이너리를 찾지 못해 통합 테스트를 건너뜁니다")
    return found


@pytest.fixture(autouse=True)
def _isolate_binary_cache(monkeypatch: pytest.MonkeyPatch) -> Iterator[None]:
    """탐색 캐시를 테스트마다 비운다 — 앞 테스트의 환경변수가 새지 않게."""
    _binary.clear_cache()
    yield
    _binary.clear_cache()


@pytest.fixture
def wired_binary(binary_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """실제 바이너리를 ``RHWP_BIN`` 으로 물린다."""
    monkeypatch.setenv("RHWP_BIN", str(binary_path))
    _binary.clear_cache()
    return binary_path


@pytest.fixture(scope="session")
def sample_hwp() -> Path:
    """누름틀이 있는 실측 샘플."""
    candidate = REPO_ROOT / "samples" / "field-01.hwp"
    if not candidate.is_file():
        pytest.skip(f"샘플이 없습니다: {candidate}")
    return candidate


@pytest.fixture(scope="session")
def sample_hwpx_with_tables() -> Path:
    """표가 많은 실제 배포 정부 양식."""
    candidate = REPO_ROOT / "samples" / "2025년 기부·답례품 실적 지자체 보고서_양식.hwpx"
    if not candidate.is_file():
        pytest.skip(f"표 샘플이 없습니다: {candidate}")
    return candidate


@pytest.fixture
def fake_binary(tmp_path: Path) -> Path:
    """지정한 대로 행동하는 가짜 rhwp — 종료 코드·출력 계약을 단위로 검증할 때.

    파이썬 스크립트를 실행 파일처럼 흉내 낸다. 실제 문서 처리 없이 "exit 3 이면
    이렇게 동작해야 한다"를 검증할 수 있다.
    """
    name = "rhwp.exe" if sys.platform == "win32" else "rhwp"
    target = tmp_path / name
    if sys.platform == "win32":
        # 윈도우는 .exe 확장자를 요구하므로 배치 래퍼를 만든다.
        script = tmp_path / "fake_rhwp.py"
        script.write_text(_FAKE_SCRIPT, encoding="utf-8")
        target = tmp_path / "rhwp.cmd"
        target.write_text(f'@echo off\r\n"{sys.executable}" "{script}" %*\r\n', encoding="utf-8")
    else:
        target.write_text(f"#!{sys.executable}\n{_FAKE_SCRIPT}", encoding="utf-8")
        target.chmod(0o755)
    return target


_FAKE_SCRIPT = '''
import io, json, sys
# 실물(rhwp, Rust)은 콘솔 코드페이지와 무관하게 항상 UTF-8 바이트를 내보낸다.
# 픽스처도 같아야 인코딩 계약을 제대로 검증한다 — 플랫폼 기본값을 따르면
# 윈도우에서만 깨져서 "바인딩 버그"로 오인된다.
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", newline="\\n")
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", newline="\\n")
args = sys.argv[1:]
# 규약: 첫 인자가 시나리오 이름이다.
scenario = args[0] if args else "ok"
if scenario == "ok":
    print(json.dumps({"schemaVersion": "1.0", "ok": True}))
    sys.exit(0)
if scenario == "verdict":
    print(json.dumps({"schemaVersion": "1.0", "verify": {"identical": False, "diffCount": 3}}))
    sys.exit(3)
if scenario == "pages":
    print(json.dumps({"schemaVersion": "1.0", "pageCount": 2}))
    sys.exit(4)
if scenario == "usage":
    sys.stderr.write("오류: 알 수 없는 명령입니다\\n힌트: 가장 가까운 명령은 'export-svg' 입니다\\n")
    sys.exit(2)
if scenario == "runtime":
    sys.stderr.write("오류: 파일을 읽을 수 없습니다\\n")
    sys.exit(1)
if scenario == "garbage":
    print("이건 JSON 이 아니다")
    sys.exit(0)
if scenario == "empty":
    sys.exit(0)
if scenario == "ndjson":
    for i in range(3):
        print(json.dumps({"schemaVersion": "1.0", "source": f"f{i}.hwp", "pageCount": i + 1}))
    sys.exit(0)
if scenario == "ndjson-partial":
    print(json.dumps({"schemaVersion": "1.0", "source": "ok.hwp", "pageCount": 1}))
    print(json.dumps({"schemaVersion": "1.0", "source": "bad.hwp", "error": "읽기 실패"}))
    sys.exit(1)
if scenario == "unknown-exit":
    sys.exit(42)
sys.stderr.write(f"알 수 없는 시나리오: {scenario}\\n")
sys.exit(1)
'''
