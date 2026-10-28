# PR #6548 검토 - anchor-delay 1 ULP 안정화

- 검토일: 2026-09-01
- 작성자: `planet6897`
- 원 PR/source head: `578afd06265a664584ab9516af47342ce54ecc26`
- 적용 commit: `604230770`
- 통합 원장 보정: `cae16410d`
- 상태: 통합 병합 완료
- 통합 PR / merge: #6541 / `e9d2f8b258b8310fd10d465b486b9ab4d85e771e`

## 판정

저장 anchor 지연 여부를 부동소수점의 정확한 대소 비교에 맡기지 않고 1 ULP 오차에서 안정화한다.
#5941 fixture는 누락됐던 두 번째 쪽을 복원하고, 인접한 #2070 문서는 315쪽을 유지한다.

## conflict 처리와 검증

예상된 `tests/fixtures/ir_field_sweep_baseline.tsv` conflict에서는 양쪽 행을 단순 병합하지 않았다.
#6546과 #6548 fixture를 모두 포함해 1,061건을 다시 계측하고 실제 비영 발산만 `cae16410d`에 반영했다.

- #5941 focused page-count: 2쪽 복원
- #2070 page-count: 315쪽 유지
- IR sweep: 1,061 samples, 3 skipped, 564 divergence paths, total 253,076
- dump/baseline SHA-256:
  `82ad76bdd63829c65836ba3b9466f497032e12cbca30d76a48339b8874c9c795`
- 네 종류 global layout ratchet 각 16 partition 통과

따라서 ULP 경계 수정과 누적 baseline 모두 재계측 근거가 있으며 통합 candidate에 수용한다.

## Merge 후 contributor PR comment 계획

- 원 head `578afd062` → 적용 `604230770` → 통합 원장 `cae16410d` →
  통합 merge `e9d2f8b25` 계보를 남긴다.
- 대상 fixture 2쪽 복원, #2070 315쪽 유지, IR sweep 재산출 결과를 알린다.
- #5941은 73건 회귀를 추적하는 넓은 이슈이므로 이번 1 ULP 사례만으로 close하지 않고 계속 open으로 둔다.
- 계보 comment를 게시한 뒤 원 PR #6548을 중복 병합하지 않고 close한다.
