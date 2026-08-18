"""[#4661] 릴리스 간 차등 회귀 — 두 바이너리로 같은 과제를 돌려 동작 변화를 잡는다.

## 착상

운동장 채점기는 정답을 박제하지 않고 채점 시점에 rhwp 로 재계산한다(#4653).
그러니 **같은 제출물을 두 바이너리로 채점하면 답이 같아야 한다** — 다르면 그
사이 릴리스에서 동작이 바뀐 것이다. #4658(교차형식 차등)이 검증한 원리를
형식축에서 시간축으로 돌리는 것뿐이다. 새 메커니즘 0.

리더보드 총점은 통과/실패의 이진값이라 둔감하다. 대신 각 과제 검사의 **관측값**
(봉투에서 길어낸 raw — 쪽수·표수·필드값·해시·판정 문자열)을 두 바이너리에서
뽑아 대조한다. 골든 없이, 관측이 갈리는 지점이 곧 회귀 후보다.

## 오검출 관문 (도구가 거짓말하지 않도록)

1. **명령 표면 대조** — 두 바이너리의 capabilities digest 가 같으면 관측 변화는
   순수 동작 회귀(regression). 다르면 표면이 바뀐 릴리스(surface-changed)로
   분류 — 의도된 변경일 수 있어 사람 판정 몫.
2. **판정성 종료 코드 허용** — exit 3(판정 데이터)은 실패가 아니다.
3. **비결정 관측 배제** — 파일 경로·산출 파일 크기처럼 릴리스와 무관하게
   흔들리는 자리는 대조에서 뺀다(파일 산출 과제의 file_exists 는 관측이 아니라
   존재 여부라 애초에 raw 비교 대상이 아니다).

분류·관측 동일성·보고 조립은 순수 함수라
`scripts/tests/test_gym_release_diff.py` 가 바이너리 없이 고정한다.

## 정직 조항

이 도구는 "무엇이 바뀌었나" 를 가리키지 "어느 쪽이 옳은가" 를 판정하지 않는다
(한컴 정답지 없음 — #4658 과 같은 결). 판정은 사람이 한다.

사용:
  python gym/tools/release_diff.py --old <구 바이너리> --new <신 바이너리>
                                   [--pack <id> ...] [-o 리포트.json]
"""

from __future__ import annotations

import argparse
import hashlib
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
from gym.core import checks as check_registry  # noqa: E402

ROOT = runner.ROOT

REPORT_KIND = "gymReleaseDiff"
SCHEMA_VERSION = "1.0"

#: 봉투를 부르지 않는 파일 연산자 — 관측이 아니라 존재/동일성이라 raw 대조 제외.
FILE_OPS = {"file_exists", "same_hash", "differs_from_input", "files_differ"}

CLASSIFICATIONS = ("stable", "regression", "surface-changed")
EXIT_BY_CLASS = {"stable": 0, "regression": 3, "surface-changed": 2}
CLASSIFICATION_REASON = {
    "stable": "명령 표면과 관측이 같다",
    "regression": "명령 표면은 같고 관측이 갈렸다 — 순수 동작 변화",
    "surface-changed": "명령 표면(capabilities digest)이 달라 사람 판정이 필요하다",
}


def capabilities_digest(bin_path):
    proc = subprocess.run([bin_path, "capabilities"], cwd=ROOT, capture_output=True)
    return hashlib.sha256(proc.stdout).hexdigest()


def surface_changed(old_digest, new_digest):
    """capabilities digest 가 다르면 표면이 바뀐 것이다. 순수."""
    return old_digest != new_digest


def classify(surface, divergences):
    """오검출 관문. 표면 변경이 회귀보다 앞선다.

    divergences 는 분기 목록·건수·bool 모두 받는다. 표면이 바뀌면 분기 유무와
    무관하게 surface-changed — 의도된 명령 추가를 회귀로 오신고하지 않는다.
    """
    if surface:
        return "surface-changed"
    if divergences:
        return "regression"
    return "stable"


