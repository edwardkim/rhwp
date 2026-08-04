---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: project-github-metrics-interpretation
description: "GitHub 프로필 지표 해석 — Activity Overview 도넛은 비율이라 커밋 다산(하이퍼-워터폴) 워크플로에서 리뷰 비중 정체는 정상 (2026-07-15 실측)"
metadata:
  type: project
---

GitHub 기여 지표 판독 규칙 (2026-07-15, 작업지시자 문의로 실측):

- **프로필 Activity Overview(도넛)는 절대량이 아니라 4유형 비율**. 하이퍼-
  워터폴은 PR 1건 처리에 커밋 +3~5(검토 기록·최종 보고·orders·merge×2)가
  따라붙어 커밋이 분모를 지배(실측: 커밋 3,111 = 88% vs 리뷰 64 = 1.8%,
  1년 rolling). 리뷰 +10 = +0.2%p → 도넛 변화 안 보임. **집계 누락 아님.**
- 리뷰 집계 여부의 정답 확인법: GraphQL
  `contributionsCollection.totalPullRequestReviewContributions` +
  `pullRequestReviewContributions.nodes` — gh CLI 리뷰도 웹과 동일 attribute.
- "지표가 안 움직인다" 문의가 오면 절대량(집계) vs 표시 방식(비율/필터)을
  먼저 분리해 판독한다. [[project_clone_traffic_interpretation]] 과 동계열
  (uc NAT 집계 착시).
