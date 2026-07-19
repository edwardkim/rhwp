---
kind: snapshot
status: historical
canonical: mydocs/tech/investigations/issue-139/README.md
last_verified: 2026-07-17
---

# HWP 수식 파서 지원 현황

작성일: 2026-03-23

## 개요

HWP 수식 스크립트의 파싱→레이아웃→SVG/Canvas 렌더링 파이프라인 지원 현황.
스펙 문서: `mydocs/tech/hwp_spec_equation.md`

현재 구현 수준: **약 85~90% 완성**

## 코드 위치

| 파일 | 역할 |
|------|------|
| `src/renderer/equation/tokenizer.rs` | 수식 스크립트 토크나이징 |
| `src/renderer/equation/parser.rs` | AST 파싱 (1068줄) |
| `src/renderer/equation/ast.rs` | AST 노드 정의 |
| `src/renderer/equation/symbols.rs` | 기호 및 명령어 맵 |
| `src/renderer/equation/layout.rs` | 레이아웃 엔진 |
| `src/renderer/equation/svg_render.rs` | SVG 렌더러 |
| `src/renderer/equation/canvas_render.rs` | Canvas 렌더러 |

## 구현 완료 항목

### 기본 구문

| 명령어 | 설명 | 예시 |
|--------|------|------|
| OVER | 분수 | `a OVER b` |
| ATOP | 분수 (선 없음) | `a ATOP b` |
| SQRT | 제곱근 | `SQRT{x}` |
| ROOT | n제곱근 | `ROOT{n}{x}` |
| ^ | 위첨자 | `x^{2}` |
| _ | 아래첨자 | `x_{n}` |
| LEFT-RIGHT | 자동 크기 괄호 | `LEFT( x RIGHT)` |
| CHOOSE, BINOM | 조합 | `CHOOSE{n}{r}` |
| COLOR | 색상 | `COLOR{255,0,0}{text}` |

### 행렬/배열

| 명령어 | 설명 |
|--------|------|
| MATRIX | 괄호 없는 행렬 |
| PMATRIX | 소괄호 행렬 |
| BMATRIX | 대괄호 행렬 |
| DMATRIX | 세로줄 행렬 |
| CASES | 조건식 (중괄호 왼쪽) |
| PILE | 세로 쌓기 (가운데 정렬) |
| LPILE | 세로 쌓기 (왼쪽 정렬) |
| RPILE | 세로 쌓기 (오른쪽 정렬) |

### 큰 연산자

| 명령어 | 기호 | 설명 |
|--------|------|------|
| INT | ∫ | 적분 |
| OINT | ∮ | 선적분 |
| SUM | Σ | 합 |
| PROD | Π | 곱 |
| UNION | ∪ | 합집합 |
| INTER | ∩ | 교집합 |
| COPROD | ∐ | 여곱 |
| BIGUNION | ⋃ | 큰 합집합 |
| BIGINTER | ⋂ | 큰 교집합 |
| BIGSQUNION | ⊔ | 큰 분리합집합 |

### 극한

| 명령어 | 설명 |
|--------|------|
| lim | 극한 (소문자) |
| Lim | 극한 (대문자) |

### 글자 장식 (Decoration)

