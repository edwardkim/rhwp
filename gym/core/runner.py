"""[#4653] pack 인지 채점 엔진.

원칙(1부부터 이어지는 것):
1) 정답을 골든 파일로 박제하지 않는다 — 기대값은 채점 시점에 rhwp 로 라이브
   재계산한다.
2) 산출물 과제는 제출 파일을 rhwp 로 재검증한다.
3) 표준 라이브러리 전용, Windows/리눅스 경로 안전, 실패도 데이터.

pack 확장이 더한 것:
4) **점수는 pack 별로 보존한다** — 하나의 거대한 만점으로 합치면 어느 능력이
   모자란지 사라진다. profile 은 pack 을 고르는 도구이지 점수를 뭉치는
   도구가 아니다.
5) **부재는 실패가 아니다** — pack 이 요구하는 명령이 바이너리에 없으면
   `unavailable` 로 보고한다.
6) **점수에는 신원이 붙는다** — 실행 바이너리의 version·commit·capabilities
   digest 를 스코어카드에 남기고 pack 의 기준 실행과 대조한다.
"""

import io
import json
import os
import subprocess

from . import checks as check_registry
from . import schema as pack_schema

HERE = os.path.dirname(os.path.abspath(__file__))
GYM = os.path.dirname(HERE)
ROOT = os.path.dirname(GYM)
PACKS_DIR = os.path.join(GYM, "packs")
PROFILES_DIR = os.path.join(GYM, "profiles")


def find_bin(cli_arg):
    """--bin > RHWP_BIN > target 기본값. 상대경로는 절대화한다 — Windows
    CreateProcess 는 자식 cwd 가 아니라 부모 cwd 기준으로 상대 실행파일을
    찾으므로, 절대화 없이는 WinError 2 로 전 과제가 무너진다(1호 주행 실측)."""
    cand = cli_arg or os.environ.get("RHWP_BIN")
    if cand:
        cand = cand.replace("/", os.sep)
        if os.path.isabs(cand):
            return cand
        for base in (os.getcwd(), ROOT):
            p = os.path.abspath(os.path.join(base, cand))
            if os.path.exists(p):
                return p
        return cand
    for rel in ("target/debug/rhwp.exe", "target/debug/rhwp",
                "target/release/rhwp.exe", "target/release/rhwp"):
        p = os.path.join(ROOT, rel.replace("/", os.sep))
        if os.path.exists(p):
            return p
    return "rhwp"


def run_cli(bin_path, args):
    """rhwp 실행 → (exit, 봉투 json 또는 None, stdout 원문 머리)."""
    proc = subprocess.run([bin_path] + args, cwd=ROOT, capture_output=True)
    out = proc.stdout.decode("utf-8", errors="replace")
    env = None
    if out.strip():
        try:
            env = json.loads(out)
        except ValueError:
            env = None
    return proc.returncode, env, out[:200]


def resolve_args(cmd, task, sub_dir):
    out = []
    for a in cmd:
        if a == "{input}":
            out.append(task["input"])
        elif a.startswith("{file:") and a.endswith("}"):
            out.append(os.path.join(sub_dir, a[6:-1]))
        elif a.startswith("{sha256:") and a.endswith("}"):
            # [#4600] 제출물의 해시를 채점 시점에 계산해 인자로 넘긴다 — 기대값을
            # 과제 파일에 박제하지 않고 rhwp 자신에게 재현 판정을 시키는 통로.
            out.append(check_registry.sha256_of(os.path.join(sub_dir, a[8:-1])))
        else:
            out.append(a)
    return out


class CheckContext:
    """연산자가 보는 세계 — 제출 폴더·과제·봉투."""

    def __init__(self, check, task, sub_dir, answer, envelope):
        self.check = check
        self.task = task
        self.sub_dir = sub_dir
        self.answer = answer
        self.envelope = envelope

    def sub_path(self, name):
        return os.path.join(self.sub_dir, name)

    def root_path(self, name):
        return os.path.join(ROOT, name)

    def dug(self):
        return check_registry.dig(self.envelope, self.check.get("path", ""))


def eval_check(check, task, sub_dir, answer, bin_path):
    op = check.get("op")
    detail = {"name": check.get("name", op), "op": op, "ok": False}
    entry = check_registry.REGISTRY.get(op)
    if entry is None:
        detail["error"] = f"미지 op: {op}"
        return detail
    fn, uses_cli = entry
    try:
        envelope = None
        if uses_cli:
            args = resolve_args(check["cmd"], task, sub_dir)
            code, envelope, head = run_cli(bin_path, args)
            expect_exits = check.get("expect_exits")
            if expect_exits is None:
                expect_exits = [check.get("expect_exit", 0)]
            if (not isinstance(expect_exits, list) or not expect_exits
                    or any(type(v) is not int for v in expect_exits)):
                detail["error"] = f"잘못된 expect_exits: {expect_exits!r}"
                return detail
            if code not in expect_exits:
                detail["error"] = f"exit {code} (허용 {expect_exits}): {head}"
                return detail
            if envelope is None:
                detail["error"] = f"봉투 파싱 실패: {head}"
                return detail
        detail.update(fn(CheckContext(check, task, sub_dir, answer, envelope)))
    except FileNotFoundError as e:
        detail["error"] = f"파일 없음: {e}"
    except (KeyError, IndexError, TypeError) as e:
        detail["error"] = f"경로 평가 실패: {type(e).__name__} {e}"
    return detail


