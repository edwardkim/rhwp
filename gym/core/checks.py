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

import csv
import hashlib
import json
import os
from xml.etree import ElementTree


MAX_TEXT_FILE_ENVELOPE_BYTES = 8 * 1024 * 1024


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


def op_xml_root_eq(ctx):
    """제출 XML의 실제 root local-name을 확인한다.

    `file_exists`와 크기만으로는 임의 바이트가 SVG 산출물로 통과할 수 있다. XML을
    파싱해 root를 확인하면 최소한 도구가 요구한 형식의 문서인지 판정할 수 있다.
    """
    expected = ctx.check["value"]
    try:
        root = ElementTree.parse(ctx.sub_path(ctx.check["file"])).getroot()
        actual = root.tag.rsplit("}", 1)[-1]
    except (OSError, ElementTree.ParseError) as exc:
        return {"expected": expected, "actual": f"XML 파싱 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": actual == expected}


def op_json_value_eq(ctx):
    """제출 JSON의 지목된 값을 확인한다."""
    expected = ctx.check["value"]
    try:
        with open(ctx.sub_path(ctx.check["file"]), encoding="utf-8") as fh:
            actual = dig(json.load(fh), ctx.check.get("path", ""))
    except (OSError, ValueError, KeyError, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"JSON 경로 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": norm(actual) == norm(expected)}


def op_csv_cell_eq(ctx):
    """제출 CSV의 좌표 셀을 확인한다."""
    expected = ctx.check["value"]
    row_index = ctx.check["row"]
    col_index = ctx.check["col"]
    try:
        if not isinstance(row_index, int) or not isinstance(col_index, int):
            raise TypeError("row/col은 정수여야 함")
        if row_index < 0 or col_index < 0:
            raise IndexError("row/col은 음수일 수 없음")
        with open(ctx.sub_path(ctx.check["file"]), encoding="utf-8-sig", newline="") as fh:
            for index, row in enumerate(csv.reader(fh)):
                if index == row_index:
                    actual = row[col_index]
                    break
            else:
                raise IndexError(f"행 {row_index} 없음")
    except (OSError, UnicodeError, csv.Error, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"CSV 좌표 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": norm(actual) == norm(expected)}


def op_utf8_bom(ctx):
    """제출 파일이 UTF-8 BOM으로 시작하는지 확인한다."""
    expected = bool(ctx.check.get("value", True))
    try:
        with open(ctx.sub_path(ctx.check["file"]), "rb") as fh:
            actual = fh.read(3) == b"\xef\xbb\xbf"
    except OSError as exc:
        return {"expected": expected, "actual": f"파일 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": actual == expected}


def _load_json_at(ctx):
    """제출 JSON을 읽고 `path` 좌표의 값을 돌려준다."""
    with open(ctx.sub_path(ctx.check["file"]), encoding="utf-8") as fh:
        return dig(json.load(fh), ctx.check.get("path", ""))


def iter_ndjson_lines(path):
    """비어 있지 않은 NDJSON 줄. 앞뒤 공백은 버리고 빈 줄은 센 대상이 아니다."""
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            stripped = line.strip()
            if stripped:
                yield stripped


def op_json_len_eq(ctx):
    """제출 JSON의 지목된 배열/객체 길이를 확인한다."""
    expected = ctx.check["value"]
    try:
        got = _load_json_at(ctx)
        if not isinstance(got, (list, dict)):
            raise TypeError(f"배열/객체가 아님: {type(got).__name__}")
        actual = len(got)
    except (OSError, ValueError, KeyError, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"JSON 길이 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": norm(actual) == norm(expected)}


def op_csv_row_count_eq(ctx):
    """제출 CSV의 행 수(utf-8-sig)를 확인한다."""
    expected = ctx.check["value"]
    try:
        with open(ctx.sub_path(ctx.check["file"]), encoding="utf-8-sig", newline="") as fh:
            actual = sum(1 for _ in csv.reader(fh))
    except (OSError, UnicodeError, csv.Error) as exc:
        return {"expected": expected, "actual": f"CSV 행수 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": norm(actual) == norm(expected)}


def op_ndjson_count_eq(ctx):
    """제출 NDJSON의 비어 있지 않은 줄 수를 확인한다."""
    expected = ctx.check["value"]
    try:
        actual = sum(1 for _ in iter_ndjson_lines(ctx.sub_path(ctx.check["file"])))
    except (OSError, UnicodeError) as exc:
        return {"expected": expected, "actual": f"NDJSON 줄수 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": norm(actual) == norm(expected)}


