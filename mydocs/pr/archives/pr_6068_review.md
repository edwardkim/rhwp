---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6068 review - TAC 표 하한 합 초과 행 성장 (#6030)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6068](https://github.com/edwardkim/rhwp/pull/6068) |
| 작성자 | [@kevin9327](https://github.com/kevin9327) |
| 원 head | `07b86e70e1b33c362c73bc64cb0199fc17fddec1` |
| 통합 적용 commit | `ade615df6`, `f329221b1`, `9bd2b5533` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI 22 success/4 skip/CodeQL neutral |
| 판정 | **수용 권고** |

## 검토 요약

TAC 표에서 셀 내용의 최소 하한 합이 선언 행 높이를 넘는 경우, 마지막 선택지 줄이 clip되는 문제를
행 성장으로 보정한다. 후속 commit에서 `src` 내부 `#[cfg(test)]` 모듈을 제거해 unit-test-tier
게이트를 회복했고, 거대 overfill 표가 균일 축소를 유지하도록 fallback 범위를 소량 초과로 제한했다.

원 PR comment에 남은 CI Lint 실패와 Archive B 실패는 최신 head의 후속 commit에서 해소됐고,
최신 GitHub checks는 성공 상태다.

## 검증 근거

- 원 PR CI: 22 success, 실패 0, CodeQL neutral
- 통합 후보 로컬 검증:
  - `cargo check --profile release-test --target-dir target/pr-review --tests`: 통과
  - `cargo fmt --check`: 통과
- 원 PR의 before/after 증적:
  - `mydocs/report/cell-row-grow-lastline-6030/before.png`
  - `mydocs/report/cell-row-grow-lastline-6030/after.png`

## 권고

원 PR의 추가 push가 기존 CI 실패 원인을 설명하고 수정했으며, 통합 후보에서도 컴파일·포맷 문제가 없다.
메인터너 보정 없이 통합 PR에 포함해 수용 가능하다.
