---
kind: pr-review
status: accepted-with-ci-condition
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6482
author: planet6897
---

# PR #6482 review - run 경계 장식선 폭을 레이아웃 폭에 맞춤

## 검토 기준

- 원 PR head: `33c07e570910a3a906a957677e4926c5a614f629`
- 통합 적용 commit: `5afa8eee42e11b3b3b4753fd4f0077c756e7b9c1`
- 통합 기준 base: `upstream/devel@77bcaaa49c89dc12761282c759717188a880064c`
- 작성 시점 원 PR은 Open/non-draft, `MERGEABLE/CLEAN`이며 Build & Test와 CodeQL이 성공했다. merge 직전에
  최신 head, aggregate check와 mergeability를 다시 확인한다.

## 검토와 검증

- `SvgRenderer`가 말미 공백 trim이 없는 밑줄·취소선에 render-tree `bbox.width`를 사용하도록 한 변경을
  확인했다. 말미 공백 trim 경로는 기존 글자 위치 계산을 유지한다.
- `cargo nextest run --locked --workspace --tests --cargo-profile release-test --target-dir target/pr-review
  --no-fail-fast -E 'test(issue_6310_cell_zoom_image_fill) | test(issue_6451_underline_run_fragments) |
  test(issue_6060_cjk_quote_paint_measure_parity) | test(issue_5804_dash_leader_glyphs) |
  test(issue_5830_dash_leader_last_line) | test(issue_1891) | test(issue_2020)'`를 실행해 22 passed를
  확인했다. 이 중 `underline_fragments_meet_exactly`는 1쪽 y=583.75의 세 조각이 0.01px 이내로
  정확히 맞닿는지 확인한다.
- 2022 저장 HWPX를 2020 엔진으로 직접 PDF 변환했다. `pdf/underline_run_fragments-2020.pdf`
  (6쪽, SHA-256 `f623bd1442e240e41b0e7897b37b9d818e2fcfbf5ba9b6d1fb2e18128806324f`)를 기준으로
  1쪽을 144 DPI `webfont` visual sweep으로 비교했다.
- 결과: flagged 후보 0건, pixel match 87.41426%, visual accuracy proxy 21.61606%.
  문서 전체의 font/position 차이는 overlay 수치에 포함되지만, 사람이 대표 PNG를 직접 확인해 밑줄 대상
  문단에 결손·겹침·깨진 glyph가 없음을 확인했다. 자동 line-band drift 후보도 없다.
- 안정 증적: `mydocs/pr/assets/pr_6482_issue6451_p001_review.png`,
  `mydocs/pr/assets/pr_6482_issue6451_visual_sweep_summary.json`,
  `mydocs/pr/assets/pr_6482_issue6451_overlay_metrics.json`.

## Merge 후 contributor PR comment 계획

- [Visual Sweep GitHub merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을
  링크한다.
- 1쪽, flagged 0건, pixel match 87.41426%, proxy 21.61606%와 수치가 문서 전체 font/position 차이를
  포함한다는 한계, 그리고 사람이 확인한 장식선 연속성을 함께 적는다.
- devel에 asset이 반영된 뒤
  `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6482_issue6451_p001_review.png`
  를 Markdown image로 표시한다. `--body-file` 게시 뒤 API 재조회로 실제 LF와 이미지 URL을 확인한다.

## 최종 판정

**승인.** 변경은 장식선 폭의 단일 경계로 제한되고, 3조각 exact-contiguity 계약, focused regression,
직접 PDF visual sweep이 이를 함께 뒷받침한다. 통합 PR #6490의 code head
`2024da02cde595a179655b8e697d6f0ab8f8b509`에서 Build & Test, Lint, Native Skia, CodeQL을 포함한
34 check-run이 실패 없이 끝났고 mergeability가 `clean`임을 확인했다. 이 trailing 문서 head의 CI만
마저 확인하면 merge 조건이 충족된다.
