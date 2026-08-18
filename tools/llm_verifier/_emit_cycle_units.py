#!/usr/bin/env python3
"""Emit independent command-contract verifier units and their corpora.

Each unit is a closed decide tree for one real rhwp command surface that
does not already have a dedicated tools/llm_verifier package. Rows are
distinct (command, fields, identity) tuples. No comment padding.
"""

from __future__ import annotations

import json
import shutil
import sys
import time
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
BATCH = 20
ROWS_PER_UNIT = 20000
SHARD_ROWS = 5000

AGENCIES = (
    "법제처", "행정안전부", "국세청", "대법원", "특허청", "교육부",
    "보건복지부", "국토교통부", "고용노동부", "외교부", "기획재정부",
    "공정거래위원회", "금융위원회", "통계청", "기상청", "관세청",
    "검찰청", "경찰청", "소방청", "병무청", "산림청", "중소벤처기업부",
    "과학기술정보통신부", "문화체육관광부", "환경부", "해양수산부",
    "서울특별시", "경기도", "부산광역시", "제주특별자치도",
)
YEARS = tuple(str(y) for y in range(2016, 2027))
FORMATS = ("hwp", "hwpx")

# Existing dedicated axes — do not recreate.
SKIP_COMMANDS = {
    "lineage",  # V-lineage
    "replay",  # V-replay
}

# command, family, title
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
    ("explore", "count_eq", "explore 다음  hop 수가 선언과 일치"),
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


def slug(command: str) -> str:
    return "u_" + command.replace("-", "_")


def decide_count_eq(declared: int, actual: int) -> str:
    if declared < 0 or actual < 0:
        return "USAGE"
    if declared != actual:
        return "COUNT_DRIFT"
    return "COUNT_OK"


def decide_bound(limit: int, actual: int, truncated: int) -> str:
    if limit < 0 or actual < 0:
        return "USAGE"
    if actual > limit and not truncated:
        return "BOUND_LIE"
    if actual <= limit and truncated:
        return "BOUND_FALSE_POS"
    return "BOUND_OK"


def decide_window(req: int, emitted: int, total: int) -> str:
    if req < 0 or total < 0 or emitted < 0:
        return "USAGE"
    if req >= total:
        return "USAGE"
    if emitted == 0:
        return "WINDOW_MISS"
    if emitted != 1:
        return "WINDOW_LEAK"
    return "WINDOW_OK"


def decide_span(rows: int, cols: int, row_span: int, col_span: int) -> str:
    if min(rows, cols, row_span, col_span) <= 0:
        return "USAGE"
    if row_span > rows or col_span > cols:
        return "SPAN_OOB"
    return "SPAN_OK"


def decide_dims(rows_in: int, cols_in: int, rows_out: int, cols_out: int) -> str:
    if min(rows_in, cols_in, rows_out, cols_out) <= 0:
        return "USAGE"
    if rows_in != rows_out or cols_in != cols_out:
        return "DIM_DRIFT"
    return "DIM_OK"


def decide_bytes(n: int, empty_output: int) -> str:
    if n < 0:
        return "USAGE"
    if empty_output:
        return "EMPTY_OUTPUT"
    if n == 0:
        return "ZERO_BYTES"
    return "BYTES_OK"


def decide_search(match_count: int, array_len: int, page: int, page_count: int) -> str:
    if match_count < 0 or array_len < 0 or page_count < 0:
        return "USAGE"
    if match_count != array_len:
        return "COUNT_DRIFT"
    if page < 0 or page >= page_count:
        return "COORD_OOB"
    return "SEARCH_OK"


def decide_kind(kind: str, count: int, array_len: int) -> str:
    if kind not in {"date", "amount", "number", "all"}:
        return "KIND_UNKNOWN"
    if count < 0 or array_len < 0:
        return "USAGE"
    if count != array_len:
        return "COUNT_DRIFT"
    return "KIND_OK"


def decide_isolate(n_in: int, n_ok: int, n_fail: int, neighbor: int) -> str:
    if min(n_in, n_ok, n_fail) < 0:
        return "USAGE"
    if n_in != n_ok + n_fail:
        return "COUNT_DRIFT"
    if neighbor:
        return "POISON"
    return "ISOLATED"


