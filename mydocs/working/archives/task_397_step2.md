# Task 397 — 2단계 완료 보고서: cosmic-text 심층 분석

## 아키텍처

### 파이프라인

```
FontSystem (앱 전역 1개)
  → fontdb 기반 폰트 탐색/캐싱

Buffer (텍스트 위젯당 1개)
  → set_text(text, Attrs) 또는 set_rich_text(spans, Attrs)
  → BufferLine[] (줄 단위 관리)
      → shape(FontSystem)     // BiDi 분석 → 셰이핑 → ShapeLine
      → layout(font_size, width, wrap, hinting)  // 줄바꿈 → LayoutLine[]
  → layout_runs()             // 렌더링용 이터레이터
  → hit(x, y)                 // 좌표 → Cursor 역매핑
  → draw(callback)            // 렌더링
```

### 핵심 구성요소

| 구성요소 | 역할 |
|----------|------|
| **FontSystem** | fontdb 기반 폰트 탐색. 3단계 캐싱 (폰트/매칭/코드포인트) |
| **Buffer** | 다중 줄 텍스트 컨테이너. 셰이핑/레이아웃 결과 캐싱 |
| **BufferLine** | 단일 줄(문단). text + AttrsList로 리치 텍스트 지원 |
| **AttrsList** | 문자 범위별 Attrs 매핑. 한 줄 내 다중 스타일 |
| **Attrs** | 폰트 패밀리, weight, stretch, style, color, letter_spacing, font_features |
| **Metrics** | font_size(px) + line_height(px) 2개 값만 |
| **ShapeLine** | 셰이핑 결과 (harfrust 기반 글리프 시퀀스) |
| **LayoutLine** | 레이아웃 결과 (w, max_ascent, max_descent, glyphs) |
| **Editor** | Buffer 래퍼. 커서/선택/편집 기능 |

### 내부 엔진

- **텍스트 셰이핑**: harfrust (HarfBuzz의 Rust 포팅) — 리가처, 커닝, 복잡 스크립트 지원
- **줄바꿈**: unicode-linebreak + 4가지 Wrap 모드 (None/Word/Glyph/WordOrGlyph)
- **BiDi**: unicode-bidi (전체 Unicode BiDi Algorithm)
- **폰트 폴백**: fontdb + 플랫폼별 폴백 리스트 (Chromium/Firefox 정적 리스트 재사용)
- **렌더링**: swash (글리프 래스터라이징, 4×4 서브픽셀 비닝)

### 캐싱 전략

| 레벨 | 대상 | 무효화 조건 |
|------|------|------------|
| Shape 캐시 | BufferLine별 | 텍스트/속성 변경 시 |
| Layout 캐시 | BufferLine별 | Metrics/width/wrap 변경 시 |
| Font 캐시 | FontSystem 전역 | 영구 (Arc 공유) |
| Font match 캐시 | FontSystem 전역 | LRU 256개 |
| Glyph 래스터 캐시 | SwashCache | 영구 |
| Shape buffer 재사용 | 호출 간 | 매 호출 재사용 |

## HWP 속성 매핑 분석

### CharShape ↔ Attrs

| HWP CharShape | cosmic-text Attrs | 매핑 가능성 |
|---------------|-------------------|-------------|
| `font_ids[7]` (언어별) | `family(Family)` | △ 단일 패밀리만. 언어별 폰트 전환은 폰트 폴백에 의존 |
| `base_size` (HWPUNIT) | `Metrics.font_size` (px) | ◎ 단위 변환 필요 |
| `ratios[7]` (장평) | `stretch(Stretch)` | △ Stretch는 CSS font-stretch로 HWP 장평과 다른 개념 |
| `spacings[7]` (자간) | `letter_spacing_opt` (EM 단위) | △ 단일 값, 언어별 자간 불가 |
| `bold` | `weight(Weight::BOLD)` | ◎ |
| `italic` | `style(Style::Italic)` | ◎ |
| `text_color` | `color_opt(Color)` | ◎ |
| `underline/strikethrough` | 해당 없음 | ✗ 텍스트 장식은 cosmic-text 범위 밖, 외부 렌더링 |
| `shadow` | 해당 없음 | ✗ 외부 렌더링 |
| `subscript/superscript` | `metrics_opt` (크기 조절) | △ 위치 오프셋은 수동 |
| `relative_sizes[7]` | `metrics_opt` | △ 언어별 개별 적용 불가 |
| `char_offsets[7]` | 해당 없음 | ✗ |
| `kerning` | harfrust 자동 처리 | ◎ |
| `font_features` (OpenType) | `font_features` | ◎ Attrs에 직접 지원 |
| `emboss/engrave/emphasis_dot` | 해당 없음 | ✗ 외부 렌더링 |

### ParaShape ↔ Buffer/Metrics

