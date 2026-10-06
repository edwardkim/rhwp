# Task M100 #1574 Stage 1 완료보고서 — workflow trigger와 preflight candidate 보정

- 이슈: #1574
- 브랜치: `task_m100_1574_ci_fast_pass_trigger`
- 작성일: 2026-06-26
- 단계: Stage 1 — trigger와 candidate 판정 보정

## 1. 변경 요약

PR #1572에서 review 문서 전용 변경이 `paths-ignore`/`paths` 필터에 막혀 preflight job이 생성되지 않는
문제를 보정했다.

수정 파일:

- `.github/workflows/ci.yml`
- `.github/workflows/codeql.yml`
- `.github/workflows/render-diff.yml`

## 2. CI / CodeQL trigger 보정

`pull_request.paths-ignore`에서 `mydocs/**` 제외를 제거했다.

의도:

- `mydocs/pr/**`, `mydocs/orders/*.md` 변경에서도 workflow가 시작된다.
- workflow가 시작된 뒤 preflight가 allowed review doc 범위인지 판정한다.
- allowed review doc 범위가 아니면 fast-pass하지 않고 기존 full job으로 fallback한다.

`push.paths-ignore`는 그대로 유지했다. devel/main push에서 일반 문서 변경만으로 CI를 실행하지 않던 기존 동작은
이번 범위에서 바꾸지 않았다.

## 3. Render Diff trigger 보정

`pull_request.paths`에 다음 허용 경로를 추가했다.

```text
mydocs/pr/**
mydocs/orders/*.md
```

이로써 review 문서 전용 후속 커밋에서도 `Render Diff preflight`가 생성될 수 있다. candidate에
`Canvas visual diff` check가 없거나 green이 아니면 기존처럼 full `Canvas visual diff`로 fallback한다.

## 4. 순수 review 문서 PR candidate 보정

기존 preflight는 PR commit 수가 2개 미만이거나, 모든 PR commit이 review 문서 전용이면 `no-code-candidate`로
fast-pass하지 않았다.

이번 변경에서는 PR 전체 commit이 모두 allowed review doc 변경이면 `pr.base.sha`를 candidate로 사용한다.

유지한 안전 조건:

- PR commit이 0개면 fast-pass하지 않는다.
- 허용 경로 밖 변경이 있으면 해당 commit을 기존 candidate로 삼는다.
- candidate check-run이 없거나, 진행 중이거나, 실패/취소 상태이면 fast-pass하지 않는다.
- merge commit 형태의 review 문서 후속 커밋은 fast-pass하지 않는다.

## 5. 검증

| 명령 | 결과 |
|---|---|
| `ruby -e 'require "yaml"; ... YAML.load_file(...)'` | 통과 |
| `git diff --check` | 통과 |
| `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml .github/workflows/render-diff.yml` | 통과 (`actionlint` 1.7.12) |

## 6. 다음 단계

- Stage 1 변경과 계획/보고 문서를 커밋한다.
- 후속 PR을 생성해 #1574 자체 GitHub Actions 결과를 확인한다.
- #1572 브랜치에 review 문서 전용 후속 커밋을 추가해 check 생성 여부와 `BLOCKED` 해소 여부를 확인한다.
