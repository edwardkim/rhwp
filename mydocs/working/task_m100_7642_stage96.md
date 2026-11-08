# Stage 96 — 중괄호 조립 잉크와 복합 조건식 전진폭의 독립 대조

Issue: #7642. 구현 source는 `156edf0a9fc9b4cf90651ef7149fb60b1160bd11`이며
Stage95 결과 기록은 `27e6e5ca0`다. 전체 55개 원본·853쪽, 요청한 회귀 범위를 유지한다.
독립 출력 분석을 바탕으로 조판·표시 보정을 구현 중이다. 새로운 정식 회귀나 기대값 변경은 없다.

## 현재 관측과 검증할 가정

Stage95의 정렬 묶음·중첩 분수·한/두 행 중괄호 진단은 90% 이상이지만,
복합 `f(x)` 진단은 79.62361%다. 수학 원본 40쪽의 수치 통과로 이를 면제하지 않는다.
새 확대 이미지에서도 큰 중괄호의 위·아래 잉크 끝이 독립 Print와 다르다.

한/두 행의 일반 중괄호와 세/네 행의 조립 중괄호를 나누어 대조한다.
논리 행 점유와 글꼴 부품의 실제 잉크 경계를 구별한다. PDF span의 size는
비균등 glyph 변환에서 세로 크기 자체가 아니므로 그대로 높이로 사용하지 않는다.
PDF의 문자 전진·변환 크기와 subset의 hmtx·glyph bbox만 읽어 원점을 복원한다.
사설 글꼴 바이트·윤곽은 저장하거나 Git에 포함하지 않는다.

조건식은 같은 11pt/22pt의 독립 LEFT/RIGHT 식과 복합 CASES 안의 식을 대조한다.
원본 script·공백·관계 기호·괄호별 전진을 확인해 누락된 구문 간격인지,
열 맞춤·원래 글자 크기 메트릭인지 구분한다. 특정 문서 좌표에 맞추는 예외를 추가하지 않는다.

## 독립 입력과 출력

12개 수식의 크기·기준선·저장 LineSeg를 비운 진단 HWPX를 만든다.
현재 고정 Native 바이너리로 HWP를 만들고 변환 전후 수식 수를 확인한다.
canonical MCP client의 명시적 engine 2020으로 한컴 정상 저장본을 받은 뒤,
그 저장본을 Print PDF로 산출한다. 입력/산출 SHA·job·실제 한컴 버전·method 0과
one-up 출력 모드를 비공개 인증 정보 없이 ignored output에 보존한다.
합성 진단 통과를 55개 실물 원본의 시각 통과로 대신하지 않는다.

## 다음 구현과 검증

독립 대조가 설명하는 원인만 묶어 보정하고, 논리 점유→실제 glyph 원점→각 출력 backend의
소비를 함께 확인한다. 큰 중괄호를 짧게 보이게 하기 위한 clamp나 임의 고정 px 보정은 하지 않는다.
관련 진단과 원본을 한 번의 빌드 후 재출력하며 새로운 확대 review PNG를 직접 연다.
최종 Native/fresh WASM 전체 TSV·확대 판독 후 관계형 회귀 변경과 전체 필수 검증을 진행한다.

## 독립 출력과 초기 관측

12개 수식의 변환 전후 개수·스크립트·BaseUnit을 실제 확인했다.
한컴 정상 저장 job은 `61a34d38-349b-4aab-8959-abac7c846ae8`,
그 저장본의 Print job은 `dbd11b09-791e-49a5-8e88-6cb61c26d0ed`다.
실제 한컴은 `11.0.0.9136`, engine 2020, method 0,
`hancom2020_pdf_driver_one_up`이며 PDF는 한 쪽이다.
`stage96-brace-spacing-probe/conversion-pipeline.json`에 입력/출력 해시와 경로를 보존한다.
MCP에서 HWP를 정상 저장하고 PDF를 인쇄했다. 분석용 HWPX는 같은 HWP를
RHWP로 읽어 내보낸 로컬 진단이며 MCP의 HWPX 산출로 보고하지 않는다.

일반 행의 독립 프레임 1125HU/2175HU와 다음 높이가 대응한다.

| BaseUnit | CASES 3행 | CASES 4행 | LEFT/RIGHT 3행 | LEFT/RIGHT 4행 |
| ---: | ---: | ---: | ---: | ---: |
| 1100 | 3705HU | 4995HU | 3375HU | 4500HU |
| 2200 | 7185HU | 9690HU | 6525HU | 8700HU |

