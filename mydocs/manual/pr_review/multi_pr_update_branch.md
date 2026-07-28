---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# 다수 PR과 update branch 처리

이 가이드는 한 contributor의 대량 PR, 다수 PR 누적 검토, update branch 뒤 stale CI 정리에만 적용한다.
각 원 PR의 reviewer assign, review 문서, CI 판정은 [PR 접수와 리뷰 기록](intake_and_review.md)처럼
PR 번호별로 분리한다.

## 2.0 대량 PR 유입 사전 분류

한 contributor의 열린 PR이 많으면 개별 review 전에 통합 그룹과 변경 축을 사실로만 수집한다.

~~~bash
scripts/pr_triage.sh <author>
scripts/pr_triage.sh <author> --list
~~~

- 기본 조회 상한은 500이다. 더 크면 RHWP_PR_LIMIT, 다른 저장소면 RHWP_REPO를 지정한다.
- 축별 합계와 열린 PR 수를 대조한다. gh pr list 기본 limit이나 jq 무매치로 행이 빠졌다고 처리 완료로
  오판하지 않는다.
- 이 도구는 충돌 목록과 통합 그룹 후보를 수집할 뿐, merge·close·rebase 결정을 대신하지 않는다.

## 2.5 update branch 뒤 이전 SHA CI 강제 취소

contributor 또는 maintainer가 Update branch를 수행해 PR head가 바뀌면, 이전 SHA run이 최신 required check와
섞여 보일 수 있다. 최신 head의 CI는 절대 취소하지 않는다.

1. `devel` 대상 PR에서는 `Cancel stale PR runs` reaper가 `synchronize` event로 시작했는지,
   최신 head SHA와 함께 확인한다. 이 workflow는 PR source를 checkout·실행하지 않고, 같은
   PR head(head_repository+head_branch)의 이전 SHA `pull_request` run만 force-cancel한다.
   이중 트리거(#3508)다 — same-repo PR은 `pull_request` 경로로 즉시, **fork PR은
   `pull_request_target` 경로인데 이 트리거는 default 브랜치(main)의 파일 기준으로
   등록되므로(#3503 실측) 워크플로가 릴리즈로 main에 실린 뒤부터 발동한다.** 그 전의
   fork PR은 아래 3의 script 폴백을 쓴다.
2. reaper가 성공했다면 이전 SHA run이 `completed/cancelled`가 되었고 최신 head run이 시작됐는지 확인한다.
3. reaper 실패·미실행 시에는 `scripts/cancel_stale_pr_runs.sh <PR번호>`로 정리한다(#3508 —
   현재 head 확인 → 이전 SHA active run 나열 → force-cancel → 완료 재확인을 한 명령으로,
   `--dry-run`은 목록만). 이 경로도 **일반 `gh run cancel`을 먼저 시도하지 않고** force-cancel
   API를 쓴다. script를 쓸 수 없는 환경에서만 아래 수동 API 절차를 따른다.
4. 수동 취소 뒤에도 완료 상태와 `cancelled` 결론을 재확인한다.

러너 구성 전환 등으로 배정 가능한 label이 사라진 run은 `queued`에 고착될 수 있다. 이 run은 일반 cancel이
끝나지 않고 같은 concurrency group을 계속 점유해, 후속 run이 job을 하나도 시작하지 못한 `pending`으로
연쇄 고착될 수 있다. 새 run이 `pending`이면 최신 run만 재실행하지 말고 같은 PR·workflow의 이전
`queued`/`pending`/`in_progress` run부터 확인한다. 정확한 stale SHA를 확인한 직후 아래 force-cancel API를
사용하고, 이전 run이 실제 `completed/cancelled`가 된 뒤 후속 run 상태를 다시 확인한다.

~~~bash
gh pr view N --repo edwardkim/rhwp --json headRefOid
gh run list --repo edwardkim/rhwp --commit <old-sha> \
  --json databaseId,workflowName,status,conclusion,headSha,url --limit 20
gh api --method POST repos/edwardkim/rhwp/actions/runs/<run-id>/force-cancel
~~~

자동·수동 어느 경로든 force-cancel 대상 SHA, run URL, 완료 상태를 review 문서 또는 작업 기록에 남긴다.
stale run 정리는 최신 head의 새 CI를 기다리는 일과 병렬로 할 수 있지만, 대상 SHA 검증과 force-cancel
API 호출은 순차로 한다.

PR close/reopen만으로는 GitHub의 merge ref가 항상 재계산된다고 가정하지 않는다. 고착 run을 정리한 뒤에도
merge ref 또는 required check가 갱신되지 않으면, head SHA가 바뀌는 push의 `synchronize` 이벤트로
재계산한다.

## 4.2.1 여러 PR 체리픽 누적 검토

여러 PR이 같은 영역을 단계적으로 수정하고 오래된 순서로 merge해야 하면, upstream/devel 기준의 별도
검토 branch에서 기능·문서 commit만 누적 cherry-pick할 수 있다.

- 순서는 오래된 PR 번호 또는 작업지시자가 지정한 순서를 따른다.
- PR 안의 Merge branch devel commit은 검토 체리픽에서 제외한다.
- 누적 branch는 충돌·테스트·시각 검증용 임시 branch다. review 문서는 원 PR 번호별로 작성한다.
- 각 review 문서에 체리픽 순서, 적용 SHA, conflict, 선행 PR 의존성을 그 PR 기준으로 적는다.
- 여러 PR을 한꺼번에 검증했어도 merge 전에는 각 PR의 최신 head, mergeable, required check를 개별 재확인한다.

fetch·visibility branch·Cargo 검증은 [로컬 검증](local_validation.md)을, 렌더 영향 증적은
[시각·fixture 증적](visual_fixture_evidence.md)을 함께 따른다.
