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

- 선행 대조 검사 입력: [ta-pic-caption-tight.hwpx](../assets/caption_autonumber_projection_20261021/inputs/ta-pic-caption-tight.hwpx).
- 입력 SHA-256: `32cdf807f2a9ceae26671a07ab88ea69bcbc5e2082a6aabe75ae265b2e5b8c4f`.
- 독립 Print PDF: [ta-pic-caption-tight-2020.pdf](../assets/caption_autonumber_projection_20261021/pdf/ta-pic-caption-tight-2020.pdf).
- PDF SHA-256: `0800c0ffc142e50abb76b4db6f69a8201c0849813c924e928b10893a4947f9d8`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 99.98107% | 99.98107% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8049/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8049/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8049/native/overlay.png) |
| wasm | 1 | 99.98110% | 99.9811% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8049/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8049/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8049/wasm/overlay.png) |

위 표의 수치와 PNG는 선행 대조 검사 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

자동 번호의 원문 자리표 한 칸과 화면 표시 문자열을 composed run에 함께 두고 캡션 배치·caret·hit-test가 같은 투영을 소비한다. 원 기대 문자열은 유지하고 검사 소비자를 display_or_text로 바로잡았다. 그림/표 접두사·실제 번호·앞뒤 공백이 없는 반례를 독립 한컴 Print와 대조한다. 출력 번호를 원문에 덧붙여 logical 길이를 바꾸지 않는다.


실제 회귀 source:
- [tests/cases/caption_auto_number_position.rs](../../../tests/cases/caption_auto_number_position.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8049#issuecomment-6440914752)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

<!-- source-claim-native-start -->
## 추가 확보 문서의 Native 직접 대조

원 PR에 입력이 첨부되지 않아 내용이 대응하는 공식 배포 문서를 확보했다. 원 작성자 파일과의 바이트 동일성은 주장하지 않는다. [pr8049-official.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8049-official.hwp)와 같은 입력의 [한컴 Print PDF](../assets/pr8319_evidence_alignment_20261027/official-pdf/hancom-print/pr8049-official-2020.pdf)를 사용했다.
후보 Native 전체 쪽수는 82쪽, PDF는 82쪽이다. 선택 쪽 검토 상태는 `passed`다. 새로 확보한 문서의 전쪽 90% 보정은 작업지시에서 면제했으며 점수 미달·쪽수 차이를 통과로 바꾸지 않는다.

선택한 20쪽은 원 PR이 지목한 표 캡션이다. 통합 전 Native의 렌더 트리도 번호 `7`을 표시값으로 보유하고 있어, 이 Native 대조만으로 수정 전 번호 누락을 재현했다고 주장하지 않는다. 전체 82쪽 중 10쪽과 82쪽은 90% 미만이며 고정 base와 같은 점수다.

- 물리 20쪽: raw 99.98028% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8049/native/review_020.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8049/native/overlay_020.png)

[전체 페이지 Native TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8049/native/silhouette.tsv)

Native의 고정 `upstream/devel` 대비 가능한 쪽별 raw 0.1%p 초과 하락은 없었다. 문서 쪽수가 다르면 공통 쪽 비교는 누락 쪽을 검증하지 못한다.
<!-- source-claim-native-end -->

## #8319 통합 전 동일 입력 대조

통합 merge의 첫 부모 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`에서 같은 입력과 Print PDF를 Native로 출력했다. 전체 쪽수는 RHWP 82쪽, PDF 82쪽이며 아래 선택 쪽 최저 raw는 99.97922%다. 선택 영역의 일치율은 실제 주장 동작의 개선이나 전체 문서 통과를 단독으로 입증하지 않는다.

- 물리 20쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8049/native-before-integration/review_020.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8049/native-before-integration/overlay_020.png)

<!-- source-claim-wasm-start -->
## 추가 확보 문서의 fresh WASM 대조

동일 원문과 한컴 Print를 96dpi에서 비교했다. 전체 82쪽의 raw 최저는 26.63057%이며, 90% 미만 쪽은 10, 82이다. 선정한 주장 영역의 결과는 아래와 같다.

- 물리 20쪽: raw 99.25188% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8049/wasm/review_020.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8049/wasm/overlay_020.png)

[전체 페이지 WASM TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8049/wasm/silhouette.tsv)

고정 base와 비교 가능한 같은 번호의 쪽에서 raw 0.1%p 초과 하락은 없었다. 원문 전체 페이지의 기존 90% 미만은 작업지시에 따라 명시하며 통과로 표현하지 않는다.
<!-- source-claim-wasm-end -->

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/caption_autonumber_projection_20261021/inputs/ta-pic-caption-tight.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 99.98107%, 해당 쪽 99.98107% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8049/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8049/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8049/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 99.98110%, 해당 쪽 99.98110% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8049/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8049/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8049/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
