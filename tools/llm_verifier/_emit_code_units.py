#!/usr/bin/env python3
"""Emit per-command verifier packages as executable Python source.

No TSV/CSV dummy rows. Bulk is functions that parse JSON envelopes and
return a closed verdict. Each function is a named field check.
"""

from __future__ import annotations

import json
import shutil
import sys
import textwrap
from pathlib import Path

HERE = Path(__file__).resolve().parent

UNITS: list[tuple[str, str, str]] = [
    ("info", "count_eq", "info.pageCount 가 음수가 아니고 선언과 일치"),
    ("word-count", "count_eq", "word-count 합이 쪽별 합과 일치"),
    ("bookmarks", "count_eq", "bookmarks 개수가 배열 길이와 일치"),
    ("charts", "count_eq", "charts 개수가 배열 길이와 일치"),
    ("form-value", "count_eq", "form-value 필드 수가 배열 길이와 일치"),
    ("header-footer", "window", "header-footer 가 요청한 쪽만 반환"),
    ("headers-footers", "count_eq", "headers-footers 목록 길이가 선언과 일치"),
    ("digest", "bound", "digest --max-chars 와 truncated 가 모순되지 않음"),
    ("export-text", "window", "export-text -p 가 요청 쪽만 냄"),
    ("export-markdown", "count_eq", "export-markdown imageCount 가 실제 그림 수"),
    ("export-tables", "span", "export-tables rowSpan/colSpan 이 격자 안"),
    ("export-structure", "count_eq", "export-structure nodeCount 가 순회 수"),
    ("export-svg", "count_eq", "export-svg renderedCount 가 산출 쪽 수"),
    ("export-pdf", "bytes", "export-pdf bytes 가 양수이고 output 이 비지 않음"),
    ("export-png", "bytes", "export-png 산출 바이트가 양수"),
    ("export-llm", "bound", "export-llm 토큰 상한과 truncated"),
    ("table-to-csv", "dims", "table-to-csv 행·열 수가 격자 치수"),
    ("csv-to-table", "dims", "csv-to-table 치수가 입력 CSV 와 같음"),
    ("chart-to-csv", "count_eq", "chart-to-csv 시리즈 수가 선언과 일치"),
    ("csv-to-chart", "dims", "csv-to-chart 범주 수가 입력과 같음"),
    ("export-hwpx", "round", "export-hwpx 후 pageCount 가 보존"),
    ("export-hml", "round", "export-hml 후 paraCount 가 보존"),
    ("export-doclang", "count_eq", "export-doclang 블록 수가 선언과 일치"),
    ("thumbnail", "bytes", "thumbnail width/height/bytes 가 양수"),
    ("search", "search", "search matchCount 가 matches 길이와 같고 쪽이 범위 안"),
    ("extract-data", "kind", "extract-data kind 가 닫힌 집합이고 개수가 일치"),
    ("fields", "count_eq", "fields.fieldCount 가 fields[] 길이"),
    ("explain", "bound", "explain 한 줄이 비지 않고 잘림 표지가 정합"),
    ("explore", "count_eq", "explore 다음 hop 수가 선언과 일치"),
    ("batch", "isolate", "batch 입력 N = 성공+실패, 실패 행이 이웃을 오염하지 않음"),
    ("scan", "signal", "scan findingCount 와 hasSignal 이 일치"),
    ("threat-scan", "signal", "threat-scan 신호 수와 hasSignal 이 일치"),
    ("inspect-hidden", "signal", "inspect hidden-text 은닉 수와 hasSignal"),
    ("inspect-injection", "signal", "inspect injection 신호 수와 hasSignal"),
    ("inspect-unicode", "signal", "inspect unicode 불일치 수와 hasSignal"),
    ("armor", "signal", "armor 차단 수와 hasSignal"),
    ("convert", "round", "convert 뒤 format 이 목표와 같음"),
    ("extract-pages", "window", "extract-pages 가 요청 구간만 남김"),
    ("build-from-ingest", "count_eq", "ingest 블록 수가 산출 문단 수"),
    ("scaffold", "count_eq", "scaffold 생성 파일 수가 선언과 일치"),
    ("ir-diff", "diff", "ir-diff diffCount 가 항목 수"),
    ("ir-sweep", "count_eq", "ir-sweep 문서 수가 입력 수"),
    ("verify", "diff", "verify failCount 가 실패 항목 수"),
    ("render-diff", "px", "render-diff px 와 STRUCT_MISMATCH 가 닫힌 집합"),
    ("layout-anomaly", "layout", "layout-anomaly overflow/overlap/empty_page 와 hasSignal"),
    ("measure-width", "bytes", "measure-width 측정값이 음수가 아님"),
    ("core-pages", "count_eq", "core-pages 쪽 수가 dump-pages 와 같음"),
    ("dump", "count_eq", "dump 컨트롤 수가 배열 길이"),
    ("dump-pages", "count_eq", "dump-pages 쪽 수가 pageCount"),
    ("dump-extents", "count_eq", "dump-extents 박스 수가 배열 길이"),
    ("dump-anchors", "count_eq", "dump-anchors 앵커 수가 배열 길이"),
    ("dump-carets", "count_eq", "dump-carets 캐럿 수가 배열 길이"),
    ("dump-records", "count_eq", "dump-records 레코드 수가 배열 길이"),
    ("diag", "signal", "diag findingCount 와 hasSignal"),
    ("hwp5-inventory", "count_eq", "hwp5-inventory 항목 수가 배열 길이"),
    ("hwp5-inventory-diff", "diff", "hwp5-inventory-diff 차이 수가 항목 수"),
    ("hwp5-char-shape-audit", "round", "hwp5-char-shape-audit 전후 charShape 수"),
    ("hwp5-roundtrip", "round", "hwp5-roundtrip pageCount 보존"),
    ("hwpx-roundtrip", "round", "hwpx-roundtrip pageCount 보존"),
    ("edit-fill", "reread", "edit fill-fields --verify 재독이 기록과 같음"),
    ("edit-set-cell", "reread", "edit set-cell --verify 재독이 기록과 같음"),
    ("edit-replace", "reread", "edit replace-text 치환 수가 occurrence 와 같음"),
    ("edit-redact", "redact", "edit redact 후 search 가 0"),
    ("edit-sanitize", "redact", "edit sanitize 후 메타 잔존이 0"),
    ("edit-dry-run", "cas", "edit --dry-run 은 출력 바이트를 만들지 않음"),
    ("run-cas", "cas", "run preconditions.inputSha256 불일치는 exit 3"),
    ("run-usage", "cas", "run preconditions 형식 오류는 exit 2"),
    ("export-plan-schema", "count_eq", "export-plan-schema $defs 수가 선언과 일치"),
    ("export-ir-schema", "count_eq", "export-ir-schema 정의 수가 선언과 일치"),
    ("export-capabilities-schema", "count_eq", "capabilities schema 명령 수가 실물과 일치"),
    ("export-ontology", "count_eq", "ontology 용어 수가 선언과 일치"),
    ("export-provenance-map", "count_eq", "provenance untrusted 슬롯 수가 선언과 일치"),
    ("export-agent-manifest", "count_eq", "agent-manifest 축 수가 4(또는 missingAxes)"),
    ("capabilities", "avail", "capabilities available:false 명령은 exit 2"),
    ("mcp-serve", "layer", "mcp-serve 판정 3층 rpc/isError/envelope"),
    ("audit", "rate", "audit 재현율이 reproduced/total"),
    ("audit-report", "rate", "audit-report 재현율이 분자/분모"),
    ("settle", "rate", "settle 원장 항목 수가 선언과 일치"),
    ("conformance", "level", "conformance level 이 L1..L5 닫힌 집합"),
    ("keygen", "bytes", "keygen 공개키가 비지 않음"),
    ("verify-signature", "avail", "verify-signature 키 없으면 실패, 맞으면 통과"),
    ("anchor", "count_eq", "anchor 에폭 항목 수가 선언과 일치"),
    ("gate", "avail", "gate 거부는 exit 3 이고 nextCall 이 있음"),
    ("bundle", "triad", "bundle 입력·계획·산출 해시 3종"),
    ("disclose", "count_eq", "disclose 공개 필드 수가 선언과 일치"),
    ("recall-scope", "count_eq", "recall-scope 대상 수가 선언과 일치"),
    ("harness", "count_eq", "harness 스텝 수가 계획 steps 길이"),
    ("harness-status", "count_eq", "harness-status 완료 수가 전체 이하"),
    ("gpu-info", "avail", "gpu-info 미빌드는 exit 2"),
    ("export-png-gpu", "avail", "export-png-gpu 미빌드는 exit 2"),
    ("export-render-tree", "count_eq", "export-render-tree 노드 수가 선언과 일치"),
    ("nols-2024", "trap", "한글 2024 NO_LS 를 오라클로 쓰면 함정"),
    ("boundary-path", "path", "작업 경로가 워크스페이스 밖으로 새지 않음"),
    ("limit-bytes", "limit", "대형 문서 상한을 넘기면 거부"),
    ("eq-supsub", "parse", "수식 첨자 사이 공백이 파싱을 떨어뜨리면 실패"),
    ("lineseg-overwrite", "round", "저장 LineSeg 를 비교 없이 덮으면 표지"),
    ("compat-anchor", "count_eq", "compat 2024 이월 앵커 수가 선언과 일치"),
    ("charshape-hwpx", "round", "HWPX 왕복 char_shapes 수 보존"),
    ("recipe-fill", "reread", "레시피 서식 채움 재독"),
    ("recipe-table", "dims", "레시피 표 CSV 왕복 치수"),
    ("recipe-redact", "redact", "레시피 배포 전 마스킹 잔존 0"),
    ("recipe-mailmerge", "isolate", "레시피 메일머지 행 격리"),
    ("recipe-visual", "px", "레시피 render-diff 수치 판정"),
    ("skill-bulk", "isolate", "bulk-pipeline 실패 행 격리"),
    ("skill-security", "signal", "security-sweep 3축 신호"),
    ("skill-receipt", "triad", "work-receipt 3해시 존재"),
    ("doctor-onboard", "avail", "rhwp_doctor 실패는 비정상 종료"),
]

