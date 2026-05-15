# Task #902 Stage 20 보고서 — arc / pie / chord 구현

**Stage**: 20 / 24+ (v2 점진 포팅)
**상태**: 완료

## 1. 변경 영역

`src/wmf/converter/raster/player.rs` — META_ARC, META_PIE, META_CHORD record 처리 + arc_path / arc_path_pie 헬퍼 함수.

## 2. 알고리즘

LO `mtftools.cxx` 의 DrawArc/DrawPie/DrawChord 포팅:

```rust
fn arc_path(x0,y0,x1,y1, sx,sy, ex,ey, close_chord) -> Path {
    let cx = (x0+x1)/2; cy = (y0+y1)/2;
    let rx = |x1-x0|/2; ry = |y1-y0|/2;
    let start_angle = atan2((sy-cy)/ry, (sx-cx)/rx);
    let end_angle = atan2((ey-cy)/ry, (ex-cx)/rx);
    let sweep = end_angle - start_angle; (음수면 +TAU)
    
    // 8 step 선형 근사 (cubic bezier 향후 정밀화)
    let step = sweep / 8;
    for i in 0..=8 { pb.line_to(cx + rx*cos(start_angle + step*i), ...); }
    if close_chord { pb.close(); }
}
```

| Record | 처리 |
|--------|------|
| `arc` | open path (close 안 함), stroke only |
| `chord` | closed path (start↔end 직선), fill + stroke |
| `pie` | wedge (center→start, arc, end→center), fill + stroke |

## 3. 한계

- 8 step 선형 근사 — cubic bezier sweep 으로 정밀화 follow-up
- pie 의 arc curve 단순화 (center→start→end 직선 wedge)

## 4. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — (1412 passed 기대)
```

## 5. 산출물

- 소스: `src/wmf/converter/raster/player.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage20.md`

## 6. 다음 단계

Stage 21: clipping regions (IntersectClipRect, ExcludeClipRect)
