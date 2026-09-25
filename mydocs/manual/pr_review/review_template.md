---
kind: guide
status: active
canonical: mydocs/manual/pr_review/intake_and_review.md
last_verified: 2026-09-24
---

# PR #N 리뷰 — 변경 요약

## 최종 판정

**머지 보류 — 검증 진행 중.** 완료되지 않은 검증과 해제 조건을 한 문장으로 쓴다.
검증이 끝나면 [공통 판정 용어](../pr_review_workflow.md#11-최종-판정-용어와-원격-조치의-분리)의
`승인`, `머지 보류`, `메인터너 보정 후 수용 가능` 중 정확히 하나로 고친다. 원 head와 보정
head를 혼동하지 않는다. 최신 head CI와 mergeability 같은 merge 전 조건은 판정 아래에 구분한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | #N / 작성자 / devel |
| 원 head·검토 후보 | 두 SHA와 역할 |
| 관련 이슈 | 링크와 닫기 또는 참조 여부 |
| reviewer·작성 시점 상태 | reviewer, draft·mergeability·CI 참고값 |

## 변경과 검토 범위

핵심 동작, 영향받는 입력·호출 경로, 범위 밖 변경과 의존 PR을 적는다.

## 검증 입력과 결과

실제 사용한 HWP/HWPX·한컴 기준 PDF의 저장소 경로, 출처·SHA-256·검토 commit 포함
여부를 적는다. 실행한 명령과 대상 head·통과/실패/미실행 범위를 구분한다. 조판 원칙,
focused·전체 회귀·lint·Native/WASM·시각 검증 중 적용되는 것을 기록한다.

## 시각 증적과 남은 차이

입력·기준 PDF·쪽 대응, Native/fresh WASM review와 overlay의 안정 경로, 실제 지표와
사람이 직접 판독한 결론을 쓴다. 예외는 정본의 글꼴 증거 요건에 맞춰 별도로 증명한다.

## Merge 후 contributor PR comment 계획

시각 검증을 판정에 사용했다면 최종 merge SHA와 CI URL, Visual Sweep 정본 링크,
merge SHA 고정 raw image URL, 남은 차이, `--body-file` 게시와 API 재조회를 계획한다.
기여자에게 게시할 문안은 한국어 존댓말로 준비한다. 메인터너 보정이 있으면 원 기여가 해결한
문제와 추가 보정이 필요했던 이유·범위를 구분해 설명하고 반말이나 책임 전가 표현을 쓰지 않는다.

이 파일은 작성 예시다. 각 review에 해당하지 않는 절은 이유를 적고, 실제 검증 전에는
예시 문장을 완료 사실로 옮기지 않는다. 필수 기록과 실행 순서는
[PR 접수와 리뷰 기록](intake_and_review.md)이 정한다.
