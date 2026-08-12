"""rhwp 에이전트 짐 — 기계 채점기 진입점.

판정 논리와 채점 절차는 [#4653] 에서 `gym/core/` 로 옮겼다. 이 파일은 진입점과
하위 호환만 담당한다 — 기존 실행법이 그대로 동작해야 하기 때문이다.

사용:
  python gym/score.py --agent <이름> [--submissions gym/submissions/<이름>]
                      [--bin <rhwp 경로>] [--out <결과 폴더>]
                      [--pack <pack id> ...] [--profile <profile id>]

pack 을 고르지 않으면 전 pack 을 채점한다. 점수는 pack 별로 보존되며
(`scorecard.json` 의 `packs[]`), 총점은 편의값이다 — 어느 능력이 모자란지는
pack 별 점수가 말한다.
"""

import argparse
import io
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

from gym.core import runner  # noqa: E402
from gym.core.checks import (  # noqa: E402,F401  하위 호환 재수출
    deep_contains,
    dig,
    find_cell,
    norm,
    sha256_of,
)

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = runner.ROOT

# 구 API 재수출 — 기존 계약 테스트와 외부 스크립트가 이 이름들을 부른다.
find_bin = runner.find_bin
run_cli = runner.run_cli
resolve_args = runner.resolve_args
eval_check = runner.eval_check
score_task = runner.score_task


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--agent", required=True)
    ap.add_argument("--submissions", default=None)
    ap.add_argument("--bin", default=None)
    ap.add_argument("--out", default=None)
    ap.add_argument("--pack", action="append", default=None,
                    help="채점할 pack id (여러 번 지정 가능). 생략하면 전 pack")
    ap.add_argument("--profile", default=None, help="pack 묶음 프로파일 id")
    a = ap.parse_args()

    bin_path = find_bin(a.bin)
    sub_root = a.submissions or os.path.join(HERE, "submissions", a.agent)
    out_dir = a.out or sub_root
    os.makedirs(out_dir, exist_ok=True)

    card = runner.score_all(sub_root, bin_path, pack_ids=a.pack, profile_id=a.profile)
    card["agent"] = a.agent

    card_path = os.path.join(out_dir, "scorecard.json")
    with io.open(card_path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(card, ensure_ascii=False, indent=2))
    with io.open(os.path.join(out_dir, "report.md"), "w", encoding="utf-8",
                 newline="\n") as fh:
        fh.write(runner.render_report(card, a.agent))
    # [#4659] 입장 판정 봉투 — 리더보드 등재 사슬의 게이트 슬롯. 판정 기준은
    # "채점이 유효하게 완주했는가"(pack 1개 이상 채점)이지 만점 여부가 아니다 —
    # 리더보드는 낮은 점수도 순위이지, 입장 거부 사유가 아니다.
    admission = {
        "schemaVersion": "1.0", "kind": "gymAdmission", "agent": a.agent,
        "verdict": "allow" if card["total"]["packsScored"] >= 1 else "deny",
        "packsScored": card["total"]["packsScored"],
        "packsUnavailable": card["total"]["packsUnavailable"],
        "score": card["total"]["score"], "max": card["total"]["max"],
        "runner": card["runner"],
    }
    with io.open(os.path.join(out_dir, "admission.json"), "w", encoding="utf-8",
                 newline="\n") as fh:
        fh.write(json.dumps(admission, ensure_ascii=False, indent=2))

    total = card["total"]
    print(f"{a.agent}: {total['score']}/{total['max']}  "
          f"(pack {total['packsScored']} 채점"
          + (f", {total['packsUnavailable']} unavailable" if total["packsUnavailable"] else "")
          + f")  → {card_path}")
    for p in card["packs"]:
        if p["status"] == "unavailable":
            print(f"  - {p['id']:<18} unavailable (없는 명령: {', '.join(p['missingCommands'])})")
        else:
            print(f"  - {p['id']:<18} {p['score']}/{p['max']}  ({p['passed']}/{p['taskCount']} 과제)")
    return 0 if total["score"] == total["max"] else 3


if __name__ == "__main__":
    sys.exit(main())