CASES의 높이는 일반 행 프레임의 합에 행 경계마다 `0.15 × BaseUnit`을 더하면
네 값 모두 정확히 일치한다. 현재 152/1024 leading과 저장 메트릭의 작은 차이도
이 근거로 설명할 수 있다. LEFT/RIGHT는 행 프레임의 합이며 CASES의 leading을 소비하지 않는다.

독립 조건식의 폭은 11pt에서 `<` 4507HU·`GEQ` 4657HU, 22pt에서 각각
9615HU·9990HU다. 원본 복합 조건식과 비교해 괄호·공백·관계 기호의 전진을 추적한다.
`stage96-before-skia/receipt.json`의 현재 코드 대조는 85.36585%로 미달이다.
새 기준과 원인 조사를 확보했으며 구현·시각 보정·정식 회귀 완료는 아니다.


## 추가 독립 대조와 구현 묶음

음수 접두·관계 기호 24식, 단일 기호/괄호/정체 16식, 혼합 글줄 24식을
같은 정상 저장 → Print 경로로 산출했다. 각 저장본에서 수식 수·script·BaseUnit을 확인했다.
Print job은 각각 `d36105af-df31-4077-90c2-83f9a4ff69f2`,
`2a652b98-863c-4397-97a0-d19ea108ad1e`, `6ccdd36d-13a5-4758-99e0-91bc4a33487c`다.
입력·출력 SHA와 실제 한컴 버전/인쇄 모드는 ignored output의
`stage96-prefix-operator-probe`, `stage96-operator-atoms-probe`,
`stage96-mixed-line-frame-probe` 안의 `conversion-pipeline.json`에 기록했다.

- 명시적 공백이 두 연산자 사이에 있으면 자동 간격을 지우지 않는다. 공백 없는 `x=-2`와
  명시적 공백이 있는 음수 조건식을 별도로 대조했다. ASCII `<-`는 한컴에서 왼쪽 화살표로
  해석되므로 tokenizer/parser가 같은 구문을 보존한다.
- 22pt의 논리 em은 2250HU이며 일반 둥근 괄호 두 개는 각각 900HU다.
  `LEFT(x)`의 3075HU와 기본 x의 1275HU 차이에서 괄호 폭을 독립적으로 확인했다.
  GEQ/LEQ의 전진·잉크 프레임은 논리 em과 UPM 1024의 bbox를 소비한다.
- 22pt의 일반 글줄은 2175HU/86%, 명시적 공백·정체 글줄은 2200HU/86%,
  단일 위첨자 줄은 2588HU/88%다. 기본 숫자를 모두 정체 프레임으로 접지 않는다.
  prime·미측정 복합 구조의 프레임은 추정하지 않는다.
- CASES 행 간격은 `0.15 × BaseUnit`이다. 큰 중괄호의 cap/tail 원점 간 거리와
  실제 bbox를 연결해 `논리 높이 − 일반 글줄 높이 + 594/1024 × 글자 크기`로 잉크 높이를
  구한다. 선택은 논리 높이, 부품 조립은 잉크 높이를 소비한다. 한/두 행 일반 glyph는 보존한다.
- 조판의 `Paren` 결과에 양쪽 구분자 윤곽을 보존하고 SVG·Rust Canvas·Skia가 그대로 소비한다.
  Paint JSON은 같은 윤곽을 기존 FontGlyph/Row 계약으로 전달해 CanvasKit의 별도 재계산을 피한다.
  혼합 구분자의 미보정 쪽은 기존 Paren 표시 경로를 유지한다.
- 지원하는 원 face의 논리 루트 프레임을 기존 줄 프레임 helper로 감싼다. 자식 기준선의 상대 원점은
  보존한다. 저장 높이가 명목 글자 크기보다 작은 정상 글줄을 자연 glyph 높이와 비교해 임의 축소하는
  경로에 같은 논리 높이를 공급한다. 빈 수식은 줄 상자로 바꾸지 않는다.

공개 Latin Modern Math의 관계 기호·둥근 괄호 6개 윤곽은 공개 WOFF2의 RecordingPen 명령과
모두 동일함을 확인했고 `THIRD_PARTY_LICENSES.md`에 파생 데이터의 출처를 추가했다.
사설 글꼴은 hmtx/bbox 메트릭만 읽었으며 바이트·윤곽을 복제하지 않았다.
`stage96-public-relation-data-check.json`은 출처 대조이며 시각 통과나 정식 회귀가 아니다.

