"""스펙 3종 → API 원장 (P0).

원장은 **진척의 유일한 진실**이다. 사람이 "구현했다"고 적는 곳이 아니라, 오라클 대조가
`verified` 를 채우는 곳이다(계획서 §5).

    npm/hwpctrl-ocx/spec/api_ledger.json

## 상태 값

- `unimplemented` — 아직 없다.
- `implemented`   — 구현했지만 오라클 대조를 통과하지 않았다. **진척으로 세지 않는다.**
- `verified`      — 오라클 대조 0 diff. 이 값만 진척이다.
- `substituted`   — 브라우저 제약으로 계약을 바꿔 구현했다(계획서 §1.1). 사유 필수.
- `by-design-noop`— 규격상 아무 일도 하지 않는 것이 맞다(예: UI 표시 API).

## 쓰임

    python tools/hwpctrl_compat/build_ledger.py            # 없으면 만들고, 있으면 병합
    python tools/hwpctrl_compat/build_ledger.py --check    # 스펙과 원장이 어긋나면 exit 1

`--check` 는 CI 게이트 ①이다(계획서 §5). 스펙에 있는데 원장에 없거나 그 반대면 실패한다.
기존 상태는 **병합 시 보존**된다 — 재생성이 진척을 지우면 원장이 진실일 수 없다.
"""

from __future__ import annotations

import argparse
import io
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SPEC_DIR = REPO / "npm" / "hwpctrl-ocx" / "spec"
LEDGER = SPEC_DIR / "api_ledger.json"
LEGACY_DIR = REPO / "rhwp-studio" / "src" / "hwpctl"

VALID_STATUS = {"unimplemented", "implemented", "verified", "substituted", "by-design-noop"}


def load(name: str) -> dict:
    path = SPEC_DIR / name
    if not path.exists():
        raise SystemExit(f"스펙 없음: {path}\n먼저 extract_spec.py 를 돌려라.")
    with io.open(path, encoding="utf-8") as fh:
        return json.load(fh)


def legacy_surface() -> dict[str, set[str]]:
    """기존 `rhwp-studio/src/hwpctl/` 이 이미 이름으로 가진 것들.

    신규 패키지의 진척과 **무관**하다. 참고 열로만 싣는다 — P7 이관(계획서 §6.2) 때
    "기존 층이 무엇을 하고 있었는지"를 원장 하나로 볼 수 있어야 한다.
    """
    methods: set[str] = set()
    actions: set[str] = set()
    index = LEGACY_DIR / "index.ts"
    if index.exists():
        text = index.read_text(encoding="utf-8")
        # 클래스 본문의 공개 메서드 선언만 센다.
        for m in re.finditer(r"^  ([A-Z]\w*)\(", text, re.M):
            methods.add(m.group(1))
        if re.search(r"^  addEventListener\(", text, re.M):
            methods.add("AddEventListener")
    for path in [LEGACY_DIR / "action-registry.ts", *(LEGACY_DIR / "actions").glob("*.ts")]:
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        for m in re.finditer(r"(?:\[|id: )'([A-Za-z]\w*)'", text):
            actions.add(m.group(1))
    return {"methods": methods, "actions": actions}


def build_entries() -> list[dict]:
    api = load("webhwpctrl_api.json")
    sets = load("parameter_sets.json")
    actions = load("actions.json")
    legacy = legacy_surface()

    entries: list[dict] = []
    for e in api["entries"]:
        entries.append(
            {
                "id": f"{e['object']}.{e['kind']}.{e['name']}",
                "unit": e["kind"],
                "object": e["object"],
                "name": e["name"],
                "arity": e["arity"],
                "section": e["section"],
                "status": "unimplemented",
                "notes": "",
                "legacyHwpctl": e["object"] == "HwpCtrl" and e["name"] in legacy["methods"],
                # 문서 자체가 서명과 `Parameters N` 을 다르게 적은 항목. OCX 실측으로 확정한다.
                "specArityAmbiguous": e["arityMismatch"],
                "oracle": {"scenarios": [], "lastRun": None, "diff": None},
            }
        )
    for a in actions["actions"]:
        entries.append(
            {
                "id": f"Action.{a['actionId']}",
                "unit": "action",
                "object": "Action",
                "name": a["actionId"],
                "parameterSetId": a["parameterSetId"],
                "requiresExternalSet": a["requiresExternalSet"],
                "status": "unimplemented",
                "notes": "",
                "legacyHwpctl": a["actionId"] in legacy["actions"],
                "oracle": {"scenarios": [], "lastRun": None, "diff": None},
            }
        )
    for s in sets["sets"]:
        entries.append(
            {
                "id": f"ParameterSet.{s['setId']}",
                "unit": "parameterSet",
                "object": "ParameterSet",
                "name": s["setId"],
                "aliases": s["aliases"],
                "itemCount": s["itemCount"],
                "status": "unimplemented",
                "notes": "",
                "legacyHwpctl": False,
                "oracle": {"scenarios": [], "lastRun": None, "diff": None},
            }
        )
    return entries


