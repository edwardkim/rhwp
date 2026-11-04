# PR #7589 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

원 PR은 같은 `FIRST\nSECOND`에서 MultiLine=1/0의 SVG가 달라야 한다고 주장한다.
파서·직렬화가 개행과 속성을 보존하는지 검사한 점은 유지한다. 그러나 그 뒤의
`assert_ne!`는 이 입력의 한컴 인쇄 동작을 입증하지 않는다.

사용자가 두 HWPX를 수정 없이 한컴의 파일 → 인쇄 → PDF로 출력했고,
win10-ted의 `C:\temp`에서 원본 바이트를 가져왔다. 두 PDF는 각각 A4 한 쪽이며
실제 Creator는 `Hwp 2022 12.0.0.4605`다. 파일명의 2020과 실제 제품을 구분한다.
두 Print의 래스터는 동일하고 입력 상자에는 `FIRST`만 표시된다.

공통 `FormObjectNode`의 표시 문자열에서 첫 저장 줄을 선택하되 모델 원문과
MultiLine 속성은 보존하도록 보정했다. SVG·Canvas·Native가 같은 표시 결과를
소비하며 특정 문서나 좌표에 따른 예외는 없다. 모든 입력 상자의 편집 중 화면까지
MultiLine이 무의미하다고 일반화하지 않는다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 이 CI는 통합 당시의 실행 이력이며 후속 원문 대조에서 추가한 엔진·회귀 변경의 검증을 대신하지 않는다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_7589_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440877968`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/pr_7589_form_multiline/pr7589-multiline-true.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7589/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7589/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7589/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 92.37734%, 해당 쪽 92.37734% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7589/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7589/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7589/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
