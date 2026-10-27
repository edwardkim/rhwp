# PR #8052 리뷰 — fix(typeset): 꽉 찬 쪽 뒤 쪽 나누기 빈 마지막 문단은 빈 쪽을 만들지 않는다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8052](https://github.com/edwardkim/rhwp/pull/8052) / semanticist21.
- 원 contributor head: `9017ad94acee48ac8d5dbb58af64ff5fc2f1d9f1`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8052_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **사행행위 시설완료 신고서 HWP의 마지막 쪽**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8050-law-current.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8050-law-current.hwp), SHA-256 `6371d1fcf8f9574430c0df8e6141cd9d7e6b2c11aa10bc3afeeb8e5cdf07cf04`.

출처: [공식 게시물](https://www.law.go.kr/LSW/lsInfoP.do?lsiSeq=261039). #8050과 동일한 원본이다. 원 PR의 신고서 전체 항목 및 처리절차가 일치한다. 쪽수 차이는 별도 검증 사항이며 문서 식별 성공과 구분한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [200.hwpx](../assets/s21_20261027/inputs/200.hwpx).
- 입력 SHA-256: `6050a3e9573c4bfca97e3554d6cae92a17b6979cd660341c0b400fb7a2c01c92`.
- 독립 Print PDF: [200-2020.pdf](../assets/s21_20261027/inputs/200-2020.pdf).
- PDF SHA-256: `2144d8c6cb8f0bd9a8b7edd6288ced43e0819d2e26f048280da10b1e959befbf`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 2 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8052/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8052/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8052/native/overlay.png) |
| wasm | 2 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8052/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8052/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8052/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

원 PR의 끝 빈 쪽 제거 주장은 공개 API로 만든 정상 저장본 Print와 구분한다. 현재 fresh_full_page_break_preserves_terminal_paragraph_on_next_page는 독립 Print가 실제 2쪽임을 확인한 38/41줄 입력을 사용해 잘못된 1쪽 기대를 폐기했다. 문단 수 보존, 첫 문단 1쪽·마지막 문단 2쪽 소속을 함께 검사한다. 빈 끝 문단이라는 이유만으로 명시적 쪽 나누기를 흡수하지 않는다.


실제 회귀 source:
- [tests/cases/terminal_empty_page_break_after_full_page.rs](../../../tests/cases/terminal_empty_page_break_after_full_page.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8052#issuecomment-6440918467)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
