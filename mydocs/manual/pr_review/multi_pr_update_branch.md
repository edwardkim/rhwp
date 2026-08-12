---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
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
   목록 조회와 force-cancel 요청 사이에 대상 run이 스스로 끝나 GitHub가 409을 반환할 수 있다.
   이 경우 reaper는 대상 run을 다시 조회해 `completed`임을 확인한 경우에만 정상 경과로 기록한다.
   409 자체를 무조건 무시하거나, 여전히 active인 run을 성공으로 처리하면 안 된다.
3. reaper 실패·미실행 시에는 `scripts/cancel_stale_pr_runs.sh <PR번호>`로 정리한다(#3508 —
   현재 head 확인 → 이전 SHA active run 나열 → force-cancel → 완료 재확인을 한 명령으로,
   `--dry-run`은 목록만). 이 경로도 **일반 `gh run cancel`을 먼저 시도하지 않고** force-cancel
   API를 쓴다. script를 쓸 수 없는 환경에서만 아래 수동 API 절차를 따른다.
4. 수동 취소 뒤에도 완료 상태와 `cancelled` 결론을 재확인한다.

### 한 기여자의 대량 PR 사후 정리 — `--author` 모드 (#4025)

에이전트 오작동 등으로 **한 기여자의 PR 이 대량 생성돼 러너가 포화**된 경우
(2026-08-04 실측: fork PR 69건, 진행 15 + 대기 39), 위 PR 단위 절차로는 풀리지 않는다
— PR 모드는 **최신 head run 보존**이 설계 목적인데 사고에서는 그 최신 run 들이 러너를
먹기 때문이다.

```
scripts/cancel_stale_pr_runs.sh --author <로그인> [--dry-run]
```

- 해당 author 의 **열린 PR head 브랜치 목록**과 대조해 목록 밖(devel·main·타 기여자)은
  **취소하지 않는다.** 8/4 실측에서 이 대조가 devel CI 를 지켰다.
- `gh run cancel` 1차 → 잔여분 `force-cancel` 2차. **1차만으로는 부족하다**
  (8/4 실측: 1차 후 44건 잔여 — 취소 반영 지연·신규 큐잉).
- 요약에 `대상 N건 → 잔여 M건 (보호 K건)` 을 출력한다. 잔여가 있으면 재실행한다.
- 조회는 `status=in_progress|queued` 로 좁힌다(전량 `--paginate` 는 수 분 소요 실측).

**선행 판단**: 정리 전에 원인을 확인한다. 기여자 실수인지 에이전트 오작동인지에 따라
후속 커뮤니케이션 톤이 달라진다 — 8/4 에는 이 확인이 늦어 close 코멘트를 정정해야 했다.
평시 `approval_policy` 는 `first_time_contributors` 를 유지하고
(`all_external_contributors` 강화는 릴리즈 시에만), 사고는 이 도구로 사후 대응한다.

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

## 2.6 검토 중 기준선 갱신

검토를 시작한 뒤 contributor가 새 commit을 push하거나, feature branch에 최신 `devel`을 merge하면 이전
review branch와 검증 결과는 자동으로 최신 head에 귀속되지 않는다. 특히 최신 `devel` merge commit만
그래프 맨 위에 보이는 경우에도, PR의 기능 commit은 그 아래 history에 남아 있으므로 local checkout을
최신 PR head와 동일하게 만든 뒤 PR 고유 diff를 다시 분리한다.

1. 진행 중인 Cargo, wasm-pack, npm 빌드가 있으면 먼저 종료 결과를 수집하거나 더 이상 최신 head에
   해당하지 않는 실행임을 기록하고 중단한다. 같은 target/cache를 공유하는 새 빌드를 겹쳐 실행하지 않는다.
2. 현재 review branch, worktree 상태, 기존 검증 head SHA를 기록한다. review 문서 초안처럼 보존해야 할
   uncommitted 파일이 있으면 그대로 유지하되, 소스 변경과 섞이지 않게 구분한다.
3. `upstream/devel`과 PR head를 모두 fetch한다. `devel`이 fast-forward 가능한지 확인한 뒤 최신
   `upstream/devel`로 동기화한다. local `devel`에 local-only commit이나 미확인 변경이 있으면 임의 rebase,
   reset, branch 강제 이동을 하지 말고 상태를 보고한다.
4. 기존 visibility review branch가 최신 PR head의 조상이면 그 branch를 `git merge --ff-only`로 갱신한다.
   이 방식은 VS Code graph에서 `devel` 기준선, feature commit, contributor의 merge commit을 모두 보존한다.
   contributor가 rebase 또는 force-push하여 fast-forward할 수 없으면 기존 branch를 PR head로 보정하지
   않는다. 이전 SHA와 새 SHA, 변경 이유를 기록하고 최신 head 전용 review branch를 새로 만든다.
5. 새 branch와 `upstream/devel`의 관계, PR 고유 변경 범위, whitespace 오류를 재확인한다. `devel` merge
   자체를 contributor 기능 변경으로 오인하지 않도록 diff 기준은 항상 최신 `upstream/devel...<review-branch>`다.
6. 새 head에서 바뀐 파일과 의존성 갱신 범위에 맞춰 focused test, 통합 test, package install/build, fixture
   검증을 재실행한다. 새 head가 단순 `devel` merge여도 lockfile 또는 shared runtime이 바뀌면 그 영향을
   받는 검증은 이전 결과를 재사용하지 않는다.
