---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: feedback-nonfunctional-dx-investment
description: "기능은 사용자를, 비기능적 견고함은 위대한 기여자를 끌어들인다 — rhwp는 개발자 경험(DX) 비기능 투자를 지속해야 한다는 방향 지침"
metadata: 
  node_type: memory
  type: feedback
---

작업지시자의 방향 지침 (2026-07-18): "소프트웨어의 기능은 사용자를 끌어들이지만,
비기능적 견고함은 위대한 기여자를 끌어들인다." rhwp는 기능 구현뿐 아니라
**압도적인 개발자 경험을 위한 비기능적 구현**에 끊임없이 투자해야 한다.

**Why**: 이 프로젝트의 실증 — 기여자 품질이 비기능 인프라와 함께 상승했다.
red→green 관례·잠정 핀 이력 규약(CONTRIBUTING 명문화), 원장 트립와이어
(#2329/#2336), e2e MANIFEST+검사기(#2353), 문서 거버넌스(#2331), 정답지
PDF·오라클 동봉 관례가 있는 곳에서 외부 기여자들이 스스로 수정 전 실패 증명·
반례 자기 가드·좌표 실측 반론까지 수행했다. 인프라가 기준을 보이면 기여가
그 기준으로 온다.

**How to apply**: ①검토·설계 판단에서 기능 대비 비기능(검증 도구·가드·문서·
재현성·명명 체계) 투자를 동급 우선순위로 취급 ②반복 마찰(드리프트, 공허 통과,
수동 대조)을 발견하면 일회 정정으로 끝내지 말고 구조화(검사기·manifest·규약
명문화)를 제안 ③기여자용 스캐폴드(CONTRIBUTING·오라클·자가 검증 도구)의
공백을 보이는 즉시 보완 제안. 관련: [[project-governance-bootloader]],
[[feedback-process-must-follow]].
