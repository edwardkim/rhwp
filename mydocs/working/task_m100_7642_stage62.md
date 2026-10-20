# 이슈 #7642 Stage 62 — 표시 글꼴·LADDER·전쪽 배치의 원인 보정

Issue: #7642

## 구현 전 근거와 검증 범위

Stage 61의 Native source는 74d795e0d이며 빌드·fmt·Native Clippy와
기호 진단은 통과했다. 90% 이상이어도 실제 ±/∓ 모양 차이가 남았다.
전체 55개·853쪽 및 관련 회귀의 관계형 전환은 계속 원래 범위다.

이번에는 다음 원인의 생산·측정·실제 배치 경로를 모은 뒤 수정한다.

1. **±/∓의 측정과 표시 face 불일치.** HY 전진폭을 쓰면서 Symbol은
   LM으로 그린다. 같은 메트릭·여백에서 HY 표시만 바꾼 진단이 실제
   Print 모양에 가까워졌다. FontGlyph의 공통 출력 경로와 outline이
   없는 글리프의 row 여백 처리도 함께 대조한다. 사설 글꼴 파일·윤곽은
   소스에 넣지 않고 올바른 설치 글꼴과 정상 대체 face를 사용한다.
2. **LADDER의 Plain 행렬 fallback.** 원본 비-TAC 수식은 실제 LADDER다.
   괘선과 마지막 행 정렬이 빠졌고 전체 한 쪽이 52.11970%다.
   8/12/16/24pt에서 trailing/leading empty cell, 열 수·행 수,
   LADDER/SLADDER와 일반 MATRIX 대조군을 한컴 Save/Print로 확인한다.
   선언 폭을 clamp하거나 특정 문서·크기의 예외로 대체하지 않는다.
3. **미주 변형 HWPX의 15쪽 줄·단 경계.** 문23 마지막 본문이 추가 줄로
   나뉘고 문26의 일부가 Print보다 먼저 오른쪽 단으로 넘어간다.
   같은 줄에 놓여야 할 텍스트·TAC 수식의 소유, 원본 저장 LineSeg,
   공통 측정 결과와 실제 배치의 소비 분기를 추적한다.
4. **sample16의 이어짐과 줄 나눔.** 64쪽 중 7쪽이 미달이다.
   대표 22쪽에서 이전 쪽의 본문이 남고 제목·뒤 내용의 배치도 다르다.
   저장본의 유효한 줄 정보·문단 속성·실제 글꼴 공급과 조판 세대를
   확인한 뒤 수정한다. 좌표 허용치나 페이지 기준값을 먼저 바꾸지 않는다.

## 실행 순서와 판정

이전 source의 전쪽 작업과 현재 고정 Native의 대표 비교는 실행 중인
소유 프로세스를 확인하며 결과를 수집한다. 새 source의 fresh WASM은
위 배치 원인의 구현 방향과 Native 직접 판독을 확인한 뒤 빌드한다.
이전 package나 진단 투영을 새 코드의 증적으로 보고하지 않는다.

변경 후 같은 원본·독립 Print PDF에서 Native/fresh WASM의 전쪽 TSV,
미달/대표 경계 review와 수식 확대 그림을 산출한다. 최신 head의
관련 모든 페이지가 90% 이상이고 알려진 결함이 해소된 뒤 #1139 전체와
나머지 관련 회귀를 원본 소유·순서·보존·포함·겹침 관계로 변경한다.
수정 전 실패·수정 후 통과 및 정상 대조군을 기록하고, 전체 nextest와
필수 lint·Native Skia를 끝낸다. PR/CI/merge/후속 처리는 그 뒤에 진행한다.

TSV·로그·중간 JSON과 진단 SVG는 ignored output에만 둔다. 최종 수용
근거로 사용한 입력·Print PDF와 대표 PNG는 정식 보존 경로에 포함한다.
아직 구현·전쪽 통과·관계형 전환을 완료한 단계가 아니다.

## 첫 보정: 측정 face를 표시에도 전달