7. review 문서에는 이전 head 결과를 역사 기록으로만 남기고, 최종 판정에는 새 head SHA와 새 실행 결과만
   쓴다. CI도 최신 head의 required check만 수용하며 이전 SHA run은 2.5절의 stale-run 절차로 정리한다.

~~~bash
# 모든 명령은 clean한 review worktree에서 순차 실행한다.
git fetch upstream devel
git fetch upstream pull/N/head:refs/remotes/upstream/prN-head
git switch devel
git merge --ff-only upstream/devel
git switch review/<contributor>-<YYYYMMDD>
git merge --ff-only upstream/prN-head
git merge-base --is-ancestor upstream/devel HEAD
git diff --stat upstream/devel...HEAD
git diff --check upstream/devel...HEAD
git rev-parse HEAD
~~~

위 예시의 `git merge --ff-only`가 실패하면 마지막 두 `switch`/`merge`를 억지로 재시도하거나 history를
재작성하지 않는다. 새 PR head가 force-push된 경우에는 최초 검토 branch를 보존하고, 새 head에서 만든
별도 visibility review branch와 전용 target으로 검증을 다시 시작한다.

### 2.6.1 외부 PR review 기록의 source head 정렬

collaborator가 외부 contributor PR의 source branch에 archive review·오늘할일·허용된 증적을 직접 push할
경우에는, contributor가 검토 중 새 commit을 push한 시점부터 최종 review 묶음을 만들기 전까지 아래
정렬 규칙을 적용한다. 목적은 contributor code head 뒤에 collaborator 기록만 이어지는 단순 history를
만들어 review-only fast-pass의 후보 탐색이 code 변경을 숨기지 않으면서 동작하게 하는 것이다.

1. 이전 source SHA와 새로 fetch한 source SHA를 각각 기록한다. 원격 PR head와 `git ls-remote`가 새 SHA와
   일치하는지 확인하기 전에는 archive review, 오늘할일, 증적 commit을 만들거나 push하지 않는다.
2. 이전 source 위에 만든 collaborator commit이 아직 contributor source branch에 push되지 않았고,
   `old_source..HEAD`가 collaborator 소유의 single-parent commit만으로 구성됐으면 같은 visibility branch에서
   그 commit만 새 source 위로 replay한다. contributor commit은 rebase, amend, reset, force-push하지 않는다.

   ~~~bash
   old_source_sha=<이전 fetch source SHA>
   new_source_sha=<새 fetch source SHA>
   git log --format='%H %P %s' "$old_source_sha..HEAD"
   git rev-list --merges "$old_source_sha..HEAD"
   # merge 출력이 없고 위 범위가 collaborator의 미공개 commit일 때만 실행
   git rebase --onto "$new_source_sha" "$old_source_sha"
   git merge-base --is-ancestor "$new_source_sha" HEAD
   ~~~

3. collaborator commit 위로 `git merge <new-contributor-head>`를 실행하지 않는다. 그 merge는 대개
   current `devel`을 직접 parent로 갖지 않아 fast-pass가 `mydocs-only-merge-not-reusable`로 거부한다.
   current `devel`을 source에 이미 병합한 예외는 [review-only fast-pass](review_only_fast_pass.md)의
   자동 merge tree 일치 조건을 충족할 때만 제한적으로 재사용할 수 있지만, 이 예외도 새 merge 생성을
   정당화하지 않는다. VS Code 그래프의 가시성은 같은 branch에서 contributor head를 조상으로 두는 선형
   history로 유지한다.
4. collaborator commit이 이미 contributor source branch에 push됐거나, replay 범위에 merge·미확인 commit이
   있으면 history를 억지로 정리하거나 contributor branch를 force-push하지 않는다. 최신 source에서 full CI를
   받거나, 작업지시자의 명시 승인을 받아 별도 처리 경로를 선택한다.
5. 새 source가 local `HEAD`의 조상이고 `new_source..HEAD`에 collaborator commit만 남았음을 확인한 뒤에
   archive review·오늘할일·증적을 하나의 최종 review 묶음으로 commit한다. 그 직후 9.3.0의 LFS 판독과
   remote SHA 재확인을 수행한다.

## 4.2.1 여러 PR 체리픽 누적 검토

여러 PR이 같은 영역을 단계적으로 수정하고 오래된 순서로 merge해야 하면, upstream/devel 기준의 별도
검토 branch에서 기능·문서 commit만 누적 cherry-pick할 수 있다.

- 순서는 오래된 PR 번호 또는 작업지시자가 지정한 순서를 따른다.
- PR 안의 Merge branch devel commit은 검토 체리픽에서 제외한다.
- 누적 branch는 충돌·테스트·시각 검증용 임시 branch다. review 문서는 원 PR 번호별로 작성한다.
- 각 review 문서에 체리픽 순서, 적용 SHA, conflict, 선행 PR 의존성을 그 PR 기준으로 적는다.
- 원 PR별 archive 검토 기록과 필요한 오늘할일은 통합 branch의 같은 PR diff에 둔다. 통합 PR 번호를 위한
  별도 review 문서를 만들거나, 체리픽 통합 결과의 검토 기록만 담은 별도 docs-only PR을 만들지 않는다.
  문서 보완은 동일 통합 PR의 code candidate CI 성공 뒤 trailing docs-only commit으로 처리한다.
- 여러 PR을 한꺼번에 검증했어도 merge 전에는 각 PR의 최신 head, mergeable, required check를 개별 재확인한다.

fetch·visibility branch·Cargo 검증은 [로컬 검증](local_validation.md)을, 렌더 영향 증적은
[시각·fixture 증적](visual_fixture_evidence.md)을 함께 따른다.
