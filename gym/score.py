"""rhwp 에이전트 짐 — 기계 채점기.

원칙:
1) 정답을 골든 파일로 박제하지 않는다 — 모든 기대값은 채점 시점에 rhwp 로
   **라이브 재계산**한다(오라클 부패 없음, 픽스처가 바뀌면 기대값도 따라간다).
2) 산출물 과제는 제출 파일을 rhwp 로 재검증한다(검색·재조회·해시).
3) 표준 라이브러리 전용, Windows/리눅스 경로 안전, 실패도 데이터(스코어카드에
   있는 그대로 남는다).

사용:
  python gym/score.py --agent <이름> [--submissions gym/submissions/<이름>]
                      [--bin <rhwp 경로>] [--out <결과 폴더>]
"""

import argparse
import hashlib
import io
import json
import os
import subprocess
import sys

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)


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


def dig(value, path):
    """점 경로 평가: 'a.b[2].c'. 빈 경로면 전체."""
    if not path:
        return value
    cur = value
    for part in path.split("."):
        while "[" in part:
            name, rest = part.split("[", 1)
            idx, part_tail = rest.split("]", 1)
            if name:
                cur = cur[name]
            cur = cur[int(idx)]
            part = part_tail.lstrip(".") if part_tail else ""
            if not part:
                break
        if part:
            cur = cur[part]
    return cur


def deep_contains(value, needle):
    if isinstance(value, str):
        return needle in value
    if isinstance(value, dict):
        return any(deep_contains(v, needle) for v in value.values())
    if isinstance(value, list):
        return any(deep_contains(v, needle) for v in value)
    return False


