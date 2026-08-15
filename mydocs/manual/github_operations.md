---
kind: canonical
status: active
canonical: mydocs/manual/github_operations.md
last_verified: 2026-08-15
---

# GitHub 저장소 운영 매뉴얼

> 이 문서는 rhwp의 GitHub 저장소 설정과 GitHub Actions를 반복 운영하는 절차의 정본이다.
> 제품 소스의 구현·검증은 [문서와 Git 워크플로우](codex/docs_and_git_workflow.md), PR의
> 검토·통합은 [PR 리뷰·통합 워크플로우](pr_review_workflow.md), 실제 패키지 배포는
> [배포 가이드](publish_guide.md)를 함께 따른다. 이 문서는 그 절차를 복제하지 않고 운영 변경의
> 분류, 비례 검증, 적용 후 관찰과 복구를 담당한다.

## 1. 목적과 범위

GitHub 운영은 제품 소스 개발과 같은 저장소에서 이뤄지지만 실패 형태와 검증 방법이 다르다. 작은
`paths-ignore` 변경을 renderer 변경처럼 전체 Cargo·WASM·시각 검증에 태우는 것도 비효율이고, 반대로
required check나 권한 변경을 단순 YAML 수정으로 취급하는 것도 위험하다. 이 매뉴얼은 두 오류를 피하기
위해 **변경이 영향을 주는 운영 표면만 검증한다**.

이 문서가 다루는 범위는 다음과 같다.

- Actions workflow의 event, path filter, job, permission, concurrency, cache, artifact
- branch protection, ruleset, required check, 기본 브랜치와 merge 설정
- repository·environment의 Actions 권한, secret·variable의 이름과 배선
- workflow 실행·취소·재실행·활성화·비활성화와 적용 후 관찰
- Issue·PR·label·milestone 등 저장소 운영 metadata
- GitHub Pages, Release, package publish로 넘어가는 운영 경계
- Git LFS, Actions cache, runner와 사용량 장애의 조사·복구

소스 코드의 동작, parser·renderer 설계, 제품 API는 이 문서의 범위가 아니다. workflow가 실행하는
빌드 명령이나 테스트 범위를 바꾸는 경우에만 해당 제품 영역의 검증 절차를 추가한다.

## 2. 권위와 공통 원칙

### 2.1 권위의 경계

