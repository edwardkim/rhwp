# PR #3333 검토 기록 — actions/setup-node 7

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: 83a505dd5466e8317730cb42057a5fd3d6272626
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3333](https://github.com/edwardkim/rhwp/pull/3333) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `github_actions` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +7/-7, 5 files, 1 commit |
| head | `83a505dd5466e8317730cb42057a5fd3d6272626` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | Node setup Actions Route B 통합에 수용 |

## 변경과 통합

- CI, Pages, renderer sweep, npm publish, Render Diff의 `actions/setup-node`를 v6에서 v7로 올린다.
- source `83a505dd5466e8317730cb42057a5fd3d6272626`을 `git cherry-pick -x`로 적용해
  integration `38733dcf6594fb8235cd9dbaad040f812ac5e2fc`를 만들었다.
- `node-version`, cache 입력과 workflow 조건은 바꾸지 않았고 원 저자·`Signed-off-by`·source SHA를 보존했다.

## 검증과 리스크

- 원 PR 최신 head의 `Frontend package gates`, `Build & Test`, CodeQL, Render Diff는 문서 작성 시점 성공이다.
- 누적 tree의 diff는 action version 교체만 포함하며 `actionlint -shellcheck=`와 `git diff --check`가 성공했다.
- PR에서 실행되는 frontend/Render Diff Node setup은 integration PR CI로 재검증한다.
  publish/deploy 전용 경로는 실제 event에서 추가 확인이 필요하다.
- renderer source나 fixture를 바꾸지 않아 visual sweep 대상이 아니다.

**권고**: #3333은 직접 merge하지 않고 Actions integration PR로 대체한다. 최신 integration head CI와
작업지시자 승인을 merge 조건으로 두고 merge 뒤 별도 승인으로 원 PR을 close한다.
