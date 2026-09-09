# PR #6604 검토 - 나란한 TAC 표의 선언 폭 반영

- 원 PR head: `c01a6dca4ee5a011edeb58740477d37bbabd5ecb`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 후속 관계: #6610이 같은 `#6601` 인라인 표 위치 계산을 추가 보정하므로 함께 적용했다.
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

인라인 표의 흐름 폭을 열 합계만으로 축소하지 않고 선언 폭과 여백을 반영한다. 그 결과는 #6610의 제어문자 위치 보정과 함께만 완전하므로, 두 PR을 분리 merge하지 않고 같은 통합 후보에서 검증했다.

## 검증

- `issue_6601_inline_tac_tables_share_a_line` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.

## 시각 증적

- 기준 `upstream/devel` 직접 출력: [before page 001](../assets/pr_6586_6649_planet6897_integration_20260902/review_6604_6610_issue6601_before_page_001.png)
- 통합 후보 직접 출력: [after page 001](../assets/pr_6586_6649_planet6897_integration_20260902/review_6604_6610_issue6601_after_page_001.png)
- 기준은 첫 두 TAC 표가 세로로 밀리고 `359.4px` overflow를 냈다. 후보는 두 표를 같은 줄에 배치했고 이 대형 overflow를 제거했다. 두 이미지는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 고정했다.
- 후보의 3쪽 `2.7px` overflow는 기준에도 동일하게 존재해 이 PR의 신규 회귀로 기록하지 않는다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 #6610과 함께 통합 PR에서 수용한다.
