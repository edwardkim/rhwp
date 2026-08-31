# PR #6515 검토 - 되감김 없는 미주 단 overflow 관문

- 원 PR head: 29fefa0473e43093973771af07b2d15d8811fb12
- 통합 cherry-pick: a089e76abc4e2b5c3a3f6dacb72369c70b5979df
- 통합 기준: 76532b4da0e720026fb24211ad0c382884d3b970

## 판정: 메인터너 보정 됨 수용 가능

## 확인한 범위

#6492 위에서 되감김 신호가 없어도 가용 높이 초과 시 미주 simulation 관문을 보게 한다.

## 검증 및 증적

issue_6495_column_overrun_without_rewind 2/2와 공통 전체 회귀를 통과했다.

입력과 oracle 및 current-head sweep 결과는 공통 증적 문서에 보존한다.

## 다음 조건

upstream/devel과 통합 head를 비교해 overflow 16→13 변화와 9쪽 simulation-paint 잔차를 분리한다.

공통 검증 세부 내용은 pr_6489_6517_planet6897_integration_evidence.md를 따른다.
## 2026-08-31 메인터너 보정 검증

**최종 판정: 메인터너 보정 됨 수용 가능.**

- #6492와 동일한 base/current Hancom 2024 PDF 비교로 보정 효과를 확인했다. base의 12쪽 terminal tail overflow 및 13쪽 flow collapse/marker drift는 현재 후보에서 재현되지 않는다.
- 19·23쪽의 잔여 PDF 차이는 base에도 있어 이번 통합의 회귀가 아니다. 따라서 현 보정 범위를 넘어 전역 layout 동작을 변경하지 않았다.
- base/current 12쪽 이미지가 `maintainer-20260831/pr6492-base-p012-review.png`, `maintainer-20260831/pr6492-current-p012-review.png`에 있다.
