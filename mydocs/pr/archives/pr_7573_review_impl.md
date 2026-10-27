# PR #7573 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

`copySelection`과 `deleteRange`가 선택의 경계 표를 다르게 소유하면 같은 범위를 오려
빈 문단에 붙일 때 표가 둘로 늘어난다. 원 PR은 공개 API로 `abc`·표·`xyz`·빈 문단을
생성하고 정상 중간 선택1개와 양 끝 경계 실패2개를 제시한다.

보정은 `commands/text_editing.rs`의 공통 `delete_paragraph_boundary`에서 복사와 같은
반개방 글자 범위를 사용한다. 문단 끝까지 선택한 경우 후행 개체를 포함하며, 경계 안의
표·그림을 역순으로 제거한 뒤 텍스트를 삭제한다. 개체 슬롯 제거는 모델의 공통
`remove_control`을 사용해 글자 모양과 필드의 control 번호도 함께 갱신한다. 같은 문단,
여러 문단과 셀 경계 삭제가 이 연산을 소비한다. 특정 문서·표 번호의 예외를 추가하지 않는다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 이 CI는 통합 당시의 실행 이력이며 후속 원문 대조에서 추가한 엔진·회귀 변경의 검증을 대신하지 않는다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_7573_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440875355`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/s21_20261027/inputs/181.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 99.57627%, 해당 쪽 99.57627% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 99.07728%, 해당 쪽 99.07728% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
