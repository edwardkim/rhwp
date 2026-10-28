# Stage 2 완료보고 — Task #1700 구현

## 변경 (접근법 B: 쿼리 표면화)

`src/document_core/queries/rendering.rs` `dump_page_items()`:

1. **사전 패스** (구역별): items 의 문단→표시페이지(global+1) 매핑(`item_disp_page`,
   다중 페이지 표는 **마지막 페이지**). 이어 표면화 대상(`extra_by_page`)을 페이지별로 그룹화.
   - 어울림(wrap-around) 문단 → 앵커 표(`table_para_index`)의 페이지에 귀속.
   - 빈 줄 감춤(`hidden_empty_paras`) → 직전 item 문단(대개 표)의 페이지에 귀속.
   - dedup(`emitted_extra`)로 중복 방지.
2. **출력**: 페이지별 items 출력 뒤 `extra_by_page[page]` 를 `WrapAroundPara pi=…` /
   `HiddenEmptyPara pi=…` 라인으로 추가.

**순수 가산** — 기존 item 출력 라인은 일절 변경하지 않음. 레이아웃/렌더 트리/페이지네이션
미변경 → 기하·페이지 수·시각 불변.

## 빌드
`cargo build --release` 성공 (rhwp v0.7.17).

## 즉시 확인 (대표 2건)
- 별표 `17978249`: 누락됐던 `pi=2`(표 직후 빈 문단) → `WrapAroundPara pi=2 table_pi=1 "(빈)"` 표면화 (page 1).
- 다중 페이지 표 `2067603`: `pi=2` → 표 끝 page 10 에 귀속(한글 일치).

세부 검증은 Stage 3.
