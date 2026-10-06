---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4345 검토 — 영문 README 설치·MCP·Python 진입점

## 라우팅

base route: `maintainer_general.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `review_only_fast_pass.md`

loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`pr_review/maintainer_general.md`, `pr_review/intake_and_review.md`,
`pr_review/local_validation.md`, `pr_review/multi_pr_update_branch.md`,
`pr_review/review_only_fast_pass.md`

## 메타데이터와 적용 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4345](https://github.com/edwardkim/rhwp/pull/4345) / @kevin9327 |
| base | `devel` |
| 원 PR head | `9574ed2aa7588eee903e75fc230d4946efbb3a70` (2026-08-10 작성 시점 참고값) |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 검토 브랜치 | `review/kevin9327-20260810-pr4345` |
| 원 변경 규모 | `README_EN.md` 1파일, `+46/-0`, 커밋 1개 |
| 작성 시점 참고 상태 | `MERGEABLE` / `BLOCKED`, draft 아님, check 0개. merge 전 재확인 필요 |

원 변경은 루트 `README_EN.md`에 release binary 설치, MCP 설정, Python binding의
최소 진입점을 추가한다. source, test, workflow, fixture, baseline과 기존 asset은 바꾸지
않는다. contributor의 원 커밋은 amend, rebase 또는 squash하지 않았으며, 이 검토 기록만
그 뒤의 single-parent 메인터너 문서 커밋으로 추가한다.

구현 동작과 렌더링·레이아웃·페이지 출력에 영향이 없는 루트 문서 변경이므로 visual sweep은
대상이 아니다. 보정도 `mydocs/pr/pr_4345_review.md` 한 파일뿐이어서 별도
`pr_4345_review_impl.md`는 필요하지 않다.

## 원 head의 check 0개 원인과 최소 보정

원 head는 루트 `README_EN.md`만 변경한다. 현재 `.github/workflows/ci.yml`의
`pull_request.paths-ignore`에는 루트 `*.md`가 포함되어 있으므로 이 변경만으로는 CI workflow가
시작되지 않는다. required check를 만들 run이 없어 GitHub의 merge state가 `BLOCKED`이고 check가
0개인 상태다.

`mydocs/pr/pr_4345_review.md`는 `pull_request.paths-ignore`의 루트 `*.md` 패턴에 해당하지
않는다. 작업지시자 승인 뒤 이 trailing commit을 원 PR source head에 반영하면 `synchronize`
이벤트에서 CI가 시작될 수 있다. 다만 전체 PR diff에는 fast-pass 허용 경로가 아닌
`README_EN.md`가 계속 포함되므로 review-only fast-pass를 미리 확정하지 않는다. 최신 head에서
실제로 생성된 required checks의 결과를 최종 판단 근거로 사용한다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `python scripts/check_markdown_links.py README_EN.md --changed-from origin/devel` | 통과. 검사 문서 1개, 변경 파일 1개, 내부 상대 링크 오류 없음 |
| `python scripts/check_markdown_links.py README_EN.md mydocs/pr/pr_4345_review.md --changed-from origin/devel` | 메인터너 문서 추가 뒤 통과. 두 변경 문서의 내부 상대 링크 오류 없음 |
| `python scripts/check_document_metadata.py` | 통과. 장기 문서 521개의 메타데이터 이상 없음 |
| `git diff --check origin/devel...HEAD` | 원 head와 메인터너 문서 보정 뒤 모두 통과 |
| `git merge-base --is-ancestor origin/devel 9574ed2aa7588eee903e75fc230d4946efbb3a70` | 통과. 기준 devel이 원 head의 조상임을 확인 |
| Cargo·WASM·시각 검증 | 생략. 루트 README와 `mydocs/pr` 기록만 변경하며 실행 코드·렌더 출력 영향 없음 |

## 발견 사항과 리스크

- README 변경 자체에서는 merge를 막는 문서 내용·상대 링크·whitespace 문제를 발견하지 않았다.
- 현재 blocker는 원 head의 root-Markdown-only 변경과 CI path filter 조합으로 check가 생성되지
  않는 운영 문제다.
- 이 로컬 메인터너 commit은 GitHub에 push하지 않았다. source head 반영에는 별도 명시 승인이
  필요하다.

## 현재 권고

**최소 메인터너 문서 보정 후 조건부 merge 권고.** 원 contributor 변경은 그대로 보존하고 이
review 기록만 trailing commit으로 반영한다. 최종 merge 조건은 최신 PR head에서 required checks가
실제로 생성되어 모두 통과하고, mergeable 상태를 다시 확인하며, 작업지시자가 merge를 승인하는 것이다.
