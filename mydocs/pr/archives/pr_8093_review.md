# PR #8093 리뷰 — fix(edit): 글자처럼 취급한 양식 개체도 캐럿 한 칸을 차지한다

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8093](https://github.com/edwardkim/rhwp/pull/8093) / semanticist21.
- 원 contributor head: `75c93fe0cc91c6ff74ebd85ac3c8f3d3b708a6d9`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8093_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [form-slot.hwp](../../../tests/fixtures/pr8093_form_caret/form-slot.hwp).
- 입력 SHA-256: `1c7e40474066cfc7c4892ab8d3b2e6ae7dde7055335ae2330c772e1cdc159b6c`.
- 독립 Print PDF: [form-slot-hwp-2020.pdf](../../../pdf/pr8093_review/form-slot-hwp-2020.pdf).
- PDF SHA-256: `fd36f88c6e8766ee36bf6dbfd36dea9defe33e6efa11aea18c0c53caa3f87817`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8093/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8093/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8093/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8093/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8093/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8093/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

Form의 treatAsChar를 논리 한 칸으로 투영하고, inline_control_projection·equation_line_metrics·inline_box_rows·nested_table_frames가 Form 메트릭을 소비한다. 같은 줄 본문과 개체의 높이·기준선 합집합을 사용하며 정상 저장 LineSeg를 임의로 덮지 않는다. 기존 화면 폭 90px 하한은 원본 Form 폭과 실제 caret 구간의 관계로 바꿨다. 입력·Enter 뒤 문단 소유도 유지하고 양쪽 100% 페이지의 독립 가/나 원점만 ±2px로 보호한다.


실제 회귀 source:
- [tests/cases/form_object_caret_slot.rs](../../../tests/cases/form_object_caret_slot.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8093#issuecomment-6440934691)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `tests/fixtures/pr8093_form_caret/form-slot.hwp`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8093/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8093/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8093/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8093/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8093/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8093/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
