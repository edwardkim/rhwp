---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6430
author: kevin9327
---

# PR #6430 review - CellBreak 조각 셀의 다음 행 겹침 방지

## Metadata

- 원 PR: [#6430](https://github.com/edwardkim/rhwp/pull/6430), source head
  `6c9d0e725695b205daa9f82c6dd53efb93f47f11`.
- 작성자: `kevin9327`; reviewer `jangster77` 요청 완료. CI green, non-draft source를
  latest `upstream/devel`에 conflict 없이 적용했다.

## 변경과 검토

- CellBreak table cell 조각의 clip start/end를 계산해 다음 행 위로 content가 넘쳐 겹치는
  renderer 결함을 막는다.
- contributor가 Archive C 실패 뒤 실문서의 항목란을 인접 위치에서 다시 찾도록 test를
  보정한 이력도 확인했다.

## 시각 증적

- source fixture `samples/2025 행정업무운영 편람(최종).hwpx`의 Hancom Office 2024 저장 정보를
  따라 기존 PDF `pdf/2025 행정업무운영 편람(최종)-hwpx-2024.pdf`를 재사용했다. SHA-256은
  `e77634f4c63c15875797a4d6d354fd90cad27fd65d708cee4cd85b80c4cb1009`, 총 384쪽이다.
- 변경 설명의 cell/row 경계가 있는 149, 150, 153쪽을 Visual Sweep으로 완료했고 flagged page는
  없었다. pixel match 평균 87.14470%, worst 86.11382%, visual-accuracy proxy 평균
  42.23457%, worst 28.82050%다. 이 지표는 raster/text-font 차이를 포함한 proxy다.
- 사람 확인은 세 페이지에서 표 행 순서, cell clip, 다음 행 text band의 겹침 부재를 확인한
  범위이며, 문서 전체의 pixel-perfect fidelity 판정은 아니다.
- 보존 asset: `mydocs/pr/assets/pr_6430_pyeollam_p149_review.png`,
  `pr_6430_pyeollam_p150_review.png`, `pr_6430_pyeollam_p153_review.png`,
  `pr_6430_pyeollam_visual_sweep_summary.json`.

## 댓글 계획과 권고

- merge 뒤 실제 source PR comment에는 Visual Sweep guide, 149/150/153쪽, 무flag와 지표/한계,
  사람 확인 결론을 기록한다. devel asset 검증 뒤 `--body-file`로 다음 raw image를 포함한다.

  ```markdown
  ![PR #6430 page 149 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6430_pyeollam_p149_review.png)
  ```

**수용.** 통합 branch full nextest `8772 passed, 43 skipped` (430.908초, exit 0)와
149/150/153쪽 시각 증적을 근거로 next-row overlap을 차단하는 clip 경계 보정으로 수용한다.
