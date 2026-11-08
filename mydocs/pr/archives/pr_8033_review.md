# PR #8033 리뷰 — fix(typeset): 저장 줄 없는 글자처럼 취급 표의 바깥 여백을 쪽 맞춤에서 한 번만 센다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8033](https://github.com/edwardkim/rhwp/pull/8033) / semanticist21.
- 원 contributor head: `8a02d977b6e72e9a8117a15a0f786e0cc0cfb598`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8033_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **서울시교육청 가정통신문의 공익제보센터 안내 표**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

원 PR의 공개 메모리 IR factory를 저장한 HWPX를 대체 입력으로 진단했다. 메모리 문서는 1쪽이지만 저장본은 새 Native/fresh WASM에서 각각 2쪽, 같은 저장본의 MCP 2020 Print는 1쪽이었다. 이 추가 진단은 실패이며 성공 TSV·새 golden·원 가정통신문의 해결 증거로 사용하지 않는다.

[진단 입력과 결과](../assets/pr8319_evidence_alignment_20261027/baseline_diagnostic/pr8033/diagnostic.json)를 보존한다. 이 증적 정정에서는 엔진이나 회귀의 기대값을 변경하지 않았다.

## 구현과 회귀의 검증 범위

TAC fit은 format이 이미 포함한 tac_outer_margin_v_px를 중복 계상하지 않고 최종 소유 줄·실측 표 높이를 소비한다. 현재 tac_fit::prepare의 단일 소유 frame, 저장 frame, 편집 후 성장 floor를 함께 확인했다. 한 번의 여백으로 들어가는 입력과 같은 식에서 실제로 넘치는 입력을 각각 1쪽·2쪽으로 검사해 무조건 fit하는 완화를 막는다.

tests/cases/tac_table_no_lineseg_outer_margin_fit.rs의 fits/overflows 경계. 실제 저장 frame과 no-LineSeg 재조판은 서로 다른 분기이며 한쪽 성공으로 다른 분기를 대체하지 않는다.


실제 회귀 source:
- [tests/cases/tac_table_no_lineseg_outer_margin_fit.rs](../../../tests/cases/tac_table_no_lineseg_outer_margin_fit.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8033#issuecomment-6440900196)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
