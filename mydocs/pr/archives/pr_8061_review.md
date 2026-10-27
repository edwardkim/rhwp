# PR #8061 리뷰 — 셀 그림의 편집 소유와 쪽 그리기 순서

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8061](https://github.com/edwardkim/rhwp/pull/8061) / semanticist21.
- 원 contributor head: `a9c06c8a8a526205a59c68b1ed40eb66d84ad9c2`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8061_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **여권 발급 신청서 HWP의 사진 칸**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8045-mofa.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8045-mofa.hwp), SHA-256 `4bf58b220d6a70e7efbc9ad5bf7d501e2f1291430a1fff9b9bb3afc9f25fba6c`.

출처: [공식 게시물](https://overseas.mofa.go.kr/cn-qingdao-ko/brd/m_1373/view.do?seq=1291199). #8045와 같은 원본이다. 사진 칸의 얼굴 윤곽선·사진 크기·머리 크기 안내 문구가 원 PR 캡처와 일치한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [picture-z2.hwpx](../../../tests/fixtures/pr8061_cell_picture_layer/picture-z2.hwpx).
- 입력 SHA-256: `759bf88c72ac5d1b2ff221756abbad2bbaac7776af752350f003bd4db9e83998`.
- 독립 Print PDF: [picture-z2-2020.pdf](../../../pdf/pr8061_review/picture-z2-2020.pdf).
- PDF SHA-256: `fc519b025cd35fb85e2be7e6505b74a0e0a4b0328f7bb3b1c18bb109dcf308a9`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8061/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8061/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8061/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8061/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8061/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8061/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

- [anchored_cell_paint.rs](../../../src/renderer/layout/anchored_cell_paint.rs)의 그림 생성 후
  layer 부여 → 셀 순서 조립 → 같은 쪽 도형과 묶음 정렬 → 기존 paint replay가 소비한다.
- [render_tree.rs](../../../src/renderer/render_tree.rs)의 내부 묶음은 `serde(skip)`이며
  임시 NodeId로 z-order를 정하지 않는다. 편집 주소와 절대 좌표는 유지한다.
- [새 회귀](../../../tests/cases/verified_cell_picture_layer_scope.rs)의 8개 조건은 셀 주소,
  하나의 bitmap·PHOTO 보존, 원본 단위의 포함, 독립 Print 좌표 ±2px, 실제 그리기 순서를
  검사한다. Native 전용 1개는 같은 8조건의 실제 PNG 가림 색상도 검사한다.
- 잘못된 이전 `cell_behind_text_picture_above_paper_shapes.rs` 검사는 삭제했다.
  과거 후보 `61528013511c87c732524f91c6f224286a055337`에서 이 검사가 PASS했던 것은
  잘못된 가정을 검출하지 못한 결과다. 해당 PASS를 현재 결함의 검증 근거로 쓰지 않는다.


실제 회귀 source:
- [tests/cases/verified_cell_picture_layer_scope.rs](../../../tests/cases/verified_cell_picture_layer_scope.rs).
- [tests/cases/verified_rowbreak_successor_print_frame.rs](../../../tests/cases/verified_rowbreak_successor_print_frame.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8061#issuecomment-6440928574)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
