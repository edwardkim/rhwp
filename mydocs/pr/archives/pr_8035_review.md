# PR #8035 리뷰 — fix(edit): 개체를 지우면 뒤 글자 모양 경계도 당긴다

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8035](https://github.com/edwardkim/rhwp/pull/8035) / semanticist21.
- 원 contributor head: `a6c94e5bc20a3586a1c7d640d460901479ee64cd`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8035_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [212.hwpx](../assets/s21_20261027/inputs/212.hwpx).
- 입력 SHA-256: `d8c5ad8e8f31461d2b74b63bac9dc1a6e60541f02ec58a1048a9b079028f52c9`.
- 독립 Print PDF: [212-2020.pdf](../assets/s21_20261027/inputs/212-2020.pdf).
- PDF SHA-256: `c310ea3c5ec9d880e6997282712e15f990bc07e6055ddf4b2e3d3b3bd4472508`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8035/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8035/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8035/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8035/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8035/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8035/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

개체 삭제 뒤 텍스트, char_offsets 및 char_shapes 경계를 개체 없는 동일 문단과 비교한다. 맨 앞 그림·중간 그림·선두 수식과 서식 시작/끝 경계가 다른 사례를 구분한다. HWP/HWPX 저장 정규화 때문에 바이트 전체를 맞추지 않고, 각각의 형식으로 왕복한 독립 대조 문단과 글자모양 경계를 비교한다.

tests/cases/delete_inline_control_char_shapes.rs의 3함수와 두 저장 형식. 공통 paragraph 슬롯 제거를 picture/equation/shape/table 삭제 소비자가 호출하는 변경 범위를 별도로 대조한다.


실제 회귀 source:
- [tests/cases/delete_inline_control_char_shapes.rs](../../../tests/cases/delete_inline_control_char_shapes.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8035#issuecomment-6440901569)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
