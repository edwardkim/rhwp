---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6472
author: kevin9327
---

# PR #6472 review - 누락된 HWPX RowBreak/TAC host band 보정

## Metadata

- 원 PR: [#6472](https://github.com/edwardkim/rhwp/pull/6472), source head
  `68a5ad954936b985dcd8fd6087906f35eaf42035`.
- 작성자 `kevin9327`, reviewer `jangster77` 요청 완료. CI green non-draft source를 latest
  `upstream/devel` 위 통합 branch에 적용했다.

## 원본 판단과 메인터너 보정

- 원 PR의 선언-height leftover 특례는 `tac_cell_leftover_fits.hwpx`를 2쪽으로 고정했지만,
  실제 Hancom 2020 PDF는 HEAD / 3행 표 / `AFTER TABLE`을 각각 1/2/3쪽에 둔다.
- fixture의 표 선언 높이는 약 906.7px인데 빈 host의 유일한 non-synthetic `LINE_SEG`는
  약 13.3px다. 이를 표 band로 신뢰하는 것이 페이지 축소의 원인이다.
- 따라서 원본의 넓은 declared-fit 특례는 일반 경로로 되돌렸다. 대신 HWPX stored layout,
  RowBreak, `treatAsChar`+`flowWithText`, control 하나인 빈 host, `3행 x 1열`·3셀,
  `repeatHeader=0`, 비합성 line segment 하나, 그리고 그 segment가 measured table보다 짧다는
  조건이 모두 성립할 때만 physical table band와 trailing spacing을 사용한다. 뒤 일반 문단도 한 번
  엄격하게 fit해 TAIL을 3쪽으로 이월한다.
- 이 PR은 **원본 source 단독으로는 수용하지 않고**, 위 좁은 메인터너 보정과 회귀 test를 포함한
  통합 branch 상태로만 수용한다.

## 시각 증적

- 기준 PDF: `pdf/tac_cell_leftover_fits-2020.pdf`, SHA-256
  `09f0373671ac8dd4dab35ea6dde2190c71b6fb2d0e25e0dc26235a4122df96e6`, A4 3쪽.
- Visual Sweep은 1-3쪽을 모두 완료했고 flagged page는 없다. pixel match 평균 99.59387%,
  worst 98.96160%; visual-accuracy proxy 평균 6.98244%, worst 1.67782%다. proxy는 glyph
  raster 차이에 민감한 보조 지표다.
- 사람 확인에서 p1 HEAD-only, p2 3행 table-only, p3 `AFTER TABLE` 흐름이 Hancom 기준과
  일치했고, 표/TAIL의 같은 쪽 과밀이나 partial table이 없었다.
- 보존 asset: `mydocs/pr/assets/pr_6472_tac_cell_leftover_p1_review.png`,
  `pr_6472_tac_cell_leftover_p2_review.png`, `pr_6472_tac_cell_leftover_p3_review.png`,
  `pr_6472_tac_cell_leftover_visual_sweep_summary.json`.

## 댓글 계획과 권고

- merge 뒤 source PR에는 메인터너 보정의 조건 범위, Hancom 기준 3쪽, 1-3쪽 sweep 무flag,
  자동 지표의 한계와 사람 결론을 명시한다. asset 존재를 devel API로 확인하고 `--body-file`로
  게시하며 raw image는 다음 형식을 사용한다.

  ```markdown
  ![PR #6472 page 2 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6472_tac_cell_leftover_p2_review.png)
  ```

**수용(메인터너 보정 포함).** 좁은 predicate와 3쪽 regression, 통합 branch full nextest
`8772 passed, 43 skipped` (430.908초, exit 0), 그리고 1-3쪽 시각 증적을 근거로 수용한다.
