# #7642 — 수학 명령과 인용 문자의 실제 글리프 투영

Issue: #7642. 작업 브랜치는 `task/7375-rectangle-corners`다.
새 base `170b886f167b705b6e630c5b34bb638d92fae0a9`에 18개 로컬 커밋을 충돌 없이
재배치했으며, 전후 range-diff의 모든 패치가 같다. 새 head는 `136f08c2aa`다.
이 사실만으로 리베이스 기능 보존·새 head 시각 검증이 끝난 것으로 판정하지 않는다.

## 선행 이력과 현재 남은 결함

- 원 PR #7600의 dyad와 이후 한컴산뜻돋움 폭·미분 기호 face 보정을 구분한다.
  기존 #7642 stage1·PR7600 검토 기록·정식 진단 fixture의 생성·Print 출처를 확인했다.
- 현재 고정 ece9 base와 채택 후보의 수식 진단은 모두 3쪽이며 Native
  99.95137/99.94100/96.16576%, fresh WASM 99.95196/99.95533/96.26274%다.
  본문 접두어의 과거 74.88730%를 현재 실패로 재인용하지 않는다. 90% 이상이어도
  3쪽의 삼각형과 인용 작은따옴표가 Print와 다른 것을 직접 확인했다.
- 원본 11월 21쪽도 정확한 PR7600 input-09 Print와 대조해 Native 최저 98.78595%,
  fresh WASM 98.77736%다. 별도 재저장본은 #7640에서 보정했으며 두 입력을 섞지 않는다.

## 독립 근거와 위반 규칙

- 입력은 `tests/fixtures/equation-font-metrics/equation-prime-width-diagnostic-2020.hwp`,
  SHA `03f38b458201832b4bab3cb2c040b6bb2640e125740c78277ed8be276afc1c44`다.
  대응 3쪽 Print는 `pdf/equation-font-metrics/equation-prime-width-diagnostic-print-2020.pdf`,
  SHA `12057cf4571d018ea32db8546189712a2dce933fdbf3f4a7eaea92243b08d0d0`다.
  한컴 2020 Print job·저장 방식은 fixture의 PRIME_WIDTH_DIAGNOSTIC.md에 보존돼 있다.
- 여섯 크기의 `rm triangle ABC`는 Print에서 `Haansoft Batang` U+2206 `∆`와
  HYhwpEQ ABC로 출력된다. 현재 명령의 AST는 그대로 두고 공통 layout이 잘못 발행하는
  일반 수학 face의 U+25B3 `△`만 실제 표시 glyph·face·논리 전진으로 바꿔야 한다.
- 여섯 크기의 `"'"`는 Print에서 HYhwpEQ U+E050이다. 인용 밖 미분은 별도
  Haansoft Batang U+2032다. 설치 공통 HYhwpEQ의 UPM 1024·전진폭 163과 기존 논리 em
  격자가 인용 입력의 독립 저장 폭 75/150/150/150/225/300HU를 설명한다.
  현재 인용 문자가 Number로 분류돼 일반 수학 face의 ASCII U+0027을 표시하는 오류를 고친다.
- 글꼴 바이트·사설 윤곽·문서별 이동값을 추가하지 않는다. 명령과 literal △,
  인용 문자와 미분 prime, HYhwpEQ와 HancomEQN·legacy HFT를 각각 구분한다.

## 공통 생산·소비 경로

`명령/인용 범위 → 공통 layout의 실제 glyph·face·전진 → 동일 LayoutBox를 읽는
SVG/Canvas/Native paint`에서 보정한다. parser 원문을 전역 치환하거나 backend별로
기호를 다시 고르지 않는다. 인용 문자가 여러 문자 안에 섞여도 기존 가로 줄 합성·순서를
재사용하며, 작은따옴표 하나만 맞는 문자열 예외로 끝내지 않는다.

삼각형의 다음 ABC 원점은 같은 크기의 독립 `rm ABC` 상자와 차이를 대조한다.
기준선·실제 잉크도 Print와 확대 판독한다. 아직 설명되지 않은 크기별 위치 수치를
추가하거나 숫자 점수만으로 미세한 차이를 면제하지 않는다.

## 단계별 검증과 보존

1. 새 devel을 독립 source archive로 고정하고 Native/fresh WASM producer를 만든다.
   Mac에서 Cargo를 순차 실행하고 jobs8·총 capture8을 유지한다. 이전 base archive·frozen
   CLI/pkg·이전 전쪽 TSV는 보존한다. 실행 중인 같은 작업을 중복 시작하지 않는다.
2. 새 #8542의 실제 3쪽 줄·칸 소속·종료와 기존 정상 대조군을 함께 확인한다. 리베이스 전후
   #7375·#7640·수식 진단의 Native/fresh WASM 전쪽 TSV를 비교하고 0.1%p 하락 기준을 적용한다.
3. 관련 전쪽 90%·쪽수 일치의 선행 조건 뒤 새 관계 검사의 수정 전 FAIL을 확인한다.
   표시 glyph·원문·소유·누락/중복과 원본의 독립 전진을 검사한다. 일반 face·legacy·
   인용 밖 미분은 반례다. 98% 미만 3쪽의 배치를 절대 px로 고정하지 않는다.
4. 공통 보정의 Native/fresh WASM 전3쪽과 원본 11월·재저장본·과학 시험지·사각형 대조군을
   실제 출력하고 대표 review/overlay를 직접 판독한다. 개선 전후의 정확한 source/hash를 남긴다.
