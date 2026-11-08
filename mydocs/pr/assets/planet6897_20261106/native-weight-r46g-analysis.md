# Native 굵기 기능 보존 분석

## 관측한 차이

같은 `pr8642/report-838-original.hwp`와 기존 독립 한컴 Print PDF의 24쪽을 비교했다.
원 PR의 각주·표·캡션 소유 검토 중 제목과 캡션의 굵기가 이전 Native 출력보다 가늘어진 것을
확인했다. raw 99.94%만으로 직접 판독을 대신하지 않는다. 글꼴 프로그램을 배포물에 추가하거나
기준 PDF를 다시 인쇄해 이 차이를 가리지 않는다.

- 이전 Native `0261687265f3a10a64e51809d006b2305a36d950`과 현재 Native
  `65195e9730f35bdbd7ea0c9a393e07e66125202e`에 같은 글꼴 환경·경로·compat 2022·96 DPI를 공급했다.
- 이전/현재 글꼴 환경을 교차 적용해도 굵기 차이는 실행 파일을 따라간다. 새 base
  `7f76cee60e38fceb63feacaff706cf984b19c427`에서도 같은 Native 제목 차이를 재현했다.
  따라서 글꼴 환경 JSON 교체만으로 해결됐다고 판정하지 않는다.
- 원본의 제목·표 캡션·표 머리말은 공개 레이어 getter에서 모두 명시 `bold=true`다.
  family 이름만으로 합성 굵기 여부를 추측하지 않는다.
- 독립 PDF의 같은 제목과 캡션에는 fill 및 stroke 텍스트가 함께 있다. 원본 문서의 bold와
  실제 프로그램의 weight, OS가 덧붙인 합성 weight, 최종 paint의 획을 구분해 추적한다.
- 이전/현재/Print 확대 영상의 검은 잉크 합계는 굵기 차이를 찾는 진단값이다. 정식 일치율이나
  새로운 통과 임계값으로 사용하지 않는다.

## 보정 범위와 검증

PR #8754의 실제 프로그램·cmap·collection face 선택과 20조건/562쪽 보호는 유지한다.
그 변경 전체를 되돌리지 않으며, Native의 실제 선택 face에서 최종 굵기 paint까지 공통 경로를
확인한 뒤 필요한 부분을 보정한다. WASM의 완료된 제품 source와 실제 바이트가 영향을 받지 않으면
그 증적을 재사용하고, Native는 새 실행 파일로 영향 영역과 필수 게이트를 다시 검증한다.

## 확인한 원인과 보정

호스트의 두 Fontconfig pattern 규칙은 `한컴바탕` 요청에 HCR 대체를 넣고 실제
Haansoft face를 다시 우선한다. OS의 한 번의 face 매칭과 스타일 목록 열거는 같은 결과를
보장하지 않는다. 실제 `family_order=0` 후보에 HCR Batang과 Haansoft Batang이 함께
들어왔으며, 새 TTF 우선 처리에서 다른 계열의 Bold TTF가 원 계열을 대신했다.
물리 프로그램 재개방 자체나 명시 `bold`가 사라진 문제가 아니다.

- `matching_system_programs`는 같은 요청 이름의 스타일 목록을 실제 family 소유로
  간주했다. 목록의 각 프로그램이 SFNT family를 선언했는지 확인하도록 보정한다.
- `match_system_family_style`의 같은 스타일 TTF 우선 선택도 실제 family 소유를
  확인한다. 같은 계열 안의 TTF → TTC → OTF 정책과 물리 프로그램의 cmap·paint 일치는 유지한다.
- 공개 GUST CFF와 OFL TTF 두 개만 격리 공급하고 같은 pattern 규칙을 공개 이름으로
  구성한 대조군을 만들었다. 원본 글꼴의 name 테이블이 독립 기대값이다.
  수정 전 요청 `Latin Modern Math` 대신 `Rhwp Family Contract`가 선택되어 exit 101로 실패했다.
