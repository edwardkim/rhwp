# Task #249 단계별 완료보고서: 단계 3 — 표 외곽 테두리 fallback + clip_rect 개선

> Issue: [#249](https://github.com/edwardkim/rhwp/issues/249)
> 완료일: 2026-04-22

---

## 구현 내용

`table.border_fill_id` 설정 시 표 외곽 테두리를 그리도록 fallback 로직을 추가하고, 셀로 커버되지 않는 영역에만 한정 적용. 또한 clip_rect를 콘텐츠 레이아웃 후 확정하여 표 외곽 테두리가 잘리지 않도록 수정.

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/layout/table_layout.rs` | `border_fill_id` fallback 로직 + 셀 커버 영역 제외 처리 |
| `src/renderer/layout.rs` | clip_rect를 콘텐츠 레이아웃 후 자식 노드 bbox 반영하여 확정 |

## 주요 구현 사항

**table_layout.rs:**
- `table.border_fill_id`가 있을 때 외곽 테두리 영역 계산
- 셀 bbox 합집합을 구하여 커버된 영역을 제외한 나머지에만 fallback 테두리 적용
- 과도한 fallback으로 인한 이중 테두리 방지

**layout.rs:**
- `clip_rect`를 `body_area`로 즉시 고정하던 방식에서, 콘텐츠 레이아웃 완료 후 자식 노드 bbox를 재귀 반영하여 확정하는 방식으로 변경
- 표 외곽 테두리가 `body_area` clip 경계에서 잘리는 문제 해결

## 검증

- `cargo test`: 793개 통과, 0 실패
- Visual Diff: 표 외곽 테두리가 한컴 렌더링과 일치, 기존 페이지 regression 없음
