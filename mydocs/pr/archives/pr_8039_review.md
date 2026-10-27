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

- 실제 검증 입력: [pr8039-control-tail-true.hwpx](../../../tests/fixtures/hwpx_cell_frame_print/pr8039-control-tail-true.hwpx).
- 입력 SHA-256: `22391a6e27cde14e353279be278eb184e4cc2df5a9a3306c3d5a0cd41b671ea2`.
- 독립 Print PDF: [pr8039-control-tail-true-2020.pdf](../../../pdf/semanticist21-20261019/generated-hwpx/pr8039-control-tail-true/pr8039-control-tail-true-2020.pdf).
- PDF SHA-256: `0a1dd0de95c683deabf54499477415fd4ebe9926889748a6154812da07c9375f`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8039/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8039/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8039/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8039/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8039/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8039/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

글자 없는 control-only 저장 줄도 유효한 line box를 소유한다. TextRun이 없다는 사실로 줄의 전진을 없애지 않고, 같은 점유를 한 줄로 합친 대조군과 뒤 문단 N의 원점이 같아야 한다. 실제 소비는 composer의 줄 높이, typeset의 전진, layout의 개체 배치이며 control 개수만 더해 같은 표를 세로로 중복 합산하지 않는다.


실제 회귀 source:
- [tests/cases/hwpx_cell_frame_print.rs](../../../tests/cases/hwpx_cell_frame_print.rs).
- [tests/cases/tac_host_control_only_tail_line.rs](../../../tests/cases/tac_host_control_only_tail_line.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8039#issuecomment-6440906509)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
