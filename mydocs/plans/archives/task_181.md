# 타스크 181: 한컴 수식 렌더링 기능 구현 — 수행계획서

## 배경

현재 HWP 파일의 수식(Equation) 컨트롤은 파싱·렌더링이 미구현된 상태이다.

### 현재 상태

| 항목 | 바이너리 HWP | HWPX |
|------|-------------|------|
| 파서 | `CTRL_EQUATION` 미처리 → `Control::Unknown` | `parse_equation()` → `Control::Shape(Rectangle)` (레이아웃만) |
| 모델 | 전용 모델 없음 | 동일 |
| 렌더링 | 빈 사각형 또는 무시 | 빈 사각형 |

### HWP 수식 시스템 개요

한컴 수식은 LaTeX와 유사한 **스크립트 언어**로 표현된다:
- `1 over 2` → 분수 ½
- `sqrt x` → 제곱근 √x
- `E=mc^2` → 위첨자
- `sum_{i=0}^n` → 시그마 합
- `matrix{a & b # c & d}` → 행렬

스크립트는 `HWPTAG_EQEDIT` 레코드(바이너리) 또는 `<hp:script>` 요소(HWPX)에 UTF-16 문자열로 저장된다.

### 참조 구현

`/home/edward/vsworks/shwp/hwp_semantic/equation/` (Python)
- `tokenizer.py` — 스크립트 토크나이저 (TokenType: COMMAND, NUMBER, SYMBOL, SUBSCRIPT, SUPERSCRIPT 등)
- `symbols.py` — HWP 명령어 → LaTeX 기호 매핑 테이블
- `latex.py` — 토큰 → LaTeX 문자열 변환기

## 목표

1. 바이너리 HWP / HWPX 양쪽에서 수식 스크립트 추출
2. 수식 스크립트를 파싱하여 AST(추상 구문 트리) 생성
3. AST를 SVG 요소로 렌더링 (분수선, 루트 기호, 첨자 배치 등)
4. 기존 렌더링 파이프라인에 수식 컨트롤 통합

## 렌더링 전략

LaTeX 변환 대신 **HWP 수식 스크립트 → AST → SVG 직접 렌더링** 방식을 채택한다.

이유:
- LaTeX 렌더링 엔진(MathJax 등)을 내장하는 것은 과도한 의존성
- SVG 환경에서는 텍스트, 선, 경로를 직접 배치하는 것이 자연스러움
- 수식의 레이아웃 규칙은 상대적으로 단순(분수·첨자·루트 등의 배치 공식)

### AST 노드 구조 (개념)

