---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6088 review - #6031 sb 누락 사다리 판정 좌표

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6088
- 작성자: `planet6897`
- 원 PR head: `7b64e9156ffa`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9` (#6142 merge 포함)
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. 저장 줄 세그먼트의 sb 누락 사다리에서 판정 좌표와 실제 배치 좌표가 어긋나는 문제를
좁은 renderer/typeset 경계에서 보정한다. #6142가 이미 devel에 병합되어 있으므로 #6142 자체의 패치는
중복 적용하지 않았고, #6088 패치만 최신 기준 위에 남겼다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/ladder-sb-tail-6031/before_p3.png`,
  `mydocs/report/ladder-sb-tail-6031/after_p3.png`,
  `mydocs/report/ladder-sb-tail-6031/before_p6.png`,
  `mydocs/report/ladder-sb-tail-6031/after_p6.png`
- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6031_3249937_asset_management_rules_info.json`
  - MCP Hancom 2020 PDF: `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue6031_asset_management_rules_3249937_asset_management_rules-2020.pdf` (`sha256=19226f8e3e564bcc8e1c4cad6a1a8259f5e1bf7821d8a81b371aa84f1e5c0c3f`)
  - 대표 review PNG: `mydocs/pr/assets/pr_6088_issue6031_mcp2020_visual_review_003.png`, `mydocs/pr/assets/pr_6088_issue6031_mcp2020_visual_review_006.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_mcp2020_visual_sweep_metrics.tsv` (`pages=3,6`, `flagged_page_count=0`, worst pixel match `90.59778%`)
  - local Hancom 2020 action-open retry도 보조 증적으로 완료: `pdf/pr_6088_6144/local_hancom2020/pr_6088_issue6031_asset_management_rules_action-local2020.pdf`
- 통합 후보 전체 검증:
  - `cargo fmt --all -- --check` 통과
  - `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings` 통과
  - `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast`
    결과 8,399 pass, 43 skip
  - WASM `scripts/wasm-pack-locked.sh --target web --out-dir pkg` 통과
  - native-Skia 공식 범위 통과

## 후속

통합 PR CI가 녹색이면 원 PR에는 #6031 회귀가 통합 후보에서 검증됐다는 코멘트 후 close한다.
