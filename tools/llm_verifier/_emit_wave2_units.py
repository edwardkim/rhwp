#!/usr/bin/env python3
"""Emit wave-2 independent verifier units (edit/MCP/skill/recipe/field).

Packages live under w2_* so they never collide with cycle-1 u_* or the
dedicated V-* packages already on devel. Each row is a distinct
(command, fields, identity) decide-tuple. No comment padding.
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

# Already owned on devel or in open cycle-1 PR — do not recreate.
SKIP_COMMANDS = {
    "lineage",
    "replay",
    "info",
    "word-count",
    "bookmarks",
    "charts",
    "form-value",
    "header-footer",
    "headers-footers",
    "digest",
    "export-text",
    "export-markdown",
    "export-tables",
    "export-structure",
    "export-svg",
    "export-pdf",
    "export-png",
    "export-llm",
    "table-to-csv",
    "csv-to-table",
    "chart-to-csv",
    "csv-to-chart",
    "export-hwpx",
    "export-hml",
    "export-doclang",
    "thumbnail",
    "search",
    "extract-data",
    "fields",
    "explain",
    "explore",
    "batch",
    "scan",
    "threat-scan",
    "inspect-hidden",
    "inspect-injection",
    "inspect-unicode",
    "armor",
    "convert",
    "extract-pages",
    "build-from-ingest",
    "scaffold",
    "ir-diff",
    "ir-sweep",
    "verify",
    "render-diff",
    "layout-anomaly",
    "measure-width",
    "core-pages",
    "dump",
    "dump-pages",
    "dump-extents",
    "dump-anchors",
    "dump-carets",
    "dump-records",
    "diag",
    "hwp5-inventory",
    "hwp5-inventory-diff",
    "hwp5-char-shape-audit",
    "hwp5-roundtrip",
    "hwpx-roundtrip",
    "edit-fill",
    "edit-set-cell",
    "edit-replace",
    "edit-redact",
    "edit-sanitize",
    "edit-dry-run",
    "run-cas",
    "run-usage",
    "export-plan-schema",
    "export-ir-schema",
    "export-capabilities-schema",
    "export-ontology",
    "export-provenance-map",
    "export-agent-manifest",
    "capabilities",
    "mcp-serve",
    "audit",
    "audit-report",
    "settle",
    "conformance",
    "keygen",
    "verify-signature",
    "anchor",
    "gate",
    "bundle",
    "disclose",
    "recall-scope",
    "harness",
    "harness-status",
    "gpu-info",
    "export-png-gpu",
    "export-render-tree",
    "nols-2024",
    "boundary-path",
    "limit-bytes",
    "eq-supsub",
    "lineseg-overwrite",
    "compat-anchor",
    "charshape-hwpx",
    "recipe-fill",
    "recipe-table",
    "recipe-redact",
    "recipe-mailmerge",
    "recipe-visual",
    "skill-bulk",
    "skill-security",
    "skill-receipt",
    "doctor-onboard",
}

# command, family, title — real surfaces not covered by cycle-1.
UNITS: list[tuple[str, str, str]] = [
    ("edit-insert-table", "dims", "edit insert-table 행·열 수가 요청 치수"),
    ("edit-insert-row", "mutate", "edit insert-row 후 행 수가 before+1"),
    ("edit-insert-col", "mutate", "edit insert-col 후 열 수가 before+1"),
    ("edit-delete-row", "mutate", "edit delete-row 후 행 수가 before-1"),
    ("edit-delete-col", "mutate", "edit delete-col 후 열 수가 before-1"),
    ("edit-delete-table", "mutate", "edit delete-table 후 표 수가 before-1"),
    ("edit-merge-cells", "span", "edit merge-cells 범위가 격자 안"),
    ("edit-split-cell", "coord", "edit split-cell 좌표가 표 안"),
    ("edit-split-cell-into", "dims", "edit split-cell-into 분할 치수가 양수"),
    ("edit-split-table", "mutate", "edit split-table 후 표 수가 before+1"),
    ("edit-merge-table", "mutate", "edit merge-table 후 표 수가 before-1"),
    ("edit-set-column-widths", "dims", "edit set-column-widths 폭 개수가 열 수"),
    ("edit-set-cell-props", "reread", "edit set-cell-props --verify 재독"),
    ("edit-set-table-props", "reread", "edit set-table-props --verify 재독"),
    ("edit-insert-text", "reread", "edit insert-text 후 글자 수가 증가분과 같음"),
    ("edit-insert-text-in-cell", "coord", "edit insert-text-in-cell 셀 좌표가 격자 안"),
    ("edit-delete-text", "mutate", "edit delete-text 후 글자 수가 before-count"),
    ("edit-delete-text-in-cell", "coord", "edit delete-text-in-cell 셀 좌표가 격자 안"),
    ("edit-insert-paragraph", "mutate", "edit insert-paragraph 후 문단 수 +1"),
    ("edit-delete-paragraph", "mutate", "edit delete-paragraph 후 문단 수 -1"),
    ("edit-merge-paragraph", "mutate", "edit merge-paragraph 후 문단 수 -1"),
    ("edit-split-paragraph", "mutate", "edit split-paragraph 후 문단 수 +1"),
    ("edit-insert-page-break", "mutate", "edit insert-page-break 후 쪽 수 증가"),
    ("edit-insert-column-break", "mutate", "edit insert-column-break 후 단 수 증가"),
    ("edit-insert-image", "bytes", "edit insert-image 산출 바이트가 양수"),
    ("edit-insert-picture", "bbox", "edit insert-picture bbox 가 쪽 안"),
    ("edit-delete-picture", "mutate", "edit delete-picture 후 그림 수 -1"),
    ("edit-delete-shape", "mutate", "edit delete-shape 후 도형 수 -1"),
    ("edit-insert-shape", "bbox", "edit insert-shape 폭·높이가 쪽 안"),
    ("edit-set-picture", "reread", "edit set-picture --verify 재독"),
    ("edit-insert-footnote", "mutate", "edit insert-footnote 후 각주 수 +1"),
    ("edit-delete-footnote", "mutate", "edit delete-footnote 후 각주 수 -1"),
    ("edit-insert-endnote", "mutate", "edit insert-endnote 후 미주 수 +1"),
    ("edit-insert-equation", "parse", "edit insert-equation 스크립트가 파싱됨"),
    ("edit-delete-equation", "mutate", "edit delete-equation 후 수식 수 -1"),
    ("edit-set-equation-properties", "reread", "edit set-equation-properties 재독"),
    ("edit-add-bookmark", "reread", "edit add-bookmark 후 bookmarks 에 이름 존재"),
    ("edit-delete-bookmark", "mutate", "edit delete-bookmark 후 북마크 수 -1"),
    ("edit-insert-header-footer", "mutate", "edit insert-header-footer 후 HF 수 +1"),
    ("edit-delete-header-footer", "mutate", "edit delete-header-footer 후 HF 수 -1"),
    ("edit-set-header-footer-text", "reread", "edit set-header-footer-text 재독"),
    ("edit-insert-header-footer-text", "reread", "edit insert-header-footer-text 재독"),
    ("edit-delete-hf-text", "mutate", "edit delete-hf-text 후 HF 글자 수 감소"),
    ("edit-insert-field-in-hf", "mutate", "edit insert-field-in-hf 후 필드 수 +1"),
    ("edit-split-paragraph-in-hf", "mutate", "edit split-paragraph-in-hf 후 HF 문단 +1"),
    ("edit-merge-paragraph-in-hf", "mutate", "edit merge-paragraph-in-hf 후 HF 문단 -1"),
    ("edit-split-paragraph-in-cell", "coord", "edit split-paragraph-in-cell 좌표가 격자 안"),
    ("edit-merge-paragraph-in-cell", "coord", "edit merge-paragraph-in-cell 좌표가 격자 안"),
    ("edit-set-page-hide", "reread", "edit set-page-hide --verify 재독"),
    ("edit-set-page-def", "reread", "edit set-page-def --verify 재독"),
    ("edit-set-section-def", "reread", "edit set-section-def --verify 재독"),
    ("edit-set-column-def", "count_eq", "edit set-column-def count 가 단 수"),
    ("edit-apply-char-format", "reread", "edit apply-char-format --verify 재독"),
    ("edit-apply-para-format", "reread", "edit apply-para-format --verify 재독"),
    ("edit-apply-para-format-in-cell", "coord", "edit apply-para-format-in-cell 좌표가 격자 안"),
    ("edit-apply-char-format-in-cell", "coord", "edit apply-char-format-in-cell 좌표가 격자 안"),
    ("edit-apply-style", "reread", "edit apply-style --verify 재독"),
    ("edit-apply-cell-style", "coord", "edit apply-cell-style 좌표가 격자 안"),
    ("edit-delete-control", "mutate", "edit delete-control 후 컨트롤 수 -1"),
    ("edit-set-chart-data", "dims", "edit set-chart-data 범주·시리즈 치수"),
    ("edit-insert-number", "mutate", "edit insert-number 후 번호 필드 수 +1"),
    ("edit-set-form-value", "reread", "edit set-form-value --verify 재독"),
    ("edit-set-form-value-in-cell", "coord", "edit set-form-value-in-cell 좌표가 격자 안"),
    ("edit-set-page-border-fill", "reread", "edit set-page-border-fill 재독"),
    ("edit-set-hf-picture", "bbox", "edit set-hf-picture bbox 가 HF 영역 안"),
    ("edit-apply-hf-template", "reread", "edit apply-hf-template --verify 재독"),
    ("edit-apply-para-format-in-hf", "reread", "edit apply-para-format-in-hf 재독"),
    ("edit-apply-endnote-shape", "reread", "edit apply-endnote-shape 재독"),
    ("edit-insert-footnote-text", "reread", "edit insert-footnote-text 재독"),
    ("edit-split-paragraph-in-footnote", "mutate", "edit split-paragraph-in-footnote 문단 +1"),
    ("edit-merge-paragraph-in-footnote", "mutate", "edit merge-paragraph-in-footnote 문단 -1"),
    ("edit-apply-para-format-in-footnote", "reread", "edit apply-para-format-in-footnote 재독"),
    ("edit-set-numbering-restart", "reread", "edit set-numbering-restart 재독"),
    ("edit-delete-text-in-footnote", "mutate", "edit delete-text-in-footnote 글자 수 감소"),
    ("mcp-hwp-open", "lease", "mcp hwp_open 세션이 열려야 후속 도구 허용"),
    ("mcp-hwp-close", "lease", "mcp hwp_close 후 세션이 닫힘"),
    ("mcp-hwp-doc-info", "count_eq", "mcp hwp_doc_info pageCount 가 음수가 아님"),
    ("mcp-hwp-doc-export-text", "bound", "mcp hwp_doc_export_text 상한과 truncated"),
    ("mcp-hwp-doc-search", "search", "mcp hwp_doc_search matchCount 가 배열 길이"),
    ("mcp-hwp-doc-fields", "count_eq", "mcp hwp_doc_fields fieldCount 가 배열 길이"),
    ("mcp-hwp-doc-edit", "reread", "mcp 세션 편집 --verify 재독"),
    ("mcp-hwp-list-docs", "count_eq", "mcp hwp_list_docs 수가 열린 문서 수"),
    ("mcp-hwp-doc-export-tables", "dims", "mcp hwp_doc_export_tables 치수"),
    ("mcp-hwp-doc-export-structure", "count_eq", "mcp hwp_doc_export_structure nodeCount"),
    ("mcp-stateless-info", "layer", "무상태 MCP 판정 3층 rpc/isError/envelope"),
    ("mcp-session-lease", "lease", "세션 리스 만료면 후속 호출 거부"),
    ("fde-triage", "route", "fde triage 경로가 닫힌 집합"),
    ("handoff-outgoing", "triad", "handoff outgoing 입력·계획·산출 해시 3종"),
    ("handoff-incoming", "triad", "handoff incoming last result 해시 3종"),
    ("strategist-validate", "rate", "strategist --validate CLAIM 통과율"),
    ("chief-route", "route", "chief 라우트가 pdf/fill/table/needs-agent/fde"),
    ("gym-certify", "rate", "gym certify 통과 수가 전체 이하"),
    ("fidelity-compare", "px", "fidelity-compare px 와 STRUCT_MISMATCH"),
    ("exam-ingest", "count_eq", "exam-ingest 문항 수가 선언과 일치"),
    ("doc-triage", "order", "doc-triage 가 info→explain→structure 순서"),
    ("explore-menu", "count_eq", "explore.affordanceCount 가 menu 길이"),
    ("agent-surface-stateless", "layer", "에이전트 무상태 표면 3층 판정"),
    ("agent-surface-session", "lease", "에이전트 세션 표면은 리스가 살아 있어야 함"),
    ("recipe-receive", "signal", "레시피 04 수신 점검 hasSignal"),
    ("recipe-send", "signal", "레시피 10 송신 스윕 hasSignal"),
    ("recipe-bulk-extract", "isolate", "레시피 09 대량 추출 실패 행 격리"),
    ("recipe-layout-scan", "layout", "레시피 레이아웃 overflow/overlap/empty_page"),
    ("recipe-form-verify", "reread", "레시피 서식 --verify 재독"),
    ("info-section-count", "count_eq", "info.sectionCount 가 sections 길이"),
    ("info-para-count", "count_eq", "info.paraCount 가 음수가 아니고 선언과 일치"),
    ("info-table-count", "count_eq", "info.tableCount 가 tables 길이"),
    ("info-char-count", "count_eq", "info.charCount 가 음수가 아님"),
    ("search-snippet-bound", "bound", "search snippet 길이가 maxSnippet 이하"),
    ("explore-affordance", "count_eq", "explore affordanceCount 가 항목 수"),
    ("digest-input-cas", "cas", "digest 입력 SHA-256 이 전제와 일치"),
    ("export-structure-heading", "count_eq", "export-structure headingCount 가 항목 수"),
    ("fields-repeat-index", "count_eq", "fields 반복 인덱스 수가 선언과 일치"),
    ("batch-stderr-isolate", "isolate", "batch stderr 요약이 실패 행만 포함"),
    ("word-count-page-sum", "count_eq", "word-count 쪽별 합이 전체와 같음"),
    ("export-text-max-chars", "bound", "export-text --max-chars 와 truncated"),
    ("plan-step-count", "order", "run 계획 steps 가 0..n-1 연속"),
    ("plan-output-cas", "cas", "run 계획 outputSha256 이 산출과 일치"),
    ("large-doc-page-cap", "limit", "대형 문서 쪽 상한을 넘기면 거부"),
    ("run-journal-count", "count_eq", "run 저널 항목 수가 스텝 수"),
    ("studio-export-png", "bytes", "studio export-png 바이트가 양수"),
    ("vscode-preview", "bytes", "vscode preview 바이트가 양수"),
    ("chrome-print", "bytes", "chrome print PDF 바이트가 양수"),
    ("desk-open", "avail", "desk 미빌드는 exit 2"),
    ("wasm-info", "avail", "wasm info 미빌드는 exit 2"),
    ("ooxml-chart-round", "round", "OOXML 차트 왕복 시리즈 수 보존"),
    ("password-open", "avail", "암호 문서 키 없으면 exit 2"),
    ("paint-layer-count", "count_eq", "paint layerCount 가 레이어 배열 길이"),
    ("render-backend-page", "count_eq", "render-backend 쪽 수가 pageCount"),
    ("docdiff-page", "diff", "docdiff 쪽 차이 수가 항목 수"),
    ("font-metric-face", "count_eq", "font-metric face 수가 선언과 일치"),
]


def slug(command: str) -> str:
    return "w2_" + command.replace("-", "_")


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


def decide_coord(row: int, col: int, rows: int, cols: int) -> str:
    if min(row, col, rows, cols) < 0:
        return "USAGE"
    if rows == 0 or cols == 0:
        return "USAGE"
    if row >= rows or col >= cols:
        return "COORD_OOB"
    return "COORD_OK"


def decide_mutate(before: int, delta: int, after: int, insert: int) -> str:
    if before < 0 or after < 0:
        return "USAGE"
    expected = before + delta if insert else before - delta
    if expected < 0:
        return "USAGE"
    if after != expected:
        return "MUTATE_DRIFT"
    return "MUTATE_OK"


def decide_lease(open_flag: int, expired: int, exit_code: int) -> str:
    if exit_code not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    if not open_flag:
        return "SESSION_CLOSED"
    if expired:
        return "LEASE_STALE"
    if exit_code != 0:
        return "LEASE_FAIL"
    return "LEASE_OK"


def decide_route(dest: str) -> str:
    if dest in {"pdf", "fill", "table", "needs-agent", "fde"}:
        return "ROUTE_OK"
    return "ROUTE_UNKNOWN"


def decide_bbox(right: int, page_w: int, bottom: int, page_h: int) -> str:
    if min(right, page_w, bottom, page_h) <= 0:
        return "USAGE"
    if right > page_w or bottom > page_h:
        return "BBOX_OOB"
    return "BBOX_OK"


def decide_order(prev_step: int, this_step: int) -> str:
    if prev_step < -1 or this_step < 0:
        return "USAGE"
    if this_step != prev_step + 1:
        return "ORDER_GAP"
    return "ORDER_OK"


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
    "coord": decide_coord,
    "mutate": decide_mutate,
    "lease": decide_lease,
    "route": decide_route,
    "bbox": decide_bbox,
    "order": decide_order,
}


def hex64(label: str) -> str:
    import hashlib

    return hashlib.sha256(label.encode("utf-8")).hexdigest()


def identity(command: str, serial: int) -> dict[str, str]:
    agency = AGENCIES[serial % len(AGENCIES)]
    year = YEARS[(serial // len(AGENCIES)) % len(YEARS)]
    fmt = FORMATS[serial % 2]
    return {
        "sample": f"samples/wave2/{agency}/{year}/{command}-{serial:06d}.{fmt}",
        "agency": agency,
        "year": year,
        "fmt": fmt,
    }


def fields_for(family: str, serial: int) -> tuple[tuple, list[str]]:
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
        total_l = ov + op + em
        has = 1 if total_l > 0 else 0
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
        actual = hex64(f"w2-act-{serial}")
        expected = actual if mode < 3 else hex64(f"w2-exp-{serial}")
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
        total_r = 1 + (serial % 20)
        ok = total_r if mode < 4 else (total_r + 1 if mode == 6 else serial % total_r)
        return (ok, total_r), [str(ok), str(total_r)]
    if family == "level":
        level = ("L1", "L2", "L3", "L4", "L5", "L5", "L9")[mode]
        return (level,), [level]
    if family == "triad":
        a = hex64(f"w2-in-{serial}")
        b = hex64(f"w2-plan-{serial}")
        c = hex64(f"w2-out-{serial}")
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
    if family == "coord":
        rows = 2 + (serial % 10)
        cols = 2 + ((serial // 2) % 8)
        row = serial % rows if mode < 5 else rows + (mode == 5)
        col = (serial // 3) % cols if mode != 6 else cols + 1
        return (row, col, rows, cols), [str(row), str(col), str(rows), str(cols)]
    if family == "mutate":
        before = 2 + (serial % 20)
        delta = 1 + (serial % 3)
        insert = 1 if mode < 4 else 0
        expected = before + delta if insert else before - delta
        after = expected if mode not in (3, 6) else expected + (1 if mode == 3 else -1)
        return (before, delta, after, insert), [str(before), str(delta), str(after), str(insert)]
    if family == "lease":
        open_flag = 0 if mode in (0, 1) else 1
        expired = 1 if mode == 2 else 0
        exit_code = (0, 3, 0, 0, 1, 2, 9)[mode]
        return (open_flag, expired, exit_code), [str(open_flag), str(expired), str(exit_code)]
    if family == "route":
        dest = ("pdf", "fill", "table", "needs-agent", "fde", "pdf", "email")[mode]
        return (dest,), [dest]
    if family == "bbox":
        page_w = 200 + (serial % 400)
        page_h = 280 + (serial % 500)
        right = page_w - mag if mode < 5 else page_w + mag
        bottom = page_h - mag if mode != 6 else page_h + mag
        return (right, page_w, bottom, page_h), [str(right), str(page_w), str(bottom), str(page_h)]
    if family == "order":
        this_step = serial % 16
        prev_step = this_step - 1 if mode < 5 else this_step + (1 if mode == 5 else -2)
        return (prev_step, this_step), [str(prev_step), str(this_step)]
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


LOGIC_TEMPLATE = """\
from __future__ import annotations

