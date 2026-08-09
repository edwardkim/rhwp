#!/usr/bin/env python3
"""[#4389] 하네스 성질 실검증 러너 — 주장마다 실행, 판정은 PASS/FAIL.

이 저장소의 에이전트 하네스가 문서로 주장하는 성질을 **제3자가 명령 하나로
직접 검증**한다. 폐쇄 런타임의 "믿어 달라"와 반대편에 서는 실물이다.

사용법::

    python tools/harness_proofs.py                 # 표 출력, 하나라도 FAIL 이면 exit 1
    python tools/harness_proofs.py --json          # 기계용 JSON
    RHWP_BIN=path/to/rhwp python tools/harness_proofs.py   # 바이너리 지정

바이너리 탐색: RHWP_BIN → target/release/rhwp → target/debug/rhwp → PATH.
검증 6종은 전부 devel 머지본만으로 돈다(미머지 성질은 스코어카드 문서가 계약
테스트·PR 링크로 안내한다 — 이 러너는 거짓 PASS 를 만들지 않는다).
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SAMPLE = ROOT / "samples" / "basic" / "issue2007_nested_cell_pagination_42065.hwp"


def find_binary() -> str:
    exe = ".exe" if os.name == "nt" else ""
    env = os.environ.get("RHWP_BIN", "").strip()
    candidates = [env] if env else []
    candidates += [
        str(ROOT / "target" / "release" / f"rhwp{exe}"),
        str(ROOT / "target" / "debug" / f"rhwp{exe}"),
    ]
    for c in candidates:
        if c and Path(c).is_file():
            return c
    which = shutil.which("rhwp")
    if which:
        return which
    sys.exit("rhwp 바이너리를 찾지 못했습니다 — RHWP_BIN 지정 또는 cargo build")


def run(bin_path: str, args: list, timeout: int = 120):
    return subprocess.run(
        [bin_path, *args], capture_output=True, timeout=timeout, cwd=ROOT
    )


def proofs(bin_path: str) -> list:
    results = []

    def record(pid: str, claim: str, command: str, ok: bool, detail: str) -> None:
        results.append(
            {"id": pid, "claim": claim, "command": command, "pass": bool(ok), "detail": detail}
        )

    # P1 결정론 — 같은 호출은 바이트까지 같다 (자기서술에 모델·시각이 끼지 않는다).
    a = run(bin_path, ["capabilities"])
    b = run(bin_path, ["capabilities"])
    record(
        "P1",
        "자기서술 결정론 — capabilities 2회 호출의 stdout 이 바이트 동일",
        "rhwp capabilities (×2 비교)",
        a.returncode == 0 and a.stdout == b.stdout and len(a.stdout) > 1000,
        f"exit={a.returncode}, bytes={len(a.stdout)}, identical={a.stdout == b.stdout}",
    )

    # P2 자기서술 규모 — 명령 표면이 기계 계약으로 전수 서술된다.
    try:
        caps = json.loads(a.stdout)
        n_cmd = len(caps.get("commands", []))
        ok = n_cmd >= 50 and "exitCodes" in caps and "jsonContract" in caps
        detail = f"commands={n_cmd}, exitCodes/jsonContract 존재={ok}"
    except Exception as e:  # noqa: BLE001 - 판정용 러너
        ok, detail = False, f"JSON 파싱 실패: {e}"
    record(
        "P2",
        "명령 표면 전수 자기서술 — capabilities 가 50+ 명령의 계약을 싣는다",
        "rhwp capabilities | jq '.commands|length'",
        ok,
        detail,
    )

    # P3 종료코드 사전 — 미지 옵션은 exit 2, stdout 은 0바이트(반쪽 JSON 금지).
    c = run(bin_path, ["info", str(SAMPLE), "--nope", "--json"])
    record(
        "P3",
        "사용법 오류 사전 — 미지 옵션은 exit 2 + stdout 0바이트",
        "rhwp info <sample> --nope --json",
        c.returncode == 2 and c.stdout == b"",
        f"exit={c.returncode}, stdout_bytes={len(c.stdout)}",
    )

    # P4 실패 stdout 순수성 — 런타임 실패도 stdout 을 오염시키지 않는다.
    d = run(bin_path, ["info", "no_such_file_hopefully.hwp", "--json"])
    record(
        "P4",
        "실패 경로 stdout 순수성 — 없는 파일 info 는 exit 1 + stdout 0바이트",
        "rhwp info no_such_file.hwp --json",
        d.returncode == 1 and d.stdout == b"",
        f"exit={d.returncode}, stdout_bytes={len(d.stdout)}",
    )

    # P5 출처 표지 — 문서 파생 값을 싣는 봉투는 신뢰 경계를 스스로 밝힌다.
    e = run(bin_path, ["info", str(SAMPLE), "--json"])
    try:
        env = json.loads(e.stdout)
        ok = "untrustedContent" in env and "untrustedFields" in env
        detail = f"untrustedContent={env.get('untrustedContent')!r}"
    except Exception as ex:  # noqa: BLE001
        ok, detail = False, f"JSON 파싱 실패: {ex}"
    record(
        "P5",
        "출처 표지 S1 — 봉투가 untrustedContent/untrustedFields 를 스스로 싣는다",
        "rhwp info <sample> --json",
        e.returncode == 0 and ok,
        detail,
    )

    # P6 설명 결정론 — explain 도 2회 동일(생성 문장 아님이 드리프트 가드의 전제).
    f1 = run(bin_path, ["explain", str(SAMPLE), "--json"])
    f2 = run(bin_path, ["explain", str(SAMPLE), "--json"])
    record(
        "P6",
        "explain 결정론 — 같은 문서, 같은 서술(바이트 동일)",
        "rhwp explain <sample> --json (×2 비교)",
        f1.returncode == 0 and f1.stdout == f2.stdout and len(f1.stdout) > 100,
        f"exit={f1.returncode}, identical={f1.stdout == f2.stdout}",
    )

    return results


def main() -> None:
    # Windows cp949 콘솔에서도 스스로 깨지지 않는다 (#4106 선례).
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(encoding="utf-8", errors="replace")
    as_json = "--json" in sys.argv
    bin_path = find_binary()
    results = proofs(bin_path)
    passed = sum(1 for r in results if r["pass"])
    if as_json:
        print(
            json.dumps(
                {"binary": bin_path, "passed": passed, "total": len(results), "proofs": results},
                ensure_ascii=False,
                indent=2,
            )
        )
    else:
        print(f"하네스 성질 실검증 — {bin_path}")
        for r in results:
            mark = "PASS" if r["pass"] else "FAIL"
            print(f"  [{mark}] {r['id']} {r['claim']}")
            print(f"         $ {r['command']}")
            print(f"         {r['detail']}")
        print(f"판정: {passed}/{len(results)}")
    sys.exit(0 if passed == len(results) else 1)


if __name__ == "__main__":
    main()
