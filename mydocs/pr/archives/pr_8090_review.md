# PR #8090 리뷰 — fix: 구역이 둘 이상인 문서에서 앞 구역 미주의 편집 위치를 찾는다

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8090](https://github.com/edwardkim/rhwp/pull/8090) / semanticist21.
- 원 contributor head: `642d7330274e7a2c5a7516959a430cc95e8ac091`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8090_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [209.hwpx](../assets/s21_20261027/inputs/209.hwpx).
- 입력 SHA-256: `b7e549f3aa68ec0a338fe15f287db03cf683f2444fb765e422e0b159538e63c1`.
- 독립 Print PDF: [209-2020.pdf](../assets/s21_20261027/inputs/209-2020.pdf).
- PDF SHA-256: `6cda0e705d476cd00a99cad338df523018bf3160dd2774cc34a026e686a52109`.
- 대표 대응: 이 입력의 독립 PDF 2쪽 ↔ RHWP 2쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 2 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8090/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8090/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8090/native/overlay.png) |
| wasm | 2 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8090/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8090/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8090/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

find_endnote_edit_target은 원본 구역 주소와 미주를 그린 마지막 구역의 페이지 주소를 구분한다. 앞 구역 미주 변경 시 마지막 구역의 dirty 전파까지 있어야 실제 편집 뒤 target이 갱신된다. 두 구역을 만든 후 첫 구역에 미주를 넣고 본문을 편집하는 회귀는 edit info의 pageNum과 note caret의 pageIndex를 함께 확인한다. 파일 최초 열기의 정상 결과만으로 편집 후 캐시 갱신을 대체하지 않는다.


실제 회귀 source:
- [tests/cases/endnote_edit_target_multi_section.rs](../../../tests/cases/endnote_edit_target_multi_section.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8090#issuecomment-6440933505)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
