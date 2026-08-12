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

## 정직 조항

이 도구는 "무엇이 바뀌었나" 를 가리키지 "어느 쪽이 옳은가" 를 판정하지 않는다
(한컴 정답지 없음 — #4658 과 같은 결). 판정은 사람이 한다.

사용:
  python gym/tools/release_diff.py --old <구 바이너리> --new <신 바이너리>
                                   [--pack <id> ...] [-o 리포트.json]
"""

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

#: 봉투를 부르지 않는 파일 연산자 — 관측이 아니라 존재/동일성이라 raw 대조 제외.
FILE_OPS = {"file_exists", "same_hash", "differs_from_input", "files_differ"}


def capabilities_digest(bin_path):
    proc = subprocess.run([bin_path, "capabilities"], cwd=ROOT, capture_output=True)
    return hashlib.sha256(proc.stdout).hexdigest()


def observe(bin_path, check, task, sub_dir):
    """한 검사의 관측값을 뽑는다 — 봉투의 지목된 자리(raw). 판정이 아니라 값."""
    cmd = check.get("cmd")
    if not cmd:
        return {"kind": "no-cmd"}
    args = runner.resolve_args(cmd, task, sub_dir)
    code, env, head = runner.run_cli(bin_path, args)
    expect = check.get("expect_exits") or [check.get("expect_exit", 0)]
    if code not in expect:
        return {"kind": "exit", "code": code, "head": head[:80]}
    if env is None:
        return {"kind": "nojson", "head": head[:80]}
    try:
        val = check_registry.dig(env, check.get("path", ""))
    except (KeyError, IndexError, TypeError) as e:
        return {"kind": "digfail", "error": f"{type(e).__name__}"}
    # 표·목록은 통째 대조하면 잡음이 크다 — 지목 연산자는 그 좌표만 관측한다.
    if check["op"] == "cell_text_eq":
        cell = check_registry.find_cell(val, check["table"], check["row"], check["col"])
        val = None if cell is None else cell.get("text")
    return {"kind": "value", "value": val}


def diff_task(old_bin, new_bin, task, sub_root, pack_id):
    sub_dir = os.path.join(sub_root, pack_id, task["id"])
    if not os.path.isdir(sub_dir):
        sub_dir = os.path.join(sub_root, task["id"])  # 평면 제출 호환
    rows = []
    for check in task.get("checks", []):
        if check["op"] in FILE_OPS:
            continue
        o = observe(old_bin, check, task, sub_dir)
        n = observe(new_bin, check, task, sub_dir)
        if o != n:
            rows.append({"task": task["id"], "check": check.get("name", check["op"]),
                         "op": check["op"], "path": check.get("path", ""),
                         "old": o, "new": n})
    return rows


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
    surface_changed = old_dig != new_dig

    sub_root = os.path.join(runner.GYM, "submissions", a.agent)
    pack_ids = a.pack or runner.discover_packs()
    diffs, tasks_seen, checks_seen = [], 0, 0
    for pack_id in pack_ids:
        _manifest, tasks = runner.load_pack(pack_id)
        for task in tasks:
            tasks_seen += 1
            checks_seen += sum(1 for c in task.get("checks", []) if c["op"] not in FILE_OPS)
            for row in diff_task(old_bin, new_bin, task, sub_root, pack_id):
                row["pack"] = pack_id
                diffs.append(row)

    classification = ("surface-changed" if surface_changed
                      else ("regression" if diffs else "stable"))
    report = {
        "kind": "gymReleaseDiff", "schemaVersion": "1.0",
        "old": {"bin": os.path.basename(old_bin), "capabilitiesSha256": old_dig},
        "new": {"bin": os.path.basename(new_bin), "capabilitiesSha256": new_dig},
        "surfaceChanged": surface_changed,
        "tasksCompared": tasks_seen, "observationsCompared": checks_seen,
        "divergences": len(diffs), "classification": classification,
        "diffs": diffs,
    }
    out = a.out or os.path.join(runner.GYM, "release-diff.json")
    with io.open(out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(report, ensure_ascii=False, indent=2))

    print(f"과제 {tasks_seen} · 관측 대조 {checks_seen}건")
    print(f"명령 표면(capabilities): {'다름 → surface-changed' if surface_changed else '같음'}")
    print(f"관측 분기: {len(diffs)}건 → 분류 [{classification}]")
    for row in diffs[:30]:
        ov = row["old"].get("value", row["old"])
        nv = row["new"].get("value", row["new"])
        print(f"  {row['pack']}/{row['task']} · {row['check']}: {ov!r} → {nv!r}")
    print(f"→ {out}")
    # stable=0, regression=3(회귀), surface-changed=2(사람 판정 필요)
    return {"stable": 0, "regression": 3, "surface-changed": 2}[classification]


if __name__ == "__main__":
    sys.exit(main())
