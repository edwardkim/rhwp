"""예제 스모크 — 문서에 실린 코드가 실제로 도는지.

문서의 예제가 깨진 채로 남는 것이 문서 없는 것보다 나쁘다. 사용자는 그걸 믿고
쓰기 때문이다. 여기서는 **인자 없이 실행했을 때의 사용법 출력**과 **실물 문서로
돌린 결과**를 둘 다 본다.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path
from typing import List

import pytest

EXAMPLES_DIR = Path(__file__).resolve().parents[1] / "examples"
SRC_DIR = Path(__file__).resolve().parents[1] / "src"


def _all_examples() -> List[Path]:
    return sorted(EXAMPLES_DIR.glob("*.py"))


def _run(script: Path, args: List[str], binary: Path | None = None) -> subprocess.CompletedProcess:
    env = {
        **dict(__import__("os").environ),
        "PYTHONPATH": str(SRC_DIR),
        "PYTHONIOENCODING": "utf-8",
    }
    if binary is not None:
        env["RHWP_BIN"] = str(binary)
    return subprocess.run(  # noqa: S603
        [sys.executable, str(script), *args],
        capture_output=True,
        encoding="utf-8",
        errors="replace",
        env=env,
        timeout=300,
    )


def test_examples_directory_is_not_empty() -> None:
    assert _all_examples(), "예제가 하나도 없다"


@pytest.mark.parametrize("script", _all_examples(), ids=lambda p: p.name)
def test_example_compiles(script: Path) -> None:
    """문법 오류가 있는 예제는 문서가 아니라 함정이다."""
    result = subprocess.run(  # noqa: S603
        [sys.executable, "-m", "py_compile", str(script)],
        capture_output=True,
        encoding="utf-8",
        errors="replace",
    )
    assert result.returncode == 0, f"{script.name} 컴파일 실패:\n{result.stderr}"


@pytest.mark.parametrize("script", _all_examples(), ids=lambda p: p.name)
def test_example_has_module_docstring_with_usage(script: Path) -> None:
    """사용법이 없으면 예제를 어떻게 돌리는지 알 수 없다."""
    source = script.read_text(encoding="utf-8")
    assert source.lstrip().startswith('#!/usr/bin/env python3'), f"{script.name}: shebang 없음"
    assert '"""' in source, f"{script.name}: docstring 없음"
    head = source.split('"""')[1]
    assert "python examples/" in head, f"{script.name}: docstring 에 실행법이 없음"


@pytest.mark.parametrize("script", _all_examples(), ids=lambda p: p.name)
def test_example_without_arguments_exits_usage(script: Path) -> None:
    """인자 없이 부르면 사용법 + exit 2 — 도구 사전과 같은 어휘를 쓴다."""
    result = _run(script, [])
    # 06 은 인자가 선택이라 정상 종료할 수 있다.
    if script.name.startswith("06_"):
        return
    assert result.returncode == 2, (
        f"{script.name}: 인자 없이 부르면 exit 2 여야 한다 "
        f"(받음 {result.returncode})\n{result.stdout}\n{result.stderr}"
    )


# ── 실물 실행 ───────────────────────────────────────────────────────────


@pytest.mark.integration
def test_read_document_example_runs(binary_path: Path, sample_hwp: Path) -> None:
    result = _run(EXAMPLES_DIR / "01_read_document.py", [str(sample_hwp)], binary_path)
    assert result.returncode == 0, f"{result.stdout}\n{result.stderr}"
    assert "포맷:" in result.stdout


@pytest.mark.integration
def test_fill_form_example_runs(binary_path: Path, sample_hwp: Path, tmp_path: Path) -> None:
    out = tmp_path / "예제채움.hwp"
    result = _run(
        EXAMPLES_DIR / "02_fill_form.py", [str(sample_hwp), str(out)], binary_path
    )
    # 0(성공) 또는 1(누름틀 없음) 또는 3(검증 실패) — 전부 정상 판정이다.
    assert result.returncode in (0, 1, 3), f"{result.stdout}\n{result.stderr}"


@pytest.mark.integration
def test_session_example_runs(binary_path: Path, sample_hwp: Path, tmp_path: Path) -> None:
    out = tmp_path / "예제세션.hwp"
    result = _run(
        EXAMPLES_DIR / "03_session_edit.py", [str(sample_hwp), str(out)], binary_path
    )
    assert result.returncode == 0, f"{result.stdout}\n{result.stderr}"
    assert "열림:" in result.stdout


@pytest.mark.integration
def test_ir_schema_example_lists_definitions(binary_path: Path) -> None:
    result = _run(EXAMPLES_DIR / "06_ir_schema.py", [], binary_path)
    assert result.returncode == 0, f"{result.stdout}\n{result.stderr}"
    assert "IR 스키마 v" in result.stdout
    assert "Document" in result.stdout


@pytest.mark.integration
def test_ir_schema_example_shows_one_type(binary_path: Path) -> None:
    result = _run(EXAMPLES_DIR / "06_ir_schema.py", ["Paragraph"], binary_path)
    assert result.returncode == 0, f"{result.stdout}\n{result.stderr}"
    assert "Paragraph" in result.stdout


@pytest.mark.integration
def test_ir_schema_example_rejects_unknown_type(binary_path: Path) -> None:
    result = _run(EXAMPLES_DIR / "06_ir_schema.py", ["없는타입XYZ"], binary_path)
    assert result.returncode == 2, f"{result.stdout}\n{result.stderr}"


@pytest.mark.integration
def test_batch_example_handles_folder(
    binary_path: Path, sample_hwp: Path, tmp_path: Path
) -> None:
    import shutil

    folder = tmp_path / "묶음"
    folder.mkdir()
    shutil.copy(sample_hwp, folder / "a.hwp")
    shutil.copy(sample_hwp, folder / "b.hwp")

    result = _run(EXAMPLES_DIR / "05_batch_pipeline.py", [str(folder)], binary_path)
    assert result.returncode in (0, 1), f"{result.stdout}\n{result.stderr}"
    assert "성공" in result.stdout


@pytest.mark.integration
def test_batch_example_reports_empty_folder(binary_path: Path, tmp_path: Path) -> None:
    empty = tmp_path / "빈폴더"
    empty.mkdir()
    result = _run(EXAMPLES_DIR / "05_batch_pipeline.py", [str(empty)], binary_path)
    assert result.returncode == 1
    assert "없습니다" in result.stdout