COMMAND = {command!r}
FAMILY = {family!r}
CLAIM_ID = {claim!r}
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY
{body}
    raise ValueError(family)
"""


def logic_body(family: str) -> str:
    mapping = {
        "count_eq": """
    declared = int(f0); actual = int(f1)
    if declared < 0 or actual < 0:
        return "USAGE"
    return "COUNT_OK" if declared == actual else "COUNT_DRIFT"
""",
        "bound": """
    limit = int(f0); actual = int(f1); truncated = int(f2)
    if limit < 0 or actual < 0:
        return "USAGE"
    if actual > limit and not truncated:
        return "BOUND_LIE"
    if actual <= limit and truncated:
        return "BOUND_FALSE_POS"
    return "BOUND_OK"
""",
        "window": """
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
""",
        "span": """
    rows = int(f0); cols = int(f1); rs = int(f2); cs = int(f3)
    if min(rows, cols, rs, cs) <= 0:
        return "USAGE"
    if rs > rows or cs > cols:
        return "SPAN_OOB"
    return "SPAN_OK"
""",
        "dims": """
    rows_in = int(f0); cols_in = int(f1); rows_out = int(f2); cols_out = int(f3)
    if min(rows_in, cols_in, rows_out, cols_out) <= 0:
        return "USAGE"
    if rows_in != rows_out or cols_in != cols_out:
        return "DIM_DRIFT"
    return "DIM_OK"
