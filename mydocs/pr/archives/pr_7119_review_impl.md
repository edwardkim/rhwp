---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7119 후속 self-review 순서

1,000줄 초과 혼합 변경의 검토 순서를 고정한다. 새 제품 구현 계획이 아니며 현재 push 승인으로
GitHub approve·merge까지 승인된 것으로 해석하지 않는다.

1. 최종 게시 head의 CI/CodeQL/Render Diff/required check를 정확한 SHA에서 확인한다.
   로컬 통과를 원격 성공으로 대체하지 않는다.
2. 승인된 정식 self-review에서 snapshot 수명·세대·한도, 측정/paint 공유, 결합열 보호,
   portable 출력 복원, 제한 복구를 source와 회귀 증거에 대조한다.
   [review 준수 표](pr_7119_review.md)를 확정하고 검출 문제와 미지원 범위를 구분한다.
3. 수정이 필요하면 변경/검증 영향을 먼저 보고한다. 문제가 없으면 작업지시자에게 병합 승인을 요청한다.
   review-only 후행 기록에는 제품 변경을 섞지 않으며 최신 base 전진만으로 반복 merge하지 않는다.

현재 소스 보존점은 `d3a189d5d`; 모든 원격 작업은 `task_m100_7084` PR branch에 한정한다.
병합 전 중단 시 devel은 변경되지 않으며 WIP·전용 review worktree·공유 target을 임의 삭제하지 않는다.

## 2026-09-14 실행 결과

- 1번 완료: 원격 `6434189bc`의 CI·CodeQL·Render Diff·Adapter·Proptest 및 GHAS/Policy 성공 확인.
- 2번 완료: 측정 snapshot→조판/문자 위치→Canvas paint, 세대 검증·결합열 보호·portable 복원·복구
  경로를 직접 대조했다. 검토 범위의 병합 차단 결함은 발견하지 못했고 review 판정은 승인이다.
- 3번 대기: review 기록은 먼저 로컬에 보존한다. 후행 기록 push 승인 후 새 head의 CI를 확인하고
  별도 병합 승인을 받는다. GitHub review/comment, merge, issue close는 하지 않았다.
