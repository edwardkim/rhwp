# PR #3318 통합 검토·보정 계획

## 적용 기준

- 검토 branch: `review/planet6897-font-20260726`
- 기준 devel: `61b13fad4fc022b0c00f99dbe995dfe8a923ab45`
- contributor 원 commit: `2e60b0645b99d748f2abe858da81cbf20ec4dd1b`
- 누적 cherry-pick: `7da56d169`
- 통합 보정: `678494aa0`

## 단계

1. #3310을 먼저 누적해 Skia custom·system·bundle typeface 우선순위를 확정한다.
2. #3318을 적용해 Native Skia의 family list에 base family가 들어가는지 확인한다.
3. Canvas 2D의 네 font-family 조립점을 공용 helper로 보정하고 native release-test에서 회귀를 고정한다.
4. 시각 fixture/PDF 부재를 review에 기록하고, 원본이 제공될 때의 PDF/PNG 대조를 후속 조건으로 남긴다.
5. #3310과 하나의 integration PR로 검증·제출한다. 원 contributor PR에는 보정 commit을 push하지 않는다.

## rollback

통합 PR merge 전 `678494aa0`을 되돌리면 원 #3318의 SVG·HTML·Skia 변경만 남는다. 다만 Canvas 경로가
동일 계약에서 이탈하므로 이 상태는 merge 후보가 아니다.
