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

이 단계에서 사용한 것은 R43의 자동 생성 글꼴 환경이었다. 이후 R50에서 확인한 실제 승인
Print 대체 설정과는 다르다. 이 대조만으로 글꼴 환경을 원인에서 제외할 수 없다.
기준 PDF·raw 알고리즘·기준값은 바꾸지 않는다.


## R49 원 해결 커밋 대조와 두 선택 경로의 보정

원 PR의 실제 코드와 독립 증거를 다시 추적했다. 다음 두 수정은 아직 아래 집중 재검증을
진행하는 단계이며, 새 head의 전쪽 일치율 복구 완료를 뜻하지 않는다.

| 원 해결 | 보존할 계약 | 현재 경로의 차이와 보정 |
| --- | --- | --- |
| PR #8389·PR #8397의 실제 후보 순서, PR #8754 통합 `7f76cee60e38fceb63feacaff706cf984b19c427` | 같은 원본 계열의 프로그램·실제 스타일, TTF → TTC → OTF와 실제 cmap/paint 일치 | 후보 열거는 `unemboldened_typeface`로 물리 Regular를 확인하지만 마지막 정렬은 OS 가상 Bold를 실제 Bold로 취급했다. 마지막 정렬도 동일한 물리 스타일을 소비하게 했다. 실제 Bold·가변 wght 판정과 family 소유 검사는 유지한다. |
| 이슈 #7092의 원 해결 `01e2c9905a8a9ae5767d20ead9e264229c327e38`, 자동 프로그램 공급 PR #8565 `5551091085` | 실제 주 TTF·문서 선언 대체 TTF와 HFT의 서로 다른 문장부호 폭 | 원본 `HY헤드라인M`을 같은 프로그램의 family `HYHeadLine M`으로 인증한 뒤 기존 메트릭 신뢰를 잃었다. 원본 TTF·기존 신뢰·원본 조회 쿼리·실제 측정이 소비하는 원본 표를 모두 확인한 별칭만 신뢰를 보존한다. 별칭에 독립 메트릭 키가 없어도 원본 표를 그대로 쓰는 경우를 포함하며, 다른 메트릭 표를 선택하면 제외한다. 다른 프로그램·HFT에 신뢰를 새로 부여하지 않는다. |

`matching_system_programs → chain 최종 정렬 → select_typeface_for_character → candidate_typeface → paint`
경로에서 실제 스타일 소유를 공유한다. `원본 슬롯 신뢰 → 환경 프로그램 인증 → metric_trusted →
공통 문자 전진 → 후속 run 원점` 경로에서 같은 표의 별칭이 신뢰를 덮어쓰지 않게 한다.

프로그램 재개방 자체를 원인으로 보았던 가설은 배제했다. 정식 실행 파일을 바꾸지 않은
별도 진단 사본에서 기존 clone 경로만 실행해도 중첩 표 1쪽은 95.68127%로 같았다.
따라서 이슈 #7640의 실제 프로그램/cmap 일치 수정은 되돌리지 않았다. 이 사본은 원인 분리용이며
정식 후보·통과 증거로 제출하지 않는다. 원장과 로그는 ignored output의
`r49-native-program-diagnostic-receipt.json` 및 `logs/r49-original-program-native-diagnostic.log`다.

검증은 기존 독립 Print·96 DPI·기존 승인 대체 설정을 유지한 중첩 표의 하락 9쪽,
보육 13·24·25쪽, 수식 보고서의 하락 및 보정 대조 15쪽, NARS2021 1쪽부터 수행한다. 전쪽 검증과 PR #8754의
실제 프로그램 일치·원본 수식 대조군 및 새 base의 clip 경계 보존은 별도로 확인한다.
새 관계형 검사는 설치 글꼴 없이 공개 `DocumentCore`·`FontEnvironment`를 통해 같은 별칭의
메트릭 보존과 다른 프로그램·HFT 반례를 확인한다. 기대값·독립 PDF·측정 알고리즘은 바꾸지 않는다.

## R50 실제 실행 결과와 가설 정정

