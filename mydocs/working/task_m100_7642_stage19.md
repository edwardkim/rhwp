# 이슈 #7642 — 구두점 슬롯의 측정과 실제 표시 공유

이전 단계: [HFT 구두점 슬롯](task_m100_7642_stage18.md), source `83f790957`.

## 현재 관측과 수정 근거

이전 단계의 math8 Native는 94.16183%지만 완성된 표시 계약의 증거가 아니다. 실제 SVG의
`(가)/(나)` 두 괄호는 영문 메트릭으로 재면서 `font-family=HY신명조`로 표시한다.
독립 Print의 두 괄호는 영문 프로그램 T2, 한글은 T3다. 원본 글자 모양의 영문 face는
`한양신명조`이므로 서로 다른 프로그램을 같은 결과로 간주할 수 없다.

원인 경로는 `resolve_single_char_style → ascii_punct_latin_slot → char_width_decision`과
`compose_lines → ComposedTextRun.lang_index → text_style → paint`의 차이다.
기존 구두점 run 분리는 저장 줄이 없는 문단의 비ASCII 구두점에만 적용된다. 저장 줄의
ASCII 괄호는 원래 한글 run에 남고 폭만 영문 값으로 덮어쓴다. 앞 단계의 측정 개선을 유지하되
측정과 실제 표시가 같은 슬롯 결과를 소비하도록 공통 구성 결과를 보정해야 한다.

## 계획과 반례

- explicit 구두점 슬롯 정책을 따르는 run을 구성 단계에서 나눈다. 본문·기호·공백의 기존
  carry, 원문 scalar 소속·순서·개수, 컨트롤 소유와 PUA 표시 변환을 보존한다.
- 저장 줄의 ASCII 구두점도 실제 영문 슬롯을 소비하게 한다. 비ASCII 구두점의 저장 face
  보존과 기존 재조판 계약을 구분하고 기존 정책을 적용하지 않는 run을 바꾸지 않는다.
- 기존 큰 composer 파일은 책임 경계별 source 조각으로 분리해 수정 파일을1000줄 이하로
  유지한다. 모듈/함수 경로와 기존 계산 계약을 보존하고 토큰 비교·컴파일·분류 검사로 확인한다.
- math8 원본과 eq002 HWP/HWPX·국어1p, 기존 LegacyLatin과 같은 face/미확인 폭/비치환
  TTF의 반례를 대조한다. 동일 문단의 SVG 글꼴·원점, 페이지 전체 TSV와 확대 판독을 연결한다.
- #7593 원본과 최소 입력도 최신 Native/fresh WASM Print 대조 후 AB 장식의 소유·순서와
  모든 조상 transform을 보존하는 검사로 교정한다. 단일 SVG `<text>AB</text>` 가정은
  글자별 출력의 의미를 검사하지 못하며 장식 폭 비율의 독립 계약은 유지한다.

## 이번 브랜치의 처리 범위

전체 회귀 실패90개는 사용자의 지시에 따라 모두 현재 브랜치에서 보정한다. 미달 원본이나
실패 검사를 제거·별도 이슈로 이관하지 않는다. 실제 출력의90% 이상 및 확대 판독을 먼저
충족하고 필요한 검사만 소유·순서·보존·포함·겹침 관계로 바꾼다. 정확한 기존 검사는
유지하고 구현을 고친다. 평균값·단순 점수·CI 통과로 잔여 결함을 면제하지 않는다.

추가 #7593 Native 확대에서 보기 ㄴ의 AB 수식이 독립 Print보다 본문 기준선에 가깝게
놓이는 차이가 보인다. 원본97.08779%·최소98.48485% 통과로 이 위치 차이를 면제하지 않는다.
장식 폭 검사 교정 전에 실제 수식 글리프의 기준선과 저장 프레임의 관계를 추적하고 보정한다.

구두점 run 분리와 source 분리는 구현했고 새 Native를 검증할 예정이다. 회귀 변경과 최종
시각 판정은 아직 실행 전이다. 앞 단계의 fresh WASM
빌드는 exit0, root pkg/Studio JS·WASM 해시 일치 확인 후 package를 고정했다. source83의
math8·#7593 Print 비교는 후속 표시 보정 전의 증거로 구분한다. 이어서 source 변경을 시작한다. Cargo 공유 target은 동시에 사용하지 않으며 ignored
output의 로그·TSV·JSON·nextrun·진단을 Git에 포함하지 않는다.


