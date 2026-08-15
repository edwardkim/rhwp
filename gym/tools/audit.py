"""gym 정합 감사 — 모든 pack 이 "그 방식"(해결 가능·고유·정합)을 지키는지 전수 검사.

## 왜 이 도구인가 (강제된 표준)

gym 이 자라고 기여자가 늘수록, 새 pack 이 조용히 규약을 어길 수 있다: 과제에 기준
풀이가 없거나(=해결 가능성 미선언), 과제 ID 가 다른 pack 과 충돌하거나, 스키마를
벗어나거나. 개별 검증(`schema.validate_pack`/`validate_task`)은 pack 하나·과제 하나만
본다 — **전 저장소에 걸친 정합**(과제↔기준 짝·과제 ID 전역 고유·고아 기준풀이)은
아무도 안 본다. 그 틈으로 정합이 무너진다.

이 감사기가 그 전수 정합을 강제한다. gym 에 기여하는 모든 에이전트의 pack 은 이걸
통과해야 한다 — 벗어날 수 없되 감옥이 아니라 **품질 관문**이다(규칙은 열려 있고,
검사하는 것은 해결 가능성·고유성·정합 같은 품질이다). 바이너리 없이 순수 파일 검사라
CI 에서 상시 돈다.

## 검사하는 것 (그 방식)

- **스키마 정합** — `schema.validate_pack` + `validate_task`(바이너리 불요, 명령 존재
  검사만 러너에 위임).
- **해결 가능성 선언** — 모든 과제에 짝 기준풀이(`tasks/X.json` ↔ `reference/X.json`,
  id 일치). 기준풀이 없는 과제는 "풀 수 있다" 는 근거가 없다.
- **고아 기준풀이 없음** — 기준풀이는 반드시 과제를 가진다.
- **과제 ID 전역 고유** — pack 간 ID 충돌 금지(리더보드·집계가 ID 로 과제를 가른다).

## 사용

    python gym/tools/audit.py           # 전 pack 감사, 문제 있으면 exit 1
    python gym/tools/audit.py --json
"""

from __future__ import annotations

import argparse
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
sys.path.insert(0, GYM_ROOT)

from core import schema  # noqa: E402  (gym/core)


def _load(path):
    with open(path, encoding="utf-8") as fh:
        return json.load(fh)


def audit(packs_root: str) -> dict:
    """전 pack 정합 감사 — 순수 파일 검사(바이너리·네트워크 없음).

    packs_root: `gym/packs` 를 담은 디렉토리(보통 gym/).
    반환: {ok, packs:[{id, issues}], taskIdCollisions, issueCount}.
    """
    packs_dir = os.path.join(packs_root, "packs")
    pack_reports = []
    task_id_owners: dict[str, list[str]] = {}

    for pack_id in sorted(os.listdir(packs_dir)):
        pack_dir = os.path.join(packs_dir, pack_id)
        if not os.path.isdir(pack_dir):
            continue
        issues: list[str] = []
        manifest_path = os.path.join(pack_dir, "pack.json")
        if not os.path.isfile(manifest_path):
            pack_reports.append({"id": pack_id, "issues": ["pack.json 이 없다"]})
            continue
        try:
            manifest = _load(manifest_path)
        except (ValueError, OSError) as e:
            pack_reports.append({"id": pack_id, "issues": [f"pack.json 파싱 실패: {e}"]})
            continue

        schema.validate_pack(manifest, pack_dir, issues)

        tasks_dir = os.path.join(pack_dir, "tasks")
        ref_dir = os.path.join(pack_dir, "reference")
        task_ids: set[str] = set()
        task_files = sorted(f for f in os.listdir(tasks_dir)) if os.path.isdir(tasks_dir) else []
        ref_files = set(f for f in os.listdir(ref_dir)) if os.path.isdir(ref_dir) else set()

        for name in task_files:
            if not name.endswith(".json"):
                continue
            try:
                task = _load(os.path.join(tasks_dir, name))
            except (ValueError, OSError) as e:
                issues.append(f"tasks/{name} 파싱 실패: {e}")
                continue
            # 명령 존재 검사만 러너에 위임(known_commands=None) — 구조는 그대로 검증.
            schema.validate_task(task, manifest, None, issues)
            tid = task.get("id")
            if tid:
                task_ids.add(tid)
                task_id_owners.setdefault(tid, []).append(pack_id)
            # 해결 가능성: 짝 기준풀이가 있고 id 가 맞는가.
            if name not in ref_files:
                issues.append(f"과제 {name} 에 짝 기준풀이(reference/{name})가 없다 — 해결 가능성 미선언")
            else:
                try:
                    ref = _load(os.path.join(ref_dir, name))
                    if ref.get("id") != tid:
                        issues.append(f"reference/{name} 의 id({ref.get('id')}) 가 과제 id({tid}) 와 다르다")
                except (ValueError, OSError) as e:
                    issues.append(f"reference/{name} 파싱 실패: {e}")

        # 고아 기준풀이: 과제 없는 reference.
        task_names = {f for f in task_files if f.endswith(".json")}
        for name in sorted(ref_files):
            if name.endswith(".json") and name not in task_names:
                issues.append(f"고아 기준풀이 reference/{name} — 짝 과제(tasks/{name})가 없다")

        pack_reports.append({"id": pack_id, "issues": issues})

    collisions = {tid: owners for tid, owners in task_id_owners.items() if len(owners) > 1}
    issue_count = sum(len(p["issues"]) for p in pack_reports) + len(collisions)
    return {
        "kind": "gymAudit",
        "schemaVersion": "1.0",
        "ok": issue_count == 0,
        "packCount": len(pack_reports),
        "packs": [p for p in pack_reports if p["issues"]],
        "taskIdCollisions": collisions,
        "issueCount": issue_count,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="gym 전 pack 정합 감사 (해결가능·고유·정합)")
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    report = audit(GYM_ROOT)
    if a.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    elif report["ok"]:
        print(f"gym 정합 감사: {report['packCount']} pack 전부 통과 — 위반 0")
    else:
        print(f"gym 정합 감사: 위반 {report['issueCount']}건")
        for p in report["packs"]:
            for issue in p["issues"]:
                print(f"  [{p['id']}] {issue}")
        for tid, owners in report["taskIdCollisions"].items():
            print(f"  [전역] 과제 ID '{tid}' 충돌: {', '.join(owners)}")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
