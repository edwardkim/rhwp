---
kind: pr-review
status: approved-integration-candidate
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5816 검토 - 목차 탭 리더

## 판정

- source head `a7ade094b9c510b9b51725bbbd6fa79bfa5d7cbb`를 적용했다.
- 인라인 탭과 목차 탭의 채움 종류를 공유해 점선 리더를 출력한다. 차단 결함은 없다.

## 검증과 시각 증적

- `issue_5799_tab_leader` 2/2 및 통합 전체 nextest 8,109/8,109 통과.
- 기존 기준 `pdf/SO-SUEOP-2024.pdf` p2와 SVG를 visual sweep으로 비교했다. pixel match 94.84984%, proxy 34.17711%이며, 사람 대조에서 목차 점선·쪽 번호 구조를 확인했다.
- HWP 2020 MCP 재변환은 `run_status=139`로 실패해 기존 추적 PDF를 기준으로 사용했다. 이 한계와 대표 화면은 `mydocs/pr/assets/pr_5816_issue5799_review_002.png`에 보존했다.
