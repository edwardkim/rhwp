---
kind: reference
status: active
canonical: mydocs/tech/hancom_native_line_semantics.md
last_verified: 2026-07-19
---

# 한컴 네이티브 렌더링 시맨틱 — 정합 모델 정리

> 목적: 한컴 워드프로세서의 **렌더링 파이프라인/백엔드/폰트-글리프 시맨틱**을 정규 모델로 정리하고, rhwp 렌더링 문서(`hancom_font_system_analysis.md`, `rendering_engine_design.md`, `hwp_table_rendering.md`, `emf_spec.md` 등)와 대조한다. 구현 세부가 아니라 **결과가 따르는 규칙(behavioral spec)** 을 기술한다.

## 1. 두 개의 텍스트 렌더링 경로 (핵심)

### 1.1 본문 텍스트 — 자체 글리프 엔진
- **OS 텍스트 배치 API 미사용.** 본문 텍스트는 OS 의 텍스트 배치 API 로 advance 를 얻지 않고 **자체 글리프 엔진으로 스스로 계산**한다.
- **글리프 합성 모델:** 하나의 **GlyphChar** 는 **N개의 ELEMENT** 로 구성되며, 각 element 는 `glyph index + dx + offset` 을 가진다. 총 advance 는 GlyphChar width 로 설정한다. → 옛한글/복합 음절을 자모 element 로 합성하는 기반이며, rhwp `.hft` **type:2 한글 음절 분해**와 정확히 대응한다.
- **폭(advance) 소스:** 한컴 자체 메트릭(`.hft`): `type0` 균일 / `type1` 문자별 / `type2` 한글 음절 분해. 폭 보정: **bold ≈ +5%**, **super/sub ×0.64**.
- **아웃라인(그리기):** 글리프 아웃라인을 직접 얻어 자체 배치·래스터한다.
- 이 경로가 라인 나눔 모델의 누적 advance `A = Σ(glyph + ratio + spacing + pair + object)` 의 근거다.

### 1.2 도형/DrawingML 텍스트 — GDI+
- 도형 텍스트는 GDI+ 폰트 계층을 사용한다: Font/FontFamily(임베드 폰트용 Private 컬렉션), FontMetrics(EmSize/Height/BaseAlign/Margin). 폰트 메트릭은 GDI+ FontFamily 메트릭으로 위임한다.

## 2. 도형 렌더링 엔진

- **백엔드:** **GDI+**(벡터) + **GDI32**(래스터/EMF) + **3D 서브시스템** + **양방향/RTL 서브시스템**. **Direct2D/DirectWrite 미사용**.
- **색 모델:** `DrawingType::{Rgb,Hsl,Cmyk,Color}`; COLORREF(BGR 바이트 순서).
- **채우기:** Brush, GradientBrush/CircleGradientBrush/GradientStops(Style, Degree=각도).
- **경로:** GDI+ GraphicsPath 기반.
- **효과:** Bevel(BevelStyle), Inner/OuterShadow(Color, Degree=각도, blur), 이미지 효과 Duotone/Grayscale/BiLevel/AlphaMod/ArtEffect.
- **3D:** Scene3D(Camera Latitude/Longitude/Revolution, LightRig type/direction), Sp3D(MaterialStyle, ExtrusionHeight), 3D Object(ShapeType, WireFrame, Path), PolygonMesh(face/side/back × fill/stroke), ApplyLightShade/ApplyProjective. → WordArt/도형 3D 베벨·조명·압출(OOXML DrawingML 3D 호환).
- **래스터/이미지:** DIB 스트레치 blit + EMF.
- OOXML 매핑: DrawingML 루트 파트/관계 구조로 대응.

## 3. 단위 / 좌표

- `HWPUNIT = 1/7200 inch`; `px = hwpunit × dpi / 7200`(기본 96dpi). A4 = 59528×84188 HWPUNIT.
- EMF: 물리 `0.01mm`, `XFORM` 2×3 아핀, 좌표 파이프라인 `logical → World(XFORM) → Window/Viewport(MapMode) → physical`.

