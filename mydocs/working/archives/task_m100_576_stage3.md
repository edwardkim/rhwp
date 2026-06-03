# Task #576 Stage 3 보고서 — 구현 + 검증

- **이슈**: [#576](https://github.com/edwardkim/rhwp/issues/576)
- **브랜치**: `local/task576`
- **단계**: Stage 3 (구현 + 검증)
- **선행 산출**: Stage 1 (`task_m100_576_stage1.md`), Stage 2 (`task_m100_576_impl.md`)
- **작성일**: 2026-05-04

## 1. 변경 요약

`src/renderer/equation/tokenizer.rs` 변경:

### 1.1 read_command (L104) — 키워드 list 확장 + 주석 보강

```rust
/// [Task #576] times/sim 연산자 키워드도 변수와 인접 시 분리.
/// HWP 수식 script 에서 "{a timesm}" → "a × m", "rm X simZ" → "X ~ Z" 의미.
/// 광범위 sweep (158 fixture / 563 unique scripts) 결과 결함 발현 키워드는
/// times/sim 만 (대소문자 4 개). alpha/over/sqrt 등은 항상 공백 구분되어
/// prefix-split 불필요 — 그리스 문자 prefix 충돌 회귀 위험 0.
fn read_command(&mut self) -> Token {
    let start = self.pos;

    for kw in ["bold", "it", "rm", "times", "sim", "TIMES", "SIM"] {
        ...
    }
    ...
}
```

### 1.2 신규 unit tests (6 개)

```rust
test_task576_times_lowercase_prefix_split    "a timesm" → ["a", "times", "m"]
test_task576_sim_lowercase_prefix_split      "rm X simZ" → ["rm", "X", "sim", "Z"]
test_task576_times_uppercase_prefix_split    "1TIMES10" → ["1", "TIMES", "10"]
test_task576_sim_uppercase_prefix_split      "rmA SIMC" → ["rm", "A", "SIM", "C"]
test_task576_alpha_no_split                  "alpha"/"alphabet" 분리 안 됨 (회귀 차단)
test_task576_times_followed_by_space         "a times b" 공백 구분 보존
```

### 1.3 변경 LOC

`src/renderer/equation/tokenizer.rs`: **+50 / -1** (코드 +1, 주석 +5, 신규 tests +44)

## 2. 검증 결과

### 2.1 자동 테스트

| 테스트 | Before | After |
|--------|--------|-------|
| `cargo test --lib` | 1125 | **1131 passed** (+6 tokenizer tests) |
| `cargo test --test svg_snapshot` | 6/6 | **6/6 passed** |
| `cargo clippy --release --lib` | 사전 결함 2건 | 동일 (신규 경고 0) |

### 2.2 광범위 fixture sweep (8 fixture / 60+ pages)

| Fixture | 페이지 | 변경 |
|---------|------|------|
| **`exam_science.hwp`** | 4 | **page 3, page 4 변경** (의도된 정정) |
| `atop-equation-01.hwp` | 1 | byte-identical ✓ |
| `equation-lim.hwp` | 1 | byte-identical ✓ |
| `eq-01.hwp` | 1 | byte-identical ✓ |
| `exam_eng.hwp` | 8 | byte-identical ✓ |
| `exam_math.hwp` | 20 | byte-identical ✓ |
| `exam_kor.hwp` | 20 | byte-identical ✓ |
| `biz_plan.hwp` | 6 | byte-identical ✓ |

**회귀 0** — `exam_science.hwp` 외 모든 fixture byte-identical. exam_science.hwp 도 page 1/2 byte-identical (해당 결함 키워드 없음).

### 2.3 핵심 정정 측정 — pi=128 page 4 20번 응답

#### ctrl[0] "{b} over {a timesm}" 분수 분모

| 항목 | Before | After |
|------|--------|-------|
| 분모 토큰화 | `[a, timesm]` | `[a, times, m]` |
| SVG 분모 렌더 | `<text>a</text> <text italic>timesm</text>` | `<text>a</text> <text>×</text> <text italic>m</text>` |
| 의미 | "a timesm" italic 식별자 | **"a × m"** (a 곱하기 m) ✓ |

#### ctrl[1] "rm X simZ"

| 항목 | Before | After |
|------|--------|-------|
| 토큰화 | `[rm, X, simZ]` | `[rm, X, sim, Z]` |
| SVG 렌더 | `<text>X</text> <text italic>simZ</text>` | `<text>X</text> <text>∼</text> <text italic>Z</text>` |
| 의미 | "X simZ" italic 식별자 | **"X ∼ Z"** (X tilde Z) ✓ |

### 2.4 영향 paragraph 검증 (sweep)

| 페이지 | 영향 paragraph | 결함 keyword |
|------|--------------|------------|
| page 3 | pi=79 (15번 본문) "rm W simY/Z", "rmX/W simZ" 등 | sim |
| page 3 | pi=82/68 (15번/13번 보기) "rmA SIMC" 등 | SIM |
| page 4 | pi=126 (20번 본문) "1TIMES10^-14" | TIMES |
| page 4 | pi=128 (20번 응답) "{b} over {a timesm}", "rm X simZ" | times, sim |

## 3. 변경 파일

```
src/renderer/equation/tokenizer.rs                +50 / -1 LOC (코드 +1, 주석 +5, 신규 tests +44)
```

## 4. 산출물

- `mydocs/working/task_m100_576_stage3.md` — 본 보고서
- 변경 전 baseline: `/tmp/task576/sweep_before/`
- 변경 후 산출: `/tmp/task576/sweep_after/`

## 5. 메모리 정합 검토

- ✓ `feedback_essential_fix_regression_risk`: 광범위 sweep 8 fixture 60+ 페이지 / exam_science 외 byte-identical / 신규 unit tests 6 통과 / 기존 tokenizer tests 20 통과.
- ✓ `feedback_rule_not_heuristic`: 명시적 키워드 list (룰), 휴리스틱 미도입.
- ✓ `feedback_pdf_not_authoritative`: SVG 좌표 + Unicode 코드포인트 (× = U+00D7, ∼ = U+223C) 검증. PDF 미사용.

## 6. Stage 4 권고

작업지시자 시각 판정:
1. **의도된 정정** 검증 — exam_science page 4 20번 응답 (`{b} over {a timesm}` → "b/(a×m)" 분수, `rm X simZ` → "X ∼ Z")
2. **인접 효과** 검증 — exam_science page 3 15번 본문 (`rm W simY/Z`, `rmX simZ` 등) 정상 표시
3. PR 분리 (planet6897:pr-task576 → upstream:devel)

## 7. 승인 요청

본 Stage 3 검증 결과를 바탕으로 Stage 4 (시각 판정 + 최종 보고 + PR 분리) 진입을 승인 요청합니다.
