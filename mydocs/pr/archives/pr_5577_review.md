# PR #5577 검토 기록

- PR: <https://github.com/edwardkim/rhwp/pull/5577>
- 작성자: `planet6897`
- 관련 이슈: #5568
- base / 원 head: `devel` / `ec6e06faba0200c4569d81149f9da32da10c63f8`
- 누적 적용: `ace18930f` (`review/planet6897-20260819`)
- 공통 기록: [planet6897_20260819_integration_review.md](planet6897_20260819_integration_review.md)

## 변경 검토

묶음(container) 자식 Picture를 render tree로 만들 때 `crop`과 `original_size_hu`를 본문·각주
Picture 경로와 동일하게 전달한다. 기존 SVG crop 계산은 이 필드를 소비하므로, imgClip 띠를 쓰는
자식 그림이 원본 전체로 압착되는 문제를 고친다. `issue_5568_group_picture_crop`이 crop과 원본 크기
전달 계약을 고정한다.

## 판정

회귀는 적절하지만 renderer 출력 자체를 바꾸므로 누적 수용은 조건부다. 통합 PR에는 대표 SVG/PDF
비교 등 시각 증적과 최신 CI가 필요하며, 공통 배치 검증이 정리되기 전에는 최종 merge를 승인하지 않는다.