def merge(fresh: list[dict], existing: dict | None) -> tuple[list[dict], list[str]]:
    """새로 생성한 골격에 기존 상태를 얹는다. 진척을 지우지 않는다."""
    problems: list[str] = []
    old = {e["id"]: e for e in existing["entries"]} if existing else {}
    merged = []
    for entry in fresh:
        prev = old.pop(entry["id"], None)
        if prev:
            status = prev.get("status", "unimplemented")
            if status not in VALID_STATUS:
                problems.append(f"{entry['id']}: 알 수 없는 상태 '{status}'")
                status = "unimplemented"
            entry["status"] = status
            entry["notes"] = prev.get("notes", "")
            entry["oracle"] = prev.get("oracle", entry["oracle"])
        merged.append(entry)
    for stale_id in old:
        problems.append(f"스펙에 없는 원장 항목(제거됨): {stale_id}")
    return merged, problems


def summarize(entries: list[dict]) -> dict:
    counts: dict[str, int] = {}
    for e in entries:
        counts[e["status"]] = counts.get(e["status"], 0) + 1
    by_unit: dict[str, dict[str, int]] = {}
    for e in entries:
        bucket = by_unit.setdefault(e["unit"], {})
        bucket[e["status"]] = bucket.get(e["status"], 0) + 1
    # 진척 = verified + substituted + by-design-noop (더는 할 일이 없는 상태들)
    done = sum(counts.get(k, 0) for k in ("verified", "substituted", "by-design-noop"))
    return {
        "total": len(entries),
        "done": done,
        "byStatus": counts,
        "byUnit": by_unit,
    }


def ingest_verdict(entries: list[dict], verdict_path: Path) -> list[str]:
    """오라클 판정으로 상태를 채운다. **사람이 올리지 못하는 유일한 경로다.**

    호출 이름 하나가 여러 시나리오에 나올 수 있으므로, 그 이름의 **모든 행이 MATCH 이고**
    그 시나리오의 L3 도 통과했을 때만 `verified` 로 올린다. 한 번이라도 어긋나면 내린다.
    """
    notes: list[str] = []
    with io.open(verdict_path, encoding="utf-8") as fh:
        verdict = json.load(fh)

    by_id = {e["id"]: e for e in entries}
    touched: dict[str, dict] = {}  # id → {ok, scenarios[]}

    for report in verdict["reports"]:
        passed = report.get("pass", False)
        ids = list(report.get("ledger", []))
        if not ids:
            notes.append(f"{report['scenario']}: ledger 선언이 없다 — 무엇을 검증했는지 알 수 없다")
        for entry_id in ids:
            if entry_id not in by_id:
                notes.append(f"{report['scenario']}: 원장에 없는 항목 선언 {entry_id}")
                continue
            slot = touched.setdefault(entry_id, {"ok": True, "scenarios": []})
            # 시나리오 하나라도 실패하면 그 항목은 검증되지 않은 것이다.
            slot["ok"] = slot["ok"] and passed
            if report["scenario"] not in slot["scenarios"]:
                slot["scenarios"].append(report["scenario"])

    for entry_id, slot in touched.items():
        entry = by_id[entry_id]
        entry["oracle"]["scenarios"] = slot["scenarios"]
        entry["oracle"]["diff"] = 0 if slot["ok"] else 1
        if entry["status"] == "substituted":
            # `substituted` 는 "오라클이 이 항목의 판정자가 될 수 없다"는 선언이다. 그런
            # 항목도 시나리오가 **계약의 일부**(반환 모양·경계 따위)는 재는데, 그 초록으로
            # `verified` 를 채우면 재지 않은 부분까지 잰 것처럼 보인다. 사유를 적어 둔
            # 상태는 오라클이 덮지 않는다.
            if not entry.get("notes"):
                notes.append(f"{entry['id']}: substituted 인데 사유가 없다")
            continue
        if slot["ok"]:
            entry["status"] = "verified"
        elif entry["status"] == "verified":
            # 한 번 verified 였어도 오라클이 어긋나면 내린다. 원장은 최신 실측이다.
            entry["status"] = "implemented"
            notes.append(f"{entry['id']}: verified → implemented (오라클 불일치)")
    return notes