source_font_symbol은 ±/∓를 기존 전진폭·기준선의 FontGlyph로 전달하고
HYhwpEQ를 우선 표시 face로 요청한다. 설치되지 않은 환경의 공개 face는
대체 목록에 남긴다. 폰트 바이트·사설 윤곽·좌표 상수를 코드에 추가하지 않았다.
row_children은 outline 유무와 관계없이 이 FontGlyph의 같은 연산자
메트릭을 소비한다. 이를 빠뜨리면 표시 변경 뒤 자동 여백이 사라지므로
기존 Symbol과 같은 행 경계를 유지하도록 함께 보정했다.

현재 source 변경은 아직 빌드·Native/fresh WASM으로 검증하지 않았다.
앞 절의 진단 투영은 이 source의 실행 결과가 아니다. 다른 배치 원인의
분석·보정을 모으고 실제 출력으로 다시 판정한다.

## LADDER 구현의 독립 근거와 소비 경로

한컴 공개 수식 사양 1.3의 10쪽은 LADDER의 마지막 행을 결과 수의
나열로 설명하며, SLADDER는 나머지를 오른쪽에 표시하는 템플릿이다.
근거: https://cdn.hancom.com/link/docs/%ED%95%9C%EA%B8%80%EB%AC%B8%EC%84%9C%ED%8C%8C%EC%9D%BC%ED%98%95%EC%8B%9D_%EC%88%98%EC%8B%9D_revision1.3.pdf

48개 독립 입력은 2024 Save job d2c2367f-e5ba-4461-a88e-ff32d6239113,
Print job 479ba0e8-fc3c-4288-91b8-b4374489b9bc로 완료됐다.
13.0.0.3901, PrintMethod 0, frame_print_to_pdf_ex_one_up이다.
Print SHA-256은 b713f227dfe5a0b2d8197dabf7d44d5c8729dfd1e33c5161a902d58184c6faab이다.
입력·저장본·Print·문자 상자는 ignored stage62-ladder-contract-probe에 있다.

같은 세 열 LADDER의 폭은 최대 셀 폭에 0.2em을 더한 열 슬롯 3개다.
8/12/16/24pt의 저장 폭은 2730/4320/5910/8640 HWPUNIT이며,
두 열은 같은 슬롯 2개다. 결과 행의 첫 수는 두 번째 열에 놓이고
끝의 빈 칸은 자리표시다. 결과 행을 일반 행렬처럼 첫 열부터 그리지 않는다.
한 행만 있는 LADDER는 그 행에도 ㄴ자 괘선을 표시한다.
SLADDER 첫 행의 세 번째 칸은 표시하지 않고, 뒤 행의 나머지는
두 번째 값 칸 다음 점 칸을 지나 독립 나머지 칸에 표시된다.

행 피치는 기존 논리 글줄 높이에 0.15em을 더한 값이다.
8/12/16/24pt에서 기존 글줄 높이 825/1200/1575/2400 HU가
945/1380/1815/2760 HU의 피치를 만들고, 세 행 저장 높이와 일치한다.
숫자 표시 글꼴의 자연 EM 상자와 논리 예약 프레임을 혼동하지 않는다.
복합 셀은 독립 논리 프레임이 있는 구조는 그 프레임, 나머지는
실제 셀의 기준선 위·아래 점유를 사용한다. 복합 셀 Print는 아직 미검증이다.

소비 경로는 parser의 LADDER/SLADDER 스타일 보존 → layout_matrix →
layout_ladder의 셀 폭·논리 줄 프레임·공유 괘선 → Matrix의 셀/표시 상자 →
SVG·Canvas·Skia의 기존 FontGlyph 윤곽 경로다. 출력 backend에서
셀 위치나 괘선 좌표를 재계산하지 않는다. paint JSON과 golden 덤프는
새 스타일 이름을 읽도록 한정 수정했으며 golden 기대값을 갱신하지 않았다.

이 구현은 실제 최신 Native/fresh WASM 출력 검증 전이다. 원본 비-TAC
수식과 48개 진단 및 일반 MATRIX 대조를 직접 대조한 뒤 판단한다.

