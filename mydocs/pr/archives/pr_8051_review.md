# PR #8051 리뷰 — fix(layout): 쪽을 넘어 나뉜 표 조각에도 cellzone·표 바깥 테두리를 적용한다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8051](https://github.com/edwardkim/rhwp/pull/8051) / semanticist21.
- 원 contributor head: `98a9500d70ac1878b41b822c5bc5fc21ee56396d`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8051_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **문체부 세계 책의 날 보도자료 HWPX의 붙임 표**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8051-official.hwpx](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8051-official.hwpx), SHA-256 `690e1cad747b643c331f563780271f1323690aded489f205a5a1eefb037b5b1e`.

출처: [공식 게시물](https://www.mcst.go.kr/site/s_notice/press/pressView.jsp?pSeq=21784). 붙임 3 전국 공공도서관과 민간 행사 목록, 서울도서관·세계 책과 저작권의 날 기념 토크콘서트·4.23(수) 19:00~20:30 및 첫 행 주요 내용이 원 PR과 일치한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [199.hwpx](../assets/s21_20261027/inputs/199.hwpx).
- 입력 SHA-256: `6659ed9830af931405704b5dd23bcb3da05b11ca76be4f7b9ba599b5300865ac`.
- 독립 Print PDF: [199-2020.pdf](../assets/s21_20261027/inputs/199-2020.pdf).
- PDF SHA-256: `37cdbbd886447e06d992d7a8a9f7aea29a47e1782fc919050194cd719978caea`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 2 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8051/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8051/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8051/native/overlay.png) |
| wasm | 2 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8051/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8051/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8051/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

일반 표만 적용하던 cellzone·외곽 보충을 partial table에도 전달하되 조각의 소유 행·첫/마지막 변을 확인한다. split_table_cellzone_border는 40행 입력이 실제 두 쪽 이상으로 나뉜 후 첫 조각의 zone 색 선을 수집한다. 위/아래 선은 실제 조각 bbox와 원본 첫 행 높이로 검사하며 전체 SVG 해시를 고정하지 않는다. 반복 머리행의 별도 증거가 없는 assertion만으로 전체 반복 머리행 정확성을 주장하지 않는다.


실제 회귀 source:
- [tests/cases/split_table_cellzone_border.rs](../../../tests/cases/split_table_cellzone_border.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8051#issuecomment-6440917285)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
