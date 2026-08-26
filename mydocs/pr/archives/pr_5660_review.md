# PR #5660 검토 기록

- 대상 PR: [#5660](https://github.com/edwardkim/rhwp/pull/5660)
- 통합 PR: [#5680](https://github.com/edwardkim/rhwp/pull/5680)
- 원 이슈: [#5637](https://github.com/edwardkim/rhwp/issues/5637)
- 검토 기준: `planet6897:fix/5637-emfplus-preview`

## 메인터너 보정

- 원 PR에 포함된 생성 Cargo test target 세 건은 제거했다.
- source-side unit test 증가 정책에 맞춰 EMF+ 회귀 여섯 건을
  `tests/cases/issue_5637_emfplus_preview.rs`로 옮겼다.
- EMF+ comment가 실제로 관측된 경우에만 paintable prefix salvage를 허용해, 기존 비-EMF+
  손상 stream 오류 계약을 유지했다. 마지막 보정은 paintable Rectangle 뒤의 손상 prefix도
  비-EMF+로는 실패함을 고정한다.

## 검증

- focused EMF+ 회귀 8건과 shard 배정 `regression_suite_007`이 통과했다.
- 통합 후보의 전체 `release-test` nextest, Native Skia, 직접 PDF, doctest, clippy,
  Docker 없는 WASM build가 통과했다.
- GitHub code candidate CI의 Build & Test, Lint, Native Skia, Canvas visual diff, CodeQL,
  archive, regular/slow shard, Proptest, adapter inter-diff가 통과했다.
- 시각 증적: `pdf/pr_5671_planet6897_visual_20260820/`에서 원 fixture SVG는 1쪽,
  765,127 bytes이며 layout anomaly signal이 없고 SVG image가 1개다.

## 결론

- 보정 후 통합 후보에 포함해 병합 가능하다.
