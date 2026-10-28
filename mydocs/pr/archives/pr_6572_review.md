# PR #6572 검토 기록: #4599 ladder band 이중 계상

- 원 PR: [#6572](https://github.com/edwardkim/rhwp/pull/6572)
- 원 head: `6e0674115f0b852c217606755143a21b2309c9de`
- 통합 후보: `review/planet6897-6572-6576-20260902`
- 기반: `upstream/devel` `ea045abff28468722084f5cde9df047a9887453e`
- provenance 보존 적용: `cdf358d75`

## 최종 판정

- 판정: 승인
- 직접 병합하지 않는다. #6572의 변경은 위 통합 후보에서 provenance-preserving cherry-pick으로만 수용한다.

## 검토 근거

- `src/renderer/layout/paragraph_layout.rs`의 ladder band 높이 계산과 `issue_4599_ladder_band_double_count` 회귀가 같은 결함 축을 검증한다.
- `samples/issue4599/36374873_night_guard_log.hwpx`의 대상 쪽을 한글 2020 기준 PDF와 대조했다.
- 원 PR CI는 Build & Test, CodeQL, Native Skia, Render Diff, Adapter inter-diff, Proptest가 성공했고 WASM Build는 정책상 skip이었다. 현재 상태는 `MERGEABLE/CLEAN`이다.

## 통합 후보 검증

- focused 회귀: `issue_4599_ladder_band_double_count` 통과.
- 전체: `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast` 결과 `8923 passed, 46 skipped`.
- Native Skia lib 및 대상 focused 검증, `cargo fmt --all -- --check`, native/WASM clippy, workspace build/clippy, manifest check를 통과했다.
- 시각 대조: [한글 2020 PDF](../../../pdf/36374873_night_guard_log-2020.pdf), [review PNG](../assets/pr_6572_6576_6580_planet6897_integration_20260902/visual-6572/review/review_001.png). 단일 숫자 파일명의 1:1 fallback 대조에서 structural flag 0, pixel similarity 93.19383%, ink proxy 7.84882%였다. 글꼴 차이에 따른 ink 차이는 있으나 표와 페이지 하단 흐름의 구조적 불일치는 관찰되지 않았다.

## 범위와 후속

- #4599 이외의 알려진 #6298/#6167 잔차는 이 변경이 해결한다고 주장하지 않는다.
- 통합 PR 병합 및 `devel` CI 완료 전에는 #4599을 닫거나 원 PR에 원격 변경을 하지 않는다.
