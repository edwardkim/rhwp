# PR #8056 리뷰 — fix(layout): 줄 끝 공백은 넘침 판정에 넣지 않는다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8056](https://github.com/edwardkim/rhwp/pull/8056) / semanticist21.
- 원 contributor head: `a0aac732f91ffc12368c5303b2f3d7df25f11671`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8056_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **한국은행 국제외환시장 동향 11쪽의 숫자 칸**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8056-official.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8056-official.hwp), SHA-256 `56b7a19affa96a586c1db2502f3a8f4d597cbafaf1250b7d4214417b94c31905`.

출처: [공식 게시물](https://www.bok.or.kr/portal/bbs/B0000502/view.do?menuNo=200690&nttId=11065634). 물리 11쪽의 인니 7,080·8,647·6,525·6,071·5,943·9,135·5,342, 브라질 120,283·161,125·177,419·186,340·206,506·209,067·142,200 및 러시아 숫자열까지 원 PR 캡처와 일치한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [205.hwpx](../assets/s21_20261027/inputs/205.hwpx).
- 입력 SHA-256: `58b6739b969d07cb0f486566c0667fcf81fe33d327989fd714ff847f64f2e63c`.
- 독립 Print PDF: [205-2020.pdf](../assets/s21_20261027/inputs/205-2020.pdf).
- PDF SHA-256: `ca9e361b88f476ec94a348de0016ae0ccb0a8bc18e61c92520fa7ab18a43524f`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8056/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8056/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8056/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8056/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8056/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8056/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

넘침 압축 판단에서 장식 없는 꼬리 보통 공백만 visible extent에서 제외한다. 실제 문자열·logical 슬롯은 보존하며 밑줄 등 공백 잉크가 있는 경우는 별도 조건이다. trailing_space_does_not_compress_fitting_right_aligned_number는 같은 숫자의 자연 폭을 먼저 재고, 가시 글자만 들어가는 칸에서 꼬리 공백 유무에 따른 글자 원점들이 같아야 한다. 처음 독립 Print의 생성 입력 두 개는 Native/fresh 모두 100%였고 최종 head 결과와 혼동하지 않는다.


실제 회귀 source:
- [tests/cases/cell_trailing_space_overflow_compression.rs](../../../tests/cases/cell_trailing_space_overflow_compression.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8056#issuecomment-6440923301)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
