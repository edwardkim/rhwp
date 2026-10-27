# PR #8043 리뷰 — feat(hyperlink): 문서 내부 책갈피 링크를 넣고 고치고 읽는다

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8043](https://github.com/edwardkim/rhwp/pull/8043) / semanticist21.
- 원 contributor head: `a0bb863a08edf4e1461fbbf18ac2baaeaaea4450`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8043_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [197.hwpx](../assets/s21_20261027/inputs/197.hwpx).
- 입력 SHA-256: `8cac55deaa8102e4afcb66535679f5ae852028cc13d939758194c9480249e507`.
- 독립 Print PDF: [197-2020.pdf](../assets/s21_20261027/inputs/197-2020.pdf).
- PDF SHA-256: `25a5c691a535a18ee06edc17eae6c292c3a83ec973b605ee55399819ec46a76e`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8043/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8043/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8043/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8043/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8043/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8043/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

책갈피 내부 링크의 encoding/classification을 URL 문자열 추정과 구분한다. insert/update/query와 HWP/HWPX 저장 뒤 재열기에서 bookmark 대상이 보존되는지 확인하고, 기존 웹 링크가 내부 책갈피로 바뀌지 않는 대조군을 둔다. 기존 passthrough invalidation guard의 호출 소비와 연결되며 테스트 통과를 URL 외부 이동의 실제 실행 증거로 확대하지 않는다.


실제 회귀 source:
- [tests/cases/issue_6963_bookmark_hyperlink.rs](../../../tests/cases/issue_6963_bookmark_hyperlink.rs).
- [tests/issue_2724_passthrough_invalidation_guard.rs](../../../tests/issue_2724_passthrough_invalidation_guard.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8043#issuecomment-6440912081)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
