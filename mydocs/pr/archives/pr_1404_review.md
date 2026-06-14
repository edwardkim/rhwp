# PR #1404 검토 — 표 생성 직후 F5 포커스 복원

- PR: https://github.com/edwardkim/rhwp/pull/1404
- 제목: `fix(studio): 표 생성 직후 F5가 브라우저 새로고침으로 빠지는 포커스 누락 (#1140 케이스 1)`
- 작성일: 2026-06-14
- 작성자: `oksure`
- 관련 이슈: #1140
- base: `devel`
- head: `oksure:contrib/fix-1140-table-dialog-focus` (`8dd2b83349573c6835502b68bb964bdf9b49e9c7`)
- 검토 브랜치: `review/pr-1404`

## 1. 요약 판단

**수용 가능**으로 판단한다.

PR은 표 만들기와 셀 나누기 dialog의 적용 직후 편집 textarea로 포커스를 되돌려, 직후 F5 입력이
브라우저 기본 새로고침으로 빠지는 문제를 보완한다. `rhwp-studio`의 keyboard handler는 textarea에
바인딩되어 있고, `insert.ts`와 `page.ts`의 다른 dialog들은 이미 `(ih as any).textarea?.focus()` 패턴을
사용하고 있으므로 이번 변경은 누락 지점을 기존 관례에 맞추는 좁은 수정이다.

#1140의 두 번째 케이스인 F5 셀 블록 선택 후 `Ctrl+Left` 축소 시 오른쪽 셀 폭이 늘어나는 문제는
wasm `resizeTableCells` 쪽 폭 분배 검증이 필요한 별도 범위로 남긴다. PR 본문도 `Related #1140`로
부분 해결임을 명시하므로 merge 후 이슈를 닫지 않는다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | `MERGEABLE` |
| mergeStateStatus | `BLOCKED` |
| 변경량 | 1 file, +5 / -0 |
| 작성자 | `oksure` |
| 관련 이슈 | `Related #1140` — 케이스 1만 해결, close하지 않음 |

GitHub checks 확인 시점:

| 체크 | 결과 |
|---|---|
| Build & Test | in progress |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| Canvas visual diff | pass |
| WASM Build | skipped |

이 검토 문서와 오늘할일 커밋을 PR head에 push한 뒤 GitHub Actions를 다시 확인한다. CI 통과 사실만
문서에 반영하기 위한 추가 push는 하지 않는다.

## 3. 변경 검토

`rhwp-studio/src/command/commands/table.ts`:

- `registerTableCreateCommand()`의 `createTableDialog()` 적용 직후 `(ih2 as any).textarea?.focus();`를
  호출한다.
- `registerCellSplitCommand()`의 `createCellSplitDialog()` 적용 직후 같은 방식으로 textarea focus를
  복원한다.
- 두 지점 모두 overlay 제거 후 focus가 body로 떨어질 수 있는 경로라, F5가 editor keydown handler에
  도달하도록 보정하는 목적이 분명하다.

## 4. 로컬 검증

검토 브랜치: `review/pr-1404`

| 명령 | 결과 |
|---|---|
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cd rhwp-studio && npm test` | 통과, 70 passed |
| `cd rhwp-studio && npx tsc --noEmit` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo build --release` | 통과, 3m 01s |
| `cargo test --release --lib` | 통과, 1752 passed / 0 failed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과, 1m 07s |

`wasm-pack` 산출물 `pkg/`는 ignored 검증 산출물이므로 커밋 대상에 포함하지 않는다.

## 5. 리스크

| 리스크 | 평가 | 비고 |
|---|---|---|
| 실제 브라우저 focus timing | 낮음 | 기존 dialog focus 복원과 같은 패턴 |
| 단위 테스트 부재 | 낮음 | DOM focus와 F5 기본 동작 결합이라 unit test 신뢰도가 낮음 |
| #1140 전체 해결 오해 | 중간 | PR 본문과 검토 문서에 케이스 1 한정 및 이슈 open 유지 명시 |

## 6. 권고

로컬 검증 기준으로는 merge 준비 가능하다.

머지 전 마지막 확인:

- PR #1404 head에 이 검토 문서와 오늘할일 갱신 커밋을 push
- PR diff에 `mydocs/pr/archives/pr_1404_review.md`와 `mydocs/orders/20260614.md`가 포함됐는지 확인
- 문서 커밋 push 후 GitHub Actions 전체 통과 확인
- #1140은 부분 해결이므로 merge 후 자동 close하지 않는다
