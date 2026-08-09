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
| `ERROR_UNDECLARED` | 양쪽이 죽었는데 시나리오가 그 예외를 선언하지 않았다 |
| `EXPECT_DIFF` | 시나리오가 선언한 기대 반환·기대 오류를 어겼다 |
| `OCX_ERROR` | 오라클이 실패했다 — 시나리오나 COM 규약을 의심하라 |
"""

from __future__ import annotations

import argparse
import io
import json
import subprocess
import sys
from pathlib import Path

from scenario_spec import check_call, contracts

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


def classify(ocx_call: dict, rhwp_call: dict, contract: dict | None = None) -> tuple[str, str]:
    contract = contract or {}
    ocx_err = ocx_call.get("error")
    rhwp_err = rhwp_call.get("error")
    # **`MissingApi` 는 어떤 경우에도 일치가 아니다.** 그것은 "rhwp 가 그 API 를 아직 안
    # 만들었다"는 뜻이라, 마침 오라클도 그 자리에서 죽었다고 해서 맞은 것이 될 수 없다.
    #
    # 이 구멍이 실제로 초록을 만들어 냈다: `p2-group-chain` 이 사슬 끝을 넘어 역참조해
    # 양쪽이 죽었는데 다섯 건이 `MATCH` 로 세어졌다(#4274 리뷰). 없는 것을 없다고 말하는 것과
    # 서로 같은 이유로 죽는 것은 다르다.
    if rhwp_err and str(rhwp_err).startswith("MissingApi"):
        return "MISSING_API", rhwp_err
    # 선언한 계약은 **양쪽 다** 지켜야 한다. 기대 반환값도 여기서 본다 — 자체 검사(Linux)와
    # 이 대조(Windows)가 같은 한 값을 보게 하는 것이 경로가 갈리는 시나리오의 유일한 닻이다.
    if contract.get("expectError") or "expect" in contract:
        for record, side in ((ocx_call, "ocx"), (rhwp_call, "rhwp")):
            breach = check_call(contract, record, side)
            if breach:
                return "EXPECT_DIFF", f"{side}: {breach}"
    if ocx_err and rhwp_err:
        # 선언 없이 양쪽이 죽은 자리는 일치가 아니다 — `MissingApi` 를 막은 것과 같은 이유다.
        # 왜 죽어야 하는지 시나리오가 적어야 그 초록에 뜻이 생긴다.
        if not contract.get("expectError"):
            return "ERROR_UNDECLARED", f"ocx={ocx_err} rhwp={rhwp_err}"
        return "MATCH", "선언한 예외 — 양쪽 모두 그대로 죽었다"
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


def ir_sweep(exe: Path, before: Path, after: Path) -> dict | None:
    """두 저장본의 IR **전수** 차이. `ir-diff` 가 아니라 `ir-sweep` 인 이유가 있다.

    `ir-diff` 의 비교 목록은 손으로 쌓은 것이라 `z_order`·도형 변환 행렬 따위를 아예 안 본다.
    실제로 `ShapeObjBringToFront` 를 걸어 한글이 저장본에 적어 둔 것을 `ir-diff` 는 "동일" 로
    답했고 `ir-sweep` 은 `common.z_order` 1↔2 를 그대로 짚었다.
    """
    if not before.exists() or not after.exists():
        return None
    proc = subprocess.run(
        [str(exe), "ir-sweep", str(before), str(after), "--json"],
        capture_output=True,
        check=False,
    )
    # 차이가 있으면 3 으로 끝난다(`ir-diff` 와 같은 규약). 2·1 은 진짜 실패다.
    if proc.returncode not in (0, 3):
        return {"error": proc.stderr.decode("utf-8", "replace").strip()[:200]}
    try:
        return json.loads(proc.stdout.decode("utf-8", "replace"))
    except json.JSONDecodeError:
        return {"error": "ir-sweep 출력이 JSON 이 아니다"}


def delta_key(row: dict) -> tuple:
    return (row.get("path", ""), str(row.get("left")), str(row.get("right")))


def compare_deltas(exe: Path, definition: dict, ocx_dir: Path, rhwp_dir: Path) -> list[dict]:
    """**액션이 문서에 남긴 자취**를 양쪽에서 재서 대조한다 (L3 확장).

    어느 API 도 결과를 안 비추는 액션이 많다(z-순서·뒤집기·표 칸 조절…). 그래도 저장본은
    적으므로, 액션 앞뒤로 저장한 두 벌의 **차이**를 양쪽에서 뽑아 그 차이끼리 견준다.

    **차이끼리 견주는 것이 요점이다.** 두 층의 저장본은 원래부터 다르다(직렬화기가 다르다).
    앞뒤의 차분을 보면 그 바탕 차이가 상쇄되고 액션이 한 일만 남는다.
    """
    # **잰 비결정만** 걸러 낸다. 시나리오가 이름을 적고 그 이유를 `note` 에 남긴다.
    # 거른 수를 판정에 실어 조용히 숨을 수 없게 한다.
    ignore = tuple(definition.get("ignorePaths", []))

    def keep(row: dict) -> bool:
        return not (ignore and str(row.get("path", "")).endswith(ignore))

    out = []
    for spec in definition.get("deltas", []):
        label = spec.get("label") or f"{spec['from']}→{spec['to']}"
        pair = {}
        for side, base in (("ocx", ocx_dir), ("rhwp", rhwp_dir)):
            names = definition.get("paths", {})
            rows = []
            for end in ("from", "to"):
                name = spec[end]
                variant = names.get(name, {})
                # 러너가 실제로 쓴 자리는 각자의 산출 폴더다 — 이름만 떼어 붙인다.
                rows.append(base / Path(str(variant.get("win") or variant.get("posix") or name)).name)
            pair[side] = ir_sweep(exe, rows[0], rows[1])
        if pair["ocx"] is None or pair["rhwp"] is None:
            out.append({"label": label, "verdict": "DELTA_MISSING",
                        "detail": "앞뒤 저장본이 없다"})
            continue
        errs = [f"{s}: {p['error']}" for s, p in pair.items() if p and "error" in p]
        if errs:
            out.append({"label": label, "verdict": "DELTA_ERROR", "detail": "; ".join(errs)})
            continue
        all_ocx = pair["ocx"].get("divergences", [])
        all_rhwp = pair["rhwp"].get("divergences", [])
        ocx_rows = {delta_key(r) for r in all_ocx if keep(r)}
        rhwp_rows = {delta_key(r) for r in all_rhwp if keep(r)}
        only_ocx = sorted(ocx_rows - rhwp_rows)
        only_rhwp = sorted(rhwp_rows - ocx_rows)
        out.append({
            "label": label,
            "verdict": "MATCH" if not only_ocx and not only_rhwp else "DELTA_DIFF",
            "ocxCount": len(ocx_rows),
            "rhwpCount": len(rhwp_rows),
            "ignored": (len(all_ocx) - len(ocx_rows)) + (len(all_rhwp) - len(rhwp_rows)),
            # 양쪽 다 아무 자취도 안 남겼으면 그 초록에는 뜻이 없다 — 따로 표시한다.
            "empty": not ocx_rows and not rhwp_rows,
            "onlyOcx": [f"{p}: {a} → {b}" for p, a, b in only_ocx[:20]],
            "onlyRhwp": [f"{p}: {a} → {b}" for p, a, b in only_rhwp[:20]],
        })
    return out


def compare_one(exe: Path, ocx_path: Path, rhwp_path: Path) -> dict:
    ocx = load(ocx_path)
    rhwp = load(rhwp_path)
    scenario_file = SCENARIO_DIR / f"{ocx['scenario']}.json"
    definition = load(scenario_file) if scenario_file.exists() else {}
    call_contracts = contracts(definition)
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
        code, detail = classify(o, r, call_contracts[i] if i < len(call_contracts) else {})
        rows.append({"index": i, "call": o.get("call"), "code": code, "detail": detail})

    counts: dict[str, int] = {}
    for row in rows:
        counts[row["code"]] = counts.get(row["code"], 0) + 1
    l3 = compare_saved(exe, ocx, rhwp)
    # 시나리오가 어떤 원장 항목을 검증하려 했는지. 원장은 **시나리오 단위로** 통과해야
    # 올라간다 — 반환값만 맞고 부작용이 없는 no-op 이 통과하는 구멍을 막는다.
    declared = definition.get("ledger", [])
    deltas = compare_deltas(exe, definition, ocx_path.parent, rhwp_path.parent)
    # **빈 자취는 통과가 아니다.** 양쪽이 나란히 아무 일도 안 해서 생긴 초록은 `MissingApi`
    # 를 일치로 세던 구멍과 같은 부류다 — 무엇을 검증했는지 말할 수 없다.
    deltas_ok = all(d.get("verdict") == "MATCH" and not d.get("empty") for d in deltas)
    return {
        "scenario": ocx["scenario"],
        "impl": rhwp.get("impl"),
        "oracle": ocx.get("oracle"),
        "ledger": declared,
        "l2": {"total": len(rows), "counts": counts, "rows": rows},
        "l3": l3,
        "l3Deltas": deltas,
        "pass": counts.get("MATCH", 0) == len(rows)
        and (l3 is None or l3["verdict"] == "MATCH")
        and deltas_ok,
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
        for d in rep.get("l3Deltas", []):
            mark = "빈 자취" if d.get("empty") else d["verdict"]
            skipped = f" · 거른 것 {d['ignored']}" if d.get("ignored") else ""
            print(f"      자취 {d['label']}: {mark}"
                  f" (오라클 {d.get('ocxCount', '-')} · rhwp {d.get('rhwpCount', '-')}{skipped})")
            for line in d.get("onlyOcx", [])[:5]:
                print(f"        오라클만: {line}")
            for line in d.get("onlyRhwp", [])[:5]:
                print(f"        rhwp만: {line}")
    print(f"→ {out_dir / 'verdict.tsv'}")
    return 0 if all(report["pass"] for report in reports) else 1


if __name__ == "__main__":
    raise SystemExit(main())
