# PR #8030 리뷰 — fix(render): 반각 구두점은 글리프가 전진폭보다 넓을 때만 눌러 그린다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8030](https://github.com/edwardkim/rhwp/pull/8030) / semanticist21.
- 원 contributor head: `08d0a12a60b74da18eeb0cd9b834c7bfc7730522`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8030_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **표준근로계약서의 따옴표**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8030-new-official.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8030-new-official.hwp), SHA-256 `8acbe67ba95fd53432b80cbdbc82e9cb118534775ada58b832437eae2b4ea99c`.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

현재 연결한 실행 근거는 파일을 소비하지 않는 단위 계약이다. 함수의 반환·경계 검증과 코드 CI 결과를 기록한다. 다른 파일의 PNG를 그 단위 검사의 실물 문서 증거로 붙이지 않는다.

## 구현과 회귀의 검증 범위

halfwidth_punct_paint_scale은 전진폭과 실제 glyph 폭의 비율을 최대 1로 제한한다. 이미 좁은 따옴표를 0.5배로 다시 누르거나 작은 glyph를 늘리지 않는다. 폭을 얻지 못한 기존 fallback과 정상 전각 측정은 별도 경계로 남긴다. 측정의 논리 advance를 바꾸는 수정이 아니라 Canvas·Skia가 소비하는 paint 배율을 공유하는 수정이다.

tests/cases/halfwidth_punct_paint_scale.rs의 넓음/좁음/같음/폭 없음 네 경계. 수치 예제의 20·10은 합성 함수 인자이며 실물 문서의 절대 위치 핀이 아니다.


실제 회귀 source:
- [tests/cases/halfwidth_punct_paint_scale.rs](../../../tests/cases/halfwidth_punct_paint_scale.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8030#issuecomment-6440894199)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
