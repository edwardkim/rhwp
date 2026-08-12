"""[#4653] 검사 연산자 등록부 — 판정의 단일 출처.

pack 이 늘어나도 판정 어휘는 여기 한 곳에서만 자란다. 과제 파일은 연산자를
**고르기만** 하고 정의하지 않는다 — 과제마다 판정 논리가 흩어지면 #4600 같은
오검출이 pack 수만큼 늘어난다.

## 연산자를 고르는 규칙

1. **대상을 지목하라.** 봉투 전역을 훑는 검사(`deep_contains`)는 "값이 어딘가
   있으면 통과"라 엉뚱한 곳을 고친 제출을 걸러내지 못한다. 편집 과제는
   `value_eq`·`cell_text_eq` 처럼 좌표를 지목하는 연산자를 쓴다.
2. **기대값을 박제하지 마라.** 정답은 채점 시점에 rhwp 로 재계산하거나
   (`answer_eq` 계열), rhwp 자신에게 판정을 시킨다(`{sha256:}` + replay).
3. **부재를 통과로 위장하지 마라.** 좌표가 없으면 `None` 으로 실패한다.
"""

import hashlib
import os


def sha256_of(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


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


def find_cell(tables, table_index, row, col):
    """[#4600] 표 좌표로 셀을 지목한다.

    `cells[0]` 같은 순서 가정 대신 (row, col) 로 찾는다 — 순서 가정은 내보내기
    구현이 바뀌면 조용히 엉뚱한 셀을 검사하게 되고, 그것이 #4600 이 잡은
    오검출과 같은 부류의 결함이다.
    """
    table = tables[table_index]
    for cell in table["cells"]:
        if cell.get("row") == row and cell.get("col") == col:
            return cell
    return None


# --- 파일 연산자 — CLI 를 부르지 않고 제출물 자체를 본다 ---


def op_same_hash(ctx):
    hashes = [sha256_of(ctx.sub_path(f)) for f in ctx.check["files"]]
    return {"expected": hashes[0][:16], "actual": hashes[1][:16],
            "ok": len(set(hashes)) == 1}


def op_differs_from_input(ctx):
    """[#4600] 무편집 복사본 거부 — 산출물이 과제 입력과 바이트가 같으면
    아무 작업도 하지 않은 것이다."""
    submitted = sha256_of(ctx.sub_path(ctx.check["file"]))
    source = sha256_of(ctx.root_path(ctx.task["input"]))
    return {"expected": f"!= {source[:16]} (과제 입력)", "actual": submitted[:16],
            "ok": submitted != source}


def op_file_exists(ctx):
    path = ctx.sub_path(ctx.check["file"])
    exists = os.path.isfile(path)
    size = os.path.getsize(path) if exists else 0
    minimum = ctx.check.get("minBytes", 1)
    return {"expected": f"존재 · >= {minimum} 바이트", "actual": size if exists else "없음",
            "ok": exists and size >= minimum}


def op_files_differ(ctx):
    """두 제출물이 서로 달라야 한다 — 같은 산출을 두 번 낸 위장 제출 거부."""
    hashes = [sha256_of(ctx.sub_path(f)) for f in ctx.check["files"]]
    return {"expected": "두 파일이 서로 다름", "actual": f"{hashes[0][:8]} vs {hashes[1][:8]}",
            "ok": len(set(hashes)) == len(hashes)}


# --- 봉투 연산자 — CLI 봉투의 지목된 자리를 본다 ---


def op_answer_eq(ctx):
    got = ctx.dug()
    actual = ctx.answer.get(ctx.check["answer"])
    return {"expected": got, "actual": actual, "ok": norm(got) == norm(actual)}


def op_len_answer_eq(ctx):
    got = ctx.dug()
    actual = ctx.answer.get(ctx.check["answer"])
    return {"expected": len(got), "actual": actual, "ok": norm(len(got)) == norm(actual)}


def op_len_ge(ctx):
    got = ctx.dug()
    return {"expected": f">={ctx.check['value']}", "actual": len(got),
            "ok": len(got) >= ctx.check["value"]}


def op_value_eq(ctx):
    got = ctx.dug()
    return {"expected": ctx.check["value"], "actual": got,
            "ok": norm(got) == norm(ctx.check["value"])}


def op_value_ge(ctx):
    got = ctx.dug()
    try:
        ok = float(got) >= float(ctx.check["value"])
    except (TypeError, ValueError):
        ok = False
    return {"expected": f">={ctx.check['value']}", "actual": got, "ok": ok}


def op_value_in(ctx):
    got = ctx.dug()
    allowed = ctx.check["values"]
    return {"expected": f"in {allowed}", "actual": got,
            "ok": any(norm(got) == norm(v) for v in allowed)}


def op_deep_contains(ctx):
    got = ctx.dug()
    found = deep_contains(got, ctx.check["value"])
    return {"expected": f"contains {ctx.check['value']!r}", "actual": found,
            "ok": found is True}


def op_not_contains(ctx):
    """지워졌는지 본다 — 마스킹·정리 과제의 판정(있으면 실패)."""
    got = ctx.dug()
    found = deep_contains(got, ctx.check["value"])
    return {"expected": f"{ctx.check['value']!r} 부재", "actual": found,
            "ok": found is False}


def op_cell_text_eq(ctx):
    """[#4600] 표 좌표 지목 대조."""
    got = ctx.dug()
    cell = find_cell(got, ctx.check["table"], ctx.check["row"], ctx.check["col"])
    return {"expected": (f"tables[{ctx.check['table']}] "
                         f"({ctx.check['row']},{ctx.check['col']}) == {ctx.check['value']!r}"),
            "actual": None if cell is None else cell.get("text"),
            "ok": cell is not None and norm(cell.get("text")) == norm(ctx.check["value"])}


REGISTRY = {
    # 파일 연산자(CLI 미호출)
    "same_hash": (op_same_hash, False),
    "differs_from_input": (op_differs_from_input, False),
    "file_exists": (op_file_exists, False),
    "files_differ": (op_files_differ, False),
    # 봉투 연산자(CLI 호출)
    "answer_eq": (op_answer_eq, True),
    "len_answer_eq": (op_len_answer_eq, True),
    "len_ge": (op_len_ge, True),
    "value_eq": (op_value_eq, True),
    "value_ge": (op_value_ge, True),
    "value_in": (op_value_in, True),
    "deep_contains": (op_deep_contains, True),
    "not_contains": (op_not_contains, True),
    "cell_text_eq": (op_cell_text_eq, True),
}

#: 대상을 지목하지 못하는 연산자 — 편집 과제에서 쓰면 스키마 검증이 막는다.
GLOBAL_SCAN_OPS = {"deep_contains", "not_contains"}


def needs_cli(op):
    return REGISTRY[op][1]