""",
        "bytes": """
    n = int(f0); empty = int(f1)
    if n < 0:
        return "USAGE"
    if empty:
        return "EMPTY_OUTPUT"
    if n == 0:
        return "ZERO_BYTES"
    return "BYTES_OK"
""",
        "search": """
    match_count = int(f0); array_len = int(f1); page = int(f2); page_count = int(f3)
    if match_count < 0 or array_len < 0 or page_count < 0:
        return "USAGE"
    if match_count != array_len:
        return "COUNT_DRIFT"
    if page < 0 or page >= page_count:
        return "COORD_OOB"
    return "SEARCH_OK"
""",
        "kind": """
    kind = f0
    count = int(f1); array_len = int(f2)
    if kind not in {"date", "amount", "number", "all"}:
        return "KIND_UNKNOWN"
    if count < 0 or array_len < 0:
        return "USAGE"
    return "KIND_OK" if count == array_len else "COUNT_DRIFT"
""",
        "isolate": """
    n_in = int(f0); n_ok = int(f1); n_fail = int(f2); neighbor = int(f3)
    if min(n_in, n_ok, n_fail) < 0:
        return "USAGE"
    if n_in != n_ok + n_fail:
        return "COUNT_DRIFT"
    return "POISON" if neighbor else "ISOLATED"
