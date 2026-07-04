# PR #1410 pre-review — 공식 미주 모양 모델 정규화

- PR: https://github.com/edwardkim/rhwp/pull/1410
- 제목: `task 1293: 공식 미주 모양 모델 정규화`
- 작성일: 2026-06-14
- 작성자: `jangster77`
- base: `devel`
- head: `jangster77:task_m100_1293`
- 최초 검토 head: `6cf69c87fabe5b85ee798b2edbcb6dae04dea46d`
- 검토 브랜치: `review/pr-1410`

## 1. 요약 판단

**GitHub Actions 최종 통과 후 merge 가능**으로 판단한다.

PR #1410은 #1293의 공식 미주 모양 모델 정규화 작업이다. 한컴 UI 기준의 `구분선 위`,
`구분선 아래`, `미주 사이` 의미를 `FootnoteShape` 접근자로 고정하고, HWP5/HWPX 파서와
타입셋/렌더/height cursor/sweep이 같은 의미 체계를 쓰도록 큰 폭으로 정리했다.

변경 규모가 매우 크고 `src/renderer/typeset.rs` 영향이 넓으므로 일반 소형 PR처럼 즉시 merge할 수는
없다. 다만 PR 본문과 `mydocs/report/task_m100_1293_report.md`,
`mydocs/working/task_m100_1293_stage124.md`에 로컬 전체 검증과 WASM 빌드 결과가 기록되어 있고,
검토 시점에 `git diff --check upstream/devel`와 `cargo fmt --check`를 재확인했다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | `MERGEABLE` |
| mergeStateStatus | `BLOCKED` |
| 변경량 | 155 files, +20449 / -719 |
| 작성자 | `jangster77` |
| maintainerCanModify | true |
| 관련 이슈 | #1293 |

GitHub checks, 최초 검토 시점:

| 체크 | 결과 |
|---|---|
| Canvas visual diff | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| Build & Test | pending |
| Analyze rust | pending |
| WASM Build | skipped |

이 pre-review 문서와 오늘할일 갱신, EOF blank line 정리를 PR head에 push한 뒤 GitHub Actions를
다시 확인한다. CI 통과 사실만 문서에 반영하기 위한 추가 push는 하지 않는다.

문서/오늘할일 push 후 GitHub Actions 1차 재확인에서 `Build & Test`가 실패했다. 원인은
`cargo clippy -- -D warnings`의 3개 lint 실패로, `layout.rs`의 중복 boolean 조건,
`typeset.rs`의 `bind_instead_of_map`, `let_and_return` 경고였다. 이번 보정 커밋에서
동작 의도를 유지한 채 해당 clippy 경고만 정리하고, 로컬 `cargo clippy -- -D warnings`로
재검증한다.

## 3. 변경 검토

### 3.1 FootnoteShape 공식 의미 정규화

`src/model/footnote.rs`는 기존 원본 슬롯 주석을 HWP5/HWPX 보존 관점으로 정리하고,
한컴 UI 의미를 드러내는 접근자를 추가했다.

- `separator_above_margin_hu()`: 한컴 UI `구분선 위`
- `separator_below_margin_hu()`: 한컴 UI `구분선 아래`
- `between_notes_margin_hu()`: 한컴 UI `각주/미주 사이`

HWP5 `attr` bit의 번호 모양, 배치, numbering, superscript, inline 플래그도
`apply_attr_fields_from_raw()`와 `encode_attr()`로 왕복할 수 있게 정리했다.

### 3.2 HWP5/HWPX 파서와 테스트 정합

`src/parser/body_text.rs`, `src/parser/hwpx/section.rs`는 HWP5/HWPX 입력이 같은 공식 의미 접근자로
해석되도록 보정했다. `tests/issue_1050_footnote_serialize.rs`와
`tests/issue_1139_inline_picture_duplicate.rs`는 HWPX `<hp:noteSpacing>`의
`betweenNotes`, `belowLine`, `aboveLine`이 각각 공식 `미주 사이`, `구분선 아래`, `구분선 위`로
해석되는지 확인한다.

### 3.3 타입셋/렌더 흐름 보정

`src/renderer/typeset.rs`, `src/renderer/height_cursor.rs`, `src/renderer/layout.rs`는 공식 미주 모양
profile을 기반으로 visible/no-separator, default/large `미주 사이`, 큰/기본 `구분선 아래`,
rewind/title-tail, equation/TAC tail 흐름을 분기한다.

특히 `typeset.rs` 변경량이 가장 크다. 증상별 y/gap 보정이 누적되던 흐름을 정규화 profile 중심으로
묶는 방향은 #1293 목표와 맞지만, 후속 유지보수 리스크가 있으므로 PR merge 후에도 잔여 sweep 후보는
별도 이슈/태스크로 계속 추적해야 한다.

### 3.4 Visual sweep 보강

`scripts/task1274_visual_sweep.py`는 중복 target인 `2024-09-below20above20`를 제거하고,
question title overlap 최소 픽셀, line order false positive 억제, frame tail tolerance를 보강했다.
최종 보고서 기준 full sweep은 `flagged=7/323`, clean target 12개이며, 잔여 3개 key는 공식 미주
간격식 직접 불일치가 아니라 tail/cascade 후보로 분류됐다.

## 4. 검증

PR head 문서 기록 기준:

| 명령 | 결과 |
|---|---|
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 통과, `1816 passed`, `6 ignored` |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `PATH="$HOME/.cargo/bin:$PATH" wasm-pack build --target web --out-dir pkg` | 통과 |
| `python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage122_rebase_full_sweep` | `flagged=7/323`, clean target 12개 |

검토 시점 재확인:

| 명령 | 결과 |
|---|---|
| `git diff --check upstream/devel` | 통과 |
| `cargo fmt --check` | 통과 |

검토 중 발견한 EOF blank line 3건은 이 pre-review 커밋에 함께 정리한다.

문서 push 후 CI에서 발견된 clippy 3건은 후속 보정 커밋에서 함께 정리한다.

## 5. 리스크

| 리스크 | 평가 | 비고 |
|---|---|---|
| `typeset.rs` 대형 변경으로 인한 후속 회귀 | 중간~높음 | full sweep과 회귀 테스트를 통과했지만 영향 범위가 넓음 |
| `issue_1082` sep20/20 가드 완화 | 중간 | 5px에서 40px로 완화. stage124에서 잔여 tail/cascade로 재분류한 근거 존재 |
| 잔여 visual sweep 후보 | 중간 | 3개 key, 7/323 후보는 공식 미주 간격식 직접 불일치가 아니라고 문서화됨 |
| 문서/단계 파일 대량 추가 | 낮음 | 작업 추적성은 좋지만 PR diff가 매우 커짐 |

## 6. 권고

다음 조건을 만족하면 merge 가능하다.

- 이 pre-review 문서와 오늘할일 갱신, EOF blank line 정리 커밋을 PR head에 push
- 문서 push 후 발견된 `cargo clippy -- -D warnings` 3건을 보정하고 로컬 clippy 재검증 통과
- PR diff에 `mydocs/pr/archives/pr_1410_review.md`, `mydocs/orders/20260614.md`가 포함됐는지 확인
- push 후 GitHub Actions의 `Build & Test`, CodeQL, `Canvas visual diff`가 모두 통과
- merge 후 #1293 자동 close 여부 확인. `closingIssuesReferences`가 비어 있어 수동 close가 필요할 수 있음
