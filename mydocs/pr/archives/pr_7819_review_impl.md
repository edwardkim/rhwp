# PR #7819 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

CharOverlap과 note의 logical 한 칸을 원문 scalar 길이와 구분한다. model::Control의 논리 슬롯 분류부터 공개 getter·caret·selection·copy/split/merge 소비까지 같은 축을 확인한다. empty overlap·동일 앵커·non-BMP·말미 hit·실제 note 순서가 반례다. 정상 source 문자와 합성 overlap payload를 혼동하지 않으며, px 검사는 issue_7819_caret_axis에 독립 Print와 양쪽 98% 이상을 기록한 해당 페이지로 한정한다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 이 CI는 통합 당시의 실행 이력이며 후속 원문 대조에서 추가한 엔진·회귀 변경의 검증을 대신하지 않는다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_7819_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440884974`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-true.hwp`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7819/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7819/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7819/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7819/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7819/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7819/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
