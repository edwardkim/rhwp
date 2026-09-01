# PR #6552 검토 - 미주 page reset 사다리

- 검토일: 2026-09-01
- 작성자: `planet6897`
- 원 PR/source head: `f7aa7d4c6d5052d4598825ff0f841f7cc919cea2`
- 적용 commit: `ddb6f43f1`
- 상태: 통합 candidate 수용 가능

## 판정

쪽을 넘어온 미주 꼬리가 첫 저장 진행량 되감김에서 페이지 사다리를 통째로 잃지 않도록 한다.
#6545 p23에서 수식과 뒤 문단이 겹치던 문제는 사라졌고, 저장 진행량을 사용한 문단 배치는 유지된다.

- [before/after/Hancom 2024 비교](../report/endnote-page-reset-ladder-6545/p23_before_after_oracle.png)
- 누적 head의 p23 직접 렌더에서도 수식과 후속 텍스트 비겹침 확인
- 형제 `3-10월_교육_통합_2022.hwpx`: 18쪽 유지
- 형제 layout anomaly: off-canvas 0, overlap 0, text-overlap 0, empty-page 0

## 교차 검증

#6542 HWP5 vpos rewind, #6495 column overrun, #5886 column/off-canvas guard와 함께 focused test를
실행해 다른 페이지 사다리 계약을 훼손하지 않았음을 확인했다. 전체 nextest, Native Skia, Docker WASM도
통과했으므로 #6541 통합 candidate에 수용한다.
