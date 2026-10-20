# Stage 97 — 첨자의 구문 경계와 curly·화살표의 논리 전진

Issue: #7642. 이전 구현 source는 `3c2c3b7eb827df31a315be7fb2aef699993a6703`다.
전체 55개 원본/853쪽과 292개 요청 회귀, #1139 전체 관계형 전환 범위를 유지한다.
HML 수식의 명시적 제외도 유지한다.

## 독립 근거와 반례

Stage96의 직접 Skia 진단 8종은 모두 90% 이상이지만 mixed 확대에서 첨자 소속이 다르다.
한컴 정상 저장/Print의 공백 없는 `x^-2`, `x^2-a`는 뒤 식을 첨자에 포함한다.
현재 parser는 단일 토큰 인자만 소비한다. 공백·명시 중괄호·관계 기호·다음 첨자·개행의
경계를 대조해 필요한 구문 규칙을 정하며 문자열별 예외를 추가하지 않는다.
명시 중괄호가 있는 `x^{2}-a`, 일반 공백이 있는 `x^2 -a`를 같은 경로로 검증한다.

curly 22pt 슬롯은 1125HU다. CASES 3행 2400HU에서 x 1275HU를 빼고,
LEFT/RIGHT 3행 3525HU에서 같은 x를 빼면 양쪽 슬롯 합 2250HU가 된다.
같은 logical em 공급을 CASES와 LEFT/RIGHT가 공유한다. 한컴 왼쪽 화살표의 전진은
11/22pt에서 1050/2250HU이며 설치 face의 advance도 UPM 1024와 일치한다.
기존 관계 기호의 일반 여백을 이 화살표에 붙여 같은 출력이라고 주장하지 않는다.

## 실행 계획

16개 구문을 11pt/22pt로 정상 저장하고 Print PDF를 산출한다. MCP canonical 절차의
명시적 engine 2020과 start/status/download를 사용한다. 입력/저장/Print 해시,
실제 제품·method 0·one-up 모드, 수식 수와 원문 script를 ignored output에 기록한다.
분석용 HWPX는 로컬 export이며 MCP HWPX 산출로 보고하지 않는다.

독립 대조 후 구문 경계·curly 공통 슬롯·화살표 전진을 묶어 보정한다.
AST 소속 → 조판 프레임/전진 → SVG·Canvas·Skia·Paint JSON 실제 소비를 대조한다.
새 source의 lint/빌드 후 기존 8종과 새 구문 대조를 한 번에 재출력하고 확대 판독한다.
새 Native/fresh WASM 전체 TSV와 실제 원본의 확대 판독을 충족한 뒤 관련 회귀를
소유·순서·보존·포함·비겹침 관계로 변경한다. 원본·기존 검사를 제거하거나 수치 통과로
미세 차이를 면제하지 않는다. 전체 필수 검증·새 PR·CI·머지 후속 처리는 남아 있다.


## 일반 위첨자의 독립 구문 판정

정상 저장 job `ffbe7de1-5326-4274-91e3-6ee00c02bb18`, Print job
`e11874fc-a4ca-4b5e-b00b-dbb95a56d487`가 성공했다. 실제 한컴 11.0.0.9136,
engine 2020, method 0, one-up 출력이며 PDF는 전체 2쪽이다.
저장본의 32개 수식에서 script·BaseUnit 보존을 확인했다. 각 문자 origin·글꼴 크기는
`stage97-script-boundary-probe/print-script-role-groups.json`에 보존했다.
사설 폰트 윤곽은 읽지 않았으며 PDF의 문자 위치와 실제 글꼴 크기로 첨자 소속을 판독했다.

| 구문 | 독립 Print의 뒤 식 소속 |
| --- | --- |
| `x^2-a`, `x^2+a`, `x^2a`, `x^2=a`, `x^2/a` | 붙은 식 전체가 첨자 |
| 일반 공백 `x^2 -a`, 명시적 Thin 공백이 있는 식 | 2만 첨자, 뒤 부호·a는 본문 |
| `x^{2}-a`, `x^{-2}+a` | 중괄호 안만 첨자 |
| `x^-2`, `x^-2+a` | 붙은 부호·식 전체가 첨자 |
| `x^(2-a)`, `x^{2-a}` | 괄호 또는 중괄호 안 식이 첨자 |
| `x^2 GEQ a` | 2만 첨자, 관계 기호 뒤는 본문 |
| `x^2#y` | 2만 첨자, y는 다음 줄 |