```
EqNode
├── Text("x")              // 일반 텍스트/변수
├── Number("123")           // 숫자
├── Symbol(SymbolKind)      // 수학 기호 (×, ±, →, α 등)
├── Fraction(num, den)      // 분수: a over b
├── Sqrt(index, body)       // 제곱근: sqrt x, ^3 sqrt x
├── Superscript(base, sup)  // 위첨자: x^2
├── Subscript(base, sub)    // 아래첨자: x_i
├── SubSup(base, sub, sup)  // 위+아래: x_i^2
├── BigOp(kind, sub, sup, body) // 큰 연산자: ∫, Σ, Π, ∪, ∩
├── Limit(sub)              // lim
├── Matrix(rows, cols, style) // 행렬
├── Cases(rows)             // 경우 분기
├── Paren(left, right, body) // 괄호 (LEFT/RIGHT)
├── Decoration(kind, body)  // 장식: hat, bar, vec 등
├── FontStyle(style, body)  // rm, bold, it
├── Pile(align, rows)       // 세로 쌓기
├── Space(kind)             // ~, `
├── Row(children)           // 수평 나열
├── Newline                 // # (줄바꿈)
└── Color(r,g,b, body)     // COLOR
```

### SVG 렌더링 배치 규칙

| 구조 | 배치 방식 |
|------|----------|
| 분수 | 분자/분모 수직 중앙, 분수선 가로 |
| 첨자 | 기본 글자 대비 축소(~70%), 위/아래 오프셋 |
| 루트 | √ 경로(path) + 상단 가로선 |
| 큰 연산자 | 기호 확대, 위/아래 첨자 중앙 배치 |
| 행렬 | 격자 배치, 열 폭 최대값 기준 |
| 괄호 | 내용 높이에 맞춰 스케일 |

## 구현 단계

### 1단계: 모델 + 파서 (Equation 데이터 추출)

**목표**: 바이너리 HWP / HWPX에서 수식 스크립트 문자열을 추출하여 모델에 저장

- `src/model/control.rs`: `Equation` 구조체 + `Control::Equation` variant 추가
- `src/parser/control.rs`: `CTRL_EQUATION` → `parse_equation_control()` (HWPTAG_EQEDIT에서 스크립트 추출)
- `src/parser/hwpx/section.rs`: `parse_equation()` 수정 (스크립트 요소 파싱)
- 기존 write-back 코드에 Equation 처리 추가

### 2단계: 수식 토크나이저 + 기호 매핑

**목표**: 수식 스크립트 문자열을 토큰 스트림으로 변환

- `src/renderer/equation/mod.rs`: 모듈 구조
- `src/renderer/equation/tokenizer.rs`: Tokenizer 구현 (Python 참조 포팅)
- `src/renderer/equation/symbols.rs`: HWP 명령어 → Unicode/SVG 기호 매핑

### 3단계: 수식 파서 (토큰 → AST)

**목표**: 토큰 스트림을 AST(EqNode 트리)로 변환

- `src/renderer/equation/parser.rs`: 재귀 하강 파서
- `src/renderer/equation/ast.rs`: EqNode enum 정의
- 주요 구조: OVER(분수), SQRT(루트), ^/_(첨자), MATRIX, CASES, LEFT/RIGHT, 장식

### 4단계: 수식 레이아웃 엔진 (AST → 배치 박스)

**목표**: AST를 크기+위치가 결정된 LayoutBox 트리로 변환

- `src/renderer/equation/layout.rs`: 레이아웃 엔진
- 각 노드별 크기 계산 (글꼴 메트릭 기반)
- 자식 노드 위치 결정 (분수선 위치, 첨자 오프셋 등)
- baseline 정렬 처리

### 5단계: SVG 렌더링 + 파이프라인 통합

**목표**: LayoutBox 트리를 SVG 요소로 변환, 기존 렌더링 파이프라인에 통합

- `src/renderer/equation/svg_render.rs`: SVG 요소 생성
- `src/renderer/layout.rs`: Equation 컨트롤 레이아웃 처리
- `src/renderer/svg.rs`: Equation 렌더 노드 SVG 출력
- `src/renderer/render_tree.rs`: RenderNodeType::Equation 추가

### 6단계: 테스트 + 검증

**목표**: 수식 샘플 문서로 렌더링 품질 검증

- 단위 테스트: 토크나이저, 파서, 레이아웃 각 모듈
- 통합 테스트: 수식 포함 HWP 파일 SVG 내보내기
- 시각적 검증: 실제 한컴 렌더링 결과와 비교

## 범위

### 포함
- 기본 명령어: OVER, SQRT, ^, _, INT/SUM/PROD, MATRIX, CASES, LEFT/RIGHT
- 글자 장식: hat, bar, vec, dot, tilde 등
- 그리스 문자 + 수학 기호 (Unicode 매핑)
- 글꼴 스타일: rm, bold, it
- 공백/줄바꿈: ~, `, #, &
- 괄호 자동 크기: LEFT/RIGHT

### 제외 (후속 타스크)
- LADDER, SLADDER, LONGDIV (특수 레이아웃)
- SCALE (크기 비율 조정, HWP3 레거시)
- REL, BUILDREL (화살표 위/아래 텍스트)
- COLOR (색상 지정)
- 수식 편집 UI
