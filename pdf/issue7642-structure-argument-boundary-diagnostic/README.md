# 구조 명령 인자 경계의 독립 한컴 진단

회귀 fixture/golden의 승인 자료가 아니라 미달 원인의 진단 자료다.
입력 HWPX를 RHWP로 HWP5 전송본으로 변환한 뒤 한컴으로 일반 저장하고,
그 저장본을 별도 Print job으로 출력했다. 비교 대상은 실제 한컴 저장본이다.
원 HWPX의 직접 MCP 열기는 worker 오류로 실패했으며 그 입력의 일치로 보고하지 않는다.

한컴 저장 전후 수식 script 24개와 추가 대조군 30개는 원문 순서·바이트가 같다.
첫 입력은 누락·빈·정상 인자와 중첩 그룹·분수·LaTeX 이름의 문서 해석을 비교한다.
추가 입력은 MATRIX/PMATRIX/PILE/EQALIGN/CASES 각각의 누락·빈·공백·`#`·`&`·정상 인자를 비교한다.

## 출력 출처

- Hancom engine `2020`, build `11.0.0.9136`, PrintMethod `0`, `hancom2020_pdf_driver_one_up`.
- 첫 입력 저장 job: `a17bfb55-fa9c-4c02-9c42-c8b2d11d03e2`.
- 첫 Print job: `9d05293f-7e11-4403-a3b3-2ccce07bd1a9`, PDF 전체 1쪽.
- 추가 hwp job: `4b80ced6-f9a0-4683-98a3-f41830221de2`.
- 추가 pdf job: `6216a18a-ffc1-4630-b7c7-58c2b4079a71`.

