# Task 370: HWP 표 계산식 구현

## 목표
HWP 표의 계산식(블럭계산식/쉬운계산식/계산식 입력) 기능을 구현한다.

## 범위

### 지원 기능
1. **셀 참조**: A1, B3 (열=A~Z, 행=1~N)
2. **범위 참조**: A1:B5
3. **와일드카드**: ?1:?3 (모든 열), A?:C? (모든 행)
4. **방향 지정자**: left, right, above, below
5. **사칙연산**: +, -, *, /
6. **괄호**: 중첩 가능
7. **시트 함수**: 22개

### 시트 함수 목록

| 카테고리 | 함수 | 설명 |
|----------|------|------|
| 집계 | SUM | 합계 |
| 집계 | AVERAGE (AVG) | 평균 |
| 집계 | PRODUCT | 곱 |
| 집계 | MIN | 최소값 |
| 집계 | MAX | 최대값 |
| 집계 | COUNT | 비공백 셀 수 |
| 삼각 | COS, SIN, TAN | 삼각함수 |
| 삼각 | ACOS, ASIN, ATAN | 역삼각함수 |
| 수학 | ABS | 절대값 |
| 수학 | EXP | e의 거듭제곱 |
| 수학 | LOG | 자연로그 |
| 수학 | LOG10 | 상용로그 |
| 수학 | SQRT | 제곱근 |
| 변환 | RADIAN | 도→라디안 |
| 판별 | SIGN | 부호 (1/0/-1) |
| 반올림 | CEILING | 올림 |
| 반올림 | FLOOR | 내림 |
| 반올림 | ROUND | 반올림 |
| 반올림 | TRUNC | 절삭 |
| 논리 | MOD | 나머지 |
| 조건 | IF | 조건 분기 |
| 변환 | INT | 정수 변환 |

### 계산식 문법
```
formula     = "=" expr | "@" expr
expr        = term (("+"|"-") term)*
term        = factor (("*"|"/") factor)*
factor      = NUMBER | cell_ref | func_call | "(" expr ")" | "-" factor
cell_ref    = COLUMN ROW          // A1, B3
range_ref   = cell_ref ":" cell_ref  // A1:B5
func_call   = FUNC_NAME "(" arg_list ")"
arg_list    = arg ("," arg)*
arg         = expr | range_ref | direction
direction   = "left" | "right" | "above" | "below"
COLUMN      = [A-Z] | "?"
ROW         = [0-9]+ | "?"
```

## 구현 단계

| 단계 | 내용 | 파일 |
|------|------|------|
| 1 | 수식 토크나이저 + 파서 (AST 생성) | `src/document_core/table_calc/parser.rs` |
| 2 | 셀 참조 해석기 (A1→row,col 변환, 방향 지정자) | `src/document_core/table_calc/resolver.rs` |
| 3 | 평가기 (AST → 숫자 결과) | `src/document_core/table_calc/evaluator.rs` |
| 4 | WASM API + hwpctl 연동 | `src/wasm_api.rs` + `rhwp-studio/src/hwpctl/` |
| 5 | 블럭 계산 (선택 영역 일괄 계산) | UI 연동 |

## 아키텍처

```
계산식 문자열 (예: "=SUM(A1:A5)+B3*2")
  │
  ▼
[토크나이저] → 토큰 스트림
  │
  ▼
[파서] → AST (추상 구문 트리)
  │
  ▼
[셀 참조 해석기] → 셀 좌표 + 값 매핑
  │
  ▼
[평가기] → 숫자 결과
  │
  ▼
결과를 셀에 기록
```

## 셀 주소 규칙
- 열: A=1열, B=2열, ... Z=26열 (최대 26열)
- 행: 1=1행, 2=2행, ... (1부터 시작)
- `?`: 와일드카드 (현재 셀의 행 또는 열)

## 테스트 계획
- 단위 테스트: 토크나이저, 파서, 평가기 각각
- 통합 테스트: 실제 표에서 계산식 실행 → 결과 검증
- hwpctl-test: UI에서 계산식 실행 테스트 케이스
