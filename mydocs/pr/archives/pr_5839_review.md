---
kind: pr-review
status: approved-integration-candidate
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5839 검토 - HWP5 bookmark CTRL_DATA

## 판정

- source head `e82f0bfd1fb3f551b972b06daf7de8a4d74c4f3b`를 적용했다.
- HWPX→HWP 변환에서 bookmark 이름을 CTRL_DATA로 보존한다. 차단 결함은 없다.

## 검증

- `issue_5838_bookmark_ctrl_data` 1/1은 oracle과 이름을 비교하며 통과했고, 통합 전체 nextest도 통과했다.

## 최종 판단과 GitHub 기록

- **수용**: #5844 전체 CI와 bookmark oracle 계약이 성공했다. 추가 메인터너 보정과 보류 항목은 없다.
- merge 뒤 원 PR에는 bookmark CTRL_DATA 보존 수용 근거를 comment로 남기고 close한다.
