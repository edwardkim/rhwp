# Task #877 Stage 3 완료 보고서 — 시각 차이 4건 진단 + 3건 수정

**관련 계획서**: [task_m100_877_impl.md](../plans/task_m100_877_impl.md)
**참조 spec**: [한글문서파일구조3.0.md](../tech/한글문서파일구조3.0.md)
**브랜치**: `local/task877_v2` (분기: `local/task873`)

## 배경

Stage 1+2 적용 후 sample16 panic 없음 + 65 페이지 인식. 그러나 한컴오피스 viewer 와 시각 차이 4건 발견:

1. **표지 박스/외곽선 누락**
2. **빈 페이지 2 (목차 페이지 어긋남)**
3. **로마숫자 prefix (Ⅰ, Ⅱ, Ⅲ ...) 누락**
4. **16페이지 다이어그램 미표시**

## 진단 결과 및 수정

### ✅ 1. 표지 박스/외곽선 누락 — **수정 완료**

probe 결과: paragraph 5 의 RFP 박스는 **Rectangle drawing object (pic_type=3)**:
- HWP3 raw: `border line color=0x00000000, width=84, style=0x0000` (LineType=0 → 외곽선 미표시)
- HWP5 변환본: `style=0xc0010041` (LineType=1 Solid + cap + arrow 등)

렌더러 [renderer/layout/utils.rs:163] 의 `border.attr & 0x3F == 0` 시 외곽선 미표시 규칙.

**수정** ([src/parser/hwp3/drawing.rs:748-768](../../src/parser/hwp3/drawing.rs#L748-L768)):
```rust
attr: {
    let raw_attr = header.basic_attr.line_style as u32;
    if (raw_attr & 0x3F) == 0 && header.basic_attr.line_width > 0 {
        raw_attr | 0x01    // bit 0..5 = 1 (Solid LineType)
    } else { raw_attr }
}
```
근거: HWP3 raw line_style=0 + line_width>0 + line_color 정상 = 한컴 viewer 실선 표시.

### ✅ 2. 빈 페이지 2 (목차 페이지 어긋남) — **수정 완료**

HWP3 vs HWP5 변환본 vpos 비교:
| paragraph | HWP3 vpos | HWP5 변환본 vpos | 차이 |
|-----------|----------|---------------|------|
| 0.5 (RFP 박스) | line_spacing=4768 | line_spacing=840 | 3928 |
| 0.23 | 63676 | 59748 | 3928 |
| 0.24 | 72360 | 68372 | 3988 |

원인: paragraph 5 의 RFP 박스는 **Rectangle (treat_as_char=true)**. [mod.rs:1647] 의 `has_tac_picture` 검사가 Picture (image) 만 포함하고 **Rectangle / Ellipse / Polygon / Line / Arc / Curve / Group 누락**. 그 결과 line_spacing 이 `text_height (7948) × 60% = 4768 HU` 거대값으로 계산 → 후속 paragraph vpos 누적 3928~3988 HU 어긋남 → paragraph 24 vpos=72360 페이지 1 영역 (~74435 HU) 초과 → 빈 페이지 2.

**수정** ([src/parser/hwp3/mod.rs:1647-1670](../../src/parser/hwp3/mod.rs#L1647-L1670)):
```rust
let has_tac_picture = para.controls.iter().any(|c| {
    match c {
        Control::Picture(p) => p.common.treat_as_char,
        Control::Shape(s) => match s.as_ref() {
            ShapeObject::Picture(p) => p.common.treat_as_char,
            ShapeObject::Rectangle(r) => r.common.treat_as_char,
            ShapeObject::Ellipse(e) => e.common.treat_as_char,
            ShapeObject::Polygon(p) => p.common.treat_as_char,
            ShapeObject::Line(l) => l.common.treat_as_char,
            ShapeObject::Arc(a) => a.common.treat_as_char,
            ShapeObject::Curve(c) => c.common.treat_as_char,
            ShapeObject::Group(g) => g.common.treat_as_char,
            _ => false,
        },
        _ => false,
    }
});
```

결과: paragraph 5 ls 4768 → 600 정합 → 후속 vpos 누적 정합 → **페이지 수 65 → 64** (한컴 viewer 정확 일치).

### ✅ 3. 로마숫자 (Ⅰ, Ⅱ, Ⅲ ...) 누락 — **수정 완료**

HWP3 사적 인코딩 `0x3590~0x3599` = Unicode `U+2160~U+2169` (Ⅰ~Ⅹ) 매핑 부재.

**수정** ([src/parser/hwp3/johab.rs:64-71](../../src/parser/hwp3/johab.rs#L64-L71)):
```rust
if (0x3590..=0x3599).contains(&ch) {
    return char::from_u32(0x2160 + (ch - 0x3590) as u32);
}
```

### ❌ 4. 16페이지 다이어그램 — **본 task 범위 밖**

사용자 확인: **HWP5 변환본도 동일 증상** — HWP3/HWP5 IR 은 동일 표현, **rhwp 의 drawing object tree 렌더러** 영역.
→ 별도 task 로 분리.

## 최종 결과 (sample16)

| 항목 | Stage 1 만 | Stage 2 후 | **Stage 3 최종** | 한컴 viewer |
|------|---------|---------|---------------|---------|
| Panic | ❌ 발생 | ✅ 없음 | ✅ 없음 | (n/a) |
| 문단 수 | 77 | 1058 | 1058 | (= HWP5 변환본) |
| 페이지 수 | 28737 | 65 | **64** | 64 ✓ |
| 페이지 2 | 빈 | 빈 | **목차** | 목차 ✓ |
| 표지 박스 | ❌ | ❌ | **✅** | ✓ |
| 로마숫자 Ⅰ~Ⅹ | ❌ | ❌ | **✅** | ✓ |
| 16쪽 다이어그램 | ❌ | ❌ | ❌ (HWP5 도 동일) | ✓ |

## 검증

### cargo test
```
test result: passed: 1381 failed: 0
```

### HWP3 sample 6종 회귀 없음
| 샘플 | 문단 수 | 페이지 수 |
|------|--------|----------|
| hwp3-sample.hwp | 195 | 16 |
| hwp3-sample10.hwp | 26767 | 763 |
| hwp3-sample14.hwp | 256 | 11 |
| hwp3-sample4.hwp | 1273 | 36 |
| hwp3-sample5.hwp | 1931 | 64 |

## 변경 파일 (Stage 3 누적)

- `src/parser/hwp3/johab.rs` — `decode_hwp3_extra` 에 Ⅰ~Ⅹ 로마숫자 매핑 추가 (커밋 7f35fa3)
- `src/parser/hwp3/drawing.rs` — drawing line_style=0 + width>0 → Solid LineType 보강 (커밋 b0bf58f)
- `src/parser/hwp3/mod.rs` — drawing object 모든 variant 의 treat_as_char 검사 (커밋 2d737be)

## 후속 별도 이슈 (1건)

- **HWP3/HWP5 drawing object tree (ch=11 pic_type=3) 렌더링 정합**: sample16 페이지 16 의 다이어그램. HWP5 변환본도 동일 미표시. rhwp 렌더러 영역.

## 다음 단계

최종 결과 보고서 작성 → task #877 완료 → orders 갱신 → merge.
