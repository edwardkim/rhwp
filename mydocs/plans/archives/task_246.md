# Task 246 수행 계획서: 한컴 수준 도형 완성

## 현황 분석

### 이미 구현된 기능
- 도형 7종 모델/파서/렌더러: 직선, 사각형, 타원, 호, 다각형, 곡선, 묶음
- 연결선($col) 3종: 직선/꺽인/곡선 + 연결점 스냅 + 자동 추적
- 도형 속성 편집: 테두리 색/굵기, 채우기(단색/그라디언트), 회전/대칭
- 그룹 묶기/풀기, 다중 선택, Z-order

### 미구현 기능 (한컴 대비)
1. **화살표 렌더링** — 모델에 ArrowStyle이 있으나 SVG에 미반영
2. **도형 그림자** — shadow_type/color/offset이 모델에 있으나 렌더링 안 됨
3. **둥근 사각형** — round_rate 필드 있으나 렌더링 미적용
4. **선 종류** — 이중선/삼중선 등 LineRenderType 미반영
5. **도형 카테고리 확장** — 말풍선, 순서도 등 (한컴 기본도형 셋)

## 구현 계획

### 1단계: 화살표 렌더링
- SVG marker를 사용한 선/연결선 화살표 표시
- ArrowStyle 6종: 화살표, 오목화살표, 다이아몬드, 원, 사각형
- 화살표 크기 3단계 (소/중/대)
- 대상: 직선($lin), 연결선($col)

### 2단계: 도형 그림자
- SVG filter (feDropShadow 또는 feOffset+feGaussianBlur)
- shadow_type(방향) + shadow_color + shadow_offset + shadow_alpha
- 사각형, 타원, 다각형, 호 등 모든 도형에 적용

### 3단계: 선 종류 + 둥근 사각형
- 이중선/삼중선 렌더링 (LineRenderType)
- 둥근 사각형: round_rate → SVG rect rx/ry
- 선 끝 모양 (lineEndShape)

### 4단계: 도형 속성 UI 보완
- 화살표 속성 편집 (시작/끝 모양, 크기)
- 그림자 속성 편집 (종류, 색상, 오프셋)
- 선 종류 선택 UI

## 참조
- 한컴 도움말: draw/line/, draw/face/
- HWP 스펙: 표 84~86 (개체 요소 속성)
- hwplib: ArrowStyle, LineRenderType
