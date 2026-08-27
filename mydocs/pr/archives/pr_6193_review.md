---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-27
---

# PR #6193 review - #5929 square host line advance

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6193
- 작성자: `planet6897`
- 원 PR head: `e28f977e0d6365308473ee4b7434c9173a355783`
- 통합 검토 브랜치: `review/open-prs-6178-6198-20260827`
- 기준: `upstream/devel@529ab90c25d5`
- 적용 문서: `maintainer_general`, `intake_and_review`, `local_validation`,
  `multi_pr_update_branch`, `visual_fixture_evidence`

## 검토 판단

**수용 가능**. 사다리 없는 문서의 어울림 anchor 문단에서 host line advance가 사라지는 축을
복구하며, 대표 crop에서 하단 표 선 위치가 oracle과 같은 축으로 맞는 것을 확인했다. 통합 브랜치에서
충돌이나 별도 메인터너 코드 보정은 필요하지 않았다.

## 증적과 검증

- focused: `node scripts/run-rust-test.mjs issue_5929_square_host_line_advance -- --cargo-profile release-test --target-dir target/pr-review`
  - `1 passed`, `148 skipped`
- 전체 회귀: `8438 passed`, `43 skipped`, `10 slow`
- native-skia 대표 검증:
  - `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib`: `167 passed`
  - `render_p37_direct_pdf_export`: `4 passed`
- 시각 증적 직접 확인:
  - `mydocs/report/issue-5929-square-host-line-advance/after.png`
  - `mydocs/report/issue-5929-square-host-line-advance/oracle.png`
- 증적 SHA: `mydocs/pr/assets/pr_6178_6182_6188_6193_6195_6198_visual_evidence_sha256.tsv`

## 후속

병합 후 원 PR과 관련 이슈에는 focused test, 전체 회귀, 대표 crop 직접 확인 결과를 함께 남긴다.
