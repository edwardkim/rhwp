# PR #9220 검토 — fix(caption): 캡션 크기·위치를 바꾸면 다시 줄 나누고, 셀 아래 캡션·머리말 그림 캡션·캡션 속 그림을 한/글처럼 다룬다

## 통합 후보의 검토 결과

캡션의 줄바꿈·아래 여백·셀·머리말·캡션 속 그림을 여덟 개 저장본으로 검증했다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](semanticist21_20261121_review.md)에 기록한다.

## 접수 당시 판정

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

## 메인테이너 보완 검증 중간 결과

캡션 검사 8건을 공개 편집 API를 사용하는 통합 검사로 옮겼다. 모두 실행됐고 devel 대비 RustUnitTier 검사도 4,094건으로 통과했다. 독립 Print와 대조하면 셀 그림과 달리 글자처럼 취급한 머리말 그림·캡션 안 그림의 실제 출력에 차이가 남는다. [쪽별 비교 원장](assets/semanticist21-20261121/maintainer-comparison.tsv)

## 최신 devel 병합 후 재검증

최신 devel의 #9277을 반영한 후보 `bbd1d7b90d`에서 편집으로 생성한 8가지 HWP를 각각 한컴 2020 Print와 대조했다. Native 전쪽 최저는 99.54%다. 본문 그림에 캡션을 달기, 캡션이 있는 그림을 머리말에 복사하기, 머리말에 복사한 뒤 캡션을 달기, 캡션 안 그림의 두 번째 캡션은 각각 100.00%다. 마지막 사례에서는 안쪽 그림과 ‘그림 2’ 표시도 확인했다. 머리말에서 캡션을 추가한 사례는 setter 보정 후 생성한 파일로 측정했다.

같은 HWP 8개를 fresh WASM Canvas로 인쇄한 결과 전쪽 최저 99.59%다. 머리말에 복사한 뒤 캡션을 단 경우와 캡션 속 그림은 양쪽 모두 100.00%다. [입력·한컴 PDF 해시 및 Native/WASM 쪽별 수치](assets/semanticist21-20261121/captions/pages.tsv) · [각 상태의 독립 Print](../../pdf/pr9220-picture-captions)
