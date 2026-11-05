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
