# #8153: 셀 배경의 물리 Print clip과 본문 clip 소유

## 원본과 독립 기대값

원본 두 편람과 기존 독립 2024 Print를 유지한다. 291·301쪽의 표 바탕 image351과
Q 셀 image352는 HWPX BorderFill 717/718의 TOTAL 그림 채움이다. 셀의 본문
소유/잘림과 그림 채움의 원본 창을 구분하며 HWP5 표·셀/BorderFill 속성을 유지한다.
PDF의 이미지를 원본 대용으로 넣거나 원본 clip·그림을 바꾸지 않는다.

source b3ad의 실제 두 그림 자원은 독립 PDF xref2137/2138과 각각 **RGB 전픽셀이 같다**.
이미지 paint 원점 y=200.7804605px는 독립 PDF 200.780009px와 맞고, 원래 물리
bitmap clip y=200.940445px도 유지된다. 그런데 Q 셀에는 상위 논리 TableCell
clip y=201.066667px가 더 있어 그림 배경의 첫 장치 행을 잘라 내는 경계가 있다.
이 값들은 원인 관측이며 제품에 넣을 위치 상수가 아니다.

## 실제 원인 분리와 공통 소비

두 원본의 291·301쪽, 네 조건의 수정 전 재생 RGB가 실제 Native source b3ad와
전픽셀 같음을 먼저 확인했다. 직접 셀 배경의 PrintBitmapFill 자식만 기존 내용
clip 밖에 놓고, 그림 자체의 물리 clip·입력 바이트·프레임·본문 clip은 그대로 두었다.
그 재생에서 네 조건 모두 미대응 20 → 0픽셀, raw 100%다. 제품 수정이나
fresh WASM 통과의 증거로 확대하지 않는다.

실제 소비는 `render_cell_background의 원본 배경 역할 → TableFillOrigin →
PagePaintContext.bitmap_fill_frame의 그림/물리 clip → build_node(TableCell)의
논리 내용 clip → Native/Canvas replay`다. 같은 프레임 helper를 거쳐도 뒤의
다른 clip이 덮어쓰면 결과 공유가 끝난 것이 아니므로 이 마지막 경계를 수정한다.
모든 셀 clip을 제거하거나 Native·Canvas에서 따로 좌표를 고치지 않는다.

## 기존 계약과 다음 검증

셀 wrapper 원형은 `6f973d6076e`의 layered paint tree이며, 명시적 ClipKind는
`55e406d723b`의 Task #1197 paint order 변경이다. 물리 PrintBitmapFill clip의
현행 도입은 `2d072afca0`이다. 실제 변경 코드와 관련 단계/검토 기록을 대조하고,
기존 내용 overflow·중첩 셀·분할 쪽 컷의 보호 목적을 확인한 뒤 구현한다.
특히 분할 viewport는 본문 내용 상자와 다른 물리 컷이므로 같은 배경 우회로
가정하지 않는다. 아직 정식 제품 코드는 변경하지 않았다.

공통 paint builder에서 원본 배경 역할과 자체 물리 clip을 확인해 두 소유를 분리하고,
내용 clip·source ID·내용 순서·누락/중복·기존 paint order는 유지한다. 작은 구조
계약과 실제 네 쪽의 before FAIL/after PASS, 주변 정상 배경과 분할/중첩 반례를
먼저 검증한다. 실제 Native/fresh WASM 출력과 전쪽 TSV·확대 판독을 마친 후
정식 `tests/cases/issue_8153_*.rs` 회귀에 포함한다. 파일별 제출 TSV와 대표 PNG,
최종 표준 최적화 WASM·전체 회귀/lint/Skia/Studio와 PR 준비는 완료되지 않았다.

ignored 증거는 `remaining-raw100-diagnostic/continuous-text-shadow/
p289-301-source-contract/`의 `actual-top-image-paint.log`,
`p291-actual-independent-image-comparison.json`, `p291-301-independent-top-images.json`,
`cell-background-replay/comparison.json`, `cell-background-replay-hwp/comparison.json`이다.
원본·PDF·폰트·측정 기준 변경, upstream 동기화와 원격 변경은 하지 않았다.

## 공통 구현 후보

`cell_clip::build`는 원본 `table_background_paint` 역할과 직접 자식의
`PrintBitmapFill`을 함께 확인한다. 일반 셀에서만 그 배경을 자체 clip으로 그리고,
그 앞뒤의 연속 본문 묶음에는 기존 셀 clip을 유지한다. 따라서 중간에 배경이 있어도
paint 순서를 재정렬하지 않으며, 일반 그림은 배경으로 승격하지 않는다. 셀 소유 group의
source ID·cache hint·layer를 보존하고 상위 셀·본문 clip은 건드리지 않는다.