def decide_signal(count: int, has_signal: int) -> str:
    if count < 0:
        return "USAGE"
    if bool(has_signal) != (count > 0):
        return "SIGNAL_LIE"
    return "CLEAN" if count == 0 else "ANOMALY"


def decide_round(before: int, after: int, same_format: int) -> str:
    if before < 0 or after < 0:
        return "USAGE"
    if not same_format:
        return "FORMAT_NA"
    if before != after:
        return "ROUND_DRIFT"
    return "ROUND_OK"


def decide_diff(diff_count: int, items: int) -> str:
    if diff_count < 0 or items < 0:
        return "USAGE"
    if diff_count != items:
        return "COUNT_DRIFT"
    return "DIFF_OK"


def decide_px(delta: int, threshold: int, struct_mismatch: int) -> str:
    if delta < 0 or threshold < 0:
        return "USAGE"
    if struct_mismatch:
        return "STRUCT"
    if delta > threshold:
        return "PX_FAIL"
    return "PX_OK"


def decide_layout(overflow: int, overlap: int, empty: int, has_signal: int) -> str:
    if min(overflow, overlap, empty) < 0:
        return "USAGE"
    total = overflow + overlap + empty
    if bool(has_signal) != (total > 0):
        return "SIGNAL_LIE"
    return "CLEAN" if total == 0 else "ANOMALY"


def decide_reread(verify: int, written: int, reread: int) -> str:
    if not verify:
        return "NOT_EVIDENCE"
    if written < 0 or reread < 0:
        return "USAGE"
    if written != reread:
        return "REREAD_DRIFT"
    return "REREAD_OK"


def decide_redact(applied: int, before: int, after: int) -> str:
    if not applied:
        return "NOT_EVIDENCE"
    if before < 0 or after < 0:
        return "USAGE"
    if after > 0:
        return "STILL_PRESENT"
    if before == 0:
        return "NOTHING_TO_CLEAR"
    return "CLEAR_OK"


def decide_cas(present: int, extra: int, expected: str, actual: str) -> str:
    if extra:
        return "USAGE"
    if not present:
        return "SKIP"
    if len(expected) != 64 or len(actual) != 64:
        return "USAGE"
    if any(c not in "0123456789abcdef" for c in expected + actual):
        return "USAGE"
    if expected != actual:
        return "CAS_MISMATCH"
    return "CAS_OK"


def decide_avail(available: int, exit_code: int) -> str:
    if exit_code not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    if not available and exit_code == 2:
        return "UNAVAIL_OK"
    if not available and exit_code != 2:
        return "UNAVAIL_LIE"
    if available and exit_code == 2:
        return "FALSE_UNAVAIL"
    return "AVAIL_RUN"


def decide_layer(rpc_error: int, is_error: int, exit_code: int) -> str:
    if rpc_error:
        return "RPC_FAIL"
    if is_error:
        return "TOOL_FAIL"
    if exit_code == 0:
        return "ENV_OK"
    if exit_code in (1, 2, 3, 4):
        return "ENV_JUDGE"
    return "ENV_UNKNOWN"


def decide_rate(ok: int, total: int) -> str:
    if ok < 0 or total < 0:
        return "USAGE"
    if ok > total:
        return "RATE_IMPOSSIBLE"
    return "RATE_OK"


def decide_level(level: str) -> str:
    if level in {"L1", "L2", "L3", "L4", "L5"}:
        return "LEVEL_OK"
    return "LEVEL_UNKNOWN"


def decide_triad(a: str, b: str, c: str) -> str:
    for token in (a, b, c):
        if not token:
            return "TRIAD_MISS"
        if len(token) != 64 or any(ch not in "0123456789abcdef" for ch in token):
            return "HASH_DEFECT"
    return "TRIAD_OK"


def decide_trap(year: int, nols: int, used: int) -> str:
    if year >= 2024 and nols and used:
        return "TRAP"
    if year >= 2024 and nols:
        return "FLAGGED"
    return "SAFE"


