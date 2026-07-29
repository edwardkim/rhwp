# PR #3336 검토 기록 — actions/upload-pages-artifact 5

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: b9a36c58fee963a6cd281a55477c3ee137ecd564
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3336](https://github.com/edwardkim/rhwp/pull/3336) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `github_actions` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +1/-1, 1 file, 1 commit |
| head | `b9a36c58fee963a6cd281a55477c3ee137ecd564` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | Pages artifact Actions Route B 통합에 수용 |

## 변경과 통합

- `.github/workflows/deploy-pages.yml`의 `actions/upload-pages-artifact`를 v3에서 v5로 올린다.
- source `b9a36c58fee963a6cd281a55477c3ee137ecd564`를 `git cherry-pick -x`로 적용해
  integration `d95a1c37af44cd83c89b3b066b6a21044f848e9e`를 만들었다.
- artifact path와 Pages job 연결은 바꾸지 않았고 원 저자·`Signed-off-by`·source SHA를 보존했다.

## 검증과 리스크

- 원 PR 최신 head의 `Build & Test`와 CodeQL은 문서 작성 시점 성공이다.
- 누적 tree의 diff는 action version 교체만 포함하며 `actionlint -shellcheck=`와 `git diff --check`가 성공했다.
- 실제 Pages artifact upload와 deploy 연결은 PR CI가 실행하지 않으므로 merge 뒤 최초 Pages run을 확인한다.
- renderer/layout/asset 변경이 없어 visual sweep 대상이 아니다.

**권고**: #3336은 직접 merge하지 않고 Actions integration PR로 대체한다. 최신 CI와 작업지시자 승인을
merge 조건으로 두며 merge 뒤 별도 승인으로 원 PR을 close한다.