- 새 정식 검사는 `native_system_font_family_ownership.rs`이며 새 프로세스의 전용
  Fontconfig에서만 두 프로그램을 노출한다. 보통/굵게, 원 CFF 계열/대조 TTF 계열의
  양방향 소유를 검사한다. 설치된 한컴·OTF나 다른 검사에서 초기화한 FontMgr에 의존하지 않는다.
  합성 이름 선택 계약이며 실문서의 px·쪽수·이미지 기대값을 만들지 않는다.

[수정 전 정식 검사 실패](native-family-ownership-r46g/before-test.txt) ·
[실패 실행 출처](native-family-ownership-r46g/before-provenance.json) ·
[사전 진단](native-family-ownership-r46g/before-probe.txt) ·
[실제 후보 묶음](native-family-ownership-r46g/actual-chain-before.json).

보정 직후에는 재검증 완료 전이었다. 아래 R47의 실제 선행·정식 검사 결과와 최종 전쪽 대조를 구분한다.


## R47 실제 출력 선행 확인

제품 source `0275f9c3bb69b216c96146e98506271eead9a601`의 새 Native 실행 파일에서
같은 원본·Print·글꼴 환경으로 PDF 24쪽 raw 100.00%, 49쪽 99.99%를 확인했다.
24쪽의 제목·캡션은 이전 정상 R43과 같은 검은 잉크 합계를 복원했다. 잉크 합계는 진단값이며
통과 임계값으로 사용하지 않았다. 실제 review·overlay에서 제목의 굵기, 본문·각주 62~66,
표 캡션·4개 열·마지막 행·쪽 번호 소유를 함께 판독했다.

[선택 쪽 TSV](native-family-ownership-r46g/after-selected-native.tsv) ·
[관측·출처](native-family-ownership-r46g/after-selected-provenance.json) ·
[review](native-family-ownership-r46g/review-after-p024.png) ·
[overlay](native-family-ownership-r46g/overlay-after-p024.png).

이 절은 선택 쪽의 실제 선행 확인이다. 새 회귀·전체 필수 검사와 전쪽 Native/fresh WASM
검증 완료는 아직 선언하지 않는다.

## R47 정식 이름 소유 회귀 검증

같은 제품 source에서 최소 글꼴 wrapper로 새 회귀를 실행해 **1 passed / 0 failed**로 통과했다. 선택 밖 308건은 전체 회귀의 skipped로 합산하지 않는다. Native feature의 라이브러리·해당 suite Clippy도 `-D warnings`로 exit 0을 확인했다. 수정 전 실패와 같은 assertion·입력을 유지한 검사이며 원 계열의 프로그램 소유를 확인한다.

[수정 후 정식 검사](native-family-ownership-r46g/after-test.txt) · [source·파일·로그 SHA](native-family-ownership-r46g/after-test-provenance.json). 전체 Rust·fresh WASM·전쪽 시각 검증은 별도 진행 중이다.

## R47 필수 기본 검증 완료

같은 제품 source에서 전체 Rust **13,069 passed / 0 failed / 45 skipped**(exit 0), Native lib 4개 crate 합계 **4,035 passed / 0 failed / 12 ignored**, Native 집중 **86 passed**를 확인했다. 같은 최소 글꼴 환경의 format·Native/WASM32/workspace lint·workspace build·정책 검사·문서 예제까지 후보 실행 17단계가 모두 exit 0이다. 실제 source와 출력 경로를 바꾸지 않았다.

최적화 WASM은 7분 55초에 성공했다. 공식 Native 편람 HWPX gate는 전 383쪽에서 기준 하락 0건이며, 382쪽 100.00%·독립 PDF 130쪽 raw 99.96%다. 알려진 WMF 차이를 마스킹하거나 100%로 고치지 않았다. 나머지 전쪽 Native/fresh WASM·최신 base·원 PR 기준 대조와 개별 최종 판독은 계속 진행한다.

## R48 전쪽 대조에서 확인한 추가 하락과 보정 계획

R47의 완료된 실제 결과를 이전 R43의 같은 원본·Print·compat·DPI와 대조해 report-628의
16쪽 96.88→94.21%, 18쪽 99.72→97.38%, 120쪽 96.55→94.81%,
122쪽 99.58→97.22% 하락을 확인했다. 중첩 표·보육 문서의 작은 하락도 함께 추적한다.
이는 조사 결과이며 준비 완료 판정이 아니다. 새 base 대비 검사와 별개로 이전 개선의 보존을 확인한다.

