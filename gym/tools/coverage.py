"""gym 능력 커버리지 측정 — 에이전트-대면 명령 중 몇 %가 gym 과제로 실측되나.

## 왜 이 도구인가

새 gym 과제를 만들기 전 "이 능력이 이미 커버돼 있나"를 재는 장치가 없어서, 이미
`fill-fields` 를 커버한 core-cli T07 위에 중복 pack(#4781)이 만들어졌다 자진 철회된
사고가 있었다. 이 도구는 그 재발을 막는다 — 만들기 전에 **진짜 빈 곳**만 잰다.

## 무엇을 재나 (정직한 분모)

capabilities 의 `category` 로 **에이전트-대면 명령**(`batch`·`edit`·`export`·`query`)
만 분모로 삼는다. `diagnostic`(hwp5-*·dump-* 개발 probe)·`internal`·`serve`(인프라)는
제외한다 — 진단 도구를 빈 곳으로 세면 커버리지가 실제보다 낮게 나와 오해를 부른다.

한 명령은 gym 과제·기준풀이의 `checks[].cmd[0]` 또는 `steps[].run[0]` 에 나타나면
'노출'로 친다.

## 사용

    python gym/tools/coverage.py --bin target/debug/rhwp   # 바이너리로 capabilities
    python gym/tools/coverage.py --capabilities cap.json    # 저장된 capabilities
    python gym/tools/coverage.py --bin target/debug/rhwp --json
"""

from __future__ import annotations

import argparse
import glob
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)

# 에이전트가 실제로 쓰는 능력 카테고리만 분모. 나머지는 개발·인프라 도구.
AGENT_CATEGORIES = {"batch", "edit", "export", "query"}
EXCLUDED_CATEGORIES = {"diagnostic", "internal", "serve"}


def used_commands(packs_root: str) -> set[str]:
    """gym 과제·기준풀이가 실제로 부르는 명령(첫 토큰) 집합."""
    used: set[str] = set()
    patterns = [
        os.path.join(packs_root, "packs", "*", "tasks", "*.json"),
        os.path.join(packs_root, "packs", "*", "reference", "*.json"),
    ]
    for pattern in patterns:
        for path in glob.glob(pattern):
            with open(path, encoding="utf-8") as fh:
                doc = json.load(fh)
            for check in doc.get("checks", []):
                cmd = check.get("cmd") or []
                if cmd:
                    used.add(cmd[0])
            for step in doc.get("steps", []):
                run = step.get("run") or []
                if run:
                    used.add(run[0])
    return used


def measure(commands: list[dict], used: set[str]) -> dict:
    """순수 측정 — 바이너리·파일 접근 없음(가드가 픽스처로 시험 가능).

    commands: capabilities 의 `commands` 배열(각 원소에 name·category).
    used: gym 이 부르는 명령 집합.
    """
    agent = [c for c in commands if c.get("category") in AGENT_CATEGORIES]
    agent_names = {c["name"] for c in agent}
    covered = sorted(agent_names & used)
    uncovered = sorted(agent_names - used)

    by_cat: dict[str, list[str]] = {}
    for c in agent:
        if c["name"] in uncovered:
            by_cat.setdefault(c["category"], []).append(c["name"])
    for cat in by_cat:
        by_cat[cat].sort()

    excluded = sorted(
        c["name"] for c in commands if c.get("category") in EXCLUDED_CATEGORIES
    )
    total = len(agent_names)
    return {
        "kind": "gymCoverage",
        "schemaVersion": "1.0",
        "agentFacingTotal": total,
        "covered": len(covered),
        "uncovered": len(uncovered),
        # 정수 백분율 — 분모 0 이면 100(잴 게 없으면 빈 곳도 없다).
        "coveragePercent": (100 * len(covered) // total) if total else 100,
        "uncoveredByCategory": by_cat,
        "coveredCommands": covered,
        "excludedNonAgent": excluded,
    }


def _capabilities_from_bin(bin_path: str) -> list[dict]:
    out = subprocess.run(
        [bin_path, "capabilities"], capture_output=True, cwd=REPO_ROOT
    )
    return json.loads(out.stdout)["commands"]


def main() -> int:
    ap = argparse.ArgumentParser(description="gym 에이전트-대면 능력 커버리지 측정")
    ap.add_argument("--bin", help="rhwp 바이너리 (capabilities 취득)")
    ap.add_argument("--capabilities", help="저장된 capabilities JSON 파일")
    ap.add_argument("--json", action="store_true", help="JSON 출력")
    a = ap.parse_args()

    if a.capabilities:
        with open(a.capabilities, encoding="utf-8") as fh:
            commands = json.load(fh)["commands"]
    elif a.bin:
        commands = _capabilities_from_bin(a.bin)
    else:
        print("필수: --bin <경로> 또는 --capabilities <파일>", file=sys.stderr)
        return 2

    report = measure(commands, used_commands(GYM_ROOT))
    if a.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        return 0

    print(
        f"에이전트-대면 gym 커버리지: {report['covered']}/{report['agentFacingTotal']}"
        f" ({report['coveragePercent']}%)"
    )
    if report["uncoveredByCategory"]:
        print("미노출 (진짜 빈 곳 — 여기부터 새 과제):")
        for cat in sorted(report["uncoveredByCategory"]):
            names = ", ".join(report["uncoveredByCategory"][cat])
            print(f"  [{cat}] {names}")
    else:
        print("에이전트-대면 능력 전부 노출됨.")
    print(f"제외(비-에이전트 {len(report['excludedNonAgent'])}개): "
          "diagnostic·internal·serve 는 분모 밖")
    return 0


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
