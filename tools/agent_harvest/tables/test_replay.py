#!/usr/bin/env python3
"""고정 표본 3개만 다시 연다 — 전 코퍼스를 CI 에서 돌리지 않는다."""
from pathlib import Path
import json
import subprocess
import sys

AGENT = Path(r"C:\Users\swsz9\rhwp-agent-cli-pack\target\debug\rhwp-agent.exe")
SAMPLES = Path(r"C:\Users\swsz9\rhwp-agent-cli-pack\samples")
HERE = Path(__file__).resolve().parent
FIX = [
    "form-01.hwp",
    "hwp3-sample.hwp",
    "hwp_table_test.hwp",
]


def main() -> int:
    if not AGENT.exists():
        print("skip: rhwp-agent 없음", AGENT)
        return 0
    goldens = HERE / "goldens"
    assert goldens.exists(), goldens
    files = list(goldens.glob("*.json"))
    assert files, "golden 없음"
    sample = json.loads(files[0].read_text(encoding="utf-8"))
    assert "runs" in sample or "source" in sample
    for name in FIX:
        path = SAMPLES / name
        if not path.exists():
            matches = list(SAMPLES.rglob(name))
            if not matches:
                continue
            path = matches[0]
        proc = subprocess.run(
            [str(AGENT), "info", "--json", str(path)],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=60,
        )
        assert proc.returncode == 0, (name, proc.stderr)
        env = json.loads(proc.stdout)
        assert env.get("schemaVersion") == "1.0"
        assert env.get("command") == "info"
        assert env.get("tool") == "rhwp-agent"
    print("ok", pack_name())
    return 0


def pack_name() -> str:
    return "tables"


if __name__ == "__main__":
    sys.exit(main())
