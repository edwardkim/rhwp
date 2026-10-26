# #8153: 명시 개행 앞 비가시 공백과 왼쪽 글줄의 실제 점유

## 원본·사양·독립 기준

원본 두 편람과 기존 2024 Print를 유지한다. HWP5 §4.2.10 표 43·44의 문단 정렬과 원본의 강제 줄 나눔,
HWPX의 paraPr 1169 `LEFT`·`condense=0`, charPr 355의 크기 1000HU·자간 -3%를 대조했다.
228쪽 표의 원문은 `󰋯연구자의 정책연구 윤리 ` 뒤에 `lineBreak`가 있고 다음 줄에서
`준수 의무를 일차적으로 관리 감독`을 잇는다. 구분 공백과 원본 줄·셀 소유를 없애지 않는다.

같은 원본 Print에서 첫 줄의 마지막 `리` 원점은 x=473.44035px다. Native source `ffc7cf81b4`의
실제 공급 후에는 x=470.46857px이며, 추가 자간 -0.247619px가 들어간다. 줄의 가용 폭은
141.33333px이고 원래 자연 폭 144.8px에는 말미의 비가시 공백 6.45333px가 포함돼 있다.
가시 끝은 원래 폭 안에 든다. 원본 -3% 자간에 별도 압축을 더하는 규칙은 사양에 없다.

기존 함수가 연속 폭을 나누는 방법과 실제 글리프 점유는 구분한다. 독립 Print 원점과
실제 공급 Native 그림을 기준으로 판단하며, 글꼴을 공급하지 않은 기본 context 진단은
다른 메트릭이므로 같은 원인 재현의 근거로 쓰지 않는다.

## 기존 경로·공통 결과와 반례

관련 선행 로컬 보정은 `89b929b8c0162b5267b8410d0b89d05d0ca3ae40`다.
`VisibleTextLineMeasure`로 가운데·비분배 Justify 줄의 공백·말미 글리프를 함께 측정했고,
46·373쪽의 이전 실패와 정상 대조를 `issue_8153_final_fit_contract_20261026.md`에 기록했다.
현재 Left/Right fit에는 그 결과가 전달되지 않아 명시 개행 앞 공백까지 압축 예산으로 삼는다.

`VisibleTextLineMeasure::new → line_fit_width → compute_line_extra_spacing_with_dash_plan →
run.style.extra_char_spacing/layout_positions → PaintOp.TextRun`의 같은 결과를 공유한다.
왼쪽·오른쪽 정렬도 같은 가시 점유로 fit을 판정한다. 문서명·쪽·문자열 분기는 제품에 넣지 않는다.
기존 helper의 밑줄·취소선·음영·글자 테두리 공백 보존과 개체·탭·각주·겹침 제외를 유지한다.

실제 가시 내용이 넘는 셀은 압축돼야 한다. 원래 #6196의 저장 한 줄 수용과
`bd8919896553684f2a0e1e3d5da968965c8eac4f`(#6303)의 수렴 목적을 유지하고 해당 검사를
수정 전·후 정상 대조로 실행한다. `issue_7081_cell_trailing_space_alignment`의 Right/Center,
`issue_6028_underline_soft_wrap_trailing_space`의 장식 공백과 기존 #8153 가운데·Justify·
condense·명시 탭을 함께 확인한다. 실제 렌더·원점·포함을 검사하며 helper 반환값만 검증하지 않는다.

## 수정 전 증거와 이후 검증

진단 재생은 실제 Native와 수정 전 RGB가 완전히 같음을 확인한 뒤 문제 런의 추가 자간만
원래 0으로 돌렸다. PDF·원본·공간·다른 런을 바꾸지 않았고, 미대응 18 → 0픽셀,
raw 99.98024 → 100%였다. 이는 제품 수정이 아닌 원인 분리다. 실제 공급 context의 두
원본에서 동일 원점 FAIL을 먼저 확인하고 제품의 공통 생산 경로를 수정한다.

현재 한자 source `ffc7cf81b4`의 전쪽 Native와 fresh DEV WASM은 고정 producer에서 계속 진행 중이다.
새 source는 별도 빌드와 before/after 원점·내용 소유 검사, Native/fresh WASM 영향 페이지,
전쪽 TSV로 검증한다. 새 회귀 등록은 실제 시각 조건 충족 뒤에 한다. 전 1,532조건의 raw 100%·
정수 미대응 0과 최종 표준 최적화 WASM·전체 Rust·lint·Skia·Studio는 여전히 완료 조건이다.
upstream 동기화와 원격 변경은 하지 않는다.

작업 증거는 `output/pr-review/issue8153-handbook-raw100-20261022/remaining-raw100-diagnostic/continuous-text-shadow/`의
`p228-forced-break-raw-source-contract.json`, `p228-independent-line-origins.json`,
`p228-trailing-replay/comparison.json`과 `p228-prepared-before-ffc7-contract.log`에 보존한다.

## source9bab 실제 제품 집중 검증

`9bab87505c970f392563986e1ddfc468b2aa5eab`의 Native 빌드와 집중 58조건을 완료했다.
두 형식의 228쪽은 각각 raw 99.98024%·미대응 18픽셀에서 100%·0픽셀로 개선됐다.
58조건에서 미대응 증가가 없고, 136·166·167·46·373쪽의 정상 범위도 유지했다.
독립 Print와 제품 review PNG로 같은 표 셀의 명시 개행·뒤 문장·마지막 글자를 대조했다.

실제 공급 context의 같은 검사 파일은 source `ffc7cf81b4`에서 2건 FAIL, 새 source에서 2건 PASS였다.
마지막 글자 원점은 470.46857 → 473.44000px, 독립 Print는 473.44035px다.
기존 저장 셀 축소·수렴·Right/Center·밑줄 공백·Justify 마지막 줄·한자·탭 등의
21개 harness에서 총 49건 PASS·0 FAIL·0 ignored를 확인했다. 이 수량은 전체 Rust suite가 아니다.
CLI 소비 검사는 새 동결 Native 바이너리 경로를 compile/runtime 환경에 고정했고,
임시 SVG는 ignored output의 TMPDIR 아래에만 생성했다.

현재 source의 전쪽 Native 766조건과 fresh DEV WASM의 원본 29쪽씩·탭 대조군 3개를
별도 실행 중이다. 실제 228쪽 검사는 Native 공급 context를 요구하므로 진단 자료에 보존하며,
이 절의 PASS를 아직 정식 자동 수집 검사 등록이나 최종 필수 gate 통과로 쓰지 않는다.
현재 전쪽 100%·미대응 0 조건, 표준 최적화 WASM, 전체 Rust·lint·Skia·Studio와 PR 준비는
완료되지 않았다.

결과는 공통 작업 디렉터리의 `plain-fit-native/contracts-result.json`,
`plain-fit-native/native-focus/result.json`, `plain-fit-p228-contract.log`,
`plain-fit-native-focus.log`와 `plain-fit-native/native-focus/handbook-hwpx/p228/`의
review·overlay PNG에 연결한다.
