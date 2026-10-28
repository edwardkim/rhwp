# PR #6160 검토 기록

- 원본 PR: [#6160](https://github.com/edwardkim/rhwp/pull/6160)
- 통합 PR: [#6191](https://github.com/edwardkim/rhwp/pull/6191)
- 원본 head: `c668743f`
- 관련 이슈: #6110

## 변경 검토

빈 셀 문단 뒤의 float anchor 배치를 table layout에서 보정하고 1쪽 실물 회귀를 추가했다.

## 검증과 증적

- 원본 current-head CI가 통과한 상태에서 반입했다.
- 통합 전체 regression: `8,417 passed`, Native Skia lib: 통과.
- 시각/실행 증적: [PR #6191 증적 인덱스](../assets/pr_6191/README.md)의 #6160 항목.

## 결론

통합 범위에서 차단 결함을 찾지 못했다. #6191의 현재 head CI 완료 후 병합 및 후속 절차 대상으로 기록한다.
