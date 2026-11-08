# 이슈 #7642 — 근호의 공통 기하와 피근호식의 논리 프레임 대조

시작 커밋: `4cf6c207d`. 이번 작업의 회귀 실패는 모두 현재 브랜치에서 보정한다.

## 독립 관측과 원인 확인

Stage27에서 정체 글자·함수의 전진폭은 정상 한컴 저장값과 98/98 일치했다.
근호 28건의 Native 일치율은 94.62%이지만 확대 판독에서 근호의 올라가는 선이
너무 수직이고 분수·중첩 근호의 상단과 하단도 다르다. θ는 Print의 기본 이탤릭에
대해 정체로 표시된다. 이 차이는 90% 통과로 면제하지 않는다.

현재 `layout/fraction_geometry.rs`는 근호의 높이를 자식의 자연 상자와 대칭 여백으로
계산한다. 논리 글꼴 프레임은 별도의 소비 경로에 있으며 중첩 근호의 실제 기하에는
반영되지 않는다. SVG·Canvas·Skia·CanvasKit에는 같은 근호 스케치가 중복되어 있다.
측정과 paint가 소비하는 피근호식 프레임·기하를 먼저 연결한다.

## 실행 순서와 반례

1. 기존 근호 대조 입력을 바탕으로 크기·기준선·저장 줄을 비운 입력을 만든다.
   14 글자 크기에서 숫자·변수·분수와 그 근호, 근 지수·중첩 근호·위첨자 혼재,
   기본 θ와 `rm theta`를 정상 한컴 Save 후 Print로 독립 재생성한다.
2. 원자·분수·중첩 구조의 높이·기준선·전진과 실제 Print 원점을 대조한다.
   단순 사례의 비율을 모든 구조로 확장하거나 실물 px 좌표를 고정하지 않는다.
3. 독립 결과가 지지하는 논리 프레임과 공개 표시 글꼴의 기하를 공통 결과로 만든다.
   사설 폰트 바이트·윤곽은 커밋하지 않는다. 다른 face·명시적 정체·복합 구조를
   반례로 확인한다. 표시 기울임과 논리 전진을 같은 임의 수치로 묶지 않는다.
4. 진단 입력과 원본 문제 페이지의 Native 직접 비교·확대를 먼저 실행한다.
   코드가 고정되면 fresh WASM, 영향 입력의 전체 페이지, 필수 lint·전체 회귀를
   순서대로 검증하고 그 근거에 따라 관계형 검사로 변경한다.

2022년 10월 HWPX 9·10쪽의 단 경계, 2023년 9월 HWP/HWPX의 21/20쪽 차이,
`exam_math.hwp`·`exam_math_no.hwp`의 미달 페이지도 현재 브랜치 보정 범위에 남는다.
현재 단계는 PR 준비 완료나 회귀 실패의 해결 판정이 아니다.

## 독립 정상 Save/Print 산출

진단 입력은 `pdf/issue7642-radical-frame-contract/`의 HWPX·정상 HWP·Print PDF다.
154개 수식, 14 글자 크기이며 Print는 5쪽이다. Save job은
`1f04f531-0fb3-48e8-bc45-91b92888ca56`, Print job은
`ea38c9ca-0713-40fe-8db3-8d1bbea82e19`다. 한컴 `11.0.0.9136`,
`pdf_print_method: 0`, `hancom2020_pdf_driver_one_up`을 확인했다.
원본 해시·출력 해시·측정표·실행 로그는 ignored 증적 공통 경로의
`radical-frame-contract-independent-print.json`과
`radical-frame-source-contract.json`에 있다. 새 회귀 fixture/golden이 아니다.

숫자·변수·근 지수·분수·중첩 근호 70건 모두에서 근호가 더하는 폭은
`논리 em 전진 + 글자 크기의 0.17배`로 일치했다. 논리 em은 Source face의
짝수 ppem 격자를 소비하므로 7pt와 8pt 같은 반례에서 명목 1em과 다르다.
높이·기준선은 단순 0.15배 추가만으로 전체 크기가 일치하지 않는다.
`radical-frame-width-rule-check.json`의 70/70 결과와 높이 차이를 구분하며
높이를 같은 비율로 단정하지 않는다.

그리스 변수의 표시 투영은 ASCII 변수와 같은 Unicode 수학 알파벳 규칙을
소비하도록 수정 중이다. AST·논리 전진은 보존하고 기본 이탤릭과 명시적
`rm`을 구분한다. 단순 θ만의 문서별 분기를 추가하지 않는다.

## 그리스 변수 표시 투영의 소단계 검증

기본 그리스 변수를 Unicode 수학 이탤릭 글리프로 투영하고 `rm`에는 정체를
유지하도록 공통 layout 결과를 수정했다. SVG·Canvas·Skia는 같은 Text 결과를
소비하며 ASCII 변수의 기존 매핑을 유지한다. AST와 논리 전진은 보존한다.

Native 빌드는 exit 0, 바이너리 SHA-256은
`29980cb6ccd455d3d3dfcf30d785f71f1a5d89f07c638ab9383e5037f085f2b9`다.
source 파일 해시와 명령은 증적 공통 경로의 `greek-variable-native-provenance.json`에 있다.
Native Clippy는 exit 0(53.55초), 포맷과 `git diff --check`도 exit 0이다.

| 검증 범위 | 결과 | 증적 |
| --- | --- | --- |
| 정체 글자·함수 98건 전진 | 정상 저장값과 98/98 일치 유지 | `greek-variable-width-contract.json` |
| 기본 θ·정체 θ 14 크기의 전진 | 정상 저장값과 28/28 일치 | 위 같은 계약 파일 |
| 기존 근호·함수 28건 Native 1쪽 | 94.62459%, 누락 없음, exit 0 | `greek-variable-diagnostic-native/radical-function/summary.json` |
| 확대 판독 | θ의 이탤릭은 개선됨. 함수 글리프의 크기·두께와 근호 모양 차이는 남음 | `representative-native/greek-variable-after/theta-and-functions-3x.png` |
| 원본 2022년 11월 18·21쪽 Native | 96.87208%, 96.85740%, 누락 없음, exit 0 | `nov2022-greek-variable-native/summary.json` |
| 추가 근호 대조 154건 Native 5쪽 | 4쪽 89.73133%, 5쪽 84.23074%, exit 1 | `greek-variable-diagnostic-native/radical-frame/summary.json` |

추가 대조의 수정 전 결과는 4쪽 89.72317%, 5쪽 84.23880%다. 기본 θ의
스타일 수정으로 근호의 모양·높이를 해결했다고 판정하지 않는다. 특히 5쪽의
작은 점수 하락을 없다고 보고하지 않는다. 전후 직접 비교·미달 페이지를 보존한다.
기존 정상 원본의 새 fixture/golden이나 회귀 기대값은 추가·변경하지 않았다.
근호 폭 규칙은 입증했지만 아직 코드에 반영하지 않았고, 근호 높이와 그림 기하는
계속 보정해야 한다. fresh WASM·전체 회귀·나머지 필수 lint 게이트는 미검증이다.