R49 제품 커밋은 `40c90ca187201527f5b7776f86f0fe6e493cef52`다. 최소 공개 글꼴 환경의
집중 검사 29건은 모두 통과했다. 같은 TTF 별칭의 메트릭 신뢰 보정 후 보육 문서 13쪽의
실제 Native raw는 99.21%에서 100.00%로 회복됐다. 그러나 물리 스타일의 마지막 정렬을
바꾼 실험은 중첩 표의 하락을 해결하지 못했고 일부 다른 쪽의 raw를 낮췄다. 따라서 위 표의
정렬 변경은 원인 해결로 채택하지 않고 종전 정렬로 복원한다. 그와 별개로 입증한 family
소유 검사와 같은 TTF 메트릭 신뢰 보정은 유지한다.

### 독립 Print의 글꼴과 승인 설정 복원

중첩 표 PDF의 제목·표 머리글은 실제 `Gulim`인데, 자동 공급은 설치된 `휴먼명조` CFF를
선택했다. 원 PR 검증의 정식 자산
`mydocs/pr/assets/planet6897_integration_final/pr8389_8397/hwp3-2020-font-environment.json`과
작업지시자가 승인한 `휴먼명조 → Gulim` 계약을 재확인했다. R43 자동 환경을 이 승인 설정으로
부른 것은 잘못이므로 정정한다. 전체 시스템 글꼴 공급은 유지하고, 해당 원본의 자동 환경에
승인된 대체와 `Gulim → Haansoft Batang` 글리프 fallback만 명시한다. 같은 원본의 scoped
프로그램 선택이 이 대체를 다시 덮어쓰지 않는지도 확인했다. 다른 PDF에는 일괄 적용하지 않는다.

독립 Print·입력·96 DPI·raw 알고리즘을 유지한 `40c90ca187` Native 실제 결과는 다음과 같다.

| 중첩 표 쪽 | R43 기준 | 승인 Print 대체 복원 후 |
| --- | ---: | ---: |
| 1 | 96.58% | 98.20% |
| 6 | 98.11% | 100.00% |
| 7 | 97.74% | 99.10% |
| 16 | 96.00% | 97.72% |

이는 선택 4쪽의 실제 결과이며 전17쪽·fresh WASM 완료를 뜻하지 않는다. 실행 원장은
ignored output의 `r50-print-gulim-nested-selected.json`, 환경은
`r50-nested-print-gulim-environment.json`이다. 최종 제출에는 전쪽 증거를 별도로 고정한다.

### 원본 CHAR와 선택 문자열 생략 수식의 전진 소유

PR #8754 통합 `7f76cee60e38fceb63feacaff706cf984b19c427`의 전후 Git 소스를 대조했다.
원본 CHAR의 장치 격자 반올림과 이탤릭 전진표는 원본 CHAR에 필요한 계약이다. 그러나
같은 표시 face를 사용하는 선택 문자열 생략 HWP5 경로까지 이 계약을 적용하면서, 기존
연속 전진과 정체 전진 위의 별도 shear를 바꿨다. 이전 코드는
`legacy_glyphs::measure`에서 `units * fs / 1000`을 누적하고 이탤릭도 같은 정체 전진을 쓴다.

`implicit_legacy_font → source_glyph_runs → legacy_glyphs::measure → LayoutBox.width →
후속 수식 원점`에서 출처를 전달하도록 `AdvanceFrame`을 구분한다. 원본 CHAR의 반올림·
이탤릭 전진은 유지하고, 선택 문자열 생략에만 이전 연속 전진·정체 폭과 shear를 복원한다.
원 CHAR에서 추가 수용한 소수점·가운데점도 생략 경로의 기존 fallback을 바꾸지 않는다.
분수·제곱근은 생략 표시 지원 AST가 아니므로 그 경로에 조건을 더하는 가설은 채택하지 않았다.

새 합성 관계 검사는 실제 `DocumentCore` 삽입·배치 경로에서 크기를 두 배로 바꾼 전진과
정체/기울임의 폭 관계를 검사한다. 설치 글꼴·실물 좌표 핀을 쓰지 않는다. 수정 전 실제
`40c90ca187` 라이브러리에 연결한 실행은 `a: 9.96 → 20.92`로 실패했다(종료101).
수정 후 집중 검사와 수식 보고서의 독립 Print 대조는 계속 수행하며, 아직 통과로 기록하지 않는다.
수정 전 원장은 ignored output의 `r50-implicit-advance-before.json`과
`logs/r50-implicit-advance-before.log`다.