## 4. rhwp 대조 / 함의

- rhwp 가 이미 규명한 `.hft`/advance/border(18종·width index 0–15)/EMF 의미가 **네이티브 구조와 정합**한다. 본문 텍스트 parity 는 (a) `.hft` 폭 + (b) 글리프-element 합성(dx/offset/glyph-index) + (c) `A` 누적 순서를 함께 따라야 한다.
- 도형 parity 는 **GDI+ 시맨틱**(gradient stop/degree, dash, Bevel/Shadow 의 Degree·blur, 이미지 효과) 재현이 필요하고, 3D 는 카메라/조명/압출 파라미터를 따른다.

## 5. 이미지 효과 알고리즘

이미지 이펙트는 **GDI+ 가 아닌 한컴 자체 픽셀 이펙터**(원시 ARGB 배열 대상)로 구현된다.

- **Duotone**: **256-엔트리 gray→color LUT**(color0..color1 보간)를 만들어 모든 픽셀을 휘도 인덱스로 매핑. OOXML duotone 이미지 효과에 대응.
- **Blur**: **분리형(separable) 슬라이딩 윈도우** 블러 — 채널별(ARGB) 누적합을 `[i-radius, i+radius]` 창으로 이동(박스/삼각 커널), 행 끝 edge-clamp. Inner/OuterShadow 의 그림자 블러에 사용. 알파-전용 변형 존재.
- **참고:** 그라디언트 *보간* 자체는 GDI+ 가 담당하며 — 한컴은 GradientStops/Degree/TileStyle 설정만 한다. 즉 이펙트(duotone/blur/grayscale/art)는 자체 구현, 벡터 채우기/그라디언트/이미지 blit 은 GDI+ 위임이다.

## 6. `.hft` 폭 엔진 — 모델

### 6.1 문자 폭 계산
- 요청 스케일 폭 = **원시폭을 요청 em(1000) 기준으로 정규화**(폰트별 emsize 1000/1200/512/1024 → 요청 em 정규화).
- raw 폭: 폰트 엔트리에서 문자별 raw width 조회(4-엔트리 스타일 폴백; type 0/1/2). 미싱 글리프 기본폭 1200.
- **플래그별 폭 보정(모델):**
  - **bold**: `width += (raw+10)/20` (≈ +5%) — rhwp `hancom_font_system_analysis.md` 의 `(emsize+10)/20` 과 동일.
  - 별도 굵기 플래그: `width += (raw+8)/16` (~+6%).
  - **super/sub**: `width ×= 16/25` (×0.64) — rhwp 문서와 동일.

### 6.2 GlyphChar 합성
- GlyphChar = N element(각 glyph index + dx + offset) 합성 + 총 width. §1.1 모델의 실체.

### 6.3 이미지 이펙트 추가
- **Grayscale**: `gray = luminance(px)`; 중간톤 washout([128,255])로 압축; 알파 보존. Duotone 의 gray 인덱스와 luminance 를 공유한다.

## 7. 네 개의 서브시스템 — 결과

- **type2 한글 음절 분해**: range 별 타입 판정; type0 균일, type1 문자별(옛한글은 KS/자소 검색 폴백), **type2** = (cho, jung, jong) 3-그룹 인덱싱으로 음절→글리프 해소, type3 = 폰트별 콜백. rhwp `.hft` type0/1/2 정합.
- **글리프 래스터/안티에일리어싱** — 커스텀 래스터라이저가 **아니다**. 전역 폰트 품질 설정만 자체적으로 하며, 실제 아웃라인/AA/힌팅은 **OS 글리프 아웃라인 경로**가 담당한다.
- **수식(Equation)** — 수식 모듈은 별도 컴포넌트로 분리되어 레이아웃/렌더된다. rhwp `equation_support_status.md` 가 이미 85–90% 커버.
- **표 셀 레이아웃** — 실제 레이아웃은 문단 흐름(FL flow) 코드에 분산되어 있다(표 조각화 = FL 경로). 셀 col/row 수식은 rhwp `hwp_table_rendering.md` 에 문서화되어 있다.
