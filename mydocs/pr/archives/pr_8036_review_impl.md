# PR #8036 구현·검증 기록 정정

## 현재 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

## 구현과 과거 실제 CI

원 contributor의 `prev_vpos_end > 5000` 조건과 2쪽 기대는 같은 최소 입력의 독립 Print와 충돌한다.
Print와 고정 base는 1쪽이고, 후보 Native257만 2쪽이었다. `55f06decaf`에서 기존 시작 vpos
기준을 복원해 Native258/fresh WASM075 모두 1쪽·raw 100%로 보정했다. 원 PR의 조건을
그대로 수용했다는 판정은 하지 않는다. 원 PR이 언급한 별도의 13쪽 문서는 이 최소 입력의
검증으로 대신하지 않는다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 이 CI는 통합 당시의 실행 이력이며 후속 원문 대조에서 추가한 엔진·회귀 변경의 검증을 대신하지 않는다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8036_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

위 대체 입력에서 전체 페이지 Native/fresh WASM TSV와 같은 페이지 review/overlay를 새로 산출했다. 배경 실물 문서의 검증과 구분한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440902731`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.

<!-- source-claim-native-start -->
## 추가 확보 문서의 Native 직접 대조

원 PR에 입력이 첨부되지 않아 내용이 대응하는 공식 배포 문서를 확보했다. 원 작성자 파일과의 바이트 동일성은 주장하지 않는다. [pr8036-seoul-official.hwpx](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8036-seoul-official.hwpx)와 같은 입력의 [한컴 Print PDF](../assets/pr8319_evidence_alignment_20261027/official-pdf/hancom-print/pr8036-seoul-official-2020.pdf)를 사용했다.
후보 Native 전체 쪽수는 13쪽, PDF는 13쪽이다. 선택 쪽 검토 상태는 `passed`다. 새로 확보한 문서의 전쪽 90% 보정은 작업지시에서 면제했으며 점수 미달·쪽수 차이를 통과로 바꾸지 않는다.

11~12쪽의 연속 표 소유를 보정해 Print와 같은 13쪽으로 출력한다. base는 12쪽이어서 12개 공통 쪽만 비교한다.

- 물리 11쪽: raw 99.99015% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/native/review_011.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/native/overlay_011.png)
- 물리 12쪽: raw 98.07662% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/native/review_012.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/native/overlay_012.png)

[전체 페이지 Native TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/native/silhouette.tsv)

Native의 고정 `upstream/devel` 대비 가능한 쪽별 raw 0.1%p 초과 하락은 없었다. 문서 쪽수가 다르면 공통 쪽 비교는 누락 쪽을 검증하지 못한다.
<!-- source-claim-native-end -->

## #8319 통합 전 동일 입력 대조

통합 merge의 첫 부모 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`에서 같은 입력과 Print PDF를 Native로 출력했다. 전체 쪽수는 RHWP 12쪽, PDF 13쪽이며 아래 선택 쪽 최저 raw는 40.64021%다. 선택 영역의 일치율은 실제 주장 동작의 개선이나 전체 문서 통과를 단독으로 입증하지 않는다.

- 물리 11쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/native-before-integration/review_011.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/native-before-integration/overlay_011.png)
- 물리 12쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/native-before-integration/review_012.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/native-before-integration/overlay_012.png)

<!-- source-claim-wasm-start -->
## 추가 확보 문서의 fresh WASM 대조

동일 원문과 한컴 Print를 96dpi에서 비교했다. 전체 13쪽의 raw 최저는 90.02149%이며, 90% 미만 쪽은 없음이다. 선정한 주장 영역의 결과는 아래와 같다.

- 물리 11쪽: raw 99.51010% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/wasm/review_011.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/wasm/overlay_011.png)
- 물리 12쪽: raw 97.63828% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/wasm/review_012.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/wasm/overlay_012.png)

[전체 페이지 WASM TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8036/wasm/silhouette.tsv)

고정 base와 비교 가능한 같은 번호의 쪽에서 raw 0.1%p 초과 하락은 없었다. 원문 전체 페이지의 기존 90% 미만은 작업지시에 따라 명시하며 통과로 표현하지 않는다.
<!-- source-claim-wasm-end -->

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `tests/fixtures/pr8036_stored_tall_line/continuing.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8036/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8036/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8036/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8036/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8036/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8036/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