추가 Print PDF도 전체 1쪽이다. 일반 저장으로 개체 크기·기준선·LineSeg가 확정되므로
전송 HWP/HWPX와 저장본을 구분한다. 명령·실행 응답·PDF trace·script 비교·TSV는
ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage318-*`,
`stage320-*`, `stage322-*`에 보존한다. endpoint·인증 값은 공개하지 않는다.

## 파일 SHA-256

- `structure-argument-boundary-hancom-2020.hwp`: `bdde1e2e7aa80c491e21b17a66cbd8b5ebe90c786de5d0807a8e81cddbe08de2`
- `structure-argument-boundary-hancom-2020.pdf`: `19c464e5f3157e96693e59d0bea4db04d6033bfb79d8768f2659b770c279600c`
- `structure-argument-boundary-input.hwpx`: `8ee4d26235fd1c841fba95a9d652973c664bc95d4c5e2f8fe78a3140129f5baa`
- `structure-argument-boundary-rhwp-input.hwp`: `f7b33f3c7bdf9b6047d5775b846bde93875e945eecf86992064af879d92b2c2d`
- `structure-empty-arguments-hancom-2020.hwp`: `9783a925dfbae87c1e5db899f9b405e589e01ffd7047594b1a46ea4261380d67`
- `structure-empty-arguments-hancom-2020.pdf`: `ab22ab17823189853b413ade1cbefbf54d067aebdc49d85509ec7a8f97e122e8`
- `structure-empty-arguments-input.hwpx`: `40f242f6d06c5c69df516f4f9f6f82e77b867837c6c3bd711085d153d2a34ac1`
- `structure-empty-arguments-rhwp-input.hwp`: `b204d519e2f3aa33c9b517d0b56bff9517940a730cec3033306a412a004e645d`

## 추가 명령·첨자 오류 경계의 Print 자료

- `native-keyword-precedence`: 저장 job `7062ed9a-1dd6-4a22-8626-b5c23dff378b`, Print job `e57ee230-482a-4ae4-90ab-319acefff042`, 전체 2쪽.
  - `native-keyword-precedence-hancom-2020.hwp`: `d6c1d3142b6b309707c7d8f96556e757df86c7dff841c175da87a0ba7cbd925a`.
  - `native-keyword-precedence-hancom-2020.pdf`: `3303a56f88c094776bc0345acad541d1ee7d5ca16e28e7bdfd77f551650d979c`.
  - `native-keyword-precedence-input.hwpx`: `3f274efc14f441d5a04d135fb082a97c42fd6b88e3814a4f2153b92296e29690`.
  - `native-keyword-precedence-rhwp-input.hwp`: `30d4a6a85a157bb7ec0bcf57e24bb48f543fc01dfe862226e85a0d529a94f1c8`.
- `script-failure-boundary`: 저장 job `ffa2df12-a677-4e0f-bbd7-2c8c0cc0d177`, Print job `10fe4c1a-d360-4d88-8368-5cfcafce9d2c`, 전체 1쪽.
  - `script-failure-boundary-hancom-2020.hwp`: `e01078beebe463b7c8bf7905f30d59403de5c40d2dac7a792dc3728504e0f7d8`.
  - `script-failure-boundary-hancom-2020.pdf`: `87abb0f9ea45ca63a536fe0107f7375584b20776f62ebdddedb3672665eda020`.
  - `script-failure-boundary-input.hwpx`: `9b698d7561a7d311474f4f9ff99cc20d15b062a9812cdf3f2f96cb22b5343d88`.
  - `script-failure-boundary-rhwp-input.hwp`: `5ec33a179e3424bcfd82fc9e0070d4f2c3809f8ba8979e8c04653d07a7118174`.

두 자료도 Hancom engine `2020`, build `11.0.0.9136`, PrintMethod `0`의
`hancom2020_pdf_driver_one_up`으로 출력했다. 모든 수식 script의 저장 전후 보존을 확인했다.
빈 첨자와 명령 우선순위의 진단 자료이며 아직 시각 통과 fixture로 등록하지 않는다.

## 최장 명령과 백슬래시 대조군

`native-keyword-longest`의 24개 script도 일반 저장 전후 보존됐고, 한컴 2020의
PrintMethod 0으로 전체 1쪽을 출력했다. OVERLINE과 OVER의 우선순위, UNDER의
별칭, 백슬래시 뒤 명령의 문서 해석을 대조한다. 시각 통과 fixture가 아닌 진단 자료다.

- hwp job: `2f4dc6ef-b3e1-4555-8260-c4f4b341657e`.
- pdf job: `a6486a2b-5c7b-401b-9e3e-f961b379455d`.
- `native-keyword-longest-hancom-2020.hwp`: `aacd2a219bc256e33c475b4b22ae4cedbde2ef4274f4fd46aa2bdc7de57c4157`.
- `native-keyword-longest-hancom-2020.pdf`: `bb6e10cc15a8fe1f2cfd00d82b5ee3bba113eef79f85d53011d8797bd075e612`.
- `native-keyword-longest-input.hwpx`: `ce0a80dd9b13487b34644de2f318177bb9f5f70377140c5410167703ef747c02`.
- `native-keyword-longest-rhwp-input.hwp`: `34a3dd113b024dd8d37da9582c9f1bfd5d1f00fc30e433859b2d603e0a43920e`.

## 백슬래시 문자 글꼴의 전환 범위와 기호 대소문자

`native-command-escape-scope`의 30개 script는 저장 전후 같고 전체 1쪽이다.
한컴 2020 build 11.0.0.9136, PrintMethod 0으로 출력했다.
- hwp job: `251306ed-2409-4da8-bdfa-f9c53d58c427`.
- pdf job: `67b79bbd-3ee4-4a31-a603-33c3a151e9e0`.
- `native-command-escape-scope-hancom-2020.hwp`: `2b0a84ab560db949879f225979f77570b8b27820c6d5eac80eff6bfbb422d819`.
- `native-command-escape-scope-hancom-2020.pdf`: `910ff8646770416e2d93dfb28d6c2ac9d4b8e5babaef6abd52c51c8b11efaa32`.
- `native-command-escape-scope-input.hwpx`: `afb42d4458d1526f64e09f5ed393be1107316251998c033863dd1208e3c51263`.
- `native-command-escape-scope-rhwp-input.hwp`: `206605c80669142b3919389bf71013aa084a1ef29ac46fe0ac9575ff35d16b8b`.

백슬래시 뒤 일반 문자는 ASCII 글리프로 출력되고, 중괄호가 끝나도 유지된다.
명시적 `it` 전환 뒤 문자는 다시 변수 PUA 글리프로 출력된다. `LINE`/`line`/`Line`은
세로 기호이며, 대문자 `LEFTARROW`와 소문자 `leftarrow`는 각각 이중·단일 화살표다.
escaped UNDERLINE은 인자 첫 여는 중괄호를 보여 주고, 내부 그룹과 닫힘은 구분한다.
원시 PDF trace와 명령·해시는 ignored Stage 334 output에 보존한다.

## Source 335의 직접 검토 이미지

code `d5006b134`, 실제 빌드 source `339c2f25100cead6537fc4290ea902d9159b7b7e`
(차이는 문서 정정)의 Native/fresh WASM은 최장 명령 99.40591%, 문자 전환
99.34022%다. 새 Native review를 직접 판독했으며 다음 파일로 보존했다.
이는 해당 진단의 결과이고 전체 PR 승인이나 최신 후속 head의 증거를 대신하지 않는다.

- `escape-30-source335-native-review.png`: `f584c337d8260630ff2cbc8fe79a00034dbc8fe3ad9790edb951eef60d0c27d7`.
- `longest-24-source335-native-review.png`: `f95f303ea56b515f56e87c46d0d92ceb054282d057e011541e7208394deb5f38`.

## Source 348 명령 문자 범위의 전쪽 재확인

보정 코드 `7c30f0a6a6fef1c7732078b8faafc2f867d5e166`에서 같은 한컴 저장본·Print PDF를 Native/fresh WASM으로 다시 산출했다.

`native-keyword-longest` 1쪽은 양쪽 최저 99.40591%, `native-command-escape-scope` 1쪽은 양쪽 최저 99.34022%다. 전체 쪽수 일치와 누락 없음도 확인했다.

두 문서의 실제 Native/fresh WASM 페이지 픽셀은 같았다. 같은 배율로 escaped 명령의 정체 범위, `it` 복귀, 여는 중괄호, 단일·이중 화살표와 세로 기호 및 앞뒤 문자를 확대 판독했다.

정식 새 이름의 검사 원본은 `tests/cases/issue_7642_native_command_literal_relations.rs`다. 소유·누락·중복·내용 순서와 문자/기호의 의미를 검사하며 절대 화면 px와 SVG hash를 고정하지 않는다.

같은 검사 3건은 수정 전 `3a8bd6aac3b995d994be3c0c3bbc248157d2faf7`에서 의도한 문자 스타일·명령어 잔류로 FAIL, 보정 코드에서 PASS했다. 최장 장식 명령 자체는 수정 전 코드에도 이미 보정돼 있고, 해당 검사의 escaped 문자 assertion이 뒤의 스타일 결함을 검출한다.

전쪽 TSV·로그·수정 전후 실행 증거는 ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage352-*`, `stage353-*`, `stage358-*`, `stage359-*`, `stage360-*`, `stage364-*`에 보존한다.

앞 절의 구조·catalog 미달을 이 두 문서의 통과로 면제하지 않는다.

| 대표 이미지 | SHA-256 |
| --- | --- |
| [escape-30-source348-native-review.png](escape-30-source348-native-review.png) | `dc03a51965d068cef0fb2c6e4c1e2ef825ac544b057ef912810c1d505bf85581` |
| [escape-30-source348-wasm-review.png](escape-30-source348-wasm-review.png) | `ae76490e170758a1e7e8f617762a3019fdc9e9be434278ea8cb040ac2d09414b` |
| [longest-24-source348-native-review.png](longest-24-source348-native-review.png) | `05074348e55045f5300a4d5370ddd3c759c1952dd469f8acf0c746701ea30a08` |
| [longest-24-source348-wasm-review.png](longest-24-source348-wasm-review.png) | `19fe96dc130a8abf32835421797d1cced34b32cad3f4f377328186df290aa544` |
