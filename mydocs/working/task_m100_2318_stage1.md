# #2318 1단계 완료보고 — 재현 고정 실패 테스트

- 계획서: `mydocs/plans/task_m100_2318.md`
- 브랜치: `local/task2318`

## 산출물

`tests/issue_2318_master_page_plane.rs` (테스트 2건):

1. `issue_2318_master_page_ops_capped_at_behind_text` — shortcut.hwp p1 의
   PageLayerTree 에서 MasterPage 그룹 내부 모든 op 가 BehindText 이하 plane 인지
   검사. **현행 코드에서 실패** (정정 목표를 고정하는 red 테스트):
   ```
   바탕쪽 op 2개가 본문 기준 plane 으로 승격됨 (한컴=바탕쪽은 본문 뒤):
   [InFrontOfText, InFrontOfText]
   ```
2. `issue_2318_master_page_layer_wrap_preserved_for_internal_order` — 바탕쪽
   글상자의 원본 wrap(InFrontOfText) layer 가 보존되는지 검사 (바탕쪽 **내부**
   정렬 `sort_paper_render_nodes` 의 근거이므로 wrap 덮어쓰기 방식 정정을 금지).
   **현행 통과** — 2단계 정정 후에도 통과를 유지해야 한다 (plane 분류층 cap 강제).

## 확인 사항

- 실패 양상이 착수 정찰 진단과 일치: 바탕쪽 글상자(wrap=InFrontOfText,
  attr=0x046A4000 비트 21~23=3)의 op 2개가 front plane 으로 분류.
- plane 수집은 paint 재생과 동일한 layer 상속 규칙(node.layer.or(inherited))을
  사용 — 분류기 재구현이 아니라 공개 API `paint_op_replay_plane_with_layer` 직접 호출.

## 다음 단계

2단계: rust 정정 — replay plane 분류에 MasterPage 그룹 컨텍스트 반영(cap=BehindText).
replay_order + web_canvas 필터 + canvaskit_policy replay plan 세 지점.
