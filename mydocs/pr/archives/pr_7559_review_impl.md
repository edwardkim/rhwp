# PR #7559 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

공개 API로 생성한 새 표의 저장 전후 원점 불일치를 재현한 기여다. 공통 computed_host_frame의 표 원점·점유 끝을 fit과 실제 배치가 함께 소비하도록 보정했다. 작은/큰 빈 줄, 양수 offset, 다음 문단 앞 간격, 표 위로 들어가는 글줄과 회피 글줄을 구분한다. 원점 불변의 상대 오차는 독립 Print의 절대 좌표 핀과 구분한다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 이 CI는 통합 당시의 실행 이력이며 후속 원문 대조에서 추가한 엔진·회귀 변경의 검증을 대신하지 않는다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_7559_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440872531`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/computed_empty_host_line_20261020/public_api_originals/inputs/offset-hwpx.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7559/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7559/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7559/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7559/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7559/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7559/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
