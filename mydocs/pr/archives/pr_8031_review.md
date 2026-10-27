# PR #8031 리뷰 — fix(pagination): 표 셀 안의 쪽 번호 위치도 읽는다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8031](https://github.com/edwardkim/rhwp/pull/8031) / semanticist21.
- 원 contributor head: `3b79ee07292b6d386af1ee9e909e0a5549ec52f8`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8031_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **농식품부 보도자료 HWP의 꼬리말**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8031-official.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8031-official.hwp), SHA-256 `f66e6c1d782d35489adb8a610ea88d4a336a014306d811c0521ddb509c757413`.

출처: [공식 게시물](https://www.korea.kr/briefing/pressReleaseView.do?newsId=156785547). 2026-10-18 보도시점, 제목·첫 문단·페이지 하단 번호가 원 PR과 대응한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [pr8031-api-defined-page-number-cell.hwpx](../../../tests/fixtures/pr8031_page_number_cell/pr8031-api-defined-page-number-cell.hwpx).
- 입력 SHA-256: `7e331d2c5a828bf4f915491ee8db3cd0a9eb253d7cc6caad80acebc37044e8b8`.
- 독립 Print PDF: [pr8031-api-defined-page-number-cell-2020.pdf](../../../pdf/semanticist21-20261019/pr8031-api-defined-page-number-cell-2020.pdf).
- PDF SHA-256: `feb6345ec85aa9166fb3d7a6ce5a2359879cdff66f009c293fd237559986f1a9`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8031/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8031/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8031/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8031/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8031/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8031/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

collect_page_number_pos는 문단·control·cell을 역순으로 찾아 기존 마지막 선언 우선순위를 중첩 표 셀까지 확장한다. 머리말·꼬리말의 내용 자체를 구역 선언으로 승격하지 않는다. 새 typeset finalize와 legacy pagination engine이 같은 collector를 소비하며, 공개 render tree의 실제 -1- 표시와 legacy 결과의 page_number_pos를 각각 검사한다.

tests/cases/page_number_pos_in_table_cell.rs의 공개 표시 검사와 legacy paginator 검사. 함수 존재만으로 두 소비 경로가 동일하다고 판정하지 않고 finalize 호출을 대조했다.


실제 회귀 source:
- [tests/cases/hwpx_cell_frame_print.rs](../../../tests/cases/hwpx_cell_frame_print.rs).
- [tests/cases/page_number_pos_in_table_cell.rs](../../../tests/cases/page_number_pos_in_table_cell.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8031#issuecomment-6440897417)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
