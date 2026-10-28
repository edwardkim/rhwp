---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6357
---

# PR #6357 review - 표 여백 폴백 축을 수직으로 한정하고 조판 계측을 추가한다

## 검토 판단

**수용 권고.** 넓은 변경 범위는 통합 head에서 전체 회귀와 계측 스크립트 테스트로 재검증했다.
다만 source baseline은 오래된 기준에서 작성되어 #6284 값을 포함해 통합 결과와 달랐으므로, 원인별
증가만 반영한 maintainer baseline 보정 `6a2e2ff7d`를 추가했다.

## 라우팅과 검증

- 원 PR: https://github.com/edwardkim/rhwp/pull/6357
- 작성자 / reviewer: `jeong-sik` / `jangster77` review request 등록
- source head: `d5ed9636b7b7d78935269a8f93149acfeb7c32d7`
- source branch 충돌은 최신 `devel` 기준으로 해소했고, generated suite/manifest는 생성만 확인하고
  source에 포함하지 않았다.
- `layout_anomaly_glyph_band`: 6/6, layout coverage와 cell-lineseg agreement Node tests: 26/26 통과.
- `text_overlap_baseline`: 보정 후 통과. #6284는 source 이전 head `54`에서 #6357 반영 후 `56`으로
  증가함을 분리 worktree로 확인했다.

## 시각 증적과 코멘트

- 대표 fixture: `samples/issue6284/child_policy_top_caption_charts.hwpx`.
  `hancom-office-2018` 저장본이라 2020 기준 PDF를 사용했다.
- p10-31 visual sweep: 22/22 완료, 자동 flag 0, 평균 pixel match `87.07317%`.
  폰트 모양 차이는 남지만 표/본문의 쪽 흐름과 표 하단 경계 이탈은 확인되지 않았다.
- 보관 asset: `mydocs/pr/assets/pr_6357_issue6284_{info,visual_sweep_summary}.json`,
  `mydocs/pr/assets/pr_6357_issue6284_p12_review.png`.
- merge 후 원 PR에는 baseline 보정의 범위와 p10-31 flag 0, 대표 이미지를 명시한다.