def sha256_of(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def norm(v):
    """비교 정규화 — 숫자 문자열과 숫자를 같게 본다."""
    if isinstance(v, bool):
        return v
    if isinstance(v, (int, float)):
        return float(v)
    if isinstance(v, str):
        s = v.strip()
        try:
            return float(s)
        except ValueError:
            return s
    return v


def resolve_args(cmd, task, sub_dir):
    out = []
    for a in cmd:
        if a == "{input}":
            out.append(task["input"])
        elif a.startswith("{file:") and a.endswith("}"):
            out.append(os.path.join(sub_dir, a[6:-1]))
        elif a.startswith("{sha256:") and a.endswith("}"):
            # [#4600] 제출물의 해시를 채점 시점에 계산해 인자로 넘긴다 — 기대값을
            # 과제 파일에 박제하지 않고 rhwp 자신에게 재현 판정을 시키기 위한 통로.
            out.append(sha256_of(os.path.join(sub_dir, a[8:-1])))
        else:
            out.append(a)
    return out


def find_cell(tables, table_index, row, col):
    """[#4600] 표 좌표로 셀을 지목한다.

    `cells[0]` 같은 순서 가정 대신 (row, col) 로 찾는다 — 순서 가정은 내보내기
    구현이 바뀌면 조용히 엉뚱한 셀을 검사하게 되고, 그것이 이 이슈가 잡은
    오검출과 같은 부류의 결함이다.
    """
    table = tables[table_index]
    for cell in table["cells"]:
        if cell.get("row") == row and cell.get("col") == col:
            return cell
    return None


def eval_check(check, task, sub_dir, answer, bin_path):
    op = check["op"]
    detail = {"name": check.get("name", op), "op": op, "ok": False}
    try:
        if op == "same_hash":
            files = [os.path.join(sub_dir, f) for f in check["files"]]
            hashes = [sha256_of(f) for f in files]
            detail["expected"] = hashes[0][:16]
            detail["actual"] = hashes[1][:16]
            detail["ok"] = len(set(hashes)) == 1
            return detail
        if op == "differs_from_input":
            # [#4600] 무편집 복사본 거부 — 산출물이 과제 입력과 바이트가 같으면
            # 아무 작업도 하지 않은 것이다. 동일성만 보는 검사(same_hash)는
            # 입력을 두 번 복사한 제출을 통과시켰다.
            submitted = sha256_of(os.path.join(sub_dir, check["file"]))
            source = sha256_of(os.path.join(ROOT, task["input"]))
            detail["expected"] = f"!= {source[:16]} (과제 입력)"
            detail["actual"] = submitted[:16]
            detail["ok"] = submitted != source
            return detail
        args = resolve_args(check["cmd"], task, sub_dir)
        code, env, head = run_cli(bin_path, args)
        expect_exits = check.get("expect_exits")
        if expect_exits is None:
            expect_exits = [check.get("expect_exit", 0)]
        if (not isinstance(expect_exits, list) or not expect_exits
                or any(type(value) is not int for value in expect_exits)):
            detail["error"] = f"잘못된 expect_exits: {expect_exits!r}"
            return detail
        if code not in expect_exits:
            detail["error"] = f"exit {code} (허용 {expect_exits}): {head}"
            return detail
        if env is None:
            detail["error"] = f"봉투 파싱 실패: {head}"
            return detail
        got = dig(env, check.get("path", ""))
        if op == "answer_eq":
            detail["expected"] = got
            detail["actual"] = answer.get(check["answer"])
            detail["ok"] = norm(got) == norm(detail["actual"])
        elif op == "len_answer_eq":
            detail["expected"] = len(got)
            detail["actual"] = answer.get(check["answer"])
            detail["ok"] = norm(len(got)) == norm(detail["actual"])
        elif op == "len_ge":
            detail["expected"] = f">={check['value']}"
            detail["actual"] = len(got)
            detail["ok"] = len(got) >= check["value"]
        elif op == "value_eq":
            detail["expected"] = check["value"]
            detail["actual"] = got
            detail["ok"] = norm(got) == norm(check["value"])
        elif op == "deep_contains":
            detail["expected"] = f"contains {check['value']!r}"
            detail["actual"] = deep_contains(got, check["value"])
            detail["ok"] = detail["actual"] is True
        elif op == "cell_text_eq":
            # [#4600] 표 좌표 지목 대조 — 봉투 전체 deep_contains 는 "아무 셀에나
            # 있으면 통과"라 잘못된 셀을 고친 제출을 걸러내지 못했다.
            cell = find_cell(got, check["table"], check["row"], check["col"])
            detail["expected"] = (f"tables[{check['table']}] "
                                  f"({check['row']},{check['col']}) == {check['value']!r}")
            detail["actual"] = None if cell is None else cell.get("text")
            detail["ok"] = cell is not None and norm(cell.get("text")) == norm(check["value"])
        else:
            detail["error"] = f"미지 op: {op}"
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
            answer = json.load(io.open(ans_path, encoding="utf-8"))
        except ValueError as e:
            result["error"] = f"answer.json 파싱 실패: {e}"
            return result
    for check in task["checks"]:
        result["checks"].append(eval_check(check, task, sub_dir, answer, bin_path))
    result["pass"] = bool(result["checks"]) and all(c["ok"] for c in result["checks"])
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--agent", required=True)
    ap.add_argument("--submissions", default=None)
    ap.add_argument("--bin", default=None)
    ap.add_argument("--out", default=None)
    a = ap.parse_args()
    bin_path = find_bin(a.bin)
    sub_root = a.submissions or os.path.join(HERE, "submissions", a.agent)
    out_dir = a.out or sub_root
    os.makedirs(out_dir, exist_ok=True)

    tasks = []
    task_dir = os.path.join(HERE, "tasks")
    for name in sorted(os.listdir(task_dir)):
        if name.endswith(".json"):
            tasks.append(json.load(io.open(os.path.join(task_dir, name), encoding="utf-8")))

    results = [score_task(t, sub_root, bin_path) for t in tasks]
    score = sum(r["tier"] for r in results if r["pass"])
    maximum = sum(r["tier"] for r in results)
    card = {"kind": "gymScorecard", "schemaVersion": "1.0", "agent": a.agent,
            "score": score, "max": maximum, "tasks": results}
    card_path = os.path.join(out_dir, "scorecard.json")
    io.open(card_path, "w", encoding="utf-8", newline="\n").write(
        json.dumps(card, ensure_ascii=False, indent=2))

    lines = [f"# 짐 스코어카드 — {a.agent}", "",
             f"**{score} / {maximum}** (통과 {sum(1 for r in results if r['pass'])}/{len(results)}과제)",
             "", "| 과제 | 티어 | 판정 | 세부 |", "|---|---|---|---|"]
    for r in results:
        if "error" in r:
            det = r["error"]
        else:
            det = " · ".join(
                ("O" if c["ok"] else "X") + " " + str(c["name"]) for c in r["checks"])
        mark = "통과" if r["pass"] else "실패"
        lines.append(f"| {r['id']} {r['title']} | {r['tier']} | {mark} | {det} |")
    lines.append("")
    lines.append(f"채점기: score.py (라이브 오라클) · 바이너리: {os.path.basename(bin_path)}")
    io.open(os.path.join(out_dir, "report.md"), "w", encoding="utf-8", newline="\n").write("\n".join(lines))
    print(f"{a.agent}: {score}/{maximum}  → {card_path}")
    return 0 if score == maximum else 3


if __name__ == "__main__":
    sys.exit(main())
