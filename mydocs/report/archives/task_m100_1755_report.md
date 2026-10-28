# Task #1755 최종 보고서 — 지연 자리차지 표 host 제목 줄의 이월-전-쪽 잔류

## 요약
#1753 한계 항목 완결. 지연 이월되는 visible-host 자리차지 RowBreak 표의 host 제목 줄이
layout `defer_visible_rowbreak_host_text` 경로로 **마지막 fragment 뒤(11쪽)**에 렌더되던
순서 결함을, typeset pre-emit + layout 억제 신호로 **이월 전 쪽(9쪽, 한글 정합)** 잔류로
수정. 대표 2814765 **완전 MATCH** 전환 (9쪽 순서: 제목 → ※주석 → 2)보정계수 = 한글 PDF 동일).

## 수정
- `renderer/typeset.rs`: `prefill_before_deferred_table`(#1753 가드) 진입 시 host 텍스트
  줄을 `PartialParagraph{0..n}` 로 현재 쪽 pre-emit + 높이 소비. host 줄이 안 들어가면
  prefill 도 중단(순서 역전 방지). `TypesetState/PaginationResult.pre_emitted_host_paras`.
- `renderer/pagination.rs`(+engine/rendering 구성처): PaginationResult 필드 신설.
- `renderer/layout.rs`: `set_pre_emitted_host_paras`(hidden_empty_paras 패턴) + 분할 표
  host 렌더 2경로(마지막 fragment 뒤 / 첫 부분) 억제.
- `document_core/queries/rendering.rs`: 섹션별 신호 배선.

## 검증
| 항목 | 결과 |
|------|------|
| 재현·코퍼스 원본 (한글 OLE) | PI_MISMATCH(pi51) → **완전 MATCH** (21=21쪽) |
| SVG | 제목 9쪽 렌더 / 11쪽 이중 렌더 없음 |
| cargo test --lib | 2051 passed / 0 failed |
| 통합 10크레이트 (신규 issue_1755 포함) | 103 passed / 0 failed |
| 페이지 게이트 9종 (국제고속선기준 251 포함) | 무회귀 |
| 코퍼스 mismatch / MATCH 표본 150 | 악화 0 / 150 유지 |
| rustfmt / clippy --lib | 통과 |

## 한계 / 후속
- pre-emit 은 #1753 가드 케이스(visible-host 자리차지 RowBreak 이월) 한정 — 일반 분할 표
  host 렌더 경로 무변경.
- Stage 1·2 는 PaginationResult 필드가 전 구성처 컴파일에 걸려 통합 커밋 (stage1 보고서 기록).

## 산출물
- 소스: `src/renderer/{typeset,layout,pagination}.rs`, `pagination/engine.rs`,
  `document_core/queries/rendering.rs` (+ `tests/issue_1755_host_heading_pre_emit.rs`)
- 재현: `samples/task1753/deferred_takeplace_fill_ahead.hwpx` (재사용)
- 검증 TSV: `output/poc/hwpdocs_pipage/{bad39,match150}_recheck_1755.tsv`
