# Task 397 — 1단계 완료 보고서: SkParagraph 심층 분석

## 아키텍처

### 파이프라인

```
ParagraphBuilder
  → addText(text, TextStyle)   // 텍스트 + 스타일 누적
  → Build()
  → Paragraph (불변 객체)
      → layout(max_width)       // 줄바꿈, 글리프 배치 계산
      → paint(canvas, x, y)     // 렌더링
      → getLineMetrics()        // 줄별 메트릭 조회
      → getRectsForRange()      // 범위별 바운딩 박스
      → getGlyphPositionAtCoordinate()  // 좌표→글리프 역매핑
```

### 핵심 구성요소

| 구성요소 | 역할 |
|----------|------|
| **ParagraphBuilder** | 빌더 패턴. 텍스트와 스타일을 누적하여 Paragraph 생성 |
| **Paragraph** | 불변 레이아웃 객체. layout() 호출 시 줄바꿈/배치 계산 |
| **TextStyle** | 문자 수준 스타일 (폰트, 크기, 색상, 자간, 단어간격, 장식) |
| **ParagraphStyle** | 문단 수준 스타일 (정렬, 최대 줄수, 말줄임, 텍스트 방향) |
| **StrutStyle** | 줄 높이 강제 설정 (폰트 크기, height, leading, 강제 적용) |
| **LineMetrics** | 줄별 메트릭 (baseline, ascent, descent, 너비, 높이) |

### 내부 엔진

- **텍스트 셰이핑**: HarfBuzz (리가처, 커닝, 복잡 스크립트)
- **줄바꿈**: ICU 기반 Unicode Line Breaking Algorithm
- **BiDi**: ICU BiDi (RTL/LTR 혼합)
- **폰트 폴백**: 플랫폼별 + 커스텀 폴백 체인

## HWP 속성 매핑 분석

### CharShape ↔ TextStyle

| HWP CharShape | SkParagraph TextStyle | 매핑 가능성 |
|---------------|----------------------|-------------|
| `font_ids[7]` (언어별) | `setFontFamilies(vector)` | ○ 1:1 매핑 불가. HWP는 언어별 7개 폰트, Skia는 폴백 체인 |
| `base_size` (HWPUNIT) | `setFontSize(SkScalar)` | ◎ 단위 변환 필요 (HWPUNIT → pt) |
| `ratios[7]` (장평) | 해당 없음 | ✗ Skia에 장평 개념 없음. font-stretch와 다름 |
| `spacings[7]` (자간) | `setLetterSpacing(SkScalar)` | △ HWP는 언어별 자간, Skia는 단일 값 |
| `bold` / `italic` | `setFontStyle(SkFontStyle)` | ◎ |
| `underline_type/shape` | `setDecoration(TextDecoration)` + `setDecorationStyle()` | ○ HWP 선 종류가 더 다양 |
| `strikethrough` / `strike_shape` | `setDecoration(kLineThrough)` | ○ |
| `text_color` | `setColor(SkColor)` | ◎ |
| `shadow_type/offset/color` | `addShadow(TextShadow)` | ○ |
| `subscript` / `superscript` | `setBaselineShift(SkScalar)` + 크기 조절 | △ 수동 구현 필요 |
| `relative_sizes[7]` (상대크기) | 해당 없음 | ✗ 언어별 상대 크기 직접 지원 안 됨 |
| `char_offsets[7]` (글자위치) | `setBaselineShift()` | △ 언어별 개별 적용 불가 |
| `kerning` | HarfBuzz 자동 처리 | ◎ Skia가 자동 처리 |
| `emboss` / `engrave` | 해당 없음 | ✗ 커스텀 렌더링 필요 |
| `emphasis_dot` (강조점) | 해당 없음 | ✗ 커스텀 렌더링 필요 |

### ParaShape ↔ ParagraphStyle/StrutStyle

