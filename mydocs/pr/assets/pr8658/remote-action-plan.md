# PR #8658 원격 처리안 — 승인·수용 완료

최초 계획은 로컬 승인과 원격 승인을 구분했다. 이후 작업지시자가 이 원격 처리안과 #500 보안 분류를 명시 승인해 수용 PR #8694를 병합했다. 아래는 승인받은 순서를 보존하며, [실제 실행 결과](remote-completion.md)에서 완료·남은 후속 단계를 구분한다.

- 원 PR: [#8658](https://github.com/edwardkim/rhwp/pull/8658), 원 head `218c8856ad0a338fd1820c43e62b40dbcdcb86bb`.
- 원 fork: `kidsnote/rhwp`, branch `task/8576-cell-square-no-overlap`. API의 현재 일반 저장소 권한은 `push=false`이고 이번에 push를 시도하지 않았다. 앞선 같은 fork의 권한 제한과 원 기여 SHA 보존을 고려해 upstream 수용 PR 경로를 제안한다.
- 제출 branch: `review/pr-8658-cell-square-no-overlap`, upstream에 같은 새 branch를 만들고 `devel` 대상 Open PR로 제출한다. 원 기여 commit을 변경·squash하지 않는다.
- 제품 code `19de2f3b54`와 회귀 code `08cd5ab833`에 최종 검토 기록·실제 전쪽 TSV·대표 PNG를 더한 정확한 제출 head를 고정한다. 필수 로컬 검사 17개는 모두 통과했다. 최종 제출 head를 고정하고 증적을 연결한다.

## 승인받을 순서

1. 고정한 제출 head와 최신 `upstream/devel`, 본인 열린 PR·CI, 원 PR head를 다시 조회한다. 기존 보정을 포함한 실제 merge-tree와 원 기여 head의 조상 관계를 확인한다. 새 base가 생기면 해당 절차의 기능 보존·재검증을 완료한 뒤 push한다.
2. upstream 새 branch에 push하고 준비한 본문으로 Open 수용 PR을 생성한다. 원 PR과 연결 이슈를 명시하고 공개 증적은 자산이 실제 포함된 정확한 head SHA로 고정한다.
3. 해당 head의 GitHub 필수 CI와 최종 정책 집계를 확인하고 self-review를 기록한다. 성공한 동일 code head 로컬 검사를 반복하지 않는다. workflow 취소·재실행이나 CI 설정 변경이 필요하면 운영 규약상 별도 승인 범위로 다룬다.
4. merge 직전 head·필수 checks·최신 base·mergeable을 다시 확인한다. 원 기여를 보존하는 merge commit 방식과 정확한 `--match-head-commit`을 사용한다.
5. 실제 merge SHA와 원 [#8658](https://github.com/edwardkim/rhwp/pull/8658)의 간접 MERGED 반영을 조회한다. 병합 뒤 검증 CI를 새로 시작하거나 재실행하지 않고 `Refresh nextest target duration data`의 성공 또는 갱신 보류를 확인한다. 개정된 [post-merge 순서](../../../manual/pr_review/post_merge.md#7-필수-실행-순서)에 따라 review archive·오늘할일·확정된 merge SHA를 운영 기록으로 반영하고, 기본 devel을 최신 `upstream/devel`로 fast-forward한다. 직접 반영 범위를 넘으면 정본의 후속 기록 PR 경로를 사용한다.
6. 기록 반영·devel 동기화 뒤 원 PR의 완료 상태와 [#8576](https://github.com/edwardkim/rhwp/issues/8576)의 해결 범위를 재확인한다. 원 PR에 자동 MERGED가 반영되지 않았으면 실제 통합 수용 사실을 명시해 완료 처리한다. 연결 이슈는 완료 반영과 해당 merge·검증 근거의 후속 댓글을 남긴다. 범위 밖의 기존 [#8579](https://github.com/edwardkim/rhwp/issues/8579)는 닫지 않는다.
7. [검토 문서의 댓글 계획](../../archives/pr_8658_review.md#merge-후-contributor-pr-comment-계획)에 따라 원 주장 3입력의 전 4쪽 review·overlay와 실제 지표·정본 링크를 넣은 감사 댓글을 게시한다. 실제 merge SHA의 raw 이미지 형식·bytes·해시를 먼저 확인하고 게시 후 본문과 이미지 표시를 검증한다.
8. local·remote 검토 branch, worktree와 검토 전용 target을 정본의 실제 사용 판정 뒤 정리한다. 종료 시 루트 `output/` 전체와 사용이 끝난 기본 `target/pr-review/debug`를 삭제하고 잔여 여부를 확인한다. 현재와 무관한 worktree/WIP와 원 fork branch는 보존한다.

원격 작업은 사용자 승인 뒤 실행했다. 운영 기록 반영·최종 devel 동기화 뒤 이슈/PR 댓글과 실제 화면 확인을 완료했다. 마지막 종료 정리는 사용 판정과 삭제 로그를 남겨 완료한다.
