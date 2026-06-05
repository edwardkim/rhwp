# PR #1266 검토 - HWPX 문단 borderFill NONE side 렌더링 정정

- PR: https://github.com/edwardkim/rhwp/pull/1266
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1205
- 작성일: 2026-06-03
- PR 작성자: @postmelee
- base: `devel`
- head: `issue-1205-para-border-none-side` / `7f368080be10ac699acd0a20f1e2d1a85942c5b0`
- 규모: 14 files, +1095 / -77
- GitHub mergeable: `CONFLICTING`
- PR 댓글: 없음

## 1. PR 요약

PR #1266은 HWPX 문단 `borderFill`에서 side별 `type="NONE"`이 렌더링 단계에서 무시되는 문제를 보정한다.

문제 조합은 다음과 같다.

| side | HWPX border type | 기대 렌더링 |
|---|---|---|
| left | `NONE` | 그리지 않음 |
| right | `NONE` | 그리지 않음 |
| top | `SOLID` | 가로선 렌더링 |
| bottom | `SOLID` | 가로선 렌더링 |

기존 렌더러는 문단 border를 top border 대표 stroke처럼 취급해 4면 `RectangleNode` stroke를 만들거나,
partial/cross-column 경로에서 좌우 수직선을 side visibility와 무관하게 만들 수 있었다.

## 2. 주요 변경 범위

| file | 변경 |
|---|---|
| `src/renderer/layout.rs` | 문단 border side 가시성 helper 추가, 4면 동일 visible stroke일 때만 rect stroke 경로 사용, 그 외 visible side별 `LineNode` 생성 |
| `src/renderer/layout/integration_tests.rs` | left/right `NONE` + top/bottom `SOLID` 회귀 테스트, rect stroke 최적화 조건 테스트 추가 |
| `tests/golden_svg/issue-*/*.svg` | `fill="none"` + no-stroke 문단 박스 제거에 따른 snapshot 갱신 |
| `mydocs/plans/task_m100_1205*.md` | 수행/구현 계획 추가 |
| `mydocs/working/task_m100_1205_stage*.md` | 단계 보고 추가 |
| `mydocs/report/task_m100_1205_report.md` | 결과 보고 추가 |
| `mydocs/orders/20260603.md` | 작업 현황 갱신 |

## 3. 검토 결과

수정 방향은 타당하다.

- HWPX parser는 이미 side별 border를 보존한다.
- 문제는 renderer가 side별 가시성을 충분히 반영하지 않는 경로에 있다.
- 4면이 모두 visible이고 동일 stroke인 경우에만 기존 `RectangleNode` stroke 최적화를 유지하는 방식은 회귀 위험을 줄인다.
- side가 `NONE`이거나 stroke가 서로 다르면 fill-only rect + visible side별 line으로 분해하는 방향은 한컴 출력 모델과 더 가깝다.
- 기존 partial/cross-column 문단 border 회귀 테스트를 함께 보강한 점도 좋다.

주의점:

- GitHub가 PR을 `CONFLICTING`으로 표시한다.
- 실제 충돌은 `mydocs/orders/20260603.md`에서 확인된다.
- PR branch가 현재 `devel`보다 오래된 base에서 갈라져 있어, `devel..PR` 전체 diff에는 최근 `devel` 변경이 사라지는 것처럼 보이는 항목이 있다. 실제 PR commit diff는 `hcursor.prev_item_content_bottom_y` 등 최근 페이지네이션 변경을 건드리지 않는다.
- `mydocs/plans`, `mydocs/report`, `mydocs/working` 루트 문서는 현재 archive 정책에 맞춰 수용 시 `archives/`로 이동해야 한다.

## 4. GitHub Actions

PR head 기준:

| workflow/check | 결과 |
|---|---|
| CI / Build & Test | pass |
| Render Diff / Canvas visual diff | pass |
| CodeQL / Analyze rust | pass |
| CodeQL / Analyze javascript-typescript | pass |
| CodeQL / Analyze python | pass |
| WASM Build | skipped |

## 5. 로컬 사전 확인

수행한 확인:

```text
git fetch origin pull/1266/head:local/pr1266-current
git diff devel..local/pr1266-current -- src/renderer/layout.rs
git diff local/pr1266-current^..local/pr1266-current -- src/renderer/layout.rs
git merge-tree devel local/pr1266-current
```

확인 결과:

| 항목 | 결과 |
|---|---|
| PR head fetch | 성공 |
| 실제 commit diff | 문단 border 렌더링 영역만 수정 |
| GitHub merge conflict | `mydocs/orders/20260603.md` |
| 소스 충돌 예상 | 없음 |

## 6. 권장 처리

권장: **PR 기능은 수용하되, 최신 `devel` 기준 메인테이너 통합 브랜치에서 cherry-pick 후 문서 위치와 orders 충돌을 정리한다.**

진행 순서:

1. `devel` 기준 통합 브랜치 생성
2. PR #1266 commit `7f368080` cherry-pick
3. `mydocs/orders/20260603.md` 충돌은 현재 `devel` 기록을 보존하고 #1205 항목만 필요한 방식으로 반영
4. PR 추가 문서는 다음 위치로 이동
   - `mydocs/plans/archives/task_m100_1205*.md`
   - `mydocs/working/archives/task_m100_1205_stage*.md`
   - `mydocs/report/archives/task_m100_1205_report.md`
5. 자동 검증
   - `cargo fmt --all --check`
   - `cargo test --lib task_1205 -- --nocapture`
   - `cargo test --lib test_469_partial_start_box_does_not_cross_col_top -- --nocapture`
   - `cargo test --lib test_471_cross_column_box_no_bottom_line_in_col0 -- --nocapture`
   - `cargo test --tests`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo check --target wasm32-unknown-unknown --lib`
6. 필요 시 `wasm` 빌드 후 rhwp-studio 시각 판정
7. `devel` 병합/push 후 CI 확인
8. PR #1266 및 이슈 #1205 종료 처리

## 7. PR 코멘트 초안

```markdown
검토했습니다. HWPX parser가 이미 side별 border를 보존하고 있으므로, renderer에서 `BorderLineType::None` side를 실제 비가시 side로 처리하는 이번 접근은 이슈 #1205의 원인과 잘 맞습니다.

4면이 모두 visible이고 동일 stroke일 때만 기존 `RectangleNode` stroke 경로를 유지하고, 그 외에는 visible side별 line으로 분해하는 방향도 기존 4면 border 회귀 위험을 낮추는 방식이라 수용 가능하다고 판단했습니다.

현재 PR은 최신 `devel` 기준으로 `mydocs/orders/20260603.md` 충돌이 있어, 메인테이너 통합 과정에서 문서 archive 위치 정리와 함께 반영하겠습니다.
```