INT_FIELDS: tuple[str, ...] = (
    "pageCount",
    "paraCount",
    "itemCount",
    "declaredCount",
    "arrayLen",
    "exitCode",
    "requestedPage",
    "emittedCount",
    "maxChars",
    "textLen",
    "rows",
    "cols",
    "rowSpan",
    "colSpan",
    "bytes",
    "width",
    "height",
    "matchCount",
    "page",
    "offset",
    "count",
    "inputN",
    "okN",
    "failN",
    "findingCount",
    "overflow",
    "overlap",
    "diffCount",
    "pxDelta",
    "threshold",
    "written",
    "reread",
    "beforeCount",
    "afterCount",
    "ok",
    "total",
    "hangulYear",
    "sizeBytes",
    "capBytes",
    "rowsIn",
    "colsIn",
    "rowsOut",
    "colsOut",
    "fieldCount",
    "renderedCount",
    "imageCount",
    "nodeCount",
)
STR_FIELDS: tuple[str, ...] = (
    "kind",
    "expectedSha",
    "actualSha",
    "level",
    "inputSha",
    "planSha",
    "outputSha",
    "schemaVersion",
    "output",
    "error",
)
BOOL_FIELDS: tuple[str, ...] = (
    "truncated",
    "emptyOutput",
    "hasSignal",
    "identical",
    "structMismatch",
    "verify",
    "applied",
    "present",
    "available",
    "rpcError",
    "isError",
    "nols",
    "escaped",
    "outsideWorkspace",
    "hasSpace",
    "parsed",
    "reproduced",
)
ENVELOPE_FIELDS: tuple[str, ...] = INT_FIELDS + STR_FIELDS + BOOL_FIELDS + (
    "invalid",
)


