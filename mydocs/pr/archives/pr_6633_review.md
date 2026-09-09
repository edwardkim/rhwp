# PR #6633 검토 - 문단 끝 TAC 표의 저장 줄 배치

- 원 PR head: `37d187e99461dd8aec5a7fec6c9e20dca40e6add`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

유일한 TAC 표의 마지막 저장 줄이 표 band와 일치할 때만 y를 아래로 이동한다. 접수증 필러 문단을 제외하고, 큰 마지막 줄·유일 표·단 내부 조건을 모두 요구해 기존 TAC 갈래를 넓히지 않는다.

## 검증

- `issue_6614_para_tail_tac_table` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.
- 원문 `156658611`은 Hancom Office 2018 저장 HWPX, 1쪽으로 확인했다.

## 시각 증적

- 후보 직접 출력: [page 001](../assets/pr_6586_6649_planet6897_integration_20260902/review_6633_issue6614_page_001_font_fallback.png)
- 현재 Mac의 원문 font fallback으로 한국어 본문은 상자 글리프로 보인다. 이 자산은 표가 문단 상단에 겹치지 않고 페이지 하단에 배치되는 구조 관찰까지만 사용하며 Hancom PDF 문자 좌표 일치를 주장하지 않는다.
- 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 있다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 통합 PR에서 이 `-x` 적용분을 수용한다.
