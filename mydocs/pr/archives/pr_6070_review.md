---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6070 review - near-top 저장 리셋 잔여 쪽 분할 방지 (#5921)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6070](https://github.com/edwardkim/rhwp/pull/6070) |
| 작성자 | [@kevin9327](https://github.com/kevin9327) |
| 원 head | `a82e665f5f79c24b8177b2a37afa48e6f6400dd0` |
| 통합 적용 commit | `12074fbea` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI 21 success/3 skip/CodeQL neutral |
| 판정 | **수용 권고** |

## 검토 요약

저장 near-top 리셋이 잔여 예산에 포함될 때 불필요한 쪽 분할이 발생하는 회귀를 좁은 조건으로 막는다.
변경은 `typeset`의 쪽 분할 판단과 회귀 테스트에 제한되어 있으며, 증적 이미지로 before/after 차이를
보존했다.

## 검증 근거

- 원 PR CI: 21 success, 실패 0, CodeQL neutral
- 통합 후보 로컬 검증:
  - `cargo check --profile release-test --target-dir target/pr-review --tests`: 통과
  - `cargo fmt --check`: 통과
- 원 PR의 before/after 증적:
  - `mydocs/report/neartop-reset-fit-5921/before.png`
  - `mydocs/report/neartop-reset-fit-5921/before-p2.png`
  - `mydocs/report/neartop-reset-fit-5921/after.png`

## 권고

원 PR CI가 녹색이고, 통합 후보의 컴파일·포맷 검증도 통과했다. 메인터너 보정 없이 통합 PR에 포함해
수용 가능하다.