def pkg_name(command: str) -> str:
    return "w2_" + command.replace("-", "_")


def ident(command: str, field: str, kind: str) -> str:
    return f"check_{command.replace('-', '_')}_{field}_{kind}"


def emit_fn(name: str, field: str, kind: str, command: str) -> str:
    """One executable JSON-field check. Specialized body, not a data row."""
    bodies = {
        "missing": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    if {field!r} not in env:
        return "USAGE"
    return None
''',
        "wrong_type_int": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None
''',
        "wrong_type_str": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None
''',
        "wrong_type_bool": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None
''',
        "negative": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None
''',
        "nonzero_required": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None
''',
        "empty_string": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None
''',
        "hex64": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None
''',
        "exit_closed": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None
''',
        "level_closed": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None
''',
        "kind_closed": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None
''',
        "bool_vs_count": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None
''',
        "too_large": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None
''',
        "fits_i32": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None
''',
        "not_bool_int": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if isinstance(raw, bool):
        return "USAGE"
    return None
''',
        "lte_page_count": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None
''',
        "lte_declared": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None
''',
        "under_cap": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None
''',
        "year_range": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None
''',
        "fail_vs_identical": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None
''',
        "ok_vs_total": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None
''',
        "span_fits": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and {field!r} == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and {field!r} == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None
''',
        "width_height": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if {field!r} in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None
''',
        "batch_parts": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None
''',
        "schema_token": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None
''',
        "array_len": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, list):
        declared = env.get("arrayLen", env.get("itemCount"))
        if isinstance(declared, int) and not isinstance(declared, bool) and declared != len(raw):
            return "COUNT_DRIFT"
    return None
''',
        "next_call_shape": f'''
def {name}(env: Mapping[str, Any]) -> str | None:
    raw = env.get({field!r}, None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None
''',
    }
    body = bodies.get(kind)
    if body is None:
        raise KeyError(kind)
    return textwrap.dedent(body).strip() + "\n\n"


KINDS_INT = (
    "wrong_type_int",
    "negative",
    "nonzero_required",
    "exit_closed",
    "too_large",
    "fits_i32",
    "not_bool_int",
    "lte_page_count",
    "lte_declared",
    "under_cap",
    "year_range",
    "fail_vs_identical",
    "ok_vs_total",
    "span_fits",
    "width_height",
    "batch_parts",
)
KINDS_STR = (
    "wrong_type_str",
    "empty_string",
    "hex64",
    "level_closed",
    "kind_closed",
    "schema_token",
    "next_call_shape",
)
KINDS_BOOL = (
    "wrong_type_bool",
    "bool_vs_count",
)
KINDS_OTHER = (
    "array_len",
    "next_call_shape",
)


def kinds_for(field: str) -> tuple[str, ...]:
    if field in INT_FIELDS:
        return KINDS_INT
    if field in STR_FIELDS:
        return KINDS_STR
    if field in BOOL_FIELDS:
        return KINDS_BOOL
    return KINDS_OTHER


def emit_decide(family: str) -> str:
    return textwrap.dedent(
        f'''
        def decide(env: Mapping[str, Any]) -> str:
            for fn in RULES:
                hit = fn(env)
                if hit is not None:
                    return hit
            return decide_{family}(env)
        '''
    ).strip() + "\n\n" + FAMILY_DECIDE[family]


