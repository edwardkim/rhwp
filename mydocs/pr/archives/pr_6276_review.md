---
kind: pr-review
status: pending-push-and-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29
---

# PR #6276 self-review — `src/main.rs` 잔여 렌더 경계 분리

## 라우팅과 접수 메타데이터

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`, `multi_pr_update_branch.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.

| 항목 | 2026-08-29 확인값 |
| --- | --- |
| PR / 작성자 | [#6276](https://github.com/edwardkim/rhwp/pull/6276) / [@postmelee](https://github.com/postmelee) |
| base / head | `devel` / `task_m100_3789-render-boundary` |
| 원격 head | `3e439a534dd33d77aa286d9ed037340a92181cf3` |
| 로컬 리뷰 보정 | `eeffb3e8fa86d80d149531425de0b60e3b6e5e23` |
| 로컬 source candidate | `16ea38cd2bd44ff0b7cbfaab84fbdf9d62af55f7` |
| 기준 `upstream/devel` | `f6a6bee8f3ef66f43f85f74e9a286a2669db8f35` |
| source candidate 규모 | 35 files, +1,900 / -546, 17 first-parent commits |
| 원격 상태 | Open Draft, `MERGEABLE / CLEAN`; 이전 head CI 성공 |
| source candidate 관계 | `ahead 17 / behind 0`, 보정 candidate push 전 |
| 관련 issue | `Closes #3789`; related #5511, #5776, #6001 |

## 최초 변경과 리뷰에서 확인한 blocker

최초 원격 candidate는 다음 source 책임을 분리했다.

- `test-caption`의 문서 mutation·직접 SVG render를
  `src/cli/commands/caption_validation.rs`로 이동
- `export-structure`와 `structure_json_value`를 `src/cli/queries/structure.rs`의 단일 authority로 이동
- vector output, batch query와 MCP structure 응답을 새 structure authority로 연결

그러나 [리뷰 comment](https://github.com/edwardkim/rhwp/pull/6276#issuecomment-5452147207)에서 최초 CI
경계 전제에 세 blocker가 확인됐다.

1. `main.rs`를 Render Diff에서 뺐지만 PDF·PNG가 공유하는 문서 로더·인증 입력 helper가 남아 있었다.
2. `caption_validation.rs`는 direct SVG caller여도 Render Diff workflow가 실행하지 않는 경로였다.
3. root 음성 가드가 `.render_page_svg(` 한 spelling만 검사해 다른 page renderer 진입점을 놓쳤다.

direct caller 전수 검사가 `outputs/`에만 한정된 구조 문제, rustfmt brace에 결합된 dispatch assertion과
`vector.rs`의 낡은 module doc도 함께 확인했다. `test-caption`이 모든 mutation 실패 뒤에도 exit 0인 문제는
이동 전부터 있던 동작이므로 별도 false-pass 이슈 후보로 분리했다.

## 리뷰 보정

blocker와 같은 원인의 구조 문제를 `eeffb3e8f`에서 다음처럼 보정했다.

- `load_document`, `load_document_core`, `classify_hwp_error`, 입력·출력 비밀번호 상태와 전역 인증 pre-scan을
  `src/cli/document_io.rs`로 이동했다.
- `hu_to_mm`, `hu_to_mm_i`를 `src/cli/units.rs`로 이동했다.
- `src/main.rs`에는 command parse·dispatch·composition만 남겼고 `.render_page_` family 음성 가드를 뒀다.
- Render Diff와 Native Skia가 실제 공유하는 입력 경계 `document_io.rs`를 workflow·classifier·policy
  mirror에 등록했다.
- 현행 workflow가 실행하지 않는 caption·vector는 일반 Rust, raster는 Native Skia로 유지했다.
- `src/cli/**/*.rs` direct page renderer caller를 전수 검색하고 explicit consumer bucket을 요구한다.
- dispatch assertion은 arm 존재와 module API 호출을 분리해 rustfmt의 brace 접힘에 의존하지 않는다.
- `vector.rs` module doc을 SVG·render tree 출력 소유에 맞췄다.

공개 command spelling, stdout/stderr, exit code, SVG naming, structure JSON schema와 provenance, renderer
알고리즘은 바꾸지 않았다.

## 현재 CI 영향 행렬

| 변경 경로 | Rust | Render Diff | Native Skia | 판단 |
| --- | ---: | ---: | ---: | --- |
| `src/main.rs` | true | false | false | 일반 root composition |
| `src/cli/document_io.rs` | true | true | true | PDF·PNG 공유 문서 입력 |
| `src/cli/commands/caption_validation.rs` | true | false | false | 현행 workflow 미소비 |
| `src/cli/queries/structure.rs` | true | false | false | 비렌더 JSON query |
| `src/cli/outputs/mod.rs`, `pdf.rs` | true | true | true | #5776 실제 Render Diff consumer |
| `src/cli/outputs/raster.rs` | true | false | true | Native Skia consumer |
| `src/cli/outputs/vector.rs`, `src/cli/units.rs` | true | false | false | 현행 workflow 미소비 |

Render Diff trigger, trusted classifier와 policy mirror는 같은 경로를 사용한다. classifier test는 직접 CLI
page renderer caller의 explicit decision을 강제하고, fixture는 main·caption·structure negative와
document input Render Diff·Native Skia positive를 고정한다.

## 대형 PR 판정

source candidate는 1,000줄을 넘으므로 `rework_and_exceptions.md`의 대형 PR 경로를 유지한다. 현재
`upstream/devel...16ea38cd2` 기준 35 files, +1,900/-546이며 이 중 `mydocs/`는 +1,107/-0, 비문서는
+793/-546이다. source 책임 이동, 최초 CI 보정, 리뷰 보정과 각 current-base merge가 독립 commit으로 남아
있어 판단 계보는 추적 가능하다.

문서 비중이나 로컬 전체 회귀를 이유로 admin merge하지 않는다. 최신 원격 candidate의 Full required CI,
mergeability와 작업지시자의 별도 merge 승인을 확인한다.

## 최신화와 로컬 검증

원격 review head 뒤 `upstream/devel@96da78a9c`와 `@f6a6bee8f`를 각각 `2357800d2`, `16ea38cd2`로
current-base merge했다. 두 병합 모두 충돌이 없었고, 최신 devel의 관련 경로 직접 변경도 없었다.

| 검증 | 결과 |
| --- | --- |
| #3789 focused Rust | 113/113 PASS |
| 전체 release-test nextest | runnable 8,553/8,553 PASS, 43 ignored, 실패 0 |
| 필수 `cargo clippy --locked --all-targets ... -D warnings` | PASS |
| classifier·policy Node | 69/69 PASS |
| Render Diff·CI impact workflow Python | 37/37 PASS |
| actionlint / Cargo fmt / diff check | PASS |
| review worktree manifest | 1,017 sources / 4,503 attrs / 48 targets, PASS |
| source unit-tier | 4,221 tests / 299 modules, PASS |

nextest 실행은 `--no-fail-fast` 종료 코드 0을 확인했다. TTY가 최종 집계 행을 남기지 않아 JSON inventory
8,596개에서 ignored 43개를 대사해 runnable 8,553개를 확인했다. generated integration suite는 ignored
상태이며 제출 대상에 포함하지 않는다.

## 시각 검증 판정

이 PR은 renderer/layout/typeset/paint 알고리즘, HWP/HWPX/PDF fixture, golden이나 특정 페이지의 시각 개선을
바꾸거나 주장하지 않는다. 기존 caller와 공유 입력 helper를 소유 모듈로 move-only 이전하고 CI path
ownership을 조정한다. 따라서 별도 visual sweep과 WASM build는 로컬 추가 게이트에서 제외했고, 전체 Rust
회귀와 CI 정책 계약으로 동작 보존을 확인했다.

## GitHub Actions

현재 원격 head `3e439a534`의 CI, CodeQL, Render Diff, Native Skia, Adapter inter-diff와 Proptest는 모두
성공했다. 하지만 이 결과는 리뷰 보정 전 head의 증적이다. 로컬 candidate `16ea38cd2`와 후속 문서 commit은
아직 원격에 없으므로 성공으로 기록하거나 trusted reuse 대상으로 간주하지 않는다.

이 PR은 workflow와 CI impact classifier·policy를 바꾼다. 보정 candidate를 push하면 최신 head의 Full CI,
CodeQL과 Render Diff·Native Skia 선택 결과를 확인한다. required check가 누락·pending·failed이거나 exact
head identity가 맞지 않으면 merge하지 않는다.

## 발견 사항과 잔여 위험

- 리뷰 1·2·3번 blocker와 4·6·7번 구조·문서 지적은 로컬에서 보정·검증 완료했다.
- `test-caption` all-fail exit 0은 이동 전 동작을 보존한 결과로 남아 있다. 에이전트 false-pass를 막는 별도
  이슈가 필요하지만 이번 책임 이동 PR에 제품 동작 변경을 섞지 않는다.
- 앞으로 Render Diff가 `export-svg`나 `test-caption`을 실제 소비하면 consumer 변경과 같은 commit에서
  vector·caption 분류를 승격해야 한다. 현재는 파일명이나 direct call만으로 과분류하지 않는다.
- Stage 1~4 완료 문서는 구현 당시가 아니라 사후 작성됐다. 원 이력을 재작성하지 않고
  `task_m100_3789_hyper_waterfall_recovery.md`에 부분 미준수와 Stage 9 실제 계보를 기록했다.

## 최종 권고

현재 판정은 **보정 candidate push 권고, merge 보류**다. 로컬에서는 리뷰 blocker와 비례 회귀가 모두
해소됐지만 원격 PR은 아직 이전 Draft head다. 작업지시자의 push 승인 뒤 보정 candidate와 이 기록을
반영하고, 최신 head Full required checks와 `MERGEABLE / CLEAN`을 확인한 다음 보정 완료 comment를 게시한다.
PR을 ready로 전환하거나 merge하는 것은 그 결과를 보고 별도로 결정한다.
