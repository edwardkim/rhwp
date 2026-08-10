"""캐럿 **위치**를 한글 오라클과 rhwp 로 견준다 — 폰트별로.

## 무엇을 재나

한글은 픽셀 캐럿 API 가 없다(`GetPosBySet` 은 논리 List/Para/Pos 뿐). 유일한 캐럿-x 오라클은
`KeyIndicator` 의 `pos` 인데, **캐럿 x 를 어떤 단위로 나눠 반올림한 값**이다(계획서 §4.67).
그래서 rhwp 의 정밀한 캐럿 x(문서 좌표, 줌 무관)를 같은 단위로 나눠 반올림하면 오라클 pos 와
같아야 한다 — 그러면 글자 advance 가 한글과 일치하는 것이다.

단위는 표본에서 **최소제곱으로 맞춘다**(폰트 크기마다 다르다). 맞춘 뒤 모든 오프셋에서
`round(scaled)` 가 오라클 pos 와 같은지 본다. 어긋나는 오프셋이 그 폰트의 metric 차이다.

## 왜 오프셋이 밀리나

본문 첫 문단은 **앞머리 자리차지**로 캐럿이 0 이 아니라 그 뒤에서 시작한다(`SetPos(0,0,0)` 이
스냅되는 자리). 그래서 `오라클 getpos = rhwp char_offset + leading` 로 정렬한다. leading 은
`SetPos(0,0,0)→GetPos` 로 잰다.

## 쓰임

    python tools/hwpctrl_compat/caret_compare.py samples/re-font-batang-hancom.hwp
    python tools/hwpctrl_compat/caret_compare.py --all   # re-font-*-hancom 전부
"""

from __future__ import annotations

import argparse
import glob
import io
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
EXE = REPO / "target" / ("release" if (REPO / "target/release/rhwp.exe").exists() else "debug") / "rhwp.exe"
OCX_DIR = REPO / "output" / "poc" / "hwpctrl" / "ocx"


