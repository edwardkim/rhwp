# PR #3310 통합 검토·보정 계획

## 적용 기준

- 검토 branch: `review/planet6897-font-20260726`
- 기준 devel: `61b13fad4fc022b0c00f99dbe995dfe8a923ab45`
- contributor 원 commit: `dc2f505ae5cbc468b26e692153b63b1ea596e4d4`
- 누적 cherry-pick: `2c640c46e`

## 단계

1. #3310 원 head를 SHA 고정·fetch하고 최신 devel 기반 검토 branch에 cherry-pick한다.
2. #3318을 뒤이어 적용해 공통 `text_replay.rs`의 폴백 순서가 충돌 없이 합쳐지는지 확인한다.
3. form caption이 `bundled_typefaces`를 사용하지 않는 누락을 `678494aa0`으로 보정하고 Native Skia
   회귀 test를 추가한다. 원 contributor branch에는 push하지 않는다.
4. #3310·#3318별 archive review, 오늘할일을 같은 통합 PR diff에 넣고 최신 code head의 full CI를 확인한다.
5. 작업지시자 승인 뒤 upstream의 새 통합 head branch로 push·PR을 만든다. 원 PR은 통합 PR이 merge된 뒤에만
   close와 완료 comment를 처리한다.

## rollback

통합 PR merge 전에는 `678494aa0` 또는 해당 cherry-pick commit만 되돌려 원인별로 재검증한다. contributor
원 commit과 fork head는 rebase·amend·force-push하지 않는다.