`page_fragment`는 실제 쪽 컷을 소유하므로 배경에도 기존 셀 clip을 유지한다.
이 분기는 문서·좌표 조건이 아니라 기존 #5862의 명시적 원인 구분을 소비한다.
원래 #1197 Stage 4의 layer 상속과 그리기 순서를 확인했으며 Screen·자체 물리 clip이
없는 배경도 종전 구조로 남긴다. 구조 계약은 배경/본문의 교차 순서와 일반 그림,
분할 컷, 중첩 상위 셀의 clip을 확인한다. 현재 실행 전 후보이며 시각 통과는 미판정이다.

## 실제 Native 후보 실행

제품 source `259ecf8303d97d17a8afd5598fdd504340abdeb1`을 고정했다.
`target/pr-review`, jobs 8의 Native CLI·library 빌드는 481.57초·종료 코드 0이며,
다음 Cargo 빌드 전에 producer와 SHA-256 receipt를 ignored output에 보존했다.

실제 원본 HWP/HWPX 291·301쪽 네 조건은 각각 미대응 **20 → 0픽셀**,
raw **100%**다. 원본·독립 PDF·DPI·글꼴·그림 바이트·frame·자체 물리 clip을
유지했다. 291쪽의 같은 좌표 확대와 291·301쪽 review를 직접 읽어, Q 아이콘
윗부분에 잘못 나타났던 회색 가로선이 사라지고 본문·다음 항목이 유지됨을 확인했다.
그림 자원 전체의 RGB 동일성을 raw 100%의 의미로 주장하지 않는다.

주변·정상 대조군을 넓힌 Native 80조건은 70조건 raw 100%·미대응 0이며,
기준 source b3ad의 같은 조건보다 미대응 픽셀이 증가한 조건은 0개다. 전체
766조건은 별도 실행 중이며 이 집중 검증으로 전쪽 완료를 대신하지 않는다.

공개 API 원본 진단은 실제 PNG와 같은 글꼴 환경·exporter 준비를 적용했다.
수정 전 source b3ad의 네 조건 모두 본문 clip 중복 적용 assertion에서 실패하고,
제품 후보의 네 조건은 통과했다. 초기 진단에는 글꼴 준비 누락과 원본 Q 배경 수를
하나로 가정한 오류가 있었으며, guard가 해당 실패를 거부했다. 그 기록은 유효한
before FAIL 증거에서 제외하고 원인과 원본 로그를 ignored output에 따로 남겼다.

원본 진단과 #5862 분할 clip, #6861/#5587 중첩 표의 양쪽 계약, 배경 변경,
비변환 그림 및 기존 편람/TAC 대조군 22개 harness의 개별 종료 코드는 모두 0이며
49 PASS·0 FAIL·0 ignored다. 최초 실행 드라이버의 관측 종료 코드는 143으로
돌아와 원인을 미확정으로 기록했고, 같은 동결 실행 파일의 정상 종료 재확인을
진행 중이다. 공통 clip 구조 검사와 기존 builder 단위 검사는 별도 Cargo로 실행 중이다.
현재 후보의 fresh WASM·전쪽 766·최종 필수 검증과 정식 원본 회귀 등록은 남아 있다.

실제 자료는 `cell-clip-native/{producer,native-boundary,native-focus}/`와
`cell-clip-native-validation.log`, `cell-clip-native-terminal-recheck.log`,
`cell-clip-unit.log`, `cell-clip-native-whole.log`다. 작업 경로는 앞 절의
`continuous-text-shadow/` 아래다. 이미 보존한 원본 PDF를 재출력하지 않았다.

동결 실행 파일 22개 harness를 별도 드라이버에서 다시 실행했다. 같은 source·
파일 SHA를 확인했고 **49 PASS·0 FAIL·0 ignored, 드라이버 종료 코드 0**이었다.
첫 드라이버의 143 원인은 확정하지 않으며 해당 관측도 보존한다. 재실행의 각 실행
파일·로그 SHA와 실제 종료 코드는 `cell-clip-native/contracts-terminal-recheck.json`에 있다.
공유 Cargo 산출물은 다시 컴파일하지 않고 기존 동결 실행 파일만 재사용했다.

## fresh WASM·전쪽 Native와 정식 원본 회귀

동일 제품 source 259의 새 DEV WASM은 source clean·jobs 8·codegen units 16,
표준 build mode로 228.52초·종료 코드 0이었다. 동결 package의 JS/WASM SHA를
확인한 뒤 같은 Native 글꼴 snapshot으로 실행했다. 두 원본 291·301쪽 네 조건은
**fresh DEV WASM에서도 모두 raw 100%·미대응 0픽셀**이다. Native/fresh WASM/
독립 Print의 동일 좌표 확대 4행과 WASM 전체 review를 직접 읽어, 잘못 노출됐던
회색 행이 없어지고 Q 윤곽·본문·다음 문항이 유지됨을 확인했다. 최종 표준 최적화
전쪽 WASM과 구분하며, 넓힌 fresh DEV WASM 83조건은 실행 중이다.

