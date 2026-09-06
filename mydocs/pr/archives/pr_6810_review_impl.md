# PR #6810 처리 계획 — 공개 기여 검증 안내

- Issue: [#6791](https://github.com/edwardkim/rhwp/issues/6791)
- PR: [#6810](https://github.com/edwardkim/rhwp/pull/6810)
- 경로: collaborator self; [review](pr_6810_review.md)
- 원격 push·Open PR 생성 승인: 2026-09-06 사용자 “진행해줘”.
- merge·이슈 close·원격 branch 삭제는 이번 승인 범위에 포함되지 않는다.

## 완료한 commit과 검증

| Commit | 내용 |
| --- | --- |
| b7f931acf | 수행계획 |
| a2c00f16d | 수행계획 승인 |
| 0f76777df | 구현계획 |
| 16e2ea171 | 구현계획·Stage 1 승인 |
| eb046527a | 범위별 검증·frontend 안내 |
| c210cb7e7 | Stage 2 승인 |
| b5a33bfe3 | 공개 Rust worktree·제출 절차; 실제 검증한 문서 내용 |
| 16c41bc87 | Stage 3 승인 |
| 05fa7dc72 | 실검증 결과·PR 본문 준비 |
| 15f5a5574 | 최신 devel 병합; 공개 파일 내용 불변, 최초 원격 제출 |

## 실행 순서와 승인 경계

1. **완료**: 새 clean worktree에서 공개 prepare·fmt·manifest·SHA·clean을 검증하고 결과를 commit했다.
2. **완료**: 사용자 승인 후 upstream task branch push와 Open PR 채번을 수행했다. 본문 API 재조회가
   준비한 UTF-8 본문과 일치했고 담당자·마일스톤을 확인했다.
3. **현재 기록 단계**: 번호 확정 후 review·이 문서·기존 오늘할일 및 계획·보고 상태를 같은 source
   branch에 기록한다. 링크·diff를 검사하고 후속 commit을 push한다. 공개 문서는 변경하지 않는다.
4. **남은 외부 게이트**: 최신 head의 checks·mergeability·base 정합성을 확인한다. CI 실패가 나오면
   원인을 먼저 구분하며 원인과 무관한 코드·정책 수정은 이 문서 PR에 섞지 않는다.
5. **별도 사용자 판단**: 1,000줄 초과 PR의 문서 검토·절차 실행 증거를 바탕으로 merge 승인을 받는다.
   승인 없이 merge하거나 branch protection을 우회하지 않는다.
6. **별도 후속 처리**: merge와 후속 처리 승인 후 post_merge 절차로 상태·증적·#6791 close 여부를
   확인하고 이번 작업 전용 clean worktree를 정리한다. 원격 branch 삭제는 별도 승인이다.

## 보존과 되돌림 범위

현재는 로컬 검증 로그·전용 worktree를 보존한다. 다른 작업의 checkout·target·기록은 삭제하지 않는다.
수정이 필요하면 이번 PR branch의 추가 commit으로 보정하고 영향 검증을 수행한다. 원 PR #6786에 대한
변경이나 외부 comment는 하지 않는다. 병합 후 되돌림이 필요하면 별도 issue·revert PR로 검토한다.
