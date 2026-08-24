---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5993 review - #5875 중첩 표 글자 캡션 복원

## 접수

- PR: <https://github.com/edwardkim/rhwp/pull/5993>
- author: `planet6897`
- source head: `afbb08f41b9ab29d5f1c224e544928aa79888972`
- integration base: `upstream/devel@f4ba7c565e81b0236ca1c52266ff75540b164fa7`
- local branch: `review/open-ci-green-20260824`
- verdict: 수용 권고. 통합 PR 생성은 작업지시자 사전 승인 대기.

## 검토

셀 안 중첩 표의 글자 캡션을 그려 2181727 문서 7, 8쪽 표 제목 5개를 복원하는 renderer 보정이다.
샘플은 `samples/issue5875/`에 포함되어 있고, baseline row는 `tests/fixtures/ir_field_sweep_baseline.tsv`
에 함께 들어온다.

## 로컬 검증

- 전체 nextest: 8292 passed, 42 skipped
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`: 통과
- `git diff --check`: 통과

## 판단

#5996 충돌 해소 때 `ir_field_sweep_baseline.tsv`의 #5875 두 행을 함께 보존했다. 수용 권고.
