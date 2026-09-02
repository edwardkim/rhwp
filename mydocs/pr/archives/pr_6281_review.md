---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28 18:45 KST
pr: 6281
issue: 6266
author: planet6897
---

# PR #6281 review - 양식 개체가 원본 배치대로 놓이게 한다

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6281
- 작성자: `planet6897`
- reviewer: REST API로 `jangster77` review request 등록 완료
- 원 PR head: `2db07c825c5735e10de244dd584a01467cbdc5b3`
- 통합 검토 브랜치: `review/planet6897-open-ci-20260828`
- cherry-pick 결과: `3c827cd07`
- 기준: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- 상태: non-draft, mergeable, 실패·진행 check 0건
- PR comments/review comments: 0건

## 검토 판단

**수용 권고.** `FormObject`가 `CommonObjAttr`를 보존하지 않아 HWP3/HWP5/HWPX 양식 개체의 기준·정렬·
오프셋·바깥여백이 렌더러에 전달되지 않던 결함을 모델/파서/직렬화/레이어 배치 경로까지 일관되게 보완한다.
HWP3의 PushButton 배치가 제목 줄에 인라인으로 섞이는 문제를 직접 고정하는 회귀 테스트도 포함되어 있다.

## 증적과 검증

- 대상 fixture: `samples/issue6266/seizure_list_form_button.hwp`
- `rhwp info --json`: `mydocs/pr/assets/pr_6281_issue6266_info.json`
  - `format=hwp3`, `lastSavedWith=null`, `pageCount=1`
  - `lastSavedWith=null`이므로 MCP `engine 2020`, suffix `-2020.pdf` 기준 적용
- 기준 PDF: `pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6281_issue6266_seizure_list_form_button-2020.pdf`
- visual sweep 대표 page: p1
  - `mydocs/pr/assets/pr_6281_issue6266_p1_visual_review.png`
  - `mydocs/pr/assets/pr_6281_issue6266_visual_sweep_summary.json`
  - pixel match `97.00503%`, visual proxy `10.33174%`, flagged page `0`
- focused test:
  - `issue_6266_form_object_placement`: 2 pass
- 공통 로컬 검증:
  - fmt, suite manifest, unit-tier, CI 범위 clippy/check/WASM check, native-skia lib 통과

## 코멘트 처리

merge 후 원 PR/issue에는 다음을 남긴다.

- HWP3 fixture는 저장 제품 식별자가 없어 engine 2020 기준 PDF로 대조했다.
- visual sweep p1에서 자동 flag 0건이고, PushButton이 제목 줄에 끼어들지 않고 쪽 하단 가운데에 놓임을
  확인했다.
- focused 회귀는 제목 x 좌표와 Form bbox 하단/중앙 배치를 함께 고정한다.
- 대표 이미지는 merge SHA 고정 raw URL로 `pr_6281_issue6266_p1_visual_review.png`를 첨부한다.

## 후속

추가 메인터너 보정 필요 없음.
