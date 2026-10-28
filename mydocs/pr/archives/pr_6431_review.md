---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6431
author: kevin9327
---

# PR #6431 review - 쪽 경계 cell float 그림의 중복 출력 방지

## Metadata

- 원 PR: [#6431](https://github.com/edwardkim/rhwp/pull/6431), source head
  `8f231d4079635c96ebdcc641dc74a747534fa56e`.
- 작성자 `kevin9327`, reviewer `jangster77` 요청 완료. CI green non-draft head를 latest
  `upstream/devel` 위에 conflict 없이 적용했다.

## 변경과 검토

- 쪽 경계를 걸치는 table cell float 그림의 placement range를 한 쪽으로 고정해, 앞/뒤 fragment가
  모두 같은 그림을 출력하지 않도록 한다.
- contributor의 후속 `u32` page-number test type correction도 함께 적용되어 test가 컴파일된다.

## 시각 증적

- fixture `samples/issue5734/cell_float_stack_stored_vpos.hwpx`의 Hancom 2020 PDF
  `pdf/cell_float_stack_stored_vpos-2020.pdf`를 기준으로 사용했다. SHA-256은
  `04265b5080350c29246efb2c8107f30fc3af599f4fab37cc73077182a30885cc`, A4 2쪽이다.
- Visual Sweep은 1-2쪽을 모두 완료했고 flagged page는 없다. pixel match 평균 98.17280%,
  worst 96.99864%; visual-accuracy proxy 평균 1.67918%, worst 0%다. proxy는 glyph/render
  raster 차이에 민감하므로 그림 중복 판단의 단독 기준이 아니다.
- 사람 확인으로 쪽 경계의 float 그림이 양쪽 페이지에 중복 등장하지 않고, 순서와 cell 경계가
  보존됨을 확인했다.
- 보존 asset: `mydocs/pr/assets/pr_6431_cell_float_p1_review.png`,
  `pr_6431_cell_float_p2_review.png`, `pr_6431_cell_float_visual_sweep_summary.json`.

## 댓글 계획과 권고

- merge 뒤 source PR에는 비교한 1-2쪽, 무flag, 지표 한계와 사람 확인을 기록한다. asset이
  devel에 있는지 재확인한 뒤 다음 raw image 형식으로 게시한다.

  ```markdown
  ![PR #6431 page 1 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6431_cell_float_p1_review.png)
  ```

**수용.** 통합 branch full nextest `8772 passed, 43 skipped` (430.908초, exit 0)와
1-2쪽 시각 증적을 근거로 단일 placement 범위 보정으로 수용한다.