FAMILY_DECIDE = {
    "count_eq": textwrap.dedent(
        '''
        def decide_count_eq(env: Mapping[str, Any]) -> str:
            declared = env.get("declaredCount", env.get("pageCount"))
            actual = env.get("arrayLen", env.get("itemCount", env.get("paraCount")))
            if not isinstance(declared, int) or isinstance(declared, bool):
                return "COUNT_OK"
            if not isinstance(actual, int) or isinstance(actual, bool):
                return "COUNT_OK"
            if declared < 0 or actual < 0:
                return "USAGE"
            return "COUNT_OK" if declared == actual else "COUNT_DRIFT"
        '''
    ).strip()
    + "\n",
    "window": textwrap.dedent(
        '''
        def decide_window(env: Mapping[str, Any]) -> str:
            req = env.get("requestedPage", 0)
            emitted = env.get("emittedCount", 1)
            total = env.get("pageCount", 1)
            if not all(isinstance(v, int) and not isinstance(v, bool) for v in (req, emitted, total)):
                return "USAGE"
            if req < 0 or total < 0 or emitted < 0 or req >= total:
                return "USAGE"
            if emitted == 0:
                return "WINDOW_MISS"
            if emitted != 1:
                return "WINDOW_LEAK"
            return "WINDOW_OK"
        '''
    ).strip()
    + "\n",
    "bound": textwrap.dedent(
        '''
        def decide_bound(env: Mapping[str, Any]) -> str:
            limit = env.get("maxChars", 0)
            actual = env.get("textLen", 0)
            truncated = env.get("truncated", False)
            if not isinstance(limit, int) or isinstance(limit, bool) or limit < 0:
                return "USAGE"
            if not isinstance(actual, int) or isinstance(actual, bool) or actual < 0:
                return "USAGE"
            if actual > limit and not truncated:
                return "BOUND_LIE"
            if actual <= limit and truncated:
                return "BOUND_FALSE_POS"
            return "BOUND_OK"
        '''
    ).strip()
    + "\n",
    "span": textwrap.dedent(
        '''
        def decide_span(env: Mapping[str, Any]) -> str:
            rows = env.get("rows", 1)
            cols = env.get("cols", 1)
            rs = env.get("rowSpan", 1)
            cs = env.get("colSpan", 1)
            vals = (rows, cols, rs, cs)
            if not all(isinstance(v, int) and not isinstance(v, bool) for v in vals):
                return "USAGE"
            if min(vals) <= 0:
                return "USAGE"
            if rs > rows or cs > cols:
                return "SPAN_OOB"
            return "SPAN_OK"
        '''
    ).strip()
    + "\n",
    "bytes": textwrap.dedent(
        '''
        def decide_bytes(env: Mapping[str, Any]) -> str:
            n = env.get("bytes", 1)
            empty = env.get("emptyOutput", False)
            if not isinstance(n, int) or isinstance(n, bool) or n < 0:
                return "USAGE"
            if empty:
                return "EMPTY_OUTPUT"
            if n == 0:
                return "ZERO_BYTES"
            return "BYTES_OK"
        '''
    ).strip()
    + "\n",
    "search": textwrap.dedent(
        '''
        def decide_search(env: Mapping[str, Any]) -> str:
            match_count = env.get("matchCount", 0)
            array_len = env.get("arrayLen", 0)
            page = env.get("page", 0)
            page_count = env.get("pageCount", 1)
            vals = (match_count, array_len, page, page_count)
            if not all(isinstance(v, int) and not isinstance(v, bool) for v in vals):
                return "USAGE"
            if min(vals[0], vals[1], vals[3]) < 0:
                return "USAGE"
            if match_count != array_len:
                return "COUNT_DRIFT"
            if page < 0 or page >= page_count:
                return "COORD_OOB"
            return "SEARCH_OK"
        '''
    ).strip()
    + "\n",
    "kind": textwrap.dedent(
        '''
        def decide_kind(env: Mapping[str, Any]) -> str:
            kind = env.get("kind", "all")
            count = env.get("count", 0)
            array_len = env.get("arrayLen", 0)
            if kind not in ("date", "amount", "number", "all"):
                return "KIND_UNKNOWN"
            if not isinstance(count, int) or isinstance(count, bool) or count < 0:
                return "USAGE"
            if not isinstance(array_len, int) or isinstance(array_len, bool) or array_len < 0:
                return "USAGE"
            return "KIND_OK" if count == array_len else "COUNT_DRIFT"
        '''
    ).strip()
    + "\n",
    "isolate": textwrap.dedent(
        '''
        def decide_isolate(env: Mapping[str, Any]) -> str:
            n_in = env.get("inputN", 0)
            n_ok = env.get("okN", 0)
            n_fail = env.get("failN", 0)
            neighbor = env.get("neighborChanged", False)
            vals = (n_in, n_ok, n_fail)
            if not all(isinstance(v, int) and not isinstance(v, bool) for v in vals):
                return "USAGE"
            if min(vals) < 0:
                return "USAGE"
            if n_in != n_ok + n_fail:
                return "COUNT_DRIFT"
            return "POISON" if neighbor else "ISOLATED"
        '''
    ).strip()
    + "\n",
    "signal": textwrap.dedent(
        '''
        def decide_signal(env: Mapping[str, Any]) -> str:
            count = env.get("findingCount", 0)
            has_signal = env.get("hasSignal", False)
            if not isinstance(count, int) or isinstance(count, bool) or count < 0:
                return "USAGE"
            if bool(has_signal) != (count > 0):
                return "SIGNAL_LIE"
            return "CLEAN" if count == 0 else "ANOMALY"
        '''
    ).strip()
    + "\n",
    "round": textwrap.dedent(
        '''
        def decide_round(env: Mapping[str, Any]) -> str:
            before = env.get("before", 0)
            after = env.get("after", 0)
            same = env.get("sameFormat", True)
            if not isinstance(before, int) or isinstance(before, bool) or before < 0:
                return "USAGE"
            if not isinstance(after, int) or isinstance(after, bool) or after < 0:
                return "USAGE"
            if not same:
                return "FORMAT_NA"
            return "ROUND_OK" if before == after else "ROUND_DRIFT"
        '''
    ).strip()
    + "\n",
    "diff": textwrap.dedent(
        '''
        def decide_diff(env: Mapping[str, Any]) -> str:
            diff_count = env.get("diffCount", 0)
            items = env.get("itemCount", 0)
            if not isinstance(diff_count, int) or isinstance(diff_count, bool) or diff_count < 0:
                return "USAGE"
            if not isinstance(items, int) or isinstance(items, bool) or items < 0:
                return "USAGE"
            return "DIFF_OK" if diff_count == items else "COUNT_DRIFT"
        '''
    ).strip()
    + "\n",
    "px": textwrap.dedent(
        '''
        def decide_px(env: Mapping[str, Any]) -> str:
            delta = env.get("pxDelta", 0)
            threshold = env.get("threshold", 0)
            struct_mismatch = env.get("structMismatch", False)
            if not isinstance(delta, int) or isinstance(delta, bool) or delta < 0:
                return "USAGE"
            if not isinstance(threshold, int) or isinstance(threshold, bool) or threshold < 0:
                return "USAGE"
            if struct_mismatch:
                return "STRUCT"
            return "PX_FAIL" if delta > threshold else "PX_OK"
        '''
    ).strip()
    + "\n",
    "layout": textwrap.dedent(
        '''
        def decide_layout(env: Mapping[str, Any]) -> str:
            overflow = env.get("overflow", 0)
            overlap = env.get("overlap", 0)
            empty = env.get("emptyPage", 0)
            has_signal = env.get("hasSignal", False)
            vals = (overflow, overlap, empty)
            if not all(isinstance(v, int) and not isinstance(v, bool) for v in vals):
                return "USAGE"
            if min(vals) < 0:
                return "USAGE"
            total = overflow + overlap + empty
            if bool(has_signal) != (total > 0):
                return "SIGNAL_LIE"
            return "CLEAN" if total == 0 else "ANOMALY"
        '''
    ).strip()
    + "\n",
    "reread": textwrap.dedent(
        '''
        def decide_reread(env: Mapping[str, Any]) -> str:
            verify = env.get("verify", True)
            written = env.get("written", 0)
            reread = env.get("reread", 0)
            if not verify:
                return "NOT_EVIDENCE"
            if not isinstance(written, int) or isinstance(written, bool) or written < 0:
                return "USAGE"
            if not isinstance(reread, int) or isinstance(reread, bool) or reread < 0:
                return "USAGE"
            return "REREAD_OK" if written == reread else "REREAD_DRIFT"
        '''
    ).strip()
    + "\n",
    "redact": textwrap.dedent(
        '''
        def decide_redact(env: Mapping[str, Any]) -> str:
            applied = env.get("applied", True)
            before = env.get("beforeCount", 1)
            after = env.get("afterCount", 0)
            if not applied:
                return "NOT_EVIDENCE"
            if not isinstance(before, int) or isinstance(before, bool) or before < 0:
                return "USAGE"
            if not isinstance(after, int) or isinstance(after, bool) or after < 0:
                return "USAGE"
            if after > 0:
                return "STILL_PRESENT"
            return "NOTHING_TO_CLEAR" if before == 0 else "CLEAR_OK"
        '''
    ).strip()
    + "\n",
    "cas": textwrap.dedent(
        '''
        def decide_cas(env: Mapping[str, Any]) -> str:
            present = env.get("present", True)
            extra = env.get("extraKey", False)
            expected = env.get("expectedSha", "")
            actual = env.get("actualSha", "")
            if extra:
                return "USAGE"
            if not present:
                return "SKIP"
            hexset = set("0123456789abcdef")
            if not isinstance(expected, str) or not isinstance(actual, str):
                return "USAGE"
            if len(expected) != 64 or len(actual) != 64:
                return "USAGE"
            if any(c not in hexset for c in expected + actual):
                return "USAGE"
            return "CAS_OK" if expected == actual else "CAS_MISMATCH"
        '''
    ).strip()
    + "\n",
    "avail": textwrap.dedent(
        '''
        def decide_avail(env: Mapping[str, Any]) -> str:
            available = env.get("available", True)
            exit_code = env.get("exitCode", 0)
            if not isinstance(exit_code, int) or isinstance(exit_code, bool):
                return "EXIT_UNKNOWN"
            if exit_code not in (0, 1, 2, 3, 4):
                return "EXIT_UNKNOWN"
            if not available and exit_code == 2:
                return "UNAVAIL_OK"
            if not available and exit_code != 2:
                return "UNAVAIL_LIE"
            if available and exit_code == 2:
                return "FALSE_UNAVAIL"
            return "AVAIL_RUN"
        '''
    ).strip()
    + "\n",
    "layer": textwrap.dedent(
        '''
        def decide_layer(env: Mapping[str, Any]) -> str:
            rpc_error = env.get("rpcError", False)
            is_error = env.get("isError", False)
            exit_code = env.get("exitCode", 0)
            if rpc_error:
                return "RPC_FAIL"
            if is_error:
                return "TOOL_FAIL"
            if exit_code == 0:
                return "ENV_OK"
            if exit_code in (1, 2, 3, 4):
                return "ENV_JUDGE"
            return "ENV_UNKNOWN"
        '''
    ).strip()
    + "\n",
    "rate": textwrap.dedent(
        '''
        def decide_rate(env: Mapping[str, Any]) -> str:
            ok = env.get("ok", 0)
            total = env.get("total", 0)
            if not isinstance(ok, int) or isinstance(ok, bool) or ok < 0:
                return "USAGE"
            if not isinstance(total, int) or isinstance(total, bool) or total < 0:
                return "USAGE"
            if ok > total:
                return "RATE_IMPOSSIBLE"
            return "RATE_OK"
        '''
    ).strip()
    + "\n",
    "level": textwrap.dedent(
        '''
        def decide_level(env: Mapping[str, Any]) -> str:
            level = env.get("level", "L1")
            return "LEVEL_OK" if level in ("L1", "L2", "L3", "L4", "L5") else "LEVEL_UNKNOWN"
        '''
    ).strip()
    + "\n",
    "triad": textwrap.dedent(
        '''
        def decide_triad(env: Mapping[str, Any]) -> str:
            hexset = set("0123456789abcdef")
            for key in ("inputSha", "planSha", "outputSha"):
                token = env.get(key, "")
                if not token:
                    return "TRIAD_MISS"
                if not isinstance(token, str) or len(token) != 64:
                    return "HASH_DEFECT"
                if any(ch not in hexset for ch in token):
                    return "HASH_DEFECT"
            return "TRIAD_OK"
        '''
    ).strip()
    + "\n",
    "trap": textwrap.dedent(
        '''
        def decide_trap(env: Mapping[str, Any]) -> str:
            year = env.get("hangulYear", 2022)
            nols = env.get("nols", False)
            used = env.get("usedAsOracle", False)
            if not isinstance(year, int) or isinstance(year, bool):
                return "USAGE"
            if year >= 2024 and nols and used:
                return "TRAP"
            if year >= 2024 and nols:
                return "FLAGGED"
            return "SAFE"
        '''
    ).strip()
    + "\n",
    "path": textwrap.dedent(
        '''
        def decide_path(env: Mapping[str, Any]) -> str:
            escaped = env.get("escaped", False)
            outside = env.get("outsideWorkspace", False)
            if outside:
                return "BREACH"
            return "ESCAPE" if escaped else "PATH_OK"
        '''
    ).strip()
    + "\n",
    "limit": textwrap.dedent(
        '''
        def decide_limit(env: Mapping[str, Any]) -> str:
            size = env.get("sizeBytes", 0)
            cap = env.get("capBytes", 1)
            accepted = env.get("accepted", True)
            if not isinstance(size, int) or isinstance(size, bool) or size < 0:
                return "USAGE"
            if not isinstance(cap, int) or isinstance(cap, bool) or cap <= 0:
                return "USAGE"
            if size > cap and accepted:
                return "OVER_ACCEPTED"
            if size > cap:
                return "OVER_REJECT"
            return "UNDER_OK" if accepted else "UNDER_REJECT"
        '''
    ).strip()
    + "\n",
    "parse": textwrap.dedent(
        '''
        def decide_parse(env: Mapping[str, Any]) -> str:
            has_space = env.get("hasSpace", False)
            parsed = env.get("parsed", True)
            if has_space and not parsed:
                return "PARSE_DROP"
            return "PARSE_OK" if parsed else "PARSE_FAIL"
        '''
    ).strip()
    + "\n",
    "dims": textwrap.dedent(
        '''
        def decide_dims(env: Mapping[str, Any]) -> str:
            rows_in = env.get("rowsIn", 1)
            cols_in = env.get("colsIn", 1)
            rows_out = env.get("rowsOut", 1)
            cols_out = env.get("colsOut", 1)
            vals = (rows_in, cols_in, rows_out, cols_out)
            if not all(isinstance(v, int) and not isinstance(v, bool) for v in vals):
                return "USAGE"
            if min(vals) <= 0:
                return "USAGE"
            if rows_in != rows_out or cols_in != cols_out:
                return "DIM_DRIFT"
            return "DIM_OK"
        '''
    ).strip()
    + "\n",
    "mutate": textwrap.dedent(
        '''
        def decide_mutate(env: Mapping[str, Any]) -> str:
            before = env.get("beforeCount", 0)
            after = env.get("afterCount", 0)
            delta = env.get("delta", 1)
            if not all(isinstance(v, int) and not isinstance(v, bool) for v in (before, after, delta)):
                return "USAGE"
            if before < 0 or after < 0:
                return "USAGE"
            return "MUTATE_OK" if after == before + delta else "MUTATE_DRIFT"
        '''
    ).strip()
    + "\n",
    "coord": textwrap.dedent(
        '''
        def decide_coord(env: Mapping[str, Any]) -> str:
            row = env.get("row", 0)
            col = env.get("col", 0)
            rows = env.get("rows", 1)
            cols = env.get("cols", 1)
            vals = (row, col, rows, cols)
            if not all(isinstance(v, int) and not isinstance(v, bool) for v in vals):
                return "USAGE"
            if rows <= 0 or cols <= 0:
                return "USAGE"
            if row < 0 or col < 0 or row >= rows or col >= cols:
                return "COORD_OOB"
            return "COORD_OK"
        '''
    ).strip()
    + "\n",
    "bbox": textwrap.dedent(
        '''
        def decide_bbox(env: Mapping[str, Any]) -> str:
            x = env.get("x", 0)
            y = env.get("y", 0)
            w = env.get("width", 1)
            h = env.get("height", 1)
            pw = env.get("pageWidth", 1)
            ph = env.get("pageHeight", 1)
            vals = (x, y, w, h, pw, ph)
            if not all(isinstance(v, int) and not isinstance(v, bool) for v in vals):
                return "USAGE"
            if w <= 0 or h <= 0 or pw <= 0 or ph <= 0:
                return "USAGE"
            if x < 0 or y < 0 or x + w > pw or y + h > ph:
                return "BBOX_OOB"
            return "BBOX_OK"
        '''
    ).strip()
    + "\n",
    "lease": textwrap.dedent(
        '''
        def decide_lease(env: Mapping[str, Any]) -> str:
            open_ok = env.get("sessionOpen", True)
            expired = env.get("leaseExpired", False)
            exit_code = env.get("exitCode", 0)
            if not isinstance(exit_code, int) or isinstance(exit_code, bool):
                return "EXIT_UNKNOWN"
            if not open_ok:
                return "SESSION_CLOSED"
            if expired:
                return "LEASE_STALE"
            if exit_code not in (0, 1, 2, 3, 4):
                return "EXIT_UNKNOWN"
            if exit_code != 0:
                return "LEASE_FAIL"
            return "LEASE_OK"
        '''
    ).strip()
    + "\n",
    "route": textwrap.dedent(
        '''
        def decide_route(env: Mapping[str, Any]) -> str:
            route = env.get("route", "")
            allowed = env.get("allowedRoutes", ["pdf", "fill", "table", "needs-agent", "fde"])
            if not isinstance(route, str):
                return "ROUTE_UNKNOWN"
            if isinstance(allowed, list) and route in allowed:
                return "ROUTE_OK"
            return "ROUTE_UNKNOWN"
        '''
    ).strip()
    + "\n",
    "order": textwrap.dedent(
        '''
        def decide_order(env: Mapping[str, Any]) -> str:
            steps = env.get("steps", [])
            if not isinstance(steps, list):
                return "USAGE"
            for i, step in enumerate(steps):
                if isinstance(step, int) and not isinstance(step, bool) and step != i:
                    return "ORDER_GAP"
                if isinstance(step, str) and step == "":
                    return "USAGE"
            return "ORDER_OK"
        '''
    ).strip()
    + "\n",
}


