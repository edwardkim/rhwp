---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: 결함 진단 시 layer 귀속 정확화
description: LAYOUT_OVERFLOW_DRAW 같은 진단 메시지의 emission 위치와 결함의 본질 위치를 혼동하지 말 것. 시프트의 출처를 추적해야 함
type: feedback
originSessionId: 4ef64500-5782-4a52-a9a8-9330ea6c128b
---
LAYOUT_OVERFLOW_DRAW / LAYOUT_OVERFLOW 같은 진단 메시지가 emit 되는 위치 (`paragraph_layout.rs:875` 등) 는 **결함의 증상이 표면화하는 위치**이지 **결함의 본질 위치**가 아니다. 시프트의 출처 (어느 pi 부터 layout y 가 권위값에서 벗어나기 시작했는지) 를 추적해야 본질 위치를 찾을 수 있다.

**Why:** Issue #716 진단에서 처음 `paragraph_layout.rs:875` Task #332 Stage 4b 잔존 영역으로 가설 제시했으나, 작업지시자 진단으로 "pi=3 의 y=211.5 가 본질" 이라는 정확한 시프트 출처 지목 → 본질 위치는 `layout.rs:1403-1555` VPOS_CORR 진입 조건 영역으로 정정. 메인테이너 진단이 정확했고 내 첫 진단은 표면 layer 만 보았다.

**How to apply:**
1. LAYOUT_OVERFLOW / LAYOUT_OVERFLOW_DRAW 진단 시 — 디버그 오버레이 SVG 의 pi 별 y 라벨을 권위값 (HWPUNIT vpos 의 px 환산) 과 비교하여 **어느 pi 부터 시프트 시작**인지 추적
2. RHWP_VPOS_DEBUG=1 출력에서 VPOS_CORR 호출 누락 / applied=false 영역 확인
3. 시프트 시작 pi 의 직전 paragraph 의 controls (Table / Shape / Picture) 영역 점검 — overlay 가드 / treat_as_char / wrap 영역의 VPOS_CORR 진입 차단 여부
4. emission 위치만 보고 "이 코드가 결함" 이라고 결론짓지 말 것 — 그 코드가 받는 입력 (layout y) 의 출처를 거슬러 올라가기