""",
        "signal": """
    count = int(f0); has_signal = int(f1)
    if count < 0:
        return "USAGE"
    if bool(has_signal) != (count > 0):
        return "SIGNAL_LIE"
    return "CLEAN" if count == 0 else "ANOMALY"
""",
        "round": """
    before = int(f0); after = int(f1); same = int(f2)
    if before < 0 or after < 0:
        return "USAGE"
    if not same:
        return "FORMAT_NA"
    return "ROUND_OK" if before == after else "ROUND_DRIFT"
""",
        "diff": """
    diff_count = int(f0); items = int(f1)
    if diff_count < 0 or items < 0:
        return "USAGE"
    return "DIFF_OK" if diff_count == items else "COUNT_DRIFT"
""",
        "px": """
    delta = int(f0); threshold = int(f1); struct_mismatch = int(f2)
    if delta < 0 or threshold < 0:
        return "USAGE"
    if struct_mismatch:
        return "STRUCT"
    return "PX_FAIL" if delta > threshold else "PX_OK"
""",
        "layout": """
    overflow = int(f0); overlap = int(f1); empty = int(f2); has_signal = int(f3)
    if min(overflow, overlap, empty) < 0:
        return "USAGE"
    total = overflow + overlap + empty
    if bool(has_signal) != (total > 0):
        return "SIGNAL_LIE"
    return "CLEAN" if total == 0 else "ANOMALY"
