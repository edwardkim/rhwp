# PR #5673 검토 기록

- 대상 PR: [#5673](https://github.com/edwardkim/rhwp/pull/5673)
- 통합 PR: [#5680](https://github.com/edwardkim/rhwp/pull/5680)
- 원 이슈: [#5599](https://github.com/edwardkim/rhwp/issues/5599)
- 검토 기준: `planet6897:fix/5599-hancom-pua-oracle-batch`

## 변경과 판단

- 한글 2022 oracle로 확인한 한컴 PUA를 표시 대체표와 텍스트 표면에 반영한다.
- 선행 commit의 15개 매핑은 최신 `devel`에 동등하게 존재해 중복 적용하지 않았고, 최종 head의
  고유 delta만 통합했다.
- `F02C5 -> ⑫` composer 특수 경로까지 텍스트 표면에 나타나는 것을 확인했으며 차단 결함은 없었다.

## 검증

- focused 회귀 2건과 통합 후보 전체 `release-test` nextest가 통과했다.
- GitHub code candidate CI의 Build & Test, Lint, Native Skia, Canvas visual diff, CodeQL,
  archive, regular/slow shard, Proptest, adapter inter-diff가 통과했다.
- 시각 증적: `pdf/pr_5671_planet6897_visual_20260820/`의 SVG는 1쪽, 96,377 bytes,
  layout anomaly signal 없음, `□` 4개, `◇` 3개, raw `U+F03FF`/`U+F02EC` 0개다.

## 결론

- 최신 통합 후보에 포함해 병합 가능하다.
