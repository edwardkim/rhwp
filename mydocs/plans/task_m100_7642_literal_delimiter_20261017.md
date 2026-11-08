# #7642 일반 수식 괄호의 실제 글리프와 논리 폭 연결

## 독립 근거와 위반 경로

[stage2088](../working/task_m100_7642_stage2088.md)의 10월 원본 7쪽에서 일반 괄호의
측정은 HYhwpEQ의 논리 전진폭을 쓰지만 표시는 `LayoutKind::Symbol`의 다른 수학
글꼴을 쓴다. 같은 원본의 독립 Print는 HYhwpEQ U+E044/U+E045다. 설치된 실제
프로그램의 두 PUA 전진은 398/1024이며 ASCII 괄호의 512/1024와 다르다.
문서 좌표·문서 ID나 점수에 맞춘 임의 보정으로 해결하지 않는다.

`layout_symbol → layout_hancom_glyphs → GlyphRun → layout row/flow frame →
layout_for_paint → SVG/Canvas/Native 재생`에서 일반 괄호도 같은 선택 결과를 쓴다.
기존 실제 글리프 런의 기준선·전진 계산을 재사용한다. `LEFT/RIGHT`의 확장 괄호와
일반 literal 괄호를 구분하고, 다른 face·미지원 face 메트릭의 fallback을 유지한다.
새 glyph가 선형 런으로 분류되면서 저장 폭과 자연 표시 폭의 소비가 달라지는지 확인한다.

## 실행과 완료 조건

10월 HWP/HWPX 7쪽의 전후 실제 글꼴·괄호 위치·앞뒤 내용과 보기의 수식 간격을
독립 Print와 대조한다. 11월 15쪽과 다른 크기·정체/이탤릭·확장 괄호를 대조군으로
묶는다. 전쪽 Native/fresh WASM·devel 대비 하락·직접 판독을 각각 판정한다.
원본의 저장 크기·수식 문자열·본문·PNG/PDF·마스크·점수 기준은 바꾸지 않는다.
새 회귀는 관련 전쪽 양쪽 90%와 확대 판독 뒤 별도 이름으로 작성한다.

stage2089의 기존 source native Clippy는 큰 `const ADVANCES`에 대한
`large_const_arrays` 오류로 exit 101이다. 메트릭 값을 바꾸지 않고 `static`으로
한 번 보존하도록 같은 보정 batch에서 수정한다. 경고 억제를 추가하지 않는다.
이후 같은 공용 `target/pr-review`, 빌드 잡 8개로 Native/fresh WASM과 lint를 순차 실행한다.
이 계획과 실제 source 변경·후속 보고서를 함께 커밋하고, 미실행 검사를 통과로 쓰지 않는다.
