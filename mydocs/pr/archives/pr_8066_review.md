# PR #8066 리뷰 — fix(layout): 글자처럼 취급 RowBreak 표도 마지막 줄 간격만 넘친 행을 선언 높이로 둔다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8066](https://github.com/edwardkim/rhwp/pull/8066) / semanticist21.
- 원 contributor head: `11f72a8d3428c6da52980e759f61b1119badc3e8`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8066_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **강원 입사지원서 HWPX의 주소 행**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

- 실제 검증 입력: [208.hwpx](../assets/s21_20261027/inputs/208.hwpx).
- 입력 SHA-256: `e672f8042449111ebcff0bd74336b382356c77680a9e4fe0549cf65dd33a2f83`.
- 독립 Print PDF: [208-2020.pdf](../assets/s21_20261027/inputs/208-2020.pdf).
- PDF SHA-256: `99bf4e5d23a240fa7541fe6313e773f61928602623c89aaa03e59049955823bc`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8066/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8066/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8066/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8066/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8066/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8066/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

RowBreak TAC 칸의 마지막 공백 줄 간격은 실제 내용 하단이 선언 행에 들어가는지와 구분한다. tac_rowbreak_whitespace_tail_keeps_declared_row는 source declared HU를 기준으로 높이를 비교하며, 글줄 잉크·여백이 실제로 넘친 경우까지 선언 높이로 강제하는 근거가 아니다. #8065의 실제 내용/여백 성장과 반대 경계로 유지한다.


실제 회귀 source:
- [tests/cases/tac_rowbreak_whitespace_tail_keeps_declared_row.rs](../../../tests/cases/tac_rowbreak_whitespace_tail_keeps_declared_row.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8066#issuecomment-6440932363)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/s21_20261027/inputs/208.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8066/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8066/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8066/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8066/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8066/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8066/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