## 소스 진단 결과

48개 중 일반 MATRIX 4개를 제외한 44개는 수정 전 parser에서
Plain으로 접혔다. 같은 소스 모듈의 진단 실행은 수정 전 exit 1,
수정 후 exit 0이다. 기존 이항 기호·인용·첨자 진단도 통과했다.
진단 컴파일의 경로 오류를 실제 결함 재현으로 세지 않았으며,
최종 수정 전 실행에서 스타일 손실 44건을 확인했다.

실제 layout 모듈의 최소 harness로 동일 HYhwpEQ 조건을 실행했다.
한컴 저장본에서 읽은 네 크기의 폭/높이와 계산한 값이 모두 같고 exit 0이다.
2730/2835, 4320/4140, 5910/5445, 8640/8280 HU다.
harness의 환경 변환 함수는 96dpi/7200HU의 원래 변환과 같으며
정식 제품 빌드나 페이지 Visual Sweep을 대체하지 않는다.
정식 회귀와 기대값은 아직 변경하지 않았다.

cargo fmt --all -- --check와 git diff --check는 통과했다.
이제 고정 target/pr-review의 Native 빌드와 48개 Print·원본 비-TAC·
연산자 대조를 실행한다. 실제 exit code와 원본 page review를 확인하기
전에는 90% 통과나 결함 해결을 선언하지 않는다.

## Studio 소비 경로 보정

CanvasKit의 matrix 분기는 plain 이외의 모든 스타일을 세로 괄호로
그리는 기존 조건이었다. 새 ladder/short_ladder는 공유 셀 안에
괘선이 있으므로 paren/bracket/vert에서만 별도 괄호를 그리도록 바꿨다.
공개 TypeScript 스타일 타입에도 두 값을 추가했다.
`rhwp-studio/node_modules/.bin/tsc --noEmit -p rhwp-studio/tsconfig.json`은
exit 0이다. Rust 빌드 시작 뒤 Rust 소스는 바꾸지 않았다.
빌드 source 00bac93e4와 Studio 소비 경로 변경을 구분해 기록한다.
Studio 실제 fresh WASM 및 CanvasKit 출력은 아직 미검증이다.

## #1139 전체 범위의 원본 대응 재확인

실제 77개 test 함수의 파일 열기와 이름 있는 helper 호출을 추적해
ignored stage62-issue1139-test-input-audit.tsv/json을 산출했다.
73개는 원본 파일 검사, 4개는 빈 문서 생성 API의 합성 검사다.
테이블로 지정한 원본도 별도로 확인했으며 파일명이 미확인인 행은 없다.
이 후보 원장은 실행이나 관계형 전환 완료를 의미하지 않는다.

문28 CQRT 검사는 9월 입력이 아니라 3-11월_실전_통합_2022.hwp다.
이전에 9월 입력에서 해당 문자열을 찾은 진단을 이 검사의 근거로
사용하지 않는다. 올바른 원본의 미주 s0:p318:ci0:note3에서
control 0은 g(theta), control 1은 CQRT 식이고 둘 다 저장 위치 0과
composed line 0을 공유한다. Native source 74d795e0d의 소유 진단에서
CQRT는 가상 문단 804/control 1 및 같은 원본 NoteControlRef로 확인됐다.
기존 고정 x/y 영역과 SVG의 CQRT/g 글자 문자열 대신 이 원본 소유와
기준선·내용 순서로 검사해야 한다. 정식 검사는 시각 선행 조건 뒤 변경한다.

비-TAC 원본의 font는 HancomEQN이며 MCP 기준 PDF도 HancomEQN이다.
48개 새 진단의 HYhwpEQ와 폰트명을 동일하다고 추정하지 않는다.
설치된 두 폰트의 수학 PUA 숫자는 UPM 1024, 전진폭 512이지만
ASCII 숫자의 전진폭은 서로 다르다. 관련 메트릭은 ignored
stage62-ladder-font-comparison.json에 보존했고 사설 폰트는 추가하지 않았다.
최신 원본 대조에서 이 차이를 함께 판독한다.
