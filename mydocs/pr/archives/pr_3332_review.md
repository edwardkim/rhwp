# PR #3332 검토 기록 — softprops/action-gh-release 3

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: 75e81063cbc670d36ce62e2148cb4454f2057b07
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3332](https://github.com/edwardkim/rhwp/pull/3332) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `github_actions` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +1/-1, 1 file, 1 commit |
| head | `75e81063cbc670d36ce62e2148cb4454f2057b07` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | Release Actions Route B 통합에 수용 |

## 변경과 통합

- `.github/workflows/release-binary.yml`의 `softprops/action-gh-release`를 v2에서 v3로 올린다.
- source `75e81063cbc670d36ce62e2148cb4454f2057b07`을 `git cherry-pick -x`로 적용해
  integration `84a22e897bc5eb69bb9b8a980a84f3e133194d1b`을 만들었다.
- release 입력과 artifact glob은 바꾸지 않았고 원 저자·`Signed-off-by`·source SHA를 보존했다.

## 검증과 리스크

- 원 PR 최신 head의 `Build & Test`와 CodeQL은 문서 작성 시점 성공이다.
- 누적 tree의 diff는 action version 교체만 포함하며 `actionlint -shellcheck=`와 `git diff --check`가 성공했다.
- 실제 tag release와 artifact attach는 PR CI에서 실행되지 않는다. integration PR의 일반 CI 성공 뒤에도
  다음 실제 release run 확인이 운영상 잔여 gate다.
- 전체 actionlint가 표시한 release script shellcheck 정보는 기존 script 줄이며 이번 diff가 바꾸지 않았다.
- renderer/layout/asset 변경이 없어 visual sweep 대상이 아니다.

**권고**: #3332는 직접 merge하지 않고 Actions integration PR로 대체한다. 최신 CI와 작업지시자 승인을
merge 조건으로 두며 merge 뒤 별도 승인으로 원 PR을 close한다.