| HWP ParaShape | SkParagraph | 매핑 가능성 |
|---------------|-------------|-------------|
| `alignment` (정렬) | `setTextAlign()` | ◎ Left/Right/Center/Justify 대응 |
| `margin_left/right` | 해당 없음 (외부 처리) | — Skia는 문단 여백 미지원, 레이아웃 레벨에서 처리 |
| `indent` (들여쓰기) | 해당 없음 | — 첫 줄 들여쓰기 직접 미지원 |
| `spacing_before/after` | 해당 없음 | — 문단 간격은 외부 처리 |
| `line_spacing` + `line_spacing_type` | StrutStyle `setHeight()` + `setLeading()` | △ HWP 줄 간격 종류(Percent/Fixed/BetweenLines/AtLeast)와 완전 대응 어려움 |
| `tab_def_id` (탭 정의) | `setReplaceTabCharacters()` | ✗ Skia 탭 지원 극히 제한적 |
| `numbering_id` (번호/글머리표) | 해당 없음 | ✗ 외부 구현 필요 |
| `border_fill_id` (문단 테두리) | 해당 없음 | ✗ 외부 구현 필요 |

### LINE_SEG ↔ LineMetrics

| HWP LINE_SEG | SkParagraph LineMetrics | 매핑 가능성 |
|--------------|------------------------|-------------|
| `vpos` (수직 위치) | `baseline` | △ 좌표계 다름 |
| `line_height` | `height` (ascent + descent + leading) | ○ |
| `text_height` | `ascent + descent` | ○ |
| `baseline` | `baseline` | ◎ |
| `line_spacing` | `height - (ascent + descent)` | △ |
| `column_spacing` | 해당 없음 | — 외부 처리 |
| `segment_width` | `width` | ◎ |

## skia-safe Rust 바인딩 및 WASM 가능성

### Rust 바인딩 (skia-safe 크레이트)

- `skia_safe::textlayout` 모듈에서 Paragraph, ParagraphBuilder 등 제공
- Paragraph::layout(), paint(), get_line_metrics() 등 주요 API 바인딩 완료
- UTF-16 오프셋 지원 메서드 포함 (`get_line_number_at_utf16_offset()` 등)

### WASM 빌드 상태

| 타겟 | 상태 | 비고 |
|------|------|------|
| `wasm32-unknown-emscripten` | ○ 지원 | Emscripten 환경 필요 |
| `wasm32-unknown-unknown` | ✗ 미지원 | wasm-bindgen/wasm-pack 사용 불가 |
| `wasm32-wasi` | ✗ 미지원 | 호환성 문제 |

**핵심 문제**: rhwp는 `wasm32-unknown-unknown` + `wasm-pack`으로 빌드하는데, skia-safe는 이 타겟을 지원하지 않음. Emscripten 기반으로 전환하면 기존 WASM 빌드 체인 전면 교체 필요.

## 장단점 정리

### 장점

1. **검증된 프로덕션 품질**: Flutter, Chrome에서 사용 중. 텍스트 셰이핑/줄바꿈/BiDi 완전 지원
2. **풍부한 조회 API**: LineMetrics, getRectsForRange(), getGlyphPositionAtCoordinate() — 에디터 기능에 직접 활용 가능
3. **HarfBuzz 통합**: 커닝, 리가처, 복잡 스크립트 자동 처리
4. **Rust 바인딩 존재**: skia-safe 크레이트를 통한 접근 가능

### 단점

1. **WASM 호환성 치명적**: `wasm32-unknown-unknown` 미지원. rhwp WASM 빌드 체인과 호환 불가
2. **C++ 의존성**: 순수 Rust가 아닌 FFI 바인딩. 빌드 복잡도 대폭 증가
3. **HWP 특수 속성 미지원**: 장평, 언어별 자간/폰트, 강조점, 양각/음각 등 HWP 고유 속성 매핑 불가
4. **문단 레벨 기능 부족**: 여백, 들여쓰기, 문단 간격, 탭 정의 등은 Skia 외부에서 처리 필요
5. **거대한 의존성**: Skia 전체 빌드가 필요 (빌드 시간, 바이너리 크기 대폭 증가)

## 결론

SkParagraph는 텍스트 셰이핑/줄바꿈 품질은 최고 수준이나, **WASM(`wasm32-unknown-unknown`) 미지원**과 **HWP 고유 속성 매핑 한계**로 인해 rhwp에 직접 도입하기 어렵다. 다만 API 설계 패턴(ParagraphBuilder, LineMetrics, 좌표 역매핑)은 rhwp 조판 시스템 재설계 시 참고할 가치가 있다.
