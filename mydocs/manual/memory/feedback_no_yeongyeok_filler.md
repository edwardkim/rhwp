---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: ""
metadata: 
  node_type: memory
  originSessionId: da1865ca-614e-44a5-8c3e-ce3fe8956096
---

"영역" 이라는 단어를 placeholder filler 로 사용하지 않는다.

**Why:** 본 세션에서 누적된 결함 — 정확한 단어/조사가 떠오르지 않을 때 "영역"으로 빈 곳을 메우는 습관이 굳어졌고, 작업지시자가 "왜 자꾸 출력에 영역이라는 글자가 반복되는지요?" 라고 명시적으로 지적함. 의미 단위가 모호해지고 문장이 추상화되어 가독성이 크게 떨어짐.

**How to apply:**
- "영역" 은 **실제 공간/범위 (area, region, scope)** 를 의미할 때만 사용
- 빈 자리 메우기에는 사용 금지 — 그 자리에 적합한 조사 (의/에/와/를/는) 만 쓰거나 단어 자체 생략
- PR review/report 문서, 보고서, 단계별 stage 문서, 일반 대화 모두 동일 적용
- 작성 후 다시 읽어보고 "영역" 이 의미 없이 들어간 곳 제거

### 정정 예시

❌ "rhwp 영역 영역 적용" → ✅ "rhwp 적용"
❌ "본 세션 영역 내내" → ✅ "본 세션 내내"
❌ "PR review 문서 영역" → ✅ "PR review 문서에서" 또는 "PR review 문서"
❌ "충돌 해결 영역 영역" → ✅ "충돌 해결" 또는 "충돌 해결 과정"
❌ "자동 검증 영역 모두 통과" → ✅ "자동 검증 모두 통과"
❌ "**측정 영역**" (소제목) → ✅ "**측정 항목**"

### 합법적 사용 예시

✅ "본 task scope 영역" (실제 범위 의미)
✅ "rendering 영역과 parsing 영역 분리" (코드 영역/도메인 의미)
✅ "충돌 영역" (실제 코드 hunk 위치 의미)
