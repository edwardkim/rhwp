# PR #1072 검토 — HWPX TAC 표 line0 + 본문줄 post-text overflow 정정

- 검토일: 2026-05-22
- PR: https://github.com/edwardkim/rhwp/pull/1072
- 연결 이슈: `#1070`
- 검토자: Codex

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1072 |
| 제목 | Task #1070: HWPX TAC 표 line0 + 본문줄 post-text 표줄 제외 — 본문 하단 overflow 해소 |
| 작성자 | planet6897 |
| base ← head | `devel` ← `pr/task1070-tac-post-text-overflow` |
| head SHA | `96a7de2e675c2062462c6216ffe9281953f44601` |
| PR 상태 | OPEN / mergeable true / draft false |
| 변경 | 9 files, +416 / -0 |
| 본질 변경 | `src/renderer/typeset.rs` |
| 신규 가드 | `tests/issue_1070_tac_table_post_text_overflow.rs` |

## 2. 문제와 원인

거의 한 페이지 크기의 `treat_as_char` 표가 문단 첫 줄에 있고 뒤에 본문 줄이
있는 HWPX 문서에서, 표는 거의 fit하지만 후속 본문 텍스트가 표 높이만큼 추가
하강해 편집영역 하단을 348~472px 초과했다.

PR의 원인 분석:

- `place_table_with_text`의 `post_table_start`가 HWP5 TAC 비트
  `table.attr & 0x01`에만 의존했다.
- HWPX TAC 표는 `table.common.treat_as_char == true`이지만 `attr` bit0이
  0일 수 있다.
- 이 경우 `post_table_start = pre_table_end_line(=0)`이 되어 표줄 line0이
  후속 `PartialParagraph(0..total_lines)`에 포함된다.
- 결과적으로 표줄이 다시 post-text로 렌더되어 후속 본문 줄이 표 높이만큼
  아래로 밀린다.

## 3. 변경 내용

`src/renderer/typeset.rs`의 `post_table_start` 계산에 HWPX TAC 전용 조건이
추가되었다.

```rust
} else if table.attr & 0x01 != 0 {
    pre_table_end_line.max(1)
} else if table.common.treat_as_char && total_lines > pre_table_end_line + 1 {
    pre_table_end_line + 1
} else if is_last_table && !is_first_table {
    0
} else {
    pre_table_end_line
};
```

의미:

- HWP5 TAC(`attr & 0x01`) 기존 경로는 유지한다.
- HWPX TAC이면서 표줄 뒤에 실제 본문 줄이 있는 경우에만 표줄을 post-text에서
  제외한다.
- 단일줄 TAC 표는 조건에 걸리지 않아 기존 동작을 보존한다.

## 4. 검토 결과

### 4.1 차단 이슈

**차단 이슈 없음.**

변경은 `post_table_start` 산식 한 지점에 한정되어 있고, 기존 `tac_wrap_split`,
HWP5 TAC bit0, 다중 표 처리 순서를 건드리지 않는다. `total_lines >
pre_table_end_line + 1` 조건으로 단일줄 TAC 표를 제외하는 점도 회귀 위험을
잘 줄인 형태다.

### 4.2 코드 리스크

- 조건은 `table.common.treat_as_char`와 줄 수만 본다. 실제 "본문 텍스트" 존재
  여부는 뒤의 `should_add_post_text`에서 `!para.text.is_empty()`로 제한된다.
- PUA/공백 기반 TAC filler 케이스는 이론상 영향 가능성이 있지만, PR 보고서의
  전수 sweep에서 회귀 0으로 검증되었다.
- 신규 테스트는 SVG의 `<text y="...">` 최대값이 페이지 높이를 넘지 않는지 보는
  실증형 가드다. 대상 결함의 수백 px 하강 회귀를 잡는 데 충분하다.

## 5. 처리 상태

PR head commit을 현재 `devel` 위에 cherry-pick했다.

```text
0facb1b4 Task #1070: HWPX TAC 표 line0 + 본문줄 post-text 표줄 제외 — 본문 하단 overflow 해소 (closes #1070)
```

검증:

| 항목 | 결과 |
|------|------|
| `cargo fmt --check` | 통과 |
| `cargo test --release --lib` | 통과 — 1335 passed / 0 failed / 6 ignored |
| `cargo test --release --test issue_1070_tac_table_post_text_overflow` | 통과 — 3 passed |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 — Done in 1m 58s |

## 6. 판단

코드상 차단 이슈는 발견하지 못했다. 이후 작업지시자 시각 판정까지 통과하여
`origin/devel` push, PR #1072 close, Issue #1070 close를 완료했다. 최종 처리
내역은 `mydocs/pr/pr_1072_report.md`에 기록한다.
