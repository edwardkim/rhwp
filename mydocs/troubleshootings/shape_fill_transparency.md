---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 도형 채우기 투명도(alpha) 미처리 문제

## 날짜
2026-02-17

## 관련 타스크
타스크 105 (쪽 테두리/배경 기능 구현) — 후속 수정

## 증상

- `Worldcup_FIFA2010_32.hwp`에서 배경 이미지 위에 흰색 사각형 도형 8개가 불투명하게 렌더링됨
- HWP 프로그램에서는 반투명 흰색(배경 이미지가 비쳐 보임)으로 표시됨
- 도형의 채우기 투명도가 전혀 적용되지 않는 상태

## 원인 분석

### 1차 원인: 채우기 바이너리 파싱 불완전 (parse_fill)

공식 HWP 스펙(표 30)은 채우기 정보의 마지막 부분을 다음과 같이 기술:

```
DWORD  추가 채우기 속성 길이(size)
BYTE   추가 채우기 속성 [size 바이트]
```

그러나 스펙에는 **추가 채우기 속성 이후의 바이트**에 대한 설명이 없다. hwplib(Java 참조 구현)의 `ForFillInfo.java`를 분석한 결과:

```java
// hwplib ForFillInfo.java
private static void additionalProperty(FillInfo fi, StreamReader sr) {
    long size = sr.readUInt4();  // 추가 속성 크기
    if (size > 0) {
        sr.skip((int) size);     // 추가 속성 데이터 건너뜀
    }
}

private static void unknownBytes(FillInfo fi, StreamReader sr) {
    // 활성화된 채우기 유형마다 1바이트씩 읽음
    if (fi.getType().getValue() & 0x01 != 0) sr.readUInt1();  // 단색 채우기 alpha
    if (fi.getType().getValue() & 0x04 != 0) sr.readUInt1();  // 그러데이션 alpha
    if (fi.getType().getValue() & 0x02 != 0) sr.readUInt1();  // 이미지 채우기 alpha
}
```

**핵심 발견**: `unknownBytes`로 명명된 바이트들이 실제로는 각 채우기 유형의 **투명도(alpha)** 값이다.

#### 수정 전 parse_fill()
```rust
// 추가 속성만 읽고, unknownBytes(alpha)는 읽지 않음
let additional_size = r.read_u32().unwrap_or(0) as usize;
let _ = r.skip(additional_size);
// → alpha 바이트들이 소비되지 않아 후속 파싱(shadow info 등) 바이트 정렬 불일치
```

#### 수정 후 parse_fill()
```rust
// 추가 속성 읽기
let additional_size = r.read_u32().unwrap_or(0) as usize;
if additional_size > 0 {
    if fill_type_val & 0x04 != 0 {
        let _blurring_center = r.read_u8().unwrap_or(0);
    } else {
        let _ = r.skip(additional_size);
    }
}
// 미확인 바이트 = 채우기 alpha (hwplib unknownBytes)
if fill_type_val & 0x01 != 0 { fill.alpha = r.read_u8().unwrap_or(0); }
if fill_type_val & 0x04 != 0 { let a = r.read_u8().unwrap_or(0); if fill.alpha == 0 { fill.alpha = a; } }
if fill_type_val & 0x02 != 0 { let a = r.read_u8().unwrap_or(0); if fill.alpha == 0 { fill.alpha = a; } }
```

### 2차 원인: ShapeComponent 파싱 순서 오류

채우기 정보 이후의 바이트를 글상자 속성(margin)으로 파싱하고 있었으나, hwplib `ForShapeComponent.java` 참조 결과 실제 순서는:

```
commonPart → lineInfo → fillInfo → shadowInfo → instid → skip → transparent
```

#### 수정 전 (잘못된 순서)
```rust
// 채우기 이후: 글상자 마진으로 파싱 (오류)
let left = r.read_i16().unwrap_or(0);    // 실제로는 shadow_type의 일부
let right = r.read_i16().unwrap_or(0);
let top = r.read_i16().unwrap_or(0);
let bottom = r.read_i16().unwrap_or(0);
```

