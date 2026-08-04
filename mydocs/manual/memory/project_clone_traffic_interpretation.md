---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: project-clone-traffic-interpretation
description: "rhwp GitHub 클론 트래픽 해석 규칙 — uc는 기관 NAT 뒤에서 1로 잡힘, 2026-07-11 폭증 사례(0.7.18 릴리즈 직후 23.9k/uc 113)"
metadata: 
  node_type: memory
  type: project
---

rhwp 클론 트래픽 해석 규칙 (2026-07-13, 작업지시자 확인):

- **unique cloners(uc)는 egress IP 기준** — 기관 내부망(NAT)에서 수백 대가
  클론해도 uc 1로 잡힐 수 있다. uc가 평시 수준인데 총 클론만 폭증하면
  "소수 기관의 대량 내부 빌드"로 해석한다 ("이용자 급증"으로 오독 금지).
- **2026-07-11 사례**: v0.7.18 공개(7/11 01:48 KST) 직후 클론 23,871(uc 113),
  7/12 10,728(uc 91). 점검 결과 저장소 내부 요인 없음(자체 Actions 304 run,
  npm 스크립트/문서 clone 지시 없음). 다운스트림 cargo git 의존 공개 저장소
  6곳+ 존재하나 공개 CI는 규모 미달 → 비가시 기관 CI 추정. 조치 불요 판단.
- **구조적 완화책 = crates.io 배포** (0.8.0 시점 검토): git 의존 → registry
  의존 전환으로 클론 부하가 CDN 이전. `rhwp` 이름 미선점 확인(2026-07-13),
  메타데이터 요건 충족, git/path 의존 없음. 사전 작업: 패키지 다이어트
  (10MB 제한 vs 저장소 868MB — include 필드), publish --dry-run 게이트,
  feature 점검. [[project_v080_hwpx_save_milestone]]
