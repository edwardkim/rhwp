# PR #8057 리뷰 — fix(render): 가운뎃점으로 쓴 U+2024를 가운데 점으로 그린다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8057](https://github.com/edwardkim/rhwp/pull/8057) / semanticist21.
- 원 contributor head: `76271c14e4823a10a69747930f1c128f7bc7380a`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8057_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **통계청 가계동향 11쪽의 U+2024**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8057-official.hwpx](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8057-official.hwpx), SHA-256 `7180ce00f71bb9fc4285d045b6a0fd8bfbdd1c60642176c95a0496c19463ff6e`.

출처: [공식 게시물](https://mods.go.kr/board.es?act=view&bid=214&list_no=446633&mid=a10301040400). 물리 11쪽의 4) 주거․수도․광열 (↑), 표 첫 두 행의 347·122와 U+2024 표기가 원 PR 캡처와 일치한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [gulim.hwpx](../assets/verified_point_suppliers_20261025/inputs/gulim.hwpx).
- 입력 SHA-256: `446a7d75666788b9dde8586716a592b507ad1c689dd50709890771d84735e3e8`.
- 독립 Print PDF: [gulim-2020.pdf](../../../pdf/verified-point-suppliers/gulim-2020.pdf).
- PDF SHA-256: `31d9188b0212d222d7eb46e391febaaf0bc591516fa8939a12c2720f7c1fdcc5`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8057/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8057/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8057/native/overlay.png) |
| wasm | 1 | 98.37398% | 98.37398% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8057/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8057/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8057/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

Unicode U+2024를 글꼴과 무관하게 벡터 원으로 바꾸는 원 검사는 독립 Print와 맞지 않아 수용하지 않았다. 현재 verified_point_glyph_supplier는 Gulim/Arial의 실제 공급자와 HCR Batang의 원 프로그램 우선을 분리하고 동일 glyph로 측정/paint한다. 세 입력의 전쪽 Native224/fresh063 raw 100%·직접 확대 판독과 독립 PDF 좌표가 선행 근거이며, 원문 기호·뒤 글자 순서 관계도 유지한다.


실제 회귀 source:
- [tests/cases/verified_point_glyph_supplier.rs](../../../tests/cases/verified_point_glyph_supplier.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8057#issuecomment-6440926189)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
