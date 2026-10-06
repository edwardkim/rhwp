# Task M100 #1547 Stage 1 완료보고서 — GitHub check metadata 조사

- 이슈: #1547
- 브랜치: `local/task1547`
- 작성일: 2026-06-26
- 단계: Stage 1 — GitHub check metadata 조사

## 1. 조사 목적

리뷰 문서 전용 후속 커밋 fast-pass 를 구현하려면, 직전 코드 검증 대상 SHA 에 붙은 GitHub check 를
정확히 조회해야 한다. 따라서 실제 PR 에서 check run 이 PR head SHA 와 GitHub test-merge SHA 중 어디에
귀속되는지, 어떤 check 이름을 기준으로 green 판정해야 하는지 확인했다.

## 2. 조사 대상

대표 샘플:

| PR | 상태 | 작성자 | head SHA | test-merge SHA | 비고 |
|---|---|---|---|---|---|
| #1544 | open | `planet6897` | `bff0a603942c300f5a08fb1cd056411c02f2e7cd` | `160dcc2e5af485fc252485e5282a1586c3b3b5a6` | CI/CodeQL/Render Diff 모두 존재 |
| #1541 | open | `planet6897` | `287ac2f76b5e401dc95718fc900acacf29905b85` | `d39946042fd15c02ba8f83a7d6ad290ca899cf3f` | Render Diff 미대상 PR 샘플 |

## 3. 확인 결과

### 3.1 check run 은 PR head SHA 에 붙는다

PR #1544 기준:

- `head`: `bff0a603942c300f5a08fb1cd056411c02f2e7cd`
- `merge_commit_sha`: `160dcc2e5af485fc252485e5282a1586c3b3b5a6`

`head` SHA 의 check-runs 조회 결과에는 다음 check 가 존재했다.

| check | conclusion | app |
|---|---|---|
| `Build & Test` | `success` | `github-actions` |
| `Analyze (javascript-typescript)` | `success` | `github-actions` |
| `Analyze (python)` | `success` | `github-actions` |
| `Analyze (rust)` | `success` | `github-actions` |
| `Canvas visual diff` | `success` | `github-actions` |
| `WASM Build` | `skipped` | `github-actions` |
| `CodeQL` | `success` | `github-advanced-security` |

동일 PR 의 `merge_commit_sha` 에 대한 check-runs 조회 결과는 빈 배열이었다. 따라서 fast-pass 판정은
PR test-merge SHA 가 아니라 candidate PR head SHA 의 check-runs 를 기준으로 삼아야 한다.

### 3.2 check-run 객체의 `head_sha` 도 PR head SHA 와 일치한다

PR #1544 의 주요 check-run 샘플:

| check | `head_sha` | conclusion |
|---|---|---|
| `Build & Test` | `bff0a603942c300f5a08fb1cd056411c02f2e7cd` | `success` |
| `Analyze (rust)` | `bff0a603942c300f5a08fb1cd056411c02f2e7cd` | `success` |
| `Canvas visual diff` | `bff0a603942c300f5a08fb1cd056411c02f2e7cd` | `success` |

### 3.3 classic commit status 는 현재 판정 기준으로 부적합하다

PR #1544 head SHA 의 classic combined status 조회 결과:

- `statuses`: 빈 배열
- `state`: `pending`

실제 PR check 는 모두 pass 상태였으므로, classic status endpoint 는 이 저장소의 GitHub Actions 판정에
사용하지 않는다. 구현은 Check Runs API 기준으로 잡는다.

### 3.4 Render Diff 는 대상 경로가 아니면 check 자체가 없다

PR #1541 head SHA 에는 다음 check 만 존재했다.

- `Build & Test`
- `Analyze (javascript-typescript)`
- `Analyze (python)`
- `Analyze (rust)`
- `WASM Build` (`skipped`)
- `CodeQL`

`Canvas visual diff` 는 없었다. 이는 `.github/workflows/render-diff.yml` 의 `paths` 조건과 일치한다.
따라서 Render Diff workflow 안의 fast-pass 는 candidate SHA 에 `Canvas visual diff` 가 없으면 통과시키지
않고 heavy job 을 실행하는 방향이 안전하다.

### 3.5 branch protection required checks 는 현재 토큰으로 상세 조회 불가

`devel` 브랜치는 protected 상태다.

```text
{"name":"devel","protected":true}
```

하지만 required status checks endpoint 는 404 를 반환했다. 보호 설정이 없어서가 아니라 현재 토큰/권한에서
상세 설정을 읽을 수 없는 가능성이 있다. 따라서 required check 목록은 GitHub Actions check 이름 기준으로
보수적으로 설계하고, 필요 시 maintainer/admin 이 branch protection UI 에서 별도 확인해야 한다.

## 4. 구현 판단

Stage 2 이후 구현은 다음 전제를 둔다.

1. candidate SHA 는 PR head 계열의 실제 commit SHA 여야 한다.
2. green 판정은 Check Runs API 기준으로 한다.
3. 허용 conclusion 은 `success`, `skipped`, `neutral` 로 둔다.
4. CI workflow 의 필수 판정 check 는 `Build & Test` 로 둔다.
5. CodeQL workflow 는 `Analyze (javascript-typescript)`, `Analyze (python)`, `Analyze (rust)` 3개가 모두
   green 일 때만 fast-pass 한다.
6. Render Diff workflow 는 `Canvas visual diff` 가 green 일 때만 fast-pass 한다.
7. check-run 이 없거나 권한/응답이 불명확하면 fast-pass 하지 않는다.

## 5. 검증 명령

실행한 주요 명령:

```bash
gh pr list --repo edwardkim/rhwp --base devel --state all --limit 20 \
  --json number,title,state,author,headRefOid,headRefName,headRepositoryOwner,isCrossRepository,mergedAt,updatedAt,statusCheckRollup

gh api repos/edwardkim/rhwp/pulls/1544 \
  --jq '{number:.number, state:.state, head:.head.sha, base:.base.sha, merge_commit_sha:.merge_commit_sha, head_repo:.head.repo.full_name, head_ref:.head.ref}'

gh api repos/edwardkim/rhwp/commits/bff0a603942c300f5a08fb1cd056411c02f2e7cd/check-runs \
  --jq '[.check_runs[] | {name, status, conclusion, app:.app.slug, url:.details_url}]'

gh api repos/edwardkim/rhwp/commits/160dcc2e5af485fc252485e5282a1586c3b3b5a6/check-runs \
  --jq '[.check_runs[] | {name, status, conclusion, app:.app.slug, url:.details_url}]'

gh api repos/edwardkim/rhwp/commits/bff0a603942c300f5a08fb1cd056411c02f2e7cd/status \
  --jq '{state:.state, statuses:[.statuses[] | {context, state, target_url}]}'

gh pr checks 1544 --repo edwardkim/rhwp

gh api repos/edwardkim/rhwp/branches/devel \
  --jq '{name:.name, protected:.protected, protection_url:.protection_url}'
```

## 6. 다음 단계

Stage 2 에서는 `.github/workflows/ci.yml` 에 preflight 판정과 `Build & Test` job-level skip 조건을
적용한다. workflow 파일 수정 단계이므로 작업지시자 승인 후 진행한다.
