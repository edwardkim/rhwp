"""바이너리 탐색 계약 — 순서가 계약이다.

``RHWP_BIN`` → 패키지 동봉 → ``PATH``. 이 순서가 뒤집히면 개발자가 로컬 빌드를
가리켜도 동봉본이 실행돼 "왜 수정이 반영 안 되지"라는 진단 불가 상황이 생긴다.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

from rhwp import _binary
from rhwp.errors import BinaryNotFoundError


def _make_executable(path: Path) -> Path:
    """플랫폼에 맞는 '실행 가능한' 더미 파일을 만든다."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    if sys.platform != "win32":
        path.chmod(0o755)
    return path


def test_env_var_takes_priority(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    target = _make_executable(tmp_path / _binary.binary_name())
    monkeypatch.setenv(_binary.ENV_VAR, str(target))
    _binary.clear_cache()
    assert _binary.find_binary() == target.resolve()


def test_env_var_accepts_directory(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """디렉터리를 줘도 그 안의 실행 파일을 찾는다 — 흔한 사용 실수 흡수."""
    target = _make_executable(tmp_path / "bin" / _binary.binary_name())
    monkeypatch.setenv(_binary.ENV_VAR, str(target.parent))
    _binary.clear_cache()
    assert _binary.find_binary() == target.resolve()


def test_broken_env_var_fails_loudly(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """환경변수를 줬는데 못 쓰면 조용히 다음 경로로 넘어가면 안 된다.

    사용자는 그 바이너리를 쓰고 있다고 믿는데 다른 게 실행되면 디버깅 불가다.
    """
    monkeypatch.setenv(_binary.ENV_VAR, str(tmp_path / "없는파일"))
    _binary.clear_cache()
    with pytest.raises(BinaryNotFoundError) as caught:
        _binary.find_binary()
    assert _binary.ENV_VAR in str(caught.value)


def test_directory_is_not_mistaken_for_binary(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """이름만 같은 디렉터리를 실행 파일로 착각하면 안 된다."""
    fake_dir = tmp_path / _binary.binary_name()
    fake_dir.mkdir()
    monkeypatch.setenv(_binary.ENV_VAR, str(tmp_path))
    _binary.clear_cache()
    with pytest.raises(BinaryNotFoundError):
        _binary.find_binary()


def test_not_found_message_lists_every_attempt(monkeypatch: pytest.MonkeyPatch) -> None:
    """'없다'만 알려주면 사용자가 어디에 둬야 할지 모른다."""
    monkeypatch.delenv(_binary.ENV_VAR, raising=False)
    monkeypatch.setattr(_binary, "_from_bundle", lambda: None)
    monkeypatch.setattr(_binary, "_from_path", lambda: None)
    _binary.clear_cache()
    with pytest.raises(BinaryNotFoundError) as caught:
        _binary.find_binary()
    message = str(caught.value)
    assert _binary.ENV_VAR in message
    assert "동봉" in message
    assert "PATH" in message


def test_cache_avoids_rescanning(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    target = _make_executable(tmp_path / _binary.binary_name())
    monkeypatch.setenv(_binary.ENV_VAR, str(target))
    _binary.clear_cache()
    first = _binary.find_binary()
    # 환경변수를 지워도 캐시가 살아 있어야 한다.
    monkeypatch.delenv(_binary.ENV_VAR, raising=False)
    assert _binary.find_binary() == first


def test_refresh_bypasses_cache(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    first = _make_executable(tmp_path / "a" / _binary.binary_name())
    second = _make_executable(tmp_path / "b" / _binary.binary_name())
    monkeypatch.setenv(_binary.ENV_VAR, str(first))
    _binary.clear_cache()
    assert _binary.find_binary() == first.resolve()
    monkeypatch.setenv(_binary.ENV_VAR, str(second))
    assert _binary.find_binary(refresh=True) == second.resolve()


def test_binary_name_matches_platform() -> None:
    expected = "rhwp.exe" if sys.platform == "win32" else "rhwp"
    assert _binary.binary_name() == expected


def test_env_var_is_in_module_all() -> None:
    """[트랙 G R61 D-15] ENV_VAR 가 __all__ 에 없었다(import 는 됐지만 메타데이터 누락)."""
    assert "ENV_VAR" in _binary.__all__


def test_binary_name_and_bundled_dir_exported_at_package_root() -> None:
    """[트랙 G R61 D-15] binary_name/BUNDLED_DIR 가 패키지 루트에 노출되지 않았다."""
    import rhwp

    assert rhwp.binary_name is _binary.binary_name
    assert rhwp.BUNDLED_DIR == _binary.BUNDLED_DIR
