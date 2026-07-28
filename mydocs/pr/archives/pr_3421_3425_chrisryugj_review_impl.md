# chrisryugj PR 2건 통합 검토 구현 기록 — 2026-07-27

## 통합 범위

사용자 가시 primary worktree의 `review/chrisryugj-20260727`을 최신 `upstream/devel`
`2d7303c5bea13eaf072e782cd7f7b4a6db59b35e`에서 만들고, contributor source head의 기능 commit만
fetch·누적했다. source branch의 devel merge commit과 source branch 자체에는 손대지 않았다.

| PR | source head / 기능 commit | 통합 commit | 판정 |
| --- | --- | --- | --- |
| #3421 | `539920af` / `23d94ac6` | `e656e0381` | 메인터너 보정 후 수용 가능 |
| #3425 | `3984f2a8` / `683c14e0` | `597dabf07` | 수용 가능 |
| maintainer | 공용 HWPX event writer XML 1.0 보완 | `ad7ce8ca6` | #3421의 미포함 저장 경로 보정 |

두 source PR은 작성 시점에 `MERGEABLE` / `CLEAN`이고 CI, CodeQL, Render Diff, Native Skia 및
default-feature 8 shard가 모두 성공했다. maintainer hold·review comment는 없었다.

## 보정 사유

#3421의 XML 1.0 filter는 수동 string serializer를 보호하지만, HWPX의 `quick-xml` event writer는
invalid scalar를 직접 검증하지 않아 text·attribute 경로가 남는다. `ad7ce8ca6`은 공용 helper 하나로
`text()`, `start_tag_attrs()`, `empty_tag()`와 field 수동 attribute를 함께 보정하고 실제 writer
출력 회귀 시험을 추가했다. #3425는 control 축과 `ctrl_data_records` 축의 index 정합만 다루므로
이 보정과 독립적이며 충돌하지 않는다.

## 검증 묶음

전용 target `CARGO_TARGET_DIR=target/review-chrisryugj-20260727`, `CARGO_INCREMENTAL=0`에서 Cargo
작업을 순차 실행했다.

- #3214 focused 2건과 #3382 filter·event writer focused tests 성공.
- Rust `cargo test --profile release-test --tests`: **2,962 passed / 0 failed**, IR field sweep 포함.
- Native Skia 공식 3종: **57/0**, **2/0**, **4/0**.
- `cargo fmt --all -- --check`, `git diff --check`, `cargo clippy -- -D warnings`,
  `cargo check --target wasm32-unknown-unknown --lib`: 성공.

#3382 이슈에는 실제 재현 HWPX·한글 기준 PDF가 첨부되지 않았고 변경 자체도 정상 문서의 화면 배치를
바꾸지 않는 XML 유효성 계약이다. 따라서 임의의 PNG visual sweep은 수행하지 않았고, 이 한계와
공용 writer 실출력 회귀 검증을 #3421 개별 기록에 남겼다. 새 fixture가 없으므로 IR baseline TSV 변경도
없다.

## 통합 PR과 후속 조건

두 PR과 `ad7ce8ca6`을 하나의 통합 PR 후보로 유지한다. PR 생성, remote push, source PR comment/close/
merge는 작업지시자의 별도 승인 뒤에만 수행한다. 그 뒤 통합 head의 CI와 mergeable을 다시 확인하고,
실제 merge 뒤에만 원 PR·관련 issue 상태와 감사 comment를 처리한다.
