# PR #6498 검토 - 빈 본문 문단 margin_left 중복

- 원 PR head: 4b0ccf0a794accdeb2e955ab897aed9b6c5b549c
- 통합 cherry-pick: 60e4a4be498eaf421c44ed9ff5beefe4c7b3c1c2
- 통합 기준: 76532b4da0e720026fb24211ad0c382884d3b970

## 판정: 승인

## 확인한 범위

빈 HWP3 본문 문단의 저장 segment geometry 선택을 좁혀 margin_left 이중 적용을 막는다.

## 검증 및 증적

issue_5677_empty_body_para_margin_applied_twice 1/1과 공통 전체 회귀를 통과했다.

재현 입력은 samples/hwp3-sample.hwp다. 시각 oracle PDF가 없어 focused render-tree 회귀를 증적으로 사용한다.

## 다음 조건

추가 차단 조건 없음.

공통 검증 세부 내용은 pr_6489_6517_planet6897_integration_evidence.md를 따른다.