| HWP ParaShape | cosmic-text | 매핑 가능성 |
|---------------|-------------|-------------|
| `alignment` | `Align` (Left/Right/Center/Justified/End) | ◎ |
| `line_spacing` + `line_spacing_type` | `Metrics.line_height` (px 단일 값) | △ HWP 4가지 줄간격 종류와 대응 어려움 |
| `margin_left/right` | 해당 없음 | — 외부 레이아웃에서 처리 |
| `indent` | 해당 없음 | — 외부 레이아웃에서 처리 |
| `spacing_before/after` | 해당 없음 | — 외부 레이아웃에서 처리 |
| `tab_def_id` | `Buffer.set_tab_width(u16)` | △ 고정 탭 너비만, 커스텀 탭 스톱 미지원 |
| `numbering_id` | 해당 없음 | ✗ 외부 구현 |
| `border_fill_id` | 해당 없음 | ✗ 외부 구현 |

### LayoutLine ↔ LINE_SEG

| cosmic-text LayoutLine | HWP LINE_SEG | 매핑 가능성 |
|------------------------|--------------|-------------|
| `w` (너비) | `segment_width` | ◎ |
| `max_ascent` | `baseline` (부분적) | △ |
| `max_descent` | `text_height - baseline` | △ |
| `max_ascent + max_descent` | `text_height` | ○ |
| `line_height_opt` | `line_height` | △ HWP line_height는 간격 포함 |
| `glyphs` (Vec\<LayoutGlyph\>) | 해당 없음 | — HWP는 글리프 수준 정보 미보유 |

## WASM 빌드 가능성

### 현황

| 항목 | 상태 |
|------|------|
| 순수 Rust | ◎ C/C++ 의존성 없음 |
| `wasm32-unknown-unknown` 타겟 | △ 공식 확인 안 됨, 하지만 구조상 가능 |
| `no_std` 지원 | ○ `default-features = false`로 셰이핑+레이아웃 사용 가능 |
| fontdb (폰트 탐색) | ✗ WASM에서 시스템 폰트 접근 불가 |
| swash (래스터라이징) | — rhwp는 Canvas/SVG 렌더링이므로 불필요 |

### WASM 적용 시 고려사항

1. **폰트 로딩**: `FontSystem::new()` 대신 `FontSystem::new_with_fonts([])` + 수동 폰트 등록 필요
2. **폰트 데이터**: WASM 번들에 폰트 파일 포함하거나 fetch로 동적 로드
3. **no_std 모드**: 셰이핑+레이아웃만 사용 시 시스템 의존성 제거 가능
4. **바이너리 크기**: harfrust 포함 시 WASM 크기 증가 예상 (추정 1~3MB)

## 장단점 정리

### 장점

1. **순수 Rust**: C/C++ FFI 없음. 빌드 간결, `wasm32-unknown-unknown` 호환 가능성 높음
2. **harfrust 텍스트 셰이핑**: 커닝, 리가처, 복잡 스크립트 자동 처리
3. **BiDi 지원**: unicode-bidi 기반 RTL/LTR 혼합 텍스트
4. **리치 텍스트**: AttrsList로 한 줄 내 다중 스타일 범위 지원
5. **편집 기능 내장**: Editor, Cursor, Selection, hit() 좌표 역매핑
6. **공격적 캐싱**: Shape/Layout/Font 3단계 캐시로 편집 시 점진적 재계산 가능
7. **활발한 유지보수**: System76/COSMIC 데스크톱 핵심 라이브러리 (v0.18.2, 2026-02)
8. **라이선스**: MIT / Apache-2.0 (rhwp MIT와 호환)

### 단점

1. **HWP 고유 속성 미지원**: 장평, 언어별 자간/폰트, 강조점, 양각/음각 등
2. **Metrics 제한**: font_size + line_height 2개뿐. HWP의 4가지 줄간격 종류, 문단 간격 등 미지원
3. **문단 레벨 기능 부족**: 여백, 들여쓰기, 문단 간격, 커스텀 탭 스톱 등 외부 처리 필요
4. **텍스트 장식 미지원**: 밑줄, 취소선, 그림자 등은 외부 렌더링 필요
5. **WASM 폰트 로딩**: 시스템 폰트 접근 불가, 수동 등록 또는 동적 로드 필요
6. **WASM 바이너리 크기**: harfrust 포함 시 크기 증가

## 결론

cosmic-text는 순수 Rust + 텍스트 셰이핑(harfrust) + BiDi + 편집 기능을 갖춘 현대적 텍스트 레이아웃 라이브러리로, WASM 호환 가능성과 rhwp 기술 스택(Rust + wasm-pack)과의 친화성이 높다.

다만 cosmic-text는 **범용 텍스트 편집기**를 위한 라이브러리로, HWP의 **문서 조판 엔진**이 요구하는 문단 레벨 기능(여백, 들여쓰기, 줄간격 종류, 문단 간격, 탭 정의 등)은 범위 밖이다. 따라서 cosmic-text를 전면 도입하기보다는 **셰이핑 엔진(harfrust)과 줄바꿈 엔진만 선별 도입**하고, 문단/페이지 레이아웃은 rhwp 자체 구현을 유지·강화하는 방향이 현실적이다.