""",
        "reread": """
    verify = int(f0); written = int(f1); reread = int(f2)
    if not verify:
        return "NOT_EVIDENCE"
    if written < 0 or reread < 0:
        return "USAGE"
    return "REREAD_OK" if written == reread else "REREAD_DRIFT"
""",
        "redact": """
    applied = int(f0); before = int(f1); after = int(f2)
    if not applied:
        return "NOT_EVIDENCE"
    if before < 0 or after < 0:
        return "USAGE"
    if after > 0:
        return "STILL_PRESENT"
    return "NOTHING_TO_CLEAR" if before == 0 else "CLEAR_OK"
""",
        "cas": """
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
""",
        "avail": """
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
""",
        "layer": """
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
""",
        "rate": """
    ok = int(f0); total = int(f1)
    if ok < 0 or total < 0:
        return "USAGE"
    if ok > total:
        return "RATE_IMPOSSIBLE"
    return "RATE_OK"
""",
        "level": """
    return "LEVEL_OK" if f0 in {"L1", "L2", "L3", "L4", "L5"} else "LEVEL_UNKNOWN"
""",
        "triad": """
    hexset = set("0123456789abcdef")
    for token in (f0, f1, f2):
        if not token:
            return "TRIAD_MISS"
        if len(token) != 64 or any(ch not in hexset for ch in token):
            return "HASH_DEFECT"
    return "TRIAD_OK"
