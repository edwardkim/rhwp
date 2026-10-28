# Stage 158: renderer runtime 계약 잔여 감사

## 목적

`src/renderer` production 경로에서 QA, allowance, reserve, tail, cut, sample, fixture,
manual 이름 또는 특정 문서의 구조를 직접 표현하는 상수와 predicate를 다시 감사한다.
남은 문서 지문이 있으면 source provenance, 저장 line segment, control kind, style,
geometry 중 하나의 공통 계약으로 교체한다.

## 분석 범위

- 테스트 모듈과 진단 문자열을 제외한 runtime `const`, `static`, predicate
- HWPX/HWP5 변환 계보와 실제 저장 geometry를 쓰는 형식 계약
- 페이지, 행, 열, 문장, 문서명, fixture 이름을 쓰는 문서 지문형 gate

## 분류 기준

- HWP/HWPX binary format bit, unit 변환, 폰트 metric, source provenance는 형식 계약이다.
- 일반적인 overlap/overflow 안전 한계는 geometry 계약 후보이며, 대상 문서 구조가
  포함되면 문서 지문으로 본다.
- 특정 fixture 이름, 문장, 행·열 수, page index, 임의 tail allowance는 허용하지 않는다.

## 완료 기준

- production 상수와 predicate 후보를 전수 분류한다.
- 문서 지문이 확인되면 일반 구현과 결과 보고를 같은 Stage 커밋에 남긴다.
- 문서만 커밋하지 않는다.

## 상태

완료.

## 감사 결과

- Stage 156~157에서 제거한 문서명, 문장, 행·열, 페이지 기반 selector는 production
  runtime에서 재발견되지 않았다.
- 남은 `tail` 계열 후보 중 `TAIL_BREAK_OVERFLOW_TOLERANCE_PX`와
  `SAVED_TAIL_VPOS_OVERFLOW_TOLERANCE_PX`는 저장 LINE_SEG의 실제 bottom을 확인한 뒤에도
  임의 px allowance를 더하는 구조였다.
- `16px` 저장-flow 정렬 여유도 같은 source line의 실제 높이를 쓰지 않는 고정값이었다.
- RowBreak, bottom squeeze, endnote log 계열은 각각 다른 source/geometry 계약이므로
  이번 Stage에 함께 바꾸지 않는다. 다음 Stage에서 하나의 계약 단위로 분리한다.

## 구현

- `paragraph_saved_visible_bounds`가 synthetic segment를 제외한 문단의 실제 저장 top/bottom을
  계산한다.
- vpos-reset 직전 tail은 source bottom이 현재 body 안에 있고 현재 flow가 그 source line과
  겹칠 때만, 현재 조판 높이가 source bottom까지 도달하는 데 필요한 정확한 overflow를 쓴다.
- saved-tail 및 footnote-tail 정렬은 고정 16px 대신 저장 line 자체의 높이로 판정한다.
- 고정 `20px`, `128px`, `16px` tail allowance를 제거했다.

## 검증

- 이번 Stage에서는 사용자 지시에 따라 build 또는 test를 실행하지 않았다.

## 결과

tail page-fit은 특정 문서의 작은 px 보상 대신 저장 LINE_SEG가 제공하는 실제 page-bottom
좌표와 현재 flow의 차이로만 결정된다.
