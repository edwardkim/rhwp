# #8153: 양수 원본 자간과 글리프 폭 맞춤의 분리

## 원본·사양·기존 해결

두 편람의 111쪽 숫자 `1311334`, `135`는 장평 98%·자간 50%다.
글자 모양 638과 실제 Native trace에서 원본 속성, KoPub 원 메트릭, 저장 문자
전진 11.2px를 확인했다. 입력과 독립 Print는 기존 자료를 그대로 사용한다.
HWP 5.0 revision 1.3 표 33의 언어별 장평·자간은 별도 속성이다. 자간을
글리프 모양의 추가 장평으로 해석하지 않는다.

기존 #2809 기록과 `4ca011cd5423b344db232ccd0486ced663e24e83`을 추적했다.
음수 자간과 배분 간격은 원점과 글리프 폭을 구분하지만, 현재 공통
`glyph_fit_positions`는 양수 원본 자간을 분리하지 않는다. 이후 Native·Canvas의
`canvas_cluster_fit_scale`이 ASCII에 대해 최종 전진을 다시 폭으로 맞추므로,
장평 계산을 정정해도 마지막 단계에서 다른 가로 비율로 덮어쓸 수 있다.

## 실제 실행 근거와 한계

동결 제품 source `be1b4f2ddb`에서 실제 Native layer를 재생했다.
원래 문자 위치를 명시적으로 고정한 identity는 111쪽 RGB 전 픽셀이 같다.
그 상태에서 양수 원본 자간의 ASCII 폭 맞춤만 제외한 진단은 미대응을
47→6픽셀로 줄였다. 이 진단은 paint 사본에서 fit의 원인을 분리한 것이며,
음수 자간으로 바꾸는 방식이나 위치 직렬화의 반올림을 제품에 넣지 않는다.
145쪽은 직렬화 위치 고정의 RGB identity가 달라 그 비교를 제품 무회귀 근거로
사용하지 않는다. 정상 대조군 239쪽의 0픽셀은 유지됐다.

남은 6픽셀은 문단 끝 `135`의 밑줄 우측 끝이다. 독립 Print의 수평선은
첫 숫자 구간 x=138.810..197.666pt, 문단 끝 x=219.722..242.258pt 및
218.764..241.179pt다. 실제 source glyph의 advance와 마지막 glyph 원점을
사용해 끝 자간을 구분하며, 이 좌표를 제품 분기나 이동 상수로 쓰지 않는다.
문단 중간 구간의 자간과 다음 내용은 보존한다.

별도 GDI 측정에서 실제 프로그램 SHA가 Linux 공급본과 같고, PDF의 숫자
`1/3/4/5` 윤곽도 공급본과 같다. 600dpi GDI의 평균 폭 75/76은 독립 Tz와
일치하지만, 그 보정만으로 111쪽의 47픽셀은 줄지 않았다. 폭 맞춤의 실제
소비를 먼저 정정하며 글꼴·원본·PDF·점수 조건을 바꾸지 않는다.

## 공통 구현과 소비 지점

1. 원본 `TextStyle`과 확정 layout positions는 보존한다. 기존 공통 측정기로
   양수 원본 자간을 제외한 intrinsic glyph-fit 경계를 계산한다. 음수 자간,
   kerning, 배분 간격 및 HFT/legacy bold tracking의 기존 분리를 유지한다.
2. `source spacing → 확정 문자 위치`와 `intrinsic 문자 메트릭 → glyph-fit
   target → 실제 가로 배율`을 구분한다. Native·Canvas·SVG와 관련 진단은
   같은 공통 결과를 소비한다. downstream fit에서 추적 간격이 다시 들어가지
   않는지 확인한다. CanvasKit의 별도 소비도 확인한다.
3. 문단/줄 끝의 밑줄은 마지막 가시 glyph의 원점과 intrinsic advance를
   소비한다. 원래 source run·문자 위치·본문 소유·중간 범위의 간격은 유지한다.
   장식의 공통 replay에서 처리하며 backend별 끝 좌표 보정을 추가하지 않는다.
   말미 공백·탭·다중선·세로/회전·script 이동은 실제 적용 경로와 반례를 확인한다.
4. 문서·페이지·face 이름으로 적용 조건을 좁히지 않는다. 코드가 큰 기존
   renderer 모듈은 필요한 공통 helper를 작은 파일로 분리한다.

## 검증과 완료 조건

현재 두 원본의 전쪽 Native/fresh WASM 최저는 각각 99.46283%/99.25254%이며
90% 선행 조건을 충족한다. 신규 검사는 `tests/cases/issue_8153_...`에서 실제
원본의 숫자 폭·원점·문단 끝과 중간 밑줄의 관계를 보호한다. 기대값은 독립 PDF의
글꼴 메트릭·수평선과 원본 속성에서 정하며 helper의 결과를 재인용하지 않는다.
같은 검사로 수정 전 FAIL/수정 후 PASS를 확인한다. 기존 정확한 음수/0 자간과
배분 간격 검사도 유지·실행한다. CI 개별 호출은 실제 누락이 없으면 추가하지 않는다.

