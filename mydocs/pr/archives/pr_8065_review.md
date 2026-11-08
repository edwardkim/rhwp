# PR #8065 리뷰 — fix(layout): 안 여백이 선언 높이보다 큰 칸도 행을 내용만큼 키운다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8065](https://github.com/edwardkim/rhwp/pull/8065) / semanticist21.
- 원 contributor head: `72c825408e39d2084a85b746440fd29469f2805c`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8065_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **건축·대수선·용도변경 허가 신청서 HWP**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8065-law-current.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8065-law-current.hwp), SHA-256 `1cbb4f77a3b190a9057081293f4a030055530369c986012327ac49511e82009f`.

출처: [공식 게시물](https://www.law.go.kr/LSW/lsInfoP.do?lsiSeq=273047). 2025-07-31 개정, 6쪽 중 제1쪽, 협회 회원번호·전자우편 송달동의·대지조건 및 전체 개요 칸이 원 PR 캡처와 일치한다. 강남구의 2021-06-25 판본은 제외한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 선행 대조 검사 입력: [207.hwpx](../assets/s21_20261027/inputs/207.hwpx).
- 입력 SHA-256: `698ca8c3b6cb6c9696a11b30af703fa1e079f184a03d61f73b18ed5e3512af91`.
- 독립 Print PDF: [207-2020.pdf](../assets/s21_20261027/inputs/207-2020.pdf).
- PDF SHA-256: `7c1d945fe8282e1ea0575b08450de97521be6f332ea9b8293bbb7763c0427ab2`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8065/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8065/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8065/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8065/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8065/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8065/wasm/overlay.png) |

위 표의 수치와 PNG는 선행 대조 검사 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

선언 높이보다 큰 실제 안 여백+글줄 점유는 측정과 행 배치 모두에서 유지한다. cell_padding_exceeding_declared_height_grows_row는 셀이 원본 PAD_TOP+line_height+PAD_BOTTOM 이상이고 글줄이 위 여백 아래에 포함되는지를 검사한다. unused padding 필드와 진짜 선언 여백을 구분하는 #6442, 기존 최소 높이 및 content-cut fit을 보호하는 #7234 경계를 함께 확인한다. RowBreak 이름을 행 경계 전용 정책으로 확대 해석하지 않는다.


실제 회귀 source:
- [tests/cases/cell_padding_exceeding_declared_height_grows_row.rs](../../../tests/cases/cell_padding_exceeding_declared_height_grows_row.rs).
- [tests/cases/issue_5751_dense_table_row_growth.rs](../../../tests/cases/issue_5751_dense_table_row_growth.rs).
- [tests/cases/issue_6442_unused_cell_padding_is_garbage.rs](../../../tests/cases/issue_6442_unused_cell_padding_is_garbage.rs).
- [tests/cases/issue_7234_scaffold_table_cell_height.rs](../../../tests/cases/issue_7234_scaffold_table_cell_height.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8065#issuecomment-6440931170)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

<!-- source-claim-native-start -->
## 추가 확보 문서의 Native 직접 대조

원 PR에 입력이 첨부되지 않아 내용이 대응하는 공식 배포 문서를 확보했다. 원 작성자 파일과의 바이트 동일성은 주장하지 않는다. [pr8065-law-current.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8065-law-current.hwp)와 같은 입력의 [한컴 Print PDF](../assets/pr8319_evidence_alignment_20261027/official-pdf/hancom-print/pr8065-law-current-2020.pdf)를 사용했다.
후보 Native 전체 쪽수는 6쪽, PDF는 6쪽이다. 선택 쪽 검토 상태는 `passed`다. 새로 확보한 문서의 전쪽 90% 보정은 작업지시에서 면제했으며 점수 미달·쪽수 차이를 통과로 바꾸지 않는다.

- 물리 1쪽: raw 99.31007% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/native/review_001.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/native/overlay_001.png)
- 물리 2쪽: raw 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/native/review_002.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/native/overlay_002.png)

[전체 페이지 Native TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/native/silhouette.tsv)

Native의 고정 `upstream/devel` 대비 가능한 쪽별 raw 0.1%p 초과 하락은 없었다. 문서 쪽수가 다르면 공통 쪽 비교는 누락 쪽을 검증하지 못한다.
<!-- source-claim-native-end -->

## #8319 통합 전 동일 입력 대조

공식 대체 문서 1쪽의 렌더 트리에서 셀 81개의 외곽 상자는 통합 전후 동일했다. 이 Native 출력만으로 원 PR이 보고한 행 높이 차이를 수정 전 재현했다고 주장하지 않는다.

통합 merge의 첫 부모 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`에서 같은 입력과 Print PDF를 Native로 출력했다. 전체 쪽수는 RHWP 6쪽, PDF 6쪽이며 아래 선택 쪽 최저 raw는 99.30931%다. 선택 영역의 일치율은 실제 주장 동작의 개선이나 전체 문서 통과를 단독으로 입증하지 않는다.

- 물리 1쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/native-before-integration/review_001.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/native-before-integration/overlay_001.png)
- 물리 2쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/native-before-integration/review_002.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/native-before-integration/overlay_002.png)

<!-- source-claim-wasm-start -->
## 추가 확보 문서의 fresh WASM 대조

동일 원문과 한컴 Print를 96dpi에서 비교했다. 전체 6쪽의 raw 최저는 98.96611%이며, 90% 미만 쪽은 없음이다. 선정한 주장 영역의 결과는 아래와 같다.

- 물리 1쪽: raw 99.14776% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/wasm/review_001.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/wasm/overlay_001.png)
- 물리 2쪽: raw 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/wasm/review_002.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/wasm/overlay_002.png)

[전체 페이지 WASM TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8065/wasm/silhouette.tsv)

고정 base와 비교 가능한 같은 번호의 쪽에서 raw 0.1%p 초과 하락은 없었다. 원문 전체 페이지의 기존 90% 미만은 작업지시에 따라 명시하며 통과로 표현하지 않는다.
<!-- source-claim-wasm-end -->

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/s21_20261027/inputs/207.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8065/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8065/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8065/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8065/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8065/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8065/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
