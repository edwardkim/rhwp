# PR #6512 검토 - HWP3 제본 여백과 LineSeg 폭

- 원 PR head: 6c24d37bfbee6da7211e3e16196e48e317d84f46
- 통합 cherry-pick: f3a7785af91d0dad60dacb7dd64f354a097b7e00
- 통합 기준: 76532b4da0e720026fb24211ad0c382884d3b970

## 판정: 승인

## 확인한 범위

HWP3 parser의 synthetic segment width에 binding margin을 차감한다.

## 검증 및 증적

issue_5696_hwp3_lineseg_subtracts_binding_margin 2/2와 공통 전체 회귀를 통과했다.

입력은 samples/hwp3-sample19.hwp, 저장소 내 비교 대상은 samples/hwp3-sample19-hwp5.hwp다. 제본 여백 0 sample 무영향 회귀도 포함한다.

## 다음 조건

추가 차단 조건 없음.

공통 검증 세부 내용은 pr_6489_6517_planet6897_integration_evidence.md를 따른다.