## R51 원 PR의 비적용 범위와 남은 문장부호 전진

[원 PR #8605](https://github.com/edwardkim/rhwp/pull/8605)의 실제 head
`5d37286267136c9978d7ac2f804913730ab6becf`와 코드 `4f8570309e`를 Git에서 다시 확인했다.
이 변경은 WASM 합성 굵기 전용이며 Native를 바꾸지 않는다. 기여자의 공개 바탕 대체 측정과
현재 설치 프로그램을 공급한 측정을 혼동하지 않는다. 이후 통합한
[PR #8754](https://github.com/edwardkim/rhwp/pull/8754)의 프로그램 선택과 현재 보정도 함께 대조했다.

R50의 실제 집중 회귀는 최소 글꼴 환경에서 11/11 통과했다. 생략 수식 보정으로 보고서
16·18·120·122쪽은 각각 기준과 같은 96.88·99.72·96.55·99.58%로 회복됐다. 중첩 표는
승인 Print 대체를 적용한 전17쪽이 90% 이상이며 최저95.69%다. 알려진 Native 하락27행 중
0.1%p 초과 하락은 보육13·24·25쪽만 남았다. 따라서 fresh WASM 빌드는 아직 시작하지 않았다.

보육24쪽은 이전 출력과 현재 PNG의 실제 차이가 법조문 첫 줄에 한정된다. 독립 PDF의
`H2gtrE`에서 `직 → · → 파` 원점은 108.72 → 118.44 → 128.04pt다. 가운데점은 약9.60pt를
점유하며 전각 계약이다. 이전 Native는 같은 run을 약12.9 CSSpx로 측정했지만 현재는3.8px로
줄여 뒤 문장을 당겼다. 글자 크기나 좌표를 맞추는 임의 보정이 아니라 가운데점 전진의 소유 문제다.

실제 자동 환경은 원본 `HY견고딕`을 같은 조회 쿼리의 TTF family `HYgtrE`로 인증한다.
원래 측정 키 `HYGothic-Extra`와 별칭 키 `HYgtrE`는 em·문자 전진이 같지만 서로 다른
정적 객체다. BMP 전체 전진 대조에서 차이는0개였다. R49 helper가 포인터 동일성만 확인해
이 동일한 전진을 다른 표로 판정하고 기존 신뢰를 잃어 좁은 문장부호 overlay로 떨어졌다.
이는 글꼴 전체 교체나 합성 굵기 차이로 해결할 문제가 아니다. 보육 문서에 중첩 표의 Gulim
설정을 확장한 진단도 해결하지 못했으므로 최종 설정으로 채택하지 않는다.

원본 TTF·기존 신뢰·같은 원본 조회·원본 CSS 선두 소유 조건은 유지한다. 표의 같은 객체는
즉시 수용하고, 객체가 다를 때는 스타일·em·문자 범위와 모든 전진·한글 조합 매핑 및 폭이
동일한지 비교한다. 이름만 다른 같은 전진의 표를 다른 프로그램의 폭으로 오인하지 않는다.
Arial처럼 전진이 다른 표와 HFT·다른 조회 경계는 계속 거절한다. 설치 글꼴 없이 실제
`DocumentCore` 경로에서 `HY견고딕/HYgtrE`를 기존 별칭 관계 검사에 추가했고 수정 전 실제
R50 라이브러리에서 실패했다. 이 실패를 허용하거나 기대 전진을 줄이지 않는다.

수정 전 실제 로그는 ignored output의 `logs/r51-original-truetype-alias-before.log`,
원장은 `r51-alias-before.json`이다. `r51-source-metric-identity-unit-provenance.json`의
작은 대조는 함수의 구조 동등성만 확인하며 실제 집중 회귀·Native/fresh WASM·전쪽 판독을
대체하지 않는다. 보정 후 같은 원본·Print로 세 쪽과 대조군을 다시 확인한다.
