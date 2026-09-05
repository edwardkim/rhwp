---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5796 검토 - skill Markdown LF 체크아웃 고정

| 항목 | 내용 |
| --- | --- |
| PR / 작성자 | [#5796](https://github.com/edwardkim/rhwp/pull/5796) / `kevin9327` |
| source head / 적용 commit | `aaa8ffdf3389fcade5271d3bc273771fc24674e4` / `d5cfed664` |
| 관련 issue | [#5795](https://github.com/edwardkim/rhwp/issues/5795) |
| GitHub 상태 | Open, non-draft, `MERGEABLE`; source CI 성공 |
| 라우팅 | `maintainer_general` + `intake_and_review` + `multi_pr_update_branch` |

Windows `core.autocrlf=true` checkout에서 Rust skill frontmatter contract가 `---\\n` 대신 `---\\r\\n`을 읽어
실패하던 문제다. `.gitattributes`에 `.agents/skills/**/*.md`, `.agents/skills/**/*.md`의 `text eol=lf`를
명시해 checkout 바이트를 고정한다. TSV/CSV fixture 범위는 건드리지 않는다.

통합 후보에서 `git check-attr -a`와 `git ls-files --eol`로 두 경로의 attribute `text`, `eol=lf`와
index/worktree LF를 재확인했다. 코드 런타임 변경은 없지만 통합 후보 Full CI의 Lint, CodeQL과 archive
worker가 성공했다.

**수용 권고.** merge 후 #5795의 자동 close 상태를 확인하고 merge SHA와 Windows checkout 보정 범위를
comment로 남긴다.
