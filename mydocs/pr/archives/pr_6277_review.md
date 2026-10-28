---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28 18:45 KST
pr: 6277
issue: 6267
author: planet6897
---

# PR #6277 review - 자리차지 표가 호스트 문단 본문과 겹치지 않게 한다

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6277
- 작성자: `planet6897`
- reviewer: REST API로 `jangster77` review request 등록 완료
- 원 PR head: `3ca22dab60acedd2d90c1355e2c35e6425af4e19`
- 통합 검토 브랜치: `review/planet6897-open-ci-20260828`
- cherry-pick 결과: `3d943170b`
- 기준: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- 상태: non-draft, mergeable, 실패·진행 check 0건
- PR comments/review comments: 0건

## 검토 판단

**수용 권고.** TopAndBottom 자리차지 표의 host 문단에 실제 본문 글줄이 있을 때,
`spacing_before` 이중 계상과 body-bottom clamp가 겹쳐 표가 본문 줄 위로 올라오는 문제를 좁은 조건으로
막는다. 빈 host 문단의 하단 고정 틀 예외와 분리되어 있어 기존 #1658/#1858 계열 의도와도 충돌하지 않는다.

## 증적과 검증

- 대상 fixture: `samples/issue6267/kdt_result_para_float_table.hwpx`
- `rhwp info --json`: `mydocs/pr/assets/pr_6277_issue6267_info.json`
  - `format=hwpx`, `lastSavedWith=hancom-office-2018 10.0.0.11529`, `pageCount=1`
  - 저장 제품이 2022 이하이므로 MCP `engine 2020`, suffix `-2020.pdf` 기준 적용
- 기준 PDF: `pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6277_issue6267_kdt_result_para_float_table-2020.pdf`
- visual sweep 대표 page: p1
  - `mydocs/pr/assets/pr_6277_issue6267_p1_visual_review.png`
  - `mydocs/pr/assets/pr_6277_issue6267_visual_sweep_summary.json`
  - pixel match `86.44565%`, visual proxy `15.08894%`, flagged page `0`
- focused test:
  - `issue_6267_para_float_table_overlap`: 2 pass
- 공통 로컬 검증:
  - `cargo fmt --all -- --check` 통과
  - suite manifest prepare/check 통과
  - unit-tier check 통과
  - CI 범위 clippy/check/WASM check 통과
  - native-skia lib 통과

## 코멘트 처리

merge 후 원 PR/issue에는 다음을 남긴다.

- `kdt_result_para_float_table.hwpx`는 `hancom-office-2018` 저장본이라 engine 2020 기준 PDF로 대조했다.
- visual sweep p1에서 자동 flag 0건이고, host 본문과 자리차지 표의 겹침이 보이지 않았다.
- focused 회귀 2건이 `dump-extents` 기준으로 host 문단 줄과 표 상단 간격을 고정했다.
- 대표 이미지는 merge SHA 고정 raw URL로 `pr_6277_issue6267_p1_visual_review.png`를 첨부한다.

## 후속

추가 메인터너 보정 필요 없음.
