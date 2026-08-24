---
kind: pr-review
status: trailing-docs-pending-ci
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

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5989](https://github.com/edwardkim/rhwp/pull/5989) / [@postmelee](https://github.com/postmelee) |
| base / head | `devel` / `codex/task-m100-5776-render-impact` |
| 초기 규모 | 10 files, +264 / -13, 3 commits |
| 상태 | Open, non-draft, `MERGEABLE`, initial full Actions 실행 중 |
| 관련 issue | `Closes #5776`; related #3789, #5511 |

단일 self PR이고 최신 `upstream/devel@ad2867708`을 직접 조상으로 가지므로 update branch·오래된 base 경로는
적용하지 않았다. renderer·layout·paint, 제품 source, sample, 기준 PDF와 visual fixture를 변경하지 않으므로
시각 증적 경로도 적용하지 않았다. 이 PR은 workflow와 impact policy를 바꾸므로 trailing commit의 재사용은
`review_only_fast_pass.md` A.1의 trusted controller 조건을 적용한다.

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
- #3789가 정리할 `src/main.rs`의 blanket 경계는 이번 false-negative 수정에서 제거하지 않고
  `fail-closed:main-render-boundary`로 유지한다.
- Rust 제품 source와 renderer output을 바꾸지 않으며 cache, runner, secret, branch protection과
  baseline도 변경하지 않는다.

추가 blocker는 발견하지 않았다. 향후 Render Diff가 vector native capture를 실제 소비하게 되면 그
workflow consumer 변경과 함께 `vector.rs` 영향축을 다시 넓혀야 한다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| classifier·policy Node tests | 64/64 통과 |
| CI·policy·Render Diff Python workflow contracts | 48/48 통과 |
| `actionlint .github/workflows/render-diff.yml` | 통과 |
| `cargo fmt --all` / `cargo fmt --all -- --check` | 검증용 generated suite 준비 후 모두 통과 |
| `git diff --check` | 통과 |
| generated artifacts | 32 harnesses와 manifest 제거, PR에 미포함 |

O2 CI workflow 범위에 따라 workflow 구문·정책·required check 영향을 검증했다. Rust source, renderer,
fixture가 없어 release-test, clippy, WASM, sample SVG와 시각 sweep은 실행 대상이 아니다. 조회 시점의
`devel`에는 classic branch protection rule, repository ruleset 또는 GitHub-enforced required status
context가 없었으며, 이 사실을 workflow 자체의 성공 조건이 없다는 뜻으로 확대 해석하지 않았다.

## GitHub Actions

initial full candidate `a0733fef4`의 preflight는 모두 성공했고 full worker는 작성 시점 실행 중이다.

| workflow | run | 작성 시점 상태 |
| --- | --- | --- |
| CI | [32688977374](https://github.com/edwardkim/rhwp/actions/runs/32688977374) | preflight 성공, full jobs 실행 중 |
| CodeQL | [32688977353](https://github.com/edwardkim/rhwp/actions/runs/32688977353) | preflight 성공, Analyze 실행·대기 중 |
| Render Diff | [32688977262](https://github.com/edwardkim/rhwp/actions/runs/32688977262) | preflight 성공, Canvas visual diff 대기 중 |
| Proptest roundtrip | [32688977203](https://github.com/edwardkim/rhwp/actions/runs/32688977203) | preflight 성공, worker 실행 중 |
| Adapter inter-diff | [32688977139](https://github.com/edwardkim/rhwp/actions/runs/32688977139) | preflight 성공, worker 실행 중 |

이 self-review와 오늘할일은 initial candidate 뒤의 `mydocs/` 한정 single-parent trailing commit이다.
원 PR이 CI execution surface를 바꾸므로, initial candidate의 Full CI·CodeQL·Render Diff가 같은 PR·branch·SHA에서
성공하고 기본 branch의 `CI Impact Policy Controller`가 exact trailing head에 trusted status를 발행해야만
A.1 재사용으로 판정한다. 조건이 불완전하면 Full CI fallback 결과를 기다린다.

## 최종 권고

경로별 분류는 파일명 추정이 아니라 실제 consumer와 일치하며 workflow·classifier·policy의 세 경계를 계약
테스트로 고정했다. #3789의 false-positive 경계를 이번 PR에 섞지 않았고 기존 fail-closed도 보존했다.

self-review는 **완료 / CI 대기 조건부 merge 권고**다. initial candidate와 trailing head의 trusted
Actions, 최신 `MERGEABLE/CLEAN`, 작업지시자의 별도 merge 승인을 확인하기 전에는 merge하지 않는다.