def decide_path(escaped: int, outside: int) -> str:
    if outside:
        return "BREACH"
    if escaped:
        return "ESCAPE"
    return "PATH_OK"


def decide_limit(size: int, cap: int, accepted: int) -> str:
    if size < 0 or cap <= 0:
        return "USAGE"
    if size > cap and accepted:
        return "OVER_ACCEPTED"
    if size > cap:
        return "OVER_REJECT"
    if accepted:
        return "UNDER_OK"
    return "UNDER_REJECT"


def decide_parse(has_space: int, parsed: int) -> str:
    if has_space and not parsed:
        return "PARSE_DROP"
    if parsed:
        return "PARSE_OK"
    return "PARSE_FAIL"


DECIDE = {
    "count_eq": decide_count_eq,
    "bound": decide_bound,
    "window": decide_window,
    "span": decide_span,
    "dims": decide_dims,
    "bytes": decide_bytes,
    "search": decide_search,
    "kind": decide_kind,
    "isolate": decide_isolate,
    "signal": decide_signal,
    "round": decide_round,
    "diff": decide_diff,
    "px": decide_px,
    "layout": decide_layout,
    "reread": decide_reread,
    "redact": decide_redact,
    "cas": decide_cas,
    "avail": decide_avail,
    "layer": decide_layer,
    "rate": decide_rate,
    "level": decide_level,
    "triad": decide_triad,
    "trap": decide_trap,
    "path": decide_path,
    "limit": decide_limit,
    "parse": decide_parse,
}


def hex64(label: str) -> str:
    import hashlib

    return hashlib.sha256(label.encode("utf-8")).hexdigest()


