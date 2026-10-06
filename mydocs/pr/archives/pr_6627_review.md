# PR #6627 검토 - `hp:cellzone` 네 변 테두리 덮어쓰기

- 원 PR head: `00cbddaa9f543dab959f0c68662d4b4cd31c4a15`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

zone이 지정한 non-`None` 변만 기존 셀 edge 위에 덮고, 병합 셀의 span 끝까지 endpoint를 확장한다. `None` 변으로 기존 선을 지우지 않는 보수적 계약도 시험으로 고정한다.

## 검증

- `issue_6619_cellzone_border`의 빈 셀 변, 점선 override, 병합 span endpoint 단언 3개 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.
- 원문 `156745900`은 Hancom Office 2022 저장 HWPX, 51쪽으로 확인했다.

## 시각 증적

- renderer IR 재현물 직접 출력: [synthetic border](../assets/pr_6586_6649_planet6897_integration_20260902/review_6627_synthetic_cellzone_border.png)
- 원문 2·31쪽 직접 출력: [page 002](../assets/pr_6586_6649_planet6897_integration_20260902/review_6627_source_page_002_font_fallback.png), [page 031](../assets/pr_6586_6649_planet6897_integration_20260902/review_6627_source_page_031_font_fallback.png)
- 현재 Mac에는 원문이 요구하는 `한컴바탕`/`함초롬` 계열이 없어 원문 텍스트가 fallback 상자로 보인다. 따라서 원문 이미지는 표 테두리 구조 확인까지만 사용하며, Hancom PDF 텍스트·색·굵기 좌표 일치로 주장하지 않는다.
- 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 있다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 통합 PR에서 이 `-x` 적용분을 수용한다.
