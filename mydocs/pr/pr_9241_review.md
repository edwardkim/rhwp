# PR #9241 검토 — fix(shape): 묶음 크기를 바꾸면 한컴처럼 자식도 묶음 상자를 따라 늘고 준다

## 통합 후보의 검토 결과

그룹 도형 크기를 바꾼 HWP와 HWPX를 다시 열어 한컴 인쇄본과 대조했다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](semanticist21_20261121_review.md)에 기록한다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9241](https://github.com/edwardkim/rhwp/pull/9241)의 head는 `7ec3e1e94fe323a03180af574f739ae17da1cdd6`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

묶음 크기와 자식 도형의 변환·왕복 관계 검사는 있으나 실제 손잡이로 편집한 원본·한컴 Print·Native 및 fresh WASM 전쪽 결과가 제출되지 않았다. 직선·그림·사각형·타원이 섞인 묶음을 늘린 동일 상태의 독립 출력과 두 렌더러의 전쪽 TSV 및 자식 경계를 대조한다.

## 메인테이너 보완 검증 중간 결과

실제 크기 조정 API로 만든 묶음을 HWP·HWPX로 저장했다. 독립 Print와 Native를 파일별로 비교했다. [쪽별 비교 원장](assets/semanticist21-20261121/maintainer-comparison.tsv)

## 독립 Print와 두 렌더러 대조

제품 소스 `c6fb9159bf`에서 측정했다. 크기를 바꾼 묶음을 HWP와 HWPX로 각각 저장하고 다시 열었다. 두 파일 각각의 한컴 2020 Print와 대조한 1쪽이 Native와 fresh WASM 모두 100.00000%다. [페이지별 입력·PDF 해시와 raw 측정값](assets/semanticist21-20261121/resized-group/pages.tsv)