def scenario_ledger_problems(entries: list[dict]) -> list[str]:
    """시나리오가 **없는 원장 이름**을 선언하고 있지 않은지 본다.

    이름이 틀리면 그 시나리오는 아무것도 올리지 못하는데 게이트는 초록이라 **조용히 헛돈다**.
    실제로 겪었다 — 액션은 `Action.X` 인데 컨트롤 본체는 `HwpCtrl.method.X`·
    `HwpCtrl.property.X` 라, `HwpCtrl.GetPosBySet` 이라 적은 시나리오가 통과하고도 원장이
    그대로였다. `--ingest` 때만 경고로 나오던 것을 검사 게이트로 올린다.
    """
    known = {e["id"] for e in entries}
    problems: list[str] = []
    for path in sorted((Path(__file__).resolve().parent / "scenarios").glob("*.json")):
        try:
            with io.open(path, encoding="utf-8") as fh:
                scenario = json.load(fh)
        except (OSError, json.JSONDecodeError) as exc:
            problems.append(f"{path.name}: 읽을 수 없다 — {exc}")
            continue
        for entry_id in scenario.get("ledger", []):
            if entry_id not in known:
                problems.append(f"{path.name}: 원장에 없는 항목 선언 {entry_id}")
    return problems


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="스펙과 원장 불일치 시 exit 1 (CI 게이트)")
    ap.add_argument("--ingest", help="compare.py 가 낸 verdict.json 을 읽어 상태를 채운다")
    args = ap.parse_args()

    existing = None
    if LEDGER.exists():
        with io.open(LEDGER, encoding="utf-8") as fh:
            existing = json.load(fh)

    entries, problems = merge(build_entries(), existing)
    if args.ingest:
        problems += ingest_verdict(entries, Path(args.ingest))
    summary = summarize(entries)

    if args.check:
        problems += scenario_ledger_problems(entries)
        if problems:
            print("원장 검사 실패:")
            for p in problems:
                print(f"  - {p}")
            return 1
        if existing and len(existing["entries"]) != len(entries):
            print(f"원장 항목 수 불일치: 파일 {len(existing['entries'])} vs 스펙 {len(entries)}")
            return 1
        print(f"원장 검사 통과 — {summary['done']}/{summary['total']} 완료")
        return 0

    doc = {
        "schemaVersion": "1.0",
        "target": "웹한글컨트롤 API v2.4 (WebHwpCtrl) 100% 호환",
        "oracleVersion": "한글2022 (설치본 COM)",
        "summary": summary,
        "entries": entries,
    }
    with io.open(LEDGER, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(doc, fh, ensure_ascii=False, indent=2)
        fh.write("\n")

    print(f"원장 {summary['total']}항목 → {LEDGER}")
    print(f"  단위별: " + ", ".join(f"{k} {sum(v.values())}" for k, v in summary["byUnit"].items()))
    print(f"  완료: {summary['done']}/{summary['total']}")
    legacy_count = sum(1 for e in entries if e.get("legacyHwpctl"))
    print(f"  (참고) 기존 hwpctl 층이 이름으로 가진 항목: {legacy_count}")
    for p in problems:
        print(f"  경고: {p}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
