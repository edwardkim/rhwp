---
kind: pr-review
status: approved-integration-candidate
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5826 검토 - 개정문 하이픈 글리프

## 판정

- source head `e82165567c59b6873b44d33940076ad6106885db`를 적용했다.
- 하이픈 탭 리더를 대체 실선이 아니라 글리프로 출력하고 golden으로 계약을 고정한다. 차단 결함은 없다.

## 검증

- `issue_5804_dash_leader_glyphs` 3/3, 통합 전체 nextest 8,109/8,109 통과.
- `tests/golden_svg/issue-677/bokhakwonseo-page1.svg`는 현 출력과 SHA-256이 일치하는 #5826 golden으로 보정했다. 이는 #5836의 중복 변경 마지막 commit이 예전 golden을 되돌린 충돌을 해소한 메인터너 보정이다.
- 기준 `pdf/issue1921/86712_regulatory_analysis-2024.pdf` p35는 pixel match 96.80047%, proxy 9.38572%였다. 이 문서의 전체 PDF typography 차이는 남지만, 하이픈 글리프 계약은 SVG assertion과 golden으로 직접 검증했다. 대표 화면은 `mydocs/pr/assets/pr_5826_issue5804_review_035.png`다.

## 최종 판단과 GitHub 기록

- **수용**: #5844 전체 CI와 하이픈 글리프 계약이 성공했다. #5836 중복 revert가 복원한 stale golden은 현 #5826 출력과 byte-identical한 golden으로 메인터너 보정했으며, 추가 보정이나 보류 항목은 없다.
- merge 뒤 원 PR에는 #5826 구현을 기준으로 golden 충돌을 보정한 이유와 #5844 통합 수용을 comment로 남기고 close한다.