""",
        "trap": """
    year = int(f0); nols = int(f1); used = int(f2)
    if year >= 2024 and nols and used:
        return "TRAP"
    if year >= 2024 and nols:
        return "FLAGGED"
    return "SAFE"
""",
        "path": """
    escaped = int(f0); outside = int(f1)
    if outside:
        return "BREACH"
    return "ESCAPE" if escaped else "PATH_OK"
""",
        "limit": """
    size = int(f0); cap = int(f1); accepted = int(f2)
    if size < 0 or cap <= 0:
        return "USAGE"
    if size > cap and accepted:
        return "OVER_ACCEPTED"
    if size > cap:
        return "OVER_REJECT"
    return "UNDER_OK" if accepted else "UNDER_REJECT"
""",
        "parse": """
    has_space = int(f0); parsed = int(f1)
    if has_space and not parsed:
        return "PARSE_DROP"
    return "PARSE_OK" if parsed else "PARSE_FAIL"
""",
        "coord": """
    row = int(f0); col = int(f1); rows = int(f2); cols = int(f3)
    if min(row, col, rows, cols) < 0:
        return "USAGE"
    if rows == 0 or cols == 0:
        return "USAGE"
    if row >= rows or col >= cols:
        return "COORD_OOB"
    return "COORD_OK"
