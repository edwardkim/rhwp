# PR #8032 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

입력 후 재구성하는 줄에서 마지막 TAC 그림도 줄 앞의 텍스트와 가용 폭을 함께 고려한다. 넓은 그림 앞에 실제 문자를 삽입한 공개 API 결과에서 둘째 줄 text_start가 그림의 원본 슬롯인지 검사하고, 첫 텍스트 줄이 그림 높이를 잘못 가져가지 않는지 확인한다. 좁은 그림은 같은 줄에 남는 정상 대조군으로 유지한다.

tests/cases/tac_picture_wraps_before_wide_object.rs의 wide/narrow 두 경계. 15360·4000은 입력 그림의 HWPUNIT 높이 계약이며 고정 화면 위치를 기대값으로 삼지 않는다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 이 CI는 통합 당시의 실행 이력이며 후속 원문 대조에서 추가한 엔진·회귀 변경의 검증을 대신하지 않는다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8032_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440899027`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `tests/fixtures/pr8032_print_alpha/wide.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 98.78322%, 해당 쪽 98.78322% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8032/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8032/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8032/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 98.78261%, 해당 쪽 98.78261% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8032/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8032/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8032/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
