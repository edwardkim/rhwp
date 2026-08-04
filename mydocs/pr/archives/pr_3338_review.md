# PR #3338 검토 기록 — actions/cache 6

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: 1c821e9e5010a25fbaea73cd780d7015730d20ad
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3338](https://github.com/edwardkim/rhwp/pull/3338) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `github_actions` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +11/-11, 7 files, 1 commit |
| head | `1c821e9e5010a25fbaea73cd780d7015730d20ad` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | Cache Actions Route B 통합에 수용 |

## 변경과 통합

- CI, CodeQL, Pages, renderer sweep, npm publish, release, Render Diff의 `actions/cache`,
  `actions/cache/restore`, `actions/cache/save`를 v5에서 v6로 올린다.
- source `1c821e9e5010a25fbaea73cd780d7015730d20ad`를 `git cherry-pick -x`로 적용해
  integration `07431b472cfb32536a4b228e591ab86061eb825c`를 만들었다.
- cache key, path, restore/save 조건은 바꾸지 않았고 원 저자·`Signed-off-by`·source SHA를 보존했다.

## 검증과 리스크

- 원 PR 최신 head의 `Frontend package gates`, `Build & Test`, CodeQL, Render Diff는 문서 작성 시점 성공이다.
- cache restore는 integration PR CI에서 실제 실행된다. push 전용 save step은 PR CI가 실행하지 않는다.
- 누적 tree의 diff는 action version 교체만 포함하며 `actionlint -shellcheck=`와 `git diff --check`가 성공했다.
- cache miss는 correctness 실패가 아니라 성능 저하로 나타날 수 있으므로 integration CI의 cache와 무관한
  test 성공을 merge gate로 둔다.
- renderer source나 fixture를 바꾸지 않아 visual sweep 대상이 아니다.

**권고**: #3338은 직접 merge하지 않고 Actions integration PR로 대체한다. 최신 integration head CI와
작업지시자 승인을 merge 조건으로 두고 merge 뒤 별도 승인으로 원 PR을 close한다.
