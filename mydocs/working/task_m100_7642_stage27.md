# 이슈 #7642 — 근 지수의 소속과 정체 수식 전진폭의 공통 소비

시작 커밋: `10df59abb`. Stage26의 독립 정상 Save/Print와 확대 판독을 근거로
구현한다. 전체 회귀는 최종 110 FAIL이며 모두 현 브랜치 보정 범위다.

## 규칙·원인과 구현 범위

1. `ROOT 3 OF x`에서 3은 근 지수, x는 피연산자다. 다음 토큰이 OF인 단일
   원자 인자도 지수로 소비한다. OF가 없는 `root3`의 일반 근호 소속과 기존
   묶인 지수/LaTeX 구문은 유지한다. ROOT 처리를 작은 parser 자식 모듈로 옮긴다.
2. 함수와 정체 글자의 glyph 전진은 같은 논리 글꼴 메트릭을 사용한다.
   일반 ppem의 FreeType 합산은 독립 단어 56건 중 27건만 일치했다. 한컴의
   논리 크기에 대응하는 짝수 ppem에서는 54건이 일치했지만 x/y 두 건은
   여전히 달랐다. 독립 Print의 문자 원점·정상 캐시로 측정한 전진 데이터를
   같은 논리 크기에서 재사용한다. 사설 폰트 바이트·윤곽은 포함하지 않는다.
3. 함수명은 AST의 함수 역할을 유지하고, paint용 정체 glyph 원점을 별도의
   폭 추측 없이 공통 결과로 소비한다. Source face가 아닌 경로는 기존
   표시 글꼴 메트릭을 소비한다. 굵은 함수도 독립 정체 메트릭의 검증 범위가
   아니므로 기존 경로를 유지한다. θ/각 기호의 논리 전진도 독립 캐시와 대조한다.
4. 함수 뒤 직접 여는 괄호와 그리스 변수 뒤 닫는 괄호는 operand 경계를
   소비한다. 독립 `cos(theta)` 14/14 크기의 동일 간격 계약을 근거로 삼고
   원본 명시적 Thin·닫는 괄호 묶음의 중복 간격 방지 계약은 유지한다.

## 실제 소비 경로와 검증

`parse_sqrt`의 원자/OF 소속 → AST → `layout_sqrt`의 지수/몸체 소속,
함수의 공통 정체 glyph 원점 → Row 점유 → inline/composer → SVG/Canvas
paint를 대조한다. 배치의 나중 재계산·bbox 강제 장평으로 일치를 대신하지 않는다.

동일 Print의 독립 근호 28건·정체/함수 98건과 원본 Nov 17–21쪽의 Native
출력·review/확대를 먼저 확인한다. 일반 근호와 정상 대조군을 함께 확인한다.
근호 기하·표시 글리프의 남은 차이는 별도로 보정하며 폭 일치만으로 완료하지 않는다.
기존 검사·golden·baseline은 출력 보정과 Native/fresh WASM의 선행 기준을
충족한 뒤 필요한 관계형 검사로 변경한다. 이번 소단계에는 기대값 갱신을 하지 않는다.

## 현재 브랜치의 회귀 보정 범위

이번 작업의 사용자 지시에 따라 현재 회귀 실패는 전부 이 브랜치에서 수정한다.
일반 지침의 별도 이슈 이관으로 이번 실패를 처리 완료로 바꾸지 않는다.
`5de3d6e62` 전체 검증의 110개 실패를 검사 소스에 모두 연결했으며, 이 목록과
실패 본문은 ignored `output/pr-review/issue7642/failed-inputs-20261008/`
`stage15-current-validation/`에 보존한다. 이전에 따로 이관한 HML 작업과
이번 110개 실패 목록을 혼동하지 않는다.

## 선행 확인 결과

증적 공통 경로는 위 `stage15-current-validation/`이다. 첫 Native 빌드는
`FontStyleKind` import 누락으로 exit 101이었다. import를 추가한 재실행은
exit 0이며, 출력 바이너리 SHA-256은
`9e5814a5eb1ee43af219d07e00184a92e94edc30290a6b13e1441b6e3464666c`다.
`roman-function-root-native-provenance.json`의 source 파일 해시로 고정한다.
이 바이너리에는 이후 추가한 굵은 함수의 기존 경로 유지 조건이 포함되지 않으므로
최종 코드 검증과 구분한다.

| 확인 항목 | 관측 결과 | 증적 |
| --- | --- | --- |
| 근 지수의 소속 | `root 3 of x`가 지수 3·몸체 x의 Sqrt로 해석됨 | `radical-function-source-ast-roman-function-root.log` |
| 정체 글자·함수 14 크기 98건 | 공통 layout 폭이 정상 한컴 저장 폭과 98/98 일치 | `roman-function-root-width-contract.json` |
| 근호·함수 28건 Native 1쪽 | 일치율 94.62235%, 누락·쪽수 차이 없음 | `roman-function-root-diagnostic-native/radical-function/summary.json` |
| 정체 글자·함수 Native 3쪽 | 최저 97.39501%, 누락·쪽수 차이 없음 | `roman-function-root-diagnostic-native-retry/roman-advance/summary.json` |
| 원본 2022년 11월 17–21쪽 Native | 최저 96.84871%, 누락 없음. 전21쪽 최종 검증과 구분 | `nov2022-roman-function-root-native/summary.json` |
| 같은 영역 확대 판독 | 근호의 올라가는 선·높이, 분수 괘선과 θ 기울기 등의 차이가 남음 | `representative-native/radical-function-root-after/roots-functions-3x.png` |

정체 글자 3쪽의 첫 driver는 SIGTERM(exit 143)으로 종료되었다. 원인은 미확인이고
통과·시각 결함으로 판정하지 않는다. 원 로그·중간 자료를 보존했으며 별도 retry
경로의 재실행은 exit 0이었다. 폭 일치와 90% 통과를 수식 모양의 완료로 간주하지 않는다.
이 단계의 fresh WASM·전체 회귀·필수 lint 묶음은 아직 미검증이며 PR 준비 완료가 아니다.

## 최종 코드의 소단계 검증

굵은 함수의 기존 경로 유지 조건을 포함한 Native 재빌드도 exit 0이다.
바이너리 SHA-256은 `176351d72e5ca9081447a8dcb4f6843409a1e417f97b69d46c311bdca1ce8ff7`이며
`roman-function-root-guarded-native-provenance.json`에서 소스를 고정한다.
이 바이너리로 대조 입력을 새로 출력했으며 1쪽 94.62235%, 정체 글자 3쪽
최저 97.39501%로 재확인했다. 재실행 결과는
`roman-function-root-guarded-diagnostic-native/results.json`에 있다.
`cargo fmt --all -- --check`, `git diff --check`와 Native
`cargo clippy --locked --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings`가
모두 exit 0이다. WASM Clippy·workspace all-targets Clippy·fresh WASM·전체 회귀는
추후 최종 코드에서 실행할 필수 게이트로 남는다. 기존 검사 기대값·baseline은 바꾸지 않았다.
