# PR #8054 리뷰 — fix(layout): 세로 오프셋 위아래 배치 표 뒤의 같은 문단 글을 표 아래에 둔다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8054](https://github.com/edwardkim/rhwp/pull/8054) / semanticist21.
- 원 contributor head: `f033d7c7e3317e76773b6375dedc7224030998ee`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8054_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **서울잠실초 급식 안내 HWPX의 날짜와 학교장 서명**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

- 실제 검증 입력: [203.hwpx](../assets/s21_20261027/inputs/203.hwpx).
- 입력 SHA-256: `778ca806c3b2aed367dfd283dccb85ec51229b08a7b6ef2602ae804ac2ccc25f`.
- 독립 Print PDF: [203-2020.pdf](../assets/s21_20261027/inputs/203-2020.pdf).
- PDF SHA-256: `9c364c74306723adf27bffdc7d90da7f6fa9167e5c507e083b91c1c194859738`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8054/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8054/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8054/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8054/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8054/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8054/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

TopAndBottom 표의 양수 offset은 표 본체 하단과 host 텍스트 재개 원점 양쪽에서 소비해야 한다. host_text_starts_below_table_when_offset_is_smaller_than_its_line은 실제 표 bbox 하단보다 host 글줄이 위로 들어가지 않고 뒤 학교장 문단도 host 뒤에 놓이는지 검사한다. offset 600HU가 글줄보다 작다는 경계가 핵심이며 표를 숨기거나 host를 clamp하지 않는다.


실제 회귀 source:
- [tests/cases/topbottom_offset_table_host_text_below.rs](../../../tests/cases/topbottom_offset_table_host_text_below.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8054#issuecomment-6440921033)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/s21_20261027/inputs/203.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8054/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8054/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8054/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8054/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8054/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8054/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