``` 기호가 포함된 두 번째 행은 한컴의 명시적 Thin 공백이다.
일반 parser의 기본 위첨자 경로를 이 구문 경계로 수정 중이다. 합·적분은 별도 상한
파싱 경로를 쓰므로 동일 계약 여부를 8개 구문/두 크기의 추가 Print로 확인한 뒤 연결한다.
curly 슬롯의 공통 helper와 화살표의 논리 em 전진도 같은 구현 묶음에 포함한다.
현재 새 source는 미검증이며 정식 회귀 테스트를 추가·변경하지 않았다.


## 상한·위아래 슬롯의 추가 Print와 공통 구현

합·적분의 16식은 정상 저장 `3107dbb9-fec0-4814-8cb4-c27703bb7913`,
Print `33b85391-0ae4-472a-9c78-e5f81ac7f686`로 산출했다. 위·아래 슬롯 경계 16식은
정상 저장 `a3fbb613-ecb7-4b7a-bcad-79e2c4abf51f`,
Print `82959c79-aafe-4846-9e75-6bc955c598a6`다. 두 PDF는 각각 전체 1쪽이며
실제 한컴 버전·method 0·one-up 모드와 script/BaseUnit 보존을 확인했다.

`sum_{n=1}^2-a`의 2-a는 상한에 포함되고 일반 공백·명시 중괄호는 경계를 닫는다.
부모에 아래 슬롯이 이미 있는 `sum_{n=1}^2a_n`은 a_n이 상한 내부에 들어간다.
반면 `x^2a_n`은 부모 x에 아래 n, 위 2a가 소속된다. `x^2a^3`의 추가 위첨자 3은
위 식의 마지막 a에 붙는다. `x^2_a+b`의 a+b는 아래첨자이며 `x_a^2-b`의 2-b는 위첨자다.

일반 첨자·합/적분·극한의 인자 소비를 같은 span helper로 연결한다.
부모의 반대 슬롯이 비어 있으면 그 marker에서 span을 끝내고 부모 슬롯을 채운다.
해당 슬롯이 이미 있거나 같은 종류 marker가 반복되면 마지막 원자에 중첩 첨자를 붙인다.
명시 중괄호·일반 공백·Thin/Normal/개행/탭 및 바깥 구분자 경계는 보존한다.
붙은 ASCII 화살표의 의미 변환도 하한에서 유지한다. 반복 첨자는 기존 깊이 상한을 따른다.

curly의 공통 논리 슬롯과 화살표 전진을 이 구문 보정과 함께 구현했다.
새 코드의 lint·빌드·시각 대조는 아직 미검증이며 정식 회귀/기대값 변경은 없다.


## 동일 입력의 보정 전 대조

고정 바이너리 `3c2c3b7eb`로 새 64식의 동일 저장본·Print PDF를 먼저 재출력했다.
`stage97-before-skia/receipt.json`의 실행은 complete이며 시각 gate는 미달이다.

| 진단 | 최소 직접 Skia 일치율 | RHWP/Print 전체 쪽수 |
| --- | ---: | --- |
| grammar | 46.04651% | 2 / 2 |
| bigop | 85.18258% | 1 / 1 |
| marker | 52.98572% | 1 / 1 |

이 미달은 신규 독립 구문 진단이며 기존 55개 원본의 점수로 바꾸어 보고하지 않는다.
새 보정 source의 재출력과 확대 판독 후 필요하면 같은 묶음의 원인을 추가 보정한다.


## 초기 검사와 이전 실행 정리

`b97e38052a8f27f2b1c6ef62859e520c1e89fd47`의 포맷·Native Clippy·WASM Clippy·Skia lib 타입 검사는 모두 exit 0이다.
현재 Native 빌드와 새 출력 대조는 진행 중이며 전체 workspace lint/전체 회귀 통과가 아니다.

현재 source와 다른 `0e178d78`의 두 TSV controller를 실제 PID·자식 프로세스까지 확인해 종료했다.
완료된 Native 23개/WASM 8개 문서의 PNG·TSV·JSON은 변경 전 증거로 보존한다.
종료된 프로세스가 없고 파일이 열려 있지 않음을 확인한 뒤 중간 SVG 111개,
8.41GiB만 해시 원장을 남겨 정리했다. 원본·PDF·PNG·TSV와 공용 target은 보존했다.
근거는 `stage97-retired-before-controllers.json`, `stage97-retired-before-svg-cleanup.json`이다.


## 보정 후 종료 결과와 남은 경로

Native 빌드는 exit 0, 511.50초다. source `b97e38052`와 바이너리 해시는
`stage97-brace-native-provenance.json`에 보존했다. 새로운 진단 11종/12쪽을 재출력했다.

| 진단 | 최저 직접 Skia 일치율 |
| --- | ---: |
| v2 | 97.93503% |
| v3 | 97.95841% |
| owner | 99.56214% |
| brace | 99.26976% |
| spacing | 95.43265% |
| prefix | 99.26430% |
| atoms | 99.25976% |
| mixed | 100.00000% |
| grammar | 46.04651% |
| bigop | 94.56327% |
| marker | 72.03024% |

혼합 첨자는 100%, 합·적분은 94.56327%로 개선됐다. grammar 2쪽과 marker는 미달이다.
두 실패 review PNG를 직접 열고 같은 Native 라이브러리를 링크한 ignored 진단으로
Paint JSON의 실제 LayoutBox를 읽었다. 새 Cargo target을 만들지 않았으며 font/image bytes는
생략했다. 근거는 `stage97-native-layout-diagnostic-provenance.json`과
`stage97-native-layout-json/{grammar-002,marker-001}.json`이다.

일반 변수 이름도 tokenizer에서 Command로 읽히며, 단일 인자 파싱이 명령 fallback의
`try_parse_scripts`를 호출한다. 이 지점에서 `x_a^2`의 ^2를 a에 붙여 span의 반대 슬롯
경계를 먼저 소비한다. 원자 명령의 tail 결합을 span 소비와 분리해야 한다.
일반 Row의 Newline은 가로 원점 누적에 남아 `x^2#y`의 y가 같은 줄에 붙는다.
또한 `layout_subsup`는 독립 한컴 기준선 helper를 소비하지 않아 위첨자가 본체와 겹친다.
세 실제 경로를 함께 보정하며 값·golden 변경으로 미달을 숨기지 않는다.

### curly 폭 분석의 정정

앞서 22pt 명목 fs의 curly 슬롯이 1050HU라고 적은 계산은 잘못됐다.
`source_font_text::logical_advance`는 75HU 단위로 반올림하므로 이전 코드도
1125HU다. 새 공통 helper는 확인된 11/22pt에서 같은 폭이며 슬롯 폭 결함 판정을 철회한다.
spacing 점수가 동일한 실제 실행 결과와도 일치한다. 공통 공급 refactor의 시각 개선을 주장하지 않는다.
