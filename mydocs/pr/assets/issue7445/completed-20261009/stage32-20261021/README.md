# #7445 보정32 — 저장 줄 여백과 자동 표시 대체의 원본 HFT 메트릭

Issue: #7445. 이 단계는 `80968497c2c178eeecb3b2e1d04d086ad27a0dc7` base, 부모 `18ef50940b6cc2f26a3b2d53aedda1bbfd242f25` 뒤의 공통 보정이다. 전체 PR 준비 완료가 아니다.

## 원인과 소비 경계

저장 `cs`와 문단 왼쪽 여백의 수치 일치는 여백이 이미 소비됐다는 출처가 아니다. 원본 UTF-16 TAC 배정이 인증한 줄 또는 실제 해당 줄의 인라인 표가 소유한 프레임에서만 중복 문단 여백을 제외한다. 일반 본문·나중 줄의 장식 도형은 별도로 선언한 문단 여백을 유지한다. 최종 원점과 가용 폭을 측정·정렬·paint가 같이 쓴다. 정상 #6706 인라인 표·다단/분수 대조는 기존 focused case로 보호했다.

자동 프로그램 가용성 조회가 실제 표시용 Batang을 찾았다는 사실은 원본 HFT 조판 폭의 교체 요청이 아니다. `preserveSourceMetrics`가 이 두 계약을 구분한다. 실제 TrueType 공급 신원·표시 대상은 유지하며, 확인한 원본 HFT 전용/자기 메트릭 및 부분 glyph만 보존한다. 기존 `legacy_hft_metric_face` 조회를 쓰고 부분 glyph 자료를 전체 폭 표로 승격하지 않는다. 명시 JSON 환경은 기본값이 기존 선택 프로그램 메트릭이고 내장·TTF 선언·미확인 원본의 기존 경로를 보존한다. 정책도 환경 ID 해시에 포함한다. 문서·쪽·고정 좌표 분기를 제품에 추가하지 않았다.

원 계약 계보: #6389/#7179 `263b61a64a`, #6706/#7161 `4fddb1`의 저장 줄 출처, 단계20 `521db01858`의 정상 표 보정, 주소별 공급 선택 `8c61edaaf5`와 `36440d7dc0`를 대조했다. 현재 명시 TrueType/원본 프로그램 및 공급 protocol 반례를 그대로 실행했다.

## 원본·독립 Print와 전쪽 자격

- 입력: `/Users/tsjang/rhwp-7445-completed-20261009/samples/21_언어_기출_편집가능본.hwp`
- 입력 SHA256: `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15`
- PDF: `/Users/tsjang/rhwp-7445-completed-20261009/pdf/issue7642-exam-print/exam-01-21_언어_기출_편집가능본-hwp-print-2020.pdf`
- PDF SHA256: `68aa48724b4d6f68ed04ef30d6be5d67dcd3bcf57c916aa131f7ad15c2d81ba8`
- 같은 원본 전15쪽, Native/fresh WASM, print/compat2022, 정본96DPI/PDFium smooth_rgb_v1/관용2px/raw90%를 유지했다. 양쪽 누락·추가 쪽0,90% 미만0이다.

|범위|Native raw|fresh WASM raw|
|---|---:|---:|
|전15쪽 최저|95.27598|95.26332|
|2쪽|99.87976|99.85398|
|14쪽|99.79923|99.76097|

전쪽 TSV는 [Native](native-language-all15.tsv), [fresh WASM](wasm-language-all15.tsv)다. 2·14쪽은 양쪽98% 이상과 직접 review/overlay 판독을 확인하여 아래 독립 px 계약을 추가했다. 본문/줄 순서·누락·중복과 함께 검사하며 픽셀 완전 일치를 주장하지 않는다.

## 독립 기대값과 정식 회귀

`tests/cases/issue_7445_language_source_geometry.rs`의 두 검사는 원본15쪽을 사용한다. 설치 글꼴 없이도 같은 가용성 경계를 재현하도록 caller program snapshot을 모형화한다. 모형의 programId를 실제 글꼴 공급 증거로 쓰지 않으며 실제 공급은 별도 Native/fresh producer가 입증한다.

1. 원본 한양신명조 Type3 `<`의 `/Widths=729`, `FontMatrix=.001`와 원본 글자 크기로 자연 전진을 검사한다. 원본 크기1100HU에서 잘못 선택한 Batang 전진687HU가 아니라 HFT 전진801HU를 소비한다. 원본 문단의 단일 소유·내용/순서·직렬화 보존을 확인하고, 괄호 뒤 `의`의 최종 Print 원점을 PDF `tm.x=1154`, `cm.a=.12`에서 환산한184.64CSSpx와 ±2px로 대조한다. 실제 후보 원점185.22667px, 이전182.4px다.
2. 나중 줄의 장식 도형이 일반 이어받기 줄의 문단 여백을 지우지 않는지 원본 `cs`·들여쓰기/줄 높이 비율로 검사한다. 비율은 잘못된4.042727→원본3.268182다. 전체 문단의 단일 줄 소유·내용 순서·누락·중복을 검사하며, `하나의`의 최종 Print 원점을 독립14쪽 `tm.x=3697`, `cm.a=.12`에서 환산한591.52CSSpx와 ±2px로 보호한다.

