# PR #8037 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

표 문단에 그림이 형제 control로 추가돼도 그 그림의 흐름 참여와 표 호스트의 여백 점유를 혼동하지 않는다. picture_on_table_host_keeps_following_flow는 본문/셀 삽입 직후와 각각의 HWP/HWPX 재열기에서 동일한 표 위끝·뒤 문단 원점을 비교한다. 화면의 특정 y값을 고정하지 않는 전후 관계 검사이며, 그림이 보인다는 이유로 빈 표 호스트를 없애지 않는 계약이다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 이 CI는 통합 당시의 실행 이력이며 후속 원문 대조에서 추가한 엔진·회귀 변경의 검증을 대신하지 않는다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8037_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440903986`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/pr8319_evidence_alignment_20261027/inputs/pr8037-body-picture-on-host.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 96.19628%, 해당 쪽 96.19628% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8037/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8037/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8037/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 95.89714%, 해당 쪽 95.89714% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8037/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8037/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8037/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