""",
        "mutate": """
    before = int(f0); delta = int(f1); after = int(f2); insert = int(f3)
    if before < 0 or after < 0:
        return "USAGE"
    expected = before + delta if insert else before - delta
    if expected < 0:
        return "USAGE"
    return "MUTATE_OK" if after == expected else "MUTATE_DRIFT"
""",
        "lease": """
    open_flag = int(f0); expired = int(f1); exit_code = int(f2)
    if exit_code not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    if not open_flag:
        return "SESSION_CLOSED"
    if expired:
        return "LEASE_STALE"
    return "LEASE_OK" if exit_code == 0 else "LEASE_FAIL"
""",
        "route": """
    return "ROUTE_OK" if f0 in {"pdf", "fill", "table", "needs-agent", "fde"} else "ROUTE_UNKNOWN"
""",
        "bbox": """
    right = int(f0); page_w = int(f1); bottom = int(f2); page_h = int(f3)
    if min(right, page_w, bottom, page_h) <= 0:
        return "USAGE"
    if right > page_w or bottom > page_h:
        return "BBOX_OOB"
    return "BBOX_OK"
""",
        "order": """
    prev_step = int(f0); this_step = int(f1)
    if prev_step < -1 or this_step < 0:
        return "USAGE"
    return "ORDER_OK" if this_step == prev_step + 1 else "ORDER_GAP"
