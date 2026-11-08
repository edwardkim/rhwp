# PR #7694 리뷰 — 기능: 인라인 개체를 논리 캐럿 위치로 원자적으로 이동하는 API 추가

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#7694](https://github.com/edwardkim/rhwp/pull/7694) / semanticist21.
- 원 contributor head: `0e7383d03753ac2546b0be74f19a244fe81747ff`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_7694_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [chart-after.hwp](../../../tests/fixtures/inline-control-move/chart-after.hwp).
- 입력 SHA-256: `8ff56111b683005b7d4cd34990e7214debc7785d9bef6bc76a2772d036ace617`.
- 독립 Print PDF: [pr7694-updated-198-chart-after-2020.pdf](../../../pdf/semanticist21_source_contracts_20261026/pr7694-updated-198-chart-after-2020.pdf).
- PDF SHA-256: `2f93373067c868addc624878b01c99f25fd13235de2924976fa17b5a44a90d57`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 99.71479% | 99.71479% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7694/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7694/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7694/native/overlay.png) |
| wasm | 1 | 99.73289% | 99.73289% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7694/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7694/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7694/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

move_inline_control_native는 배포 문서·원본/대상 주소·잠금·logical caret를 먼저 검증하고 영향 구역 복제본에서 remove/insert를 실행한다. 실제 문서 commit 이전에 실패할 수 있는 검사를 끝내는 순서를 확인한다. 같은 위치 no-op은 이벤트를 발행하지 않으며, source 제거 후 sibling 경로와 목적지 offset을 다시 계산한다. inline_control_move의 18개 검사와 분리한 유효 저장본 검사 1개는 Unicode·필드·형광펜·범위 태그·BinData·실제 OOXML 차트 payload 및 공개 HWP/HWPX 저장을 포함한다. 이동 전후 실제 차트의 과거 시각 미달은 뒤의 OLE/차트 공통 보정으로 해결했으며, 원 이동 API와 차트 paint 보정의 범위를 구분한다.


실제 회귀 source:
- [tests/cases/inline_control_move.rs](../../../tests/cases/inline_control_move.rs).
- [tests/cases/ooxml_chart_print_frame.rs](../../../tests/cases/ooxml_chart_print_frame.rs).
- [tests/cases/valid_inline_move_fixture_contract.rs](../../../tests/cases/valid_inline_move_fixture_contract.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/7694#issuecomment-6440883725)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `tests/fixtures/inline-control-move/chart-after.hwp`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 99.71479%, 해당 쪽 99.71479% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7694/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7694/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7694/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 99.73289%, 해당 쪽 99.73289% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7694/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7694/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr7694/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
