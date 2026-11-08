# PR #8025 리뷰 — fix(canvas): 굵은 표시용 글꼴을 SVG처럼 굵게 그린다

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8025](https://github.com/edwardkim/rhwp/pull/8025) / semanticist21.
- 원 contributor head: `6ddffd1aeb4f538f5576f33a1f882bd13c0e40db`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8025_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

현재 연결한 실행 근거는 파일을 소비하지 않는 단위 계약이다. 함수의 반환·경계 검증과 코드 CI 결과를 기록한다. 다른 파일의 PNG를 그 단위 검사의 실물 문서 증거로 붙이지 않는다.

## 구현과 회귀의 검증 범위

CanvasTextFont::for_positioned_text는 style.bold만 보지 않고 is_visually_bold를 사용한다. 실제 faux-bold 획을 이미 적용하는 face는 굵기 descriptor를 중복 적용하지 않는 조건도 유지한다. HY헤드라인M·HY견고딕·한양견고딕·HYGothic-Extra와 일반 함초롬바탕의 적용·비적용을 같은 descriptor 경로에서 검사한다.

tests/cases/canvas_heavy_display_face_bold.rs. Canvas 실제 출력 증거는 해당 글꼴·producer의 Print 대조와 별도로 연결해야 하며 descriptor 검사만으로 수용하지 않는다.


실제 회귀 source:
- [tests/cases/canvas_heavy_display_face_bold.rs](../../../tests/cases/canvas_heavy_display_face_bold.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8025#issuecomment-6440890024)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
