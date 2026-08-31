---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6435
author: kevin9327
---

# PR #6435 review - 미주 마지막 단 꼬리 frame overflow 보정

## 검토 대상

- 원 PR head: `f7706305c7aa4ae17153f2d2e590b07ac6b0fdd6`, 통합 적용 최종 commit `06f315e9d`.
- 기준 base: `upstream/devel@3afbb066fe93724ab44309163a2e04efb954bf18`.
- 2026-08-31 재조회에서 Open/non-draft, `postmelee` requested reviewer 없음이다. source head의 Build & Test,
  Lint, Native Skia, Archive A-D와 adapter/proptest는 성공했다.
- 마지막 column의 overflow 줄만 다음 쪽으로 넘기고 `ENDNOTE_PAGE_OFFCANVAS_GUARD_PX=56`은 보존한다.
  regression은 Hancom 23쪽 수를 함께 잠근다.

## 전후 시각 증적

- 입력: `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`,
  `lastSavedWith.product=hancom-office-2024`.
- 기준 PDF: `pdf/3-09월_교육_통합_2024-구분선아래20구분선위20-hwp-2024.pdf`, A4 23쪽.
- 2024 기준 PDF와 같은 `rsvg` sweep으로 p17을 base와 통합 head에서 비교했다.
  base는 frame 밖 868px, extent 14px, `render_tree_frame_tail_overflow` 1건(문29 마지막 줄)이었다.
  통합 head는 frame 밖 0px, extent 없음, 해당 tail-overflow 후보 0건이고 body bottom delta는 -10px다.
- 전후 pixel/proxy는 base 88.91183%/8.89833%, 통합 89.02757%/8.92021%다. 통합 p17에는
  question marker·line band·large ink region flag가 남지만 base에도 같은 범주의 flag가 있어 이 PR의
  frame-tail 보정과 별개인 기존 fidelity 차이로 기록한다.
- 직접 검토한 asset은
  `mydocs/pr/assets/pr_6435_issue4318_p17_before_review.png`와
  `mydocs/pr/assets/pr_6435_issue4318_p17_review.png`다. 후자는 목표인 오른쪽 단 마지막 줄이 본문
  frame 밖으로 나가지 않음을 보여 준다. `rsvg` proxy는 font raster 차이에 민감한 보조값이다.

## 통합 검증과 판단

- fmt, native/WASM clippy, workspace build, all-target clippy, manifest, Rust unit tier check가 통과했고,
  release-test 전체 nextest는 `8870 passed, 46 skipped` (450.949초, exit 0)였다.

**수용 권고.** base 대비 14px frame overflow와 render-tree tail 후보가 모두 사라졌고, 23쪽 불변식과
전체 회귀를 함께 확인했다. marker/line 차이는 이 수용의 범위가 아니며 별도 fidelity 개선 대상으로 남긴다.

## Merge 후 contributor PR comment 계획

- p17 전후의 14px/0px, tail 후보 1/0, 자동 수치의 한계와 남은 별도 flag를 사실대로 적는다.
- asset이 merge commit에 존재하는 것을 API 재조회한 뒤 다음 형식으로 게시한다.

  ```markdown
  ![PR #6435 p17 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6435_issue4318_p17_review.png)
  ```
