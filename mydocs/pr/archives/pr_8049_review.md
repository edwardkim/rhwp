# PR #8049 리뷰 — 캡션 자동 번호의 자리표와 표시 문자열

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8049](https://github.com/edwardkim/rhwp/pull/8049) / semanticist21.
- 원 contributor head: `285f86c92c27eda03616491a1d1f0c8210c9b80a`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8049_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **국토해양부 국가물류기본계획 HWP의 표 캡션**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8049-official.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8049-official.hwp), SHA-256 `29018def6969c60e6b0fb5c7cca06c8fd12a3b1239543c8f3eb1d1d03bc4294e`.

출처: [공식 게시물](https://molit.go.kr/USR/policyData/m_34681/dtl.jsp?id=75). 목차에 <표 Ⅱ-7> 물류산업 업종별 추이 14가 있으며 원 PR 캡처의 표 번호·제목과 일치한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [ta-pic-caption-tight.hwpx](../assets/caption_autonumber_projection_20261021/inputs/ta-pic-caption-tight.hwpx).
- 입력 SHA-256: `32cdf807f2a9ceae26671a07ab88ea69bcbc5e2082a6aabe75ae265b2e5b8c4f`.
- 독립 Print PDF: [ta-pic-caption-tight-2020.pdf](../assets/caption_autonumber_projection_20261021/pdf/ta-pic-caption-tight-2020.pdf).
- PDF SHA-256: `0800c0ffc142e50abb76b4db6f69a8201c0849813c924e928b10893a4947f9d8`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 99.98107% | 99.98107% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8049/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8049/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8049/native/overlay.png) |
| wasm | 1 | 99.98110% | 99.9811% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8049/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8049/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8049/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

자동 번호의 원문 자리표 한 칸과 화면 표시 문자열을 composed run에 함께 두고 캡션 배치·caret·hit-test가 같은 투영을 소비한다. 원 기대 문자열은 유지하고 검사 소비자를 display_or_text로 바로잡았다. 그림/표 접두사·실제 번호·앞뒤 공백이 없는 반례를 독립 한컴 Print와 대조한다. 출력 번호를 원문에 덧붙여 logical 길이를 바꾸지 않는다.


실제 회귀 source:
- [tests/cases/caption_auto_number_position.rs](../../../tests/cases/caption_auto_number_position.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8049#issuecomment-6440914752)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
