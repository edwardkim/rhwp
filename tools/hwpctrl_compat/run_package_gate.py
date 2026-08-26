"""공개 npm package gate의 플랫폼별 안전 옵션을 고른다.

Windows Hancom COM은 ``Quit()`` 뒤에도 ``Hwp.exe``가 잠시 남을 수 있다. live gate는 각
시나리오가 깨끗한 프로세스에서 시작해야 하므로, Windows package gate만 새로 생긴 PID를
명시적으로 정리한다. 시작 전에 이미 있던 한글 PID는 ``run_gate.py``가 ``OCCUPIED``로
거부하며 이 래퍼는 종료하지 않는다.
"""

from __future__ import annotations

import platform
import subprocess
import sys
from pathlib import Path


HERE = Path(__file__).resolve().parent


def gate_command(system_name: str) -> list[str]:
    command = [
        sys.executable,
        str(HERE / "run_gate.py"),
        "--impl",
        "npm/hwpctrl-ocx/src/index.mjs",
    ]
    if system_name.lower().startswith("win"):
        command.append("--cleanup-spawned")
    return command


def main() -> int:
    return subprocess.run(gate_command(platform.system()), check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
