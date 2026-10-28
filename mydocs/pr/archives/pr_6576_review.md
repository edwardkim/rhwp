# PR #6576 검토 기록: #6575 caption TAC box baseline

- 원 PR: [#6576](https://github.com/edwardkim/rhwp/pull/6576)
- 원 head: `9eed0d46b362489e74144dc6e4afab8484916ffe`
- 통합 후보: `review/planet6897-6572-6576-20260902`
- 기반: `upstream/devel` `ea045abff28468722084f5cde9df047a9887453e`
- provenance 보존 적용: `098eda8ff`
- 메인터너 보정: `77618ecf4` (IR baseline TSV 행의 canonical 정렬만 보정)

## 최종 판정

- 판정: 메인터너 보정 후 수용 가능
- 런타임 동작은 원 PR 변경을 유지하고, 메인터너가 현재 baseline 규약에 맞게 TSV 순서만 보정했다. 원 PR은 직접 병합하지 않고 통합 후보로 수용한다.

## 검토 근거

- `src/renderer/layout/paragraph_layout.rs`가 caption 포함 TAC box의 전체 높이를 baseline 정렬에 반영하며, `issue_6575_tac_caption_box_baseline` 회귀가 이를 고정한다.
- `samples/issue6575/156489219_satellite_pm_release.hwp`의 대상 6쪽을 한글 2018 저장본용 2020 engine PDF와 비교했다.
- 원 PR CI는 Build & Test, CodeQL, Native Skia, Render Diff, Adapter inter-diff, Proptest가 성공했고 WASM Build는 정책상 skip이었다. 현재 상태는 `MERGEABLE/CLEAN`이다.

## 통합 후보 검증

- focused 회귀: `issue_6575_tac_caption_box_baseline` 통과.
- IR field sweep은 동적 manifest 해석으로 실제 suite 005의 4개를 실행해 통과했고, 16-part overflow-cell dump를 합친 뒤 기준선 diff도 통과했다.
- 전체: `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast` 결과 `8923 passed, 46 skipped`.
- Native Skia lib 및 대상 focused 검증, `cargo fmt --all -- --check`, native/WASM clippy, workspace build/clippy, manifest check를 통과했다.
- 시각 대조: [한글 2020 PDF](../../../pdf/156489219_satellite_pm_release-2020.pdf), [review PNG](../assets/pr_6572_6576_6580_planet6897_integration_20260902/visual-6576/review/review_006.png). structural flag 0, pixel similarity 87.75108%, ink proxy 15.73908%였고 captioned TAC box의 배치 구조는 PDF와 일치했다.

## 범위와 후속

- 이 변경은 첫 captioned object의 baseline 축만 다룬다. 두 번째 18pt 잔차는 해결되었다고 주장하지 않는다.
- 통합 PR 병합 및 `devel` CI 완료 전에는 #6575을 닫거나 원 PR에 원격 변경을 하지 않는다.
