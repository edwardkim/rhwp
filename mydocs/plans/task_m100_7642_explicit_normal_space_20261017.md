# #7642 HancomEQN 명시적 normal 공간의 공통 전진

## 독립 근거와 원인

12쪽 `vector-operator-gaps-saved-2024.hwp`의 한컴 2024 Print를 확대하니
HancomEQN 18pt `rm AD~M`에서 M의 시작이 약 4px 앞섰다. 90% 통과와
별개로 미세 간격을 직접 판독하여 검출한 결함이다. 같은 96식 독립 Print에서
HancomEQN 6/9/12/18pt의 명시적 normal 공백은 각각 약 4/6/8/12px다.
PDF의 실제 glyph 원점과 공백의 끝을 사용했고 저장 개체 폭을 역산하지 않았다.
Print job은 `9ecc52f7-548a-4842-a0ea-492bd626f1f3`다.

`for_equation`은 HYhwpEQ와 HancomEQN을 구분한다. `layout_space`는 normal의
반 em 규칙을 HYhwpEQ에만 적용하고 HancomEQN은 일반 0.33em으로 흘렀다.
두 실제 face의 normal 공백 계약을 같은 분기에 연결한다. Thin은 두 face의
관측 규칙이 다르므로 그대로 둔다. 일반 글꼴/LaTeX의 normal 규칙도 보존한다.

## 소비 경로와 검증

원본 `~` 토큰 → parser의 Space(Normal) → layout_space의 width → Row 전진
→ FontStyle/장식 바깥 원점 → 실제 GlyphRun/painter를 추적한다. 특정 문서·크기·
좌표에 가산값을 넣지 않는다. 기존 두 face의 96식과 명시적 공간 48식, 실제
시험지를 같은 Print/96dpi/글꼴로 전쪽 Native/fresh WASM 대조한다. 전쪽 90% 및
확대 판독 전에는 새 회귀 테스트나 기대값을 추가하지 않는다. 실제 완료 페이지의
98% 조건을 만족한 경우에만 독립 좌표와 관계 검사를 함께 보호한다.
