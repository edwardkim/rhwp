# #7642 Stage 81 — 수식 크기 캐시가 없는 HWPX의 실제 슬롯

Issue: #7642

## 생성 방식과 독립 기준

부정 기호 진단 HWPX 두 개는 수동 합성 입력이다. 저장 크기와 기준선은
0이며 LineSeg가 없다. 이를 정상 저장 정보로 간주해 수용 조건을 완화하지
않는다. 한컴 2024에서 같은 입력을 저장한 HWP와 그 저장본의 Print가
별도의 정상 생성 대조군이다. 양쪽 원문 텍스트·수식 script·baseUnit·
컨트롤 위치가 같고 한컴 저장에서 실제 폭·높이·기준선·줄 정보가 발행된다.
저장본 통과를 저장 전 HWPX의 일치로 보고하지 않는다.

입력과 독립 출력은 `pdf/issue7642-negation-contract/`의
`negation-contract-input.hwpx` / `negation-contract-2024.hwp` / PDF 및
각 v2 쌍이다. 저장/Print job과 해시는 기존 Stage 40의 증거를 따른다.
한컴 2024 13.0.0.3901, Print method 0,
`frame_print_to_pdf_ex_one_up`을 확인한 자료이며 원본은 수정하지 않는다.

source `6c10390e0`에서 저장 HWP는 99.08506% / 99.83405%, 저장 전
HWPX는 59.68375% / 67.54569%다. 앞뒤 라벨은 원본 v1에 실제로 두 번
들어 있으며 내용 중복을 엔진이 만든 결함으로 간주하지 않는다.
문제는 수식 슬롯이 없어 뒤 라벨이 수식과 겹치고 큰 수식의 높이를
줄이 수용하지 않는 것이다. v2는 뒤 라벨 없이 같은 크기 문제를 보여 준다.
전체 원본 55개·853쪽과 관계형 회귀 변경 범위를 그대로 유지한다.

## 실제 생산·소비 경로

`flow_metrics_hwp`의 일반 반환은 누락된 저장 폭·높이 0을 그대로
반환한다. `inline_control_size_hwp`는 높이 0을 제외하고, `tac_controls`도
폭 0을 전달한다. 실제 paint는 이미 `EqLayout::for_equation`으로 계산한
수식을 그리므로 측정 슬롯과 실제 그림이 갈라진다. 생산 결과는
`composition.inc.rs`의 TAC 폭과 frame/scalar 줄 구성에 소비되고,
`inline_box::horizontal_metrics`와 `inline_baseline_px`를 거쳐 최종
수식 원점과 뒤 텍스트 원점으로 이어진다. baseline이 원본에 없다는
이유만으로 이미 계산한 수식 기준선을 flow 목록에서 제외하는 조건도 대조한다.

독립 원문·정상 저장본·현재 paint를 조회한 `stage81-negation-cache-source.log`
및 `stage81-negation-cache-paint.log`는 ignored output에 있다. 첫 600HU
`not=` 수식의 실제 layout 폭은 16이고 정상 저장 폭도 1200HU/75=16이다.
`neq`는 실제 폭 20.4, 정상 저장 폭 20.24다. 현재 원본의 수식 bbox 폭은
0이며 실제 layout은 위 폭으로 그린다. 이 값은 원인 진단이며 고정 px
회귀 기대값으로 쓰지 않는다. 일반 `EqLayout::new`의 숫자 전진폭으로
대신 계산하지 않고 실제 paint가 선택한 face 문맥을 소비한다.

## 다음 보정과 반례

새 수식처럼 저장 크기 캐시가 없는 경우 실제 수식 상자에서 흐름 폭·
높이·기준선을 함께 발행하고 측정과 paint가 이를 소비하게 한다.
유효한 저장 크기·보호된 크기·표시 프레임의 계약은 각각 구별한다.
앞뒤 본문, 원본 기준선 누락, 작은/큰 baseUnit, 복합 수식과 빈 수식,
유효한 저장본을 대조한다. 어떤 값이 빠졌는지와 적용되지 않을 사례를
확인한 뒤 구현하며 script ID·문서 ID·고정 화면 수치를 조건으로 쓰지 않는다.

Stage 80의 fresh WASM 빌드 종료·exit 0과 package 고정을 확인한 뒤
생산 소스를 보정한다. 완전한 크기 캐시가 없고 보호된 크기가 아닌
수식은 paint와 같은 face 문맥의 실제 상자와 지원되는 논리 글자 프레임을
발행한다. 불완전한 캐시의 한 축을 정답으로 승격해 새 축과 섞지 않는다.
완전한 양의 저장 크기, 크기 보호, font_size가 없거나 비어 있는 수식은
기존 경로를 유지한다. 흐름 목록은 원본 baseline 필드의 유무 대신
실제 메트릭의 유효한 기준선을 받고 paint도 같은 거리로 원점을 정한다.
새 helper는 `uncached_flow_metrics.rs`로 분리한다.

소스 보정과 같은 단계 문서를 함께 커밋한다. 원본/저장본 Native/fresh
WASM 재대조·확대 판독을 수행하고, 검증 범위가 90% 이상일 때만
정식 검사 변경과 fixture 커밋에 연결한다.

## Native 실행 결과와 확대 판독

source `1839e1a57ec86ee00e98cea16aade180939e13a7`의 Native 빌드,
fmt 검사와 Native Clippy는 exit 0이다. 같은 source의 집중 대조에서
저장 전 HWPX v1/v2는 각각 96.33628% / 99.97388%, 정상 저장 HWP
v1/v2는 99.08506% / 99.83405%다. 앞서 보정한 미주 대조군
all-18의 9–11쪽 최저값은 97.66761%, all-16의 20–22쪽은
96.74046%다. 여섯 입력·열 쪽의 자동 gate가 통과했으며 전체
55개 원본·853쪽의 최종 검증 완료를 뜻하지 않는다.

Native 큐·명령·입력/PDF 해시·페이지별 결과는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`
아래 `missing-equation-cache-focused-native/queue.json`에 있다.
전체 원본 55개의 수식 56,468개를 순회한 별도 입력 조사에서는
폭 또는 높이가 0인 수식이 없었다. 이 조사는 새 누락 캐시 분기의
적용 범위를 확인한 결과이며 시각 무회귀 증거로 대신하지 않는다.

같은 source의 `stage81-bare-not-operand-detail-review.png` 확대
판독에서는 `not {xy}`가 여전히 다르다. RHWP는 `xy` 위에 사선을
겹치고, 독립 Print는 `xy` 앞에 별도 기호를 놓는다. PDF의 글리프
원점도 앞 기호와 뒤 `xy`가 분리됨을 보여 준다. 페이지 점수가
90% 이상이어도 이 모양·내용 차이는 미충족이다. 새 fixture와
회귀 기대값은 아직 추가하지 않는다. 다음 단계에서 문자·관계
기호·그룹을 함께 한컴 저장/Print로 대조해 문법과 표시 규칙을
확인한다. fresh WASM 빌드와 집중 대조는 별도 실행 중이며,
현재 Native 결과를 fresh WASM 완료로 보고하지 않는다.
