# PR #8039 리뷰 — fix(layout): 글자처럼 취급 표 뒤 컨트롤만 있는 저장 줄의 높이를 쌓는다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8039](https://github.com/edwardkim/rhwp/pull/8039) / semanticist21.
- 원 contributor head: `765cfbb835982e7c22f3b303db352ecf0f279f55`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8039_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **국토교통부 기계설비 발전 기본계획 HWPX 표지**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8039-official.hwpx](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8039-official.hwpx), SHA-256 `abecaf23efa76ae77ef30979b20bad54470518adb6a2dd73f93ec4066ac16848`.

출처: [공식 게시물](https://www.molit.go.kr/USR/policyData/m_34681/dtl.jsp?id=4875). 표지의 제목·2026.1·국토교통부 로고가 일치하며, textpos=8의 저장 줄 높이 4700HU·간격 2352HU도 원 PR과 일치한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 선행 대조 검사 입력: [pr8039-control-tail-true.hwpx](../../../tests/fixtures/hwpx_cell_frame_print/pr8039-control-tail-true.hwpx).
- 입력 SHA-256: `22391a6e27cde14e353279be278eb184e4cc2df5a9a3306c3d5a0cd41b671ea2`.
- 독립 Print PDF: [pr8039-control-tail-true-2020.pdf](../../../pdf/semanticist21-20261019/generated-hwpx/pr8039-control-tail-true/pr8039-control-tail-true-2020.pdf).
- PDF SHA-256: `0a1dd0de95c683deabf54499477415fd4ebe9926889748a6154812da07c9375f`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8039/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8039/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8039/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8039/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8039/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8039/wasm/overlay.png) |

위 표의 수치와 PNG는 선행 대조 검사 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

글자 없는 control-only 저장 줄도 유효한 line box를 소유한다. TextRun이 없다는 사실로 줄의 전진을 없애지 않고, 같은 점유를 한 줄로 합친 대조군과 뒤 문단 N의 원점이 같아야 한다. 실제 소비는 composer의 줄 높이, typeset의 전진, layout의 개체 배치이며 control 개수만 더해 같은 표를 세로로 중복 합산하지 않는다.


실제 회귀 source:
- [tests/cases/hwpx_cell_frame_print.rs](../../../tests/cases/hwpx_cell_frame_print.rs).
- [tests/cases/tac_host_control_only_tail_line.rs](../../../tests/cases/tac_host_control_only_tail_line.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8039#issuecomment-6440906509)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

## 쪽 경계 보정 후 Native 확인

추가 보정 후 Native와 Print는 모두 22쪽이며 Native 전체 쪽의 최저 raw는 93.57902%다. 위쪽 캡션 그림 뒤 두 글줄은 5쪽에, 다음 본문은 6쪽에 보존된다. 실제 쪽수는 Print와 같다. 표지 제목과 기관명은 Print와 글자 굵기에 차이가 남는다.

[전체 페이지 Native TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/native/silhouette.tsv)

- 물리 5쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/native-pagefix/review_005.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/native-pagefix/overlay_005.png)
- 물리 6쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/native-pagefix/review_006.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/native-pagefix/overlay_006.png)

fresh WASM과 수정 후 영향 문서의 시각 대조 결과는 아래 항목에 기록한다. 전체 Rust 회귀 검증은 별도로 확인한다.

<!-- source-claim-native-start -->
## 추가 확보 문서의 Native 직접 대조

원 PR에 입력이 첨부되지 않아 내용이 대응하는 공식 배포 문서를 확보했다. 원 작성자 파일과의 바이트 동일성은 주장하지 않는다. [pr8039-official.hwpx](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8039-official.hwpx)와 같은 입력의 [한컴 Print PDF](../assets/pr8319_evidence_alignment_20261027/official-pdf/hancom-print/pr8039-official-2020.pdf)를 사용했다.
후보 Native 전체 쪽수는 22쪽, PDF는 22쪽이다. 선택 쪽 검토 상태는 `passed`다. 새로 확보한 문서의 전쪽 90% 보정은 작업지시에서 면제했으며 점수 미달·쪽수 차이를 통과로 바꾸지 않는다.

원 PR이 지적한 표지다. 쪽 경계 보정 후 전체 쪽수는 Print와 같은 22쪽이다. 표지 제목과 기관명은 글자 굵기 차이가 남는다.

- 물리 1쪽: raw 93.57902% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/native/review_001.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/native/overlay_001.png)

[전체 페이지 Native TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/native/silhouette.tsv)

Native의 고정 `upstream/devel` 대비 가능한 쪽별 raw 0.1%p 초과 하락은 없었다. 문서 쪽수가 다르면 공통 쪽 비교는 누락 쪽을 검증하지 못한다.
<!-- source-claim-native-end -->

## #8319 통합 전 동일 입력 대조

통합 merge의 첫 부모 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`에서 같은 입력과 Print PDF를 Native로 출력했다. 전체 쪽수는 RHWP 23쪽, PDF 22쪽이며 아래 선택 쪽 최저 raw는 70.75715%다. 선택 영역의 일치율은 실제 주장 동작의 개선이나 전체 문서 통과를 단독으로 입증하지 않는다.

- 물리 1쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/native-before-integration/review_001.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/native-before-integration/overlay_001.png)

<!-- source-claim-wasm-start -->
## 추가 확보 문서의 fresh WASM 대조

동일 원문과 한컴 Print를 96dpi에서 비교했다. 전체 22쪽의 raw 최저는 89.61973%이며, 90% 미만 쪽은 1이다. 선정한 주장 영역의 결과는 아래와 같다.

- 물리 1쪽: raw 89.61973% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/wasm/review_001.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/wasm/overlay_001.png)

[전체 페이지 WASM TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/wasm/silhouette.tsv)

고정 base와 비교 가능한 같은 번호의 쪽에서 raw 0.1%p 초과 하락은 없었다. 원문 전체 페이지의 기존 90% 미만은 작업지시에 따라 명시하며 통과로 표현하지 않는다.
<!-- source-claim-wasm-end -->

## 쪽 경계 fresh WASM 직접 대조

- 물리 5쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/wasm-pagefix/review_005.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/wasm-pagefix/overlay_005.png)
- 물리 6쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/wasm-pagefix/review_006.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8039/wasm-pagefix/overlay_006.png)

표지 1쪽은 WASM raw 89.61973%로 base와 같아, 5~6쪽의 관계만 회귀 검사로 다루며 전체 쪽수는 고정하지 않는다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `tests/fixtures/hwpx_cell_frame_print/pr8039-control-tail-true.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8039/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8039/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8039/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 100.00000%, 해당 쪽 100.00000% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8039/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8039/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8039/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
