# PR #5665 검토 기록

- 대상 PR: [#5665](https://github.com/edwardkim/rhwp/pull/5665)
- 통합 PR: [#5680](https://github.com/edwardkim/rhwp/pull/5680)
- 원 이슈: [#5583](https://github.com/edwardkim/rhwp/issues/5583)
- 검토 기준: `planet6897:fix/5583-anchor-para-align`

## 변경과 판단

- 본문 수식만 있는 줄도 문단 정렬을 따르도록 하되, 수식의 저장된 location이 nonzero인
  경우에는 기존 절대 위치를 보존한다.
- 수식-only centered paragraph와 nonzero location 보존 회귀를 확인했으며 차단 결함은 없었다.

## 검증

- focused 회귀 2건과 통합 후보 전체 `release-test` nextest가 통과했다.
- GitHub code candidate CI의 Build & Test, Lint, Native Skia, Canvas visual diff, CodeQL,
  archive, regular/slow shard, Proptest, adapter inter-diff가 통과했다.

## 결론

- 최신 통합 후보에 포함해 병합 가능하다.
