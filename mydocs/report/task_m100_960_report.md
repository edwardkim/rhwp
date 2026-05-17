# Task #960 — 최종 보고서

- 이슈: [#960](https://github.com/edwardkim/rhwp/issues/960)
- 마일스톤: M100 / v1.0.0
- 브랜치: `local/task960`
- 기간: 2026-05-17 (1일)

## 1. 작업 범위

원 issue #952 의 발견된 4번째 결함 — page 2 문14 의 multi-line equation (cases formula) off-by-one line 매핑 해결.

## 2. Root cause

### 2.1 증상

`samples/3-11월_실전_통합_2022.hwp` page 2 의 문14 (pi=117) 의 cases formula 가 line 0 영역 (y=329) 에 emit (예상 line 1 ~y=347).

### 2.2 RHWP_DEBUG_PARA_TAC + TAC_LINE 추적

```
DEBUG_PARA_TAC tac_controls=[(5, 1851, 1), (14, 1851, 2), (30, 13339, 3), (34, 13455, 4)]
TAC_LINE pi=117 line_idx=1 ... run_tacs=[]   ← cases (ci=3, pos=30) 누락
```

- pi=117 text 에 FFFC (object replacement char) 없음
- `control_text_positions` (model/paragraph.rs:817-838) 의 char_offsets gap 분석:
  - utf16 gap [60, 76] = 15 → 1 control at codepoint position 30 (= `\n`)
- compose_lines: line 1 chars range = [23, 30) — **position 30 (=\n) 제외**
- → cases (pos=30) 가 line 1 chars 밖

### 2.3 paragraph_layout filter 결함

`src/renderer/layout/paragraph_layout.rs:1724-1727`:

```rust
let run_tacs: Vec<(usize, f64, usize)> = tac_offsets_px.iter()
    .filter(|(pos, _, _)| *pos >= run_char_pos
        && (*pos < run_char_end
            || (is_last_run && *pos == run_char_end)))
    ...
```

- line 1: run_char_end=30, cases pos=30 → `pos < end` false, `is_last_run` false (line 1 은 paragraph 의 last line 아님)
- → cases 가 line 1 의 run_tacs 에서 누락 → shape_layout 의 default y (=329) 에 emit

## 3. Fix

`src/renderer/layout/paragraph_layout.rs:1719-1736`:

```rust
// [Task #960] has_line_break line 의 마지막 run 도 run_char_end 위치 의 TAC
// 포함. HWP3 의 char_offsets gap 분석으로 매핑된 control 위치가 `\n` 문자
// 에 떨어지면, 그 line 의 chars range [start, end) 에서 end 가 `\n` 위치
// 이므로 누락. has_line_break line 의 마지막 run 의 end position 도 TAC
// 포함하면 line 의 정확한 위치에 inline emit.
let allow_end_tac = is_last_run
    || (comp_line.has_line_break && is_last_run_of_line(run_idx));
let run_tacs: Vec<(usize, f64, usize)> = tac_offsets_px.iter()
    .filter(|(pos, _, _)| *pos >= run_char_pos
        && (*pos < run_char_end
            || (allow_end_tac && *pos == run_char_end)))
    ...
```

## 4. 검증

### 4.1 cargo test
- `cargo test --release --lib`: **1288 passed, 0 failed, 2 ignored**

### 4.2 단위 검증 (시험지 page 2)
- cases formula y: 329 → 352 ✓ (line 1 정상 위치)
- TAC_LINE pi=117 line 1: run_tacs=[(7, 177.85, 3)] ✓
- 한컴 PDF 정합 ✓

### 4.3 회귀 검증
- LAYOUT_OVERFLOW count: 41 → 41 (변화 0)
- exam_kor/math/eng, sample10~14, 시험지 4종: 시각 회귀 0

## 5. 영향 평가

| 영역 | 영향 |
|------|------|
| has_line_break + end-position control | 정상화 (이전 누락 → 정확한 line 에 inline emit) |
| has_line_break 없는 line | 영향 없음 (조건 미진입) |
| 일반 TAC control (line 안쪽) | 영향 없음 (기존 filter 그대로) |

## 6. 추가 발견 — Pre-existing 결함 (별도 issue)

본 Stage 4 시각 검증 중 작업지시자가 발견:
- 문14 의 <보기> textbox 내부 content scramble
- pi=118 (InFrontOfText TAC 사각형 + 내부 글상자) 의 inline 수식 위치 + ㄱㄴㄷ prefix 결함
- Fix A 적용 전/후 동일 (Fix A 와 무관 pre-existing)
- → 별도 issue [#962](https://github.com/edwardkim/rhwp/issues/962) 등록

## 7. 관련 작업

- 원 issue #952 + PR #956 (Issue 1 외곽선)
- PR #958 (Issue 2 sample16 page 18)
- PR #961 (Issue 3 시험지 page 1 문9 vertical)
- 본 PR (Issue 4 cases formula off-by-one)
- 신규 issue #962 — page 2 보기 textbox 별도 task

## 8. 후속

- 원 issue #952 + #960 close (cases formula 해결)
- Issue #962 (보기 textbox) — 별도 task
- 작업지시자가 PR 머지
