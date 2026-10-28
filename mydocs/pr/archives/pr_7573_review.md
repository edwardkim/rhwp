# PR #7573 검토 — 오려 붙이기 선택 경계의 표 소유 보존

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#7573](https://github.com/edwardkim/rhwp/pull/7573) / semanticist21.
- 원 contributor head: `84e80b5d82455254bdbdaca0a2b19dff92f1be12`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_7573_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [181.hwpx](../assets/s21_20261027/inputs/181.hwpx).
- 입력 SHA-256: `84283f46ef725b412d9bd70509db541f6f2d6f54417870a82fb47ebda72e9a48`.
- 독립 Print PDF: [181-2020.pdf](../assets/s21_20261027/inputs/181-2020.pdf).
- PDF SHA-256: `0dc723d7498e4651c12cfda73dfb6bea0c67717d6e1fa47f65f69a5bb359d9ca`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 99.57627% | 99.57627% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7573/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7573/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7573/native/overlay.png) |
| wasm | 1 | 99.07728% | 99.07728% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7573/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7573/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7573/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

`copySelection`과 `deleteRange`가 선택의 경계 표를 다르게 소유하면 같은 범위를 오려
빈 문단에 붙일 때 표가 둘로 늘어난다. 원 PR은 공개 API로 `abc`·표·`xyz`·빈 문단을
생성하고 정상 중간 선택1개와 양 끝 경계 실패2개를 제시한다.

보정은 `commands/text_editing.rs`의 공통 `delete_paragraph_boundary`에서 복사와 같은
반개방 글자 범위를 사용한다. 문단 끝까지 선택한 경우 후행 개체를 포함하며, 경계 안의
표·그림을 역순으로 제거한 뒤 텍스트를 삭제한다. 개체 슬롯 제거는 모델의 공통
`remove_control`을 사용해 글자 모양과 필드의 control 번호도 함께 갱신한다. 같은 문단,
여러 문단과 셀 경계 삭제가 이 연산을 소비한다. 특정 문서·표 번호의 예외를 추가하지 않는다.


실제 회귀 source:
- [tests/cases/issue_7444_cut_boundary_object.rs](../../../tests/cases/issue_7444_cut_boundary_object.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/7573#issuecomment-6440875355)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/s21_20261027/inputs/181.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 99.57627%, 해당 쪽 99.57627% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 99.07728%, 해당 쪽 99.07728% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7573/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
