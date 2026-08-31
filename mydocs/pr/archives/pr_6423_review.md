---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6423
author: kevin9327
---

# PR #6423 review - 셀 자동 축소 목표 폭 수렴

## 검토 대상과 보정

- 원 PR head: `3e426e0d5185db15c661b9976a7151fa7a626b89`, 통합 적용 최종 commit `dd61af159`.
- 기준 base: `upstream/devel@3afbb066fe93724ab44309163a2e04efb954bf18`.
- source PR은 base 변경으로 dirty였고 최신 head에는 `cancel-stale-runs`만 붙어 있었다. 통합 branch의
  전체 로컬 검증으로 대체 확인했다. 2026-08-31 재조회에서도 Open/non-draft이며 `postmelee` requested
  reviewer는 없다.
- 충돌 합성 뒤 `compute_line_extra_spacing`의 새 `converge_auto_shrink_cell` 인자가 기존 unit test 세 곳에
  누락된 것을 all-target clippy에서 발견했다. 메인터너 보정 `da5e65ae8`이 세 호출에 `false`를 명시해
  기존 테스트 의미를 보존했다.

## 시각 증적

- 입력: `samples/issue6196/cell_char_spacing_fit.hwp`, `rhwp info --json` 결과
  `lastSavedWith.product=hancom-office-2020`.
- 기준 PDF: `pdf/cell_char_spacing_fit-2020.pdf`, MCP 비동기 job
  `23ec9fd8-b2eb-42db-a2ce-013792463ba8`, 2020 profile, SHA-256
  `ca1be0d313d1a465ce33b051c5925b5883c0dfe6d570f4f611560bfaacae712c`.
- 통합 head의 `target/pr-review/debug/rhwp`와 `rsvg` rasterizer로 p1을 sweep했다. flagged=0,
  pixel match 91.91521%, visual-accuracy proxy 28.80661%다.
- 직접 확인한 `review_001.png`에서 마지막 열의 긴 행은 표 우측 괘선 안에 남는다. 대표 asset은
  `mydocs/pr/assets/pr_6423_issue6303_p1_review.png`이고, 임시 output은
  `output/visual_sweep_kevin9327_20260831/pr6423_issue6303/pr6423-issue6303/review/review_001.png`다.
- `rsvg`는 Studio webfont 경로가 아니므로 proxy는 glyph raster 차이에 민감한 보조값이며 사람의 판정을
  대체하지 않는다. 이 검토에서는 0.25px 경계 회귀 test와 실제 괘선 안쪽 배치를 함께 판단 근거로 썼다.

## 통합 검증과 판단

- 통합 후보에서 fmt, native/WASM clippy, workspace build, all-target clippy, manifest, Rust unit tier
  check가 통과했고, 전체 release-test nextest는 `8870 passed, 46 skipped` (450.949초, exit 0)였다.

**수용(메인터너 보정 포함).** 수렴 범위는 셀 자동 축소 경로에 한정돼 있으며, 충돌 보정 후 전체 회귀와
실물 HWP p1 검증이 통과했다.

## Merge 후 contributor PR comment 계획

- [PDF/SVG visual sweep 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 연결하고 p1, flagged=0, pixel/proxy 수치와
  rasterizer 한계를 명시한다.
- merge commit에 asset 존재를 API로 재조회한 뒤 `--body-file`로 다음 형식의 image를 게시한다.

  ```markdown
  ![PR #6423 p1 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6423_issue6303_p1_review.png)
  ```
