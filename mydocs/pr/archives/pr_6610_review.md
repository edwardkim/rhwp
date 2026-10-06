# PR #6610 검토 - 같은 줄 TAC 제어문자 순서 보정

- 원 PR head: `bd850e0e45ff23ad4b77dfffbb3180d3e6fa2c30`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 선행 적용: #6604 `c01a6dca4ee5a011edeb58740477d37bbabd5ecb`
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

제어문자 offset을 고정 폭 환산 대신 실제 같은 줄의 인라인 표 개수로 계산한다. #6604가 복원한 선언 폭 계산과 합쳐져 첫 두 표의 가로 배치를 완성한다.

## 검증

- `issue_6601_inline_tac_tables_share_a_line` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.

## 시각 증적

- #6604와 동일한 원문 직접 비교: [before](../assets/pr_6586_6649_planet6897_integration_20260902/review_6604_6610_issue6601_before_page_001.png), [after](../assets/pr_6586_6649_planet6897_integration_20260902/review_6604_6610_issue6601_after_page_001.png)
- 후보는 두 TAC 표를 같은 y-band에 배치하며, 기준의 첫 페이지 대형 overflow를 제거한다. 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 있다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 #6604와 함께 통합 PR에서 수용한다.
