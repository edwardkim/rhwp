# 원격 처리 승인 대상 초안

현재는 로컬 검토·검증만 수행했다. 아래는 작업지시자 승인 뒤 실행할 범위다.

1. 변경 없는 원 PR #8457 → #8449 → #8451 → #8453 → #8450 → #8452 → #8454 → #8455를 각각 처리한다. 매번 source head·required checks·최신 devel의 clean merge-tree를 확인하고 head SHA를 고정해 원 PR을 merge한다.
2. #8456의 보정·정식 증적 source branch를 실제 최신 devel과 정렬한다. 검증한 누적 e580 graph 대비 runtime·Cargo dependency·테스트·workflow가 같은지 확인한다. 8건의 코드가 모두 base에 들어온 뒤 원 PR diff에 Skia 갱신·최소 보정·허용된 검토 증적만 남는지 확인하고 한 번 normal push한다.
3. 새 #8456 head의 실제 GitHub CI와 CI Impact Policy 성공, self-review를 확인한 뒤 같은 head를 원 PR에서 merge한다. 기존 실패 head를 bypass하거나 성공한 다른 PR의 CI를 임의 재실행하지 않는다.
4. merge SHA·CI를 원 PR별 검토 기록에 반영하고 archive한다. GitHub 후속 댓글은 해당 draft의 merge/CI/record placeholder를 실제 검증한 값으로 채운 뒤 --body-file로 게시하고 API로 본문을 다시 확인한다. 8건의 merge 시점에는 아직 asset이 없을 수 있어 댓글은 #8456의 증적 merge 뒤 한 번씩 게시한다.
5. 기본 devel을 fast-forward 동기화하고 owned review worktree·branch·root output과 사용이 끝난 target/pr-review/debug를 정리한다. 사용자 기존 Studio WASM·Vite 7700·pr-8296-seal-pua worktree는 보존한다.

수동 workflow 취소·재실행·보안 metadata 변경은 포함하지 않는다. 현재 원격 source head는 모두 원 Dependabot SHA다.

## 원 PR별 게시 초안

실제 게시 전 merge·CI·record·evidence placeholder를 확정값으로 치환한다. 증적 SHA는 자료가 포함된 후속 Skia merge를 사용하며, 앞선 8건의 merge SHA와 구분한다.

| PR | 댓글 초안 |
| --- | --- |
| #8449 | [게시 초안](comment-drafts/pr-8449-merge-comment-draft.md) |
| #8450 | [게시 초안](comment-drafts/pr-8450-merge-comment-draft.md) |
| #8451 | [게시 초안](comment-drafts/pr-8451-merge-comment-draft.md) |
| #8452 | [게시 초안](comment-drafts/pr-8452-merge-comment-draft.md) |
| #8453 | [게시 초안](comment-drafts/pr-8453-merge-comment-draft.md) |
| #8454 | [게시 초안](comment-drafts/pr-8454-merge-comment-draft.md) |
| #8455 | [게시 초안](comment-drafts/pr-8455-merge-comment-draft.md) |
| #8456 | [게시 초안](comment-drafts/pr-8456-merge-comment-draft.md) |
| #8457 | [게시 초안](comment-drafts/pr-8457-merge-comment-draft.md) |
