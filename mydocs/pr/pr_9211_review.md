# PR #9211 검토 — fix(cursor): 머리말 영역 밖으로 넘친 머리말 표를 머리말 히트로 보고, 본문 히트는 머리말 내용을 주지 않는다

## 최종 판정

머지 보류. 최초 접수 시점의 판정이며 아래 독립 검증 결과에 따라 갱신한다.

## 접수와 검토 범위

- 작성자 및 브랜치 `semanticist21/pr/header-overflow-hit`
- 검토 head `ba76940d0568efb26e19148be49961767c4e2ae4`
- 리뷰어 `edwardkim`, 요청 시각 `2026-11-20T10:37:23Z` (UTC), GitHub 요청 목록 재확인
- 최신 devel `0a437a7eefb19bfe81fc12866887e4548ea9554d` 기준 개별 및 번호순 누적 병합 시뮬레이션 성공
- 원 head의 Build & Test 및 CI Impact Policy 성공. [CI 집계](https://github.com/edwardkim/rhwp/actions/runs/44582953322/job/135785930512)

머리말 영역 밖의 표 셀이 본문 주소로 반환되는 경로를 수정한다. 본문 수집에서 Header/Footer 하위를 제외하고 좌표가 표나 글줄 안인지를 검사한다. 영역 밖 일반 본문 클릭에서도 캐시된 페이지 트리를 복제하는 경로가 추가됐다. 캐시를 빌려 읽는 기존 함수로 대체할 수 있다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9211](https://github.com/edwardkim/rhwp/pull/9211)의 head는 `ba76940d0568efb26e19148be49961767c4e2ae4`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

머리말의 명목상 영역을 넘은 표의 마지막 칸을 클릭하면 머리말 히트가 잡히고, 본문 히트가 그 칸의 경로를 반환하지 않는다. 머지 직전에 원 head와 필수 CI를 다시 확인한다.