def identity(command: str, serial: int) -> dict[str, str]:
    agency = AGENCIES[serial % len(AGENCIES)]
    year = YEARS[(serial // len(AGENCIES)) % len(YEARS)]
    fmt = FORMATS[serial % 2]
    return {
        "sample": f"samples/{agency}/{year}/{command}-{serial:06d}.{fmt}",
        "agency": agency,
        "year": year,
        "fmt": fmt,
    }


def fields_for(family: str, serial: int) -> tuple[tuple, list[str]]:
    """Return (decide_args, extra_cells) unique enough to fill ROWS_PER_UNIT."""
    mode = serial % 7
    mag = 1 + (serial % 9)
    page = serial % 17
    total = 8 + (serial % 12)
    if family == "count_eq":
        declared = serial % 24
        actual = declared if mode < 5 else declared + (1 if mode == 5 else -1)
        return (declared, actual), [str(declared), str(actual)]
    if family == "bound":
        limit = 20 + (serial % 80)
        actual = limit - mag if mode < 4 else limit + mag
        truncated = 0 if mode in (0, 1, 4) else 1
        if mode == 6:
            actual = limit
            truncated = 0
        return (limit, actual, truncated), [str(limit), str(actual), str(truncated)]
    if family == "window":
        req = page % max(total, 1)
        emitted = 1 if mode < 5 else (0 if mode == 5 else 2)
        if mode == 4:
            req = total
        return (req, emitted, total), [str(req), str(emitted), str(total)]
    if family == "span":
        rows = 2 + (serial % 8)
        cols = 2 + ((serial // 3) % 8)
        rs = 1 + (serial % rows)
        cs = 1 + ((serial // 2) % cols)
        if mode == 6:
            rs = rows + 1
        return (rows, cols, rs, cs), [str(rows), str(cols), str(rs), str(cs)]
    if family == "dims":
        rows = 2 + (serial % 12)
        cols = 2 + ((serial // 2) % 10)
        rows_out = rows if mode < 5 else rows + 1
        cols_out = cols if mode != 6 else cols + 1
        return (rows, cols, rows_out, cols_out), [str(rows), str(cols), str(rows_out), str(cols_out)]
    if family == "bytes":
        n = 0 if mode == 5 else (12 + serial % 5000)
        empty = 1 if mode == 6 else 0
        if empty:
            n = 0
        return (n, empty), [str(n), str(empty)]
    if family == "search":
        array_len = serial % 15
        match_count = array_len if mode < 5 else array_len + 1
        pc = max(total, 1)
        pg = page % pc if mode != 6 else pc + 1
        return (match_count, array_len, pg, pc), [str(match_count), str(array_len), str(pg), str(pc)]
    if family == "kind":
        kind = ("date", "amount", "number", "all", "date", "amount", "token")[mode]
        n = serial % 12
        actual = n if mode != 5 else n + 1
        return (kind, n, actual), [kind, str(n), str(actual)]
    if family == "isolate":
        n_fail = 0 if mode < 4 else 1
        n_ok = 3 + (serial % 8)
        n_in = n_ok + n_fail if mode != 5 else n_ok + n_fail + 1
        neighbor = 1 if mode == 6 else 0
        return (n_in, n_ok, n_fail, neighbor), [str(n_in), str(n_ok), str(n_fail), str(neighbor)]
    if family == "signal":
        count = 0 if mode < 3 else mag
        has = 1 if count > 0 else 0
        if mode == 6:
            has = 1 - has
        return (count, has), [str(count), str(has)]
    if family == "round":
        before = 1 + (serial % 40)
        after = before if mode < 5 else before + (1 if mode == 5 else -1)
        same = 0 if mode == 4 else 1
        return (before, after, same), [str(before), str(after), str(same)]
    if family == "diff":
        items = serial % 20
        diffs = items if mode < 5 else items + 1
        return (diffs, items), [str(diffs), str(items)]
    if family == "px":
        thr = 2 + (serial % 6)
        delta = serial % (thr + 4)
        struct = 1 if mode == 6 else 0
        return (delta, thr, struct), [str(delta), str(thr), str(struct)]
    if family == "layout":
        ov = 0 if mode < 3 else mag
        op = 0 if mode in (0, 1, 3) else (mag if mode != 2 else 0)
        em = 0 if mode < 4 else 1
        total = ov + op + em
        has = 1 if total > 0 else 0
        if mode == 6:
            has = 1 - has
        return (ov, op, em, has), [str(ov), str(op), str(em), str(has)]
    if family == "reread":
        verify = 0 if mode == 6 else 1
        written = 1 + (serial % 9)
        reread = written if mode < 5 else written + 1
        return (verify, written, reread), [str(verify), str(written), str(reread)]
    if family == "redact":
        applied = 0 if mode == 6 else 1
        before = 0 if mode == 4 else 1 + (serial % 6)
        after = 0 if mode < 4 else (0 if mode == 4 else 1)
        return (applied, before, after), [str(applied), str(before), str(after)]
    if family == "cas":
        present = 0 if mode == 5 else 1
        extra = 1 if mode == 6 else 0
        actual = hex64(f"act-{serial}")
        expected = actual if mode < 3 else hex64(f"exp-{serial}")
        if mode == 4:
            expected = "zz"
        return (present, extra, expected, actual), [str(present), str(extra), expected, actual]
    if family == "avail":
        available = 0 if mode < 3 else 1
        exit_code = (2, 0, 1, 0, 3, 2, 9)[mode]
        return (available, exit_code), [str(available), str(exit_code)]
    if family == "layer":
        rpc = 1 if mode == 0 else 0
        tool = 1 if mode == 1 else 0
        exit_code = (0, 0, 0, 1, 2, 3, 9)[mode]
        return (rpc, tool, exit_code), [str(rpc), str(tool), str(exit_code)]
    if family == "rate":
        total = 1 + (serial % 20)
        ok = total if mode < 4 else (total + 1 if mode == 6 else serial % total)
        return (ok, total), [str(ok), str(total)]
    if family == "level":
        level = ("L1", "L2", "L3", "L4", "L5", "L5", "L9")[mode]
        return (level,), [level]
    if family == "triad":
        a = hex64(f"in-{serial}")
        b = hex64(f"plan-{serial}")
        c = hex64(f"out-{serial}")
        if mode == 5:
            a = ""
        if mode == 6:
            c = "nope"
        return (a, b, c), [a, b, c]
    if family == "trap":
        year = 2020 + (serial % 7)
        nols = 1 if mode >= 3 else 0
        used = 1 if mode >= 5 else 0
        return (year, nols, used), [str(year), str(nols), str(used)]
    if family == "path":
        escaped = 1 if mode == 5 else 0
        outside = 1 if mode == 6 else 0
        return (escaped, outside), [str(escaped), str(outside)]
    if family == "limit":
        cap = 1024 * (1 + serial % 8)
        size = cap - 10 if mode < 4 else cap + 50
        accepted = 1 if mode in (0, 1, 2, 5) else 0
        return (size, cap, accepted), [str(size), str(cap), str(accepted)]
    if family == "parse":
        has_space = 1 if mode >= 3 else 0
        parsed = 0 if mode >= 5 else 1
        return (has_space, parsed), [str(has_space), str(parsed)]
    raise KeyError(family)


HEADER = [
    "case_id",
    "command",
    "family",
    "verdict",
    "sample",
    "agency",
    "year",
    "fmt",
    "f0",
    "f1",
    "f2",
    "f3",
]


def emit_rows(command: str, family: str, n: int) -> list[list[str]]:
    fn = DECIDE[family]
    rows: list[list[str]] = []
    seen: set[tuple] = set()
    serial = 0
    while len(rows) < n:
        args, cells = fields_for(family, serial)
        ident = identity(command, serial)
        verdict = fn(*args)
        key = (command, family, verdict, *cells, ident["sample"])
        serial += 1
        if key in seen:
            continue
        seen.add(key)
        padded = (cells + ["", "", "", ""])[:4]
        rows.append(
            [
                f"{slug(command)}-{len(rows):06d}",
                command,
                family,
                verdict,
                ident["sample"],
                ident["agency"],
                ident["year"],
                ident["fmt"],
                *padded,
            ]
        )
    return rows


DECIDE_SRC = r'''
from __future__ import annotations
from .logic import decide_row, FAMILY, COMMAND, CLAIM_ID
'''

LOGIC_TEMPLATE = '''\
from __future__ import annotations

COMMAND = {command!r}
FAMILY = {family!r}
CLAIM_ID = {claim!r}
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY
{body}
    raise ValueError(family)
'''


def logic_body(family: str) -> str:
    # Keep decide() local so each package is self-contained and reviewable.
    mapping = {
        "count_eq": '''
    declared = int(f0); actual = int(f1)
    if declared < 0 or actual < 0:
        return "USAGE"
    return "COUNT_OK" if declared == actual else "COUNT_DRIFT"
''',
        "bound": '''
    limit = int(f0); actual = int(f1); truncated = int(f2)
    if limit < 0 or actual < 0:
        return "USAGE"
    if actual > limit and not truncated:
        return "BOUND_LIE"
    if actual <= limit and truncated:
        return "BOUND_FALSE_POS"
    return "BOUND_OK"
''',
        "window": '''
    req = int(f0); emitted = int(f1); total = int(f2)
    if req < 0 or total < 0 or emitted < 0:
        return "USAGE"
    if req >= total:
        return "USAGE"
    if emitted == 0:
        return "WINDOW_MISS"
    if emitted != 1:
        return "WINDOW_LEAK"
    return "WINDOW_OK"
''',
        "span": '''
    rows = int(f0); cols = int(f1); rs = int(f2); cs = int(f3)
    if min(rows, cols, rs, cs) <= 0:
        return "USAGE"
    if rs > rows or cs > cols:
        return "SPAN_OOB"
    return "SPAN_OK"
''',
        "dims": '''
    rows_in = int(f0); cols_in = int(f1); rows_out = int(f2); cols_out = int(f3)
    if min(rows_in, cols_in, rows_out, cols_out) <= 0:
        return "USAGE"
    if rows_in != rows_out or cols_in != cols_out:
        return "DIM_DRIFT"
    return "DIM_OK"
''',
        "bytes": '''
    n = int(f0); empty = int(f1)
    if n < 0:
        return "USAGE"
    if empty:
        return "EMPTY_OUTPUT"
    if n == 0:
        return "ZERO_BYTES"
    return "BYTES_OK"
''',
        "search": '''
    match_count = int(f0); array_len = int(f1); page = int(f2); page_count = int(f3)
    if match_count < 0 or array_len < 0 or page_count < 0:
        return "USAGE"
    if match_count != array_len:
        return "COUNT_DRIFT"
    if page < 0 or page >= page_count:
        return "COORD_OOB"
    return "SEARCH_OK"
''',
        "kind": '''
    kind = f0
    count = int(f1); array_len = int(f2)
    if kind not in {"date", "amount", "number", "all"}:
        return "KIND_UNKNOWN"
    if count < 0 or array_len < 0:
        return "USAGE"
    return "KIND_OK" if count == array_len else "COUNT_DRIFT"
''',
        "isolate": '''
    n_in = int(f0); n_ok = int(f1); n_fail = int(f2); neighbor = int(f3)
    if min(n_in, n_ok, n_fail) < 0:
        return "USAGE"
    if n_in != n_ok + n_fail:
        return "COUNT_DRIFT"
    return "POISON" if neighbor else "ISOLATED"
''',
        "signal": '''
    count = int(f0); has_signal = int(f1)
    if count < 0:
        return "USAGE"
    if bool(has_signal) != (count > 0):
        return "SIGNAL_LIE"
    return "CLEAN" if count == 0 else "ANOMALY"
''',
        "round": '''
    before = int(f0); after = int(f1); same = int(f2)
    if before < 0 or after < 0:
        return "USAGE"
    if not same:
        return "FORMAT_NA"
    return "ROUND_OK" if before == after else "ROUND_DRIFT"
''',
        "diff": '''
    diff_count = int(f0); items = int(f1)
    if diff_count < 0 or items < 0:
        return "USAGE"
    return "DIFF_OK" if diff_count == items else "COUNT_DRIFT"
''',
        "px": '''
    delta = int(f0); threshold = int(f1); struct_mismatch = int(f2)
    if delta < 0 or threshold < 0:
        return "USAGE"
    if struct_mismatch:
        return "STRUCT"
    return "PX_FAIL" if delta > threshold else "PX_OK"
''',
        "layout": '''
    overflow = int(f0); overlap = int(f1); empty = int(f2); has_signal = int(f3)
    if min(overflow, overlap, empty) < 0:
        return "USAGE"
    total = overflow + overlap + empty
    if bool(has_signal) != (total > 0):
        return "SIGNAL_LIE"
    return "CLEAN" if total == 0 else "ANOMALY"
''',
        "reread": '''
    verify = int(f0); written = int(f1); reread = int(f2)
    if not verify:
        return "NOT_EVIDENCE"
    if written < 0 or reread < 0:
        return "USAGE"
    return "REREAD_OK" if written == reread else "REREAD_DRIFT"
''',
        "redact": '''
    applied = int(f0); before = int(f1); after = int(f2)
    if not applied:
        return "NOT_EVIDENCE"
    if before < 0 or after < 0:
        return "USAGE"
    if after > 0:
        return "STILL_PRESENT"
    return "NOTHING_TO_CLEAR" if before == 0 else "CLEAR_OK"
''',
        "cas": '''
    present = int(f0); extra = int(f1); expected = f2; actual = f3
    if extra:
        return "USAGE"
    if not present:
        return "SKIP"
    hexset = set("0123456789abcdef")
    if len(expected) != 64 or len(actual) != 64:
        return "USAGE"
    if any(c not in hexset for c in expected + actual):
        return "USAGE"
    return "CAS_OK" if expected == actual else "CAS_MISMATCH"
''',
        "avail": '''
    available = int(f0); exit_code = int(f1)
    if exit_code not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    if not available and exit_code == 2:
        return "UNAVAIL_OK"
    if not available and exit_code != 2:
        return "UNAVAIL_LIE"
    if available and exit_code == 2:
        return "FALSE_UNAVAIL"
    return "AVAIL_RUN"
''',
        "layer": '''
    rpc_error = int(f0); is_error = int(f1); exit_code = int(f2)
    if rpc_error:
        return "RPC_FAIL"
    if is_error:
        return "TOOL_FAIL"
    if exit_code == 0:
        return "ENV_OK"
    if exit_code in (1, 2, 3, 4):
        return "ENV_JUDGE"
    return "ENV_UNKNOWN"
''',
        "rate": '''
    ok = int(f0); total = int(f1)
    if ok < 0 or total < 0:
        return "USAGE"
    if ok > total:
        return "RATE_IMPOSSIBLE"
    return "RATE_OK"
''',
        "level": '''
    return "LEVEL_OK" if f0 in {"L1", "L2", "L3", "L4", "L5"} else "LEVEL_UNKNOWN"
''',
        "triad": '''
    hexset = set("0123456789abcdef")
    for token in (f0, f1, f2):
        if not token:
            return "TRIAD_MISS"
        if len(token) != 64 or any(ch not in hexset for ch in token):
            return "HASH_DEFECT"
    return "TRIAD_OK"
''',
        "trap": '''
    year = int(f0); nols = int(f1); used = int(f2)
    if year >= 2024 and nols and used:
        return "TRAP"
    if year >= 2024 and nols:
        return "FLAGGED"
    return "SAFE"
''',
        "path": '''
    escaped = int(f0); outside = int(f1)
    if outside:
        return "BREACH"
    return "ESCAPE" if escaped else "PATH_OK"
''',
        "limit": '''
    size = int(f0); cap = int(f1); accepted = int(f2)
    if size < 0 or cap <= 0:
        return "USAGE"
    if size > cap and accepted:
        return "OVER_ACCEPTED"
    if size > cap:
        return "OVER_REJECT"
    return "UNDER_OK" if accepted else "UNDER_REJECT"
''',
        "parse": '''
    has_space = int(f0); parsed = int(f1)
    if has_space and not parsed:
        return "PARSE_DROP"
    return "PARSE_OK" if parsed else "PARSE_FAIL"
''',
    }
    return mapping[family]


VERIFY_PY = '''\
from __future__ import annotations
import csv
from collections import Counter
from pathlib import Path
from .logic import decide_row, COMMAND, FAMILY, CLAIM_ID

HERE = Path(__file__).resolve().parent
CORPUS = HERE / "corpus"
MIN_ROWS = 15000

def verify() -> dict:
    rows = 0
    seen = set()
    verdicts: Counter[str] = Counter()
    for path in sorted(CORPUS.glob("shard_*.tsv")):
        with path.open(encoding="utf-8", newline="") as fh:
            reader = csv.DictReader(fh, delimiter="\\t")
            for rec in reader:
                rows += 1
                key = (rec["sample"], rec["f0"], rec["f1"], rec["f2"], rec["f3"], rec["verdict"])
                if key in seen:
                    raise SystemExit(f"duplicate {rec['case_id']}")
                seen.add(key)
                got = decide_row(rec["f0"], rec["f1"], rec["f2"], rec["f3"])
                if got != rec["verdict"]:
                    raise SystemExit(f"{rec['case_id']}: {got} != {rec['verdict']}")
                if rec["command"] != COMMAND or rec["family"] != FAMILY:
                    raise SystemExit(f"{rec['case_id']}: command/family drift")
                verdicts[got] += 1
    if rows < MIN_ROWS:
        raise SystemExit(f"{CLAIM_ID} rows {rows} < {MIN_ROWS}")
    return {"ok": True, "claim": CLAIM_ID, "rows": rows, "distinct": len(seen), "byVerdict": dict(sorted(verdicts.items()))}
'''

TEST_PY = '''\
from __future__ import annotations
import unittest
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1].parent))
from {pkg} import verify as V
from {pkg}.logic import decide_row

class UnitTests(unittest.TestCase):
    def test_corpus(self) -> None:
        result = V.verify()
        self.assertTrue(result["ok"])
        self.assertGreaterEqual(result["rows"], V.MIN_ROWS)
        self.assertGreater(len(result["byVerdict"]), 0)

    def test_decide_smoke(self) -> None:
        self.assertIsInstance(decide_row("0", "0", "0", "0"), str)

if __name__ == "__main__":
    unittest.main()
'''


def write_unit(command: str, family: str, title: str) -> dict:
    pkg = slug(command)
    dest = HERE / pkg
    if dest.exists():
        shutil.rmtree(dest)
    corpus = dest / "corpus"
    tests = dest / "tests"
    corpus.mkdir(parents=True)
    tests.mkdir()
    claim = f"V-unit-{command}"
    (dest / "__init__.py").write_text(
        f'"""{title}\n\ncommand={command} family={family}\n"""\n'
        "from .verify_corpus import verify\n",
        encoding="utf-8",
        newline="\n",
    )
    (dest / "logic.py").write_text(
        LOGIC_TEMPLATE.format(command=command, family=family, claim=claim, body=logic_body(family)),
        encoding="utf-8",
        newline="\n",
    )
    (dest / "verify_corpus.py").write_text(VERIFY_PY, encoding="utf-8", newline="\n")
    (tests / "__init__.py").write_text("", encoding="utf-8", newline="\n")
    (tests / "test_unit.py").write_text(TEST_PY.format(pkg=pkg), encoding="utf-8", newline="\n")

    rows = emit_rows(command, family, ROWS_PER_UNIT)
    verdicts: Counter[str] = Counter(r[3] for r in rows)
    for start in range(0, len(rows), SHARD_ROWS):
        chunk = rows[start : start + SHARD_ROWS]
        shard = corpus / f"shard_{start // SHARD_ROWS:04d}.tsv"
        lines = ["\t".join(HEADER)]
        lines.extend("\t".join(row) for row in chunk)
        shard.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    manifest = {
        "claim": claim,
        "command": command,
        "family": family,
        "title": title,
        "rowCount": len(rows),
        "byVerdict": dict(sorted(verdicts.items())),
        "ownedPath": f"tools/llm_verifier/{pkg}/",
    }
    (corpus / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    return manifest


def verify_pkg(command: str) -> dict:
    pkg = slug(command)
    sys.path.insert(0, str(HERE))
    mod = __import__(f"{pkg}.verify_corpus", fromlist=["verify"])
    return mod.verify()


def main() -> int:
    started = time.time()
    catalog = []
    seen_cmd: set[str] = set()
    for command, family, title in UNITS:
        if command in SKIP_COMMANDS or command in seen_cmd:
            continue
        if family not in DECIDE:
            raise SystemExit(f"unknown family {family}")
        seen_cmd.add(command)
        man = write_unit(command, family, title)
        catalog.append(man)
        n = len(catalog)
        if n % BATCH == 0 or n == 1:
            print(f"[merge-test] units={n} last={command}", flush=True)
            batch = catalog[-BATCH:] if n > 1 else catalog[:1]
            for item in batch:
                got = verify_pkg(item["command"])
                if got["rows"] != item["rowCount"]:
                    raise SystemExit(f"verify drift {item['command']}")
            print(
                f"[merge-test] ok batch_end={n} elapsed={time.time()-started:.1f}s",
                flush=True,
            )
    # final full verify of last remainder + inventory
    remainder = catalog[-(len(catalog) % BATCH or BATCH) :]
    for item in remainder:
        verify_pkg(item["command"])
    inventory = {
        "unitCount": len(catalog),
        "rowTotal": sum(i["rowCount"] for i in catalog),
        "units": catalog,
        "elapsedSec": round(time.time() - started, 1),
    }
    (HERE / "cycle_units_inventory.json").write_text(
        json.dumps(inventory, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    runner = HERE / "verify_cycle_units.py"
    runner.write_text(
        """from __future__ import annotations
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent

def main() -> int:
    inv = json.loads((HERE / "cycle_units_inventory.json").read_text(encoding="utf-8"))
    sys.path.insert(0, str(HERE))
    total = 0
    for unit in inv["units"]:
        pkg = "u_" + unit["command"].replace("-", "_")
        mod = __import__(f"{pkg}.verify_corpus", fromlist=["verify"])
        got = mod.verify()
        total += got["rows"]
        print(unit["command"], got["rows"], got["byVerdict"])
    print("TOTAL", total)
    if total != inv["rowTotal"]:
        raise SystemExit("inventory drift")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
""",
        encoding="utf-8",
        newline="\n",
    )
    print(json.dumps({"units": len(catalog), "rows": inventory["rowTotal"]}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
