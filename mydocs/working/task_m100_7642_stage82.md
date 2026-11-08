# #7642 Stage 82 — NOT의 문자·관계 기호·그룹 소비 규칙

Issue: #7642

## 독립 근거와 현재 결함

Stage 81의 크기 캐시 보정 source `1839e1a57`에서 저장 전 HWPX도
페이지별 90%를 통과했다. 그러나 `not {xy}`를 확대하면 독립 한컴
Print는 `xy` 앞에 별도 기호를 표시하며, RHWP는 `xy` 위에 사선을
겹친다. 앞 기호의 의미는 아직 확정하지 않는다. 단순 폭 보정으로
이 내용 차이를 숨기지 않는다.

[한컴 수식 명령어 설명](https://help.hancom.com/hoffice_mac/webhelp/9.0/ko_kr/hwp/insert/equation/equation%28explanation%29.htm)은
NOT를 뒤 글자에 사선을 표시하는 명령으로 설명하고 `not =`를
예시로 든다. 그룹·문자열의 실제 소비 방식은 이 설명만으로
확정할 수 없으므로 한컴 2024 정상 저장본과 Print로 확인한다.

## 진단 입력과 적용 경로

기존 합성 입력의 서식·글꼴·문단 구조를 보존하고 수식 script를
교체한 20개 반례를 `pdf/issue7642-not-operand-diagnostic/`에 만든다.
문자 하나·문자열·숫자·CJK·빈 그룹·관계 기호·명령 기호·그룹 밖
문자와 큰 baseUnit을 포함한다. 생성 방식과 정확한 목록은 ignored
output의 `stage82-not-operand-cases.json`에 기록한다. 원본은
수정하지 않으며 MCP 2024의 저장 → 정상 저장 HWP Print 순서로
독립 기준을 발행한다. 원격 job 성공과 인쇄 metadata를 확인한
뒤에만 출력 결과를 독립 근거로 사용한다.

현재 생산 경로는 `parser.rs::parse_command_fallback`의
`DECORATIONS` 조회 → `parse_single_or_group` →
`EqNode::Decoration(StrikeThrough)`다. 측정은
`layout/decoration_geometry.rs`에서 body 상자를 그대로 쓰고,
SVG/canvas는 같은 장식 body 위에 사선을 그린다. 인자 소비가
잘못되면 이후 측정·paint가 같은 잘못된 AST를 사용하므로
좌표나 폭만 보정하지 않고 파서의 규칙부터 대조한다.

## 보정과 검증 순서

한컴 대조 결과로 NOT의 적용/비적용 조건을 정하고, 기존
`not =`, `x not< y`, `neq`와 다른 장식 명령을 정상 대조군으로
보존한다. source 변경 전에 Stage 81 fresh WASM 빌드의 종료와
package 고정을 확인한다. 문서 ID·특정 문자열·고정 px 폭을
분기로 쓰지 않는다. 출력 보정 뒤 같은 입력의 Native/fresh
WASM 전체 페이지 TSV와 확대 이미지를 다시 확인한다.

전체 55개 원본·853쪽 및 관련 관계형 회귀 정비 범위는 유지한다.
새 회귀 검사는 해당 독립 Print 대조의 모든 페이지가 90% 이상이고
확대 판독의 알려진 차이가 해소된 뒤에만 추가한다. 정식 검사는
내용 보존·개체 소유·앞뒤 순서 등 관계를 검사한다. 현재 단계는
독립 진단 출력과 문법 분석 중이며 완료·승인 상태가 아니다.

## 독립 출력의 관측값과 후보 구현

한컴 2024 `13.0.0.3901`의 정상 저장 job
`db97bebb-8021-44d0-93e4-76c8868b1ba7`와 Print job
`8dcf2436-3410-47af-b4dc-7adb855699d9`가 성공했다. Print method 0,
`frame_print_to_pdf_ex_one_up`이며 PDF SHA-256은
`2a29f62bf20b129a98176bafa3db1a41e86fffd3a04aa4cca534070ce5c734eb`다.
원문과 정상 저장본의 20개 script가 같음을 parser로 확인했다.

그룹 앞 NOT는 그룹의 첫 내용에 적용되지 않는다. `{`에 사선을
표시하고 그룹 내용은 뒤에 보존한다. 빈 그룹도 부정된 `{`만
표시한다. `not xy`는 첫 `x`에만 사선을 표시하며 `y`의 원점은
뒤에 따로 있다. 관계 기호·숫자·CJK·rm·큰 크기의 그룹에서도
같은 규칙이다. Print의 앞 글리프 U+E04B는 설치 HYhwpEQ의
중괄호이고 전진폭은 594/1024em이다. 글꼴 바이트나 윤곽을
저장소에 추가하지 않는다.

`parser_negation.rs`는 한 글자만 소비하고 남은 문자열과 그룹
토큰을 다음 요소로 보존한다. 그룹 닫힘과 뒤 첨자 소유는 기존
파서가 처리한다. LaTeX `\not{=}`는 lexer에서 출처를 유지해
기존 그룹 인자를 보존한다. 일반 크기의 표시 중괄호는 독립 face
전진폭을 측정과 paint에 함께 전달한다. 여는 구분자 뒤 연산자는
왼쪽 이항 간격을 예약하지 않는다. 이는 그룹 앞 NOT의 관계
기호 입력에도 적용된다. Stage 81 fresh WASM 빌드 exit 0과
JS/WASM의 public 동기화 해시를 확인한 뒤 후보 소스를 변경했다.

후보 구현의 빌드·Native/fresh WASM 비교는 아직 미완료다.
새 회귀 기대값·baseline·fixture는 추가하지 않았다.

## Native 확인과 남은 표시 차이

source `b18388335d3057e76e04d63a1af37979f414cf6f`의 Native 빌드는
exit 0, 452.48초이며 fmt와 Native Clippy도 exit 0이다. 다음
여섯 입력은 한 쪽 전체 대조와 쪽수 gate를 통과했다.

| 입력 | Native 최저값 |
| --- | ---: |
| 새 NOT 진단 원문 HWPX | 99.20143% |
| 새 NOT 진단 한컴 저장 HWP | 98.99424% |
| 기존 부정 진단 원문 v1 | 99.97777% |
| 기존 부정 진단 원문 v2 | 99.97388% |
| 기존 부정 진단 저장본 v1 | 99.86270% |
| 기존 부정 진단 저장본 v2 | 99.83405% |

같은 새 진단 저장본의 보정 전 source `1839e1a57`은 91.69811%다.
20개 script의 AST 조회와 별도 진단 계약에서 한 글자 소비·
그룹 내용 보존·뒤 첨자 소유·LaTeX 그룹 인자·일반 bar를 확인했다.
동일 진단은 보정 전 실제 AST에서 의도한 원인으로 exit 101,
보정 후 exit 0이다. 정식 회귀 추가의 선행 조건을 대신하지 않는다.

전체 원본 55개의 수식 56,468개를 조회한 별도 조사에서 한컴 NOT는
18개이며 모두 `a not= 0` 형태다. 이번에 발견한 그룹 앞 NOT와
구별되는 정상 대조군으로 유지한다. 이 입력 조사는 시각 통과로
대신하지 않는다.

ignored output의 `stage82-not-operand-before-after-print-detail-review.png`는
같은 행을 독립 Print의 내용 라벨로 찾아 수정 전·후·Print를 같은
배율로 보여 준다. 별도 기호 누락과 문자열 전체 사선은 바로잡혔다.
다만 표시 중괄호는 Print의 수학용 글리프보다 세로 점유가 작아
완료 판정은 보류한다. 설치 HYhwpEQ에서 일반 ASCII `{`의 잉크
범위는 y=-146..745/1024em, Print의 U+E04B는 -203..821/1024em이다.
같은 문자 모양이라고 두 글리프의 프레임을 같게 취급한 표시
경로를 재검토한다. 사설 글꼴 윤곽이나 바이트를 복제하지 않는다.
공개 수학 글꼴의 표시 프레임 대조는 아직 미실행이다.

새 source의 fresh WASM 빌드는 순차 실행 중이다. 현재 Native
결과를 fresh WASM 완료·전체 원본 853쪽 완료로 보고하지 않는다.
fixture/baseline/정식 회귀 기대값은 변경하지 않았다. #1139의
77개 검사도 정적 기하 후보 58개를 조사했으며 관계형 전환과
실행 검증은 아직 미완료다.
