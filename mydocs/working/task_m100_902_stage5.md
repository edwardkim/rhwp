# Task #902 Stage 5 보고서 — EXTTEXTOUT options flags 처리

**Stage**: 5 / 9 (v2)
**상태**: 완료

## 1. 변경 영역

`src/wmf/converter/svg/mod.rs` — `ext_text_out` 의 ETO_PDY / ETO_OPAQUE 처리 추가

## 2. 처리 추가 플래그

### 2.1 ETO_PDY (0x2000)

WMF spec: DX 배열을 (dx, dy) 쌍으로 해석 — char 별 x/y advance.

**변경**:
- `entries_per_byte = if pdy { 2 } else { 1 }` 로 byte 당 DX 배열 entry 수 분기
- Korean wide char (s.width=2) 시 `4 entries` (2 byte × 2 entries) 차지
- tspan 에 `x` 외 `y` 도 절대 좌표 설정 (vertical 누적)

```rust
let pdy = record.fw_opts.contains(&ExtTextOutOptions::ETO_PDY);
let entries_per_byte = if pdy { 2 } else { 1 };
let mut acc_x: i32 = i32::from(point.x);
let mut acc_y: i32 = i32::from(point.y);
// ... per grapheme: acc_x += dx_advance; if pdy { acc_y += dy_advance; }
```

### 2.2 ETO_OPAQUE (0x0002)

WMF spec: rectangle 영역을 현재 bk_color 로 채운 뒤 text 렌더.

**변경**:
- text element push 직전에 background `<rect>` 를 push
- rect 좌표는 `point_s_to_absolute_point` 로 device 변환
- fill = `text_bk_color`

```rust
if let (true, Some(rect)) = (
    record.fw_opts.contains(&ExtTextOutOptions::ETO_OPAQUE),
    record.rectangle.as_ref(),
) {
    let tl = ctx.point_s_to_absolute_point(...);
    let br = ctx.point_s_to_absolute_point(...);
    let bg_rect = Node::new("rect")
        .set("x", tl.x).set("y", tl.y)
        .set("width", (br.x - tl.x).max(0))
        .set("height", (br.y - tl.y).max(0))
        .set("fill", css_color);
    self.push_element(record_number, bg_rect);
}
```

### 2.3 ETO_CLIPPED (0x0004)

**기존 구현 유지** — shape-inside CSS polygon 사용. 단, `record.rectangle` borrow 충돌 해결 위해 `as_ref()` 로 변경.

### 2.4 다른 플래그 (ETO_GLYPH_INDEX, ETO_RTLREADING, ETO_NUMERICSLOCAL/LATIN)

본 stage scope 외 — sample16 사용 안 함.

## 3. 검증 결과

### 3.1 빌드 + 회귀 테스트

```
cargo build --release           — Finished release 1m 21s
cargo test --release --all-targets — 1412 passed / 0 failed
```

### 3.2 SVG 회귀

- sample16 page 18: SVG **byte-identical** to Stage 4 (ETO_OPAQUE/PDY 미설정 → 기존 동작)
- PNG (rsvg-convert): 207908 bytes (Stage 4 와 동일)

ETO_PDY/OPAQUE 사용 sample 발견 시 본 stage 의 효과 적용.

## 4. 산출물

- 소스 수정: `src/wmf/converter/svg/mod.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage5.md`
- 검증: `/tmp/task902_s5/`

## 5. 다음 단계

Stage 6: 미구현 WMF records 완성