| 판단 대상 | 권위 |
| --- | --- |
| GitHub 운영 변경의 분류·검증·적용·관찰·복구 | 이 문서 |
| branch·commit·일반 이슈 작업 흐름 | [문서와 Git 워크플로우](codex/docs_and_git_workflow.md) |
| PR review·merge·후속 처리 | [PR 리뷰·통합 워크플로우](pr_review_workflow.md)과 선택된 자식 가이드 |
| 변경 범위별 제품 로컬 검증 | [로컬 사전 검증 4.3](pr_review/local_validation.md#43-변경-범위별-기본-검증) |
| release·npm·확장·Pages 배포 | [배포 가이드](publish_guide.md) |
| 현재 workflow의 실제 trigger·job·permission | `.github/workflows/*.yml`과 `.github/actions/` |
| 과거 운영 결정의 배경 | `mydocs/tech/`, `mydocs/report/`, historical memory |

과거 결정 기록은 현재 상태를 자동으로 보증하지 않는다. 운영 전에 저장소 파일과 GitHub live metadata를
다시 읽고, 관찰 시각과 대상 SHA를 기록한다.

### 2.2 기본 원칙

1. **메인테이너가 운영 판단을 소유한다.** comment, push, PR 생성, ready 전환, merge, close뿐 아니라
   workflow 취소·재실행·비활성화, cache 삭제, ruleset·permission·secret 변경도 명시 승인 뒤 수행한다.
2. **읽기와 변경을 분리한다.** 현황 조회는 먼저 수행할 수 있지만 조회 결과를 변경 승인으로 해석하지 않는다.
3. **GitHub 변경은 로컬 `gh`를 사용한다.** connector와 로컬 Git·SSH 인증은 별개다. mutation은 인증된
   `gh`로 수행하고, 고수준 명령이 막히면 정확한 REST endpoint를 `gh api`로 호출한 뒤 재조회한다.
4. **비례 검증을 한다.** trigger만 바꿨으면 trigger 계약을, job 명령을 바꿨으면 그 명령을, 배포를
   바꿨으면 배포 계약을 검증한다. 영향 없는 Cargo·WASM·브라우저 검증을 의례적으로 추가하지 않는다.
5. **불확실하면 실행 범위는 넓게, 권한은 좁게 닫는다.** 분류 실패가 검증 생략으로 열리거나 권한 실패가
   write 확대의 근거가 되어서는 안 된다.
6. **secret 값은 관찰·기록하지 않는다.** 이름, 존재 여부, environment 배선만 확인한다. 로그·PR·문서에
   token, 인증 URL, 2FA·recovery code를 넣지 않는다.
7. **적용과 완료를 구분한다.** merge나 설정 변경은 적용일 뿐이다. 예상 event에서 기대한 run 또는
   no-run, required check, 비용·시간, 부작용과 rollback 가능성을 확인해야 완료다.

## 3. 운영 변경 분류와 처리 경로

| 등급 | 예 | 기본 처리와 검증 |
| --- | --- | --- |
| O0 관찰 | workflow·run·권한·required check 조회 | 읽기 전용 조회, 시각·SHA·URL 기록. 원격 승인 불필요 |
| O1 metadata | label, milestone, assignee, Issue·PR metadata | 정확한 대상과 전후 snapshot, 명시 승인, mutation 뒤 재조회 |
| O2 라우팅·비용 | trigger, path filter, concurrency, timeout, cache·artifact 보존 | 운영 단축 경로, YAML·정책 테스트, required check 영향, 예상 run/no-run 검증 |
| O3 실행·보안 계약 | job command, matrix, action, permission, CodeQL, privileged event | 관련 job 로컬 검증, fork 신뢰 경계, 최소 권한, 최신 PR head Actions 결과 |
| O4 배포·복구 | branch protection, ruleset, secret·environment, runner, Release, publish, workflow disable | 별도 명시 승인, 전후 snapshot, 복구 명령, 필요 시 배포 가이드와 실제 산출물 확인 |

한 변경이 여러 등급에 걸리면 가장 높은 등급을 적용하되, 무관한 제품 검증까지 자동으로 승격하지 않는다.
예를 들어 `paths-ignore`와 `permissions`를 함께 바꾸면 O3이지만 renderer 회귀가 필요한 것은 아니다.

### 3.1 운영 단축 경로

O1 또는 O2 변경이 다음 조건을 모두 만족하면 일반 소스 구현의 단계별 보고서 묶음을 만들지 않고 하나의
운영 변경 기록으로 처리할 수 있다.

- 제품 소스·테스트·package lock·공개 API를 바꾸지 않는다.
- secret 값, branch protection, release·publish, runner 등록·삭제를 바꾸지 않는다.
- 변경과 rollback diff가 작고 결정적이다.
- 기존 Issue 또는 메인테이너의 명시 지시가 작업 근거다.
- 적용 전 관찰값, 검증 명령, 적용 후 기대값, rollback 조건을 한 기록에서 확인할 수 있다.

운영 변경 기록은 작은 작업이면 commit·PR 본문에, 여러 단계이거나 장기간 관찰해야 하면 Issue와
`mydocs/working/` 문서에 둔다. 별도 수행·구현·단계·최종 문서를 형식적으로 모두 생성하지 않는다.
다만 remote push와 PR 생성·merge 승인 게이트는 생략하지 않는다. O3·O4 또는 범위가 커진 O2는 일반
Issue·계획 절차로 전환한다.

## 4. 작업 전 live baseline

### 4.1 인증과 저장소 권한

```bash
gh auth status --hostname github.com
gh repo view edwardkim/rhwp \
  --json nameWithOwner,defaultBranchRef,isPrivate,viewerPermission
git remote -v
git status --short --branch
git rev-parse HEAD
```

`viewerPermission`, 로컬 `gh` 인증, Git remote의 fetch/push 가능 여부와 protected branch 우회 가능성은
서로 다른 사실이다. 하나의 성공으로 다른 권한까지 추정하지 않는다.

### 4.2 workflow와 최근 실행

```bash
gh workflow list --repo edwardkim/rhwp --all
gh run list --repo edwardkim/rhwp --limit 30
gh run view <run-id> --repo edwardkim/rhwp \
  --json databaseId,name,event,headBranch,headSha,status,conclusion,url,jobs
```

workflow 파일의 목록과 trigger는 바뀔 수 있으므로 문서에 적힌 개수보다 live 목록과
`.github/workflows/`를 우선한다. run을 비교할 때는 workflow 이름만 보지 말고 `event`, `headSha`,
`headBranch`, job과 step의 실제 실행·skip 여부를 함께 확인한다.

### 4.3 branch protection과 Actions 설정

```bash
gh api repos/edwardkim/rhwp/branches/devel \
  --jq '{name,protected,required_status_checks:.protection.required_status_checks}'
gh api repos/edwardkim/rhwp/rulesets
gh api repos/edwardkim/rhwp/actions/permissions
gh api repos/edwardkim/rhwp/actions/permissions/workflow
gh secret list --repo edwardkim/rhwp
gh variable list --repo edwardkim/rhwp
```

상세 protection endpoint가 권한에 따라 404를 반환할 수 있다. 이때 보호가 없다고 결론 내리지 말고 branch
metadata와 ruleset 조회 결과를 함께 남긴다. secret은 목록과 갱신 시각까지만 확인하고 값을 조회·출력하지
않는다.

### 4.4 baseline 기록 최소 항목

- 관찰 시각과 인증 계정
- 저장소, branch, 기준 SHA
- 변경 대상 workflow·설정과 현재 값
- 관련 required check 이름
- 최근 정상·비정상 run URL과 head SHA
- 예상되는 mutation과 정확한 rollback

## 5. 표준 운영 수명주기

### 5.1 접수와 분류

1. 증상 또는 운영 목표를 한 문장으로 고정한다.
2. O0~O4 등급과 영향을 받는 workflow·설정·branch를 정한다.
3. 제품 소스 변경과 운영 변경이 섞였으면 commit과 검증 근거를 분리할 수 있는지 판단한다.
4. 같은 증상의 Issue·PR·운영 기록을 검색한다.

### 5.2 영향 분석

다음 질문에 답하지 못하면 아직 수정 단계가 아니다.

- 어떤 event에서 시작하거나 시작하지 않아야 하는가?
- 어떤 파일 상태(add/modify/delete/rename)와 경로 조합이 대상인가?
- required check의 이름과 생성 주체가 유지되는가?
- fork PR 또는 신뢰하지 않는 head의 코드를 write token과 함께 실행하는가?
- cache, artifact, runner-minute, 배포 또는 외부 저장소에 어떤 비용·부작용이 생기는가?
- 실패하면 어떤 명령 또는 revert로 원상복구하는가?

### 5.3 변경과 로컬 검증

일반 변경은 최신 `upstream/devel` 기반 작업 branch에서 수행한다. workflow와 함께 그 계약을 고정하는
테스트가 있으면 같은 변경에서 갱신한다. 주석만으로 정책을 보증하지 않는다.

### 5.4 적용 승인

push 전에는 최신 base와 현재 head, diff, 로컬 검증, rollback을 보고한다. 메인테이너의 승인 뒤 push하고,
PR 생성은 별도 승인을 받는다. GitHub 설정을 직접 바꾸는 O1·O4 작업은 실행할 정확한 `gh` 명령과 대상을
먼저 제시한다.

### 5.5 적용 후 관찰

1. mutation 직후 동일 endpoint를 재조회한다.
2. 예상 event에서 workflow가 실행되거나 억제되는지 확인한다.
3. 실행됐다면 job과 step이 의도대로 run·skip됐는지 확인한다.
4. required check가 최신 head에 생성되고 완료되는지 확인한다.
5. runner-minute, cache, artifact, 배포 같은 지연 부작용은 정한 관찰 창까지 확인한다.
6. 기대와 다르면 완료로 기록하지 않고 rollback 또는 후속 수정으로 전환한다.

## 6. 변경 유형별 검증 매트릭스

| 변경 | 필수 검증 | 제품 전체 빌드가 필요한 조건 |
| --- | --- | --- |
| 운영 문서만 | `git diff --check`, 링크·metadata 검사 | 없음 |
| event·branch·path filter | YAML 구조, mirror/policy test, 변경 파일 조합 표, required check live 조회 | job 명령·제품 파일을 함께 바꾼 경우만 |
| concurrency·cancel·timeout | event별 group key, stale/new head 경쟁, cancel 대상, timeout 실패 표현 | 실행 명령도 바뀐 경우만 |
| cache | restore/save event, key·path, 신뢰 경계, quota·dry-run, 적용 뒤 hit/save 로그 | cache 대상 빌드가 실제로 복원 가능한지 바꾼 경우 |
| artifact | 생성자·소비자 연결, 이름·retention·크기, 민감정보·저작권 자산 포함 여부 | artifact 자체가 제품 산출물일 때 해당 build |
| job command·matrix | 해당 command의 로컬 실행, matrix 누락·중복, 최신 Actions 결과 | 영향받는 parser/renderer/Studio/package 범위만 |
| action·permission | commit SHA pin, 최소 `permissions`, fork·privileged event 경계 | action이 제품 build 방식을 바꾼 경우 |
| CodeQL | language 선택, no-op/full fallback, `security-events` 권한, 실제 Analyze step | build-mode·생성 코드가 바뀐 언어만 |
| branch protection·ruleset | 전후 JSON, required context 생성 여부, admin/bypass 영향, rollback | 없음 |
| secret·environment | 이름·배선·reviewer gate·OIDC, 로그 비노출 | 실제 publish 검증은 배포 가이드 승인 범위에서만 |
| release·publish·Pages | [배포 가이드](publish_guide.md)의 대상별 gate와 실제 공개 결과 | 배포 대상 전체 |

`.github/workflows/` 변경이라는 이유만으로 Cargo·WASM·Studio 전체를 실행하지 않는다. 반대로 YAML이
파싱된다는 이유만으로 event, required check, 권한과 실제 run 검증을 생략하지 않는다.

## 7. Actions trigger와 경로 필터 운영

### 7.1 `push`와 `pull_request`를 분리한다

같은 경로라도 두 event의 책임이 다르다.

- PR은 merge 전에 변경을 검증하고 protected branch의 required check를 발행한다.
- protected branch push는 merge 뒤 cache 갱신, 배포, 후속 자동화처럼 신뢰 branch의 책임을 수행한다.
- PR에서 workflow를 `paths-ignore`로 완전히 건너뛰면 required check가 생성되지 않아 merge가 pending에
  머물 수 있다. 필요한 경우 workflow는 시작하되 preflight와 aggregate만 성공시키는 fast-pass를 쓴다.
- push에서 검증이 끝난 reference-only 변경을 다시 full lane으로 실행할 이유가 없다면 event filter에서
  억제한다. PR 설정을 그대로 복사하지 않는다.

### 7.2 `paths-ignore` 의미

`paths-ignore`는 변경 파일이 **모두** ignore pattern에 맞을 때만 workflow를 억제한다. 한 commit에
ignore되지 않은 파일 하나가 섞이면 workflow 전체가 시작된다. 또한 pattern만으로 add와 modify/delete를
구분할 수 없다. 파일 상태별 정책이 다르면 event filter가 아니라 신뢰할 수 있는 preflight에서 diff를
분류한다.

경로 필터를 바꿀 때는 최소한 다음 조합을 표와 테스트로 확인한다.

- 대상 경로 파일만 있는 commit
- 대상 경로 여러 개가 섞인 commit
- 대상 경로와 source가 섞인 commit
- 루트 파일과 중첩 경로
- add, modify, delete, rename
- `push`, 내부 PR, fork PR, `workflow_dispatch`

### 7.3 reference-only 자산

현재 반복적으로 대형 참조 자료가 들어오는 경로는 `samples/**`, `pdf/**`, `pdf-2020/**`,
`pdf-large/**`다. 운영 정책은 다음처럼 분리한다.

- protected branch의 reference-only push는 제품 소스 CI와 CodeQL을 다시 실행하지 않는 것을 기본으로 한다.
- PR에서는 새 참조 자산만 있는 변경을 review-only fast-pass로 판정할 수 있다.
- 기존 sample·PDF의 수정·삭제·rename은 데이터 회귀 의미가 있을 수 있으므로 PR에서 자동 fast-pass로
  확장하지 않는다.
- reference 자산과 source가 섞이면 source 영향축을 실행한다.
- LFS pointer와 실제 객체를 구분하고, quota·smudge 실패를 파일 수정으로 오인해 재커밋하지 않는다.

이 목록을 workflow, classifier, policy test에 중복 유지해야 한다면 mirror test로 불일치를 실패시킨다.
매뉴얼의 경로 이름만 바꾸고 구현을 바꾸지 않았거나 그 반대인 상태를 완료로 취급하지 않는다.

### 7.4 현재 정책 코드와 집중 검증

CI 영향 분류와 trigger mirror는 다음 파일에 있다.

- `scripts/ci-impact-classifier.cjs`
- `scripts/ci-impact-policy.cjs`
- `scripts/tests/ci-impact-classifier.test.cjs`
- `scripts/tests/ci-impact-policy.test.cjs`
- `scripts/tests/test_ci_impact_workflow.py`
- `scripts/tests/test_codeql_workflow.py`
- `scripts/tests/test_review_only_fast_pass_workflows.py`

관련 파일을 바꾼 경우 실제 존재하는 테스트만 선택해 실행하고, 변경 기록에 명령과 결과를 적는다. 경로
필터 변경에는 적어도 classifier·policy의 Node test와 해당 CI·CodeQL workflow Python test를 포함한다.

### 7.5 workflow PR의 후행 review 기록

workflow·action·CI impact 정책을 바꾼 PR은 PR 전체 변경 목록에 실행 정책 파일이 남으므로, 후행
`mydocs/**` commit만 보고 자체 preflight가 검증을 생략해서는 안 된다. 기본 브랜치의
`CI Impact Policy Controller`가 exact Full candidate, 이후 review-only 계보, current-base merge bridge,
현재 head·base 결합을 독립 검증해 `rfp=1` status를 발행한 same-repository PR만 제한적으로 fast-pass한다.

consumer workflow는 status context 문자열만 신뢰하지 않는다. status의 policy version·current base SHA,
creator, target Action run의 workflow name·path·`pull_request_target` event를 함께 확인한다. 증빙 누락,
API pagination 경계, candidate의 fast-pass 실행, failed·pending run, GHAS CodeQL check 누락, fork, stale base는
전부 Full 실행으로 fallback한다. 상세 허용 범위와 merge bridge 규칙은
[review-only fast-pass](pr_review/review_only_fast_pass.md#a1-ci-실행-정책을-바꾼-pr의-trusted-재사용)를 따른다.

이 controller는 default branch 등록형이므로 `devel` 병합은 배포 전 검증 단계다. 정상 release로 `main`에
반영하기 전에는 live `pull_request_target` controller가 존재하지 않으며, 그 기간의 workflow PR은 계속 Full
실행하는 것이 정상이다.

## 8. required check와 branch protection

required context 이름은 외부 계약이다. job 이름 변경, workflow 분리, path skip, matrix 이름 변경은 YAML
내부 리팩터링이 아니라 merge gate 변경일 수 있다.

운영 순서:

1. live branch metadata에서 required context를 읽는다.
2. 각 context를 어느 workflow와 aggregate job이 발행하는지 찾는다.
3. 모든 대상 PR 유형에서 context가 생성되는지 확인한다.
4. context를 교체해야 하면 새 check를 먼저 발행·관찰하고 protection을 갱신한 뒤 옛 check를 제거한다.
5. 변경 직후 내부 PR과 fork PR에서 pending·중복·우회가 없는지 확인한다.

required check 변경과 workflow 변경을 한 번에 적용해 관찰 불가능하게 만들지 않는다. branch protection
mutation은 O4이며 별도 승인을 받는다.

## 9. Actions 보안과 권한

### 9.1 신뢰 경계

- `pull_request`의 fork head는 신뢰하지 않는 입력이다.
- `pull_request_target`과 write permission이 있는 `workflow_run`은 base의 신뢰 코드만 실행한다.
- privileged workflow에서 PR head를 checkout하거나 PR이 바꾼 script를 실행하지 않는다.
- PR 파일명, label, comment, workflow output도 입력으로 검증한다.
- classification·API 실패가 heavy 검증 생략 또는 write 확대로 이어지지 않게 fail-closed한다.

### 9.2 권한과 action 의존성

- workflow top-level `permissions`는 `{}` 또는 필요한 최소 read 권한으로 시작하고 job별로 write를 연다.
- `contents: write`, `issues: write`, `actions: write`, `statuses: write`, `security-events: write`,
  `id-token: write`는 사용 step과 event를 명시한다.
- 신규 또는 변경하는 외부 action은 원칙적으로 full commit SHA에 고정하고 사람이 읽는 버전 주석을 둔다.
- local reusable workflow와 composite action의 입력·권한 전달을 확인한다.
- 기존의 tag 참조나 넓은 권한은 별도 개선 대상으로 기록하며 관련 없는 변경에 몰래 섞지 않는다.

### 9.3 secret과 environment

- secret 값은 출력·복사·백업하지 않는다. 교체는 새 값 등록, 소비 workflow 확인, 옛 credential 폐기 순서다.
- OIDC를 지원하는 배포는 장기 token보다 Trusted Publishing을 우선한다.
- environment 이름과 secret 이름을 혼동하지 않는다.
- fork PR, artifact, cache, step summary에 secret 또는 민감한 URL이 유입되지 않는지 확인한다.

보안 취약점 자체는 공개 Issue나 PR comment에 상세 재현을 쓰지 않고 [보안 정책](../../.github/SECURITY.md)의
비공개 신고 경로를 따른다.

## 10. cache, artifact, runner와 LFS

### 10.1 Actions cache

- PR은 restore-only, 신뢰 branch push는 필요할 때 save하는 원칙을 우선 검토한다.
- cache key에는 OS·도구 버전·lockfile·영향 feature가 충분히 반영되는지 확인한다.
- cache 삭제는 복구가 어려운 mutation이다. 목록과 크기를 먼저 조회하고 자동 sweep은 dry-run을 우선한다.
- 세대 정리, orphan ref 정리, 전체 quota 대응을 한 동작으로 혼합하지 않는다.
- 적용 뒤 cache hit/miss/save, runner-minute와 build 시간을 관찰한다.

```bash
gh cache list --repo edwardkim/rhwp --limit 100
gh workflow run cache-generation-sweep.yml --repo edwardkim/rhwp \
  -f dry_run=true
```

두 번째 명령은 workflow 실행 mutation이므로 승인 뒤 사용한다.

### 10.2 artifact와 runner

- artifact에 원본 private corpus, secret, `.env`, 개인 폰트, 라이선스 제한 자산을 넣지 않는다.
- producer와 consumer의 run·SHA를 연결하고 retention과 크기를 명시한다.
- `runs-on`과 실제 runner fleet은 workflow와 GitHub live 상태에서 확인한다. 과거 self-hosted 실험 기록을
  현재 runner 존재의 근거로 쓰지 않는다.
- runner 추가·삭제·label 변경은 O4이며, 동시 쓰기 경로·도구 설치·cache·메모리 격리를 먼저 설계한다.

### 10.3 Git LFS

LFS quota와 smudge 실패는 Actions cache 문제와 별개다. pointer만 있는 파일을 정상 blob으로 오인하거나,
작업트리의 거짓 `M` 상태를 정리하려고 재커밋·migrate하지 않는다. LFS 사용량·대상 pointer·원격 객체
존재를 먼저 조사하고 quota 조치는 별도 승인받는다.

## 11. release와 자동 mutation workflow

GitHub Release 생성, tag push, Pages 배포, npm·extension publish는 O4다. 실행 전에는 반드시
[배포 가이드](publish_guide.md)의 버전·산출물·OIDC·스토어 절차를 읽는다.

Issue close, stale run cancel, cache 삭제처럼 GitHub 상태를 자동으로 바꾸는 workflow도 배포와 같은 방식으로
취급한다.

- trigger가 정확한 branch·event로 제한되는지 확인한다.
- API 대상이 현재 run의 repository·head·Issue인지 검증한다.
- write permission을 mutation job에만 둔다.
- dry-run 또는 read-only 계산 단계와 실제 mutation 단계를 분리한다.
- idempotency와 재실행 결과를 정의한다.
- 부분 성공 뒤 재실행이 중복 close·publish·delete를 만들지 확인한다.

## 12. 장애와 운영 핫픽스

반복 실행, 비용 폭증, PR 전체 pending, 잘못된 자동 mutation, 배포 위험이 관찰되면 먼저 확산을 멈춘다.
workflow 취소·비활성화도 원격 mutation이므로 대상과 영향을 제시하고 승인을 받는다.

```bash
gh run cancel <run-id> --repo edwardkim/rhwp
gh workflow disable <workflow-file> --repo edwardkim/rhwp
# 복구 뒤
gh workflow enable <workflow-file> --repo edwardkim/rhwp
```

핫픽스 순서:

1. 실패 run URL·head SHA·event·실행된 job을 보존한다.
2. 중복 run 또는 위험 mutation을 멈출 최소 조치를 정한다.
3. 작은 O2 수정이면 운영 단축 경로로 trigger·policy test만 검증한다.
4. 최신 `devel` 기반 PR로 수정하고, admin 직접 반영은 메인테이너가 명시한 예외에서만 사용한다.
5. 적용 후 같은 입력과 source 혼합 입력을 모두 관찰한다.
6. 원인이 해소되지 않거나 required check가 사라지면 즉시 rollback한다.

긴 원인 분석이나 반복 가능한 장애 처방은 `mydocs/troubleshootings/`에 남기고, 확정된 장기 규칙만 이
매뉴얼에 반영한다.

## 13. 운영 변경 기록 템플릿

~~~markdown
## GitHub 운영 변경

- 등급: O0 | O1 | O2 | O3 | O4
- 근거: Issue 또는 작업지시
- 기준 시각 / SHA:
- 대상 workflow·설정:
- 현재 상태:
- 기대 상태:
- 영향 event·branch·required check:
- 변경 파일 또는 실행할 `gh` 명령:
- 로컬 검증:
- 적용 승인:
- 적용 결과와 run URL:
- 관찰 창과 완료 조건:
- rollback 조건과 명령:
~~~

기록에는 완료된 검증과 아직 실행하지 않은 GitHub 동작을 분리한다. “CI 통과 예정”을 완료 근거로 쓰지
않고 실제 head SHA와 run URL을 남긴다.

## 14. 완료 체크리스트

- [ ] 변경 등급과 권위 문서를 선택했다.
- [ ] live 설정, 기준 SHA, 최근 run을 확인했다.
- [ ] source 변경과 운영 변경의 검증 범위를 분리했다.
- [ ] required check와 fork 신뢰 경계를 확인했다.
- [ ] secret 값이나 private 자료가 diff·로그·artifact에 없다.
- [ ] 변경에 대응하는 정책·workflow 테스트가 통과했다.
- [ ] remote mutation별 메인테이너 승인을 받았다.
- [ ] mutation 직후 동일 대상을 재조회했다.
- [ ] 예상 event의 run 또는 no-run과 실제 job·step을 확인했다.
- [ ] rollback이 가능하며 완료 조건을 충족했다.

## 15. 문서 유지 규칙

이 문서에는 반복 가능한 운영 판단만 둔다. 현재 workflow 개수, 최근 run ID, 현재 required context처럼
변하는 값은 작업 기록에 남기고 live 조회한다. 제품별 명령은 배포·로컬 검증 가이드에 유지하며 여기서
복제하지 않는다.

새 GitHub 운영 절차가 생기면 먼저 이 문서의 기존 등급과 수명주기로 표현할 수 있는지 확인한다. 별도
매뉴얼이 필요하면 이 문서에 경계와 라우팅만 추가한다. 정보구조 또는 canonical 관계를 바꾸면
[문서 링크와 메타데이터 로컬 검사](markdown_link_check_guide.md)를 실행한다.
