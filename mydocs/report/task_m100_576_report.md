# Task #576 최종 결과 보고서 — 수식 토크나이저 keyword prefix-split 누락 정정

- **마일스톤**: v1.0.0 (M100)
- **이슈**: [#576](https://github.com/edwardkim/rhwp/issues/576)
- **브랜치**: `local/task576` (분기점: `local/devel`)
- **작성일**: 2026-05-04
- **선행 task**: 없음 (별개 결함 — equation 토크나이저)
- **상태**: **Stage 4 시각 판정 대기**

## 1. 배경

작업지시자 시각 보고 (Task #573 Stage 4 검토 중 발견): `samples/exam_science.hwp` 4 페이지 20번 응답 paragraph 의 인라인 수식이 잘못 렌더.

```
"  는? (단, 는 임의의 원소 기호이다.) [3점]"
  ctrl[0] script="{b} over {a timesm}"   ← 분수 (분모 잘못)
  ctrl[1] script="rm X simZ"             ← 잘못
```

수식 위치는 정상 (Tasks #565/#568/#573 fix 와 무관). 결함 본질은 수식 콘텐츠 — 토크나이저가 키워드 + 변수 결합 (`timesm`, `simZ`) 을 단일 식별자로 처리.

## 2. 본질 결함

`src/renderer/equation/tokenizer.rs::read_command` (L104) 의 prefix-split 키워드 list 가 폰트 스타일 모디파이어 3 개 (`bold/it/rm`) 만 처리. `times` / `sim` 등 연산자 키워드가 변수와 인접 시 단일 식별자로 토큰화.

```rust
for kw in ["bold", "it", "rm"] {       // ← 3 개 한정
    ...
}
// 그 외: 연속 alphanumeric 을 단일 토큰으로 — 결함 발현
```

## 3. 정정

### 3.1 코드 변경 — `tokenizer.rs::read_command` (+50 / -1 LOC)

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

### 3.2 신규 unit tests (6 개)

```rust
test_task576_times_lowercase_prefix_split    "a timesm" → ["a", "times", "m"]
test_task576_sim_lowercase_prefix_split      "rm X simZ" → ["rm", "X", "sim", "Z"]
test_task576_times_uppercase_prefix_split    "1TIMES10" → ["1", "TIMES", "10"]
test_task576_sim_uppercase_prefix_split      "rmA SIMC" → ["rm", "A", "SIM", "C"]
test_task576_alpha_no_split                  "alpha"/"alphabet" 분리 안 됨 (회귀 차단)
test_task576_times_followed_by_space         "a times b" 공백 구분 보존
```

### 3.3 핵심 설계 결정

| 항목 | 결정 | 근거 |
|------|------|------|
| 키워드 추가 범위 | sweep 발견 4 키워드만 | 광범위 sweep 결과 다른 키워드 결함 미발현 |
| 안 A 채택 (정적 list) | vs 안 B (longest-prefix match) / 안 C (휴리스틱) | 회귀 위험 0, 변경 면적 최소, 휴리스틱 회피 |
| 대소문자 별도 등록 | 소문자 / 대문자 4 개 | sweep 발견 패턴만 |

## 4. 검증 결과

### 4.1 자동 테스트

| 테스트 | Before | After |
|--------|--------|-------|
| `cargo test --lib` | 1125 passed | **1131 passed** (+6 신규 tokenizer tests) |
| `cargo test --test svg_snapshot` | 6/6 | **6/6** |
| `cargo clippy --release --lib` | 사전 결함 2건 | 신규 경고 0 (변경 전후 동일) |

### 4.2 핵심 정정 측정 — pi=128 page 4 20번 응답

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

### 4.3 광범위 fixture sweep — 회귀 0

| Fixture | 페이지 | 결과 |
|---------|------|------|
| **`exam_science.hwp`** | 4 | **page 3/4 의도된 정정**, page 1/2 byte-identical |
| `atop-equation-01.hwp` | 1 | byte-identical ✓ |
| `equation-lim.hwp` | 1 | byte-identical ✓ |
| `eq-01.hwp` | 1 | byte-identical ✓ |
| `exam_eng.hwp` | 8 | byte-identical ✓ |
| `exam_math.hwp` | 20 | byte-identical ✓ |
| `exam_kor.hwp` | 20 | byte-identical ✓ |
| `biz_plan.hwp` | 6 | byte-identical ✓ |

### 4.4 영향 paragraph 분포

| 페이지 | 영향 paragraph | 결함 keyword |
|------|--------------|------------|
| page 3 | pi=79 (15번 본문) "rm W simY/Z", "rmX/W simZ" 등 | sim |
| page 3 | pi=82/68 (15/13번 보기) "rmA SIMC" 등 | SIM |
| page 4 | pi=126 (20번 본문) "1TIMES10^-14" | TIMES |
| page 4 | pi=128 (20번 응답) "{b} over {a timesm}", "rm X simZ" | times, sim |

## 5. 메모리 정합

- ✓ `feedback_essential_fix_regression_risk`: 광범위 sweep 8 fixture 60+ 페이지, 신규 unit tests 6, 회귀 0.
- ✓ `feedback_rule_not_heuristic`: 명시적 키워드 list (룰), 휴리스틱 미도입.
- ✓ `feedback_pdf_not_authoritative`: Unicode 코드포인트 (× = U+00D7, ∼ = U+223C) 검증, PDF 미사용.

## 6. 산출물

```
src/renderer/equation/tokenizer.rs                   +50 / -1 LOC
mydocs/plans/task_m100_576.md                        수행 계획서
mydocs/working/task_m100_576_stage1.md               Stage 1 진단 + 영향 범위 측정
mydocs/plans/task_m100_576_impl.md                   Stage 2 구현 계획
mydocs/working/task_m100_576_stage3.md               Stage 3 구현+검증
mydocs/report/task_m100_576_report.md                본 최종 보고서
output/svg/exam_science_task576/                     시각 판정용 SVG (4 페이지)
```

## 7. 커밋 이력

```
0b09b73e Task #576 Stage 0: 수행 계획서
c29f8ee7 Task #576 Stage 1: 정밀 진단 + 광범위 영향 범위 측정 (코드 무수정)
70428175 Task #576 Stage 2: 구현 계획서 (안 A — 키워드 list 확장)
ac7e8a12 Task #576 Stage 3: tokenizer 키워드 list 확장 (times/sim/TIMES/SIM)
```

## 8. 작업지시자 검토 사항

1. **시각 판정 — 본 task 효과**:
   - `output/svg/exam_science_task576/exam_science_004.svg` — pi=128 (20번 응답) `{b} over {a×m}` 분수 + `X ∼ Z` 정상 렌더
   - `output/svg/exam_science_task576/exam_science_003.svg` — page 3 15번 본문 / 13번 보기 등 다수 paragraph 정상 렌더
2. **시각 판정 — 비회귀**:
   - `output/svg/exam_science_task576/exam_science_001.svg`, `exam_science_002.svg` — byte-identical (변경 없음)
   - 광범위 sweep 7 fixture 60+ 페이지 byte-identical
3. **이슈 #576 close + `local/devel` merge** 결정: 시각 판정 통과 시
4. **PR 분리**: PR #570 (Task #568) / PR #575 (Task #573) 와 별도 PR 생성 (메모리 `feedback_per_task_pr_branch` 정합)

## 9. 승인 요청

본 최종 보고서로 Task #576 완료 승인 요청합니다.
