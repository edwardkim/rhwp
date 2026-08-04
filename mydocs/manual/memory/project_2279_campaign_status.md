---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: project-2279-campaign-status
description: "#2279 정밀도 캠페인 — 92셋 100% 완결(2026-07-18), umbrella 존치, #2373(#2352 revert 권고) 결정 대기"
metadata: 
  node_type: memory
  type: project
---

2026-07-18 기준 #2279 umbrella 상태:

- **92 컨트롤셋 92/92 (100%) 완결** — planet6897 스택 #2376→#2382→#2383→#2391
  merge 로 86→92 완전 소거. 독립 #2379 로 #2291 잔여 연결맵 −34→−1 (385→414쪽,
  한글 415).
- **umbrella 는 존치** (작업지시자 결정): 통합 축 #2136/#2137/#2138/#2148/#2246
  OPEN + #2373 신규 귀속.
- **#2373 미결**: planet6897 이 merge 된 #2352(TAC host 줄박스 가산, 97acfed6)의
  +1 회귀 3건을 커밋 이분으로 확정, 발동 IR 동일로 게이트 협소화 불가 증명,
  10k 표본 순 −5 → **revert 권고**. revert 시 92셋 92→91 (36392557 트레이드).
  92셋이 결재문서 −1축만 측정해 +1 회귀가 사각이었음 — +1축 회귀 게이트 추가
  제안 포함. **revert 여부는 작업지시자 결정 대기.**

관련: [[feedback-visual-regression-grows]], [[feedback-hancom-compat-specific-over-general]]
