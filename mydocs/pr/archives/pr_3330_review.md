# PR #3330 검토 기록 — actions/github-script 9

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: 12f1421ebeb66b4f033ab074a473b5724342f23d
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3330](https://github.com/edwardkim/rhwp/pull/3330) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `github_actions` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +5/-5, 4 files, 1 commit |
| head | `12f1421ebeb66b4f033ab074a473b5724342f23d` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | Actions Route B 통합에 수용 |

metadata와 CI는 문서 작성 시점 참고값이다. 최종 판단 전 최신 integration PR head에서 다시 확인한다.

## 변경과 통합

- `actions/github-script`를 v8에서 v9로 올린다.
- 대상은 CI/CodeQL/Render Diff preflight와 devel push 뒤 issue close workflow다.
- source `12f1421ebeb66b4f033ab074a473b5724342f23d`를 `git cherry-pick -x`로 적용해
  integration `f1bcf5f510258e09bd2b30e71c78733683f8b9b1`을 만들었다.
- script 본문, 입력, 권한은 바꾸지 않았고 원 저자·`Signed-off-by`·source SHA를 보존했다.

## 검증과 리스크

- 원 PR 최신 head의 `Build & Test`, CodeQL, Render Diff는 문서 작성 시점 성공이다.
- 누적 tree의 diff는 `uses: actions/github-script@v9` 교체만 포함한다.
- `actionlint -shellcheck=`와 `git diff --check`가 성공했다.
- PR 이벤트 preflight 경로는 integration PR CI에서 실행 가능하지만
  `close-issues-on-devel-push.yml`의 실제 push mutation은 PR CI가 실행하지 않는다.
- renderer/layout/asset 변경이 없어 visual sweep 대상이 아니다.

**권고**: #3330은 직접 merge하지 않고 Actions integration PR로 대체한다. 최신 integration head의
full CI와 작업지시자 승인을 merge 조건으로 두고, merge 뒤 별도 승인으로 원 PR을 close한다.
