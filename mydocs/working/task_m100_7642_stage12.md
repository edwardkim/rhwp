# 이슈 #7642 — 수학 명령 연산자의 공통 간격

이전 단계: [글자와 구분자의 논리 전진폭](task_m100_7642_stage11.md).
시작 코드 head: `a5c3751b9`. 사용자가 지적한 2022년 9월 원본 17쪽의 수식은
Native 페이지 일치율 95.25802%와 별개로 확대 판독에서 차이가 남는다.

## 독립 근거와 원인

원본 `samples/3-09월_교육_통합_2022.hwp`의 미주 27, 문단 11에는
`times100`, `times10`이 있다. 같은 원본의 한컴 2020 Print 17쪽과 같은 물리 영역을
576dpi로 비교했다. 둘째 줄의 곱셈 기호 뒤 간격과 이후 분수 위치가 어긋나며
분수 숫자 모양에도 차이가 남는다. 페이지 점수로 이 차이를 면제하지 않는다.

`EqParser::parse_command → EqNode::MathSymbol("×") → EqLayout::row_children →`
`LayoutBox의 원점/전진폭 → SVG/Canvas/Skia`가 실제 경로다.
`symbol_spacing::operator`와 결과 종류 검사가 `Symbol`만 처리해서 `times`를 제외했다.
`layout_math_symbol`의 공개 글꼴 자연 폭이 이 경로에서 한컴 논리 전진폭과 자동 여백을
대신했다. 반면 문자 `+`, `=`는 같은 줄에서 `Symbol`이므로 이미 공통 간격을 소비한다.

정상 HWPX에서 한컴이 재생성한 9/18pt 연산자 대조군의 독립 폭과 matching Print는
이전 단계의 `font-advance-contract-counter/`에 보존했다. `1 times 100+1`은 각각
4179/8358 HWPUNIT이고 원본의 둘째 RHS는 10995 HWPUNIT이다.
명령 이름이나 특정 문서 좌표로 분기하지 않고 같은 연산자 의미에 동일한 규칙을 적용한다.
적분·그리스 문자처럼 이항 연산자가 아닌 `MathSymbol`에는 적용하지 않는다.

## 검증 계획과 현재 판정

공통 간격 생산 단계가 두 AST 종류를 모두 처리하고 기존 paint 소비 지점은 생산된
glyph 원점과 폭을 사용하도록 한다. 새 Native/fresh WASM에서 원본 17쪽과 전체 페이지,
9/18pt 독립 대조군 및 정상 eq-002 대조군을 확인한다. 분수 숫자 모양과 기준선도
같은 물리 배율로 별도 판독한다. 고정 px 회귀·기대값은 바꾸지 않는다.

Rust 컴파일 종료와 Cargo/rustc 프로세스 부재를 확인한 뒤 공통 경로를 보정했다.
Stage11의 wasm-opt는 이미 컴파일한 바이너리만 최적화하므로 해당 산출물은 이전
`a5c3751b9`의 증거로 고정한다. 이 패키지를 수정 후 증거로 사용하지 않는다.
다음 Cargo 검증은 현재 wrapper 종료 뒤 순차 실행한다. 수정 후 시각 판정은 아직이다.
TSV·로그·진단 JSON·nextrun은 ignored output에 둔다.

## 수정 후 Native 확인

코드 head `98d14099bded29215992473414789bb5242e8615`의 Native Clippy와
WASM Clippy는 각각 exit 0이다. `cargo fmt --all -- --check`도 exit 0이다.
Native `release-test` 빌드는 exit 0, 5분 30초이며 고정 바이너리와 소스 해시는
ignored `equation-command-operator-native-provenance.json`에 있다.

같은 원본 17쪽은 **95.25802 → 96.95208%**, 독립 9/18pt 연산자 대조군은
**85.84785 → 91.17251%**다. 수정 후 representative review와 같은 물리 영역의
576dpi compare를 생성해 직접 판독했다. 둘째 줄의 `×100`·`×10`의 간격과 이후
분수 시작 위치는 개선됐다. 곱셈 기호의 실제 ink 크기와 일부 기준선 차이는 남아 있어
확대 판독 전체를 충족으로 기록하지 않는다. 공개 표시 글꼴과 원 face의 ink bounds를
대조하는 진단 자료는 `math-fallback-glyph-bounds.json`에 보존한다.

최신 source의 fresh WASM 및 전체 페이지 검증은 미검증이다. 이전 stage11 패키지의
17쪽 WASM은 95.25802%로 Native와 같지만, 이것은 수정 전 결과다. 회귀 기대값이나
baseline은 갱신하지 않았으며 PR 제출·승인·통합 완료로 판정하지 않는다.
