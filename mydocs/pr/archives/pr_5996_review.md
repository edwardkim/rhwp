---
kind: pr-review
status: accepted-with-maintainer-correction-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5996 review - #5870 빈 host 자리차지 표 흐름 전진

## 접수

- PR: <https://github.com/edwardkim/rhwp/pull/5996>
- author: `planet6897`
- source head: `5500c313251bfb2e3b35aedc34d8a548dbc1ff47`
- integration base: `upstream/devel@f4ba7c565e81b0236ca1c52266ff75540b164fa7`
- local branch: `review/open-ci-green-20260824`
- verdict: 메인터너 보정 포함 수용 권고. 통합 PR 생성은 작업지시자 사전 승인 대기.

## 검토

빈 host 자리차지 표의 흐름 전진에 `v_off`와 바깥 여백을 계상해 저장 사다리 물리 일치 증거를 강화한다.
샘플은 `samples/issue5870/empty_host_float_flow_advance.hwp`에 포함되어 있고, 회귀 테스트는
`tests/cases/issue_5870_empty_host_float_flow_advance.rs`로 들어온다.

## 메인터너 보정

체리픽 중 `src/renderer/float_placement.rs`와 `tests/fixtures/ir_field_sweep_baseline.tsv`가 #5982,
#5993 변경과 충돌했다. 보정 내용은 다음과 같다.

- `native_empty_host_cellbreak_fragment_repeats_outer_margin`와
  `empty_host_physical_ladder_extras_hu`는 담당 조건이 다르므로 둘 다 보존했다.
- `ir_field_sweep_baseline.tsv`는 #5870 행과 #5875 두 행을 모두 보존했다.

## 로컬 검증

- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`: 통과
- 전체 nextest: 8292 passed, 42 skipped
- `git diff --check`: 통과

## 판단

충돌은 메인터너 보정으로 해소했고 기본 검증 경로가 통과했다. 수용 권고.
