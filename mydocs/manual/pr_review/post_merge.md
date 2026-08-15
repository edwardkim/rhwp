---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# Merge 후속 처리

이 가이드는 원 코드 PR 또는 후속 기록 PR이 merge된 뒤의 종료 절차다. 과거 review 기록의 참조 호환을 위해
7.x 절 번호를 유지하지만, 이 파일의 상세 절은 **실제 실행 순서**로 배치한다. comment 초안이나 read-only
확인은 병렬로 준비할 수 있지만, close·comment 게시·branch 삭제는 선행 gate를 앞지르지 않는다.

## 7. 필수 실행 순서

1. 원 코드 PR merge 완료와 merge SHA를 확인한다.
2. review 문서·asset·오늘할일의 후속 반영 필요 여부를 결정한다.
3. archive 이동과 오늘할일을 준비하고, maintainer 직접 반영 또는 후속 기록 PR을 완료한다.
4. 최종 devel을 upstream/devel로 fast-forward한다.
5. 관련 issue close 상태를 확인하고 필요한 issue comment를 게시한다.
6. 원 PR 또는 supersede PR에 review 결과 comment를 게시한다.
7. local·remote branch, worktree, 검토 전용 target을 정리하고 잔여 여부를 검증한다.

시각 asset URL을 comment에 쓰면 asset이 devel에 실제 존재한 뒤에만 게시한다. 이미 완료된 원 PR의 기록만
담는 별도 fast-pass PR은 5–6과 오늘할일 갱신을 반복하지 않고, devel sync와 cleanup만 수행한다.

작업지시자가 PR 병합과 `merge 후 후속 처리`를 함께 승인한 경우, 7번은 선택 보고가 아니라 완료 전 실행
게이트다. 해당 PR만을 위해 만든 clean한 local branch와 local worktree의 제거는 별도 승인 없이 이 단계에서
수행한다. 원격 branch 삭제나 기본 작업공간·공유 산출물·사용자 또는 다른 도구 소유 대상의 삭제는 포함하지
않는다.

## 7.5 renderer golden 선행조건

renderer 영향 PR의 golden 재생성은 원 PR merge 전에
[로컬 검증](local_validation.md)의 svg_snapshot 절차로 완료하는 것이 원칙이다. 이 단계에서 누락을 발견하면
merge를 진행하지 않고 원 PR head에 별도 commit으로 반영한 뒤 최신 CI를 확인한다. 이미 merge된 뒤 발견했으면
golden/baseline을 maintainer 운영 기록으로 직접 push하지 않고 전용 후속 PR을 만든다.

## 7.1 후속 문서 처리 결정

원 코드 PR merge SHA를 확인한 즉시 아래 중 하나를 상태 기록에 남긴다.

- **PR head에 이미 포함**: collaborator 경로에서 review·오늘할일·report가 함께 merge됐다. merge 뒤에만
  확정되는 SHA, issue close, supersede close 누락만 확인한다.
- **maintainer 직접 반영**: admin 또는 bypass maintainer가 archive review, 오늘할일, mydocs/pr/assets만
  devel에 한 운영 기록 commit으로 반영한다.
- **후속 기록 PR 필요**: collaborator이거나 직접 반영 범위를 넘는 경우, mydocs와 허용된 신규 기준
  sample/PDF만 포함하는 fast-pass 후보 PR로 반영한다.
- **추가 문서 불필요**: review/report/오늘할일에 merge 뒤 확정값 누락이 없다. 이 판단도 기록한다.

direct 반영은 source, test, workflow, golden/baseline, 기존 sample 수정, 신규 LFS 자료를 포함할 수 없다.
그 범위를 넘으면 후속 PR로 전환한다.

## 7.6 review 문서 archive 이동

maintainer 일반 경로의 active review 문서는 후속 기록을 반영하기 전에 archive로 이동한다.

~~~bash
mv mydocs/pr/pr_N_review.md mydocs/pr/archives/
mv mydocs/pr/pr_N_review_impl.md mydocs/pr/archives/
~~~

