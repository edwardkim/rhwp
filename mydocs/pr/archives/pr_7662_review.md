# PR #7662 리뷰 — 수정: 개체 바로 앞에서 나눈 문단이 그 개체로 시작하게 한다

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#7662](https://github.com/edwardkim/rhwp/pull/7662) / semanticist21.
- 원 contributor head: `377bcb71e7404a741105a28303a3fae8b671fc9d`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_7662_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [188.hwpx](../assets/s21_20261027/inputs/188.hwpx).
- 입력 SHA-256: `d454147e39fb3fcb6070f3b21edece5dbba1bee10dc063422043cb6b8aa7033e`.
- 독립 Print PDF: [188-2020.pdf](../assets/s21_20261027/inputs/188-2020.pdf).
- PDF SHA-256: `2f79486fefcbd252a897b2d29619b8b7f2d4f6716a81e7030466db0ca3eafd7d`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7662/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7662/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7662/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7662/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7662/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7662/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

Paragraph::split_at은 logical control 위치와 scalar 문자열 분할점을 분리한다. 새 문단으로 가는 control은 UTF-16 gap 시작부터 이동시키고, 끝 자동번호는 자리표 문자와 앞 문단에 남긴다. Field는 한 글자짜리 일반 개체로 더하지 않고 이관된 FieldRange의 control 참조로 함께 이동한다. split_before_control의 개체 앞/뒤 Enter, merge undo, 개체부터 복사, 자동번호와 제목 차례 표시를 각각 대조한다.


실제 회귀 source:
- [tests/cases/split_before_control.rs](../../../tests/cases/split_before_control.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/7662#issuecomment-6440882337)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