def score_task(task, sub_root, bin_path):
    sub_dir = os.path.join(sub_root, task["id"])
    result = {"id": task["id"], "tier": task["tier"], "title": task["title"],
              "pass": False, "checks": []}
    if not os.path.isdir(sub_dir):
        result["error"] = "제출 폴더 없음"
        return result
    answer = {}
    ans_path = os.path.join(sub_dir, "answer.json")
    if os.path.exists(ans_path):
        try:
            with io.open(ans_path, encoding="utf-8") as fh:
                answer = json.load(fh)
        except ValueError as e:
            result["error"] = f"answer.json 파싱 실패: {e}"
            return result
    for check in task["checks"]:
        result["checks"].append(eval_check(check, task, sub_dir, answer, bin_path))
    result["pass"] = bool(result["checks"]) and all(c["ok"] for c in result["checks"])
    return result


def load_pack(pack_id):
    pack_dir = os.path.join(PACKS_DIR, pack_id)
    with io.open(os.path.join(pack_dir, "pack.json"), encoding="utf-8") as fh:
        manifest = json.load(fh)
    tasks = []
    tasks_dir = os.path.join(pack_dir, "tasks")
    for name in sorted(os.listdir(tasks_dir)):
        if name.endswith(".json"):
            with io.open(os.path.join(tasks_dir, name), encoding="utf-8") as fh:
                tasks.append(json.load(fh))
    return manifest, tasks


def discover_packs():
    if not os.path.isdir(PACKS_DIR):
        return []
    return sorted(d for d in os.listdir(PACKS_DIR)
                  if os.path.isfile(os.path.join(PACKS_DIR, d, "pack.json")))


def load_profile(profile_id):
    path = os.path.join(PROFILES_DIR, f"{profile_id}.json")
    with io.open(path, encoding="utf-8") as fh:
        return json.load(fh)


def score_pack(pack_id, sub_root, bin_path, available):
    """pack 하나 채점 — 요구 명령이 없으면 unavailable(0점 아님)."""
    manifest, tasks = load_pack(pack_id)
    maximum = sum(t["tier"] for t in tasks)
    missing = []
    if available is not None:
        missing = [c for c in manifest.get("requires", {}).get("commands", [])
                   if c not in available]
    entry = {"id": pack_id, "title": manifest["title"], "axis": manifest["axis"],
             "max": maximum, "taskCount": len(tasks)}
    if missing:
        # 부재를 실패로 위장하지 않는다 — 오래된 바이너리에게 "0점"은 거짓말이다.
        entry.update({"status": "unavailable", "missingCommands": missing,
                      "score": None, "tasks": []})
        return entry
    # pack 하위 제출 폴더를 우선 보고, 없으면 평면 제출(구 배치)로 되돌아간다.
    pack_sub = os.path.join(sub_root, pack_id)
    root_for_tasks = pack_sub if os.path.isdir(pack_sub) else sub_root
    results = [score_task(t, root_for_tasks, bin_path) for t in tasks]
    entry.update({"status": "scored",
                  "score": sum(r["tier"] for r in results if r["pass"]),
                  "passed": sum(1 for r in results if r["pass"]),
                  "tasks": results})
    return entry


def score_all(sub_root, bin_path, pack_ids=None, profile_id=None):
    available = pack_schema.known_commands(bin_path)
    if profile_id:
        pack_ids = load_profile(profile_id)["packs"]
    if not pack_ids:
        pack_ids = discover_packs()
    packs = [score_pack(pid, sub_root, bin_path, available) for pid in pack_ids]
    scored = [p for p in packs if p["status"] == "scored"]
    card = {
        "kind": "gymScorecard",
        "schemaVersion": "2.0",
        "profile": profile_id,
        "runner": pack_schema.runner_identity(bin_path, ROOT),
        # 총점은 편의값이다 — 능력 판독은 pack 별 점수로 한다(§4).
        "total": {"score": sum(p["score"] for p in scored),
                  "max": sum(p["max"] for p in scored),
                  "packsScored": len(scored),
                  "packsUnavailable": len(packs) - len(scored)},
        "packs": packs,
    }
    return card


def render_report(card, agent):
    total = card["total"]
    lines = [f"# 짐 스코어카드 — {agent}", "",
             f"**{total['score']} / {total['max']}** "
             f"(pack {total['packsScored']}개 채점"
             + (f" · {total['packsUnavailable']}개 unavailable" if total["packsUnavailable"] else "")
             + ")", ""]
    r = card["runner"]
    lines += [f"실행 신원: rhwp {r['rhwpVersion']} · commit `{r['rhwpCommit'][:12]}` "
              f"· capabilities `{r['capabilitiesSha256'][:12]}`", "",
              "| pack | 능력 축 | 점수 | 과제 |", "|---|---|---|---|"]
    for p in card["packs"]:
        if p["status"] == "unavailable":
            lines.append(f"| {p['id']} | {p['axis']} | unavailable | 요구 명령 없음: "
                         f"{', '.join(p['missingCommands'])} |")
        else:
            lines.append(f"| {p['id']} | {p['axis']} | **{p['score']} / {p['max']}** | "
                         f"{p['passed']}/{p['taskCount']} 통과 |")
    for p in card["packs"]:
        if p["status"] != "scored":
            continue
        lines += ["", f"## {p['id']} — {p['title']}", "",
                  "| 과제 | 티어 | 판정 | 세부 |", "|---|---|---|---|"]
        for t in p["tasks"]:
            if "error" in t:
                det = t["error"]
            else:
                det = " · ".join(("O" if c["ok"] else "X") + " " + str(c["name"])
                                 for c in t["checks"])
            lines.append(f"| {t['id']} {t['title']} | {t['tier']} | "
                         f"{'통과' if t['pass'] else '실패'} | {det} |")
    lines += ["", "채점기: gym/core/runner.py (라이브 오라클 · pack 별 점수 보존)"]
    return "\n".join(lines)
