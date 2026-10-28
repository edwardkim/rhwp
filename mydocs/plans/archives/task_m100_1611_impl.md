# 구현계획서 — Task #1611

**제목**: footer(발신명의) Page+Bottom 블록 page-fit vpos·선언높이 정합
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1611 · **브랜치**: `local/task1611`
**확정 방향**: Stage 1(`_stage1.md`) — `VertRelTo::Page`+`Bottom` TopAndBottom 블록의
`current_height` stored vpos 동기화 + 선언 높이 fit. 승인됨.

## 수정 대상

`src/renderer/typeset.rs` — 표 배치(generic fit 체크 직전, Paper/Para 특수처리 다음).

## 단계 (3단계)

### Stage 2 — RED
- fixture: `samples/hwpx/opengov/36387725_footer_page_bottom.hwpx` (corpus 36387725 복사, opengov
  결재문서 선례 정합). 한글 정답지 2쪽.
- test: `tests/issue_1611_footer_page_bottom_pagination.rs` — `page_count()==2` 단언(현재 1 → RED).

### Stage 3 — 수정 (GREEN)
- `is_page_bottom_topbottom_block` = 비-TAC + TopAndBottom + `VertRelTo::Page` + `vert_align=Bottom`.
- 해당 + column 0 시: `target_y`(=host para first line_seg vpos), `declared_px`(=common.height),
  `block_height = table_total.max(declared_px)`, `sync_h = current_height.max(target_y)`.
  - `sync_h + block_height <= available` → 현재 쪽에 sync_h 배치.
  - 초과 → `advance_column_or_new_page()` 후 다음 쪽 단독 배치(분할 부적절).
  - `place_table_with_text(..., block_height, ...)` 로 선언 높이 advance.
- 핵심: ① vpos 동기화(Paper 만 하던 처리를 Page 로 확장) ② 선언 높이 fit(측정 302.3 대신
  선언 351.4) — 둘 다 있어야 640.7+351.4=992.1 > 990.2 → 분할.

### Stage 4 — 게이트 검증 + 보고
- render_page_gate net>0, hwpx/visual baseline, lib/clippy, SVG 시각 확인.

## 회귀 위험
Page+Bottom 전반에 vpos·선언높이 적용 → 한컴이 1쪽 유지하는 footer 를 과도하게 미는 +1
회귀 가능. 통제셋 게이트(net>0)로 방어.