def op_ndjson_field_eq(ctx):
    """제출 NDJSON의 0부터 세는 비어 있지 않은 줄에서 지목 필드를 확인한다."""
    expected = ctx.check["value"]
    row_index = ctx.check["row"]
    try:
        if not isinstance(row_index, int) or isinstance(row_index, bool):
            raise TypeError("row는 정수여야 함")
        if row_index < 0:
            raise IndexError("row는 음수일 수 없음")
        actual = None
        found = False
        for index, line in enumerate(iter_ndjson_lines(ctx.sub_path(ctx.check["file"]))):
            if index == row_index:
                actual = dig(json.loads(line), ctx.check.get("path", ""))
                found = True
                break
        if not found:
            raise IndexError(f"행 {row_index} 없음")
    except (OSError, UnicodeError, ValueError, KeyError, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"NDJSON 필드 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": norm(actual) == norm(expected)}


def op_json_keys_contain(ctx):
    """제출 JSON 객체가 `keys` 의 키를 모두 갖는지 확인한다."""
    required = ctx.check["keys"]
    try:
        if not isinstance(required, list) or any(not isinstance(k, str) for k in required):
            raise TypeError("keys는 문자열 목록이어야 함")
        got = _load_json_at(ctx)
        if not isinstance(got, dict):
            raise TypeError(f"객체가 아님: {type(got).__name__}")
        actual = sorted(got)
        missing = [k for k in required if k not in got]
    except (OSError, ValueError, KeyError, IndexError, TypeError) as exc:
        return {"expected": required, "actual": f"JSON 키 확인 실패: {exc}", "ok": False}
    return {"expected": list(required), "actual": actual, "ok": not missing}


def op_text_line_eq(ctx):
    """제출 텍스트의 0부터 세는 한 줄이 `value` 와 같은지 확인한다."""
    expected = ctx.check["value"]
    line_index = ctx.check["line"]
    try:
        if not isinstance(line_index, int) or isinstance(line_index, bool):
            raise TypeError("line은 정수여야 함")
        if line_index < 0:
            raise IndexError("line은 음수일 수 없음")
        with open(ctx.sub_path(ctx.check["file"]), encoding="utf-8") as fh:
            for index, line in enumerate(fh):
                if index == line_index:
                    actual = line.rstrip("\r\n")
                    break
            else:
                raise IndexError(f"줄 {line_index} 없음")
    except (OSError, UnicodeError, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"텍스트 줄 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": actual == expected}


def _require_nonneg_int(name, value):
    """좌표 정수를 받는다. bool 은 int 의 하위형이라 명시적으로 거절한다."""
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{name}은 정수여야 함")
    if value < 0:
        raise IndexError(f"{name}은 음수일 수 없음")
    return value


def json_type_name(value):
    """JSON 값의 타입 이름. bool 은 number 가 아니다."""
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, (int, float)):
        return "number"
    if isinstance(value, str):
        return "string"
    if isinstance(value, list):
        return "array"
    if isinstance(value, dict):
        return "object"
    return type(value).__name__


def _require_str_list(name, value):
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise TypeError(f"{name}는 문자열 목록이어야 함")
    return value


def _csv_row_at(path, row_index):
    _require_nonneg_int("row", row_index)
    with open(path, encoding="utf-8-sig", newline="") as fh:
        for index, row in enumerate(csv.reader(fh)):
            if index == row_index:
                return row
    raise IndexError(f"행 {row_index} 없음")


def _ndjson_record_at(path, row_index):
    _require_nonneg_int("row", row_index)
    for index, line in enumerate(iter_ndjson_lines(path)):
        if index == row_index:
            return json.loads(line)
    raise IndexError(f"행 {row_index} 없음")


def _text_line_at(path, line_index):
    _require_nonneg_int("line", line_index)
    with open(path, encoding="utf-8") as fh:
        for index, line in enumerate(fh):
            if index == line_index:
                return line.rstrip("\r\n")
    raise IndexError(f"줄 {line_index} 없음")


def op_json_type_eq(ctx):
    """제출 JSON의 지목된 값 타입(array/object/string/number/boolean/null)을 확인한다."""
    expected = ctx.check["value"]
    try:
        if not isinstance(expected, str):
            raise TypeError("value는 타입 이름 문자열이어야 함")
        actual = json_type_name(_load_json_at(ctx))
    except (OSError, ValueError, KeyError, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"JSON 타입 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": actual == expected}


