# PR #6580 재검토 기록: #6551 TAC shape 뒤 spacing_before

- 원 PR: [#6580](https://github.com/edwardkim/rhwp/pull/6580)
- 원 head: `05b2223f6d80b0b13d78f9644244a8a4e0105bb8`
- 통합 후보: `review/planet6897-6572-6576-20260902`
- 기반: `upstream/devel` `ea045abff28468722084f5cde9df047a9887453e`
- provenance 보존 적용: `e86f64bf5`
- 메인터너 보정: `b63ab2cd6`

## 최종 판정

- 판정: 메인터너 보정 후 수용 가능
- 원 PR의 런타임 수정은 `spacing_before`만 복원하는 것으로 타당하다. 다만 원 회귀의 `y >= 605px` 하한과 `line_spacing` 설명은 실제 한글 2024 기준을 충분히 고정하지 못했다.
- 메인터너 보정은 사용되지 않는 `line_spacing` 계산/설명을 제거하고, 7쪽 `1. 목 적`을 610.0..=612.0px로 고정하며, 8쪽 단 상단 `1. 개요` 169.3 +/- 1.0px 반례를 추가했다.

## #6551 closing 문구 재검토

- 원 PR 본문의 `` `#6551` 을 닫습니다. ``는 Markdown code span이라 GitHub closing keyword가 아니다. 따라서 이 문구만으로 #6551이 자동 종료되지는 않는다.
- #6551은 이 통합 후보의 PR 본문에 유효한 closing reference를 두고, 병합 SHA의 `devel` CI/CodeQL이 모두 성공한 뒤에만 후속 처리한다.

## 검토 근거와 검증

- `layout_column_item`의 TAC shape 하단 advance가 `layout_paragraph`가 만든 다음 문단의 `spacing_before`를 덮지 않도록 하는 원 수정과 회귀가 동일한 결함을 다룬다.
- `samples/issue6551/113424_evaluation_guideline.hwpx`는 한글 2024 저장본으로 확인해 2024 engine 기준 PDF를 사용했다.
- 실제 render-tree 측정값은 7쪽 `목 적` 611.4px, 8쪽 단 상단 `개요` 169.3px이며 강화한 focused 회귀가 통과했다.
- 원 PR CI는 Build & Test, CodeQL, Native Skia, Render Diff, Adapter inter-diff, Proptest가 성공했고 WASM Build는 정책상 skip이었다. 현재 상태는 `MERGEABLE/CLEAN`이다.
- 전체: `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast` 결과 `8923 passed, 46 skipped`.
- Native Skia lib 및 대상 focused 검증, `cargo fmt --all -- --check`, native/WASM clippy, workspace build/clippy, manifest check를 통과했다.
- 시각 대조: [한글 2024 PDF](../../../pdf/113424_evaluation_guideline-2024.pdf), [7쪽 PNG](../assets/pr_6572_6576_6580_planet6897_integration_20260902/visual-6580/review/review_007.png), [8쪽 PNG](../assets/pr_6572_6576_6580_planet6897_integration_20260902/visual-6580/review/review_008.png). structural flag 0, 2쪽 평균 pixel similarity 89.58624% (최저 87.38479%), ink proxy 평균 18.41037%였으며 두 제목의 세로 흐름은 PDF와 일치했다.

## 범위와 후속

- 원 PR 본문에 열거한 다른 페이지 잔차는 이 보정의 해결 범위가 아니다.
- 원 #6580을 직접 병합하지 않는다. 통합 PR 병합 및 `devel` CI 완료 전에는 #6551을 닫거나 원 PR에 원격 변경을 하지 않는다.
