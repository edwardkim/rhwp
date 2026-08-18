"""gym 능력 커버리지 측정 — 에이전트-대면 명령 중 몇 %가 gym 과제로 실측되나.

## 왜 이 도구인가

새 gym 과제를 만들기 전 "이 능력이 이미 커버돼 있나"를 재는 장치가 없어서, 이미
`fill-fields` 를 커버한 core-cli T07 위에 중복 pack(#4781)이 만들어졌다 자진 철회된
사고가 있었다. 이 도구는 그 재발을 막는다 — 만들기 전에 **진짜 빈 곳**만 잰다.

## 무엇을 재나 (정직한 분모)

capabilities 의 `category` 로 **에이전트-대면 명령**(`batch`·`edit`·`export`·`query`)
만 분모로 삼는다. `diagnostic`(hwp5-*·dump-* 개발 probe)·`internal`·`serve`(인프라)는
제외한다 — 진단 도구를 빈 곳으로 세면 커버리지가 실제보다 낮게 나와 오해를 부른다.

한 명령은 gym 과제·기준풀이의 `checks[].cmd[0]` 또는 `steps[].run[0]` /
`steps[].answer.*.cmd[0]` 에 나타나면 '노출'로 친다.

명령 합계만으로는 pack 축이 안 보인다. 그래서 같은 스캔으로 **pack×명령 격자**
(`packs`)와, `gym.core.checks.REGISTRY` 에 등록됐지만 어떤 과제의 `checks[].op` 에도
안 나온 **미사용 연산자**(`unusedOperators`)를 같이 낸다.

## 사용

    python gym/tools/coverage.py --bin target/debug/rhwp   # 바이너리로 capabilities
    python gym/tools/coverage.py --capabilities cap.json    # 저장된 capabilities
    python gym/tools/coverage.py --bin target/debug/rhwp --json
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from typing import Iterable, Iterator

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)

# 에이전트가 실제로 쓰는 능력 카테고리만 분모. 나머지는 개발·인프라 도구.
AGENT_CATEGORIES = {"batch", "edit", "export", "query"}
EXCLUDED_CATEGORIES = {"diagnostic", "internal", "serve"}


def _load_json(path: str) -> dict:
    with open(path, encoding="utf-8") as fh:
        return json.load(fh)


def commands_in_doc(doc: dict) -> set[str]:
    """과제·기준풀이 문서가 부르는 명령(첫 토큰) 집합."""
    used: set[str] = set()
    for check in doc.get("checks", []):
        cmd = check.get("cmd") or []
        if cmd:
            used.add(cmd[0])
    for step in doc.get("steps", []):
        run = step.get("run") or []
        if run:
            used.add(run[0])
        answer = step.get("answer") or {}
        if isinstance(answer, dict):
            for spec in answer.values():
                if isinstance(spec, dict):
                    cmd = spec.get("cmd") or []
                    if cmd:
                        used.add(cmd[0])
    return used


def operators_in_doc(doc: dict) -> set[str]:
    """과제 문서의 검사 연산자 집합. 기준풀이에는 checks 가 없다."""
    return {c["op"] for c in doc.get("checks", []) if c.get("op")}


def list_pack_ids(packs_root: str) -> list[str]:
    """pack.json 이 있는 폴더 이름. 격자 행은 이 목록이 기준이다."""
    packs_dir = os.path.join(packs_root, "packs")
    if not os.path.isdir(packs_dir):
        return []
    ids: list[str] = []
    for name in sorted(os.listdir(packs_dir)):
        pack_dir = os.path.join(packs_dir, name)
        if os.path.isdir(pack_dir) and os.path.isfile(os.path.join(pack_dir, "pack.json")):
            ids.append(name)
    return ids


def iter_pack_docs(packs_root: str, subdir: str) -> Iterator[tuple[str, str, dict]]:
    """packs/<id>/<subdir>/*.json 을 (packId, path, doc) 으로 낸다."""
    packs_dir = os.path.join(packs_root, "packs")
    if not os.path.isdir(packs_dir):
        return
    for pack_id in sorted(os.listdir(packs_dir)):
        folder = os.path.join(packs_dir, pack_id, subdir)
        if not os.path.isdir(folder):
            continue
        for name in sorted(os.listdir(folder)):
            if not name.endswith(".json"):
                continue
            path = os.path.join(folder, name)
            yield pack_id, path, _load_json(path)


def used_commands(packs_root: str) -> set[str]:
    """gym 과제·기준풀이가 실제로 부르는 명령(첫 토큰) 집합."""
    used: set[str] = set()
    for _pack_id, _path, doc in iter_pack_docs(packs_root, "tasks"):
        used |= commands_in_doc(doc)
    for _pack_id, _path, doc in iter_pack_docs(packs_root, "reference"):
        used |= commands_in_doc(doc)
    return used


def used_commands_by_pack(packs_root: str) -> dict[str, list[str]]:
    """packId → 그 pack 의 과제+기준풀이가 부르는 명령(정렬).

    과제가 없는 pack 도 빈 목록으로 남긴다 — 격자의 빈 행이 곧 빈 곳이다.
    """
    grid: dict[str, set[str]] = {pid: set() for pid in list_pack_ids(packs_root)}
    for subdir in ("tasks", "reference"):
        for pack_id, _path, doc in iter_pack_docs(packs_root, subdir):
            grid.setdefault(pack_id, set()).update(commands_in_doc(doc))
    return {pid: sorted(cmds) for pid, cmds in sorted(grid.items())}


def used_operators(packs_root: str) -> set[str]:
    """어느 과제 checks[].op 에라도 나타난 연산자. 기준풀이는 세지 않는다."""
    used: set[str] = set()
    for _pack_id, _path, doc in iter_pack_docs(packs_root, "tasks"):
        used |= operators_in_doc(doc)
    return used


def registered_operators() -> frozenset[str]:
    """gym.core.checks.REGISTRY 키. 바이너리 없이 등록부만 읽는다."""
    if GYM_ROOT not in sys.path:
        sys.path.insert(0, GYM_ROOT)
    from core.checks import REGISTRY  # noqa: WPS433 — 도구 스크립트, 지연 import

    return frozenset(REGISTRY)


def unused_operators(
    packs_root: str, registry: Iterable[str] | None = None
) -> list[str]:
    """REGISTRY 에 있으나 어떤 과제도 쓰지 않는 연산자(정렬)."""
    names = set(registry) if registry is not None else set(registered_operators())
    return sorted(names - used_operators(packs_root))


def _sorted_pack_grid(packs: dict[str, Iterable[str]] | None) -> dict[str, list[str]]:
    if not packs:
        return {}
    return {pid: sorted(cmds) for pid, cmds in sorted(packs.items())}


def measure(
    commands: list[dict],
    used: set[str],
    packs: dict[str, Iterable[str]] | None = None,
    unused_operators: Iterable[str] | None = None,
) -> dict:
    """순수 측정 — 바이너리·파일 접근 없음(가드가 픽스처로 시험 가능).

    commands: capabilities 의 `commands` 배열(각 원소에 name·category).
    used: gym 이 부르는 명령 집합.
    packs: packId → 그 pack 이 쓰는 명령. 생략하면 빈 격자.
    unused_operators: 등록됐지만 과제가 안 쓰는 연산자. 생략하면 빈 목록.

    기존 키(agentFacingTotal·covered·…)의 의미는 그대로 둔다. packs /
    unusedOperators 는 같은 봉투에 덧붙인다.
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
        "packs": _sorted_pack_grid(packs),
        "unusedOperators": sorted(unused_operators or []),
    }


def report(commands: list[dict], packs_root: str) -> dict:
    """capabilities + gym 스캔을 한 봉투로 합친다. 바이너리는 부르지 않는다."""
    return measure(
        commands,
        used_commands(packs_root),
        packs=used_commands_by_pack(packs_root),
        unused_operators=unused_operators(packs_root),
    )


def _capabilities_from_bin(bin_path: str) -> list[dict]:
    out = subprocess.run(
        [bin_path, "capabilities"], capture_output=True, cwd=REPO_ROOT
    )
    return json.loads(out.stdout)["commands"]


def format_human(rep: dict) -> str:
    """JSON 이 아닌 사람용 요약. 격자와 미사용 연산자를 빠뜨리지 않는다."""
    lines = [
        f"에이전트-대면 gym 커버리지: {rep['covered']}/{rep['agentFacingTotal']}"
        f" ({rep['coveragePercent']}%)"
    ]
    if rep["uncoveredByCategory"]:
        lines.append("미노출 (진짜 빈 곳 — 여기부터 새 과제):")
        for cat in sorted(rep["uncoveredByCategory"]):
            names = ", ".join(rep["uncoveredByCategory"][cat])
            lines.append(f"  [{cat}] {names}")
    else:
        lines.append("에이전트-대면 능력 전부 노출됨.")
    lines.append(
        f"제외(비-에이전트 {len(rep['excludedNonAgent'])}개): "
        "diagnostic·internal·serve 는 분모 밖"
    )
    packs = rep.get("packs") or {}
    lines.append(f"pack×명령 격자 ({len(packs)} pack):")
    if packs:
        for pid in sorted(packs):
            cmds = packs[pid]
            shown = ", ".join(cmds) if cmds else "(없음)"
            lines.append(f"  [{pid}] {shown}")
    else:
        lines.append("  (pack 스캔 없음)")
    unused = rep.get("unusedOperators") or []
    if unused:
        lines.append(f"미사용 연산자 ({len(unused)}): {', '.join(unused)}")
    else:
        lines.append("미사용 연산자 없음 — REGISTRY 전부가 과제에 노출됨.")
    return "\n".join(lines) + "\n"


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

    rep = report(commands, GYM_ROOT)
    if a.json:
        sys.stdout.write(json.dumps(rep, ensure_ascii=False, indent=2) + "\n")
        return 0

    sys.stdout.write(format_human(rep))
    return 0


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
