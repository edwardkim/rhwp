# PR #5850 검토 기록

## 판정

**수용**. 원 PR 최신 head `0251c0a218dfc1d7701a1a735ae885787b7b0e32`를 최신
`upstream/devel` (`1b8c39b6c32baf00574564016bd9d9f7d1d88198`) 위 통합 후보에 반영했다.
통합 후보 #5853의 GitHub CI와 CodeQL이 모두 성공했다. trailing docs-only commit의 fast-pass
확인 뒤 admin merge를 실행한다.

## 변경과 검토

- HWPX 재저장 전에 보존한 원본 `lineseg vertpos`를 serializer가 다시 방출하게 한다.
- reflow 합성 bit31 line segment만 있는 문단은 원본처럼 `linesegarray` 방출을 생략한다.
- `source_line_seg_vertical_pos`는 현재 IR line segment 개수와 같을 때만 적용하고 axis prefix로
  절단하므로, 불완전 snapshot이나 다른 축의 좌표를 쓰지 않는다.
- #5723은 cell 첫 문단이고 이전 저장 줄이 없으며 gap이 중첩 표 높이 규모인 경우에만 이미 전진한
  `para_top`을 다시 더하지 않도록 좁혔다. 이 조건으로 일반 중간 문단의 누적 좌표는 종전 경로를
  유지한다.
- #5729는 저장 line band가 `om_top + 선언높이 + om_bottom`과 허용 오차 8 HWPUNIT 이내로 일치할 때만
  TAC 표 상단에 `om_top`을 반영한다. 증거가 없으면 기존 baseline-하단 정렬식을 그대로 사용한다.
- #5847은 serializer 변경이라 시각 sweep이 merge gate가 아니며 x2x HWPX XML 계약으로 검증했다.
  #5723·#5729은 renderer fixture SVG 계약으로 검증했다.
- 별도 메인터너 보정은 필요하지 않았다. 최신 원 PR head와 통합 반영 SHA를 재확인했다.

## 로컬 검증

최신 통합 후보 `05e3789ebb222ddb7b4653462ff85d6678de6006`에서 실행했다.

- `cargo test --locked --profile release-test --target-dir target/pr-review --test issue_5847_x2x_lineseg_vertpos_preserved`
  - 1 passed, 130 skipped
  - 원본과 export HWPX의 `vertpos` 배열 및 `linesegarray` 개수를 대조하고 bit31 유출이 없음을 확인
- `cargo test --locked --profile release-test --target-dir target/pr-review --test regression_suite_025 issue_5723_square_pair_stays_level -- --nocapture`
  - 1 passed, 129 filtered out
- `cargo test --locked --profile release-test --target-dir target/pr-review --test regression_suite_027 issue_5729_stacked_tac_tables_keep_outer_margin_top -- --nocapture`
  - 1 passed, 127 filtered out
  - 네 개가 쌓인 TAC 표에서 154.5..=158.5px의 잘못된 상단선이 없고 158.5..=161.0px의 기대 상단선을 확인
- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`
- `scripts/wasm-pack-locked.sh --target web --out-dir pkg`
- `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib`
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  - 8121 passed, 3 slow, 39 skipped (217.679s)

관련 이슈: #5847, #5723, #5729. 통합 PR CI 성공과 실제 merge SHA 확인 뒤 issue/원 PR comment 및 close를 처리한다.

## 통합 CI 결과

통합 PR #5853 code head `92fc6e87f1fe146004b971d7b5bef16a3eb2f7a8`에서 Build & Test,
Lint, Native Skia, Canvas visual diff, 모든 test archive shard, CodeQL(Rust 14m09s)을 포함한
필수 검사가 성공했다. WASM Build와 Frontend unit gates는 이 변경 범위에서 skip으로 판정됐다.
