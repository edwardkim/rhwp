# PR #1072 처리 보고서 — HWPX TAC 표 line0 + 본문줄 post-text overflow 정정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#1072](https://github.com/edwardkim/rhwp/pull/1072) |
| 작성자 | @planet6897 |
| 연결 이슈 | [#1070](https://github.com/edwardkim/rhwp/issues/1070) |
| 처리 결정 | cherry-pick 통합 |
| PR head | `96a7de2e675c2062462c6216ffe9281953f44601` |
| cherry-pick commit | `0facb1b4` |
| 처리 일자 | 2026-05-22 |

## 처리 절차

### Stage 0: 검토

`mydocs/pr/pr_1072_review.md`에서 코드 검토를 수행했다.

- 본질 변경은 `src/renderer/typeset.rs`의 `post_table_start` 계산 한 지점.
- HWP5 TAC bit0 경로는 그대로 유지한다.
- HWPX TAC(`table.common.treat_as_char`)이면서 표줄 뒤에 실제 본문 줄이 있는
  경우에만 표줄을 post-text에서 제외한다.
- 단일줄 TAC 표는 조건에서 제외되어 기존 동작을 보존한다.
- 신규 회귀 가드 `tests/issue_1070_tac_table_post_text_overflow.rs`가 대상
  샘플 3개에서 SVG 텍스트가 페이지 높이를 넘지 않는지 검증한다.

차단 이슈는 발견하지 못했다.

### Stage 1: cherry-pick

처리 시작 시 현재 브랜치는 `ios/devel`이었으므로 대상 브랜치 `devel`로 전환했다.
이후 PR head commit `96a7de2e`를 현재 `devel` 위에 cherry-pick했다.

```text
0facb1b4 Task #1070: HWPX TAC 표 line0 + 본문줄 post-text 표줄 제외 — 본문 하단 overflow 해소 (closes #1070)
```

결과:

```text
9 files changed, 416 insertions(+)
```

`src/renderer/typeset.rs`는 자동 병합되었고 충돌은 없었다.

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo fmt --check` | 통과 |
| `cargo test --release --lib` | 통과 — 1335 passed / 0 failed / 6 ignored |
| `cargo test --release --test issue_1070_tac_table_post_text_overflow` | 통과 — 3 passed / 0 failed |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 — Done in 1m 58s |
| 시각 판정용 WASM 재빌드 | 통과 — Done in 58.24s |
| 작업지시자 시각 판정 | 통과 |
| 작업지시자 완료보고서 승인 | 통과 |

`cargo test --release --lib` 실행 중 기존 경고 6건이 출력되었으나 테스트 실패는
없었다.

### Stage 3: push

cherry-pick commit `0facb1b4`를 `origin/devel`에 push했다.

```text
74523bd3..0facb1b4  devel -> devel
```

GitHub branch rule의 required status check expected 경고는 maintainer 권한으로
bypass되어 push가 완료되었다.

### Stage 4: GitHub 정리

PR #1072는 cherry-pick 통합 완료 댓글을 남기고 close했다.

- PR: https://github.com/edwardkim/rhwp/pull/1072
- 상태: CLOSED
- closedAt: `2026-05-22T09:05:20Z`

Issue #1070은 completed로 close했다.

- Issue: https://github.com/edwardkim/rhwp/issues/1070
- 상태: CLOSED
- closedAt: `2026-05-22T09:05:39Z`

close 댓글에는 commit `0facb1b4`, `cargo fmt --check`, `cargo test --release --lib`,
신규 회귀 테스트, WASM 빌드, 작업지시자 시각 판정 통과를 기록했다.

## 변경 요약

HWPX TAC 표가 `table.common.treat_as_char == true`이지만 HWP5 TAC bit0은 없는
경우, 기존 `post_table_start` 산식이 표줄 line0을 후속 본문 `PartialParagraph`
범위에 포함했다. 그 결과 후속 본문 텍스트가 표 높이만큼 추가 하강해 본문 하단
overflow를 만들었다.

정정 후:

- HWPX TAC 표 뒤에 실제 본문 줄이 있을 때 `post_table_start = pre_table_end_line + 1`.
- 표줄은 post-text에서 제외된다.
- HWP5 TAC bit0 경로와 단일줄 TAC 표는 기존 정책을 유지한다.

## 최종 판단

PR #1072는 cherry-pick 방식으로 `devel`에 통합했고, 자동 검증과 작업지시자
시각 판정 및 완료보고서 승인을 모두 통과했다. PR과 연결 이슈 #1070 모두 처리
완료 상태다.
