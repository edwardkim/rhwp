"""[#4662] 릴리스 게이트 러너 — 회귀 도구를 파이프라인 판정으로 묶는다.

로컬에서도 CI 에서도 같은 판정을 낸다. 하는 일:

1. 두 바이너리로 릴리스 차등(#4661)을 돌려 분류를 얻는다.
2. 커밋된 리더보드가 있으면 해시 체인(#4659)을 검증한다.
3. 판정을 낸다 — **regression 만 차단**한다. surface-changed 는 리뷰 신호이지
   자동 차단이 아니다(도구는 '무엇이 바뀌었나'를 가리키지 '어느 쪽이 옳은가'를
   판정하지 않는다 — #4661 정직 조항).

## 종료 코드 (게이트 계약)

- 0 = pass   — 차등 stable(또는 검사 대상 없음) + 리더보드 무결
- 2 = review — 차등 surface-changed. 표면이 바뀐 릴리스라 사람 판정 필요(차단 아님)
- 3 = block  — 차등 regression, 또는 리더보드 체인 파손

## GitHub 연동

`--github-summary` 를 주면 GITHUB_STEP_SUMMARY 에 마크다운 표를 쓴다. old 바이너리가
없으면(직전 태그 미빌드) 차등은 건너뛰고 리더보드 검증만 한다 — 부재를 실패로
위장하지 않는 결 그대로 skipped 로 보고한다.
"""

import argparse
import io
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

from gym.core import runner  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))


def run_tool(script, args):
    """gym/tools 의 다른 러너를 서브프로세스로 부른다 — (exit, stdout)."""
    proc = subprocess.run([sys.executable, os.path.join(HERE, script)] + args,
                          cwd=runner.ROOT, capture_output=True)
    return proc.returncode, proc.stdout.decode("utf-8", errors="replace")


def gate(old_bin, new_bin, agent, packs, verify_board):
    verdict = {"kind": "gymReleaseGate", "schemaVersion": "1.0",
               "diff": None, "leaderboard": None}

    # 1) 릴리스 차등 — old 가 있어야 돈다.
    if old_bin and os.path.exists(runner.find_bin(old_bin)):
        out = os.path.join(runner.GYM, "release-gate-diff.json")
        args = ["--old", old_bin, "--new", new_bin, "--agent", agent, "-o", out]
        for p in packs or []:
            args += ["--pack", p]
        code, _ = run_tool("release_diff.py", args)
        with io.open(out, encoding="utf-8") as fh:
            report = json.load(fh)
        os.remove(out)
        verdict["diff"] = {
            "classification": report["classification"],
            "divergences": report["divergences"],
            "surfaceChanged": report["surfaceChanged"],
            "tasksCompared": report["tasksCompared"],
        }
    else:
        verdict["diff"] = {"classification": "skipped",
                           "reason": "구 바이너리 없음 — 차등 생략(직전 태그 미빌드)"}

    # 2) 리더보드 해시 체인 — 커밋된 원장이 있으면.
    if verify_board and os.path.exists(os.path.join(runner.GYM, "leaderboard", "ledger.ndjson")):
        # 게이트가 판정한 현재 바이너리와 같은 실행 파일로 서명·앵커를 검증한다.
        # 기본 탐색에 맡기면 CI/로컬의 target 배치에 따라 `rhwp`를 찾지 못해
        # 정상 원장을 파손으로 오판할 수 있다.
        code, out = run_tool("leaderboard.py", ["--bin", new_bin, "verify"])
        verdict["leaderboard"] = {"ok": code == 0, "exit": code}
    else:
        verdict["leaderboard"] = {"ok": None, "reason": "커밋된 리더보드 없음 — 검증 생략"}

    # 3) 판정 — regression 만 차단.
    cls = verdict["diff"]["classification"]
    board_bad = verdict["leaderboard"].get("ok") is False
    if cls == "regression" or board_bad:
        verdict["verdict"], verdict["exit"] = "block", 3
    elif cls == "surface-changed":
        verdict["verdict"], verdict["exit"] = "review", 2
    else:
        verdict["verdict"], verdict["exit"] = "pass", 0
    return verdict


def write_github_summary(verdict):
    path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not path:
        return
    d, b = verdict["diff"], verdict["leaderboard"]
    lines = ["## 운동장 릴리스 게이트", "",
             f"**판정: {verdict['verdict']}** (exit {verdict['exit']})", "",
             "| 검사 | 결과 |", "|---|---|",
             f"| 릴리스 차등 | {d['classification']}"
             + (f" · 분기 {d.get('divergences')}" if 'divergences' in d else f" ({d.get('reason', '')})")
             + " |",
             f"| 리더보드 체인 | "
             + ("무결" if b.get("ok") else ("파손!!" if b.get("ok") is False else b.get("reason", "")))
             + " |", ""]
    if d.get("classification") == "surface-changed":
        lines.append("> surface-changed 는 **차단이 아니라 리뷰 신호**다 — 명령 표면이 "
                     "바뀐 릴리스라 관측 변화가 의도된 것일 수 있다. 사람이 판정한다.")
    with io.open(path, "a", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--old", default=None, help="직전 릴리스 rhwp 바이너리(없으면 차등 생략)")
    ap.add_argument("--new", default=None, help="현재 rhwp 바이너리")
    ap.add_argument("--agent", default="claude-fable-5")
    ap.add_argument("--pack", action="append", default=None)
    ap.add_argument("--no-leaderboard", action="store_true", help="리더보드 검증 생략")
    ap.add_argument("--github-summary", action="store_true")
    ap.add_argument("-o", "--out", default=None)
    a = ap.parse_args()

    new_bin = runner.find_bin(a.new)
    verdict = gate(a.old, new_bin, a.agent, a.pack, not a.no_leaderboard)

    if a.out:
        with io.open(a.out, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(json.dumps(verdict, ensure_ascii=False, indent=2))
    if a.github_summary:
        write_github_summary(verdict)

    d = verdict["diff"]
    print(f"릴리스 차등: {d['classification']}"
          + (f" · 분기 {d.get('divergences')}" if "divergences" in d else ""))
    b = verdict["leaderboard"]
    print("리더보드 체인: "
          + ("무결" if b.get("ok") else ("파손" if b.get("ok") is False else "생략")))
    print(f"게이트 판정: [{verdict['verdict']}] (exit {verdict['exit']})")
    return verdict["exit"]


if __name__ == "__main__":
    sys.exit(main())