def op_json_len_ge(ctx):
    """제출 JSON의 지목된 배열/객체 길이가 `value` 이상인지 확인한다."""
    expected = ctx.check["value"]
    try:
        got = _load_json_at(ctx)
        if not isinstance(got, (list, dict)):
            raise TypeError(f"배열/객체가 아님: {type(got).__name__}")
        actual = len(got)
        ok = float(actual) >= float(expected)
    except (OSError, ValueError, KeyError, IndexError, TypeError) as exc:
        return {"expected": f">={expected}", "actual": f"JSON 길이 하한 확인 실패: {exc}",
                "ok": False}
    return {"expected": f">={expected}", "actual": actual, "ok": ok}


def op_json_array_item_eq(ctx):
    """제출 JSON 배열의 0부터 세는 `index` 항목이 `value` 와 같은지 확인한다."""
    expected = ctx.check["value"]
    try:
        index = _require_nonneg_int("index", ctx.check["index"])
        got = _load_json_at(ctx)
        if not isinstance(got, list):
            raise TypeError(f"배열이 아님: {type(got).__name__}")
        actual = got[index]
    except (OSError, ValueError, KeyError, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"JSON 배열 항목 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": norm(actual) == norm(expected)}


def op_csv_col_count_eq(ctx):
    """제출 CSV의 지목 행 열 수가 `value` 인지 확인한다."""
    expected = ctx.check["value"]
    try:
        actual = len(_csv_row_at(ctx.sub_path(ctx.check["file"]), ctx.check["row"]))
    except (OSError, UnicodeError, csv.Error, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"CSV 열수 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": norm(actual) == norm(expected)}


def op_csv_header_eq(ctx):
    """제출 CSV 첫 행(헤더)이 `values` 와 같은지 확인한다."""
    expected = ctx.check["values"]
    try:
        _require_str_list("values", expected)
        actual = _csv_row_at(ctx.sub_path(ctx.check["file"]), 0)
    except (OSError, UnicodeError, csv.Error, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"CSV 헤더 확인 실패: {exc}", "ok": False}
    return {"expected": list(expected), "actual": actual, "ok": actual == list(expected)}


def op_csv_row_eq(ctx):
    """제출 CSV의 지목 행 전체가 `values` 와 같은지 확인한다."""
    expected = ctx.check["values"]
    try:
        _require_str_list("values", expected)
        actual = _csv_row_at(ctx.sub_path(ctx.check["file"]), ctx.check["row"])
    except (OSError, UnicodeError, csv.Error, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"CSV 행 확인 실패: {exc}", "ok": False}
    return {"expected": list(expected), "actual": actual, "ok": actual == list(expected)}


def op_ndjson_keys_contain(ctx):
    """제출 NDJSON의 지목 행 객체가 `keys` 를 모두 갖는지 확인한다."""
    required = ctx.check["keys"]
    try:
        _require_str_list("keys", required)
        got = dig(_ndjson_record_at(ctx.sub_path(ctx.check["file"]), ctx.check["row"]),
                  ctx.check.get("path", ""))
        if not isinstance(got, dict):
            raise TypeError(f"객체가 아님: {type(got).__name__}")
        actual = sorted(got)
        missing = [key for key in required if key not in got]
    except (OSError, UnicodeError, ValueError, KeyError, IndexError, TypeError) as exc:
        return {"expected": required, "actual": f"NDJSON 키 확인 실패: {exc}", "ok": False}
    return {"expected": list(required), "actual": actual, "ok": not missing}


def op_ndjson_len_eq(ctx):
    """제출 NDJSON의 지목 행에서 배열/객체 길이가 `value` 인지 확인한다."""
    expected = ctx.check["value"]
    try:
        got = dig(_ndjson_record_at(ctx.sub_path(ctx.check["file"]), ctx.check["row"]),
                  ctx.check.get("path", ""))
        if not isinstance(got, (list, dict)):
            raise TypeError(f"배열/객체가 아님: {type(got).__name__}")
        actual = len(got)
    except (OSError, UnicodeError, ValueError, KeyError, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"NDJSON 길이 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": norm(actual) == norm(expected)}


