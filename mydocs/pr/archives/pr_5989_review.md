---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5989 self-review — 분리된 CLI render output adapter CI 영향 분류 복구

## 라우팅과 접수 메타데이터

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- initial full candidate: `a0733fef459b6bc45729a2266e1da09ce47a8576`
- current-base merge head: `7ebf6fe50c0c36b0f8dc298547e0e3e3b96a752d`
- review correction candidate: `bacec40ee5f9a15e086e0f82e6edb9c074f9d3ec`

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5989](https://github.com/edwardkim/rhwp/pull/5989) / [@postmelee](https://github.com/postmelee) |
| base / head | `devel` / `codex/task-m100-5776-render-impact` |
| 보정 candidate 기준 규모 | 12 files, +447 / -18, 6 commits |
| 상태 | Open, non-draft, 직전 head `MERGEABLE/CLEAN`; 보정 head Actions 재확인 필요 |
| 관련 issue | `Closes #5776`; related #3789, #5511 |

`upstream/devel@01e2e7422`을 두 번째 parent로 병합한 `7ebf6fe50`에서 current-base merge tree와 Full
Actions가 통과했다. 리뷰 피드백 보정은 그 위의 `bacec40ee`에 코드·계약 테스트만 분리하고, 이 문서는 그
candidate 뒤의 review-only 기록으로 잇는다. renderer·layout·paint, 제품 source, sample, 기준 PDF와 visual
fixture를 변경하지 않으므로 시각 증적 경로는 적용하지 않았다. 이 PR은 workflow와 impact policy를 바꾸므로
trailing commit의 재사용은 `review_only_fast_pass.md` A.1의 trusted controller 조건을 적용하며, 조건이
불완전하면 Full CI fallback 결과를 기다린다.

## 변경 범위와 self-review

#5511에서 render output handler가 `src/main.rs`에서 `src/cli/outputs/`로 이동했지만 Render Diff trigger,
trusted classifier와 policy mirror의 소유 경계가 따라가지 않아 adapter-only PR의 검증이 빠질 수 있었다.

| 경로 | 확인한 실제 consumer | candidate 영향축 |
| --- | --- | --- |
| `src/cli/outputs/mod.rs` | PDF·raster·vector의 sibling resource 판정 | Rust + Render Diff + Native Skia |
| `src/cli/outputs/pdf.rs` | PDF report와 direct `export-pdf` CLI test | Rust + Render Diff + Native Skia |
| `src/cli/outputs/raster.rs` | Native Skia `cli_exit_codes_native`의 `export-png` | Rust + Native Skia |
| `src/cli/outputs/vector.rs` | 일반 Rust integration의 vector·structure export | 일반 Rust 유지 |

검토 결과:

- Render Diff `pull_request.paths`와 policy mirror에는 직접 consumer인 `mod.rs`, `pdf.rs`가 같은 순서로
  추가됐다. raster와 vector는 workflow trigger에서 제외돼 불필요한 Canvas lane을 만들지 않는다.
- classifier version `4`에서 mod/PDF는 `classified:rust-render`, raster는
  `classified:native-skia-rust`, vector는 `classified:rust`로 고정됐다.
- adapter별 fixture, classifier 행렬, workflow 포함·제외와 mirror 일치 테스트가 같은 계약을 검증한다.
- `src/cli/outputs/**/*.rs` 전수 inventory는 현재 adapter 8개가 render+native, native-only, plain-Rust의
  서로 겹치지 않는 세 버킷과 정확히 일치하는지 확인한다. 향후 중첩 모듈 또는 새 adapter가 생기면 명시적
  버킷 갱신 전까지 테스트가 실패한다.
- 추적 파일 전수에서 `classification_status=classified`이고 `render_required=true`인 경로는 반드시
  Render Diff workflow가 실행된다는 일반 불변식을 추가했다. fail-closed인 `full` 분류는 이 조건에서
  의도적으로 제외했다.
- `NATIVE_SKIA_RUST_FILES` 주석은 제품 raster 경계와 integration target·support를 함께 소유한다는 현재
  집합의 역할에 맞게 고쳤다.
- #3789가 정리할 `src/main.rs`의 blanket 경계는 이번 false-negative 수정에서 제거하지 않고
  `fail-closed:main-render-boundary`로 유지한다.
- Rust 제품 source와 renderer output을 바꾸지 않으며 cache, runner, secret, branch protection과
  baseline도 변경하지 않는다.

추가 blocker는 발견하지 않았다. 향후 Render Diff가 vector native capture를 실제 소비하게 되면 그
workflow consumer 변경과 함께 `vector.rs` 영향축을 다시 넓혀야 한다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| classifier·policy Node tests | 65/65 통과 |
| CI·CodeQL·policy·Render Diff Python workflow contracts | 68/68 통과 |
| `actionlint .github/workflows/render-diff.yml` | 통과 |
| suite manifest / `cargo fmt --all` / `cargo fmt --all -- --check` | 검증용 generated suite 32개 준비 후 모두 통과 |
| `git diff --check` | 통과 |
| generated artifacts | 32 harnesses와 ignored manifest 제거, PR에 미포함 |

O2 CI workflow 범위에 따라 workflow 구문·정책·required check 영향을 검증했다. Rust source, renderer,
fixture가 없어 release-test, clippy, WASM, sample SVG와 시각 sweep은 실행 대상이 아니다. 조회 시점의
`devel`에는 classic branch protection rule, repository ruleset 또는 GitHub-enforced required status
context가 없었으며, 이 사실을 workflow 자체의 성공 조건이 없다는 뜻으로 확대 해석하지 않았다.

## GitHub Actions

current-base merge head `7ebf6fe50`의 Full Actions는 모두 성공했다.

| workflow | run | `7ebf6fe50` 결과 |
| --- | --- | --- |
| CI | [32691575411](https://github.com/edwardkim/rhwp/actions/runs/32691575411) | 성공 — Build & Test 포함 |
| CodeQL | [32691575234](https://github.com/edwardkim/rhwp/actions/runs/32691575234) | 성공 — Rust·Python·JavaScript/TypeScript 포함 |
| Render Diff | [32691575207](https://github.com/edwardkim/rhwp/actions/runs/32691575207) | 성공 — Canvas visual diff 포함 |
| Proptest roundtrip | [32691575200](https://github.com/edwardkim/rhwp/actions/runs/32691575200) | 성공 |
| Adapter inter-diff | [32691575198](https://github.com/edwardkim/rhwp/actions/runs/32691575198) | 성공 |

리뷰 보정 candidate `bacec40ee`는 classifier와 계약 테스트를 바꾼 새 code head다. 이 문서와 오늘할일은
그 뒤의 `mydocs/` 한정 single-parent trailing 기록으로 push한다. 원 PR이 CI execution surface를 바꾸므로,
`bacec40ee`의 Full CI·CodeQL·Render Diff가 같은 PR·branch·SHA에서 성공하고 기본 branch의
`CI Impact Policy Controller`가 exact trailing head에 trusted status를 발행해야만 A.1 재사용으로 판정한다.
조건이 불완전하면 최신 trailing head의 Full CI fallback 결과를 기다린다.

## 최종 권고

경로별 분류는 파일명 추정이 아니라 실제 consumer와 일치하며 workflow·classifier·policy의 세 경계를 계약
테스트로 고정했다. #3789의 false-positive 경계를 이번 PR에 섞지 않았고 기존 fail-closed도 보존했다.

self-review 보정은 **완료 / 새 head CI 대기 조건부 merge 권고**다. `bacec40ee`와 trailing head의 trusted
Actions 또는 Full fallback, 최신 `MERGEABLE/CLEAN`, 작업지시자의 별도 merge 승인을 확인하기 전에는
merge하지 않는다.

## 통합 체리픽 검토 - 2026-08-24

open PR 중 CI가 통과한 항목만 모은 `review/open-ci-green-20260824` 통합 후보에 #5989 최신 head
`bd03b65441379806ab8e8102f963396e914e3f53`를 포함했다. `mydocs/orders/20260824.md` 충돌은 #5985 기록과
#5776 기록을 모두 보존하는 방식으로 해소했다.

통합 후보의 전체 nextest 8292 passed / 42 skipped, Studio test/build, CI impact tests, render-diff workflow
unittest, 기본 all-targets clippy가 통과했다. #5989는 통합 PR에서 수용 권고로 처리한다. 통합 PR 생성은
작업지시자 사전 승인 전까지 진행하지 않는다.
