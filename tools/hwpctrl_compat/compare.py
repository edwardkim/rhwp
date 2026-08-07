"""두 러너의 산출물을 대조한다 (P0 — L2·L3).

- **L2 반환값**: 호출별 반환값이 같은가. 이것이 100% 호환의 1차 판정이다.
- **L3 문서 상태**: 시나리오가 문서를 저장했다면, 두 저장본이 같은 문서인가.
  P0 은 쪽수와 필드 값(이름→값)으로 본다. 표·서식 축은 P3 부터 넓힌다.
- L4(픽셀)는 시각에 영향을 주는 축(P4~P5)에서 붙인다. 여기서는 다루지 않는다.

## 쓰임

    python tools/hwpctrl_compat/compare.py --ocx output/poc/hwpctrl/ocx \
        --rhwp output/poc/hwpctrl/rhwp --out output/poc/hwpctrl/verdict

## 판정 코드

| 코드 | 뜻 |
|---|---|
| `MATCH` | 값이 같다 |
| `MISSING_API` | rhwp 쪽에 그 API 가 없다 |
| `VALUE_DIFF` | 둘 다 답했지만 값이 다르다 |
| `ERROR_DIFF` | 한쪽만 예외를 냈다 |
| `OCX_ERROR` | 오라클이 실패했다 — 시나리오나 COM 규약을 의심하라 |
"""

from __future__ import annotations

import argparse
import io
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
DEFAULT_EXE = REPO / "target" / "release" / "rhwp.exe"
SCENARIO_DIR = Path(__file__).resolve().parent / "scenarios"


def load(path: Path) -> dict:
    with io.open(path, encoding="utf-8") as fh:
        return json.load(fh)


def selected_oracle_paths(ocx_dir: Path, scenarios: list[str] | None) -> list[Path]:
    """명시한 시나리오만 비교해 이전 실행의 정답지가 섞이지 않게 한다."""
    paths = sorted(ocx_dir.glob("*.returns.json"))
    if scenarios is None:
        return paths
    allowed = set(scenarios)
    return [path for path in paths if path.name.removesuffix(".returns.json") in allowed]