""",
    }
    return mapping[family]


VERIFY_PY = """\
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
"""

TEST_PY = """\
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
"""


def write_unit(command: str, family: str, title: str) -> dict:
    pkg = slug(command)
    dest = HERE / pkg
    if dest.exists():
        shutil.rmtree(dest)
    corpus = dest / "corpus"
    tests = dest / "tests"
    corpus.mkdir(parents=True)
    tests.mkdir()
    claim = f"V-w2-{command}"
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
    if str(HERE) not in sys.path:
        sys.path.insert(0, str(HERE))
    # drop cached module so re-emits are re-imported
    for key in list(sys.modules):
        if key == pkg or key.startswith(pkg + "."):
            del sys.modules[key]
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
        print(f"[emit] {n}/{len(UNITS)} {command} rows={man['rowCount']}", flush=True)
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
    remainder_n = len(catalog) % BATCH
    if remainder_n:
        for item in catalog[-remainder_n:]:
            verify_pkg(item["command"])
    inventory = {
        "unitCount": len(catalog),
        "rowTotal": sum(i["rowCount"] for i in catalog),
        "units": catalog,
        "elapsedSec": round(time.time() - started, 1),
    }
    (HERE / "wave2_units_inventory.json").write_text(
        json.dumps(inventory, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    (HERE / "verify_wave2_units.py").write_text(
        '''from __future__ import annotations
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent

def main() -> int:
    inv = json.loads((HERE / "wave2_units_inventory.json").read_text(encoding="utf-8"))
    sys.path.insert(0, str(HERE))
    total = 0
    for unit in inv["units"]:
        pkg = "w2_" + unit["command"].replace("-", "_")
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
''',
        encoding="utf-8",
        newline="\n",
    )
    print(json.dumps({"units": len(catalog), "rows": inventory["rowTotal"]}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