def oracle_caret(sample: str, leading: int, count: int, timeout: int) -> dict[int, int]:
    """오라클에서 오프셋별 `KeyIndicator.pos` 를 받는다. 반환: {getpos → pos}."""
    sid = "zz-caret-" + Path(sample).stem
    calls = []
    for off in range(leading, leading + count):
        calls += [["SetPos", [0, 0, off]], ["GetPos", []], ["KeyIndicator", []]]
    spec = {"id": sid, "title": "캐럿 x 오라클", "ledger": [], "open": sample, "calls": calls}
    probe = OCX_DIR.parent / "caret_probes"; probe.mkdir(parents=True, exist_ok=True); probe = probe / f"{sid}.json"
    probe.write_text(json.dumps(spec, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    proc = subprocess.run(
        [sys.executable, str(HERE / "runner_ocx.py"), str(probe), "--out", str(OCX_DIR)],
        timeout=timeout, check=False,
    )
    if proc.returncode != 0:
        raise SystemExit(f"오라클 실행 실패: {sample}")
    data = json.load(io.open(OCX_DIR / f"{sid}.returns.json", encoding="utf-8"))
    out, gp = {}, None
    for call in data["calls"]:
        if call["call"] == "GetPos":
            gp = call["value"]["pos"]
        elif call["call"] == "KeyIndicator" and gp is not None:
            out[gp] = call["value"]["pos"]
    return out


def leading_anchor(sample: str, timeout: int) -> int:
    """`SetPos(0,0,0)` 이 스냅되는 자리 = 앞머리 자리차지 뒤 첫 캐럿."""
    sid = "zz-lead-" + Path(sample).stem
    spec = {"id": sid, "title": "앞머리", "ledger": [], "open": sample,
            "calls": [["SetPos", [0, 0, 0]], ["GetPos", []]]}
    probe = OCX_DIR.parent / "caret_probes"; probe.mkdir(parents=True, exist_ok=True); probe = probe / f"{sid}.json"
    probe.write_text(json.dumps(spec, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    subprocess.run(
        [sys.executable, str(HERE / "runner_ocx.py"), str(probe), "--out", str(OCX_DIR)],
        timeout=timeout, check=False,
    )
    data = json.load(io.open(OCX_DIR / f"{sid}.returns.json", encoding="utf-8"))
    for call in data["calls"]:
        if call["call"] == "GetPos":
            return int(call["value"]["pos"])
    return 0


def rhwp_carets(sample: str) -> dict[int, float]:
    out = subprocess.run(
        [str(EXE), "dump-carets", sample, "-p", "0", "--json"], capture_output=True, check=False
    )
    carets = json.loads(out.stdout.decode("utf-8", "replace"))["carets"]
    return {c["offset"]: c["x"] for c in carets}


def fit_unit(pairs: list[tuple[float, int]]) -> tuple[float, float]:
    """오라클 pos ≈ (x - x0)/unit + 1 를 최소제곱으로 맞춘다 → (x0, unit)."""
    xs = [x for x, _ in pairs]
    ps = [p for _, p in pairs]
    x0 = xs[0]
    # p-1 = (x-x0)/unit → unit = Σ(x-x0)(p-1) / Σ(p-1)^2
    num = sum((x - x0) * (p - 1) for x, p in pairs)
    den = sum((p - 1) ** 2 for _, p in pairs) or 1.0
    unit = num / den
    return x0, unit if unit else 1.0


def compare(sample: str, count: int, timeout: int) -> dict:
    rel = str(Path(sample).relative_to(REPO)) if Path(sample).is_absolute() else sample
    lead = leading_anchor(rel, timeout)
    opos = oracle_caret(rel, lead, count, timeout)
    rx = rhwp_carets(rel)

    pairs = []
    for off in range(count):
        gp = off + lead
        if off in rx and gp in opos:
            pairs.append((rx[off], opos[gp]))
    if len(pairs) < 4:
        return {"sample": rel, "error": "정렬 표본 부족", "leading": lead}

    x0, unit = fit_unit(pairs)
    worst = 0.0
    mism = 0
    rows = []
    for off in range(count):
        gp = off + lead
        if off not in rx or gp not in opos:
            continue
        scaled = (rx[off] - x0) / unit + 1
        dev = abs(scaled - opos[gp])
        # 판정은 **반올림 일치**다 — 오라클 pos 자체가 round(x/unit) 이라, rhwp 를 같은 단위로
        # 반올림해 같은 pos 가 나오면 글자 advance 가 맞은 것이다. 연속 편차(dev)는 진단용이고
        # X.50 같은 경계값은 어긋남이 아니다.
        matched = round(scaled) == opos[gp]
        mism += not matched
        worst = max(worst, dev)
        rows.append((off, rx[off], opos[gp], round(scaled, 2), round(dev, 2), matched))
    return {"sample": rel, "leading": lead, "unit": round(unit, 3),
            "worstDev": round(worst, 3), "mismatch": mism, "n": len(rows), "rows": rows}


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("sample", nargs="?", help="비교할 .hwp")
    ap.add_argument("--all", action="store_true", help="samples/re-font-*-hancom.hwp 전부")
    ap.add_argument("--count", type=int, default=22, help="오프셋 개수")
    ap.add_argument("--timeout", type=int, default=200)
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()

    if args.all:
        samples = sorted(glob.glob(str(REPO / "samples" / "re-font-*-hancom.hwp")))
        samples = [s for s in samples if "empty" not in s]
    elif args.sample:
        samples = [args.sample]
    else:
        ap.error("샘플을 지정하거나 --all 을 쓰라")

    print(f"{'폰트':<32} {'단위':>7} {'최대편차':>8} {'판정'}")
    fails = 0
    for s in samples:
        res = compare(s, args.count, args.timeout)
        if "error" in res:
            print(f"{Path(s).stem:<32} {'—':>7} {'—':>8} {res['error']}")
            continue
        # 판정 = 반올림 불일치 개수. 0 이면 그 폰트의 캐럿 위치가 한글과 같다.
        ok = res["mismatch"] == 0
        fails += not ok
        verdict = "일치" if ok else f"**{res['mismatch']}개 어긋남**"
        print(f"{Path(s).stem:<32} {res['unit']:>7} {res['worstDev']:>8} {verdict} (n={res['n']})")
        if args.verbose or not ok:
            for off, x, op, sc, dev, matched in res["rows"]:
                mark = "" if matched else " <<< 어긋남"
                print(f"    off{off:>3} rhwp_x={x:7.1f} oracle_pos={op:>3} scaled={sc:6.2f} dev={dev}{mark}")
    print(f"\n{len(samples)}개 중 {fails}개 어긋남")
    return 1 if fails else 0


if __name__ == "__main__":
    raise SystemExit(main())
