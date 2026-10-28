# Task #1592 — Stage 2 완료보고서 (수정)

**단계**: render_runs 빈 문단 가드
**브랜치**: `local/task1592`

## 변경
- `section.rs` render_runs 진입부: 완전 빈 문단(text·char_shapes·controls·field_ranges·
  orphan_field_ends 전부 없음)이면 **run 미방출**(빈 문자열 반환). char_shapes 있으면 종전 유지.
- `task1378_empty_paragraph_single_run_id_zero` 갱신: 빈 문단은 run 미방출(`""`)이 정답.
  char_shapes=[] 는 "원본에 run 없음"을 의미(빈 run 이면 파서가 [(0,0)] 산출) → entry 가공 금지.

## 검증
- `task1592_..._no_spurious_charshape` GREEN.
- `cargo test --lib` 1964 passed/0 failed. `hwpx_roundtrip_baseline` 4/4. clippy 무경고.

## 다음
Stage 3 — fidelity 전수 통제 비교(빈 문단 광역 영향 확인): 36386761(목록) 해소 + 악화 0.
