# PR #8058 리뷰 — fix(canvas): 글리프 맞춤이 글자를 늘리거나 기호 잉크를 지나치게 줄이지 않는다

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8058](https://github.com/edwardkim/rhwp/pull/8058) / semanticist21.
- 원 contributor head: `a5cc4f15f32426a8fe9232650a4c125b3f434f17`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8058_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

현재 연결한 실행 근거는 파일을 소비하지 않는 단위 계약이다. 함수의 반환·경계 검증과 코드 CI 결과를 기록한다. 다른 파일의 PNG를 그 단위 검사의 실물 문서 증거로 붙이지 않는다.

## 구현과 회귀의 검증 범위

canvas_cluster_fit은 양수 배분 간격을 실제 glyph fit 폭과 분리한다. 좁은 ASCII를 늘리지 않고 가운데 놓으며, box가 넓어도 잉크가 advance 안에 들면 기호를 누르지 않는다. 한글/한자와 fixed_glyph_advances의 원본 점유 계약은 별도 분기다. halfwidth punctuation과 Canvas/Skia의 동일 helper 소비를 확인하며 임의의 허용 배율로 시각 미달을 감추지 않는다.


실제 회귀 source:

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8058#issuecomment-6440927420)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