5. 필요한 집중 검사·Native Skia 필수 3종·전체 nextest·Rust lint/build·manifest/unit tier와
   최종 조판 head의 편람 Native 383쪽을 완료한다. 최종 제출 전 현재 개선과 새 devel 기능을
   모두 만족해야 하며 원격 PR은 새 승인을 받는다.

로그·중간 JSON·진단 스크립트는 ignored output에 두고, 단계가 완료되면 코드·정식 회귀·
최종 TSV·대표 이미지·짧은 기록을 한국어 로컬 커밋으로 정리한다. #7445 부모는 열린
subissue보다 먼저 종료하지 않는다.

## 2026-11-06 공통 투영 보정과 독립 원점 검사

현대 HYhwpEQ 명령의 삼각형은 공통 layout에서 Haansoft Batang U+2206으로,
인용 작은따옴표는 HYhwpEQ U+E050으로 발행한다. 여섯 크기의 독립 저장 폭을 보존하며,
혼합 인용 문자열은 기존 가로 합성을 재사용한다. 일반 math face·legacy와 인용 밖 미분은
기존 경로를 유지한다. Native/SVG/Canvas가 읽는 같은 LayoutBox에서 보정한다.

- 새 base 고정 rlib의 최종 검사 본문은 5 PASS / 2 FAIL, 후보 rlib는 7 PASS다.
  두 글리프 선택의 실제 실패를 확인했다. 고정 좌표 검사는 수정 전부터 통과하므로
  좌표 자체를 이번 보정으로 개선했다고 주장하지 않고 기존 정상 원점의 후퇴를 보호한다. 정식 파생 suite의
  `native-skia` 실행도 7 PASS이며, 일반 face와 혼합 인용 대조를 포함한다.
- 후보 runtime의 Native 집중 66 PASS, 필수 lib 4,039 PASS / 12 ignore,
  PNG 2 PASS, 직접 PDF 22 PASS다. 이후 변경은 같은 runtime을 읽는 검사 본문뿐이다.
- 새 base 대비 Native 6조건 102쪽은 전체 90% 이상·쪽수 일치·누락0이다.
  0.1%p 초과 하락0이며 가장 큰 raw 하락은 과학 시험지의 0.00181%p다.
- 수식 진단의 Native 전3쪽은 100 / 99.98115 / 99.95925%, fresh WASM은
  100 / 99.99209 / 100%다. 양쪽 직접 판독 뒤 3쪽의 인용문자·삼각형·후속 A와
  별도 ABC 대조군 원점을 독립 Print의 72pt→96px 환산값 ±2px로 보호했다.
  보정 전 98% 미만 결과를 px 추가의 선행 근거로 쓰지 않았다.
- 실제 바뀐 11월 원본·재저장본 11·17·21쪽의 삼각형 주변을 추적했다.
  새 Native 출력의 나머지 정상 대조군 PNG는 이전 producer와 해시 대응도 기록했다.
  점수는 새 전쪽 TSV로 산출했으며 이전 TSV를 새 결과로 재인용하지 않았다.

후보 fresh WASM 전102쪽과 Native 전102쪽의 비교를 완료했다. 양쪽 모두 90% 이상,
쪽수 일치·누락0·0.1%p 초과 하락0이다. fresh WASM의 가장 큰 raw 하락은 과학 시험지의
0.00205%p다. 실제 root pkg·Studio public·새 브라우저 초기화 응답의 JS/WASM hash도
동일함을 확인했다. 대표 review/overlay 판독과 독립 원점·글리프 소유 확인을 완료했다.

## 파생 검사 helper 중복 로딩 보정과 최종 로컬 게이트

첫 전체 nextest는 12,649 PASS였으나 이어진 Native feature all-target Clippy에서
`duplicate_mod`를 검출했다. #8153 기호 검사 두 개가 재배정된 같은 harness에서
동일 helper를 각각 `mod`로 읽은 것이 원인이다. 런타임이나 assertion을 바꾸지 않고,
파생 manifest 생성기가 소비 harness의 루트에 helper를 한 번 공급하도록 보정했다.
검사가 같은 suite 또는 서로 다른 suite에 배정되는 경우와 무관한 suite를 Node로 검증했다.
CI workflow·suite policy·helper 구현·두 검사의 assertion 본문은 변경하지 않았다.

- manifest·Cargo 격리 Node 검사 26 PASS, Native 집중 검사 11 PASS.
- Native feature all-target·default·WASM·workspace all-target Clippy와 workspace build,
  fmt·manifest·unit tier 검사 모두 exit 0.
- 보정된 상태에서 다시 수행한 최소 글꼴 전체 nextest는 **12,649 PASS / 0 FAIL / 45 skip**,
  테스트 실행 422.229초다. 컴파일 포함 명령은 889.113초이며 둘을 구분한다.
- 실행 전후 source·검사·스크립트 6,415개 hash와 HEAD 불변을 확인했다.
  `glyph-final-v2-gates-state.json` 및 `glyph-final-v2-full-nextest-result.json`이 실행 정본이다.
- 위에서 완료한 Native Skia 필수 3종은 같은 런타임 source의 4,039+2+22 PASS다.
  이후 차이는 검사·주석과 helper 로딩뿐이며, 새 Native 집중 검사와 Clippy도 통과했다.
  이 대응을 정식 producer 증거에 추가하고 과거 실패 로그도 보존한다.

이 단계의 최종 TSV·대표 이미지·검증 기록을 별도 증적 커밋으로 정리한다. 최종 제출 head의
편람 gate와 원격 통합은 미완료다. 로컬 단계 완료를 이슈 종료나 PR 준비 완료로 표현하지 않는다.
실행 JSON·로그·중간 확대 그림은 ignored output에 보존한다.
