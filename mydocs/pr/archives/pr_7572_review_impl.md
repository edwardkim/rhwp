# PR #7572 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

원 PR은 편집 서식의 상속 규칙 변경을 요구하지 않는다. `beginBatch`와 `endBatch` 사이의
본문 `applyCharFormat`·`replaceBodyTextLocal`이 중간 쪽 나눔을 확정하지 않아야 한다는
계약을 실패 검사 두 개로 제시한다. 원 PR의 측정 시간은 성능 보장으로 사용하지 않는다.

보정한 `commands/formatting.rs`의 본문 서식 변경은
`rebuild_paragraph_deferred_in_batch`를 통해 스타일과 해당 문단 조합을 즉시 갱신하고,
배치 중에는 구역을 dirty로 남긴다. `commands/text_editing.rs`의 local replace도 문단
조합 결과를 갱신한 뒤 배치 여부를 먼저 확인한다. `flowChanged`가 true여도 배치 중에는
직접 `paginate()`하지 않으며 `documentPaginationPending`을 true로 반환한다.
마지막 쪽 소속은 공통 `endBatch`가 확정한다. 특정 문서·음절·좌표로 분기하지 않는다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 이 CI는 통합 당시의 실행 이력이며 후속 원문 대조에서 추가한 엔진·회귀 변경의 검증을 대신하지 않는다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_7572_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440874055`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/s21_20261027/inputs/178.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 3쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7572/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7572/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7572/native/silhouette.tsv)
- WASM: 전체 3쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7572/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7572/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7572/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
