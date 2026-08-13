"""외부 검증 축 — 우리가 진짜인가를 정직하게 측정한다 (#4728).

## 왜 필요한가 (경쟁자 시각)

경쟁자는 우리 시스템을 이렇게 친다: **"다 자기가 자기를 채점하는 닫힌 고리다.
gym 이 gym 을 채점하고, 표준을 만든 자가 표준을 준수하고, 리더보드가 스스로를
봉인한다. 외부의 독립 검증도, 실채택도 없다 — 방문객 없는 성이다."**

이 비판은 옳다. 그래서 그 공격면을 **축으로 세운다**: 내부 정합(가드가 통과)과
외부 채택(타자가 쓴다)을 **절대 뭉뚱그리지 않고 분리 측정**해, 우리가 어디에
서 있는지를 스스로 속이지 않는다. 비판을 지우는 게 아니라 계측기로 삼는다.

## 무엇을 분리하나

1. **프로젝트 외부 견인** — ★·fork·npm 다운로드·기여자. 이건 진짜다(edwardkim/rhwp
   은 실제 견인이 있다). 그러나 이건 **코어 제품**의 견인이지 이 세션이 세운
   메타-시스템의 것이 아니다.
2. **메타-시스템 외부 채택** — 표준 준수자·저장소 참조·제3자 재현. 정직하게 **0**.
   프로젝트 견인에 얹혀 있을 뿐 스스로 번 채택이 아니다.
3. **내부 정합** — 가드가 통과한다. 그러나 이건 **self-graded** 다 — 자기가 자기를
   검증한 것이라 외부 검증으로 치지 않는다.

정직한 결론: 내부 정합은 높고, 외부 채택은 낮다. 그 격차를 지우지 않고 드러내는
것이 이 축의 쓸모다.

사용:
  python tools/reality_check.py            # 스냅샷 기반 정직 채점(네트워크 불요)
  python tools/reality_check.py --json      # JSON
  python tools/reality_check.py --live      # gh·npm 으로 프로젝트 신호 갱신(네트워크)
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SIGNALS = REPO_ROOT / "mydocs" / "tech" / "agent_frame" / "external_signals.json"


def load():
    return json.loads(SIGNALS.read_text(encoding="utf-8"))


def scorecard(sig: dict) -> dict:
    proj = sig.get("project", {})
    meta = sig.get("metaSystem", {})
    meta_adopt = (meta.get("externalConformers", 0)
                  + meta.get("externalReferrers", 0)
                  + meta.get("thirdPartyReproductions", 0))
    return {
        "kind": "realityScorecard", "schemaVersion": "1.0",
        "measuredAt": sig.get("measuredAt"),
        "projectTraction": {
            "stars": proj.get("stars"), "forks": proj.get("forks"),
            "contributors": proj.get("contributors"),
            "npm": proj.get("npmMonthlyDownloads"),
            "note": "코어 제품의 견인 — 실재. 메타-시스템의 것이 아니다.",
        },
        "metaSystemExternalAdoption": {
            "total": meta_adopt,
            "conformers": meta.get("externalConformers", 0),
            "referrers": meta.get("externalReferrers", 0),
            "reproductions": meta.get("thirdPartyReproductions", 0),
            "note": meta.get("honestVerdict"),
        },
        "internalCoherence": {
            "note": "가드가 통과한다 — 그러나 self-graded. 외부 검증으로 치지 않는다.",
        },
        "verdict": (
            "메타-시스템 외부 채택 " + ("있음" if meta_adopt else "0 (self-graded)")
            + " · 프로젝트 견인은 실재 · 격차를 지우지 않고 계측한다"
        ),
        "criteria": sig.get("externalValidationCriteria", []),
    }


def refresh_live(sig: dict) -> dict:
    """gh·npm 으로 프로젝트 신호만 갱신한다(메타 채택은 손으로 실사한다 — 자동으로
    부풀리지 않는다)."""
    def gh(path, jq):
        try:
            out = subprocess.run(["gh", "api", path, "--jq", jq],
                                 capture_output=True, timeout=30)
            return json.loads(out.stdout.decode("utf-8")) if out.returncode == 0 else None
        except Exception:
            return None
    repo = sig["project"]["repo"]
    meta = gh(f"repos/{repo}", "{stars:.stargazers_count,forks:.forks_count,"
                               "watchers:.subscribers_count,openIssues:.open_issues_count}")
    if meta:
        sig["project"].update({"stars": meta.get("stars"), "forks": meta.get("forks"),
                               "watchers": meta.get("watchers"),
                               "openIssues": meta.get("openIssues")})
    n = gh(f"repos/{repo}/contributors?per_page=100", "length")
    if isinstance(n, int):
        sig["project"]["contributors"] = n
    return sig


def main() -> int:
    ap = argparse.ArgumentParser(description="외부 검증 축 — 정직한 현실 채점 (#4728)")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--live", action="store_true", help="gh·npm 으로 프로젝트 신호 갱신")
    a = ap.parse_args()

    if not SIGNALS.is_file():
        print(f"신호 스냅샷 없음: {SIGNALS.relative_to(REPO_ROOT)}")
        return 1
    sig = load()
    if a.live:
        sig = refresh_live(sig)
        SIGNALS.write_text(json.dumps(sig, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print("프로젝트 신호 갱신됨(메타 채택은 손으로 실사).")

    card = scorecard(sig)
    if a.json:
        sys.stdout.write(json.dumps(card, ensure_ascii=False, indent=2) + "\n")
        return 0
    p = card["projectTraction"]; m = card["metaSystemExternalAdoption"]
    print("현실 채점 — 내부 정합 ≠ 외부 채택")
    print(f"  프로젝트 견인(코어, 실재):  ★{p['stars']} · fork {p['forks']} · 기여자 {p['contributors']} · npm {p['npm']}")
    print(f"  메타-시스템 외부 채택:      {m['total']}  ({m['note']})")
    print(f"  내부 정합:                  가드 통과 — 단 self-graded")
    print(f"→ {card['verdict']}")
    print("  외부 검증으로 인정하는 것:")
    for c in card["criteria"]:
        print(f"    · {c}")
    return 0


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