실제로 존재하는 파일만 이동한다. collaborator self-merge와 collaborator 매개 외부 PR은 처음부터 archive
경로를 쓰므로 이동하지 않는다. 후속 기록 PR에서도 archive 이동만을 위한 또 다른 PR을 만들지 않는다.

## 7.8 오늘할일 갱신

maintainer 일반 경로는 원 코드 PR merge 뒤 운영 기록을 준비할 때 오늘할일에 다음을 적는다.

- PR 번호·제목·작성자
- merge SHA
- 관련 issue close 여부
- 남은 후속 작업

maintainer 직접 반영이면 archive review·asset과 같은 commit에 포함한다. collaborator self-merge와
collaborator 매개 외부 PR의 오늘할일은 merge 전에 원 PR head에 이미 포함되어야 하므로 이 단계에서 새로
만들거나 갱신하지 않는다. 이미 완료된 원 PR의 별도 fast-pass 기록 PR도 오늘할일을 반복 갱신하지 않는다.

### 운영 기록 반영 완료

option M을 쓸 수 있는 maintainer는 작업지시자의 실제 push 승인 뒤 다음 순서로 운영 기록만 반영한다.

~~~bash
git fetch upstream devel
git switch devel
git merge --ff-only upstream/devel
git diff --check
git status --short
git add mydocs/pr/archives/pr_N_review.md \
  mydocs/pr/assets/pr_N_visual_review.png \
  mydocs/orders/YYYYMMDD.md
git diff --cached --name-status
git commit -m "docs: PR #N 검토 기록 보존"
git push upstream devel
~~~

존재하고 실제 변경된 경로만 git add에 넣는다. source, test, workflow, golden/baseline, 기존 sample,
신규 LFS 자료가 staged되면 commit하지 않고 후속 PR 경로로 전환한다.

후속 기록 PR을 쓰면 code·기존 sample·workflow를 섞지 않고 git diff --check, preflight, fast-pass 사유,
merge 가능 상태를 확인해 merge한다. 후속 기록 PR 자체가 완료 산출물이므로 다시 후속 기록 PR을 만들지 않는다.

## 7.2 devel sync

~~~bash
git fetch upstream devel
git switch devel
git merge --ff-only upstream/devel
~~~

devel이 diverge했으면 임의 rebase하지 않는다. current branch, uncommitted change, local-only commit을 확인해
보고하고 작업지시자 판단을 기다린다.

## 7.3 issue close와 후속 comment

default branch가 main이라 closes #N auto-close가 실패할 수 있다. 운영 기록 반영과 최종 devel sync 뒤에
issue 상태를 확인한다. closing keyword가 있으면 처리 지연을 고려해 2–3회 재조회한다.

~~~bash
for i in 1 2 3; do
  gh issue view N --repo edwardkim/rhwp --json state,closedAt,comments
  sleep 10
done
~~~

OPEN이면 작업지시자 승인 뒤 수동 close와 후속 comment를 남긴다.

