"""3자 차등 대조 — COM(프록시) · 기안기(정본) · rhwp(구현)를 한 표에 놓는다.

오라클 이원화(계획서 §6.3.3·§9-6 — PR #4470)의 판정기다. 목적은 게이트가 아니라 **지도**다:

- 세 값이 같은 자리는 "COM 이 유효한 프록시"라는 증명이다 — 기존 `verified` 가 그대로 선다.
- 기안기와 rhwp 가 같고 COM 만 다른 자리(`COM_DRIFT`)는 프록시의 한계다 — rhwp 는 이미
  제품과 맞다.
- 기안기와 COM 이 같고 rhwp 만 다른 자리(`IMPL_GAP`)는 두 오라클이 함께 확인한 실 결함이다.
- COM 과 rhwp 가 같고 기안기만 다른 자리(`WEB_DIVERGES`)는 웹 계약이 COM 과 갈리는 지점이다
  — rhwp 는 프록시를 따라갔으니, 그 항목은 기안기 답으로 재검증해야 한다(기안기가 이긴다).

## 쓰임

    python tools/hwpctrl_compat/compare3.py \
        --ocx output/poc/hwpctrl/ocx --rhwp output/poc/hwpctrl/rhwp \
        --web output/poc/hwpctrl/webhwp --out output/poc/hwpctrl/verdict3

## 규율

- **버전 스탬프 없는 기안기 산출물은 거부한다**(exit 2). 스탬프(URL·측정 시각) 없는 결과는
  정답지 자격이 없다 — 데모의 버전이 곧 현장 버전이 아니기 때문이다(계획서 §6.3.3 — PR #4470).
- 이 도구는 게이트가 아니다. 판정은 데이터고 exit 0 으로 끝난다(입력 오류만 비영).
  저빈도 수동 측정용이며 CI 에 물리지 않는다.
"""

from __future__ import annotations

import argparse
import io
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

# 3자 판정 코드. 값 비교는 정규화된 반환값의 완전 일치다(compare.py 와 같은 잣대).
ALL_AGREE = "ALL_AGREE"
COM_DRIFT = "COM_DRIFT"
IMPL_GAP = "IMPL_GAP"
WEB_DIVERGES = "WEB_DIVERGES"
ALL_DIFFER = "ALL_DIFFER"


def load(path: Path) -> dict:
    with io.open(path, encoding="utf-8") as fh:
        return json.load(fh)


def require_stamp(web: dict, path: Path) -> None:
    oracle = web.get("oracle") or {}
    if not oracle.get("url") or not oracle.get("measuredAt"):
        raise SystemExit(f"기안기 산출물에 버전 스탬프가 없다 — 정답지 자격 없음: {path}")


def outcome(record: dict):
    """호출 하나의 관측 결과 — 값이면 ('value', 값), 죽었으면 ('error', 문구)."""
    if record.get("error") is not None:
        return ("error", str(record["error"]))
    return ("value", json.dumps(record.get("value"), ensure_ascii=False, sort_keys=True))


# 기안기 반환 중 **업로드 채널 부산물**은 계약 공통분모 밖이다. `Open` 은
# `{result, fileName, orgName, size}` 봉투를 주는데 `fileName` 은 서버가 매 업로드마다
# 새로 붙이는 난수라 값 비교가 원리적으로 성립하지 않는다. 성공 신호(`result`)만 세 구현의
# 공통분모로 비교하고, 봉투 전체는 returns.json 에 그대로 남는다 — 지운 것이 아니라 판정
# 잣대에서만 벗긴 것이다.
WEB_ENVELOPE_PROJECTIONS = {"Open": "result"}


def project_web(record: dict) -> dict:
    key = WEB_ENVELOPE_PROJECTIONS.get(record.get("call"))
    if key is None or record.get("error") is not None:
        return record
    value = record.get("value")
    if isinstance(value, dict) and key in value:
        return {**record, "value": value[key]}
    return record


def classify3(ocx: dict, web: dict, rhwp: dict) -> tuple[str, str]:
    """세 관측을 한 판정으로. 오류는 **종류 무관하게 '죽었다'**로만 묶는다 — 오류 문구는
    러너·플랫폼마다 달라 완전 일치를 요구하면 러너 차이가 판정을 오염시킨다."""
    o_kind, o_val = outcome(ocx)
    w_kind, w_val = outcome(web)
    r_kind, r_val = outcome(rhwp)

    def same(a_kind, a_val, b_kind, b_val):
        if a_kind != b_kind:
            return False
        return True if a_kind == "error" else a_val == b_val

    ow = same(o_kind, o_val, w_kind, w_val)
    wr = same(w_kind, w_val, r_kind, r_val)
    orr = same(o_kind, o_val, r_kind, r_val)
    detail = f"ocx={o_kind}:{o_val[:80]} web={w_kind}:{w_val[:80]} rhwp={r_kind}:{r_val[:80]}"
    if ow and wr:
        return ALL_AGREE, ""
    if wr and not ow:
        return COM_DRIFT, detail
    if ow and not wr:
        return IMPL_GAP, detail
    if orr and not ow:
        return WEB_DIVERGES, detail
    return ALL_DIFFER, detail


