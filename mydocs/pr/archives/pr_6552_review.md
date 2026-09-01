# PR #6552 검토 - 미주 page reset 사다리

- 검토일: 2026-09-01
- 작성자: `planet6897`
- 원 PR/source head: `f7aa7d4c6d5052d4598825ff0f841f7cc919cea2`
- 적용 commit: `ddb6f43f1`
- 상태: 통합 병합 완료
- 통합 PR / merge: #6541 / `e9d2f8b258b8310fd10d465b486b9ab4d85e771e`

## 판정

쪽을 넘어온 미주 꼬리가 첫 저장 진행량 되감김에서 페이지 사다리를 통째로 잃지 않도록 한다.
#6545 p23에서 수식과 뒤 문단이 겹치던 문제는 사라졌고, 저장 진행량을 사용한 문단 배치는 유지된다.

- [before/after/Hancom 2024 비교](../../report/endnote-page-reset-ladder-6545/p23_before_after_oracle.png)
- 누적 head의 p23 직접 렌더에서도 수식과 후속 텍스트 비겹침 확인
- 형제 `3-10월_교육_통합_2022.hwpx`: 18쪽 유지
- 형제 layout anomaly: off-canvas 0, overlap 0, text-overlap 0, empty-page 0

## 교차 검증

#6542 HWP5 vpos rewind, #6495 column overrun, #5886 column/off-canvas guard와 함께 focused test를
실행해 다른 페이지 사다리 계약을 훼손하지 않았음을 확인했다. 전체 nextest, Native Skia, Docker WASM도
통과했으므로 #6541 통합 candidate에 수용한다.

## Merge 후 contributor PR comment 계획

- 원 head `f7aa7d4c6` → 적용 `ddb6f43f1` → 통합 merge `e9d2f8b25` 계보를 남긴다.
- Hancom 2024 p23 저장 진행량 복원, 수식·후속 문단 비겹침과 형제 문서 무회귀를 알린다.
- 신고된 #6545의 구체적 겹침과 누적 상향 이동이 해결됐으므로 #6545는 close 후보로 기록한다.
- 계보 comment를 게시한 뒤 원 PR #6552를 중복 병합하지 않고 close한다.
