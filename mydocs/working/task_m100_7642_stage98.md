# Stage 98 — 첨자 원자 소비와 일반 개행·복합 첨자의 공통 배치

Issue: #7642. 이전 구현은 `b97e38052a8f27f2b1c6ef62859e520c1e89fd47`다.
전체 55개 원본/853쪽, 요청 회귀 292개와 #1139 전체 관계형 전환 범위를 유지한다.
HML 수식의 명시적 제외를 유지하며 미달 원본이나 기존 검사를 제거하지 않는다.

## 원인과 독립 기대값

Stage97의 독립 정상 저장/Print 64식에서 세 경로의 결함을 확인했다.
명령 토큰으로 읽힌 변수의 fallback이 뒤 첨자를 먼저 소비해 `x_a^2`를
`x_{a^2}`로 만든다. 첨자 span의 원자 소비에서는 뒤 marker를 부모 슬롯 판정에
남기며, 기존 구조 명령의 인자 소비와 깊이 상한은 유지한다.

일반 Row의 `#`는 실제 배치에서 가로 줄에 남는다. 독립 Print의 `x^2#y`는
두 줄이며 11/22pt 높이는 각각 2548/4983HU다. 일반 개행의 행 구성과 논리 프레임을
기존 정렬 묶음 경로에서 공유하고, 일반 줄은 왼쪽 정렬·명시적 빈 줄을 보존한다.
바깥 leading은 실제 독립 다줄 묶음만 소유한다.

위·아래 첨자를 함께 둔 경로는 단일 첨자와 다른 기준선 공식을 사용한다.
단일 위첨자·아래첨자의 독립 프레임과 fallback 계산을 공통 helper로 소비하고,
복합 첨자의 실제 자식 원점과 점유 끝으로 높이를 계산한다. 적분의 별도 배치는 유지한다.

## 소비 경로와 검증

구문 span → 명령 원자/부모 슬롯 → AST → 공통 script/다줄 LayoutBox →
SVG·Canvas·Skia·Paint JSON의 자식 원점으로 연결한다. 별도 저장 높이로 결과를
숨기거나 문서 ID·수식 문자열·절대 px로 분기를 만들지 않는다.

세 경로를 함께 수정하고 fmt·Native/WASM Clippy·Skia check·Native 빌드를 순차 실행한다.
기존 8종과 구문·합/적분·marker 3종의 동일 Print PDF를 일괄 대조하고 새 PNG를
확대 판독한다. 직접 Skia 진단은 전체 Native/fresh WASM 검증을 대신하지 않는다.
새 source의 전체 TSV·원본 확대 판독 후 관련 회귀를 소유·순서·보존·포함·비겹침
관계로 변경한다. 필수 전체 회귀·lint·WASM·새 PR·CI·머지 후속 처리는 남아 있다.

## 구현 반영

첨자 span의 Command 원자만 뒤 첨자 부착을 유보한다. 일반 command 호출과 구조 인자는
종전 부착 경로를 사용하며 공통 재귀 깊이 가드를 유지한다. 일반 `#` Row는 빈 왼쪽·
실제 오른쪽 행으로 공통 `layout_aligned_rows`를 소비한다. 정렬용 여백은 0이며 빈 행도
보존한다. 실제 root의 다줄만 standalone leading을 소유한다.

Sup/Sub/SubSup의 위 이동량·아래 원점과 논리 점유를 두 helper로 공유했다. 복합 첨자는
실제 위·아래 자식 원점과 점유 끝을 반영하고 적분의 별도 경로는 유지한다. fmt 실행은
종료 코드 0이다. 새 source의 lint·빌드·시각 검증은 아직 미실행이며, 정식 회귀 추가나
기대값 변경은 하지 않았다.

## 초기 검증 결과

구현 SHA는 `31b0f37c32ee7846f66bb9d53ccb039482948b46`다.
`stage98-brace-type-checks.json`의 fmt·Native Clippy·WASM Clippy·Skia lib check는
모두 종료 코드 0이다. workspace 전체 lint나 전체 회귀 통과를 의미하지 않는다.

`stage98-parser-ownership/receipt.json`은 기존/현재 source를 해시 고정한 parser 진단이다.
`x_a^2`, `x^2a_n`, `x_a^2-b`의 기존 중첩 소속이 부모 x의 두 슬롯으로 교정됐다.
명시 중괄호·반복 위첨자·기존 아래 슬롯이 있는 합·근호·구분자 등 9개 대조군의
AST는 유지됐다. 처음 진단 컴파일에서 legacy 정규화 모듈 누락으로 실패했으며 이를
포함한 재실행은 양쪽 모두 종료 코드 0이다. 이 준비 실패는 결함 재현으로 세지 않는다.

