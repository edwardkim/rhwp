# PR #8030 리뷰 — fix(render): 반각 구두점은 글리프가 전진폭보다 넓을 때만 눌러 그린다

## 최종 판정

**증적 대응 정정 — 공식 대체 입력과 단위 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8030](https://github.com/edwardkim/rhwp/pull/8030) / semanticist21.
- 원 contributor head: `08d0a12a60b74da18eeb0cd9b834c7bfc7730522`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8030_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **표준근로계약서의 따옴표**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 공식 배포 문서로 대조했다. 작성자 원본과의 바이트 동일성은 확인하지 않았다.**

공식 배포처에서 확보한 대체 문서: [pr8030-new-official.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8030-new-official.hwp), SHA-256 `8acbe67ba95fd53432b80cbdbc82e9cb118534775ada58b832437eae2b4ea99c`.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

기존 단위 계약은 함수의 반환·경계를 검사한다. 추가 확보 문서의 Print 대조는 아래 별도 항목에 입력과 범위를 명시한다.

## 구현과 회귀의 검증 범위

halfwidth_punct_paint_scale은 전진폭과 실제 glyph 폭의 비율을 최대 1로 제한한다. 이미 좁은 따옴표를 0.5배로 다시 누르거나 작은 glyph를 늘리지 않는다. 폭을 얻지 못한 기존 fallback과 정상 전각 측정은 별도 경계로 남긴다. 측정의 논리 advance를 바꾸는 수정이 아니라 Canvas·Skia가 소비하는 paint 배율을 공유하는 수정이다.

tests/cases/halfwidth_punct_paint_scale.rs의 넓음/좁음/같음/폭 없음 네 경계. 수치 예제의 20·10은 합성 함수 인자이며 실물 문서의 절대 위치 핀이 아니다.


실제 회귀 source:
- [tests/cases/halfwidth_punct_paint_scale.rs](../../../tests/cases/halfwidth_punct_paint_scale.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8030#issuecomment-6440894199)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

<!-- source-claim-native-start -->
## 추가 확보 문서의 Native 직접 대조

원 PR에 입력이 첨부되지 않아 내용이 대응하는 공식 배포 문서를 확보했다. 원 작성자 파일과의 바이트 동일성은 주장하지 않는다. [pr8030-new-official.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8030-new-official.hwp)와 같은 입력의 [한컴 Print PDF](../assets/pr8319_evidence_alignment_20261027/official-pdf/hancom-print/pr8030-new-official-2020.pdf)를 사용했다.
후보 Native 전체 쪽수는 7쪽, PDF는 7쪽이다. 선택 쪽 검토 상태는 `passed`다. 새로 확보한 문서의 전쪽 90% 보정은 작업지시에서 면제했으며 점수 미달·쪽수 차이를 통과로 바꾸지 않는다.

- 물리 1쪽: raw 98.54612% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8030/native/review_001.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8030/native/overlay_001.png)

[전체 페이지 Native TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8030/native/silhouette.tsv)

Native의 고정 `upstream/devel` 대비 가능한 쪽별 raw 0.1%p 초과 하락은 없었다. 문서 쪽수가 다르면 공통 쪽 비교는 누락 쪽을 검증하지 못한다.
<!-- source-claim-native-end -->

## #8319 통합 전 동일 입력 대조

대체 계약서에서 원 PR 따옴표 위치의 네 glyph는 통합 전후 Native 픽셀이 동일했다. 이 자료에서 수정 전 작은 따옴표 결함이 재현됐다고 주장하지 않는다.

통합 merge의 첫 부모 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`에서 같은 입력과 Print PDF를 Native로 출력했다. 전체 쪽수는 RHWP 7쪽, PDF 7쪽이며 아래 선택 쪽 최저 raw는 98.54616%다. 선택 영역의 일치율은 실제 주장 동작의 개선이나 전체 문서 통과를 단독으로 입증하지 않는다.

- 물리 1쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8030/native-before-integration/review_001.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8030/native-before-integration/overlay_001.png)

<!-- source-claim-wasm-start -->
## 추가 확보 문서의 fresh WASM 대조

동일 원문과 한컴 Print를 96dpi에서 비교했다. 전체 7쪽의 raw 최저는 97.59822%이며, 90% 미만 쪽은 없음이다. 선정한 주장 영역의 결과는 아래와 같다.

- 물리 1쪽: raw 98.65555% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8030/wasm/review_001.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8030/wasm/overlay_001.png)

[전체 페이지 WASM TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8030/wasm/silhouette.tsv)

고정 base와 비교 가능한 같은 번호의 쪽에서 raw 0.1%p 초과 하락은 없었다. 원문 전체 페이지의 기존 90% 미만은 작업지시에 따라 명시하며 통과로 표현하지 않는다.
<!-- source-claim-wasm-end -->
