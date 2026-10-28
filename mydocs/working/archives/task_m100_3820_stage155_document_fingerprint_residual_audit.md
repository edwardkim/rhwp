# Stage 155: 잔여 문서 지문 전수 감사

## 목적

렌더러와 조판 경로에 남아 있는 문서별 hardcode를 전수 식별한다. 행·열·cell 수, 크기,
문단 index, 고정 위치, 특정 샘플 이름을 page-break 또는 layout gate로 사용하는 코드는
일반 source/format 계약으로 대체할 후속 Stage 후보로 분류한다.

## 분석 범위

- `src/renderer`의 HWP/HWPX layout, typeset, table 경로
- `src` 전체의 QA/fixture/sample/appendix/manual 이름 및 numeric shape selector
- Stage 152~154에서 제거한 selector의 재도입 여부

## 분류 원칙

- HWP/HWPX 파일 형식의 공통 enum, bit flag, unit conversion, 표준 page/frame 제약은
  문서 지문이 아니라 형식 계약으로 분류한다.
- 특정 문서의 표 행·열·cell 수, width/height, paragraph/page index, fixture 이름으로
  layout을 바꾸면 문서 지문으로 분류한다.
- 단지 테스트 fixture를 찾는 경로는 구현 gate가 아니므로 별도 기록하되 renderer 수정
  대상으로 섞지 않는다.

## 금지 조건

- 분석 문서만 커밋하지 않는다.
- 아직 형식 계약인지 확인하지 않은 numeric constant를 일괄 제거하지 않는다.
- Stage 154의 PageHide blank-page source ownership을 되돌리지 않는다.

## 완료 기준

- 실행 경로의 잔여 후보를 문서 지문/형식 계약/테스트 전용으로 분류한다.
- 실제 문서 지문이 있으면 한 개의 일반화 가능한 계약을 선택해 구현한다.
- 코드와 분석·결과 문서를 같은 Stage 커밋으로 남긴다.

## 1차 감사 결과

### 일반화 대상

`src/renderer/typeset.rs`와 `src/renderer/pagination/engine.rs`에는 같은 문장을 비교하는
`sample16` tail 분기가 있었다. 이 분기는 다음 두 가지 입력을 섞고 있었다.

1. 저장 `LINE_SEG`가 본문 하단에서 시작한 뒤 다음 줄의 vpos가 page origin으로 되감기는 경우
2. `LINE_SEG` 자체가 없지만 본문 하단에서 마지막 composed line을 다음 page로 넘겨야 하는 경우

두 경우 모두 문서명이나 문장 자체가 아니라 visible text, line-segment 존재 여부, inline
metadata-only control, stored vpos reset, 현재 frame의 잔여 공간으로 판정할 수 있다. 이
공통 계약을 구현 대상으로 선택한다.

### 형식 계약 또는 테스트 전용

- `hancom_pua.rs`의 PUA code point 표는 layout gate가 아니라 private glyph의 검증된 의미
  매핑이다. 행·열·크기·페이지를 판정하지 않으므로 폐기 대상이 아니다.
- TAC 표의 page-scale threshold, shrink ratio, HWP unit conversion은 형식과 측정 오차의
  공통 계약이다. 특정 fixture 식별자는 주석의 근거일 뿐 실행 predicate에 없다.
- test fixture의 경로·이름과 과거 working report의 상수명은 production renderer gate가 아니다.

### 별도 후속 조사

`composer.rs`에는 `sample16` BCP literal로 마지막 LINE_SEG를 접는 오래된 회귀 hook이 있다.
현재 주석이 설명하듯 동일 IR에서 정상 짧은 마지막 줄과 구별할 source signal이 확인되지
않았으므로, 이번 Stage의 page-break 일반화와 섞지 않는다. 다음 Stage에서 parser 원시
record 또는 한컴 저장 규칙까지 포함해 별도 조사한다.

## 상태

구현과 검증 완료.

## 구현 결과

두 pagination 경로의 문서 literal gate를 아래 공통 규칙으로 교체했다.

1. 저장 `LINE_SEG` 경로는 visible text와 `Field`/`Hyperlink`만 가진 inline metadata
   문단에서, body 하단의 vpos가 다음 줄에서 page origin 또는 page 상단 영역으로
   되감기는지를 판정한다.
2. `LINE_SEG`가 없는 경로는 같은 inline metadata 문단이 frame 하단에 있고 composed
   line이 네 줄 이상이면 마지막 줄만 다음 page로 넘긴다. 고정 `3` 대신
   `line_count - 1`을 사용한다.

제거한 항목은 exact Korean sentence, PUA prefix, `sample16` 함수명, `Some(3)` 고정
line index다. `typeset.rs`와 `pagination/engine.rs`가 같은 구조 계약을 사용한다.

## 검증 결과

1. `cargo build --target-dir target/stage155`: 성공
2. HWP SVG export: `pageCount=383`, `renderedCount=383`
3. HWPX SVG export: `pageCount=383`, `renderedCount=383`
4. 기존 sentence literal, `is_sample16_integrated_db_cluster_tail_paragraph`,
   `sample16_missing_lineseg_tail_break_line` 정적 검색: 실행 렌더러 결과 없음

export는 기존 `LAYOUT_OVERFLOW`/`LAYOUT_TABLE_OVERLAP` 진단을 출력했지만 성공했고,
이번 일반화 전후 2025 편람 두 형식의 페이지 수는 383으로 유지됐다.
