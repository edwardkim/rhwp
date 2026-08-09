"""[R70 절반] 생성 모델 드리프트 게이트 — ``gen_models --check`` 를 스위트에 편입.

기존 ``test_generated_models_match_current_schema`` 는 버전·타입명 수준만 대조해
**같은 버전 안에서 필드가 어긋나는 드리프트**를 놓친다. 생성기 자신의 ``--check``
가 바이트 단위 대조를 이미 구현하고 있으므로(단일 출처 — 여기서 대조를 다시
만들지 않는다), 이 테스트는 그것을 subprocess 로 불러 스위트에 편입한다.
``pytest tests/ -q`` 가 도는 기존 python-binding 통합 잡에 자동으로 실리므로
워크플로 파일은 바꾸지 않는다.

Node 쪽 같은 축은 이미 CI 에 있다(``npm run gen:check``) — 이 파일이 파이썬
절반을 맞춘다. R70 의 나머지 절반(런타임 적합성 스위트, R68 선행)은 다루지
않는다.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

BINDING_ROOT = Path(__file__).resolve().parents[1]


def test_generated_ir_models_are_current(wired_binary: Path) -> None:
    """``src/rhwp/ir.py`` 가 현재 IR 스키마에서 재생성한 결과와 바이트 일치한다."""
    result = subprocess.run(
        [
            sys.executable,
            str(BINDING_ROOT / "tools" / "gen_models.py"),
            "-o",
            str(BINDING_ROOT / "src" / "rhwp" / "ir.py"),
            "--check",
        ],
        capture_output=True,
        text=True,
        cwd=str(BINDING_ROOT),
    )
    assert result.returncode == 0, (
        "생성 모델이 IR 스키마와 다릅니다 — "
        "python tools/gen_models.py -o src/rhwp/ir.py 로 재생성해 커밋하세요.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )
