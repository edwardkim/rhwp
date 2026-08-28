---
kind: pr-review
status: pending-ci-and-merge-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6276 self-review — `src/main.rs` 잔여 렌더 경계 분리

## 라우팅과 접수 메타데이터

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#6276](https://github.com/edwardkim/rhwp/pull/6276) / [@postmelee](https://github.com/postmelee) |
| base / head | `devel` / `task_m100_3789-render-boundary` |
| code candidate | `764439a15c8a02656e84e5a6853fa2de2b740499` |
| 기준 `upstream/devel` | `1a43a507c9daa3bfab799d443bd982d5e3dd6d59` |
| 규모 | 30 files, +1,420 / -314, 12 commits |
| 상태 | Open, non-draft, `MERGEABLE / BLOCKED`; 최신 trailing head와 merge 직전 재확인 필요 |
| 관련 issue | `Closes #3789`; related #5511, #5776, #6001 |

## 변경 범위와 self-review

`src/main.rs`에 남아 있던 서로 다른 두 책임을 실제 소유 모듈로 분리했다.

- `test-caption`의 문서 mutation·직접 SVG render는
  `src/cli/commands/caption_validation.rs`가 소유한다.
- `export-structure`와 `structure_json_value`는 `src/cli/queries/structure.rs`의 단일 authority로 옮겼다.
- vector output, batch query와 MCP structure 응답은 새 structure authority를 참조한다.
- root에는 command parsing·dispatch·composition만 남고 직접 `render_page_svg` 호출과 구조 JSON 생성은 없다.
- 공개 command spelling, stdout/stderr, exit code, SVG naming, structure JSON schema와 provenance는 바꾸지 않았다.

CI 경계는 실제 consumer 기준으로 다음과 같이 고정했다.

| 변경 경로 | Rust | Render Diff | Native Skia | 판단 |
| --- | ---: | ---: | ---: | --- |
| `src/main.rs` | true | false | false | 일반 root composition |
| `src/cli/commands/caption_validation.rs` | true | true | true | 직접 SVG render boundary |
| `src/cli/queries/structure.rs` | true | false | false | 비렌더 JSON query |
| `src/cli/outputs/mod.rs`, `pdf.rs` | true | true | true | #5776 실제 workflow consumer |
| `src/cli/outputs/raster.rs` | true | false | true | native raster consumer |
| `src/cli/outputs/vector.rs` | true | false | false | 현행 Render Diff 미소비 |

Render Diff workflow, trusted classifier와 policy mirror가 caption의 정확한 source path를 함께 추적하고,
root negative·caption positive·structure negative fixture가 이를 검증한다. #5776이 고정한 adapter inventory와
PDF/shared/raster positive mapping은 약화하지 않았다.

## 대형 PR 판정

이 PR은 1,000줄을 넘으므로 `rework_and_exceptions.md`의 대형 PR 경로를 적용한다. 총 증가분 가운데
976줄·12파일은 계획, Stage 1~8, 절차 감사와 최종 보고인 `mydocs/` 증적이고, 비문서 변경은
18파일·+444/-314다. source 이동과 CI 보정은 `17fa14198`, `514ff74bc`로 분리돼 있다.

문서 증적이 많다는 이유로 즉시 admin merge하지 않는다. 최신 trailing head의 Full CI 또는 trusted
review-only 재사용, mergeability와 작업지시자 merge 승인을 별도 cycle로 확인한다.

## 로컬 검증

제출 직전 `upstream/devel@1a43a507c`를 포함한 code candidate `764439a15`에서 다음을 완료했다.

| 검증 | 결과 |
| --- | --- |
| #3789 focused Rust | 113/113 PASS |
| 전체 release-test nextest | 8,519/8,519 PASS, 43 skip, 실패 0 |
| 필수 `cargo clippy --locked --all-targets ... -D warnings` | PASS |
| classifier·policy Node | 67/67 PASS |
| CI workflow Python | 71/71 PASS |
| actionlint / Cargo fmt / diff check | PASS |
| review worktree manifest | 995 sources / 4,469 attrs / 48 targets, PASS |
| source unit-tier | 4,221 tests / 299 modules, PASS |
| Markdown 내부 상대 링크 | PASS |

착수 계획의 `scripts/release-test.mjs`는 최신 upstream에서 제거돼 canonical 직접 nextest 명령을 사용했다.
필수 범위 밖 `--all-features` 추가 진단에서는 upstream GPU feature 조합의 `vello 0.9/0.10` 타입 불일치를
관찰했다. 이 PR은 관련 `Cargo.toml`, `Cargo.lock`, `src/renderer/gpu.rs`를 변경하지 않으며 현재 필수 clippy는
통과했다.

## 시각 검증 판정

이 PR은 renderer/layout/typeset/paint 알고리즘, HWP/HWPX/PDF fixture, golden이나 특정 문서 페이지의 시각
개선을 바꾸거나 주장하지 않는다. 기존 direct caption renderer caller를 전용 모듈로 move-only 이전하고
CI path ownership을 조정한다. 따라서 문서 visual sweep과 WASM build는 적용 대상에서 제외했으며,
caption SVG 생성 계약은 focused Rust test로 확인했다.

## GitHub Actions

code candidate `764439a15`의 PR 생성 직후 CI, CodeQL, Render Diff, Adapter inter-diff와 Proptest preflight가
시작됐다. 이 문서 작성 시점에는 완료되지 않았으므로 성공으로 기록하지 않는다.

이 PR은 workflow와 CI impact classifier·policy를 바꾼다. 따라서 trailing review-only commit은
`review_only_fast_pass.md` A.1의 trusted controller가 exact code candidate의 Full CI를 증명한 경우에만
재사용할 수 있다. status가 누락·pending·failed이거나 조건이 불완전하면 최신 trailing head의 Full CI
fallback 결과를 기다린다.

## 발견 사항과 잔여 위험

- 차단 결함은 발견하지 못했다.
- 앞으로 Render Diff가 vector native capture를 실제 소비하면 `vector.rs`를 consumer 변경과 함께 승격해야
  한다. 현재는 파일명만으로 과분류하지 않는다.
- extra all-features GPU 컴파일 문제는 #3789 범위 밖 upstream 관찰이다. 필요하면 별도 이슈로 분리하되 이
  PR의 필수 게이트 통과로 숨기지 않는다.
- Stage 1~4 완료 문서는 구현 당시가 아니라 사후 작성됐다. 원 이력을 재작성하지 않고
  `task_m100_3789_hyper_waterfall_recovery.md`에 부분 미준수와 이후 보정 계보를 기록했다.

## 최종 권고

현재 self-review는 **최신 head CI 대기 조건부 merge 권고**다. source ownership과 CI 영향축은 실제
consumer에 맞고 로컬 필수 회귀에서 차단 결함을 발견하지 못했다. 최신 trailing head의 required checks,
`MERGEABLE / CLEAN`과 작업지시자의 별도 merge 승인을 확인하기 전에는 merge하지 않는다. 1,000줄 초과
대형 PR이므로 즉시 admin merge 예외를 적용하지 않는다.
