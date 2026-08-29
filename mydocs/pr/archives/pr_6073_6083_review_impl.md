---
kind: pr-review-implementation
status: local-validated
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28 21:27 KST
prs: [6073, 6083]
author: kevin9327
---

# PR #6073/#6083 통합 검토 구현 기록

## 적용

- branch: `review/kevin9327-6073-6083-20260828`
- base: `upstream/devel@a6c7e7bb3ae09470c225a4c90c0fc1ad88b6b5a6`
- #6073 원 head: `b90d80484249da3fbedb06355b92fc149d193e95`
- #6083 원 head: `cfb2646ee19ffb794fbb389779d56114c6f97807`
- #6083 conflict:
  - `tests/fixtures/overflow_cell_baseline.tsv`
  - 원 PR의 `2025 행정업무운영 편람(최종).hwp 35`, `.hwpx 52` 증가는 반영하지 않고,
    최신 devel의 `.hwp 32`, `.hwpx 51`을 보존했다.

## 메인터너 보정

커밋: `04e3d8ac1 fix(renderer): #6083 셀 재래핑 높이 측정을 보정한다`

- #6083의 fresh 재래핑 결과를 렌더 경로에만 반영하면 상자 하단과 다음 본문이 겹친다.
- `recompose_horizontal_cell_lines_for_width`를 공통 helper로 두고 `height_measurer`,
  `table_layout`, `table_partial`에서 같은 방식으로 사용하게 했다.
- original HWPX 저장 layout에서는 overflow-cell 원장 증가를 만들지 않도록 `native_hwp5_layout()`에서만
  저장 다줄 overflow 보정을 켰다.
- `issue_5952_cell_note_overflow`에 하단 겹침 방지 테스트를 추가했다.

## 로컬 검증

- `cargo fmt --all -- --check`: pass
- `git diff --check`: pass
- `node scripts/rust-test-suite-manifest.mjs --prepare`: pass
- `node scripts/rust-test-suite-manifest.mjs --check`: pass
- `node scripts/rust-unit-test-tiers.mjs --check`: pass
- `cargo test --locked --target-dir target/pr-review --test regression_suite_027 issue_5952 -- --nocapture`: 4 pass
- `cargo test --locked --target-dir target/pr-review --test regression_suite_013 issue_6063_hwpx -- --nocapture`: 2 pass
- `cargo test --locked --target-dir target/pr-review --test regression_suite_011 issue_3931 -- --nocapture`: 5 pass
- `RHWP_OVERFLOW_CELL_DUMP=output/pr6083-maintainer-check/overflow_cell_current.tsv cargo test --locked --target-dir target/pr-review --test overflow_cell_baseline -- --nocapture`:
  1 pass, 945 samples, skip 3, 13 non-zero docs, 352 total lines
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
  8551 pass / 43 skipped
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`: pass
- `cargo test --locked --doc --target-dir target/pr-review`: 8 pass / 3 ignored
- `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib`:
  3946 pass / 13 ignored
- `node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir target/pr-review --features native-skia`:
  2 pass
- `node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir target/pr-review --features native-skia`:
  4 pass
- `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`: pass

## 시각 증적

- #6073:
  - `mydocs/pr/assets/pr_6073_issue6063_before.png`
  - `mydocs/pr/assets/pr_6073_issue6063_after.png`
  - `mydocs/pr/assets/pr_6073_issue6063_before_tail.png`
  - `mydocs/pr/assets/pr_6073_issue6063_after_tail.png`
  - `mydocs/pr/assets/pr_6073_issue6063_info.json`
- #6083:
  - `mydocs/pr/assets/pr_6083_issue5952_handbook_info.json`
  - `mydocs/pr/assets/pr_6083_issue5952_p69_review.png`
  - `mydocs/pr/assets/pr_6083_issue5952_p69_compare.png`
  - `mydocs/pr/assets/pr_6083_issue5952_p69_overlay.png`
  - `mydocs/pr/assets/pr_6083_issue5952_visual_sweep_summary.json`
  - `mydocs/pr/assets/pr_6083_issue5952_p69_overlay_metrics.json`
  - `mydocs/pr/assets/pr_6083_issue5952_p69_analysis_metrics.json`
  - `mydocs/pr/assets/pr_6083_issue5952_overflow_cell_current.tsv`

## 판정

- #6073: 수용 권고. 원 PR 중간 상태의 오라클 완화와 상수형 승격 가드는 메인터너 보정으로 제거됨.
- #6083: 메인터너 보정 포함 수용 후보. 우측/하단 겹침과 overflow-cell 원장 증가는 보정으로 닫혔지만,
  p69 visual sweep은 자동 flag가 남으므로 merge comment와 후속 issue 후보에서 숨기지 않는다.
