# 구현계획서 — Task M100-1305: 괄호 그룹 뒤 위첨자 orphan

## 1. 설계

### 1.1 보조: 매칭 괄호 다음이 첨자인지 스캔

```text
fn paren_then_script(&self) -> bool
  현재 토큰이 LParen 이라는 전제.
  pos+1 부터 LParen/RParen depth 추적, depth==0 되는 매칭 RParen 의 다음 토큰이
  Subscript|Superscript 이면 true.
  매칭 RParen 이 없으면 false.
```

### 1.2 보조: 괄호 그룹 파싱

```text
fn parse_paren_group(&mut self) -> EqNode   // 현재 LParen 전제
  '(' 소비; RParen 전까지 try_consume_infix_over_atop + parse_element 로 items 수집;
  ')' 소비; EqNode::Paren{ left:"(", right:")", body: Row(items).simplify() }
```

### 1.3 parse_element LParen 분기 교체

```text
TokenType::LParen =>
  if self.paren_then_script() {
      let g = self.parse_paren_group();
      self.try_parse_scripts(g)
  } else {
      self.pos += 1; EqNode::Symbol("(")   // 기존 동작
  }
RParen/LBracket/RBracket => 기존 Symbol 유지 (변경 없음)
```

`(k+1)^2` → Paren{(,),Row[k,+,1]} 에 try_parse_scripts → `Superscript{ base: Paren, sup: 2 }`.

## 2. 단계

### Stage 1 — 테스트 (red)

- 대상: `(k+1)^2` → `Superscript{base:Paren, sup:2}` (orphan 아님, base 비지 않음).
- 대상: `(k-1)^2`, `(k+1)_i` (아래첨자도).
- 회귀: `(k+1)` (첨자 없음) → 기존대로 Paren 으로 묶이지 않고 Symbol 흐름 유지 (Paren 노드 미생성).
- 회귀: `a(b)` / `(a)(b)` 등 일반 괄호 영향 없음.
- 회귀: `7^2` 정상 유지, `left ( x right )^2` (#1226) 정상 유지.

### Stage 2 — 구현

1.1~1.3 구현. Stage 1 green.

### Stage 3 — 회귀·시각 검증

- `cargo test --lib` / `--tests` / clippy / fmt.
- `rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 10` → 문18 `(k+1)^2` 지수 상승 확인(PNG).
- WASM 재빌드(studio 반영).

## 3. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/parser.rs` | `paren_then_script`, `parse_paren_group`, LParen 분기 교체, 테스트 |
| `mydocs/working/task_m100_1305_stage*.md`, `mydocs/report/task_m100_1305_report.md` | 보고 |

## 4. 완료 기준

```text
1. (k+1)^2 지수 정상 상승 (Superscript{base:Paren}).
2. 첨자 없는 괄호 회귀 없음 (Paren 미생성).
3. 테스트 추가·통과, cargo test --lib/--tests green.
4. 문18 시각 정상, WASM 갱신.
```

## 5. 승인 요청

위 구현계획으로 Stage 1 부터 진행.
