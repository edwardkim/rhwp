# Task #902 Stage 6 보고서 — 미구현 WMF records 완성

**Stage**: 6 / 9 (v2)
**상태**: 완료

## 1. 변경 영역

- `src/wmf/converter/svg/device_context.rs` — offset/scale mutator 추가
- `src/wmf/converter/svg/mod.rs` — META_OFFSETWINDOWORG / META_OFFSETVIEWPORTORG / META_SCALEVIEWPORTEXT 의 `not implemented` 영역 구현

## 2. 처리 추가 records

| Record | 처리 |
|--------|------|
| **META_OFFSETWINDOWORG** | `window.origin += (x_offset, y_offset)` |
| **META_OFFSETVIEWPORTORG** | `viewport.origin += (x_offset, y_offset)` |
| **META_SCALEVIEWPORTEXT** | `viewport.ext × (num/denom)` (x, y 별도) |

기존 `META_SCALEWINDOWEXT` 는 이미 구현됨.

## 3. 잔존 미구현 records (본 stage scope 외)

낮은 우선순위로 분류 (sample16-like 다이어그램 에 미사용):
- Region ops (`META_FILLREGION`, `META_FRAMEREGION`, `META_INVERTREGION`, `META_PAINTREGION`)
- Palette ops (`META_ANIMATEPALETTE`, `META_REALIZEPALETTE`, `META_RESIZEPALETTE`, `META_SETPALENTRIES`)
- Clip ops (`META_EXCLUDECLIPRECT`, `META_OFFSETCLIPRGN`)
- 기타 (`META_SETDIBTODEV`, `META_EXTFLOODFILL`, `META_FLOODFILL`, `META_SETPIXEL`, `META_SETLAYOUT`, `META_SETMAPPERFLAGS`, `META_SETSTRETCHBLTMODE`, `META_SETTEXTCHAREXTRA`, `META_SETTEXTJUSTIFICATION`)

향후 sample 발견 시 별도 stage 또는 follow-up task.

## 4. 구현 세부

### 4.1 DeviceContext mutators

```rust
pub fn offset_window_origin(mut self, dx: i16, dy: i16) -> Self {
    self.window.origin_x = self.window.origin_x.saturating_add(dx);
    self.window.origin_y = self.window.origin_y.saturating_add(dy);
    self
}

pub fn offset_viewport_origin(mut self, dx: i16, dy: i16) -> Self {
    self.viewport.origin_x = self.viewport.origin_x.saturating_add(dx);
    self.viewport.origin_y = self.viewport.origin_y.saturating_add(dy);
    self
}

pub fn scale_viewport_ext(mut self, x_num, x_denom, y_num, y_denom) -> Self {
    let new_x = (i32::from(self.viewport.x) * i32::from(x_num)
        / i32::from(x_denom.max(1))) as i16;
    let new_y = (i32::from(self.viewport.y) * i32::from(y_num)
        / i32::from(y_denom.max(1))) as i16;
    self.viewport = self.viewport.ext(new_x, new_y);
    self
}
```

### 4.2 SVG Player 메서드

각 record 호출 시 context_current 갱신:

```rust
fn offset_window_origin(...) { self.context_current = self.context_current
    .offset_window_origin(record.x_offset, record.y_offset); Ok(self) }
fn offset_viewport_origin(...) { ... }
fn scale_viewport_ext(...) { ... }
```

## 5. 검증 결과

### 5.1 빌드 + 회귀 테스트

```
cargo build --release           — Finished
cargo test --release --all-targets — 1412 passed / 0 failed
```

### 5.2 SVG 회귀

- sample16 page 18: PNG **byte-identical** to Stage 5 (offset/scale 미호출 → fallback)
- 1412 passed 유지

## 6. 산출물

- 소스 수정: `src/wmf/converter/svg/device_context.rs`, `src/wmf/converter/svg/mod.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage6.md`

## 7. 다음 단계

Stage 7: 폰트 metric 정합 (나눔고딕 substitute 또는 임베딩)