`stage98-whole-input-catalog-audit.json`에서 55개 원본/독립 PDF의 실재·해시와
PDF 전체 853쪽을 재확인했다. 렌더링 일치나 Print 출처를 재검증한 결과는 아니다.
`stage98-regression-geometry-audit.json`은 요청 경로에 대응하는 92개 현재 파일을
조사한 정적 후보다. 37개 파일, 128개 test block에서 고정 geometry 검토 후보가
검출됐다. 원 #1139 파일의 43개 후보가 포함되며 이동한 관계형 파일은 이 패턴에
검출되지 않았다. 같은 파일의 비요청 검사가 포함될 수 있으므로 이 수치를 완료율이나
전체 관계형 여부로 해석하지 않는다. 실제 body와 독립 출력 대조가 남아 있다.

Native 빌드와 새 11종 전체 Print 대조는 실행 중이다. 새 대표 PNG의 확대 판독과
전체 Native/fresh WASM, 회귀 관계형 전환·필수 전체 검증·PR 절차는 아직 미완료다.

## Native 빌드와 12종 일괄 대조

Native 빌드는 종료 코드 0, 428.12초다. 고정 바이너리 SHA-256은
`a096e9518a71ce500b8d7eb78a7eff30671bb9ea4d05acc50a71f1276c2b0998`이다.
대조 스크립트의 입력 디렉터리 복사 오류로 첫 실행이 렌더 전에 실패했다. 원 receipt를
`stage98-independent-skia/first-input-path-failure.json`에 보존하고 실제 Stage97 입력으로
고친 뒤 재실행했다. source나 빌드의 실패로 세지 않는다.

`stage98-independent-skia/receipt.json`은 12문서/13쪽/200식의 실행 종료를 기록한다.
각 문서의 최소 직접 Skia 점수는 다음과 같다. 전체 Native SVG/fresh WASM gate는 아니다.

| 대조 | 최소 일치율 |
| --- | ---: |
| grammar | 99.82786% |
| marker | 97.35812% |
| bigop | 94.56327% |
| uppercase | 95.78246% |
| v2 | 97.93503% |
| v3 | 97.95841% |
| owner | 99.56214% |
| brace | 99.26976% |
| spacing | 95.43265% |
| prefix | 99.26430% |
| atoms | 99.25976% |
| mixed | 100.00000% |

개행 대조는 46.04651% → 99.82786%, marker는 72.03024% → 97.35812%로 개선됐다.
`grammar/review/review_002.png`에서 y의 다음 줄 소속과 왼쪽 정렬을,
`marker/review/review_001.png`에서 부모 두 슬롯과 반복 첨자의 소속을 직접 확인했다.

## 90% 통과 후 추가로 확인한 대문자 간격 차이

대문자 반례 16식은 정상 저장 `6781510a-2d3a-4681-b160-b2c597f42d27`,
Print `c9f8b4c5-0b90-454d-98a2-f53e494bfde2`로 성공했다. 실제 한컴 11.0.0.9136,
engine 2020·method 0·one-up이다. 분석용 HWPX는 로컬 export이며 MCP HWPX가 아니다.

`uppercase/uppercase-subsup-detail-review.png`는 실제 수식 bbox·fontSize로 선택한
동일 영역 확대다. 옆 행이 섞이지 않도록 세로 여백을 fontSize의 0.08배로 설정했다.
22pt `X_1^2`는 확대 89.29504%, `rm X_1^2`는 91.29213%다. 전쪽 95.78246%가
이 차이를 면제하지 않는다. 초기 확대의 큰 여백에는 옆 행이 들어가 더 낮은 점수가
나왔으며, 일반 페이지용 overlay의 범례를 좁은 crop에 넣으면 긴 세로 이미지가 돼
같은 overlay_color의 내용부만 별도 확대했다. 정식 회귀 selector가 아니다.

SubSup의 위첨자 x는 단일 Sup와 달리 대문자 여백을 소비하지 않는다. 동시에
22pt 이탤릭 X의 전진은 현재 1650HU이고 독립 Print의 아래첨자 시작은 약 1725HU다.
해당 논리 크기는 기존 전진 표에 없어 nominal em fallback을 사용한다. 이미 확인한
한컴 logical em 2250HU의 glyph advance 격자로 공유하면 X는 1725HU가 된다.
이 두 공통 소비 경로를 후속 묶음에서 보정한다.

이번 source의 fresh WASM 및 전체 55원본 대조·관계형 전환은 아직 실행하지 않았다.
미세 간격 차이를 먼저 보정하며 전체 목표와 기존 검사 보존 범위를 유지한다.
