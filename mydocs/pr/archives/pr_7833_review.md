# PR #7833 리뷰 — Fix inline object positions and boundary cut/paste in cells

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#7833](https://github.com/edwardkim/rhwp/pull/7833) / semanticist21.
- 원 contributor head: `255413cdbc74d34f7c1d1be09fb7ad640eac6feb`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_7833_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [pr7833-flat-false-tables-false-cut.hwpx](../../../tests/fixtures/hwpx_cell_frame_print/pr7833-flat-false-tables-false-cut.hwpx).
- 입력 SHA-256: `61aeefefc423eb88c4e21d36018a9b5d0bb67ff3902766049d990691609ed897`.
- 독립 Print PDF: [pr7833-flat-false-tables-false-cut-2020.pdf](../../../pdf/semanticist21-20261019/generated-hwpx/pr7833-flat-false-tables-false-cut/pr7833-flat-false-tables-false-cut-2020.pdf).
- PDF SHA-256: `57cbd0f72b13b8a0e8049c465c3230c7c7643adda0e005deb2275450ad0c4190`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7833/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7833/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7833/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7833/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7833/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7833/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

cell boundary cut은 문자열만 지우지 않고 첫/끝 문단의 개체 슬롯과 서식을 같은 선택 범위로 제거한다. flat/nested 주소와 source UTF-16·logical 오프셋을 분리하고 Field·range tag·markpen의 미지원 경계는 변경 전에 거절한다. 빈 범위가 끝 개체를 지우지 않는 대조군을 유지한다. 글꼴 trace는 CharOverlap 뒤 확정할 수 없는 원문 주소를 null/unavailable로 내보내며, WASM 검사의 독립 CFB 원문 경계 fixture가 임의 scalar 주소를 주장하지 못하게 한다. 마지막 source의 trace 계약은 새 base에 포함된 동등 구현과 현재 강화 검사를 대조했다. 동등 포함 커밋은 아래 계보 표에 구분한다.


실제 회귀 source:
- [tests/cases/cell_boundary_object_cut.rs](../../../tests/cases/cell_boundary_object_cut.rs).
- [tests/cases/hwpx_cell_frame_print.rs](../../../tests/cases/hwpx_cell_frame_print.rs).
- [tests/cases/inline_cell_physical_width_print.rs](../../../tests/cases/inline_cell_physical_width_print.rs).
- [tests/cases/issue_4961_font_decision_trace.rs](../../../tests/cases/issue_4961_font_decision_trace.rs).
- [tests/issue_1452_saved_caret.rs](../../../tests/issue_1452_saved_caret.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/7833#issuecomment-6440886217)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