여러 단락의 후속 기록은 [공통 본문 전송 규칙](../pr_review_workflow.md#34-github-markdown-본문-전송)에 따라
close와 comment를 분리하고 실제 줄바꿈을 담은 `--body-file`을 쓴다.

~~~bash
gh issue close N --repo edwardkim/rhwp
gh issue comment N --repo edwardkim/rhwp --body-file <issue-comment.md>
~~~

CLOSED여도 같은 merge commit·같은 검증 증적의 maintainer comment가 아직 없으면 다음을 담은 comment를 남긴다.

- merge PR과 merge commit
- GitHub Actions와 local 검증 요약
- 기준 자료·시각 asset link
- 남은 후속 작업 유무
- auto-close 상태 확인 사실

GitHub bot auto-close comment만으로 후속 기록이 완료된 것은 아니다. 같은 commit·같은 증적의 maintainer
comment가 있으면 중복 게시하지 않고 permalink를 상태 보고에 남긴다.

### 7.3.1 sub-issue 연동 부모 issue의 close

추적 성격의 부모 issue(discussion 발견 여러 건을 부모로 묶고 실작업을 sub-issue로 분리한 경우 등)는
마지막 sub-issue close 시점에 함께 close하는 것을 기본으로 한다. 단 다음을 모두 지킨다.

- 부모 귀속 산출물(가드 테스트 PR 등)이 모두 merge된 뒤에만 close한다. sub-issue close를 단일
  트리거로 삼지 않는다.
- sub-issue 연동은 close 시점 규칙일 뿐 절차 단축이 아니다. devel 반영 검증(`git branch --contains`)과
  작업지시자 승인 게이트는 동일하게 거친다. GitHub는 sub-issue 완료 시 부모를 자동 close하지 않으므로
  항상 수동 close다.
- close comment에 판정 경위를 명시한다. "버그 수정으로 닫힘"이 아닌 경우(오등록 판정, 이미 닫힌 축
  귀속, 가드 보강)는 그 성격과 귀속 커밋을 남겨 사후 검색이 성격을 오독하지 않게 한다.

첫 적용 사례: #3552(부모 — 재현 불가 판정, 가드 테스트 PR 귀속) / #3576(sub-issue — 실작업).
close 체크리스트: ① sub-issue close ② 가드 테스트 PR merge ③ 판정 경위 close comment
④ 작업지시자 승인.

## 7.4 contributor PR comment

원 PR에는 감사, merge 사실, 실제 검증 결과, 필요하면 후속 issue를 남긴다. issue·PR·comment는 평문 번호
대신 Markdown direct link로 쓴다.

시각 검증을 merge 판단 근거로 썼다면 실제 devel asset을 보이게 포함한다.

~~~markdown
검토 및 머지 완료했습니다. 감사합니다.

- CI: Build & Test, CodeQL, Render Diff의 최신 head 결과 확인
- 로컬 검증: 실제 실행한 focused/release-test/Native Skia 등
- visual sweep: pN, flagged=0/N, pixel match NN.NNNNN%

![PR N pN visual review](https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/<review>.png)
~~~

이미지 link만 쓰거나 output 임시 경로만 남기지 않는다. review 문서에 기록된 실제 수치·페이지·결론만
사용한다. 다음 visual PR의 재현 자료 요청은 원본 HWP/HWPX와 기대 출력 의도를 중심으로 적는다.

원 코드 PR의 최종 diff가 review-only fast-pass여도 merge commit, 허용된 파일 범위, preflight 성공과
heavy worker skip, final aggregate, issue 상태를 PR comment에 남긴다. 반면 완료된 원 PR의 기록만 담는
별도 fast-pass PR은 추가 contributor comment·issue close·오늘할일을 반복하지 않는다.

## 7.7 branch, worktree, 검토 전용 target 정리

성공 merge뿐 아니라 reject/close, supersede, review 중단, 후속 기록 fast-pass 완료도 최종 종료 gate다.
정리 또는 유지 사유를 확인하기 전에는 후속 처리 완료라고 보고하지 않는다.

이번 PR 또는 검토만을 위해 만든 별도 local worktree는 merge와 필수 후속 처리가 끝난 뒤 **제거가 기본**이다.
다음 작업의 편의를 위한 보존은 유지 사유가 아니다. 제거 전에는 clean 상태와 사용자·다른 도구의 소유 여부를
확인하며, 활성 작업 또는 작업지시자의 명시 보존 지시 때문에 제거하지 못하면 정확한 경로와 사유를 최종 상태에
기록한다. 기본 작업공간, 공유 `target/pr-review`, 사용자·다른 도구가 만든 worktree는 이 규칙의 삭제 대상이
아니다.

대상 worktree는 자기 자신을 제거할 수 없으므로, 정리 명령은 반드시 보존할 기본 작업공간 또는 다른 clean
worktree에서 실행한다. merge가 성공한 것만 확인하고 대상 worktree에 그대로 남아 "후속 처리 완료"로
보고해서는 안 된다.

먼저 정확한 대상 이름과 worktree를 확인한다.

~~~bash
gh pr view N --repo edwardkim/rhwp --json headRefName,headRepositoryOwner,headRepository
git branch --show-current
git worktree list --porcelain
~~~

PR fetch branch, merge simulation branch, review branch, docs-only/follow-up branch, 해당 worktree, collaborator가
원본 저장소에 만든 head branch가 대상이다. 사용자·다른 도구가 만든 branch·worktree·remote branch는
이름이 비슷해도 삭제 대상으로 가정하지 않는다.

worktree를 먼저 제거하고 그 뒤 local branch를 삭제한다. squash merge로 graph상 not fully merged여도
PR MERGED, merge commit의 upstream/devel 포함, worktree clean, 문서·asset의 devel 존재를 모두 확인한
경우에만 정확한 local branch를 강제 삭제한다.

~~~bash
git fetch upstream devel
git switch devel
git merge --ff-only upstream/devel
git worktree list
git worktree remove /path/to/pr-worktree
git branch -D <local-review-branch>
git branch -D <local-docs-branch>
git fetch upstream --prune
~~~

해당 작업에서 만들지 않았거나 존재하지 않는 placeholder 명령은 실행하지 않는다. PR head repository가
원본 edwardkim/rhwp이고, current collaborator가 이번 작업에서 만든 exact head branch일 때만 작업지시자
승인 뒤 upstream remote branch를 삭제한다. contributor fork의 head branch나 같은 이름의 다른 upstream
branch를 삭제하지 않는다.

~~~bash
git push upstream --delete <headRefName>
~~~

삭제 뒤에는 다음을 모두 확인한다.

~~~bash
git worktree list --porcelain
git branch --list '<local-review-pattern>' '<docs-pattern>' '<headRefName>'
git branch -r | rg '<local-review-pattern>|<docs-pattern>|<headRefName>' || true
git branch -vv | rg ': gone\]' || true
# head repository가 edwardkim/rhwp인 경우에만 확인
git ls-remote --heads upstream <headRefName>
git status --short --branch
~~~

최종 보고에는 이번 작업에서 사용한 각 local worktree를 `제거 완료` 또는 `유지`로 구분해 경로와 사유를
기록한다. 이 확인 없이 PR 병합만으로 후속 처리가 완료된 것으로 보고하지 않는다.

contributor fork의 head는 위 `upstream` 조회 대상이 아니다. PR metadata의 `headRepository`와
`headRefName`을 기록하고, fork branch 삭제를 시도하지 않은 사실을 최종 상태에 남긴다.

### 7.7.1 검토 전용 target

PR review Cargo 검증의 고정 경로 `target/pr-review`는 branch·worktree 정리 뒤에도 보존한다. 이 경로는
다음 review의 일반 컴파일 산출물을 재사용하는 shared review cache이며, 빌드 뒤 이동하면 통합 테스트에
박힌 절대 실행 경로가 깨질 수 있다. shared target/debug, target/release, target/release-test,
target/wasm32-unknown-unknown와 사용자·다른 도구 산출물도 삭제 대상으로 가정하지 않는다.

~~~bash
find target -mindepth 1 -maxdepth 1 -type d -exec du -sh {} \;
pgrep -alf '(^|/)(cargo|rustc|wasm-pack)( |$)' || true
~~~

실행 중인 Cargo/Rust가 있으면 해당 target을 유지한다. `target/pr-review`는 정리하지 않고, 종료된 과거
`target/review-*`처럼 고정 cache와 구별되는 exact legacy review directory만 소유·미사용을 확인한 뒤
제거하거나 복구 가능한 환경에서는 휴지통으로 이동한다. 이후 남은 target 하위 경로와 보존한
`target/pr-review`를 최종 상태에 기록한다.
