# seo-rii 통합 검토 4단계 — 한컴 극한 이름의 공통 출력 보정

## 원인·독립 근거·적용 경계

[3단계](task_m100_536_integration_review_stage3.md)의 원본 `equation-lim.hwp`와 독립 Print는
Native 88.42676%, WASM 89.77376%였다. 통합 전 devel에서도 같은 미달이지만
원본을 제거하거나 기대값을 완화하지 않고 실제 출력을 보정한다.

11pt·16pt 대조 입력은 기본 `ref_empty.hwpx`에 공개 `edit insert-equation` API로
`lim _{x rarrow 0} x + Lim _{x rarrow 0} x`를 넣어 생성했다. 한컴 MCP engine 2020이
전처리 없이 PrintMethod=0으로 출력했으며, 두 PDF의 ToUnicode와 실제 Tf 연산을 대조했다.
같은 HyhwpEQ의 ASCII lim/Lim과 PUA 본체에서 이름/본문 em 크기는 각각 110/92, 160/133이다.
이름과 본체의 글리프 형상·em 보정 근거이며 `sin/log` 등 함수명을 일괄 확대하는 규칙이 아니다.
과거 #280의 대체 수학 글꼴 판단과 이번 실제 HyhwpEQ 공급을 구분한다.

## 공통 결과와 실제 소비

| 생산·소비 위치 | 변경과 계약 |
| --- | --- |
| `EqLayout::layout_limit` | 기존 HYhwpEQ·비보호 상자 경로에서 ASCII 이름의 glyph run·반올림 advance·em 크기를 함께 생산한다. 이름을 기존 Limit의 optional child로 보존한다. 이름 em은 기본 크기의 6/5이며 논리 기준선은 기본 수식 글줄에 정렬한다. |
| Limit 폭·높이 | 이름과 첨자의 폭 최댓값으로 폭을 정한다. 이름 상자와 첨자 끝이 부모 높이에 포함된다. 이후 Row의 기준선 정렬과 저장 제어 상자의 horizontal scale이 공통 결과를 소비한다. |
| SVG / Native Skia / WASM Canvas2D | 이름 child가 있으면 그 glyph/상대 좌표/height의 em 크기로 재귀 출력한다. 각 backend에서 이름을 다른 글꼴로 다시 고르거나 폭을 재추정하지 않는다. |
| paint JSON / CanvasKit | optional name child를 같은 wire 상자로 전달하고 CanvasKit이 그 상자를 재귀 소비한다. child가 없는 기존 payload의 fallback은 보존한다. |
| 글꼴 요청 / preflight | Rust와 TS 순회가 이름 child까지 방문한다. 첨자 없는 lim도 실제 HyhwpEQ face를 준비하고 glyph·응답 예산 검사를 거친다. |

일반 `EqLayout::new`, 다른 수식 글꼴, 크기 보호 저장본의 이름은 기존 경로다.
기존 command golden은 새 optional field를 무시하도록 match만 갱신하며 기대값은 바꾸지 않는다.
측정이나 출력을 문서 ID로 분기하지 않고 목표 픽셀에 맞춘 clamp·출력 숨김을 추가하지 않는다.
수정된 파일은 기존 대형 파일의 국소 변경이며 새 코드 파일을 1,000줄 넘게 만들지 않았다.

## 검증 상태

직전 후보의 27개 case 집중 검사는 22개 suite를 컴파일하던 중 수식 보정을 먼저 반영하기 위해
명시적으로 중단했다. 예산·출력 공백으로 실패한 것이 아니며 이 실행을 PASS/FAIL 건수에 넣지 않는다.
실제 프로세스와 종료를 확인했고 `candidate-focused-native-interruption.json`에 이유를 기록했다.
공용 target은 삭제하지 않았다.

이 단계의 출력 개선은 아직 미검증이다. 새 Native/fresh WASM 출력과 확대 판독에서 관련 범위가
90% 이상인지 먼저 확인하고, 그 뒤 이름·첨자의 포함/중복·보존·글꼴 선택과 비적용 경계의 정식
회귀를 추가한다. 아직 실제 원본에 대한 px 핀·fixture·baseline을 추가하거나 완화하지 않았다.
코드가 바뀌었으므로 3단계의 producer/renderer 검증을 이 보정의 결과로 재사용하지 않는다.
새 후보의 관련 lint·실제 출력·브라우저·전체 회귀 검증과 원 PR별 최종 판정은 남아 있다.
