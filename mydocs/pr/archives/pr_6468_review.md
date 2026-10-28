---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6468
author: kevin9327
---

# PR #6468 review - 쪽 높이급 TAC 표의 leftover 배치 제한

## Metadata

- 원 PR: [#6468](https://github.com/edwardkim/rhwp/pull/6468), source head
  `b5302a33f07fa4e42a267e895bbe4507a278d1ea`.
- 작성자 `kevin9327`, reviewer `jangster77` 요청 완료. CI green non-draft source를 latest
  `upstream/devel` 위에 적용했다.

## 변경과 검토

- 쪽 높이급 `treatAsChar` table을 남은 space에 억지로 넣지 않고 다음 쪽의 정상 flow에 둔다.
- #6409의 큰 표가 partial/overflow로 깨지지 않도록 leftover eligibility를 제한한다.

## 시각 증적

- fixture `samples/issue6031/3249937_asset_management_rules.hwpx`의 Hancom 2020 PDF
  `pdf/3249937_asset_management_rules-2020.pdf`를 기준으로 사용했다. SHA-256은
  `264d41b16b53e7d1f9f43db256d66c78aeeee96a8b1214ec85605666480d85e2`, A4 60쪽이다.
- 변경 위치인 41-42쪽을 Visual Sweep으로 완료했고 flagged page는 없다. pixel match 평균
  91.10342%, worst 87.05844%; visual-accuracy proxy 평균 17.99966%, worst 14.21020%다.
  proxy는 글꼴/raster 차이를 포함하므로 자동 fidelity 판정으로 확대하지 않는다.
- 사람 확인에서 p41의 전행 종료와 p42의 큰 TAC 표 시작이 page boundary를 넘겨 겹치지 않았고,
  표 내부 row order가 보존됐다.
- 보존 asset: `mydocs/pr/assets/pr_6468_asset_management_p41_review.png`,
  `pr_6468_asset_management_p42_review.png`, `pr_6468_asset_management_visual_sweep_summary.json`.

## 댓글 계획과 권고

- merge 뒤 Visual Sweep guide 링크와 41-42쪽 비교 범위, 무flag, 지표의 한계 및 사람 결론을
  source PR에 남긴다. devel asset 확인 뒤 다음 형식으로 image를 넣는다.

  ```markdown
  ![PR #6468 page 42 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6468_asset_management_p42_review.png)
  ```

**수용.** 통합 branch full nextest `8772 passed, 43 skipped` (430.908초, exit 0)와
41-42쪽 시각 증적을 근거로 page-height TAC의 leftover 과밀 배치를 피하는 보정으로 수용한다.
