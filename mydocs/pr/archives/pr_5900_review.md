---
kind: pr-review
status: accepted-pending-integration-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5900 검토 - 쪽 분할 셀 마지막 글줄 clipping (#5862)

## 접수와 범위

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5900](https://github.com/edwardkim/rhwp/pull/5900) / [@kevin9327](https://github.com/kevin9327) |
| base / source head | `devel` / `8abc28dc2ae83bf71c0e6dfa22792ccd7932bddb` |
| 규모 | 15 files, +379 / -0, 2 commits |
| 접수 상태 | non-draft, reviewer `@jangster77` 지정, 작성 시점 `MERGEABLE/CLEAN` |

쪽 분할 셀의 마지막 글줄이 clip 확장 범위를 벗어나 사라지는 renderer 결함을 고친다. 원 PR의 두
source commit은 모두 통합 branch에 적용됐으며, EOF 공백 정리는 통합 검토 중 문서 보정으로 별도 반영했다.

## 검증과 증적

- 최신 source head의 check는 22 success, 1 neutral, 4 skipped이며 failure는 없었다.
- 통합 code candidate `dbb39210ca62b22b1d9507013a2191a5c55889bf`에서 전체 nextest는
  8,201 passed, slow 3, skipped 41로 통과했다. 이 뒤의 `2fa4f3212`은 MCP 사용 문서만 바꾼 commit이다.
- [전·후·한글 2024 대조](../../report/edit_demo_5862/p8_before_after_oracle.png)는 수정 전 8쪽 하단의
  빈 영역과 수정 후 복원된 본문을 보여 준다. 수정 후는 기준의 본문 흐름과 같은 방향으로 이동한다.
- 2026-08-23 현재 `upstream/devel` merge-tree는 clean이고 `git diff --check`, `cargo fmt --all -- --check`,
  unit-tier 검사는 통과했다. 상세 상태는
  [`pr_open_ci_green_20260823_acceptance_ledger.tsv`](../assets/pr_open_ci_green_20260823_acceptance_ledger.tsv)에 보관했다.

## 판정

**수용 권고.** 마지막 글줄 복원이라는 원 PR 주장에 대해 renderer 회귀시험, source CI, 통합 전체 회귀와
시각 증적이 일치한다. 통합 PR의 최신 head CI와 작업지시자 승인을 merge 전 다시 확인한다.
