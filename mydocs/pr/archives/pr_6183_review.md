# PR #6183 검토 기록

- 원본 PR: [#6183](https://github.com/edwardkim/rhwp/pull/6183)
- 통합 PR: [#6191](https://github.com/edwardkim/rhwp/pull/6191)
- 원본 head: `77050262`
- 관련 이슈: #6078

## 변경 검토

inline table 뒤의 line-seg 조회를 보정해 HWP3 표 caption 흐름을 유지했다.

## 검증과 증적

- 원본 current-head CI가 통과한 상태에서 반입했다.
- 통합 전체 regression: `8,417 passed`, Native Skia lib: 통과.
- 시각/실행 증적: [PR #6191 증적 인덱스](../assets/pr_6191/README.md)의 #6183 항목.

## 결론

통합 범위에서 차단 결함을 찾지 못했다. #6191의 현재 head CI 완료 후 병합 및 후속 절차 대상으로 기록한다.
