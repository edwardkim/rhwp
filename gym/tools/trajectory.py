"""gym 트라젝토리 필요성 감사 — 다단계 과제의 마지막 스텝이 정말 load-bearing 인가.

## 왜 이 도구인가 (종점만 채점하는 프론티어의 사각)

2026 에이전트 평가의 합의: **종점만 보면 안 된다**. 에이전트가 옳은 결과에 낭비·
위험·우회 경로로 도달해도 종점만 채점하면 만점이다(프로덕션 실패). 그래서 프론티어
프레임워크들은 트라젝토리(결정 경로)를 채점한다 — 다만 대부분 **LLM-judge** 아니면
**골든 경로**로. 둘 다 취약하다(judge 불안정, 골든 취성).

gym 도 종점-오라클이라 같은 사각을 가진다. 다단계 과제가 "N 스텝을 하라"고
광고해도, 채점이 마지막 스텝의 산출을 실제로 요구하지 않으면 그 과제는 **연극**이다
— 에이전트는 N-1 스텝만 하고도 만점을 받는다.

이 감사기는 골든 경로도 judge 도 없이 그 연극을 잡는다: 각 다단계 과제에서
**마지막 외부 의미 스텝을 빼고**(부분 트라젝토리) 기준 풀이를 재조립해 채점한다.
trailing `answer`·`keyring_from`은 제출을 모으는 내부 단계이므로 남겨야 마지막 실제
에이전트 동작이 load-bearing인지 판별할 수 있다.

- 부분 트라젝토리가 **통과** → 마지막 스텝(=선언된 최종 산출물)이 채점에 무의미.
  트라젝토리 연극이다. 리포트한다.
- 부분 트라젝토리가 **실패**(빌드 실패 포함) → 마지막 스텝이 load-bearing. 정상.

이것이 #4808 판별력 감사(종점: "산출이 입력과 다른가")를 **경로**로 민 것이다:
종점의 무편집 거부 → 경로의 무의미-스텝 거부. 모든 선언된 스텝이 결과를 바꿔야 한다.

## 사용

    python gym/tools/trajectory.py --bin target/debug/rhwp        # 전 다단계 과제 감사
    python gym/tools/trajectory.py --bin target/debug/rhwp --json
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)
sys.path.insert(0, REPO_ROOT)

from gym.core import runner  # noqa: E402

# build_baseline 을 모듈로 실어 기준 풀이 조립기를 재사용한다(부분 트라젝토리도
# 같은 조립기로 만들어 채점 경로를 동일하게 유지).
_spec = importlib.util.spec_from_file_location("gym_build_baseline", os.path.join(HERE, "build_baseline.py"))
baseline = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(baseline)

COLLECTION_STEP_KEYS = frozenset({"answer", "keyring_from"})


def last_meaningful_step_index(steps: list[dict]) -> int | None:
    """수집 전용 tail을 건너뛴 마지막 외부 의미 기준 풀이 step 위치."""
    for index in range(len(steps) - 1, -1, -1):
        if not COLLECTION_STEP_KEYS.intersection(steps[index]):
            return index
    return None


def multi_step_tasks(gym_root: str):
    """(pack_id, task, reference) 중 reference 가 ≥2 스텝인 것만."""
    packs_dir = os.path.join(gym_root, "packs")
    for pack_id in sorted(os.listdir(packs_dir)):
        tasks_dir = os.path.join(packs_dir, pack_id, "tasks")
        ref_dir = os.path.join(packs_dir, pack_id, "reference")
        if not os.path.isdir(tasks_dir) or not os.path.isdir(ref_dir):
            continue
        for name in sorted(os.listdir(tasks_dir)):
            if not name.endswith(".json"):
                continue
            ref_path = os.path.join(ref_dir, name)
            if not os.path.isfile(ref_path):
                continue
            with open(os.path.join(tasks_dir, name), encoding="utf-8") as fh:
                task = json.load(fh)
            with open(ref_path, encoding="utf-8") as fh:
                reference = json.load(fh)
            if len(reference.get("steps", [])) >= 2:
                yield pack_id, task, reference


def audit(bin_path: str, gym_root: str, work_root: str) -> dict:
    results = []
    theater = []
    for pack_id, task, reference in multi_step_tasks(gym_root):
        steps = reference["steps"]
        removed_index = last_meaningful_step_index(steps)
        if removed_index is None:
            continue
        truncated = dict(reference)
        # answer/keyring 같은 trailing 수집 단계는 유지하고 마지막 실제 동작만 뺀다.
        truncated["steps"] = steps[:removed_index] + steps[removed_index + 1:]
        sub_root = os.path.join(work_root, pack_id)
        load_bearing = True
        try:
            baseline.build_task(bin_path, pack_id, task, truncated, work_root)
            result = runner.score_task(task, sub_root, bin_path)
            # 부분 트라젝토리가 통과 = 마지막 스텝이 무의미(연극).
            load_bearing = not result.get("pass")
        except Exception:
            # 부분 트라젝토리가 유효 제출을 못 만듦 = 마지막 스텝이 필수(정상).
            load_bearing = True
        removed_kind = "/".join(sorted(steps[removed_index]))
        results.append({"pack": pack_id, "task": task["id"], "loadBearing": load_bearing,
                        "steps": len(steps), "removedStep": removed_kind})
        if not load_bearing:
            theater.append(f"{pack_id}/{task['id']} (마지막 실제 스텝 {removed_kind}을 빼도 통과 — "
                           f"{len(steps)}→{len(steps) - 1})")
    return {
        "kind": "gymTrajectoryNecessity",
        "schemaVersion": "1.0",
        "ok": len(theater) == 0,
        "taskCount": len(results),
        "loadBearing": sum(1 for r in results if r["loadBearing"]),
        "theater": theater,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="gym 트라젝토리 필요성 감사 — 무의미한 마지막 스텝(연극) 색출")
    ap.add_argument("--bin", required=True)
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    bin_path = runner.find_bin(a.bin)
    work_root = os.path.join(GYM_ROOT, "submissions", "_trajectory_audit")
    shutil.rmtree(work_root, ignore_errors=True)
    report = audit(bin_path, GYM_ROOT, work_root)
    shutil.rmtree(work_root, ignore_errors=True)
    if a.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    elif report["ok"]:
        print(f"gym 트라젝토리 필요성 감사: {report['taskCount']} 다단계 과제 전부 "
              "마지막 스텝이 load-bearing — 연극 0")
    else:
        print(f"gym 트라젝토리 필요성 감사: 연극(무의미한 마지막 스텝) {len(report['theater'])}건 — "
              "부분 트라젝토리가 통과한다:")
        for t in report["theater"]:
            print(f"  - {t}")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
