---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5709 검토 - 표 뒤 글줄이 이어지는 문단의 TAC 표 위치

## 접수 메타데이터

| 항목 | 검토 기록 |
| --- | --- |
| PR / 작성자 | [#5709](https://github.com/edwardkim/rhwp/pull/5709) / `planet6897` |
| base / 원 PR head | `devel` / `f259aed7093ed59925dd86c30e92802635711eb7` |
| 변경 규모 | 3 files, +276 / -6 |
| 통합 검토 branch | `review/planet6897-20260820` |
| local cherry-pick | `22b9156af` |
| 통합 기준 | `upstream/devel@cfe2c351e` 위에 #5709 → #5710 → #5718 순서로 적용 |
| 관련 issue | #5589 |

원 PR은 비 draft이며 작성 시점 확인에서 required CI·CodeQL·Render Diff·Native Skia가 통과했다.
mergeable 상태는 외부 GitHub 계산값이므로 merge 직전에 최신 head로 다시 확인해야 한다.

## 변경 범위와 검토 결과

`src/renderer/layout/table_layout.rs`의 중첩 TAC 표 배치가 항상
`para.line_segs.last()`를 표가 놓인 줄로 선택하던 문제를 수정했다. 표 밴드의 높이와 정확히
일치하는 저장 줄이 하나일 때만 그 줄을 선택하고, 여러 줄이 일치하면 기존 마지막 줄 선택을
유지해 #1195의 `[글줄, 표 밴드]` 계약을 보존한다.

검토 결과, 선택 로직은 표 높이와 위·아래 바깥 여백을 함께 비교하고 모호한 경우 보수적으로
기존 동작으로 돌아간다. 기존 계약을 무조건 대체하거나 일반 문단의 줄 위치를 변경하는
메인터너 보정은 필요하지 않았다.

## 체리픽 및 충돌

- 최신 `upstream/devel@cfe2c351e`에서 가시성 branch를 만들었다.
- #5709 source head `f259aed7`를 `22b9156af`로 누적 적용했다.
- 충돌은 없었고, 이후 #5710·#5718을 같은 branch에 순서대로 적용했다.

## 검증

- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check`: 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check upstream/devel...HEAD`: 통과
- 집중 테스트 `issue_5589_table_line_selection`: 2/2 통과
- 통합 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: **8,001 통과, 38 skip**
- 전체 실행에서 #5589의 두 회귀 테스트도 다시 통과했다.
- renderer 변경에 대한 원 PR의 Canvas Render Diff 결과는 통과했다. 이번 통합 검증에서는
  별도 PDF asset을 새로 만들지 않고 원 PR의 시각 검증과 통합 Rust 회귀 결과를 구분해 사용했다.

## 판정

차단 결함과 추가 메인터너 보정 필요 사항은 발견하지 못했다. #5589 범위는 통합 branch에서
수용 권고다. 원격 push·승인·merge는 수행하지 않았으며, merge 전 최신 head와 required check를
다시 확인해야 한다.
