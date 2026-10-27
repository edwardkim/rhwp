# PR #8014 리뷰 — fix(render): 선 종류가 없음인 직선을 그리지 않는다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8014](https://github.com/edwardkim/rhwp/pull/8014) / semanticist21.
- 원 contributor head: `4c857d41e760bbb80f88650558d376504bb10874`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8014_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **통계청 인구동향 보도자료 41쪽**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [2026년+7월+인구동향.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/2026년+7월+인구동향.hwp), SHA-256 `6f87a7a4de1e89bab89be40cec8523d106300a259ee953481d3d288f91fd99ad`.

출처: [공식 게시물](https://mods.go.kr/board.es?mid=a10301010000&bid=204&act=view&list_no=447111). 2026년 7월 인구동향 41쪽 보도자료의 제목·본문 대응을 확인했다. 사용자 Mac 파일과 바이트 동일성은 확인하지 못했다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [195.hwpx](../assets/s21_20261027/inputs/195.hwpx).
- 입력 SHA-256: `56fb2054fa2a57d09a4e024edc5bd29f6f67a47424e2b19096a18bbb986d4684`.
- 독립 Print PDF: [195-2020.pdf](../assets/s21_20261027/inputs/195-2020.pdf).
- PDF SHA-256: `081082724f2e50034da71cfb3a91c291820d00641eb5bc2fd3348e9c4791c5c7`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 2 | 99.98236% | 99.98384% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8014/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8014/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8014/native/overlay.png) |
| wasm | 2 | 99.76268% | 99.76268% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8014/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8014/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8014/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

직선의 border_line.attr 하위 6비트가 0이면 실제 Line 노드를 지우지 않고 visible=false로 만든다. 따라서 출력 제외와 선택·개체 소유를 구분하며, 실선 대조군의 선 하나만 줄어드는지 검사한다. 같은 shape_layout의 일반·변환 경로가 line_visible을 소비하는지 함께 확인했다.

tests/cases/line_shape_type_none.rs의 동일 입력 SOLID/NONE 대조. 이 SVG assertion 자체를 독립 Print 일치율로 해석하지 않는다.


실제 회귀 source:
- [tests/cases/line_shape_type_none.rs](../../../tests/cases/line_shape_type_none.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8014#issuecomment-6440888736)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
