---
name: feedback_pr_supersede_chain
description: "동일 컨트리뷰터의 연결 PR 처리 네 패턴 — (a) close+통합 머지, (b) 머지+supersede 통합, (c) 머지+회귀 정정 후속 통합, (d) 상호 보완 둘 다 머지. review 시작 시 같은 컨트리뷰터의 전후 PR 의도 점검 필수"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

동일 컨트리뷰터가 올린 연결된 PR들의 처리 패턴 (사례 축적):

## 패턴 (a): close + 통합 머지
**사례**: PR #649 → #650. PR A의 commit이 PR B에 포함되면 A는 close, B에서 통합 머지.
단순 cherry-pick 후속 처리라 시각 판정 게이트를 임의로 부과하지 않는다.

## 패턴 (b): 머지 + supersede 통합
**사례**: PR #657 → #662. A(휴리스틱)가 머지된 뒤 B가 같은 자리를 본질 수정으로 대체 —
동일 hunk 충돌은 rebase 요청이 아니라 메인테이너가 충돌을 해결하며 통합(A의 보존 가치 +
B의 본질 수정).

## 패턴 (c): 머지 + 회귀 정정 후속 통합
**사례**: PR #723 → #732. A 머지 후 시각 판정에서 별건 회귀 발견 — rollback 대신
같은 컨트리뷰터의 후속 PR B에 회귀 정정을 묶어 처리.

## 패턴 (d): 상호 보완 — 둘 다 머지 (2026-07-05, #1912/#1919)
**사례**: 같은 이슈(#1898, tac 그림 문단 뒤 +11.7px)를 A(#1912)는 하류(재역산 가드),
B(#1919)는 상류(초기화 면제)에서 수정. 상호 참조 없어 supersede로 보였으나 **실측
(SVG 글리프 y 시퀀스)** 결과 결함 전이 3곳 중 B 단독은 2곳만 해소(87→88 잔존 — 리셋
원인이 tac 개체가 아니라 직전 표), A는 3곳 전부 해소하되 Picture 한정이라 B의
Shape/Equation 자기-리셋 예방을 못 대신함. **어느 쪽도 상위집합이 아님** → 둘 다 머지.

**Why**: (d)에서 코드 구조 비교만으로 "B가 더 근본적이니 A close(superseded)"를 권고할
뻔했다. B의 자기 핀이 잔존 전이를 커버하지 않아 게이트 통과가 완전 수정을 보증하지
않았고, 실측으로만 뒤집혔다.

**How to apply**:
1. review 시작 시 같은 컨트리뷰터의 열린/직전 PR을 점검(gh pr list --author)하고 본문의
   supersede/연결 의도 명시 여부 확인.
2. 같은 결함을 다루는 두 PR은 supersede로 단정하지 말고 **각 PR 단독 + 결합 상태를
   같은 지표로 실측**해 커버리지 매트릭스를 만든 뒤 판단 (패턴 d).
3. PR의 자기 테스트(핀)가 증상 전체를 커버하는지 검증 — 핀 통과 ≠ 완전 수정.
4. 충돌/회귀/중복 발견 시 자동으로 rebase 요청·rollback·close를 권고하지 않고 위 패턴
   중 어디인지 판별해 옵션으로 작업지시자에게 제시.

관련: [[feedback_visual_judgment_authority]] [[feedback_close_issue_verify_merged]]
[[feedback_check_open_prs_first]] [[feedback_no_yeongyeok_filler]]
