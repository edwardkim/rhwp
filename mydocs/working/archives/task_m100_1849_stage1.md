# Task M100 #1849 Stage 1 보고서

## 개요

- 이슈: #1849 `[CI] devel push profile 배치 재조정: release-test 회귀 + release smoke`
- 부모 이슈: #1668
- 브랜치: `task-1849-ci-profile-policy`
- 단계: Stage 1 — workflow profile 분기 구현 및 정적 검증

## 변경 요약

`.github/workflows/ci.yml`의 `Build & Test` job에서 #1849 정책 후보 A를 구현했다.

| 이벤트 / ref | Build | Native Skia tests | Run lib tests | Run integration tests |
|--------------|-------|-------------------|---------------|-----------------------|
| `pull_request` | `release-test` | `release-test` | `release-test` | `release-test` |
| `push` `refs/heads/devel` | `release` | `release-test` | `release-test` | `release-test` |
| `push` `refs/heads/main` | `release` | `release` | `release` | `release` |
| tag push `refs/tags/v*` | `release` | `release` | `release` | `release` |
| `workflow_dispatch` | `release` | `release` | `release` | `release` |

핵심 변경:

- `Build` step은 PR에서만 `release-test`를 사용하고, 그 외에는 `release`를 유지한다.
- `devel` push에서 `Build` step은 `cargo build --release --verbose` release smoke 역할을 한다.
- `Native Skia tests`, `Run lib tests`, `Run integration tests`는 PR과 `push refs/heads/devel`에서 `release-test`를 사용한다.
- `main` push, tag push, `workflow_dispatch`는 full `release` test 경로로 남긴다.
- 로그에 `event`, `ref`, release smoke 역할을 출력해 CI 분석 시 profile 선택을 확인할 수 있게 했다.

## 구현 중 보정

구현계획서 초안의 단순 조건 `GITHUB_REF == refs/heads/devel`은 `workflow_dispatch`를 `devel` ref로 실행할 때도
`release-test`로 빠질 수 있었다.

따라서 실제 구현과 구현계획서 모두 다음 조건으로 보정했다.

```bash
[[ "${GITHUB_EVENT_NAME}" == "pull_request" ]] || [[ "${GITHUB_EVENT_NAME}" == "push" && "${GITHUB_REF}" == "refs/heads/devel" ]]
```

이렇게 하면:

- PR: `release-test`
- `devel` push: `release-test`
- `main` push: `release`
- `workflow_dispatch` on `devel`: `release`

로 분리된다.

## 변경하지 않은 범위

- `Cargo.toml` 변경 없음
- `tests/**` 변경 없음
- `tests/golden_svg/**` 변경 없음
- `Build & Test` job 이름 변경 없음
- cache restore/save 조건 변경 없음
- job 병렬화 없음
- `Swatinem/rust-cache` 도입 없음

## 로컬 정적 검증

| 항목 | 결과 | 비고 |
|------|------|------|
| `git diff --check` | 통과 | whitespace 문제 없음 |
| `actionlint .github/workflows/ci.yml` | 통과 | workflow lint 오류 없음 |
| YAML parse | 통과 | Ruby `YAML.load_file` 기준 |
| bash 조건식 syntax | 통과 | `bash -n -c` 기준 |
| 조건식 샘플 평가 | 통과 | PR/devel push는 `release-test`, main push/workflow_dispatch on devel은 `release` |

조건식 샘플:

| event | ref | 결과 |
|-------|-----|------|
| `pull_request` | `refs/pull/1/merge` | `release-test` |
| `push` | `refs/heads/devel` | `release-test` |
| `push` | `refs/heads/main` | `release` |
| `workflow_dispatch` | `refs/heads/devel` | `release` |

## PR CI 관측 예정 항목

PR run에서는 다음을 확인한다.

- `Build`, `Native Skia tests`, `Run lib tests`, `Run integration tests` 모두 `profile=release-test event=pull_request`인지
- PR save skipped 유지
- `CI / Build & Test` check 표면 유지
- 회귀 가드 1:1 추적성 유지
- `Build & Test` 시간이 #1666 after PR 기준 10m49s 근처에서 유지되는지

## merge 후 `devel` push 관측 예정 항목

merge 후 `devel` push에서는 다음을 확인한다.

- `Build`가 `profile=release ... role=release-smoke-or-full`로 실행되는지
- 세 test step이 `profile=release-test event=push ref=refs/heads/devel`로 실행되는지
- `Run integration tests` 시간이 #1666 merge 후 P50 42m27s / P90 44m14s 대비 줄어드는지
- `Build & Test` 시간이 #1666 merge 후 P50 56m55s / P90 59m14s 대비 줄어드는지
- cache exact hit/save skipped 또는 save success가 정상인지
- cache reservation/read-only/save failure 경고가 없는지

## 판단

Stage 1 구현은 계획 범위 안에서 완료됐다. 현재 변경은 profile 분기와 로그 가시성 보강에 한정되어 있으며,
테스트 구조와 required check 표면은 바꾸지 않았다.

다음 단계는 코드 PR 생성 후 PR CI 관측이다. measurement 원천 문서는 PR CI와 merge 후 `devel` push 관측값이
쌓인 뒤 후속 문서 PR로 분리해 갱신한다.