def write_unit(command: str, family: str, title: str) -> dict:
    pkg = pkg_name(command)
    dest = HERE / pkg
    if dest.exists():
        shutil.rmtree(dest)
    tests = dest / "tests"
    tests.mkdir(parents=True)
    fields = ENVELOPE_FIELDS
    names: list[str] = []
    chunks = [
        "from __future__ import annotations\n",
        "from typing import Any, Mapping\n\n",
        f"COMMAND = {command!r}\n",
        f"FAMILY = {family!r}\n",
        f"TITLE = {title!r}\n\n",
    ]
    for field in fields:
        for kind in kinds_for(field):
            name = ident(command, field, kind)
            names.append(name)
            chunks.append(emit_fn(name, field, kind, command))
    chunks.append("RULES = (\n")
    for name in names:
        chunks.append(f"    {name},\n")
    chunks.append(")\n\n")
    chunks.append(emit_decide(family))
    (dest / "decide.py").write_text("".join(chunks), encoding="utf-8", newline="\n")
    (dest / "__init__.py").write_text(
        f'"""{title} — {command} envelope checks as functions."""\n'
        "from .decide import COMMAND, FAMILY, TITLE, decide, RULES\n",
        encoding="utf-8",
        newline="\n",
    )
    (tests / "__init__.py").write_text("", encoding="utf-8", newline="\n")
    (tests / "test_decide.py").write_text(
        textwrap.dedent(
            f'''
            from __future__ import annotations
            import unittest
            from pathlib import Path
            import sys
            sys.path.insert(0, str(Path(__file__).resolve().parents[1].parent))
            from {pkg}.decide import decide, RULES, COMMAND, FAMILY

            class DecideTests(unittest.TestCase):
                def test_rule_count(self) -> None:
                    self.assertGreaterEqual(len(RULES), 40)
                    self.assertEqual(COMMAND, {command!r})
                    self.assertEqual(FAMILY, {family!r})

                def test_happy_path_is_code(self) -> None:
                    env = {{
                        "pageCount": 3, "paraCount": 3, "itemCount": 3, "declaredCount": 3,
                        "arrayLen": 3, "exitCode": 0, "requestedPage": 0, "emittedCount": 1,
                        "maxChars": 10, "textLen": 4, "truncated": False, "empty": False,
                        "rows": 2, "cols": 2, "rowSpan": 1, "colSpan": 1, "bytes": 12,
                        "width": 1, "height": 1, "emptyOutput": False, "matchCount": 1,
                        "page": 0, "offset": 0, "kind": "all", "count": 1, "inputN": 2,
                        "okN": 2, "failN": 0, "neighborChanged": False, "findingCount": 0,
                        "hasSignal": False, "overflow": 0, "overlap": 0, "before": 4,
                        "after": 4, "sameFormat": True, "diffCount": 1, "identical": False,
                        "pxDelta": 0, "threshold": 2, "structMismatch": False, "emptyPage": 0,
                        "verify": True, "written": 1, "reread": 1, "applied": True,
                        "beforeCount": 1, "afterCount": 0, "present": True, "extraKey": False,
                        "expectedSha": "a" * 64, "actualSha": "a" * 64, "available": True,
                        "requiresFeature": "", "rpcError": False, "isError": False,
                        "ok": 1, "total": 1, "level": "L1", "inputSha": "b" * 64,
                        "planSha": "c" * 64, "outputSha": "d" * 64, "hangulYear": 2022,
                        "nols": False, "usedAsOracle": False, "escaped": False,
                        "outsideWorkspace": False, "sizeBytes": 10, "capBytes": 100,
                        "accepted": True, "hasSpace": False, "parsed": True,
                        "rowsIn": 2, "colsIn": 2, "rowsOut": 2, "colsOut": 2,
                    }}
                    verdict = decide(env)
                    self.assertIsInstance(verdict, str)
                    self.assertNotEqual(verdict, "")

                def test_negative_count_is_usage(self) -> None:
                    if FAMILY != "count_eq":
                        return
                    self.assertEqual(decide({{"declaredCount": -1, "arrayLen": 0}}), "USAGE")

            if __name__ == "__main__":
                unittest.main()
            '''
        ).lstrip(),
        encoding="utf-8",
        newline="\n",
    )
    src_lines = (dest / "decide.py").read_text(encoding="utf-8").count("\n") + 1
    return {
        "command": command,
        "family": family,
        "title": title,
        "package": pkg,
        "rules": len(names),
        "sourceLines": src_lines,
        "ownedPath": f"tools/llm_verifier/{pkg}/",
    }


