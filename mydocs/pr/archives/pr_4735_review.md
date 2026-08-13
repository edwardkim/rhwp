---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4735 검토 - 저장 조각 줄의 본문 경계 회귀와 IR baseline 등록

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4735](https://github.com/edwardkim/rhwp/pull/4735) |
| 관련 이슈 | [#4690](https://github.com/edwardkim/rhwp/issues/4690) |
| 작성자 / source | @planet6897 / `test/4690-regression-guard` |
| 원 source head | `3c995998e6dd1b759cc2364ddcb6e03457e26fe7` |
| 기준 devel | `b5c14346d0eba652764111764ae77cb959006af4` |
| 가시성 통합 branch | `review/planet6897-20260813` |
| 적용 순서 | 두 번째: `3db39c931` → `92ddf99ee` → `3c995998e`를 로컬 `75dce4697` → `e33e07061` → `7a345db90`로 cherry-pick |
| reviewer | @jangster77 지정 완료 |

검토 경로는 `maintainer_general`이며, `intake_and_review`, `local_validation`,
`multi_pr_update_branch`를 함께 적용했다. 재현본
`samples/issue4690/30098_indent_over_stored_cs.hwp`의 p3, paragraph index 48에 저장된
column start와 width가 본문 경계 밖이나 음수 폭으로 변하지 않도록 좌표 계약을 추가한다.

## CI 실패와 보정

원 code head `3db39c931`의
[Default-feature tests (shard 2/3)](https://github.com/edwardkim/rhwp/actions/runs/31702315505/job/94457636844?pr=4735)은
새 HWP fixture가 IR field sweep 코퍼스에 자동 포함됐으나 HWP5 재생성 차이의 baseline 행이 없어서
실패했다. 실패 항목은 다음 한 종류다.

```text
hwp5rb  issue4690/30098_indent_over_stored_cs.hwp
sections[].paragraphs[].controls[].cells[].list_header_width_ref  124
```

`list_header_width_ref`는 HWP5 LIST_HEADER의 재생성 속성이며, 같은 normalized path가 기존 baseline에
이미 다수 존재한다. 최신 source의 `92ddf99ee`는 이 한 행만 추가했고 `3c995998e`는 보정 사유를 stage
기록으로 남겼다. 제품 코드나 다른 baseline 행을 넓히지 않는 최소 보정이다.

## 완료한 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| 좌표 계약 | `cargo test --profile release-test --target-dir target/pr-review --test issue_4690_indent_over_stored_column_start -- --nocapture` | 1 passed. p3의 우측 조각이 `x=679.7`, `w=38.4`로 본문 오른쪽 끝에 정확히 닿음을 확인했다. |
| IR 전수 gate | `RHWP_IR_SWEEP_DETAIL=issue4690 cargo test --profile release-test --target-dir target/pr-review --test ir_field_sweep_baseline ir_field_sweep_does_not_regress -- --nocapture` | 823 samples / 245,203 divergences, 217.02초에 통과. 새 재현본은 `hwp5=0`, `hwp5rb=124`의 한 normalized path만 기록한다. |
| 통합 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 5,930 passed / 37 skipped / 7 slow, 486.981초. 좌표 계약과 IR sweep 모두 포함해 통과했다. |
| 최신 source 대조 | `git fetch upstream pull/4735/head:refs/remotes/upstream/pr4735-head` 후 fixture, test, baseline, stage 파일 비교 | 원 source head와 체리픽 파일 내용이 일치했다. |
| 품질 | `git diff --check` | 통과. |

새 fixture는 렌더 출력 자체를 고치는 변경이 아니라 기존 본문 경계 회귀를 고정하는 테스트와 baseline
등록이다. 따라서 별도 PDF visual sweep을 추가하지 않았고, 저장 줄의 실제 좌표와 본문 경계는 회귀
테스트가 직접 검증한다.

## 판정

**통합 수용 권고.** 원격 CI 실패 원인은 신규 fixture onboarding 때 필요한 HWP5 IR baseline 한 행
누락이었고 최신 source에서 최소 범위로 보정됐다. 통합 PR 생성 뒤에는 해당 최신 head의 GitHub required
checks와 mergeability를 다시 확인하고 작업지시자 승인을 받은 뒤에만 merge한다.
