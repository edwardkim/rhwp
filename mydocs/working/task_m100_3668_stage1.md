---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3668.md
last_verified: 2026-08-01
---

# Task #3668 Stage 1 보고 — 집계 코어 + CLI 봉투

## 구현

1. **집계 코어** — layout 엔진에 `overflow_cell_lines: Cell<u32>` 추가
   (`renderer/layout.rs`). `LAYOUT_OVERFLOW_CELL` eprintln 지점
   (`layout/paragraph_layout.rs`)에서 **같은 조건**으로 증가. stderr 출력 불변.
   `take_overflow_cell_lines()`(읽기+리셋) 를 엔진과 `DocumentCore`
   (`document_core/queries/rendering.rs`)에 노출.
2. **CLI 표면** — `export-svg --json` 봉투에 페이지별 `overflowCellLines` +
   문서 합계(top-level). 페이지 귀속은 `build_page_tree(page_num)` 가 페이지 단위
   (캐시 포함)라 렌더 직후 take 로 자연 획득. capabilities 매니페스트
   (`hwp_export_svg.outputFields`)에도 필드 광고. 계약 테스트
   (`render_manifest_json_contract`)에 필드 존재 단언 추가.

## red-check — 카운트 정확성

#3236 특례 상한(`SINGLE_ROW_DECLARED_TRUST_MAX_RATIO`)을 임시 제거하고 재렌더:

| 상태 | 총계 | 페이지 귀속 |
|---|---|---|
| 현행(수정 유지) | **0** | (0,0) |
| 상한 제거(red) | **23** — #3236 조사 때 stderr 실측과 정확히 일치 | **page 0 에 23** — 문제 표가 있던 쪽 |

제거 후 원복·재빌드 완료.

## 검증

- 계약 테스트 4종(render_manifest·cli_json·mcp_server drift guard·issue_3236) **33 passed**.
- **렌더 산출물 불변**: 카운터 추가 전후 #3236 fixture SVG 바이트 동일(cmp).

## 관련 발견 — 기존 `take_overflows()` 채널

렌더 경로에 item 수준 `LayoutOverflow` 수집(`take_overflows`)이 이미 있으나 **전 소비자가
`let _overflows = …` 로 버리고 있다**(rendering.rs 6개소). 이는 다른 진단
(LAYOUT_OVERFLOW, 요소 단위)이고 셀 줄 진단과 별개다. 두 채널의 통합·소비 표면화는
범위 밖 — 후속 후보로만 기록한다.
