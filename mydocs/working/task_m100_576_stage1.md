# Task #576 Stage 1 진단 보고서 — 본질 결함 식별 + 영향 범위 측정

- **이슈**: [#576](https://github.com/edwardkim/rhwp/issues/576)
- **브랜치**: `local/task576`
- **단계**: Stage 1 — 정밀 진단 + 영향 범위 측정 (코드 무수정)
- **작성일**: 2026-05-04

## 1. 결론 (요약)

`tokenizer.rs::read_command` 의 prefix-split 키워드 목록이 `["bold", "it", "rm"]` 3 개로 한정 → **`times` / `sim` (대소문자 모두) 가 변수와 결합 시 단일 식별자로 토큰화**.

광범위 sweep (사용 가능한 모든 fixture, 563 unique 수식 script) 결과 **결함은 `exam_science.hwp` 에 한정** — 10 개 unique 수식 script 영향. 다른 fixture (exam_eng/exam_kor/exam_math/atop-equation/equation-lim 등 558 scripts) 는 결함 미발현.

알파 등 그리스 문자 prefix 충돌 (`"alphabet"` → `alpha` + `bet`) 은 실제 fixture 에서 미발현 → 안 A (키워드 list 확장) 비교적 안전.

## 2. tokenize 동작 trace

### 2.1 결함 입력

| 입력 | 현재 토큰 출력 | 기대 토큰 |
|------|--------------|----------|
| `"{b} over {a timesm}"` | `[..., Command(a), Command(timesm), }]` | `[..., Command(a), Command(times), Command(m), }]` |
| `"rm X simZ"` | `[Command(rm), Command(X), Command(simZ)]` | `[Command(rm), Command(X), Command(sim), Command(Z)]` |

### 2.2 정상 입력 (회귀 차단 검증용)

| 입력 | 토큰 출력 | 평가 |
|------|--------|------|
| `"alpha"` | `[Command(alpha)]` | ✓ 정상 |
| `"alphabet"` | `[Command(alphabet)]` | ✓ 일반 식별자 — 분리되지 않아야 함 (안 A/B 도입 시 보존 필요) |
| `"alphabeta"` | `[Command(alphabeta)]` | ✓ 일반 식별자 — 분리되지 않아야 함 |
| `"timesx"` | `[Command(timesx)]` | ✗ 결함 (실제 사용 시 `times` + `x`) |
| `"sin x"` | `[Command(sin), Command(x)]` | ✓ 공백 구분 정상 |

## 3. 본질 결함 — `read_command` 코드 (L101-124)

```rust
fn read_command(&mut self) -> Token {
    let start = self.pos;
    for kw in ["bold", "it", "rm"] {       // ← 3 개 한정
        if self.matches_at(kw) {
            let after = self.peek(kw.len());
            if matches!(after, Some(c) if c.is_ascii_alphanumeric()) {
                self.pos += kw.len();
                return Token::new(TokenType::Command, kw, start);
            }
        }
    }
    // 그 외: 연속 alphanumeric 을 단일 토큰으로
    let mut value = String::new();
    while let Some(ch) = self.current() {
        if ch.is_ascii_alphanumeric() { value.push(ch); self.pos += 1; }
        else { break; }
    }
    Token::new(TokenType::Command, value, start)
}
```

**결함**: `for kw in ["bold", "it", "rm"]` 가 prefix-split 대상을 폰트 스타일 모디파이어 3 개로만 제한. `times` / `sim` 등 연산자 키워드가 변수와 인접 시 단일 토큰화.

## 4. 광범위 영향 범위 측정

### 4.1 Sweep 방법

`samples/*.{hwp,hwpx}` 158 fixture 의 모든 paragraph 의 inline Equation control 의 `script` 추출 (셀 내 nested table paragraph 도 재귀 검색). 각 토큰을 lookup_symbol prefix 와 매칭하여 결함 의심 패턴 탐지.

### 4.2 Sweep 결과

- **총 unique 수식 script**: 563
- **결함 발현 의심**: **10** (모두 `exam_science.hwp`)

| # | 영향 fixture | Script | 잘못 토큰화 |
|---|------------|--------|-----------|
| 1 | exam_science.hwp | `"rmA SIMC"` | `SIM+C` (A ~ C) |
| 2 | exam_science.hwp | `"rm W simZ"` | `sim+Z` (W ~ Z) |
| 3 | exam_science.hwp | `"rm W simY"` | `sim+Y` (W ~ Y) |
| 4 | exam_science.hwp | `"rmX simZ"` | `sim+Z` (X ~ Z) |
| 5 | exam_science.hwp | `"rmW simZ"` | `sim+Z` (W ~ Z) |
| 6 | exam_science.hwp | `"1TIMES10^-14"` | `TIMES+10` (1 × 10⁻¹⁴) |
| 7 | exam_science.hwp | `"rm2 SIM3"` | `SIM+3` (2 ~ 3) |
| 8 | exam_science.hwp | `"{b} over {a timesm}"` | `times+m` (a × m) |
| 9 | exam_science.hwp | `"rm X simZ"` | `sim+Z` (X ~ Z) |
| 10 | exam_science.hwp | (중복) | |

### 4.3 영향 paragraph 식별

작업지시자 보고 + sweep 결과 결합:
- **20번 (pi=128)**: `{b} over {a timesm}` + `rm X simZ` ← 본 task 진입 트리거
- **15번 (pi=79)**: `rm W simY`, `rmX simZ`, `rmW simZ` 등 — 이전 task #573 SVG 출력에서 inline 수식 위치는 정상이나 콘텐츠 잘못 (시각 판정 필요)
- 다른 13/16/19 번 등 — 추가 paragraph 영향

### 4.4 결함이 발현되는 키워드 패턴

발견된 키워드 (대소문자 모두):
- `times` / `TIMES` (× 곱셈)
- `sim` / `SIM` (~ 유사)

발견되지 않은 키워드 (안 A 도입 시 비교적 안전):
- 그리스 문자 `alpha` / `beta` / 등 — 변수와 결합 사례 없음
- 다른 연산자 `over` / `cdot` / `oplus` / 등 — 항상 공백 구분
- 함수 `sin` / `cos` / 등 — 공백 구분

## 5. 정정 안 비교 (Stage 2 에서 정밀화)

### 안 A — 정적 키워드 list 확장

`read_command` 에 `times/sim/TIMES/SIM` 4 개 추가 (sweep 발견 패턴만):

```rust
for kw in ["bold", "it", "rm", "times", "sim", "TIMES", "SIM"] { ... }
```

**장점**: 변경 면적 최소 (1 줄 변경). 검증된 keyword 만 추가 → alpha/beta prefix 충돌 위험 0.
**단점**: 미래 다른 keyword 결합 케이스 발견 시 list 추가 필요. 유지보수 부담.

### 안 B — Longest-prefix match against `lookup_symbol`

토큰 시작 시 lookup_symbol 등록 keyword 와 longest-prefix match:

```rust
fn read_command(&mut self) -> Token {
    let start = self.pos;
    let remaining: String = self.chars[self.pos..].iter().collect();
    if let Some(kw) = find_longest_keyword_prefix(&remaining) {
        // kw 다음이 alphanumeric 이면 prefix 분리
        let after_idx = self.pos + kw.len();
        if matches!(self.chars.get(after_idx), Some(c) if c.is_ascii_alphanumeric()) {
            self.pos += kw.len();
            return Token::new(TokenType::Command, kw, start);
        }
    }
    // 기존 fallback
    ...
}
```

**장점**: lookup_symbol 단일 source-of-truth. 미래 keyword 추가 자동 대응.
**단점**: alpha/beta/pi 등 짧은 keyword 의 prefix 충돌 위험 — `"alphabet"` → `alpha` + `bet` 잘못 분리 가능.

### 안 C — 휴리스틱 (회피 권고)

메모리 `feedback_rule_not_heuristic` 정합 위반 가능 — 회피.

### Stage 1 권고: 안 A

근거:
1. 광범위 sweep 결과 결함 키워드는 `times`/`sim` 만 (대소문자 4 개)
2. 다른 키워드 (alpha/over/sqrt 등) 는 모든 fixture 에서 적절히 공백 구분
3. 안 A 는 변경 면적 최소, 회귀 위험 0 (검증된 4 키워드만 추가)
4. 안 B 의 prefix 충돌 위험 (alpha/pi/mu 등) 회피

## 6. 회귀 검증 범위 (Stage 2/3 에서 적용)

### 필수
- `cargo test --lib` 1125+ 통과
- `cargo clippy` 신규 경고 0
- `svg_snapshot` 6/6 통과
- tokenizer unit tests:
  - 신규: `"timesm"` → `[times, m]`, `"simZ"` → `[sim, Z]`, `"TIMES10"` → `[TIMES, 10]`
  - 회귀 차단: `"alpha"` → `[alpha]`, `"alphabet"` → `[alphabet]`, `"sin x"` → `[sin, x]`

### 권고
- 광범위 fixture sweep — 의도된 정정 (exam_science 10 paragraph) 외 byte-identical
- 한컴 2010/2020 PDF 비교 (보조 ref)

## 7. 산출물

- 본 보고서: `mydocs/working/task_m100_576_stage1.md`
- 임시 진단 example (사용 후 삭제): `examples/diag_task576.rs`

## 8. 승인 요청

본 진단 결과 (안 A 권고) 를 바탕으로 Stage 2 (구현 계획서 작성, 안 A 상세화) 진입을 승인 요청합니다.
