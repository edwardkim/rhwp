# PR #9220 검토 — fix(caption): 캡션 크기·위치를 바꾸면 다시 줄 나누고, 셀 아래 캡션·머리말 그림 캡션·캡션 속 그림을 한/글처럼 다룬다

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 캡션의 줄바꿈·아래 여백·셀·머리말·캡션 속 그림을 여덟 개 저장본으로 검증했다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

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

캡션 검사 8건을 공개 편집 API를 사용하는 통합 검사로 옮겼다. 모두 실행됐고 devel 대비 RustUnitTier 검사도 4,094건으로 통과했다. 독립 Print와 대조하면 셀 그림과 달리 글자처럼 취급한 머리말 그림·캡션 안 그림의 실제 출력에 차이가 남는다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 최신 devel 병합 후 재검증

최신 devel의 #9277을 반영한 후보 `bbd1d7b90d`에서 편집으로 생성한 8가지 HWP를 각각 한컴 2020 Print와 대조했다. Native 전쪽 최저는 99.54%다. 본문 그림에 캡션을 달기, 캡션이 있는 그림을 머리말에 복사하기, 머리말에 복사한 뒤 캡션을 달기, 캡션 안 그림의 두 번째 캡션은 각각 100.00%다. 마지막 사례에서는 안쪽 그림과 ‘그림 2’ 표시도 확인했다. 머리말에서 캡션을 추가한 사례는 setter 보정 후 생성한 파일로 측정했다.

같은 HWP 8개를 fresh WASM Canvas로 인쇄한 결과 전쪽 최저 99.59%다. 머리말에 복사한 뒤 캡션을 단 경우와 캡션 속 그림은 양쪽 모두 100.00%다. [입력·한컴 PDF 해시 및 Native/WASM 쪽별 수치](../assets/semanticist21-20261121/captions/pages.tsv) · [각 상태의 독립 Print](../../../pdf/pr9220-picture-captions)

## 기여자가 갱신한 캡션 검사

2026년 11월 21일 기여자가 `01811f5acd`에 `81c9afd75b`를 추가해 캡션 검사 8개를 `tests/cases/caption_edit.rs`로 옮겼다. 통합 후보에서는 이미 `tests/cases/picture_caption_edit_layout.rs`로 옮겨 회귀 묶음 006에 등록했다. 두 파일의 검사 이름과 assertion을 대조했다. 차이는 머리말과 캡션 안 그림을 준비하는 방법이었다. 기여자 버전은 문서 모델에 개체를 넣고, 메인테이너 버전은 공개 복사·붙여넣기 API로 같은 상태를 만든다.

같은 후보 라이브러리와 최소 글꼴 환경에서 두 파일을 각각 컴파일해 8건씩 통과했다. 이미 등록한 파일을 사용하는 병합 커밋 `5fceeb61c4`에 원 기여자의 새 커밋을 보존했다. 병합 전후 전체 Git 트리는 `a62c0341f5b7d4fa46c08922046a7bf74bfbba23`로 같다. 각 검사에서 사용하는 인쇄본과 페이지별 결과는 [캡션 검증표](../assets/semanticist21-20261121/captions/pages.tsv)에 있다.

## 최신 제품 소스의 기능 검증

캡션의 줄바꿈·아래 여백·셀·머리말·캡션 속 그림을 여덟 개 저장본으로 검증했다. 같은 원본의 1쪽 실루엣은 Native 99.57%, fresh WASM 99.57%다. [Native 비교](../assets/semanticist21-20261121/validation/final/representative/pr9220/native/review_001.png) · [WASM 비교](../assets/semanticist21-20261121/validation/final/representative/pr9220/wasm/review_001.png) [17건 통합 검토](pr_9292_review.md)

기여자가 `b9eb3eae43`에서 기존 캡션 head와 최신 devel `d7b5286cbf`를 병합했다. 두 충돌의 최종 코드를 대조했다. 그림 캡션 편집은 원본의 캡션 폭과 여백을 `CellReflowMetrics`에 전달하며, 배치는 바깥 여백 확장과 본문 캡션 여부를 각각 계산한다. 통합 후보와 차이는 지역 변수에 폭을 저장하는 방식과 독립 계산 두 개의 순서였다. 두 부모와 새 기여 head를 모두 포함한 병합 커밋은 `45c312d2cf`다. 병합 전후의 Git 트리는 모두 `34ca6fe4d22cc0d82bb6f90db40a291963991dea`였다.

## 병합 후 기여자 PR 댓글 계획

각 출력에서 8쪽을 확인했다. 전쪽 최저 raw는 Native 99.56569%, fresh WASM 99.56616%다. 대표 페이지는 `caption-narrowing_a_side_caption_rewraps_its_text` 1쪽이다. 줄어든 옆 캡션 너비에 맞춰 텍스트가 다시 나뉘고 그림과 캡션은 같은 셀에 남는다.

대표 페이지의 Native raw는 99.57%, 확대 확인할 자동 진단 항목은 0개다. fresh WASM은 99.57%, 0개다. 상세 색상과 안티앨리어싱은 인쇄 엔진에 따라 달라 96 DPI의 2px raw와 실제 잉크·위치를 함께 판독했다.

임시 비교 파일은 로컬 `output/pr-review/semanticist21-20261121/resume-1407/protection-correction/verified-entry-count/representative/`에 있다. 댓글에서는 병합 커밋의 원본 이미지를 사용한다. [검증 방법](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

| 출력 | 비교 패널 | 겹침 |
| --- | --- | --- |
| native | [비교](../assets/semanticist21-20261121/validation/final/representative/pr9220/native/review_001.png) | [겹침](../assets/semanticist21-20261121/validation/final/representative/pr9220/native/overlay_001.png) |
| wasm | [비교](../assets/semanticist21-20261121/validation/final/representative/pr9220/wasm/review_001.png) | [겹침](../assets/semanticist21-20261121/validation/final/representative/pr9220/wasm/overlay_001.png) |

댓글 본문의 이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/semanticist21-20261121/validation/final/representative/pr9220/` 아래의 해당 파일로 고정한다. 실제 병합 SHA로 바꿔 원 PR에 UTF-8 본문 파일을 게시한 뒤, GitHub API를 통해 본문과 이미지 해시를 확인한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `b9eb3eae431927e5e5dff0eaf348210cf487e437`이 병합 이력에 포함된 것을 확인했다. [원 PR #9220](https://github.com/edwardkim/rhwp/pull/9220) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
