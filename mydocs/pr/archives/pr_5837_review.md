---
kind: pr-review
status: approved-integration-candidate
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5837 검토 - secd CTRL_HEADER tail 저장 버전

## 판정

- source head `2b5de51f8f5a395e495aaf7b6b664ccd0eb4a854`를 적용했다.
- HWPX에서 HWP로 저장할 때 secd tail을 저장 버전별로 분기한다. 차단 결함은 없다.

## 검증

- `issue_5249_section_def_ctrl_tail` 2/2(한컴 oracle 비교 포함), 통합 전체 nextest 8,109/8,109 통과.
- native-Skia library 3,950 passed/13 ignored과 focused native-Skia 두 test도 통과했다.