def op_text_line_count_eq(ctx):
    """제출 텍스트의 줄 수가 `value` 인지 확인한다. 마지막 개행만 있는 빈 줄도 센다."""
    expected = ctx.check["value"]
    try:
        with open(ctx.sub_path(ctx.check["file"]), encoding="utf-8") as fh:
            actual = sum(1 for _ in fh)
    except (OSError, UnicodeError) as exc:
        return {"expected": expected, "actual": f"텍스트 줄수 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": norm(actual) == norm(expected)}


def op_text_line_contains(ctx):
    """제출 텍스트의 지목 줄이 `value` 부분 문자열을 갖는지 확인한다."""
    expected = ctx.check["value"]
    try:
        if not isinstance(expected, str):
            raise TypeError("value는 문자열이어야 함")
        actual = _text_line_at(ctx.sub_path(ctx.check["file"]), ctx.check["line"])
    except (OSError, UnicodeError, IndexError, TypeError) as exc:
        return {"expected": expected, "actual": f"텍스트 줄 포함 확인 실패: {exc}", "ok": False}
    return {"expected": expected, "actual": actual, "ok": expected in actual}


# --- 봉투 연산자 — CLI 봉투의 지목된 자리를 본다 ---


def _text_digest(value):
    encoded = value.encode("utf-8")
    return {"sha256": hashlib.sha256(encoded).hexdigest(), "bytes": len(encoded)}


def op_text_file_envelope_eq(ctx):
    """제출 텍스트 전체를 CLI JSON 봉투의 지목 문자열과 대조한다.

    CSV 수치를 task JSON에 복제하지 않고 채점 시점의 rhwp가 생성한
    ``charts[0].csv`` 같은 문자열을 정답으로 쓴다. ``newline=""``로 CRLF를
    보존해 BOM·개행·모든 셀이 일치해야 통과한다. 상세에는 출처 문자열
    대신 해시와 바이트 수만 남겨 untrusted content를 재출력하지 않는다.
    """
    try:
        expected_text = ctx.dug()
        if not isinstance(expected_text, str):
            raise TypeError(f"봉투 값이 문자열이 아님: {type(expected_text).__name__}")
        expected = _text_digest(expected_text)
        if expected["bytes"] > MAX_TEXT_FILE_ENVELOPE_BYTES:
            raise ValueError(f"봉투 텍스트가 상한 초과: {expected['bytes']} 바이트")
        submitted_path = ctx.sub_path(ctx.check["file"])
        actual_size = os.path.getsize(submitted_path)
        if actual_size != expected["bytes"]:
            return {
                "expected": expected,
                "actual": {"sha256": None, "bytes": actual_size},
                "ok": False,
            }
        with open(submitted_path, encoding="utf-8", newline="") as fh:
            actual_text = fh.read()
        actual = _text_digest(actual_text)
    except (OSError, UnicodeError, KeyError, IndexError, TypeError, ValueError) as exc:
        return {"expected": "CLI 봉투의 지목 텍스트", "actual": f"전체 텍스트 대조 실패: {exc}",
                "ok": False}
    return {"expected": expected, "actual": actual, "ok": actual_text == expected_text}


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
    "xml_root_eq": (op_xml_root_eq, False),
    "json_value_eq": (op_json_value_eq, False),
    "csv_cell_eq": (op_csv_cell_eq, False),
    "utf8_bom": (op_utf8_bom, False),
    "json_len_eq": (op_json_len_eq, False),
    "csv_row_count_eq": (op_csv_row_count_eq, False),
    "ndjson_count_eq": (op_ndjson_count_eq, False),
    "ndjson_field_eq": (op_ndjson_field_eq, False),
    "json_keys_contain": (op_json_keys_contain, False),
    "text_line_eq": (op_text_line_eq, False),
    "json_type_eq": (op_json_type_eq, False),
    "json_len_ge": (op_json_len_ge, False),
    "json_array_item_eq": (op_json_array_item_eq, False),
    "csv_col_count_eq": (op_csv_col_count_eq, False),
    "csv_header_eq": (op_csv_header_eq, False),
    "csv_row_eq": (op_csv_row_eq, False),
    "ndjson_keys_contain": (op_ndjson_keys_contain, False),
    "ndjson_len_eq": (op_ndjson_len_eq, False),
    "text_line_count_eq": (op_text_line_count_eq, False),
    "text_line_contains": (op_text_line_contains, False),
    # 봉투 연산자(CLI 호출)
    "text_file_envelope_eq": (op_text_file_envelope_eq, True),
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
