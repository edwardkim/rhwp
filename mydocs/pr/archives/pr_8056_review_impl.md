# PR #8056 구현·검증 기록 정정

## 현재 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

## 구현과 과거 실제 CI

넘침 압축 판단에서 장식 없는 꼬리 보통 공백만 visible extent에서 제외한다. 실제 문자열·logical 슬롯은 보존하며 밑줄 등 공백 잉크가 있는 경우는 별도 조건이다. trailing_space_does_not_compress_fitting_right_aligned_number는 같은 숫자의 자연 폭을 먼저 재고, 가시 글자만 들어가는 칸에서 꼬리 공백 유무에 따른 글자 원점들이 같아야 한다. 처음 독립 Print의 생성 입력 두 개는 Native/fresh 모두 100%였고 최종 head 결과와 혼동하지 않는다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 이 CI는 통합 당시의 실행 이력이며 후속 원문 대조에서 추가한 엔진·회귀 변경의 검증을 대신하지 않는다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8056_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

위 대체 입력에서 전체 페이지 Native/fresh WASM TSV와 같은 페이지 review/overlay를 새로 산출했다. 배경 실물 문서의 검증과 구분한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440923301`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.

<!-- source-claim-native-start -->
## 추가 확보 문서의 Native 직접 대조

원 PR에 입력이 첨부되지 않아 내용이 대응하는 공식 배포 문서를 확보했다. 원 작성자 파일과의 바이트 동일성은 주장하지 않는다. [pr8056-official.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8056-official.hwp)와 같은 입력의 [한컴 Print PDF](../assets/pr8319_evidence_alignment_20261027/official-pdf/hancom-print/pr8056-official-2024.pdf)를 사용했다.
후보 Native 전체 쪽수는 12쪽, PDF는 12쪽이다. 선택 쪽 검토 상태는 `re_review_required`다. 새로 확보한 문서의 전쪽 90% 보정은 작업지시에서 면제했으며 점수 미달·쪽수 차이를 통과로 바꾸지 않는다.

선택한 11쪽은 raw 90% 미만이며 페이지 전체의 위치 차이가 남는다. `120,283`의 첫 숫자와 마지막 숫자 원점 간격은 통합 전과 후 모두 약 36.64px로 한컴 Print와 같아, 이 Native 대조는 원 PR의 수정 전 압축을 재현한 증거가 아니다.

- 물리 11쪽: raw 65.65455% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8056/native/review_011.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8056/native/overlay_011.png)

[전체 페이지 Native TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8056/native/silhouette.tsv)

Native의 고정 `upstream/devel` 대비 가능한 쪽별 raw 0.1%p 초과 하락은 없었다. 문서 쪽수가 다르면 공통 쪽 비교는 누락 쪽을 검증하지 못한다.
<!-- source-claim-native-end -->

## #8319 통합 전 동일 입력 대조

통합 merge의 첫 부모 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`에서 같은 입력과 Print PDF를 Native로 출력했다. 전체 쪽수는 RHWP 12쪽, PDF 12쪽이며 아래 선택 쪽 최저 raw는 65.91221%다. 선택 영역의 일치율은 실제 주장 동작의 개선이나 전체 문서 통과를 단독으로 입증하지 않는다.

이 대체 입력의 Native에서는 원 PR이 보여 준 수정 전 결함이 같은 위치에서 재현되지 않았다. 현재 출력을 원 작성자 파일의 수정 전후 증거로 확대하지 않는다.

- 물리 11쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8056/native-before-integration/review_011.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8056/native-before-integration/overlay_011.png)

<!-- source-claim-wasm-start -->
## 추가 확보 문서의 fresh WASM 대조

동일 원문과 한컴 Print를 96dpi에서 비교했다. 전체 12쪽의 raw 최저는 66.39946%이며, 90% 미만 쪽은 11이다. 선정한 주장 영역의 결과는 아래와 같다.

- 물리 11쪽: raw 66.39946% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8056/wasm/review_011.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8056/wasm/overlay_011.png)

[전체 페이지 WASM TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8056/wasm/silhouette.tsv)

고정 base와 비교 가능한 같은 번호의 쪽에서 raw 0.1%p 초과 하락은 없었다. 원문 전체 페이지의 기존 90% 미만은 작업지시에 따라 명시하며 통과로 표현하지 않는다.
<!-- source-claim-wasm-end -->

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/s21_20261027/inputs/205.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8056/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8056/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8056/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8056/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8056/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8056/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
