# Task M100 #505 Stage 1 완료 보고서

## 작업 내용

`parse_command` 의 OVER/ATOP 폐기 문제를 정정. row-collecting 파서 (`parse_cases`, `parse_pile`, `parse_eqalign`) 에 OVER/ATOP 중위 연산자 처리 추가.

### 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/parser.rs` | `try_consume_infix_over_atop()` 헬퍼 추출 + 5개 호출지점에 적용 |
| `src/renderer/equation/tokenizer.rs` | `skip_spaces` 에 `\n`, `\r` 추가 (수식 스크립트 내 줄바꿈 무시) |

### diff 요약

```diff
+ fn try_consume_infix_over_atop(&mut self, children: &mut Vec<EqNode>) -> bool {
+     // OVER/ATOP 검출 → top pop + bottom parse_element + Fraction/Atop 결합
+ }

  fn parse_expression(&mut self) -> EqNode { ... children 수집 ...
-     // OVER/ATOP 인라인 (~22 줄)
+     if self.try_consume_infix_over_atop(&mut children) { continue; }
  }

  fn parse_group(&mut self) -> EqNode { ... 동일 ... }
  fn parse_cases(&mut self) -> EqNode { ... else 분기에 helper 호출 추가 ... }
  fn parse_pile(&mut self, ...) { ... else 분기에 helper 호출 추가 ... }
  fn parse_eqalign(&mut self) { ... 활성 측(L/R) helper 호출 ... }
```

```diff
  fn skip_spaces(&mut self) {
-     while self.current() == Some(' ') || self.current() == Some('\t') {
+     while matches!(self.current(), Some(' ') | Some('\t') | Some('\n') | Some('\r')) {
          self.pos += 1;
      }
  }
```

## 검증

### probe 측정 (정정 전 / 정정 후)

| | layout (px) | HWP (px) | scale_y |
|---|---|---|---|
| pi=151 BEFORE | 40.34 | 46.87 | 1.16 |
| pi=151 AFTER | 40.34 | 46.87 | 1.16 (변동 없음, 분수 없음) |
| pi=165 BEFORE | 40.49 | 66.27 | **1.64** |
| pi=165 AFTER | **61.47** | 66.27 | **1.08** ★ |

pi=165 의 layout height +21 px (분수 추가분 반영) → scale_y 0.56 감소.

### 단위 회귀 테스트

- `test_cases_korean_no_overlap` (PR #396 회귀) — 통과
- `test_korean_text_width_not_italic` (PR #396 회귀) — 통과
- `test_cases` 등 CASES 관련 — 통과
- `test_atop` — 통과

전체 `cargo test --lib` 1104 통과 (1 ignored, 본 정정 무관).

## 비고

`parse_expression` 과 `parse_group` 의 기존 인라인 OVER/ATOP 블록도 동일 헬퍼로 통합 (DRY) — 22줄 × 2 = 44줄 중복 제거.
