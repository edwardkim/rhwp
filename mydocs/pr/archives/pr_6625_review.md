# PR #6625 검토 - EMF world transform과 clip 적용

- 원 PR head: `ea9b530399ca5a8e9b470a71d42749f898487898`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 선행 적용: #6615, #6616, #6622
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

world transform의 네 composition mode와 intersect clip을 DC stack 및 SVG 그룹 방출에 반영한다. 퇴화 clip rect는 원문 실측 근거로 no-op 처리해 유효 도형을 숨기지 않는다.

## 검증

- `issue_6577_emf_path_to_records`의 parser, clip-path, world-transform SVG 단언과 `wmf_emf_goldens` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.
- 원문 `156627451`은 15쪽 HWPX이며 PR이 지목한 2쪽을 후보에서 직접 렌더했다.

## 시각 증적

- 후보 직접 출력: [page 002](../assets/pr_6586_6649_planet6897_integration_20260902/review_6615_6625_issue6577_page_002.png)
- 도해 아이콘은 축소된 크기로 본문 카드 안에 남아 있다. 이는 rhwp 후보 관찰이며 Hancom PDF와의 새 픽셀 동등성 주장이 아니다.
- 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 있다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 선행 EMF 스택과 함께 통합 PR에서 수용한다.
