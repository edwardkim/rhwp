---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28 18:45 KST
pr: 6294
issue: 6280
author: planet6897
---

# PR #6294 review - 저장 줄 높이가 이미 품은 개체를 다시 세지 않는다

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6294
- 작성자: `planet6897`
- reviewer: REST API로 `jangster77` review request 등록 완료
- 원 PR head: `f2605cda40fc94fb498dedbedf752d72f28f0ea5`
- 통합 검토 브랜치: `review/planet6897-open-ci-20260828`
- cherry-pick 결과:
  - `78239bba5` 제품/회귀 수정
  - `87944f66f` IR sweep baseline 등재
- 기준: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- 상태: non-draft, mergeable, 실패·진행 check 0건
- PR comments/review comments: 0건

## 검토 판단

**수용 권고.** 텍스트 없는 셀의 저장 줄 높이가 이미 TopAndBottom 개체 높이를 품는 경우에도 개체 흐름
높이를 다시 더해 표가 과대 팽창하던 문제를 기존 `non_inline_control_flow_height` 판단 경로로 좁혀
보정한다. PR 본문대로 #6288 계열 구현을 택한 방향이 이웃 증인과 재사용성이 높다.

## 체리픽 충돌 처리

두 번째 commit 적용 중 `tests/fixtures/ir_field_sweep_baseline.tsv`에서 최신 devel의 #6264 계열 row와
#6280 row가 같은 위치에 들어와 충돌했다. 메인터너 보정으로 어느 한쪽을 덮지 않고 두 row를 모두 보존했다.

```text
hwp5rb issue6264/... list_header_width_ref 50
hwp5rb issue6280/... list_header_width_ref 2
```

제품 코드의 의미 변경은 추가하지 않았다.

## 증적과 검증

- 대상 fixture: `samples/issue6280/156742029_prosecutor_transfer_list.hwp`
- `rhwp info --json`: `mydocs/pr/assets/pr_6294_issue6280_info.json`
  - `format=hwp5`, `lastSavedWith=hancom-office-2018 10.0.0.13764`, `pageCount=21`
  - 저장 제품이 2022 이하이므로 MCP `engine 2020`, suffix `-2020.pdf` 기준 적용
- 기준 PDF: `pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6294_issue6280_prosecutor_transfer_list-2020.pdf`
- visual sweep 대표 page: p21
  - `mydocs/pr/assets/pr_6294_issue6280_p21_visual_review.png`
  - `mydocs/pr/assets/pr_6294_issue6280_visual_sweep_summary.json`
  - pixel match `92.24291%`, visual proxy `19.37544%`, flagged page `0`
- focused test:
  - `issue_6280_cell_ladder_line_covers_object`: 1 pass
- 공통 로컬 검증:
  - fmt, suite manifest, unit-tier, CI 범위 clippy/check/WASM check, native-skia lib 통과

## 코멘트 처리

merge 후 원 PR/issue에는 다음을 남긴다.

- #6288/#6285 중복 사안을 최신 devel 기준으로 수용 가능한 구현 쪽으로 통합했고, baseline 충돌은 합집합으로 풀었다.
- visual sweep p21에서 자동 flag 0건이고, `의원면직` 제목을 장식 막대가 관통하지 않음을 확인했다.
- focused 회귀는 아이콘 칸 세로선 높이가 선언 높이보다 과대 팽창하지 않는 불변식을 검증한다.
- 대표 이미지는 merge SHA 고정 raw URL로 `pr_6294_issue6280_p21_visual_review.png`를 첨부한다.

## 후속

추가 메인터너 보정 필요 없음.
