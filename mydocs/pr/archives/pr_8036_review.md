# PR #8036 리뷰 — fix(typeset): 쪽 높이 줄 뒤의 저장 쪽 경계를 줄 끝 위치로 판정한다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8036](https://github.com/edwardkim/rhwp/pull/8036) / semanticist21.
- 원 contributor head: `129f19b451033e212822df6530d131f959886414`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8036_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **서울시 주택정책실 업무계획 HWPX 13쪽**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8036-seoul-official.hwpx](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8036-seoul-official.hwpx), SHA-256 `d4bfa888c37087e8632eaf2705a53c7c8d96423ea4a9aa7cd232b172963c3d6f`.

출처: [공식 게시물](https://news.seoul.go.kr/citybuild/archives/527300). 인쇄 9쪽의 파격적인 미리내집 확대 추진, 1,022호·3,500호·328대 1 및 정책 표가 원 PR과 일치한다. 저장 줄 vpos=4480, height=65136, 끝=69616도 원 PR 수치와 일치한다. 71쪽 의회 보고서는 다른 판본이므로 제외한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [continuing.hwpx](../../../tests/fixtures/pr8036_stored_tall_line/continuing.hwpx).
- 입력 SHA-256: `21bb9f714b5e3ce646142cd4d8a34cd2c50600a89d6b741f75a2f7dbdbb4df0f`.
- 독립 Print PDF: [continuing-2020.pdf](../../../pdf/pr8036_stored_tall_line/continuing-2020.pdf).
- PDF SHA-256: `8ae130f8e28669e5036359e2c2a95d2c992f78947feaf1e9ab2f2811e39b478d`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8036/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8036/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8036/native/overlay.png) |
| wasm | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8036/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8036/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8036/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

원 contributor의 `prev_vpos_end > 5000` 조건과 2쪽 기대는 같은 최소 입력의 독립 Print와 충돌한다.
Print와 고정 base는 1쪽이고, 후보 Native257만 2쪽이었다. `55f06decaf`에서 기존 시작 vpos
기준을 복원해 Native258/fresh WASM075 모두 1쪽·raw 100%로 보정했다. 원 PR의 조건을
그대로 수용했다는 판정은 하지 않는다. 원 PR이 언급한 별도의 13쪽 문서는 이 최소 입력의
검증으로 대신하지 않는다.


실제 회귀 source:
- [tests/cases/verified_stored_tall_line_print_flow.rs](../../../tests/cases/verified_stored_tall_line_print_flow.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8036#issuecomment-6440902731)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