def compare_scenario(name: str, ocx: dict, web: dict, rhwp: dict) -> dict:
    rows = []
    n = max(len(ocx["calls"]), len(web["calls"]), len(rhwp["calls"]))
    absent = {"call": "(없음)", "error": "호출 없음"}
    for i in range(n):
        o = ocx["calls"][i] if i < len(ocx["calls"]) else absent
        w = web["calls"][i] if i < len(web["calls"]) else absent
        r = rhwp["calls"][i] if i < len(rhwp["calls"]) else absent
        names = {c.get("call") for c in (o, w, r)}
        if len(names) != 1:
            rows.append({"index": i, "call": "≠".join(sorted(str(x) for x in names)),
                         "code": ALL_DIFFER, "detail": "호출 순서가 어긋났다 — 러너 버그를 의심하라"})
            continue
        code, detail = classify3(o, project_web(w), r)
        rows.append({"index": i, "call": o.get("call"), "code": code, "detail": detail})
    counts: dict[str, int] = {}
    for row in rows:
        counts[row["code"]] = counts.get(row["code"], 0) + 1
    return {"scenario": name, "webOracle": web.get("oracle"), "counts": counts, "rows": rows}


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--ocx", required=True)
    ap.add_argument("--rhwp", required=True)
    ap.add_argument("--web", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--scenario", action="append", dest="scenarios")
    args = ap.parse_args()

    ocx_dir, rhwp_dir, web_dir, out_dir = map(Path, (args.ocx, args.rhwp, args.web, args.out))
    out_dir.mkdir(parents=True, exist_ok=True)

    # 기준 집합은 **기안기에서 실측된 것**이다 — 재지 않은 시나리오를 판정표에 올리지 않는다.
    web_paths = sorted(web_dir.glob("*.returns.json"))
    if args.scenarios:
        allowed = set(args.scenarios)
        web_paths = [p for p in web_paths if p.name.removesuffix(".returns.json") in allowed]
    if not web_paths:
        print("기안기 산출물 없음 — runner_webhwp.mjs 를 먼저 돌려라")
        return 2

    reports, skipped = [], []
    for web_path in web_paths:
        name = web_path.name.removesuffix(".returns.json")
        web = load(web_path)
        require_stamp(web, web_path)
        pair = {}
        for side, directory in (("ocx", ocx_dir), ("rhwp", rhwp_dir)):
            path = directory / web_path.name
            if not path.exists():
                skipped.append(f"{name}: {side} 산출물 없음")
                break
            pair[side] = load(path)
        else:
            reports.append(compare_scenario(name, pair["ocx"], web, pair["rhwp"]))

    lines = ["scenario\tindex\tcall\tcode\tdetail"]
    total: dict[str, int] = {}
    for rep in reports:
        for row in rep["rows"]:
            lines.append(f"{rep['scenario']}\t{row['index']}\t{row['call']}\t{row['code']}\t{row['detail']}")
        for code, count in rep["counts"].items():
            total[code] = total.get(code, 0) + count
    with io.open(out_dir / "verdict3.tsv", "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines) + "\n")
    with io.open(out_dir / "verdict3.json", "w", encoding="utf-8", newline="\n") as fh:
        json.dump({"schemaVersion": "1.0", "reports": reports, "skipped": skipped}, fh,
                  ensure_ascii=False, indent=2)
        fh.write("\n")

    calls = sum(total.values())
    print(f"시나리오 {len(reports)}건 · 호출 {calls}건")
    for code in (ALL_AGREE, COM_DRIFT, IMPL_GAP, WEB_DIVERGES, ALL_DIFFER):
        if total.get(code):
            print(f"  {code}: {total[code]}")
    for rep in reports:
        codes = ", ".join(f"{k} {v}" for k, v in sorted(rep["counts"].items()))
        print(f"  {rep['scenario']}: {codes}")
    if skipped:
        print("건너뜀:")
        for s in skipped:
            print(f"  {s}")
    print(f"→ {out_dir / 'verdict3.tsv'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
