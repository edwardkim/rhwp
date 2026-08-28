---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28 18:45 KST
pr: 6292
issue: 6269
author: planet6897
---

# PR #6292 review - 선의 bbox를 잉크 범위로 통일한다

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6292
- 작성자: `planet6897`
- reviewer: REST API로 `jangster77` review request 등록 완료
- 원 PR head: `d58fb78f3b8cb6610547b7a11425c457c3c2dbae`
- 통합 검토 브랜치: `review/planet6897-open-ci-20260828`
- cherry-pick 결과: `9f81bbfb1`
- 기준: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- 상태: non-draft, mergeable, 실패·진행 check 0건
- PR comments/review comments: 0건

## 검토 판단

**수용 권고.** clip을 국소 확장하는 대신 `LineNode`의 bbox 규약을 실제 stroke ink 범위로 맞추는
근인 수정이다. 세로/가로 축 정렬 선은 stroke 반폭 방향만 넓히고, 대각선은 양축을 넓히는 산식이라
렌더 backend의 중심 정렬 stroke 모델과 맞는다.

## 증적과 검증

- 대상 fixture: `samples/issue6269/156739836_public_sector_jobs_stats.hwpx`
- `rhwp info --json`: `mydocs/pr/assets/pr_6292_issue6269_info.json`
  - `format=hwpx`, `lastSavedWith=hancom-office-2022 12.0.0.4204`, `pageCount=57`
  - 저장 제품이 2022이므로 MCP `engine 2020`, suffix `-2020.pdf` 기준 적용
- 기준 PDF: `pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6292_issue6269_public_sector_jobs_stats-2020.pdf`
- visual sweep 대표 page: p2
  - `mydocs/pr/assets/pr_6292_issue6269_p2_visual_review.png`
  - `mydocs/pr/assets/pr_6292_issue6269_visual_sweep_summary.json`
  - pixel match `87.62569%`, visual proxy `14.29271%`, flagged page `0`
- focused test:
  - `issue_6269_body_clip_line_stroke`: 2 pass
- 공통 로컬 검증:
  - fmt, suite manifest, unit-tier, CI 범위 clippy/check/WASM check, native-skia lib 통과

## 코멘트 처리

merge 후 원 PR/issue에는 다음을 남긴다.

- #6278의 clip 확장식 대체가 아니라, 선 bbox 규약을 잉크 범위로 통일하는 수정으로 검토했다.
- visual sweep p2에서 자동 flag 0건이고, 좌단 세로선이 잘려 보이지 않는 결함이 대표 페이지에서 재현되지 않는다.
- focused 회귀는 bbox 규약 자체와 body clip이 선 잉크를 자르지 않는 불변식을 함께 검증한다.
- 대표 이미지는 merge SHA 고정 raw URL로 `pr_6292_issue6269_p2_visual_review.png`를 첨부한다.

## 후속

추가 메인터너 보정 필요 없음.