| 명령어 | 설명 | 렌더링 |
|--------|------|--------|
| hat | 꺽쇠 (^) | ✓ |
| check | 역꺽쇠 (ˇ) | ✓ |
| tilde | 물결 (~) | ✓ |
| acute | 양음부호 (´) | ✓ |
| grave | 역양음부호 (`) | ✓ |
| dot | 점 (˙) | ✓ |
| ddot | 쌍점 (¨) | ✓ |
| bar | 윗줄 (¯) | ✓ |
| vec | 벡터 화살표 (→) | ✓ |
| dyad | 양방향 화살표 (↔) | ✓ |
| under | 아랫줄 | ✓ |
| arch | 아치 | ✓ |
| UNDERLINE | 밑줄 | ✓ |
| OVERLINE | 윗줄 | ✓ |
| NOT | 취소선 | ✓ |

### 글꼴 스타일

| 명령어 | 설명 |
|--------|------|
| rm | 로만체 (직립체) |
| it | 이탤릭체 |
| bold | 볼드체 |

### 기호

| 분류 | 구현 항목 |
|------|----------|
| 그리스 소문자 | alpha~omega (24종) + 변형(vartheta, varpi 등) |
| 그리스 대문자 | Alpha~Omega (24종) |
| 관계 기호 | =, NEQ, LEQ, GEQ, APPROX, CONG, EQUIV, PREC, SUCC 등 |
| 연산 기호 | TIMES, DIV, PLUSMINUS, MINUSPLUS, CDOT, BULLET 등 |
| 집합 기호 | SUBSET, SUPERSET, IN, OWNS, SUBSETEQ, SUPSETEQ 등 |
| 화살표 | larrow, rarrow, LARROW, RARROW, uparrow, downarrow, mapsto 등 |
| 점 기호 | CDOTS, LDOTS, VDOTS, DDOTS |
| 기타 | INF, EMPTYSET, ANGLE, TRIANGLE, NABLA, PARTIAL, FORALL, EXISTS 등 |

### 함수 (자동 로만체)

삼각함수: sin, cos, tan, cot, sec, csc, arcsin, arccos, arctan, sinh, cosh, tanh, coth
로그/지수: log, ln, lg, exp
기타: det, dim, ker, hom, arg, deg, gcd, lcm, max, min, mod, asin, acos, atan

### 공백 및 제어

| 기호 | 설명 |
|------|------|
| ~ | 보통 공백 |
| ` | 1/4 공백 |
| # | 줄바꿈 |
| & | 세로 칸 맞춤 (탭) |
| "" | 9자 이상 한 단어 묶음 |

### 왼쪽 첨자

| 명령어 | 설명 |
|--------|------|
| LSUB | 왼쪽 아래첨자 |
| LSUP | 왼쪽 위첨자 |

## 미구현 항목

### 우선순위 높음

| 명령어 | 설명 | 발견 위치 | 비고 |
|--------|------|-----------|------|
| **EQALIGN** | 칸 맞춤 정렬 (& 기준 세로 위치 조절) | exam_math.hwp 4번 문항 | B-003 백로그 등록. PILE과 유사하나 & 기준 열 정렬 필요 |

### 우선순위 중간

| 명령어 | 설명 | 비고 |
|--------|------|------|
| **REL** | 관계식 — 화살표 위/아래에 조건식 삽입 | `a REL{→}{조건} b` |
| **BUILDREL** | REL 변형 — 화살표 아래 생략 | `a BUILDREL{→}{위} b` |
| **LONGDIV** | 장제법 (나눗셈 표현) | 초등수학 문서에서 사용 |
| **DINT** | 이중 적분 (∬) | 기호 맵에 미등록 |
| **TINT** | 삼중 적분 (∭) | 기호 맵에 미등록 |
| **ODINT** | 이중 선적분 | 기호 맵에 미등록 |
| **OTINT** | 삼중 선적분 | 기호 맵에 미등록 |

### 우선순위 낮음

| 명령어 | 설명 | 비고 |
|--------|------|------|
| **LADDER** | 최소공배수/최대공약수 사다리꼴 | 초등수학 특화 |
| **SLADDER** | 진법 변환 사다리꼴 (10진수→2진수) | 초등수학 특화 |
| **BIGG** | 요소 크기 확대 | 현재 파싱만 됨 (크기 확대 무시) |
| **BENZENE** | 벤젠 분자 구조 | 화학 특화 |

### 부분 구현

| 명령어 | 현재 상태 | 필요 작업 |
|--------|-----------|-----------|
| **BIGG** | 파싱 시 내부 요소만 반환 (크기 확대 무시) | 크기 배율 적용 |
| **&** (탭) | 토큰화는 되나 PILE 내부에서만 정렬 처리 | EQALIGN에서 열 정렬 활용 |

## exam_math.hwp 수식 분석

samples/exam_math.hwp에서 사용되는 수식 명령어:

| 명령어 | 사용 빈도 | 구현 여부 |
|--------|-----------|-----------|
| OVER (분수) | 매우 높음 | ✓ |
| SQRT (제곱근) | 높음 | ✓ |
| ^, _ (첨자) | 매우 높음 | ✓ |
| LEFT-RIGHT | 높음 | ✓ |
| lim | 중간 | ✓ |
| INT | 중간 | ✓ |
| SUM | 중간 | ✓ |
| CASES | 중간 | ✓ |
| **EQALIGN** | 중간 | ✗ (B-003) |
| MATRIX | 낮음 | ✓ |

## 다음 단계

1. **EQALIGN 구현** — exam_math.hwp 렌더링에 직접 영향. PILE과 유사한 구조로 & 기준 열 정렬
2. **적분 변형 기호 추가** — DINT, TINT 등 symbols.rs에 유니코드 매핑 추가
3. **REL/BUILDREL 구현** — 화살표 위/아래 조건식 배치
4. **BIGG 크기 확대** — 현재 무시되는 크기 배율 적용
