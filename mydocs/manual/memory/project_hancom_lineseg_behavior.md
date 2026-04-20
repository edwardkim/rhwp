---
name: 한컴 LINE_SEG 자동 재계산 동작
description: 한컴은 LINE_SEG가 비어있어도(line_height=0) 자체 조판 엔진으로 재계산하여 렌더링. LINE_SEG는 캐시일 뿐.
type: project
---

한컴은 HWP 파일의 LINE_SEG가 비어있거나 기본값(line_height=0, sw=0)이어도 자체 조판 엔진으로 재계산하여 정상 렌더링한다.

**Why:** Task 403에서 LINE_SEG를 default로 생성한 HWP 파일이 한컴에서 정상 렌더링됨을 확인. LINE_SEG는 캐시/힌트이며 한컴은 항상 재계산 가능.

**How to apply:**
- 역공학: LINE_SEG를 비워서 생성 → 한컴에서 열어 저장 → 한컴이 채운 LINE_SEG 분석
- 편집 시 원본 LINE_SEG 보존에 집착하지 않아도 됨
- v1.0.0 전략(자체 조판 결과가 한컴과 동일하면 충분)의 근거
