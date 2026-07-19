---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: 시각 판정은 작업지시자 직접 결정 영역
description: 한컴 2022 편집기 / 브라우저 시각 검증은 Claude가 단정 회피, 정량 측정만 보조
type: feedback
originSessionId: 4861649d-834a-43c6-a262-9f08333360e8
---
렌더링/레이아웃 정합성 판단은 **작업지시자가 직접** 시각 판정으로 결정한다. Claude는 단정 회피.

**Why**: 한컴 편집기(2010/2022) = Windows에서만 동작하는 시각 정답지. PDF는 정답지가 아님. Claude는 한컴 환경 외부에서 시각을 직접 확인할 수 없으므로 의견 단정 시 잘못된 방향으로 작업 유도 위험.

**How to apply**:
- 렌더링 차이/회귀 분석 시 Claude는 **정량 측정**만 제공 (glyph 출현 횟수, byte-identical sweep, IR diff)
- "이 결과가 정답이다" 단정 회피 — "한컴 편집기로 확인 필요" 명시
- 광범위 sweep 보고: "N 샘플 M 페이지 K differ" 형식의 정량 결과 우선
- before/after SVG는 `output/svg/pr{N}_before/` + `output/svg/pr{N}_after/` 패턴

**예외**: 본인이 명시적으로 "이 출력이 옳다"고 판단을 명시한 경우만 그 판단을 기준으로 작업.