제품 후보의 Native 전쪽 766조건도 종료 코드 0으로 완료했다. HWPX는 367/383,
HWP는 364/383, 합계 **731/766조건**이 raw 100%·미대응 0이다. 최저 raw는
양쪽 99.46283%, 미대응 합계는 HWPX 973·HWP 989이며 기준 b3ad보다 증가한
미대응 픽셀은 없다. 의미 render tree 766개도 b3ad와 바이트 단위로 같다.
전쪽 100% 목표에는 아직 Native 35조건과 현재 source의 전쪽 WASM 검증이 남는다.

공통 clip 구조 3건과 기존 builder 14건은 실제 Cargo 실행에서 모두 통과했다.
새 원본 회귀 `tests/cases/issue_8153_cell_bitmap_clip_ownership.rs`는 공개 API의
원본 셀/그림 신원을 따라, 배경 paint의 누락·중복, 자체 물리 clip 유지,
같은 셀의 본문 clip 중복 적용 금지와 상위 본문 clip 유지를 검사한다. 시스템
글꼴 경로·프로그램 바이트를 요구하지 않으며 절대 px 좌표나 쪽수를 새로 고정하지 않는다.
같은 원본/동작의 기존 builder·#5862·#6861/#5587·편람 경계 검사도 확인했다.
그 보호 목적과 충돌하는 기존 assertion은 확인되지 않아 유지했다.

최종 등록 파일 자체를 이전 b3ad와 현재 259의 동결 라이브러리에 각각 컴파일했다.
이전은 의도한 clip 중복 assertion에서 **4 FAIL**, 현재는 **4 PASS**다.
HWP5 원본의 채움은 사양 표 31의 값 5인 FitToSize이고 HWPX는 TOTAL이다.
두 원본을 모두 TOTAL로 가정했던 초기 등록 진단은 제외하고, 원본별 정확한
속성을 검사하도록 고쳐 같은 파일을 재실행했다. 제품의 공통 수용 조건은 두 형식을
이미 구분해 소비하며 이 검증을 위해 구현이나 원본을 바꾸지 않았다.

suite는 정본 `--prepare`로 자동 수집했다. 최종 파일은 `regression_suite_009`에 4건이 들어가고
28 suites + 20 exceptions = 48/48 targets다. 파생 manifest/harness는 ignored이며,
Cargo registry·suite policy·CI 호출을 추가 변경하지 않았다. `--adopt-new`만 실행하면
정리되지 않은 과거 예외가 남던 관측은 정식 prepare로 해소했으며, 그 실패를 코드
회귀의 증거로 사용하지 않는다. 실제 Cargo suite 경로의 필터 4건 실행이 다음 검증이다.

실행 근거는 `cell-clip-wasm-dev/{build.json,wasm-boundary-result.json}`,
`cell-clip-native/{unit-results.json,native-whole/whole-result.json}`,
`p289-301-source-contract/formal-cell-bg-before-after-result.json`이다.
확대 PNG는 `cell-clip-native/native-boundary/native-fresh-wasm-print-four-boundaries-zoom.png`다.
최종 PR 제출 TSV·중요 전쪽 ±2px 회귀·표준 최적화 fresh WASM·전체 필수 검증은 미완료다.

실제 Cargo 자동 수집 검증은 source `1d75b62dd400e35dfbcea45f43df7666767e484a`의
`regression_suite_009`에서 **4 PASS·0 FAIL·0 ignored, 종료 코드 0**으로 완료했다.
컴파일·실행 합계는 669.09초이고 필터에서 제외된 기존 검사는 259건이다.
제품 src는 259와 같으며 이 source 차이는 원본 회귀 파일·검증 기록이다.
`cell-clip-native/formal-suite-result.json`에 정확한 명령·source·실제 summary를 보존했다.
전체 Rust suite 통과로 확대하지 않는다.

새 source 259의 fresh DEV WASM 83조건도 종료 코드 0으로 완료했다. 원본은 각
40조건 중 35조건, 독립 탭 대조군 3조건은 모두 raw 100%·미대응 0이며 합계
**73/83조건**이다. 이전 b3ad와 겹치는 75조건의 미대응 증가도 0이다.
추가 8조건은 두 원본의 130·141·144·255쪽이며 이전 집중 결과와 없는 비교를
만들지 않았다. 최종 표준 최적화 전쪽 검증은 별도로 남는다.

원본의 205·239쪽은 양쪽 형식 모두 raw 100%를 유지하면서 RGB도 개선됐다.
205쪽의 바뀐 13픽셀은 독립 Print에 대한 채널 절대 차이 합이 84→0,
239쪽의 바뀐 515픽셀은 19,599→0이다. 75조건의 실제 PNG를 비교한 뒤
변경 영역을 같은 좌표로 확대 판독했다. 오차 없는 raw 값만으로 이 RGB 차이를
무시하지 않았으며, 변경 픽셀이 모두 독립 출력과 정확히 같아짐을 확인했다.
관련 근거는 `cell-clip-wasm-dev/common75-comparison.json`,
`additional205-239-reference-comparison.json`과 HWP 대응 JSON/PNG다.
