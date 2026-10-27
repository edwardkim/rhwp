# PR #8032 리뷰 — fix(layout): 문단 끝 글자처럼 취급 개체가 줄에 안 들어가면 그 앞에서 줄을 나눈다

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8032](https://github.com/edwardkim/rhwp/pull/8032) / semanticist21.
- 원 contributor head: `db86410f5883a557b9895dc2f88e51e793684b07`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8032_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [wide.hwpx](../../../tests/fixtures/pr8032_print_alpha/wide.hwpx).
- 입력 SHA-256: `9ef1b0180eab92c311901a5be26936ac737df165c6a0cdf9b8a743553519b07d`.
- 독립 Print PDF: [pr8032-print-alpha-wide-2020.pdf](../../../pdf/semanticist21-20261019/pr8032-print-alpha-wide-2020.pdf).
- PDF SHA-256: `ca30c1fce58e933f0a20c7b842f1cd1c306d071f2f64b1fef110449532d5187d`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 98.78322% | 98.78322% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8032/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8032/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8032/native/overlay.png) |
| wasm | 1 | 98.78261% | 98.78261% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8032/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8032/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8032/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

입력 후 재구성하는 줄에서 마지막 TAC 그림도 줄 앞의 텍스트와 가용 폭을 함께 고려한다. 넓은 그림 앞에 실제 문자를 삽입한 공개 API 결과에서 둘째 줄 text_start가 그림의 원본 슬롯인지 검사하고, 첫 텍스트 줄이 그림 높이를 잘못 가져가지 않는지 확인한다. 좁은 그림은 같은 줄에 남는 정상 대조군으로 유지한다.

tests/cases/tac_picture_wraps_before_wide_object.rs의 wide/narrow 두 경계. 15360·4000은 입력 그림의 HWPUNIT 높이 계약이며 고정 화면 위치를 기대값으로 삼지 않는다.


실제 회귀 source:
- [tests/cases/print_alpha_source_grid.rs](../../../tests/cases/print_alpha_source_grid.rs).
- [tests/cases/tac_picture_wraps_before_wide_object.rs](../../../tests/cases/tac_picture_wraps_before_wide_object.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8032#issuecomment-6440899027)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
