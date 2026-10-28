---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28 18:45 KST
pr: 6286
issue: 6264
author: planet6897
---

# PR #6286 review - 앵커 아래에 담기지 않는 중첩 표는 저장 vpos를 쓰지 않는다

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6286
- 작성자: `planet6897`
- reviewer: REST API로 `jangster77` review request 등록 완료
- 원 PR head: `b89c8badb92794aedce2bfff5629e25f197505a9`
- 통합 검토 브랜치: `review/planet6897-open-ci-20260828`
- cherry-pick 결과:
  - `80e279ca5` 제품/회귀 수정
  - `d7795642d` IR sweep baseline 등재
- 기준: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- 상태: non-draft, mergeable, 실패·진행 check 0건
- PR comments/review comments: 0건

## 검토 판단

**수용 권고.** 셀 문단의 저장 `vertical_pos`가 문단 줄 높이만 설명하고 중첩 표 높이를 설명하지 못하는
형상에서, 저장 anchor를 그대로 쓰면 중첩 표가 셀 하단에서 14px 띠로 붕괴한다. 이 PR은 앵커 아래에
실제 중첩 표가 담기는지 다시 확인한 뒤, 담기지 않는 경우 자연 흐름 배치를 쓰도록 제한한다. 기존 저장
vpos 신뢰 경로를 전면 폐기하지 않고 해당 형상만 좁혀 처리한 점이 타당하다.

## 증적과 검증

- 대상 fixture: `samples/issue6264/1977964_env_satellite_report_form.hwp`
- `rhwp info --json`: `mydocs/pr/assets/pr_6286_issue6264_info.json`
  - `format=hwp5`, `lastSavedWith=hancom-office-2010 8.5.8.1630`, `pageCount=4`
  - 저장 제품이 2022 이하이므로 MCP `engine 2020`, suffix `-2020.pdf` 기준 적용
- 기준 PDF: `pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6286_issue6264_env_satellite_report_form-2020.pdf`
- visual sweep 대표 page: p1
  - `mydocs/pr/assets/pr_6286_issue6264_p1_visual_review.png`
  - `mydocs/pr/assets/pr_6286_issue6264_visual_sweep_summary.json`
  - pixel match `96.37407%`, visual proxy `13.32636%`, flagged page `0`
- focused test:
  - `issue_6264_cell_stored_vpos_nested_table`: 1 pass
- 공통 로컬 검증:
  - fmt, suite manifest, unit-tier, CI 범위 clippy/check/WASM check, native-skia lib 통과

## 코멘트 처리

merge 후 원 PR/issue에는 다음을 남긴다.

- 2010 저장본이라 engine 2020 기준 PDF로 대조했다.
- visual sweep p1에서 자동 flag 0건이고, 중첩 표가 하단 얇은 띠로 붕괴하지 않음을 확인했다.
- focused 회귀는 중첩 표 높이가 선언 높이를 유지하고 바깥 표 안에 담기는 불변식을 검증한다.
- 대표 이미지는 merge SHA 고정 raw URL로 `pr_6286_issue6264_p1_visual_review.png`를 첨부한다.

## 후속

추가 메인터너 보정 필요 없음.
