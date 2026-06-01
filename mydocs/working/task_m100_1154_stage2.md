# Task #1154 Stage 2 완료 보고서 — Algorithm 구현 + 단위 테스트

## 1. 목표

- `PageRenderTree::clip_overlapping_same_bin_images()` 메서드 구현 (algorithm 단독, 호출 지점 통합은 Stage 3)
- Stage 1 에서 도출한 strict 5 조건의 단위 테스트로 algorithm 검증
- 의도적 효과 (test-image, 3-10월_교육_통합) 보호 회귀 보장

## 2. 구현

`src/renderer/render_tree.rs` 에 추가:

```rust
impl PageRenderTree {
    /// 동일 bin_data_id 를 가진 ImageNode 가 세로로 인접 겹칠 때,
    /// 트리 순서상 먼저 그려지는 (z 가 작은) 쪽의 bbox/crop 을 위에 덮는
    /// (z 가 큰) 쪽의 top 까지 축소한다.
    pub fn clip_overlapping_same_bin_images(&mut self);
}
```

### Algorithm (3-phase)

1. **Phase 1 (수집)**: 트리 DFS pre-order 로 `(NodeId, BoundingBox, bin_id, crop)` 튜플 수집. 트리 순서 = SVG paint 순서 = z-order.
2. **Phase 2 (페어 검출)**: 5 조건 모두 만족하는 (A=lower, B=upper) 페어 검출:
   - 같은 `bin_data_id`
   - `|A.x - B.x| <= 1.0`
   - `|A.width - B.width| <= 1.0`
   - `A.y < B.y` (A 가 위)
   - `A.y + A.height > B.y` (세로 겹침)
   - A 의 new_height = `B.y - A.y`, new_crop.bottom = `crop.top + (crop.bottom - crop.top) * ratio` (round)
   - 여러 UPPER 와 겹치는 A 의 경우 가장 작은 new_height 채택
3. **Phase 3 (적용)**: HashMap<NodeId, (new_height, new_crop)> 으로 트리 walk 하여 mutation.

## 3. 단위 테스트 (11 개, 모두 통과)

| 테스트 | 검증 |
|---|---|
| `test_clip_exam_eng_pattern` | exam_eng.hwp page 2 의 정확한 geometry (LOWER bbox/crop 축소, UPPER 불변) |
| `test_clip_different_bin_id_no_clip` | 다른 bin_id → 무시 |
| `test_clip_no_vertical_overlap_no_clip` | 세로 gap → 무시 |
| `test_clip_different_x_no_clip` | x 다름 (대각선 오프셋) → 무시 — **test-image / 3-10월_교육_통합 회귀 보호** |
| `test_clip_different_width_no_clip` | width 다름 (다른 크기) → 무시 — **pic2.hwp 회귀 보호** |
| `test_clip_reversed_order_no_clip` | 트리 순서가 반대 (A.y > B.y) → 무시 |
| `test_clip_without_crop` | crop=None 케이스: bbox 만 축소 |
| `test_clip_three_overlapping_chain` | A < B < C: A→B.top, B→C.top, C 불변 |
| `test_clip_nested_children` | Body > Children 깊이의 ImageNode 도 처리 |
| `test_clip_single_image_no_op` | 단일 이미지 → no-op |

### 핵심 회귀 보호 테스트 (test_clip_different_x_no_clip)

Stage 1 에서 식별한 의도적 효과 케이스를 모방. test-image.hwp / 3-10월_교육_통합_2022 의 패턴 (x 가 8px 등 다른 그림자/2중 효과)이 변형되지 않음을 가드.

## 4. 검증 결과

```
cargo test --release --lib clip_
  test result: ok. 11 passed; 0 failed; 0 ignored

cargo test --release --lib (전체)
  test result: ok. 1318 passed; 0 failed; 6 ignored; 0 measured

cargo clippy --release --lib -- -D warnings
  Finished — no warnings

cargo fmt — applied to render_tree.rs only
```

## 5. 산출물

- 코드: `src/renderer/render_tree.rs` (+438 lines: impl + 11 tests + helpers)
- 보고서: `mydocs/working/task_m100_1154_stage2.md`

## 6. 다음 단계 (Stage 3)

- 각 렌더러 (svg/web_canvas/skia) 의 페이지 렌더 진입부에서 `clip_overlapping_same_bin_images()` 호출 통합
- exam_eng.hwp page 2 SVG 재생성 → 권위 PDF (`pdf/exam_eng-2022.pdf`) 와 시각 비교
- 회귀 sample 인 test-image / 3-10월_교육_통합 의 시각 불변 확인

승인 후 Stage 3 진행.