수식의 공통 원인 가설은 `legacy_math_face`를 원본 HFT 표시와 HWP5 선택 문자열 생략의
두 계약에서 함께 사용하는데 PR #8754의 원본 그룹·첨자 여백까지 생략 계약에 적용하는 것이다.
생략 계약의 독립 10/20pt Print와 기존 `implicit_legacy`의 공통 줄 프레임을 보존해야 한다.
`original_group_unit → original_group_frame/original_script_base → layout_node/layout_script_base`
호출 경로를 대조했다. 명시적인 원본 HFT 표시와 생략된 선택 문자열을 구분해 보정하고,
같은 수식 4쪽과 PR #8754의 원본 그룹·첨자 대조군을 함께 측정한다. 문서 ID·좌표 보정으로
수치를 맞추지 않는다. 코드 보정 전 관측 원장은 ignored output의
`r47-last-validated-live-diagnostic.json`이다. 독립 출력 대조가 가설을 뒷받침하지 않으면 채택하지 않는다.

### 보육 문서의 가운데점 전진 하락

독립 Print 13쪽의 `일회적·단편적인`에서 가운데점은 H2hdrM 15pt이며,
96 DPI로 환산한 점 원점175.52와 뒤 글자195.52의 차이는20px다. 원본 15pt의 1em이다.
R43은 같은 원본 TTF의 검증된 표를 신뢰해1500 HU를 사용했으나 R47은448 HU로 줄였다.
현재 공개 FontDecisionTrace는 `HY헤드라인M`이 물리 프로그램의 논리 family `HYHeadLine M`으로
표시 인증된 뒤 `metricNarrowPunctuationOverlay`로 잘못 내려가는 것을 보여 준다.
원본 쿼리와 같은 선택 프로그램의 신원 인증을 다른 글꼴로의 대체와 혼동했다.

`selected_source_programs → 검증된 원본 TTF의 metric 신뢰 → 공통 문자 폭 결정 → 뒤 run의 원점`
경로를 보정한다. 같은 원본 쿼리의 실제 선택임이 확인된 경우만 기존 검증된 메트릭 신뢰를 보존하고,
임의 대체 프로그램에 신뢰를 새로 부여하지 않는다. HFT와 자연 프로그램 선택의 기존 경계도 유지한다.
해당 관측은 ignored output의 `r48-childcare-dot-independent-print.json`과
`r48-childcare-dot-trace.json`이다. 이 절은 원인·보정 계획이며 아직 재검증 통과가 아니다.

### R48의 실제 선행 보정 결과와 남은 조건

공용 target·최소 글꼴·8 jobs Native 빌드는 exit0, 23분55초로 완료했다.
같은 원본·기존 Print·96 DPI·compat2022에서 그룹 계약 분리 후 16쪽96.27%,
18쪽98.79%, 120쪽96.67%, 122쪽98.84%를 실제 산출했다. 보정 전 R47의
94.21/97.38/94.81/97.22%보다 네 쪽 모두 상승했다. 이 값은 선택4쪽의 선행 확인이며
전쪽/fresh WASM 완료로 표현하지 않는다. 이전 R43 기준에는 일부 하락이 남아 있다.

[실제 선택 쪽 TSV](implicit-frame-r48/native-selected.tsv) ·
[관측·producer·변경 파일 SHA](implicit-frame-r48/native-selected-provenance.json) ·
[진단 소스 차이](implicit-frame-r48/source.json) · [빌드 로그](implicit-frame-r48/native-build.txt).

이전 승인된 글꼴 대체 환경을 고정한 추가 대조에서도 중첩 표와 일부 본문의 하락이 남았다.
따라서 자동 공급 환경 차이만으로 설명하지 않으며, 프로그램 재개방·스타일/원본 메트릭의
실제 소비 경로를 계속 추적한다. 기준 PDF·raw 알고리즘·기준값을 바꾸지 않는다.
