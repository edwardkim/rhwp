# Task #902 Stage 4 보고서 — META_SETVIEWPORTEXT/ORG 구현 + MM_ANISOTROPIC ratio 정합

**Stage**: 4 / 9 (v2)
**상태**: 완료

## 1. 변경 영역

- `src/wmf/converter/svg/device_context.rs` — `Viewport` struct 추가, `point_s_to_absolute_point` / `point_s_to_relative_point` 의 ratio 계산 정합
- `src/wmf/converter/svg/mod.rs` — `set_viewport_ext` / `set_viewport_origin` 의 `not implemented` 영역 구현

## 2. ROOT CAUSE

### 2.1 기존 미구현

```rust
fn set_viewport_ext(...) -> Result<Self, PlayError> {
    info!("META_SETVIEWPORTEXT: not implemented");
    Ok(self)
}
fn set_viewport_origin(...) -> Result<Self, PlayError> {
    info!("META_SETVIEWPORTORG: not implemented");
    Ok(self)
}
```

MM_ANISOTROPIC 의 정확한 변환 공식은 ViewportExt/Origin 정보 필요:
```
device_x = (logical_x - WindowOrg.x) × (ViewportExt.x / WindowExt.x) + ViewportOrg.x
device_y = (logical_y - WindowOrg.y) × (ViewportExt.y / WindowExt.y) + ViewportOrg.y
```

### 2.2 기존 `point_s_to_absolute_point`

```rust
let x = (f32::from((point.x - self.window.origin_x).abs())
    / self.window.scale_x) as i16;
```

`window.scale` 만 사용 — ViewportExt 명시 호출 시 ratio 적용 안 됨.

## 3. 수정 내용

### 3.1 Viewport struct

```rust
pub struct Viewport {
    pub x: i16,
    pub y: i16,
    pub origin_x: i16,
    pub origin_y: i16,
    pub ext_explicitly_set: bool,
}
```

### 3.2 logical_to_device_delta

```rust
fn logical_to_device_delta(&self, point: &PointS) -> (i16, i16) {
    let dx_logical = f32::from((point.x - self.window.origin_x).abs());
    let dy_logical = f32::from((point.y - self.window.origin_y).abs());

    let x = if self.viewport.ext_explicitly_set {
        let ratio = f32::from(self.viewport.x) / f32::from(self.window.x.max(1));
        (dx_logical * ratio) as i16
    } else {
        (dx_logical / self.window.scale_x) as i16  // Task #860 자동 확장 동작 유지
    };
    // ... y 동일
}
```

**핵심**:
- ViewportExt **명시 호출** 시: MM_ANISOTROPIC 정확 ratio (ViewportExt / WindowExt)
- ViewportExt **미호출** 시: 기존 `window.scale_x` 동작 유지 → Task #860 자동 확장 fixture 호환

### 3.3 Player 메서드 구현

```rust
fn set_viewport_ext(mut self, ...) -> Result<Self, PlayError> {
    self.context_current = self.context_current.viewport_ext(record.x, record.y);
    Ok(self)
}

fn set_viewport_origin(mut self, ...) -> Result<Self, PlayError> {
    self.context_current = self.context_current.viewport_origin(record.x, record.y);
    Ok(self)
}
```

## 4. 검증 결과

### 4.1 빌드

```
cargo build --release
   Finished `release` profile [optimized] target(s) in 1m 21s
```

### 4.2 회귀 테스트

```
cargo test --release --all-targets
Total passed: 1412 / failed: 0
```

### 4.3 SVG 회귀 점검 (sample16/14/17/18)

- sample16 page 18: SVG **byte-identical** to Stage 3 (viewport 미호출 → 기존 동작 유지)
- sample14 (Task #860 fixture): 11 페이지 SVG 정상 생성, 회귀 없음
- sample17: 12 페이지 SVG 정상
- sample18: 69 페이지 SVG 정상

### 4.4 rsvg-convert 셀프 검증

```
rsvg-convert -o /tmp/task902_s4/stage4.png /tmp/task902_s4/hwp3-sample16_018.svg
→ 207908 bytes PNG (Stage 3 byte-identical)
```

## 5. 본 Stage 의 효과

- sample16 (현 분석 대상): viewport 미호출 → 직접 효과 없음 (회귀 없음 ✓)
- viewport 호출 sample (sample14/17/18 의 다른 WMF picture 일부): MM_ANISOTROPIC 정확 ratio 적용. 향후 회귀 발생 시 본 stage 가 ROOT CAUSE 후보

## 6. 산출물

- 소스 수정: `src/wmf/converter/svg/device_context.rs`, `src/wmf/converter/svg/mod.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage4.md`
- 검증 SVG/PNG: `/tmp/task902_s4/`, `/tmp/task902_s4_s{14,17,18}/`

## 7. 다음 단계

Stage 5: EXTTEXTOUT options flags 처리 (ETO_OPAQUE/CLIPPED/PDY 등)
