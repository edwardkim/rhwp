# PR #8062 리뷰 — fix(typeset): 개체만 있는 줄과 겹치는 저장 줄은 저장 vpos 차이만큼 나아간다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8062](https://github.com/edwardkim/rhwp/pull/8062) / semanticist21.
- 원 contributor head: `e8b4e14d017d86c96be2c676b6d56bab9cfa657d`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8062_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **통계청 고용동향 HWP/HWPX 요약 쪽**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [ssec2609.hwpx](../assets/pr8319_evidence_alignment_20261027/official-inputs/ssec2609.hwpx), SHA-256 `28f84a2dd9ea81f87f892b8a61e4f72d54bb9fa128dee354bdd223b735c911ec`.

출처: [공식 게시물](https://mods.go.kr/board.es?mid=a10301010000&bid=210&act=view&list_no=447479). 요약 쪽의 70.6%·44.0%·56.1%·2.0%·5.4%·4.2% 및 취업자 2,926만 6천명이 원 PR과 일치한다. section2의 높이 3403HU인 저장 줄 vpos=500/600, textpos=0/24도 원 PR 수치와 일치한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [206.hwpx](../assets/s21_20261027/inputs/206.hwpx).
- 입력 SHA-256: `af558f112f87e6e62e8bb9ef764c8f34502fd13643b99e7a8dbf6199761b964b`.
- 독립 Print PDF: [206-2020.pdf](../assets/s21_20261027/inputs/206-2020.pdf).
- PDF SHA-256: `5b0bb6548804a340ad13d809bf057b4d644cb59b6020db1510003edeb04154b0`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8062/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8062/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8062/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8062/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8062/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8062/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

저장된 개체 줄의 잉크 높이와 다음 줄 원점까지의 흐름 걸음은 다르다. overlapping_object_lines_advance_by_the_stored_vpos_step은 조판 부호 줄과 큰 TAC shape를 둔 뒤 lineHeightSum+lineSpacingSum이 원본 마지막 vpos+height의 범위를 이중 합산하지 않는지 검사한다. 원본 HWPUNIT에서 환산한 점유 계약을 절대 화면 좌표 핀으로 오인해 제거하지 않는다.


실제 회귀 source:
- [tests/cases/object_line_overlapping_stored_ladder.rs](../../../tests/cases/object_line_overlapping_stored_ladder.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8062#issuecomment-6440929829)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
