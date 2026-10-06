# PR #6649 검토 - 문단 anchor `vertOrigin` 적용

- 원 PR head: `7723400b4f7f565c6bab8d6d56cb3cd82c7a01a1`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

문단 anchor의 세로 원점을 page와 paragraph 기준으로 구분해 적용한다. 회귀 시험은 단순 attribute 존재가 아니라 렌더 트리의 실제 y 위치를 잠근다.

## 검증

- `issue_6598_para_anchor_vert_origin` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.
- 원문 `2744465`는 Hancom Office 2020 저장 HWP5, 1쪽으로 확인했다.

## 시각 증적

- 후보 직접 출력: [page 001](../assets/pr_6586_6649_planet6897_integration_20260902/review_6649_issue6598_page_001.png)
- 표와 본문이 페이지 테두리 안에 유지되는 구조를 확인했다. 이 세션에서 새 Hancom PDF는 생성하지 못했으므로 해당 PDF와의 픽셀 동등성은 주장하지 않는다.
- 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 있다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 통합 PR에서 이 `-x` 적용분을 수용한다.