#### 수정 후 (그림자 정보)
```rust
// 채우기 이후: 그림자 정보 16바이트 (hwplib ForShapeComponent.shadowInfo)
if r.remaining() >= 16 {
    let _shadow_type = r.read_u32().unwrap_or(0);    // ShadowType
    let _shadow_color = r.read_u32().unwrap_or(0);    // COLORREF
    let _shadow_offset_x = r.read_i32().unwrap_or(0); // X 오프셋
    let _shadow_offset_y = r.read_i32().unwrap_or(0); // Y 오프셋
}
```

### 3차 원인: ShapeStyle 기본값 opacity=0.0

`ShapeStyle` 구조체가 `#[derive(Default)]`를 사용하여 `opacity` 필드의 기본값이 `f64::default()` = `0.0`(완전 투명)이었다. 테이블 셀 배경 등에서 `..Default::default()`로 나머지 필드를 채울 때 `opacity`가 0.0이 되어 셀 배경이 보이지 않았다.

#### 수정 전
```rust
#[derive(Debug, Clone, Default)]
pub struct ShapeStyle {
    // ...
    pub opacity: f64,  // Default = 0.0 (완전 투명!)
}
```

#### 수정 후
```rust
#[derive(Debug, Clone)]
pub struct ShapeStyle { /* ... */ }

impl Default for ShapeStyle {
    fn default() -> Self {
        Self {
            fill_color: None,
            stroke_color: None,
            stroke_width: 0.0,
            stroke_dash: StrokeDash::default(),
            opacity: 1.0,  // 기본값 = 불투명
        }
    }
}
```

## Alpha 값 해석

- `alpha = 0`: 미설정(기본값) → 불투명(opacity 1.0)으로 처리
- `alpha = 1~254`: 반투명 → `opacity = alpha / 255.0`
- `alpha = 255`: 불투명 → opacity 1.0

Worldcup 파일의 도형: alpha = 0xA3 (163) → opacity = 163/255 = 0.639

HWPX 포맷에서는 `<winBrush alpha="0.64">` 형태로 명시적 float 값 사용.

## 수정 파일

| 파일 | 수정 내용 |
|------|----------|
| `src/model/style.rs` | `Fill` 구조체에 `alpha: u8` 필드 추가 |
| `src/parser/doc_info.rs` | `parse_fill()`: additionalProperty + unknownBytes(alpha) 파싱 |
| `src/parser/control.rs` | `parse_shape_component_full()`: shadow info 파싱 (글상자 마진 → 그림자 정보) |
| `src/renderer/mod.rs` | `ShapeStyle::Default` 수동 구현, opacity 기본값 1.0 |
| `src/renderer/layout.rs` | `drawing_to_shape_style()`: alpha → opacity 변환 |
| `src/renderer/svg.rs` | rect/ellipse에 `opacity` 속성 출력 |
| `src/renderer/web_canvas.rs` | `globalAlpha` 설정으로 opacity 지원 |
| `src/parser/hwpx/header.rs` | HWPX `winBrush` alpha 속성 파싱 |

## 검증 결과

| 파일 | 수정 전 | 수정 후 |
|------|--------|--------|
| Worldcup 도형 8개 | 불투명 흰색 (배경 가림) | 반투명 흰색 (opacity=0.639) |
| k-water-rfp 셀 배경 | opacity=0.000 (안 보임) | opacity 속성 없음 (불투명) |
| request.hwp 도형 | opacity=0.000 (안 보임) | opacity 속성 없음 (불투명) |
| 전체 테스트 | 565개 통과 | 565개 통과 |

## 교훈

- 공식 HWP 스펙 문서는 채우기 투명도 바이트를 문서화하지 않았음 → hwplib 참조 구현 분석이 필수
- `#[derive(Default)]`로 새 필드를 추가할 때 해당 타입의 기본값이 의미상 올바른지 반드시 확인해야 함 (`f64` 기본값 0.0은 opacity에 부적합)
- 바이너리 파싱에서 바이트 소비 누락은 후속 필드 전체의 정렬을 깨뜨림 → 참조 구현과 바이트 단위로 비교할 것
