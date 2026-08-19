# PR #5655 검토 기록

- 대상 PR: [#5655](https://github.com/edwardkim/rhwp/pull/5655)
- 통합 PR: [#5680](https://github.com/edwardkim/rhwp/pull/5680)
- 원 이슈: [#5587](https://github.com/edwardkim/rhwp/issues/5587)
- 검토 기준: `planet6897:fix/issue-5587-nested-table-clip`

## 변경과 판단

- 부모 셀보다 넓게 저장된 중첩표는 부모 폭에서 계속 잘리고, 저장 폭이 부모 폭 이하인
  중첩표만 불필요한 확장을 피하도록 clip 범위를 보정한다.
- 기존 overwide 중첩표의 clipping 계약을 보존하므로 차단 결함은 없었다.

## 검증

- 통합 후보의 focused 회귀 1건과 전체 `release-test` nextest가 통과했다.
- GitHub code candidate CI의 Build & Test, Lint, Native Skia, Canvas visual diff, CodeQL,
  archive, regular/slow shard, Proptest, adapter inter-diff가 통과했다.
- 시각 증적: `pdf/pr_5671_planet6897_visual_20260820/`의 SVG 4쪽에서
  `overflowCellLines=0`을 확인했다. 기존 fixture의 page 2 layout signal은 기준선 특성으로
  분리했고, 수정 범위 계약은 통과했다.

## 결론

- 최신 통합 후보에 포함해 병합 가능하다.
