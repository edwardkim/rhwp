# Task #1567 Stage 2 완료보고서 — serializer 수정 (빈 ref pic 보존)

## 변경 (`src/serializer/hwpx/picture.rs`)
`write_img` — `resolve_bin_id(bin_id)` 실패 시:
- `bin_id==0`(빈 binaryItemIDRef placeholder) → `binaryItemIDRef=""` verbatim 방출(드롭 금지).
- 비-0 미해결 → 종전 `Err`(진짜 BinDataContent 누락 진단, 손실 은폐 방지).
```rust
let manifest_id = match ctx.resolve_bin_id(bin_id) {
    Some(id) => id,
    None if bin_id == 0 => "",
    None => return Err(...),
};
```
→ `write_picture` Ok → `section.rs:701` 정상 방출(조용한 드롭 제거).

## 검증
- `cargo build --release` 성공.
- 단위 테스트 `task1567_empty_binary_ref_pic_preserved`: **passed**(빈 ref → `binaryItemIDRef=""` 보존).
- 대표 케이스 roundtrip:
  - 36385464: **IR_DIFF/1 → PASS/0** (pic 드롭 해소) ✅
  - 36388571: IR_DIFF/2 → IR_DIFF/1 (pic 부분 해소, char-shift 잔여)
  - 36383351: IR_DIFF/1 유지 (char-shift만, pic 無 — 무관)

## 광역 효과 (hwpdocs 2601건)
| | v4(pic 미수정) | 수정 후 |
|--|--|--|
| 표셀 pic 드롭 | 907 | **0** ✅ |
| PASS율 | 35% | **71%** |
| IR_DIFF율 | 65% | **28%** (전부 char-shift F3) |

## 다음
Stage 3 — baseline 회귀 + opengov 스냅샷 갱신(교차 후속) + 보고.
