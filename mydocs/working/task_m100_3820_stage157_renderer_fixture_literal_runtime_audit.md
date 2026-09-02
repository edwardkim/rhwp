# Stage 157: renderer fixture literal 실행 gate 감사

## 목적

`src/renderer`에서 sample/fixture/문서명/문장 literal이 실제 layout 또는 pagination
분기에 사용되는지 전수 분리한다. 실행 gate가 남아 있으면 source·style·geometry의 공통
계약으로 교체한다.

## 분석 범위

- `src/renderer`의 `if`, `match`, predicate 함수에서 sample, fixture, issue, manual,
  appendix, 2025, 2022, 2024 식별자를 사용하는 경우
- 앞 Stage에서 제거한 BCP literal과 QA allowance 계열의 재도입 여부
- 테스트 전용 fixture와 주석의 실측 근거는 production 실행 gate와 구분한다.

## 분류 원칙

- 문서의 행·열·크기·문장·페이지 index로 layout을 바꾸면 문서 지문이다.
- HWP/HWPX profile, line segment, control kind, style, 실제 geometry로 판정하면
  공통 계약이다.
- 검증 fixture를 여는 테스트와 과거 실측 주석은 실행 gate가 아니다.

## 금지 조건

- 주석이나 테스트 이름만 삭제해 실행 지문 감사가 끝난 것처럼 처리하지 않는다.
- 형식 semantic인 PUA 표시 매핑을 layout hardcode와 혼동하지 않는다.
- 분석 문서만 커밋하지 않는다.

## 완료 기준

- runtime fixture literal 후보를 모두 분류한다.
- 문서 지문이 확인되면 하나의 일반 구현으로 교체한다.
- 코드와 결과 보고서를 같은 Stage 커밋으로 남긴다.

## 상태

완료.

## 실행 gate 감사 결과

- Stage 156에서 제거한 BCP 문장 literal과 QA tail allowance 식별자는 `src/renderer`의
  production 경로에 남아 있지 않다.
- sample, fixture, issue, manual 식별자는 테스트, 진단 출력, 과거 근거 주석에만 남아
  있다. PUA 문자 매핑은 HWP 형식 semantic이고 문서 지문으로 layout을 바꾸는 분기가 아니다.
- 남은 production 후보는 HWP3-origin legacy bullet 재조판의 `1.04` 가용 폭 보상이었다.
  특정 문서명은 쓰지 않았지만, 글꼴 이름 목록과 고정 배율로 한컴 재조판 공백 차이를
  우회하고 있었다.

## 원인과 공통 계약

- 저장 LINE_SEG의 공백은 글꼴 고유 advance를 보존할 수 있다.
- 한컴이 폭 변경 뒤 새 LINE_SEG를 만들면 반각 공백 advance로 다시 조판한다.
- 기존 `1.04`는 이 공백 차이를 셀 폭 확대라는 간접 보상으로 처리했다. 따라서 공백 수와
  글꼴 metric이 달라지면 동일한 의미의 문단에도 오차가 달라졌다.

## 구현

- `hancom_regenerated_space_width`는 현재 style의 저장 공백 폭과 한컴 재조판 반각 공백을
  직접 비교한다. 재조판 공백이 실제로 더 넓은 경우만 값을 반환한다.
- stale cell LINE_SEG 재생성과 LINE_SEG 부재 legacy bullet 재조판이 같은 metric을 공유한다.
- `composer`는 재조판 모드에서만 텍스트 내 공백 개수만큼 실제 metric 차이를 더한다.
  셀 폭, 페이지, 행 수, 문장 또는 글꼴 이름을 기준으로 한 allowance는 사용하지 않는다.
- 고정 `1.04`, `HY신명조`/`한양신명조`/`휴먼명조` 목록, 12pt 글꼴 크기 조건을 제거했다.

## 검증

- 이번 Stage에서는 사용자 요청에 따라 코드와 source 계약 정리까지만 수행했다.
- 별도 build 또는 test는 실행하지 않았다.

## 결과

문서 지문형 실행 gate와 글꼴명 기반 폭 allowance를 제거하고, 저장 metric과 재조판 metric의
차이로만 줄 분할 폭을 결정하게 했다.
