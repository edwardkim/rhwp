# Task M100 #1547 Stage 2 완료보고서 — CI workflow preflight 적용

- 이슈: #1547
- 브랜치: `local/task1547`
- 작성일: 2026-06-26
- 단계: Stage 2 — CI workflow preflight/gate 적용

## 1. 변경 요약

`.github/workflows/ci.yml` 에 `preflight` job 을 추가하고, `Build & Test` job 이 review 문서 전용 후속
커밋에서만 job-level skip 될 수 있도록 했다.

변경된 동작:

- PR 이벤트가 아니면 fast-pass 하지 않는다.
- PR commit 목록을 뒤에서부터 검사한다.
- trailing commit 이 모두 허용 경로(`mydocs/pr/**`, `mydocs/orders/*.md`)만 변경한 경우에만 후보로 본다.
- trailing docs-only commit 은 single-parent 여야 한다. merge commit 이면 fast-pass 하지 않는다.
- 허용 경로 밖 변경을 처음 만난 commit 을 candidate SHA 로 삼는다.
- candidate SHA 의 `Build & Test` check-run 이 `success`, `skipped`, `neutral` 중 하나일 때만
  `fast_pass=true` 를 출력한다.
- check-run 이 없거나, API 조회가 실패하거나, 파일 목록이 비어 있거나 300개 이상이면 기존 full CI 로
  떨어진다.

## 2. 구현 상세

추가 job:

- `preflight`
  - 권한: `contents: read`, `checks: read`, `pull-requests: read`
  - 도구: `actions/github-script@v8`
  - 출력:
    - `fast_pass`
    - `reason`
    - `candidate_sha`

수정 job:

- `build-and-test`
  - `needs: preflight` 추가
  - `if: ${{ always() && needs.preflight.outputs.fast_pass != 'true' }}` 추가

`always()` 를 사용해 preflight 가 실패하더라도 `Build & Test` 가 실행될 수 있게 했다. preflight step 에도
`continue-on-error: true` 를 두어 판정 실패가 전체 CI 실패로 이어지지 않게 했다.

## 3. 보수 조건

이번 Stage 에서는 fast-pass 허용 경로를 다음으로 제한했다.

```text
mydocs/pr/**
mydocs/orders/*.md
```

다음 변경은 fast-pass 대상이 아니다.

- `.github/**`
- `docs/**`
- `samples/**`
- `src/**`
- `tests/**`
- `rhwp-studio/**`
- `mydocs/plans/**`
- `mydocs/report/**`
- `mydocs/working/**`

즉, 일반 타스크 문서나 보고서를 PR head 에 추가하는 경우에는 full CI 가 실행된다.

## 4. Dry-run 결과

synthetic 입력으로 핵심 분기를 확인했다.

| 케이스 | 결과 |
|---|---|
| 코드 커밋 뒤 `mydocs/pr/**`, `mydocs/orders/*.md` 커밋 + candidate `Build & Test=success` | fast-pass true |
| trailing docs-only commit 없음 | fast-pass false |
| trailing docs-only merge commit | fast-pass false |
| candidate `Build & Test=failure` | fast-pass false |

출력:

```text
positive [true,"build-and-test-green:success","c1"]
noTrailing [false,"no-trailing-review-doc-commits"]
docsMerge [false,"docs-only-merge-commit:d1"]
failedCandidate [false,"build-and-test-not-green:failure"]
```

실제 PR #1544 최신 head `bff0a603942c300f5a08fb1cd056411c02f2e7cd` 도 확인했다.

- parents: `2`
- file_count: `34`
- allowed docs-only: `false`
- sample 경로에 `mydocs/plans/**`, `mydocs/report/**` 포함

따라서 현재 Stage 2 조건에서는 PR #1544 같은 통합 문서/보고서 포함 merge commit 은 fast-pass 되지 않는다.

## 5. 검증 결과

| 명령 | 결과 |
|---|---|
| `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); puts "yaml ok"'` | 통과 |
| `actionlint .github/workflows/ci.yml` | 통과 |
| `git diff --check` | 통과 |
| synthetic preflight dry-run | 통과 |
| PR #1544 live metadata 확인 | fast-pass 대상 아님 확인 |

## 6. 남은 작업

다음 Stage 3 에서는 같은 원칙을 `.github/workflows/codeql.yml` 과 `.github/workflows/render-diff.yml` 에
확장한다. CodeQL 은 matrix check 3개, Render Diff 는 `Canvas visual diff` check-run 이 candidate SHA 에
존재하고 green 일 때만 fast-pass 해야 한다.
