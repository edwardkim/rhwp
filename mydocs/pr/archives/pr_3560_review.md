# PR #3560 검토 — review-only 기록의 Render Diff 재실행 방지

- 검토일: 2026-07-29
- PR: [#3560](https://github.com/edwardkim/rhwp/pull/3560)
- 관련 이슈: [#3559](https://github.com/edwardkim/rhwp/issues/3559)
- 작성자 / reviewer: `@jangster77` / `@jangster77` (collaborator self-merge 후보)
- base / 원 code head: `devel` / `c4120122b916ca8fc393d81ef55c8e6b16bfd562`
- 규모: 6 files, +56 / -3 (review 기록 추가 전)

## 변경 범위와 판정

`render-diff.yml`의 pull request trigger에서 `mydocs/**`를 제외한다. review·오늘할일·검토 asset은
render 입력이 아니므로, renderer/paint와 무관한 code PR 뒤에 review-only commit이 추가됐다는 이유만으로
`Canvas visual diff`를 다시 시작해서는 안 된다.

renderer, model, paint, Studio, renderer 관련 script와 Render Diff workflow 자체의 기존 trigger는 그대로
유지한다. 따라서 실제 render 영향 변경은 계속 Canvas visual diff를 실행한다. 이번 PR은 workflow 자체를
바꾸므로 원 code CI에서 Canvas visual diff가 실행되어 성공한 것도 확인했다.

새 `scripts/tests/test_render_diff_workflow.py`는 pull request trigger에 renderer 경로가 남아 있고
`mydocs/**`가 다시 추가되지 않는 것을 고정한다. CI Lint가 이 Python unittest를 실행하므로 workflow
조건의 단순 회귀도 PR CI에서 차단한다.

## Markdown 운영 절차

[#3548](https://github.com/edwardkim/rhwp/pull/3548) 후속 comment에서 셸 큰따옴표 안의 `\n`이 literal로
게시된 사례를 재발 방지 대상으로 함께 기록했다. 공통 PR review 절차는 다단락 본문에 `--body-file`을
쓰고, issue close와 다단락 comment를 분리하며, 게시 뒤 API `body`에서 literal `\\n` 부재를 확인하도록
명시한다. merge 후 manual의 issue close 예시도 이 공통 절차로 연결했다.

## 검증

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest scripts/tests/test_render_diff_workflow.py` | 1 passed |
| `python3 scripts/check_markdown_links.py mydocs/manual/pr_review/post_merge.md mydocs/manual/pr_review_workflow.md` | 내부 상대 링크 이상 없음 |
| `actionlint .github/workflows/ci.yml .github/workflows/render-diff.yml` | 변경과 무관한 기존 Render Diff shellcheck 경고 1건만 확인; 현행 `devel`에도 동일 |
| GitHub Actions — CI | preflight, Lint(새 static test 포함), archive, Native Skia, default-feature 8 shards, `Build & Test` 모두 success |
| GitHub Actions — CodeQL | preflight와 JavaScript/TypeScript·Python·Rust 분석, aggregate 모두 success |
| GitHub Actions — Render Diff | preflight와 `Canvas visual diff` success |

이번 변경은 workflow trigger·정적 정책 검사·운영 문서만 바꾸며 HWP/HWPX serializer 또는 renderer 출력
계산을 바꾸지 않는다. Canvas 결과는 workflow 변경 자체의 CI 검증으로 확인했고, 별도 PDF/SVG review asset은
merge 판단에 필요하지 않다.

## 권고와 merge 전 조건

**권고: 수용.** 원 code candidate `c4120122b916ca8fc393d81ef55c8e6b16bfd562`의 full CI가 모두
success인 것을 확인했다. 이 review-only 기록을 추가한 latest head의 preflight와 `Build & Test` aggregate가
success이고 mergeable 상태가 유지되는지 다시 확인한 뒤, 작업지시자 승인 범위에서 squash merge한다. merge 후
#3559 상태, PR comment, devel sync와 branch/worktree 정리를 확인한다.
