# PR #9271 검토 — fix(char-overlap): 글자 없는 테두리 없는 포갠 겹침은 폭 없이 캐럿 한 칸만 둔다

## 2026-11-21 메인테이너 보완 착수

원 [PR #9271](https://github.com/edwardkim/rhwp/pull/9271)의 head는 `b6922ffed647ec34f0789da0dc329ef9e11cee9c`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

기여자가 빈 포갠 겹침의 폭을 Native 1쪽 100.00%로 보고했다. 같은 입력의 독립 PDF와 fresh WASM 전쪽 증거가 제출되지 않았다. 빈 겹침과 일반 겹침의 폭·캐럿을 같은 독립 출력과 비교하고 양 backend 증거를 보완한다.

## 메인테이너 보완 검증 중간 결과

빈 글자 겹침의 두 입력을 각각 HWPX로 저장하고 한컴으로 인쇄했다. 두 경우 모두 Native 1쪽은 raw 100.00%다. [쪽별 비교 원장](assets/semanticist21-20261121/maintainer-comparison.tsv)

## 독립 Print와 두 렌더러 대조

제품 소스 `c6fb9159bf`에서 측정했다. 겹침 테두리 유무를 나눈 두 저장 입력을 한컴 2020에서 각각 인쇄했다. 두 입력의 1쪽이 Native와 fresh WASM 모두 100.00000%다. [페이지별 입력·PDF 해시와 raw 측정값](assets/semanticist21-20261121/empty-overlap/pages.tsv)
