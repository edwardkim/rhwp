# PR #1719 리뷰 — #1718 대형 RowBreak 셀 over-fill under-pagination 수정

- PR: #1719 `Task #1718: 대형 RowBreak 셀 over-fill under-pagination 수정 (visible_tail grace any->all)`
- 작성자: @planet6897
- 기준: `devel`
- 원 PR head branch: `planet6897/pr/devel-1718`
- 검토 대상 커밋: `ec62608ca50cd180eeb1bff4cf705736d0835095`, `6adce3e79aee7941e94e41665a87e8c493017de1`
- 제외한 커밋: 원 PR head 의 `devel` merge commit 2건
- 규모: 10 files, +274/-2
- 관련 이슈: #1718
- 문서 작성 시점 상태: `MERGEABLE`, `Build & Test` 진행 중(원 PR 기준)
- 처리 방침: #1721과 함께 `upstream/devel` 기준 통합 cherry-pick PR 로 수용 후보
- 최종 처리: 통합 PR #1727 merge commit `150ca316ee557d6bf95928302166e037d7467b03` 으로 반영 완료, 원 PR #1719 supersede close 완료

## 변경 요약

대형 RowBreak 표의 거대 셀에서 `visible_tail_before_spacer` grace 가 본문 중간에도 적용되어
페이지당 1-5줄씩 over-fill 되는 문제를 줄인다. 첫 번째 커밋은 `.any(spacer)`를 `.all(spacer)`로
좁혔고, 두 번째 커밋은 page13 회귀를 반영해 `grace_visible_tail_before_spacer()` 판별로 보정했다.

핵심 판별은 다음과 같다.

- 오버플로 후보 뒤에 spacer 가 없으면 grace 거부
- 첫 spacer 전까지 전부 가시 텍스트 run 이면 본문 한복판으로 보고 grace 거부
- spacer 가 바로 뒤이거나 spacer 전 비가시 유닛이 끼면 구조적 꼬리줄로 보고 grace 유지

## 로컬 검증

통합 브랜치 `codex/pr1719-1721-bundle`에서 #1719 실제 변경 커밋 2건과 #1721 커밋 1건을
`upstream/devel` 위에 cherry-pick 한 뒤 검증했다.

- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 항목 삭제
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo build --profile release-test --bin rhwp`: 통과, 1분 34초
- `/Users/tsjang/rhwp/target/release-test/rhwp dump-pages samples/task1718/table_giant_cell_overfill.hwp | rg -c '^=== 페이지'`: `42`
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --lib row_cut_tests -- --nocapture`: 12 passed, 1분 19초
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --test issue_rowbreak_chart_overlap -- --nocapture`: 20 passed, 53초
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo fmt --check`: 통과
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo clippy --all-targets -- -D warnings`: 통과, 38초

## GitHub Actions

원 PR #1719의 문서 작성 시점 참고값:

- CI preflight: success
- CodeQL preflight: success
- Render Diff preflight: success
- WASM Build: skipped
- Analyze (javascript-typescript): success
- Analyze (python): success
- Analyze (rust): success
- Canvas visual diff: success
- CodeQL: success
- Build & Test: in progress

최종 merge 판단은 통합 cherry-pick PR head 기준 GitHub Actions 통과를 조건으로 한다.

## 리뷰 결과

Blocking finding 없음.

초기 `.all(spacer)` 단순 보정은 page13 회귀를 만들 수 있었지만, 후속 커밋이 `첫 spacer 전 가시 run`
판별로 케이스를 분리했다. 로컬에서 신규 단위 테스트와 기존 rowbreak 통합 테스트를 함께 통과해
거대 셀 under-pagination 완화와 구조적 꼬리줄 보존이 같이 확인됐다.

## 리스크 / 후속 확인

- 대표 샘플은 40쪽에서 42쪽으로 개선되지만 한글 기준 48쪽에는 아직 미달한다. 잔여 6쪽은 #1720의
  개체 단위 회귀 인프라와 후속 레이아웃 보정으로 다룰 영역이다.
- 원 PR #1719는 merge commit 이 섞여 있으므로 그대로 merge 하지 않고 실제 변경 커밋만 통합 PR에 반영한다.
- 통합 PR merge 후 #1718 auto-close 여부를 확인하고, 실패 시 수동 close 한다.
- 통합 PR merge 후 원 PR #1719에는 supersede/통합 반영 코멘트를 남기고 close 한다.

## 최종 판단

통합 cherry-pick PR 로 수용 가능.

## merge 후 확정 기록

- 통합 PR: #1727
- merge commit: `150ca316ee557d6bf95928302166e037d7467b03`
- 원 PR #1719 상태: supersede close 완료
- 관련 이슈 #1718 상태: 수동 close 완료
- 후속 이슈: #1728 `RowBreak 표 continuation 5~6쪽 PDF 기준 시각 차이 보정`
- 후속 샘플 보강:
  - `samples/table_giant_cell_overfill.hwpx`
  - `pdf/table_giant_cell_overfill-2024.pdf`
  - `samples/table_scattered_header_rowbreak.hwp`
  - `pdf/table_scattered_header_rowbreak-2024.pdf`

### 후속 이슈 분리 사유

#1727은 대형 RowBreak 셀의 over-fill under-pagination을 완화해 대표 샘플을 40쪽에서 42쪽으로
개선했다. 다만 `table_giant_cell_overfill.hwp`와 `table_scattered_header_rowbreak.hwp`의 5~6쪽은
PDF 기준 페이지 컷과 상하 배치가 아직 다르므로, merge를 막지 않고 #1728로 분리 추적한다.