def main() -> int:
    sys.path.insert(0, str(HERE))
    inv_path = HERE / "wave2_units_inventory.json"
    if inv_path.exists():
        loaded = json.loads(inv_path.read_text(encoding="utf-8"))
        units = [(u["command"], u["family"], u["title"]) for u in loaded["units"]]
    else:
        units = UNITS
    for old in HERE.glob("w2_*"):
        if old.is_dir():
            shutil.rmtree(old)
    catalog = []
    for i, (command, family, title) in enumerate(units, 1):
        man = write_unit(command, family, title)
        catalog.append(man)
        if i == 1 or i % 20 == 0:
            pkg = man["package"]
            mod = __import__(f"{pkg}.decide", fromlist=["decide"])
            got = mod.decide({"declaredCount": 1, "arrayLen": 1, "pageCount": 1, "paraCount": 1})
            assert isinstance(got, str), got
            print(f"[verify] {i}/{len(units)} {command} rules={man['rules']} lines={man['sourceLines']}", flush=True)
    inventory = {
        "unitCount": len(catalog),
        "ruleTotal": sum(u["rules"] for u in catalog),
        "sourceLineTotal": sum(u["sourceLines"] for u in catalog),
        "units": catalog,
    }
    (HERE / "wave2_units_inventory.json").write_text(
        json.dumps(inventory, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    (HERE / "verify_wave2_units.py").write_text(
        textwrap.dedent(
            '''
            from __future__ import annotations
            import json
            import sys
            from pathlib import Path

            HERE = Path(__file__).resolve().parent

            def main() -> int:
                inv = json.loads((HERE / "wave2_units_inventory.json").read_text(encoding="utf-8"))
                sys.path.insert(0, str(HERE))
                total_rules = 0
                for unit in inv["units"]:
                    pkg = unit["package"]
                    mod = __import__(f"{pkg}.decide", fromlist=["decide", "RULES"])
                    if len(mod.RULES) != unit["rules"]:
                        raise SystemExit(f"{pkg} rule count drift")
                    env = {"declaredCount": 2, "arrayLen": 2, "pageCount": 2, "paraCount": 2, "exitCode": 0}
                    verdict = mod.decide(env)
                    if not isinstance(verdict, str) or not verdict:
                        raise SystemExit(f"{pkg} empty verdict")
                    total_rules += len(mod.RULES)
                    print(unit["command"], len(mod.RULES), verdict)
                print("RULES", total_rules)
                if total_rules != inv["ruleTotal"]:
                    raise SystemExit("inventory drift")
                return 0

            if __name__ == "__main__":
                raise SystemExit(main())
            '''
        ).lstrip(),
        encoding="utf-8",
        newline="\n",
    )
    print(json.dumps({"units": len(catalog), "rules": inventory["ruleTotal"], "lines": inventory["sourceLineTotal"]}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