이 구현 묶음의 lint·빌드·새 출력은 아직 미검증이다. 이전 Stage95 결과를 적용하지 않는다.
독립 진단과 실물 원본의 재출력, Native/fresh WASM TSV, 확대 판독 후 관계형 회귀를 변경한다.


### 조립 연결 경계의 선행 보정

초기 실행은 `48e7c7224`의 Native Clippy 진행 중 조립 경계 결함을 발견해 명시적으로 중단했다.
관측 timeout이나 출력 공백 때문이 아니다. 해당 source의 lint·빌드 통과로 기록하지 않는다.
공개 MATH extender는 선택 부품이다. 3행의 실제 잉크 높이에 최소 한 개를 강제하면
connector 최대 overlap을 초과하므로 0개부터 필요한 개수를 선택하도록 보정했다.
독립 Print 3/4행 높이 8건과 공개 MATH 연결 범위를 대조했으며 모두 유효한 overlap이다.
근거는 `stage96-public-brace-connector-check.json`이다. 새 source에서 검사·빌드를 다시 실행한다.


## 보정 source의 실행 결과

검증 source는 `3c2c3b7eb827df31a315be7fb2aef699993a6703`다. 포맷·Native Clippy·WASM Clippy·
Skia lib 타입 검사는 모두 exit 0이며 Native release-test/Skia 빌드도 exit 0이다.
바이너리 SHA-256은 `7048c4b779840ceff71dea7191be82d9249d85c19fb244bfe501162abf68396a`다.
`stage96-brace-type-checks.json`, `stage96-brace-native-provenance.json`,
`stage96-independent-skia/receipt.json`에 명령·해시·실행 결과를 보존했다.

| 독립 진단 | 직접 Skia 일치율 | 전체 쪽수 |
| --- | ---: | ---: |
| v2 | 97.93503% | 1 |
| v3 | 97.95841% | 1 |
| owner | 99.56214% | 1 |
| brace | 99.26976% | 1 |
| spacing | 95.43265% | 1 |
| prefix | 99.26362% | 1 |
| atoms | 99.25976% | 1 |
| mixed | 93.32864% | 1 |

8종 모두 직접 Skia 수치 gate를 통과했다. v2는 79.62361% → 97.93503%,
중괄호·조건식 spacing은 85.36585% → 95.43265%다. Native SVG/fresh WASM TSV와
전체 55개 원본의 최종 검증, workspace all-target lint·전체 회귀는 아직 완료되지 않았다.

v2·spacing·mixed의 새 review PNG를 직접 열었다. mixed의 전체 점수는 93.32864%지만
공백 없는 `x^-2`에서 2가 본문에 남고 `x^2-a`에서 뒤 식이 본문에 남는다.
같은 입력의 독립 Print는 각각 `-2`, `2-a`를 첨자에 표시한다. 이는 점수 통과로
면제하지 않는 구문 소속 차이다. `mixed/unbraced-script-detail.png`는 실제 렌더 트리의
수식 bbox를 이용한 확대 판독이며 회귀 테스트의 고정 영역 선택자가 아니다.
공백·명시 중괄호·관계 기호 경계의 독립 Print를 먼저 추가 대조한 뒤 구문을 보정한다.

spacing의 22pt curly 슬롯도 독립 저장 폭과 대조했다. CASES 3행은 x 1275HU를 포함해
2400HU, LEFT/RIGHT 3행은 3525HU다. 따라서 curly 한 쪽의 예약은 1125HU이며
명목 fs 대신 측정한 논리 em 2250HU의 1/2를 소비해야 한다. 같은 단위 계약으로 공통
slot helper를 보정한다. 화살표의 11/22pt 독립 전진은 1050/2250HU로 논리 em과 같다.
이 두 전진 및 첨자 경계는 다음 묶음에서 함께 보정하며 전체 완료·정식 회귀 추가로 판정하지 않는다.


### 후속 분석 정정

Stage97에서 curly 폭의 이전 코드 계산을 재확인했다. 75HU 반올림 때문에 기존 22pt
슬롯도 이미 1125HU다. 앞서 명목 fs 사용이 슬롯 결함이라고 해석한 부분은 철회한다.
새 공통 helper의 해당 두 크기 출력은 같으며, 추가 시각 개선으로 세지 않는다.
