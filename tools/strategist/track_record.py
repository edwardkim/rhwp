#!/usr/bin/env python3
"""예측 대장 — 판단한 주체가 시간에 걸쳐 얼마나 맞혔는지 기계로 채점한다.

## 왜 있는가

[SWS/1.0](../../mydocs/tech/standards/strategy_work_standard.md) 의 SW-L4 는
예측형 주장에 확신도(`confidence`)와 판정 기한(`resolveBy`)을 요구한다. 그런데
**기한이 지난 뒤 실제로 채점하는 장치가 없으면 그 요구는 서류로 끝난다.**

이 도구가 그 자리다. 산출물에서 예측을 뽑아 누적 대장에 고정하고, 기한이 지난
것을 실제 결과로 채점해 **교정 점수(Brier)와 확신도 구간별 적중률**을 낸다.
개별 산출물이 검증 가능한가(SWS)의 위층 — **판단한 주체가 믿을 만한가**이다.

전략 컨설팅에서 이것이 없다는 사실이 이 도구의 존재 이유다. 어떤 컨설팅 회사도
자기 예측 적중률을 공개하지 않는다 — 공개할 형태로 기록하지 않기 때문이다.
기록이 기계로 채점되면 그 순간부터 실적은 주장이 아니라 계산이 된다.

## 게임 방지 설계 (이게 없으면 대장은 무의미하다)

1. **예측은 확정 시점에 해시된다.** id 는 (주장 텍스트 + 확신도 + 기한)의
   SHA-256 앞 12자다. 나중에 문구나 확신도를 고치면 **다른 id 가 되어** 원본이
   미판정으로 남는다 — 조용한 수정이 불가능하다.
2. **기한이 지난 미판정 예측은 감춰지지 않는다.** 유리한 것만 채점하고 불리한
   것을 방치하는 것이 가장 쉬운 조작이므로, `overdueUnresolved` 를 보고 맨 앞에
   싣고 **판정률(resolutionRate)** 을 Brier 점수와 항상 함께 낸다. 판정률이 낮은
   Brier 점수는 무의미하며 이 도구는 그렇게 말한다.
3. **표본이 적으면 교정을 주장하지 않는다.** 구간별 적중률은 표본 수를 함께
   싣고, 전체 판정 수가 `--min-sample`(기본 20) 미만이면 결론 대신 경고를 낸다.
4. **결과를 되돌려 쓸 수 없다.** 이미 `resolved` 인 항목의 결과를 바꾸려 하면
   거절한다(exit 3) — 정정이 필요하면 대장에 정정 이력을 남기고 새로 판정한다.

## 사용

    # 산출물의 예측을 대장에 고정 (여러 번 호출해도 같은 예측은 한 번만)
    python3 tools/strategist/track_record.py record deliverable.json --ledger tr.json

    # 기한이 지난 예측을 실제 결과로 채점
    python3 tools/strategist/track_record.py resolve <id> --outcome true \
        --evidence "2026-12-31 발표된 실제 수치 …" --ledger tr.json

    # 실적 보고 (Brier · 구간별 적중률 · 판정률 · 미판정 목록)
    python3 tools/strategist/track_record.py report --ledger tr.json [--json]

종료 코드: 0 = 정상 / 1 = 실행 실패 / 2 = 입력 오류 / 3 = 판정 거절
(이미 판정된 항목 덮어쓰기, 기한 전 판정 시도 등).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from datetime import date
from pathlib import Path


def log(msg: str) -> None:
    print(msg, file=sys.stderr)


def prediction_id(text: str, confidence: float, resolve_by: str) -> str:
    """예측의 신원 — 문구·확신도·기한 중 하나라도 바뀌면 다른 예측이다."""
    payload = f"{text.strip()}|{float(confidence):.4f}|{resolve_by}".encode("utf-8")
    return hashlib.sha256(payload).hexdigest()[:12]


def load(ledger: Path) -> dict:
    if ledger.is_file():
        return json.loads(ledger.read_text(encoding="utf-8"))
    return {
        "schemaVersion": "1",
        "standard": "SWS/1.0 SW-L4",
        "note": "예측은 확정 시점에 해시된다 — 문구·확신도·기한을 고치면 다른 예측이 된다.",
        "predictions": [],
    }


def save(ledger: Path, data: dict) -> None:
    ledger.parent.mkdir(parents=True, exist_ok=True)
    ledger.write_text(json.dumps(data, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")


def cmd_record(args) -> int:
    path = Path(args.deliverable)
    if not path.is_file():
        log(f"산출물이 없다: {path}")
        return 2
    d = json.loads(path.read_text(encoding="utf-8"))
    led = load(Path(args.ledger))
    known = {p["id"] for p in led["predictions"]}

    added, skipped_no_date = 0, 0
    for c in d.get("claims", []):
        rb = c.get("resolveBy")
        conf = c.get("confidence")
        if not rb:
            skipped_no_date += 1  # 예측이 아닌 주장 — 대장 대상이 아니다
            continue
        if not isinstance(conf, (int, float)) or not 0.0 <= float(conf) <= 1.0:
            log(f"확신도가 0~1 이 아니어서 건너뛴다: {c.get('id')} ({conf})")
            continue
        pid = prediction_id(c.get("text", ""), conf, rb)
        if pid in known:
            continue
        led["predictions"].append({
            "id": pid,
            "text": c.get("text", ""),
            "confidence": float(conf),
            "resolveBy": rb,
            "falsifier": c.get("falsifier"),
            "engagement": d.get("engagement"),
            "claimId": c.get("id"),
            "evidence": c.get("evidence", []),
            "status": "open",
            "outcome": None,
            "resolvedOn": None,
            "resolutionEvidence": None,
        })
        known.add(pid)
        added += 1

    save(Path(args.ledger), led)
    log(f"고정: 새 예측 {added}건 (기한 없는 주장 {skipped_no_date}건은 대상 아님)")
    print(json.dumps({"added": added, "total": len(led["predictions"])}, ensure_ascii=False))
    return 0


def cmd_resolve(args) -> int:
    led_path = Path(args.ledger)
    led = load(led_path)
    p = next((x for x in led["predictions"] if x["id"] == args.id), None)
    if p is None:
        log(f"대장에 없는 예측 id: {args.id}")
        return 2
    if p["status"] == "resolved":
        log(f"이미 판정됨({p['outcome']}) — 결과를 되돌려 쓸 수 없다. "
            "정정이 필요하면 새 예측으로 기록하고 이력을 남겨라.")
        return 3
    today = date.fromisoformat(args.today) if args.today else date.today()
    if date.fromisoformat(p["resolveBy"]) > today and not args.early:
        log(f"판정 기한({p['resolveBy']})이 아직 남았다 — 조기 판정은 --early 로 명시하라.")
        return 3
    if not (args.evidence or "").strip():
        log("판정 근거(--evidence)가 없다 — 근거 없는 채점은 대장을 오염시킨다.")
        return 2

    p["status"] = "resolved"
    p["outcome"] = args.outcome == "true"
    p["resolvedOn"] = today.isoformat()
    p["resolutionEvidence"] = args.evidence
    save(led_path, led)
    log(f"{p['id']} 판정: {p['outcome']} (확신도 {p['confidence']:.2f})")
    return 0


def brier(preds: list[dict]) -> float | None:
    """Brier 점수 — 낮을수록 좋다. 0.25 는 '전부 0.5 로 찍기'와 같다."""
    scored = [p for p in preds if p["status"] == "resolved"]
    if not scored:
        return None
    return sum((p["confidence"] - (1.0 if p["outcome"] else 0.0)) ** 2 for p in scored) / len(scored)


def calibration(preds: list[dict]) -> list[dict]:
    """확신도 구간별 실제 적중률 — 표본 수를 반드시 함께 낸다."""
    buckets = [(0.0, 0.2), (0.2, 0.4), (0.4, 0.6), (0.6, 0.8), (0.8, 1.01)]
    out = []
    scored = [p for p in preds if p["status"] == "resolved"]
    for lo, hi in buckets:
        inb = [p for p in scored if lo <= p["confidence"] < hi]
        if not inb:
            continue
        hits = sum(1 for p in inb if p["outcome"])
        out.append({
            "band": f"{lo:.1f}–{min(hi, 1.0):.1f}",
            "n": len(inb),
            "claimedMean": round(sum(p["confidence"] for p in inb) / len(inb), 3),
            "actualHitRate": round(hits / len(inb), 3),
        })
    return out


def cmd_report(args) -> int:
    led = load(Path(args.ledger))
    preds = led["predictions"]
    today = date.fromisoformat(args.today) if args.today else date.today()

    resolved = [p for p in preds if p["status"] == "resolved"]
    overdue = [p for p in preds if p["status"] == "open"
               and date.fromisoformat(p["resolveBy"]) <= today]
    pending = [p for p in preds if p["status"] == "open"
               and date.fromisoformat(p["resolveBy"]) > today]
    due_total = len(resolved) + len(overdue)
    resolution_rate = (len(resolved) / due_total) if due_total else None

    b = brier(resolved)
    enough = len(resolved) >= args.min_sample
    caveats = []
    if not enough:
        caveats.append(
            f"판정 표본 {len(resolved)}건 — {args.min_sample}건 미만이라 교정을 주장할 수 없다.")
    if resolution_rate is not None and resolution_rate < 0.9:
        caveats.append(
            f"판정률 {resolution_rate:.0%} — 기한이 지난 예측 {len(overdue)}건이 미판정이다. "
            "미판정을 남긴 채의 Brier 점수는 선택 편향으로 낮아진다.")

    report = {
        "ledger": str(args.ledger),
        "counts": {"total": len(preds), "resolved": len(resolved),
                   "overdueUnresolved": len(overdue), "notYetDue": len(pending)},
        "resolutionRate": round(resolution_rate, 3) if resolution_rate is not None else None,
        "brier": round(b, 4) if b is not None else None,
        "brierReference": {"allFiftyFifty": 0.25, "note": "낮을수록 좋다"},
        "calibration": calibration(preds),
        "caveats": caveats,
        "overdueUnresolvedIds": [p["id"] for p in overdue],
    }

    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=1))
    else:
        c = report["counts"]
        print(f"예측 대장 실적 — 총 {c['total']}건 "
              f"(판정 {c['resolved']} · 기한초과 미판정 {c['overdueUnresolved']} · 기한 전 {c['notYetDue']})")
        if resolution_rate is not None:
            print(f"판정률: {resolution_rate:.0%}")
        print(f"Brier: {report['brier'] if b is not None else '(판정 없음)'}"
              + ("  [0.25 = 전부 0.5 로 찍기]" if b is not None else ""))
        if report["calibration"]:
            print("확신도 구간별 실제 적중률:")
            for row in report["calibration"]:
                print(f"  {row['band']}  n={row['n']:<4} 주장 {row['claimedMean']:.2f} → 실제 {row['actualHitRate']:.2f}")
        for w in caveats:
            print(f"  ⚠ {w}")
        if overdue:
            print(f"  기한초과 미판정: {', '.join(p['id'] for p in overdue)}")
    return 0


def main(argv=None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--ledger", default="track_record.json", help="누적 예측 대장 경로")
    ap.add_argument("--today", help="기준일 (ISO, 테스트용)")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p_rec = sub.add_parser("record", help="산출물의 예측을 대장에 고정")
    p_rec.add_argument("deliverable")
    p_rec.set_defaults(func=cmd_record)

    p_res = sub.add_parser("resolve", help="기한이 지난 예측을 실제 결과로 채점")
    p_res.add_argument("id")
    p_res.add_argument("--outcome", choices=["true", "false"], required=True)
    p_res.add_argument("--evidence", required=True, help="판정 근거")
    p_res.add_argument("--early", action="store_true", help="기한 전 판정을 명시적으로 허용")
    p_res.set_defaults(func=cmd_resolve)

    p_rep = sub.add_parser("report", help="실적 보고")
    p_rep.add_argument("--json", action="store_true")
    p_rep.add_argument("--min-sample", type=int, default=20,
                       help="교정을 주장하려면 필요한 최소 판정 수 (기본 20)")
    p_rep.set_defaults(func=cmd_report)

    args = ap.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
