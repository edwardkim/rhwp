"""gym 능력 리포트 — 한 바이너리/에이전트의 HWP 능력을 종합 스코어카드로 합친다.

## 왜 이 도구인가 (표준 계기)

지금까지 gym 의 조각은 흩어져 있었다: 점수(score.py)는 "얼마나 잘하나", 커버리지
(coverage.py)는 "무엇을 잴 수 있나", runner 신원은 "어느 바이너리가 냈나". 각각을
따로 봐서는 한 에이전트의 능력을 한 장으로 비교할 수 없다.

이 리포트는 넷을 **하나의 계기**로 합친다:

- **커버리지** — 에이전트-대면 능력 중 gym 이 잴 수 있는 비율(측정 폭).
- **정확도** — 전 pack 통과 점수(측정된 것 중 얼마나 통과).
- **축별 능력 프로파일** — 조사·편집·검증·보안·자동화 등 어느 차원이 강한가.
- **runner 신원** — 이 점수를 낸 바이너리(재현 기준). 다른 바이너리로 다시 돌리면
  같은 계기로 비교된다.

이것이 다른 에이전트가 자기 능력을 재고 겨루는 **표준 계기**다 — 커버리지·정확도를
뭉뚱그리지 않고(각각 다른 것을 잰다) 한 장에 정직하게 담는다.

## 사용

    python gym/report.py --bin target/debug/rhwp              # 전 pack 채점+커버리지→카드
    python gym/report.py --bin target/debug/rhwp --json       # 기계용 JSON
    python gym/report.py --scorecard sc.json --coverage cov.json  # 이미 있는 산출로 합성
    python gym/report.py --bin target/debug/rhwp --out report.md
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(HERE)


def axis_label(axis: str) -> str:
    """축 라벨 — 괄호 앞의 능력 차원(예: '편집 (표 좌표 지정)' → '편집')."""
    return (axis or "미분류").split(" (")[0].strip() or "미분류"


def compile_report(scorecard: dict, coverage: dict) -> dict:
    """순수 합성 — 바이너리·파일 접근 없음(가드가 픽스처로 시험 가능).

    scorecard: score.py 산출(kind gymScorecard). coverage: coverage.py 산출.
    """
    packs = scorecard.get("packs", [])
    total = scorecard.get("total", {})

    by_axis: dict[str, dict] = {}
    for p in packs:
        if p.get("status") != "scored":
            continue
        label = axis_label(p.get("axis", ""))
        acc = by_axis.setdefault(label, {"axis": label, "score": 0, "max": 0, "packs": 0})
        acc["score"] += p.get("score", 0)
        acc["max"] += p.get("max", 0)
        acc["packs"] += 1
    for acc in by_axis.values():
        acc["percent"] = (100 * acc["score"] // acc["max"]) if acc["max"] else 0
    axis_profile = sorted(by_axis.values(), key=lambda a: (-a["percent"], a["axis"]))

    unavailable = [p["id"] for p in packs if p.get("status") == "unavailable"]
    score_max = total.get("max", 0)
    score_pct = (100 * total.get("score", 0) // score_max) if score_max else 0

    return {
        "kind": "gymCapabilityReport",
        "schemaVersion": "1.0",
        "agent": scorecard.get("agent"),
        "runner": scorecard.get("runner"),
        # 두 축을 뭉뚱그리지 않는다 — 정확도(측정된 것 통과율)와 커버리지(측정 폭).
        "accuracy": {"score": total.get("score", 0), "max": score_max, "percent": score_pct},
        "coverage": {
            "percent": coverage.get("coveragePercent"),
            "covered": coverage.get("covered"),
            "agentFacingTotal": coverage.get("agentFacingTotal"),
            "uncoveredByCategory": coverage.get("uncoveredByCategory", {}),
        },
        "axisProfile": axis_profile,
        "packsScored": total.get("packsScored", 0),
        "packsUnavailable": unavailable,
    }


def render_card(report: dict) -> str:
    acc = report["accuracy"]
    cov = report["coverage"]
    lines = [
        "# gym 능력 스코어카드",
        "",
        f"- **정확도** (측정된 것 통과): {acc['score']}/{acc['max']} ({acc['percent']}%)",
    ]
    if cov.get("percent") is not None:
        lines.append(
            f"- **커버리지** (에이전트-대면 측정 폭): {cov['covered']}/{cov['agentFacingTotal']}"
            f" ({cov['percent']}%)"
        )
    lines.append(f"- **채점 pack**: {report['packsScored']}")
    if report["packsUnavailable"]:
        lines.append(f"- **미가용 pack**(명령 부재): {', '.join(report['packsUnavailable'])}")
    r = report.get("runner") or {}
    if r:
        lines.append(
            f"- **runner**: v{r.get('rhwpVersion')} · {str(r.get('rhwpCommit'))[:12]}"
        )
    lines += ["", "## 축별 능력 프로파일", "", "| 축 | 점수 | % |", "|---|---|---|"]
    for a in report["axisProfile"]:
        lines.append(f"| {a['axis']} | {a['score']}/{a['max']} | {a['percent']}% |")
    uncovered = cov.get("uncoveredByCategory") or {}
    if uncovered:
        flat = [n for names in uncovered.values() for n in names]
        lines += ["", f"## 미측정 능력 (다음 성장 방향, {len(flat)}개)", "",
                  "`" + "` · `".join(flat) + "`"]
    return "\n".join(lines) + "\n"


def _run(tool_argv: list[str]) -> None:
    # 하위 도구(build_baseline·score)의 진행 로그는 stderr 로 넘긴다 — report.py 의
    # stdout 은 카드/JSON 전용으로 순수하게 둬, --json 을 기계가 그대로 파싱할 수 있게.
    out = subprocess.run([sys.executable, *tool_argv], cwd=REPO_ROOT, capture_output=True)
    sys.stderr.write(out.stdout.decode("utf-8", "replace"))
    sys.stderr.write(out.stderr.decode("utf-8", "replace"))
    if out.returncode != 0:
        raise SystemExit(f"하위 도구 실패: {os.path.basename(str(tool_argv[0]))}")


def _from_bin(bin_path: str) -> tuple[dict, dict]:
    """--bin 모드: 전 pack 채점 + (있으면) 커버리지를 실제로 돌려 산출을 읽는다.

    커버리지 측정기(coverage.py)는 선택적이다 — 없으면 정확도·축 프로파일만 낸다.
    """
    _run([os.path.join(HERE, "tools", "build_baseline.py"), "--agent", "_report", "--bin", bin_path])
    _run([os.path.join(HERE, "score.py"), "--agent", "_report", "--bin", bin_path])
    scorecard = json.load(
        open(os.path.join(HERE, "submissions", "_report", "scorecard.json"), encoding="utf-8")
    )
    coverage: dict = {}
    cov_tool = os.path.join(HERE, "tools", "coverage.py")
    if os.path.isfile(cov_tool):
        try:
            cov_raw = subprocess.run(
                [sys.executable, cov_tool, "--bin", bin_path, "--json"],
                cwd=REPO_ROOT, capture_output=True,
            ).stdout
            coverage = json.loads(cov_raw)
        except (ValueError, OSError):
            coverage = {}
    return scorecard, coverage


def main() -> int:
    ap = argparse.ArgumentParser(description="gym 능력 종합 스코어카드")
    ap.add_argument("--bin", help="rhwp 바이너리 — 전 pack 채점+커버리지를 돌린다")
    ap.add_argument("--scorecard", help="score.py 스코어카드 JSON(이미 있으면)")
    ap.add_argument("--coverage", help="coverage.py --json 산출(이미 있으면)")
    ap.add_argument("--json", action="store_true", help="카드 대신 JSON")
    ap.add_argument("--out", help="출력 파일(생략 시 stdout)")
    a = ap.parse_args()

    if a.scorecard and a.coverage:
        scorecard = json.load(open(a.scorecard, encoding="utf-8"))
        coverage = json.load(open(a.coverage, encoding="utf-8"))
    elif a.bin:
        scorecard, coverage = _from_bin(a.bin)
    else:
        print("필수: --bin <경로> 또는 (--scorecard + --coverage)", file=sys.stderr)
        return 2

    report = compile_report(scorecard, coverage)
    text = (json.dumps(report, ensure_ascii=False, indent=2) + "\n") if a.json else render_card(report)
    if a.out:
        open(a.out, "w", encoding="utf-8").write(text)
        print(f"작성: {a.out}")
    else:
        sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
