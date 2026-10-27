# PR #8029 리뷰 — fix(font-metrics): PostScript 굵기 접미사가 붙은 글꼴 이름도 family 메트릭을 찾는다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8029](https://github.com/edwardkim/rhwp/pull/8029) / semanticist21.
- 원 contributor head: `2ca627439e936368cc934e90faa859f8c2998594`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8029_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **중앙선관위 선거 안내 포스터**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

현재 연결한 실행 근거는 파일을 소비하지 않는 단위 계약이다. 함수의 반환·경계 검증과 코드 CI 결과를 기록한다. 다른 파일의 PNG를 그 단위 검사의 실물 문서 증거로 붙이지 않는다.

## 구현과 회귀의 검증 범위

정확한 이름/별칭으로 metric_index를 먼저 조회하고, 실패한 경우에만 마지막 하이픈의 Regular·Medium·Bold를 분리한다. Bold 접미사는 기존 bold 요청과 OR로 합성하며 family에도 같은 별칭 규칙을 적용한다. 등록된 완전한 face를 우선하므로 정상 exact lookup을 덮지 않고, 알 수 없는 family나 Light 같은 미지원 접미사를 임의로 추정하지 않는다.

tests/cases/metric_postscript_weight_suffix.rs: Regular/Medium 동일 metric, Bold의 실제 bold metric, SourceHanSansK 별칭, 미등록 family 및 Light 거절.


실제 회귀 source:
- [tests/cases/metric_postscript_weight_suffix.rs](../../../tests/cases/metric_postscript_weight_suffix.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8029#issuecomment-6440892871)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