집중 Native/fresh WASM·확대 PNG를 먼저 대조한 뒤 두 문서의 전체 페이지
TSV와 배치/소유 보존을 확인한다. 111쪽을 개선한 것으로 전쪽 목표나 전체 필수
검사를 대신하지 않는다. 최종 목표는 1,532조건 모두 raw100·정수 미대응 0,
필수 검사 완료와 PR 준비 후 대기다. upstream/devel 동기화·원격 GitHub 쓰기·
PR 생성은 하지 않는다. 모든 중간 JSON·로그·PNG·SO 분석은 ignored output에 둔다.

## 공통 구현과 실제 검증 결과

제품 코드 `ddfacbc9fa305175a6e0237a7f6e14800a3cede4`에서 양수 원본 자간을
제외한 intrinsic 측정을 `renderer/glyph_fit.rs`로 모았다. 본문 문자 위치·장평·
소유와 음수/0 자간의 기존 계약은 보존했다. 문단·명시적 개행 끝 밑줄의 끝점은
같은 intrinsic 결과를 `PaintOp::decoration_replay`에서 소비한다. 원본 run을
수정하거나 backend마다 끝 좌표를 따로 보정하지 않는다.

실제 원본을 읽는 `tests/cases/issue_8153_positive_source_tracking.rs` 4건은
기존 제품 source `be1b4f2ddb7c730075d15cea1d844dbf7826d024`에서 숫자 폭·
밑줄 길이 assertion으로 모두 FAIL, 수정 후 모두 PASS다. 자동 수집된
`regression_suite_002`에 대해 기존 `run-rust-test.mjs --cargo-test
issue_8153_positive_source_tracking -- --profile release-test --features native-skia
--verbose`도 실제 실행해 **4 PASS / 0 FAIL / 0 ignored**를 확인했다.
CI workflow에는 호출을 추가하지 않았다.

두 원본의 111쪽은 Native 각각 **47→0픽셀**, 표준 최적화 fresh WASM 각각
**41→0픽셀**이다. 숫자의 모양·간격·밑줄 끝을 독립 Print와 확대 대조했다.
기존 화살표의 획 대비 차이는 그대로이며, 숫자 보정의 raw100을 전체 RGB 일치로
해석하지 않는다. 원본·PDF·점수 알고리즘·글꼴 파일은 변경하지 않았다.

| 원본 | backend | raw100·정수 미대응 0 쪽수 | 전체 쪽수 | 전쪽 최저 raw |
| --- | --- | ---: | ---: | ---: |
| HWPX | Native | 347 | 383 | 99.46283 |
| HWP | Native | 344 | 383 | 99.46283 |
| HWPX | fresh WASM | 346 | 383 | 99.25254 |
| HWP | fresh WASM | 343 | 383 | 99.25254 |

전쪽 **1,532개 조건**에서 111쪽의 4조건만 개선됐고 나머지 raw 점수·정수
미대응 수의 하락은 없다. Native 전쪽 렌더 트리 JSON 766개도 수정 전과
바이트 단위로 같았다. fresh WASM은 standard release·wasm-opt 실행을
확인했으며 SHA-256은
`ab87b6393e934da2e555742138ad6811b175add53a6cec54b7688c69ce21d03e`다.
공유 `target/pr-review`, Cargo jobs 8, WASM codegen-units 16·Binaryen 8을
사용했고 Cargo/rustc/wasm-opt는 순차 실행했다.

작업 증거는
`output/pr-review/issue8153-handbook-raw100-20261022/remaining-raw100-diagnostic/positive-source-spacing-product/`
아래의 `same-source-whole-summary.json`, `native-whole-comparison.json`,
`wasm-whole-comparison.json`, `native-render-tree-preservation.json`,
`actual-ci-selector-receipt.json` 및 `review-native/`에 있다. 각 원본의 전체
TSV는 `native/<문서키>/whole-score/silhouette.tsv`와
`wasm-whole/<문서키>/whole-score/silhouette.tsv`다.

현재 raw100·정수 미대응 0은 **1,380/1,532개**, 잔여는 **152조건·40개
페이지 번호·5,389픽셀**이다. 전체 목표와 최종 필수 suite 완료는 아직 아니다.
다음 그림 진단에서는 원본 BMP→Print JPEG의 RGB가 같은 139쪽의 배치 경로와,
원본 공백의 언어 슬롯·장평 및 crop 재표본화 차이가 있는 141쪽을 구분한다.
단순 crop 원점 내림은 독립 자원과의 차이를 늘려 배제했다. 다른 호스트의
sample16용 작업을 가져오거나 원본/PDF를 바꿔 점수를 맞추지 않는다.
