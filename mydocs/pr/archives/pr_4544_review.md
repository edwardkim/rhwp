---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4544 리뷰 - 투명성 앵커

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4544](https://github.com/edwardkim/rhwp/pull/4544) · @kevin9327 |
| base / 최신 head | `devel` / `2c5af8d8ccd57f0d6ed5ff8e0d27eaed7f71f991` |
| 규모 | 10,105 추가 / 83 삭제, 76 파일, 18 commit |
| 작성 시점 CI | GitHub required `Build & Test` 통과. merge 전 최신 상태 재확인 필요. |

## 범위와 메인터너 보정

append-only anchor, checkpoint, Merkle path와 `anchoredOk` 계약을 추가한다. 최신 source의
`anchor add -> checkpoint` provenance 보강과 `is_multiple_of` clippy 보정은 누적 후보에 이미 존재해
cherry-pick이 빈 변경으로 확인됐다. 생성 문서는 anchor만 최상위 명령으로 두고 실제 subcommand를 가짜
명령 장으로 만들지 않도록 `75273093d`에서 정합화했다.

**권고: 최신 head와 required check를 merge 직전에 재확인하고 통합 수용.**