def exit_for(classification):
    """stable=0, surface-changed=2(사람 판정), regression=3(회귀)."""
    return EXIT_BY_CLASS[classification]


def expected_exits(check):
    return check.get("expect_exits") or [check.get("expect_exit", 0)]


def should_observe(check):
    return check.get("op") not in FILE_OPS


def _values_equal(left, right):
    """숫자 6 과 6.0 은 같고, bool 은 int 로 접히지 않는다."""
    if left is right:
        return True
    if isinstance(left, bool) or isinstance(right, bool):
        return isinstance(left, bool) and isinstance(right, bool) and left is right
    if isinstance(left, (int, float)) and isinstance(right, (int, float)):
        return float(left) == float(right)
    if isinstance(left, str) and isinstance(right, str):
        return left == right
    if isinstance(left, list) and isinstance(right, list):
        return len(left) == len(right) and all(
            _values_equal(a, b) for a, b in zip(left, right)
        )
    if isinstance(left, dict) and isinstance(right, dict):
        if set(left) != set(right):
            return False
        return all(_values_equal(left[k], right[k]) for k in left)
    return left == right


def observations_equal(left, right):
    """관측 동일성. 종류가 다르면 값이 같아도 같지 않다."""
    return _values_equal(left, right)


def observation_display(obs):
    """사람이 읽는 한 칸. 값 관측은 raw, 그 외는 kind 또는 exitN."""
    if isinstance(obs, dict):
        kind = obs.get("kind")
        if kind == "value":
            return obs.get("value")
        if kind == "exit":
            return f"exit{obs.get('code')}"
        if kind:
            return kind
    return obs


def observation_from_result(code, env, head, check, dig_fn=None, find_cell_fn=None):
    """CLI 결과에서 대조 가능한 관측을 뽑는다. 순수.

    종료 코드·JSON 부재·경로 실패를 kind 로 가른다. 판정이 아니라 값이다.
    """
    if code not in expected_exits(check):
        return {"kind": "exit", "code": code, "head": (head or "")[:80]}
    if env is None:
        return {"kind": "nojson", "head": (head or "")[:80]}
    dig_fn = check_registry.dig if dig_fn is None else dig_fn
    try:
        val = dig_fn(env, check.get("path", ""))
    except (KeyError, IndexError, TypeError) as e:
        return {"kind": "digfail", "error": type(e).__name__}
    if check.get("op") == "cell_text_eq":
        find_cell_fn = check_registry.find_cell if find_cell_fn is None else find_cell_fn
        try:
            cell = find_cell_fn(val, check["table"], check["row"], check["col"])
        except (KeyError, IndexError, TypeError) as e:
            return {"kind": "digfail", "error": type(e).__name__}
        val = None if cell is None else cell.get("text")
    return {"kind": "value", "value": val}


def observe(bin_path, check, task, sub_dir):
    """한 검사의 관측값을 뽑는다 — 봉투의 지목된 자리(raw). 판정이 아니라 값."""
    cmd = check.get("cmd")
    if not cmd:
        return {"kind": "no-cmd"}
    try:
        args = runner.resolve_args(cmd, task, sub_dir)
    except (FileNotFoundError, OSError, KeyError, IndexError, TypeError) as e:
        # 제출물 부재도 구/신 양쪽에서 비교할 수 있는 관측 상태다. 여기서
        # 예외를 내면 legacy baseline이 비어 있는 한 차등 도구 전체가 멈춘다.
        return {"kind": "resolve-error", "error": type(e).__name__}
    code, env, head = runner.run_cli(bin_path, args)
    return observation_from_result(code, env, head, check)


def make_diff_row(task_id, check, old_obs, new_obs):
    return {
        "task": task_id,
        "check": check.get("name", check["op"]),
        "op": check["op"],
        "path": check.get("path", ""),
        "old": old_obs,
        "new": new_obs,
    }


