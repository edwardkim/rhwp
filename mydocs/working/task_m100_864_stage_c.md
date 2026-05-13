# Task #864 Stage C 정정 구현 + 검증

**작성일**: 2026-05-13
**브랜치**: `local/task864`

## C.1 정정 구현

### 변경 파일

1. **`src/wmf/converter/svg/mod.rs`** — 5 개 image record handler 에 origin-relative 좌표 변환 적용:
   - `bit_blt` (line 146): WithBitmap / WithoutBitmap 양쪽
   - `device_independent_bitmap_bit_blt` (line 221)
   - `device_independent_bitmap_stretch_blt` (line 296)
   - `stretch_blt` (line 396)
   - `stretch_device_independent_bitmap` (line 471)

   **구현 패턴**:
   ```rust
   let pt = self.context_current
       .point_s_to_absolute_point(&PointS { x: x_dest, y: y_dest });
   let mut operator = TernaryRasterOperator::new(
       raster_operation,
       pt.x,
       pt.y,
       height,
       width,
   );
   ```

2. **`src/wmf/converter/svg/device_context.rs:280`** — `as_view_box` 를 `(0, 0, x.abs(), y.abs())` 로 revert (Task #860 Stage D 의 `(origin_x, origin_y, ...)` 변경 되돌림).

3. **`src/wmf/converter/svg/mod.rs:element_max_y`** — text element 의 viewBox 확장 지원 추가 (`font-size` 사용). text 가 y+font-size 만큼 viewBox 하단 확장 유발.

4. 디버그 print (RHWP_DEBUG_WMF) 모두 제거.

## C.2 검증

### 빌드

```bash
$ cargo build --release
    Finished `release` profile [optimized] target(s) in 19.45s
```

### 테스트

```bash
$ cargo test --release --lib
test result: ok. 1230 passed; 0 failed; 2 ignored; 0 measured
```

**회귀 0**. 1230 passed 유지.

### Clippy

```bash
$ cargo clippy --release --lib
    Finished `release` profile [optimized] target(s) in 7.26s
```

경고 0.

### hwp3-sample14 page 2 시각 정합

**변경 전** (rhwp): 캡션 outline + text 가 BMP 위에 그려짐 (한컴과 반대).

**변경 후** (rhwp):
- BMP image 가 TOP (y=0)
- Polygon 화살표/축이 MIDDLE
- 캡션 outline 이 BELOW (y=1008)
- 캡션 text 가 BOTTOM (y=1205)

→ **한컴 PDF page 2 정합** (image 위, 캡션 아래) ✓

### 회귀 검증 (다른 WMF sample)

| sample | 결과 |
|---|---|
| hwp3-sample14.hwp (페이지 2) | ✓ 한컴 정합 |
| hwp3-sample14-hwp5.hwp (페이지 2) | ✓ 한컴 정합 |
| hwp3-sample4.hwp (36 페이지 전체) | ✓ 시각 동등 (origin=(0,0) 이라 변경 무영향, 일부 viewBox 미세 확장만) |

### 변경 메커니즘 요약

**Task #860 Stage D** 는 BMP image 가 viewBox 밖으로 나가는 것을 막기 위해 viewBox 를 logical origin 으로 옮겼다 (`(origin_x, origin_y, ext_x, ext_y)`). 그러나 이는 **text/polygon 은 origin-relative 변환되어 있고 image 만 raw logical** 인 **coord-space mismatch** 를 정정하지 않은 채 viewBox 위치만 변경한 미봉책이었다.

**Task #864** 는 image (TernaryRasterOperator) 의 x/y 도 동일한 origin-relative 변환을 적용하여 모든 element 가 같은 device 공간을 사용하도록 정합. viewBox 는 `(0, 0, ext_x, ext_y)` 로 복원. 이는 WMF 표준 (Window/Viewport mapping) 의미를 정확히 따른다.

## C.3 종합 결과

| 검증 항목 | 결과 |
|---|---|
| cargo build --release | ✓ |
| cargo test --release --lib (1230 passed) | ✓ 0 회귀 |
| cargo clippy --release --lib | ✓ 0 경고 |
| hwp3-sample14 page 2 한컴 정합 | ✓ image 위, 캡션 아래 |
| hwp3-sample14-hwp5 동등 | ✓ |
| hwp3-sample4 회귀 | ✓ 0 |

## Stage C 완료

본질 정정 완료. WMF metafile 의 image 좌표 변환을 다른 element 와 정합하여 한컴 PDF 의 element 순서를 정확히 재현.

**커밋 메시지 (안)**:
```
Task #864: WMF image record x/y 좌표 origin-relative 변환 — element 순서 한컴 정합

bit_blt, dib_bit_blt, dib_stretch_blt, stretch_blt, stretch_device_independent_bitmap
의 image x/y 를 point_s_to_absolute_point 로 변환. text/polygon 과 동일 device 공간
정합. as_view_box 를 (0, 0, ext_x, ext_y) 로 revert (Task #860 Stage D 미봉책 정정).

closes #864
```

📋 **Stage C 완료. 최종 보고서 작성 + 커밋 승인 요청드립니다.**
