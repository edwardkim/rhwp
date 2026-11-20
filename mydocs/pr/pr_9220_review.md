# PR #9220 검토 — fix(caption): 캡션 크기·위치를 바꾸면 다시 줄 나누고, 셀 아래 캡션·머리말 그림 캡션·캡션 속 그림을 한/글처럼 다룬다

## 최종 판정

**머지 보류.** 최신 devel과 두 파일에서 충돌하고, 원 head의 Lint가 실패했다. 편집 후 캡션 배치의 fresh WASM 전쪽 비교 증거도 필요하다.

## 접수와 검토 범위

- 작성자 및 브랜치 `semanticist21/pr/caption-width-margin`
- 검토 head `01811f5acd316df971cdde73b965b0a82873025a`
- 리뷰어 `edwardkim`, 요청 시각 `2026-11-20T10:37:28Z` (UTC), GitHub 요청 목록 재확인
- 최신 devel `0a437a7eefb19bfe81fc12866887e4548ea9554d` 기준 개별 및 번호순 누적 병합 시뮬레이션 성공
- 원 head의 Build & Test 및 CI Impact Policy 실패. [CI 집계](https://github.com/edwardkim/rhwp/actions/runs/44651551319/job/136019962973)

GitHub Lint에서 source-side test 수가 4,094개에서 4,102개로 증가하고 새 cfg(test) 모듈이 추가된 점을 검출했다. 캡션 검사 8개의 동작을 보존하면서 tests/cases의 자동 수집 경로로 이관해야 한다. 작성자는 Native 편람 검증과 4개 문서 비교를 보고했으나 fresh WASM 비교는 미실행이라고 명시했다. 영향을 받는 원본과 독립 Print로 Native 및 fresh WASM 전쪽 비교가 필요하다.

## 메인테이너 실행 결과

누적 적용 3번째, 적용 직후 `4d73ae5f46`. 충돌 없이 적용됐다. 처음 10건을 합친 `c6407bf83c`에서 이 PR의 집중 검사 8개가 통과했다.

집중 캡션 검사 8개는 통과했다. CI에서 `src/` 내부 테스트가 4,094개에서 4,102개로 늘어 테스트 배치 정책에 걸렸다. 8개를 공식 자동 수집 경로로 옮겨 원 head CI를 통과시키고, 영향 문서의 Native 및 fresh WASM 결과를 독립 한컴 Print와 전쪽 비교해야 한다. [Lint 로그](https://github.com/edwardkim/rhwp/actions/runs/44651551319/job/136007540871)

## #9237 병합 후 충돌

`upstream/devel`이 `411a9f944c`로 변경됐다. `git merge-tree --write-tree`로 원 head와 누적 검토 브랜치를 각각 시험했고 두 경우 모두 `src/document_core/commands/text_editing.rs`와 `src/renderer/layout/shape_layout.rs`에서 충돌했다. 최신 devel에 추가된 도형 캡션 셀 주소 65534의 편집·적중·저장을 보존하며 새 head로 재검증해야 한다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9220](https://github.com/edwardkim/rhwp/pull/9220)의 head는 `01811f5acd316df971cdde73b965b0a82873025a`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

현재 devel과 캡션 편집 및 배치 코드 두 곳에서 충돌한다. 원 head의 lint에서 소스 내부 테스트 4,102개가 상한 4,094개를 초과했다. 기존 도형 캡션 셀 주소와 새 캡션 동작을 모두 보존해 충돌을 해소하고 테스트 배치·lint와 캡션 독립 Print 및 Native·fresh WASM 전쪽 결과를 확인한다.
