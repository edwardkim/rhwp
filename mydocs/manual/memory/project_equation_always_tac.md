---
name: 수식 컨트롤 TAC 판정
description: 수식 컨트롤도 common.treat_as_char에 따라 TAC/비TAC가 갈린다. Control::Equation 자체를 TAC로 단정하지 않는다.
type: project
originSessionId: cf52fb67-2bab-4392-9aef-7cc352063296
---
한컴의 수식 컨트롤은 `Control::Equation`이라는 타입만으로 TAC(treat_as_char)라고 단정할 수 없다.
수식도 `common.treat_as_char` 값에 따라 텍스트 흐름 안의 글자 취급 여부가 갈린다.

반례 샘플:

- `samples/수식-문자처럼취급-아님.hwp`
- `pdf/수식-문자처럼취급-아님.pdf`

이 샘플은 문단 textRun 안에 수식 컨트롤이 있지만 dump 결과가
`수식 ... tac=false`이며, 한컴 PDF도 수식이 본문 글자를 밀어내는 TAC 글자처럼 동작하지 않는다.

**Why:** shape_layout.rs의 독립 수식 배치 코드와 paragraph_layout.rs의 인라인 수식 배치는
`eq.common.treat_as_char`에 따라 각각 살아야 한다. 수식 타입만 보고 인라인으로 강제하면
비TAC 수식 샘플에서 글자 흐름과 겹침/밀림 판단이 깨진다.

**How to apply:** 수식 레이아웃/배치 수정 시 `Control::Equation(_)` 매칭만으로 TAC를 판단하지 않는다.
반드시 `eq.common.treat_as_char`를 함께 본다.

미주/각주 pagination에서도 문단의 `lineSegArray -> line_seg`가 가리키는 줄 범위와
`ComposedLine/TextRun` 안의 control 소속을 확인하되, `Control::Equation(_)`이면서
`common.treat_as_char=true`인 경우만 글자 흐름의 한 칸으로 취급한다.
