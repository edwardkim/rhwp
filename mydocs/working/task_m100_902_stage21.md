# Task #902 Stage 21 보고서 — clipping regions (IntersectClipRect)

**Stage**: 21 / 24+ (v2 점진 포팅)
**상태**: 완료 (rect-based clip)

## 1. 변경 영역

- `src/wmf/converter/raster/state.rs` — `clip_rect: Option<(i32, i32, i32, i32)>` 추가
- `src/wmf/converter/raster/player.rs` — `intersect_clip_rect` 구현

## 2. 알고리즘

LO `mtftools.cxx::IntersectClipRect` 단순화 포팅 (rect 기반):

```rust
fn intersect_clip_rect(mut self, _, record) -> Result {
    let (x0, y0) = self.logical_to_pixel(record.left, record.top);
    let (x1, y1) = self.logical_to_pixel(record.right, record.bottom);
    let new_clip = (x0.floor() as i32, y0.floor() as i32, ...);
    self.state.clip_rect = Some(match self.state.clip_rect {
        None => new_clip,
        Some(cur) => (cur.0.max(new.0), cur.1.max(new.1), cur.2.min(new.2), cur.3.min(new.3)),
    });
}
```

## 3. 한계

본 stage 는 clip_rect 를 state 에 기록만 함. 실제 drawing 시 clip 영역으로 픽셀 마스킹 미구현 — Stage 25+ 의 drawing 함수 정밀화 시 적용. 또한 ExcludeClipRect / region-based clip 단순화 미구현.

## 4. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — (1412 passed 기대)
```

## 5. 다음 단계

Stage 22: font escapement (text rotation)