## 공통 구성 결과 구현

`punctuation_slots::resolve_run_slots`를 저장/재조판 구성의 같은 위치에서 소비한다.
해소 스타일의 explicit 영문 구두점 정책이 켜진 run의 ASCII 구두점은 언어 슬롯1로 나누며,
공백·본문의 기존 소속은 보존한다. 비ASCII 구두점은 저장 run의 face를 유지하고 저장 줄이
없는 재조판에서만 기존 계약을 적용한다. 원문 text와 글자 수·줄 원점·컨트롤 소속을 바꾸지
않으며 PUA·겹침 문자 표시 변환 전에 언어 경계만 나눈다.

기존 composer의8개 책임 조각은 `include!`로 같은 모듈 어휘 범위에 둔다. 함수/타입의
공개 경로·형제 helper 접근·기존 테스트 모듈을 바꾸지 않는다. 분리 직후 재조합한 source가
이전 커밋과 바이트 단위로 같음을 확인했고, 변경 대상 composition을 제외한7개 조각은
원본 SHA-256을 유지한다. 모든 수정/신규 파일은1000줄 이하다. 이 확인은 실행 검증을
대체하지 않으며 새 컴파일과 필요한 검사로 다시 확인한다.

## AB 인라인 표시 원점의 추가 원인

`layout_for_paint`는 `inline_baseline_px / scale - intrinsic.baseline`을 루트 y에
다시 더한다. 실제 배치는 이미 저장 프레임의 기준선으로 bbox 원점을 정했으므로, 이 추가값은
보기 ㄴ의 큰 저장 프레임에 있던 글자와 본문 기준선의 차이를 상쇄한다. 기존 검사는
`bbox.y + layout_box.baseline`만 보아 루트 `layout_box.y`의 실제 소비를 놓친다.

원본의 세 AB 수식은 모두 BaseUnit1100/기준선85%지만 앞 두 프레임은1125HU, ㄴ은2580HU다.
독립 Print의 ㄴ 글자는 뒤 본문보다 위에 표시된다. 장식 글자의 독립 논리 프레임과 저장
예약 프레임을 구분하고 같은 관계가 실제 paint까지 유지되도록 후속 보정한다. 프레임을
작은 값으로 덮거나 저장 소속을 바꾸어 기준 출력을 맞추지 않는다.


## 공통 구성 Native 결과

새 Native 빌드는4분27초, exit0이다. binary SHA-256은
`c277dfc74a745ee02c058b446853f99dee1f5c50e4e7bba16ddb2e85f98102db`다.
math8 **94.14918%**, eq002 HWP/HWPX 각94.81146%, 국어1p98.90252%이고 모두 누락0이다.
Native 실제 SVG의 네 괄호가 모두 영문 `한양신명조` face를 사용한다. 전쪽 TextRun의
내용을 순서대로 연결하면 보정 전후가 동일하며 run12개→20개는 언어 경계의 분리다.
줄/개체의 소유나 본문 내용의 추가·삭제로 점수를 맞추지 않았다.

포맷/`git diff --check`와 고정 base의 source-unit-tier4198개 분류가 통과했다.
source 분리의7개 비변경 조각은 원본 해시를 유지하며 새 helper를 포함한 모든 수정 파일은
1000줄 이하다. `representative-native/exam-math-8-shared-slots/review/review_001.png`와
`math-region-3x.png`를 생성하고 직접 판독·사용자 표시했다.

이전 커밋83f의 fresh WASM도 빌드 exit0, root pkg/Studio JS·WASM 해시 일치다.
그 source의 math8=94.16183%, #7593 원본97.08779%, 최소98.48485%로 같은 Native와
일치한다. 이는 측정 보정 단계의 증거이며 이번 표시 슬롯 보정의 fresh WASM 통과를 뜻하지
않는다. 이후 실제 표시 원점 보정까지 수행한 source에서 fresh WASM을 다시 검증한다.
새 회귀·기대값·golden 변경은 아직 없으며 전체90개 실패의 해결도 아직 완료되지 않았다.