def saved_path(value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else REPO / path


def classify(ocx_call: dict, rhwp_call: dict) -> tuple[str, str]:
    ocx_err = ocx_call.get("error")
    rhwp_err = rhwp_call.get("error")
    if ocx_err and rhwp_err:
        return "MATCH", "양쪽 모두 예외"
    if ocx_err:
        return "OCX_ERROR", ocx_err
    if rhwp_err:
        if str(rhwp_err).startswith("MissingApi"):
            return "MISSING_API", rhwp_err
        return "ERROR_DIFF", rhwp_err
    if ocx_call.get("value") == rhwp_call.get("value"):
        return "MATCH", ""
    ocx_value = json.dumps(ocx_call.get("value"), ensure_ascii=False)
    rhwp_value = json.dumps(rhwp_call.get("value"), ensure_ascii=False)
    return "VALUE_DIFF", f"ocx={ocx_value} rhwp={rhwp_value}"


def cli_json(exe: Path, args: list[str]) -> dict | None:
    proc = subprocess.run([str(exe), *args], capture_output=True, check=False)
    if proc.returncode != 0:
        return None
    try:
        return json.loads(proc.stdout.decode("utf-8", "replace"))
    except json.JSONDecodeError:
        return None


def doc_state(exe: Path, path: Path) -> dict | None:
    """저장본의 상태 요약. 두 저장본을 **같은 파서**로 읽어 비교한다."""
    if not path.exists():
        return None
    info = cli_json(exe, ["info", str(path), "--json"])
    fields = cli_json(exe, ["fields", str(path), "--json"])
    if info is None:
        return {"unreadable": True}
    field_map = {}
    if fields:
        rows = fields.get("fields", fields) if isinstance(fields, dict) else fields
        for f in rows:
            field_map.setdefault(f.get("name", ""), []).append(f.get("value", ""))
    return {
        "pageCount": info.get("pageCount"),
        "fieldCount": sum(len(v) for v in field_map.values()),
        "fields": field_map,
    }


def compare_saved(exe: Path, ocx: dict, rhwp: dict) -> dict | None:
    if not ocx.get("saved") or not rhwp.get("saved"):
        return None
    ocx_state = doc_state(exe, saved_path(ocx["saved"]["path"]))
    rhwp_state = doc_state(exe, saved_path(rhwp["saved"]["path"]))
    if ocx_state is None or rhwp_state is None:
        return {"verdict": "SAVED_MISSING", "ocx": ocx_state, "rhwp": rhwp_state}
    diffs = []
    if ocx_state.get("pageCount") != rhwp_state.get("pageCount"):
        diffs.append(f"pageCount ocx={ocx_state.get('pageCount')} rhwp={rhwp_state.get('pageCount')}")
    names = set(ocx_state.get("fields", {})) | set(rhwp_state.get("fields", {}))
    for name in sorted(names):
        a = ocx_state.get("fields", {}).get(name)
        b = rhwp_state.get("fields", {}).get(name)
        if a != b:
            diffs.append(f"field[{name}] ocx={a} rhwp={b}")
    return {
        "verdict": "MATCH" if not diffs else "DOC_DIFF",
        "diffCount": len(diffs),
        # 필드가 수백 개인 문서에서 전량을 싣지 않는다. 개수는 위에 있고, 여기는 표본이다.
        "diffs": diffs[:40],
        "truncated": len(diffs) > 40,
    }


def compare_one(exe: Path, ocx_path: Path, rhwp_path: Path) -> dict:
    ocx = load(ocx_path)
    rhwp = load(rhwp_path)
    rows = []
    n = max(len(ocx["calls"]), len(rhwp["calls"]))
    for i in range(n):
        o = ocx["calls"][i] if i < len(ocx["calls"]) else {"call": "(없음)", "error": "호출 없음"}
        r = rhwp["calls"][i] if i < len(rhwp["calls"]) else {"call": "(없음)", "error": "호출 없음"}
        if o.get("call") != r.get("call"):
            rows.append(
                {"index": i, "call": f"{o.get('call')}≠{r.get('call')}", "code": "ERROR_DIFF",
                 "detail": "호출 순서가 어긋났다 — 러너 버그를 의심하라"}
            )
            continue
        code, detail = classify(o, r)
        rows.append({"index": i, "call": o.get("call"), "code": code, "detail": detail})

    counts: dict[str, int] = {}
    for row in rows:
        counts[row["code"]] = counts.get(row["code"], 0) + 1
    l3 = compare_saved(exe, ocx, rhwp)
    # 시나리오가 어떤 원장 항목을 검증하려 했는지. 원장은 **시나리오 단위로** 통과해야
    # 올라간다 — 반환값만 맞고 부작용이 없는 no-op 이 통과하는 구멍을 막는다.
    scenario_file = SCENARIO_DIR / f"{ocx['scenario']}.json"
    declared = []
    if scenario_file.exists():
        declared = load(scenario_file).get("ledger", [])
    return {
        "scenario": ocx["scenario"],
        "impl": rhwp.get("impl"),
        "oracle": ocx.get("oracle"),
        "ledger": declared,
        "l2": {"total": len(rows), "counts": counts, "rows": rows},
        "l3": l3,
        "pass": counts.get("MATCH", 0) == len(rows) and (l3 is None or l3["verdict"] == "MATCH"),
    }


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--ocx", required=True)
    ap.add_argument("--rhwp", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--exe", default=str(DEFAULT_EXE))
    ap.add_argument("--scenario", action="append", dest="scenarios", help="비교할 시나리오 id (반복 가능)")
    ap.add_argument("--empty", action="store_true", help="비교 대상 없이 빈 판정 파일만 생성")
    args = ap.parse_args()

    if args.empty and args.scenarios:
        ap.error("--empty와 --scenario는 함께 사용할 수 없습니다")

    ocx_dir, rhwp_dir, out_dir = Path(args.ocx), Path(args.rhwp), Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    exe = Path(args.exe)

    reports = []
    oracle_paths = [] if args.empty else selected_oracle_paths(ocx_dir, args.scenarios)
    for ocx_path in oracle_paths:
        rhwp_path = rhwp_dir / ocx_path.name
        if not rhwp_path.exists():
            print(f"건너뜀 — rhwp 산출물 없음: {rhwp_path.name}")
            continue
        reports.append(compare_one(exe, ocx_path, rhwp_path))

    lines = ["scenario\tindex\tcall\tcode\tdetail"]
    for rep in reports:
        for row in rep["l2"]["rows"]:
            lines.append(f"{rep['scenario']}\t{row['index']}\t{row['call']}\t{row['code']}\t{row['detail']}")
    with io.open(out_dir / "verdict.tsv", "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines) + "\n")
    with io.open(out_dir / "verdict.json", "w", encoding="utf-8", newline="\n") as fh:
        json.dump({"schemaVersion": "1.0", "reports": reports}, fh, ensure_ascii=False, indent=2)
        fh.write("\n")

    total = sum(r["l2"]["total"] for r in reports)
    match = sum(r["l2"]["counts"].get("MATCH", 0) for r in reports)
    print(f"시나리오 {len(reports)}건 · 호출 {total}건 · 일치 {match}건")
    for rep in reports:
        codes = ", ".join(f"{k} {v}" for k, v in sorted(rep["l2"]["counts"].items()))
        l3 = rep["l3"]["verdict"] if rep["l3"] else "-"
        print(f"  {rep['scenario']}: {codes} | L3 {l3}")
    print(f"→ {out_dir / 'verdict.tsv'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