def diff_task(old_bin, new_bin, task, sub_root, pack_id):
    sub_dir = os.path.join(sub_root, pack_id, task["id"])
    if not os.path.isdir(sub_dir):
        sub_dir = os.path.join(sub_root, task["id"])  # 평면 제출 호환
    rows = []
    for check in task.get("checks", []):
        if not should_observe(check):
            continue
        o = observe(old_bin, check, task, sub_dir)
        n = observe(new_bin, check, task, sub_dir)
        if not observations_equal(o, n):
            rows.append(make_diff_row(task["id"], check, o, n))
    return rows


def build_report(old_bin, old_digest, new_bin, new_digest,
                 tasks_compared, observations_compared, diffs,
                 observations_skipped=0):
    """릴리스 차등 JSON 봉투. 순수 — 바이너리를 부르지 않는다."""
    surface = surface_changed(old_digest, new_digest)
    classification = classify(surface, diffs)
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "old": {"bin": os.path.basename(old_bin), "capabilitiesSha256": old_digest},
        "new": {"bin": os.path.basename(new_bin), "capabilitiesSha256": new_digest},
        "surfaceChanged": surface,
        "tasksCompared": tasks_compared,
        "observationsCompared": observations_compared,
        "observationsSkipped": observations_skipped,
        "divergences": len(diffs),
        "classification": classification,
        "classificationReason": CLASSIFICATION_REASON[classification],
        "exit": exit_for(classification),
        "ok": classification == "stable",
        "reviewRequired": classification == "surface-changed",
        "diffs": list(diffs),
    }


def write_report(report, path):
    """UTF-8 · BOM 없음 · LF. 같은 입력이면 바이트가 같다."""
    with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(report, ensure_ascii=False, indent=2))
        fh.write("\n")


def render_summary(report, out_path):
    surface = report["surfaceChanged"]
    classification = report["classification"]
    lines = [
        f"과제 {report['tasksCompared']} · 관측 대조 {report['observationsCompared']}건",
        f"명령 표면(capabilities): {'다름 → surface-changed' if surface else '같음'}",
        f"관측 분기: {report['divergences']}건 → 분류 [{classification}]",
        f"이유: {report['classificationReason']}",
    ]
    for row in report.get("diffs", [])[:30]:
        ov = observation_display(row["old"])
        nv = observation_display(row["new"])
        pack = row.get("pack", "")
        loc = f"{pack}/{row['task']}" if pack else row["task"]
        lines.append(f"  {loc} · {row['check']}: {ov!r} → {nv!r}")
    lines.append(f"→ {out_path}")
    return lines


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--old", required=True, help="구 rhwp 바이너리 경로")
    ap.add_argument("--new", required=True, help="신 rhwp 바이너리 경로")
    ap.add_argument("--agent", default="claude-fable-5", help="관측에 쓸 제출물")
    ap.add_argument("--pack", action="append", default=None)
    ap.add_argument("-o", "--out", default=None)
    a = ap.parse_args()

    old_bin = runner.find_bin(a.old)
    new_bin = runner.find_bin(a.new)
    old_dig = capabilities_digest(old_bin)
    new_dig = capabilities_digest(new_bin)

    sub_root = os.path.join(runner.GYM, "submissions", a.agent)
    pack_ids = a.pack or runner.discover_packs()
    diffs, tasks_seen, checks_seen, skipped = [], 0, 0, 0
    for pack_id in pack_ids:
        _manifest, tasks = runner.load_pack(pack_id)
        for task in tasks:
            tasks_seen += 1
            for check in task.get("checks", []):
                if should_observe(check):
                    checks_seen += 1
                else:
                    skipped += 1
            for row in diff_task(old_bin, new_bin, task, sub_root, pack_id):
                row["pack"] = pack_id
                diffs.append(row)

    report = build_report(
        old_bin, old_dig, new_bin, new_dig,
        tasks_seen, checks_seen, diffs, observations_skipped=skipped,
    )
    out = a.out or os.path.join(runner.GYM, "release-diff.json")
    write_report(report, out)

    for line in render_summary(report, out):
        print(line)
    return report["exit"]


if __name__ == "__main__":
    sys.exit(main())
