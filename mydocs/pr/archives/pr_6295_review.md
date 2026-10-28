---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28 18:45 KST
pr: 6295
issue: 6284
author: planet6897
---

# PR #6295 review - 본문 그림 경로에서도 그림 캡션을 그린다

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6295
- 작성자: `planet6897`
- reviewer: REST API로 `jangster77` review request 등록 완료
- 원 PR head: `776dc2f7a8a1cedf0299fb9612ce9044d32652b9`
- 통합 검토 브랜치: `review/planet6897-open-ci-20260828`
- cherry-pick 결과: `0903c5703`
- 기준: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- 상태: non-draft, mergeable, 실패·진행 check 0건
- PR comments/review comments: 0건

## 검토 판단

**수용 권고.** 본문 picture 렌더 경로 `layout_picture_full`에서 캡션을 전혀 참조하지 않아 TOP 캡션이
사라지고 그림이 캡션 띠만큼 위로 올라가던 결함을, 기존 caption layout 계약과 같은 산식으로 보완한다.
캡션 위치 산정은 Top/Bottom/Left/Right를 모두 다루며, 본문 경로와 셀 내부 picture 경로에 `styles`를
전달해 caption paragraph 조판이 가능하게 했다.

## 메인터너 보정

통합 브랜치에서 `src/renderer/layout/picture_footnote.rs`의 중복
`#[allow(clippy::too_many_arguments)]` 1줄을 제거했다. 기능 변화는 없고 clippy 확인도 통과했다.

## 증적과 검증

- 대상 fixture: `samples/issue6284/child_policy_top_caption_charts.hwpx`
- `rhwp info --json`: `mydocs/pr/assets/pr_6295_issue6284_info.json`
  - `format=hwpx`, `lastSavedWith=hancom-office-2018 10.0.0.11808`, `pageCount=34`
  - 저장 제품이 2022 이하이므로 MCP `engine 2020`, suffix `-2020.pdf` 기준 적용
- 기준 PDF:
  `pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6295_issue6284_child_policy_top_caption_charts-2020.pdf`
- visual sweep 대표 page: p6
  - `mydocs/pr/assets/pr_6295_issue6284_p6_visual_review.png`
  - `mydocs/pr/assets/pr_6295_issue6284_visual_sweep_summary.json`
  - pixel match `89.56241%`, visual proxy `57.58029%`, flagged page `0`
- focused test:
  - `issue_6284_picture_top_caption_rendered`: 2 pass
- GitHub CI:
  - 초기에 pending이던 #6295의 `Analyze (rust)`, Archive B/D shard가 2026-08-28 18:35 KST 재확인 시 완료됐다.
  - Full CI/CodeQL/Adapter inter-diff/Proptest/Render Diff 모두 실패·진행 check 0건이다.
- 공통 로컬 검증:
  - fmt, suite manifest, unit-tier, CI 범위 clippy/check/WASM check, native-skia lib 통과

## 코멘트 처리

merge 후 원 PR/issue에는 다음을 남긴다.

- 초기 대기 중이던 CI까지 완료된 상태에서 검토했다.
- visual sweep p6에서 자동 flag 0건이고, TOP 캡션 텍스트가 그림 위에 표시되며 그림이 캡션 띠 아래에 놓임을 확인했다.
- focused 회귀는 캡션 텍스트 방출과 그림 상단 y 위치를 함께 고정한다.
- 대표 이미지는 merge SHA 고정 raw URL로 `pr_6295_issue6284_p6_visual_review.png`를 첨부한다.

## 후속

추가 메인터너 보정 필요 없음.
