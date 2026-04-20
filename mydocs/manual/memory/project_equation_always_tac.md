---
name: 수식 컨트롤은 항상 TAC
description: 한컴의 수식 컨트롤은 모두 treat_as_char(TAC)로 처리됨. 독립 수식 개념 없음.
type: project
originSessionId: cf52fb67-2bab-4392-9aef-7cc352063296
---
한컴의 수식 컨트롤은 모두 TAC(treat_as_char)이다. 텍스트 흐름 안에서 한 글자처럼 배치된다.

**Why:** shape_layout.rs의 독립 수식 배치 코드(정렬 기반 단독 배치)는 사실상 사용되지 않아야 하며, paragraph_layout.rs의 인라인 배치가 모든 수식 렌더링의 핵심 경로이다.

**How to apply:** 수식 레이아웃/배치 수정 시 paragraph_layout.rs의 인라인 수식 처리를 중심으로 작업. shape_layout.rs의 수식 배치는 fallback 용도로만 유지.
