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

예외 경로(#5260): 새 플래그는 없다. 채점·기록 실패는 스코어카드의
`exceptions` / pack `status=error` 와 입장 봉투 `packsErrored` 로 남긴다.
종료 코드도 예전과 같다 — 만점 0, 그 외 3.
"""

from __future__ import annotations

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
ScoreRunnerError = runner.ScoreRunnerError
exception_kind = runner.exception_kind
admission_from_card = runner.admission_from_card

SCORECARD_NAME = "scorecard.json"
REPORT_NAME = "report.md"
ADMISSION_NAME = "admission.json"


def build_parser():
    """기존 플래그만. 새 인자를 붙이지 않는다."""
    ap = argparse.ArgumentParser()
    ap.add_argument("--agent", required=True)
    ap.add_argument("--submissions", default=None)
    ap.add_argument("--bin", default=None)
    ap.add_argument("--out", default=None)
    ap.add_argument("--pack", action="append", default=None,
                    help="채점할 pack id (여러 번 지정 가능). 생략하면 전 pack")
    ap.add_argument("--profile", default=None, help="pack 묶음 프로파일 id")
    return ap


def normalize_agent(name):
    """agent 이름은 비면 안 된다. 새 플래그가 아니라 기존 --agent 검증."""
    if not isinstance(name, str):
        raise runner.ScoreRunnerError("empty-agent", "agent 가 문자열이 아니다")
    stripped = name.strip()
    if not stripped:
        raise runner.ScoreRunnerError("empty-agent", "agent 가 비었다")
    if any(ch in stripped for ch in ("\x00", "/", "\\")):
        raise runner.ScoreRunnerError("unsafe-id", f"agent 가 안전하지 않다: {name!r}")
    return stripped


def resolve_paths(agent, submissions, out):
    sub_root = submissions or os.path.join(HERE, "submissions", agent)
    out_dir = out or sub_root
    return sub_root, out_dir


def ensure_out_dir(out_dir):
    try:
        os.makedirs(out_dir, exist_ok=True)
    except OSError as e:
        raise runner.ScoreRunnerError("write-error", f"산출 폴더를 만들지 못했다: {e}")
    return out_dir


def dump_json(path, payload):
    text = json.dumps(payload, ensure_ascii=False, indent=2)
    with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    return path


def write_text(path, text):
    with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    return path


def write_score_artifacts(out_dir, card, agent):
    """scorecard.json · report.md · admission.json. 부분 실패도 목록으로."""
    written = []
    errors = []
    card_path = os.path.join(out_dir, SCORECARD_NAME)
    try:
        dump_json(card_path, card)
        written.append(card_path)
    except OSError as e:
        errors.append(runner.exception_row(
            "write-error", where="scorecard", message=str(e)))
        card_path = None
    report_path = os.path.join(out_dir, REPORT_NAME)
    try:
        write_text(report_path, runner.render_report(card, agent))
        written.append(report_path)
    except OSError as e:
        errors.append(runner.exception_row(
            "write-error", where="report", message=str(e)))
    admission = runner.admission_from_card(card, agent)
    admission_path = os.path.join(out_dir, ADMISSION_NAME)
    try:
        dump_json(admission_path, admission)
        written.append(admission_path)
    except OSError as e:
        errors.append(runner.exception_row(
            "write-error", where="admission", message=str(e)))
    return {
        "cardPath": card_path,
        "written": written,
        "errors": errors,
        "admission": admission,
    }


def deny_card(agent, bin_path, exc, profile_id=None):
    """채점 자체가 시작 전에 죽은 자리 — 빈 카드 + 예외 한 줄."""
    if isinstance(exc, runner.ScoreRunnerError):
        row = exc.as_row("main")
    else:
        row = runner.exception_row(
            runner.exception_kind(exc, "write"),
            where="main",
            message=runner.error_head(exc),
        )
    card = runner.empty_scorecard(
        profile_id=profile_id,
        bin_path=bin_path or "",
        runner=runner.safe_runner_identity(bin_path) if bin_path else None,
        exceptions=[row],
    )
    card["agent"] = agent
    return runner.attach_card_counts(card)


def run_score(agent, submissions=None, bin_arg=None, out=None,
              pack_ids=None, profile_id=None):
    """채점 한 번. 종료 코드를 돌려준다. argparse 와 분리해 시험한다."""
    agent = normalize_agent(agent)
    bin_path = find_bin(bin_arg)
    sub_root, out_dir = resolve_paths(agent, submissions, out)
    ensure_out_dir(out_dir)
    try:
        card = runner.score_all(sub_root, bin_path, pack_ids=pack_ids,
                                profile_id=profile_id)
    except runner.FATAL_EXCEPTIONS:
        raise
    except (runner.ScoreRunnerError,) + runner.CATCHABLE_EXCEPTIONS as e:
        card = deny_card(agent, bin_path, e, profile_id)
    card["agent"] = agent
    artifacts = write_score_artifacts(out_dir, card, agent)
    for row in artifacts["errors"]:
        card.setdefault("exceptions", []).append(row)
    runner.attach_card_counts(card)
    card_path = artifacts["cardPath"] or os.path.join(out_dir, SCORECARD_NAME)
    print(runner.format_console_summary(card, agent, card_path))
    return runner.exit_from_card(card), card, artifacts


def main(argv=None):
    ap = build_parser()
    a = ap.parse_args(argv)
    try:
        code, _card, _art = run_score(
            a.agent,
            submissions=a.submissions,
            bin_arg=a.bin,
            out=a.out,
            pack_ids=a.pack,
            profile_id=a.profile,
        )
        return code
    except runner.FATAL_EXCEPTIONS:
        raise
    except runner.ScoreRunnerError as e:
        print(f"score: {e.kind}: {e.message}", file=sys.stderr)
        return runner.EXIT_IMPERFECT
    except runner.CATCHABLE_EXCEPTIONS as e:
        print(f"score: {type(e).__name__}: {e}", file=sys.stderr)
        return runner.EXIT_IMPERFECT


if __name__ == "__main__":
    sys.exit(main())
