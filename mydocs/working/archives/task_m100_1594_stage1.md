# Task #1594 — Stage 1+2 완료보고서 (RED + 수정)

**단계**: holdAnchorAndSO 보존 재현(RED) + 직렬화 수정
**브랜치**: `local/task1594`

## Stage 1 (RED)
`table.rs` 테스트 `task1594_hold_anchor_preserved`: prevent_page_break=1 표 직렬화 →
holdAnchorAndSO="1" 방출 검증. 현재 "0" 하드코딩으로 RED. (+ `_zero_when_unset` 기본 0 보존.)

## Stage 2 (수정)
4지점이 holdAnchorAndSO 를 IR(`c.prevent_page_break != 0`)로 방출:
- `table.rs` write_pos, `picture.rs` write_pos, `shape.rs` write_pos, `section.rs` equation.

## 검증
- `task1594_*` 2건 GREEN, `cargo test --lib` 1969 passed/0 failed.
- `hwpx_roundtrip_baseline` 4/4, clippy 무경고.

## 다음
Stage 3 — diff_documents 에 prevent_page_break 추가 + fidelity 통제 비교 + 한글 붕괴 해소 검증.
