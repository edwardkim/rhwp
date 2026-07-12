---
name: feedback_visual_verification_selective
description: 시각 검증(OVL-step)은 기계적 전수가 아니라 PR 목적·user-visible 동작 기준 선택 적용 — 거버넌스 문서 준수
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

PR 리뷰의 시각 검증은 **모든 샘플 PR 에 기계적으로 수행하는 절차가 아니다**. PR 의 수정
목적과 검증해야 할 **사용자-visible 동작**에 맞춰 수행 여부·도구를 선택한다. 렌더링 결과
확인이 필요하면 `mydocs/manual/visual_sweep_guide.md` 가 기본 진입점.

**Why**: 2026-07-04 작업지시자 피드백(`mydocs/feedback/ovl-step.md`). 일반 PR review /
collaborator-mediated review / 체리픽 누적 검토 모두에 적용. 전수 기계 적용은 리뷰 비용만
키우고, 생략 기준이 없으면 불필요한 sweep 이 관행화된다.

**How to apply** (거버넌스 문서 `mydocs/manual/visual_verification_governance.md` 준수):
- 렌더링 코어 수정 = 시각 검증 필수(OVR 무회귀 + before/after/OVL + 작업지시자 판정).
  도구/문서/CI 전용·studio UI(렌더 무관) = 생략하고 스모크로 대체.
- 자동 도구(sweep/OVR/render-diff/roundtrip)는 후보 검출·무회귀 증명 보조 — 최종 판정
  권위는 작업지시자 한컴 정답지([[feedback_visual_judgment_authority]]).
- OVL 패널 채널 규약: R=오라클, G=B=rhwp → 검정=일치/빨강=rhwp만/청록=오라클만.
  폰트 메트릭 프린지는 일반 현상으로 캡션 명시.
- 판정 PNG 는 `mydocs/pr/assets/pr{번호}_..._review_p{페이지}.png` + SHA 고정 URL 게시.

관련: [[feedback_visual_judgment_authority]] [[feedback_self_verification_not_hancom]] [[feedback_pr_maintainer_comment_required]]