최종 정식 source SHA256 `5393bb1adc08215def5ff78a81f9afa34339067aa04b03854daa6a82c157c1eb`의 실제 수정 전2FAIL → 후2PASS를 확인했다. 보정 전은687HU와 저장 줄 비율4.042727에서 실패하며 보정 후 두 독립 기대값·px/내용 소유 검사를 통과했다. `--prepare`로1684sources/7410static attrs/48targets를 확인했고, 파생 경로 `regression_suite_001::issue_7445_language_source_geometry`의 실제 nextest도2실행2PASS/262skip,1.093초(exit0)다. 컴파일 포함 25.141초이며 fmt도exit0이다. Source-side unit·CI 개별 호출·기존 검사의 skip/완화는 추가하지 않았다. 초안의 타입 컴파일 실패와 최초 V5의 불충분한 HFT 조회는 최종 FAIL/PASS 근거가 아니다. 초기 단일-case와 두-case proof 및 최종 파일의 hash를 구분해 ignored output에 보존한다.

기존 다단/분수5, 공급 번호1, 원본 glyph프로그램21, 공급 protocol11, 명시 환경5의 총43PASS를 같은 V6 source에서 확인했다. 전체 nextest·필수 전체 게이트 완료가 아니다.

## 생산 출처

- Native CLI: `53e626c17ac87bdd75540fc2cdae230e8caa460fd6320729753ce061a79a2512`
- Native library: `5eb6392b6f89f7505d9cdf8857dc77f4daf5c805f41e4a51f7b504f233580ea3`
- fresh WASM: `caf39404f223bc7aabbf7939f8601549ba6572116aceb93b4aecf168854b356d`
- fresh JS: `d7d8cf9b5f7ce8a0f00b49a39aa1fee49e331cce6774e7ba57c3e9d078cd7030`
- Mac locked wrapper `--target web --out-dir pkg --no-opt`, exit0,389.585초. Docker 최적화 빌드가 아니다. root pkg/Studio public/frozen package hash 일치. 실제 Studio7736의 새 headless 탭 초기화 JS/WASM 응답도 각각HTTP200과 frozen hash 일치·wasmInitialized=true·오류0을 확인했다. TS bridge는 생성 환경 JSON을 `setFontEnvironment(environment)`로 그대로 넘겨 새 정책 필드를 제거하지 않는다. 이는 문서 paint·글꼴 준비·CanvasKit·Actual36 수용을 대신하지 않는다.
- Native source patch SHA256: `50634dd459c18008af5b8163241200449a5d4172bb90d963486e835002b547a0`. 아래 runtime 파일 hash로 현재 코드와 연결한다.
  - `src/renderer/font_environment.rs`: `f694d57c6a5491e6a827d3da8fca9d11956ef74d99ddacf34933fc1d582269af`
  - `src/renderer/layout/paragraph_layout.rs`: `75939ca90e9b8ba45f5c320dd14bd40585420792e8b2a17975317969540a16ba`
  - `src/renderer/style_resolver.rs`: `42d78cd8a3f1ca81f329fe65c22285fff0103369a54ab36c4a4cf79725a5b6dd`

## 대표 그림 직접 판독

2쪽 괄호 뒤 원점과 문장 진행,14쪽 이어받기 들여쓰기·뒤 문단·나중 도형의 보존을 Native/fresh 각각 확인했다. 작은 획/윤곽 차이는 남으며 전체 문서의 모든 미세 간격 완료를 뜻하지 않는다.

|출력|review|overlay|
|---|---|---|
|native 2쪽|[review](native-p002-review.png)|[overlay](native-p002-overlay.png)|
|native 14쪽|[review](native-p014-review.png)|[overlay](native-p014-overlay.png)|
|wasm 2쪽|[review](wasm-p002-review.png)|[overlay](wasm-p002-overlay.png)|
|wasm 14쪽|[review](wasm-p014-review.png)|[overlay](wasm-p014-overlay.png)|

## 남은 범위와 정정

- 언어8·15쪽은90% 이상이어도 #8064 accepted floor 대비0.1%p 기준의 미달이 남는다. 교육 변형·법령의 accepted floor도 계속 보정한다. #8064 전체179조건2082쪽은 사용자 지시대로 Native만 새 V6에서 실행 중이다. 이전 V3를 새 source 통과로 재사용하지 않는다.
- 집중7조건206쪽 Native/fresh는 완료했으나 sample16 HWPX18쪽86.59436/86.59527%, 수도시설3쪽71.85405/72.05077%가 남는다. 이 두 미달을 이번 언어 회귀 자격이나 전체 통과에 포함해 상쇄하지 않는다. sample16의 두 source 후보 PNG는 동일하며 WMF 장치 표본 재생을 별도 추적한다.
- 정확한809 base와 주요8문서769쪽+별도water3의 Native 대조는 끝났다. 편람·rowbreak·PrEP·TAC의0.1%p 하락도 계속 보정한다. 특히 수도시설을 기존 미달로 분류했던 기록은 새 base 대조에서3쪽 -28.02490%p가 확인되어 현재 보정 대상으로 정정했다. base WASM 전쪽도 별도 완료까지 수집한다.
- 모델 순회는7·10쪽 Shape2.4px overflow를 출력한다. 이 경고를 숨기지 않고 실제 paint/고정 base와 대조할 항목으로 남긴다.
- 전체 nextest·필수 lint/build/Skia/manifest/unit tiers 및 최신 실제 소비36건, 현재 source 전체 제출 범위 재검증은 남아 있다. PR 승인 요청 전까지 해결·검증을 계속한다.

로그·중간 JSON·진단 source는 ignored `output/pr-review/7445-completed-20261009/rebase-80968497-20261021/`에만 있다. 원본·PDF·글꼴·다른 작업장·공유 target 및 지침은 변경하지 않았다.
