# Task M100-993 Stage 15 계획서

## 1. 목적

Stage 14 판정 결과:

```text
03, 05, 07, 08 케이스에서 성공
```

성공 케이스의 공통 축은 첫 문단 `PARA_TEXT`다.

```text
03 = PARA_TEXT
05 = PARA_HEADER + PARA_TEXT
07 = PARA_TEXT + PARA_CHAR_SHAPE
08 = PARA_HEADER + PARA_TEXT + PARA_CHAR_SHAPE
```

따라서 1페이지 시작 부분 고스트 라인의 최소 원인은 첫 문단 `PARA_TEXT` control marker stream이다.
Stage 15의 목적은 이 `PARA_TEXT` 차이를 구현 가능한 규칙으로 좁히는 것이다.

## 2. 핵심 차이

Stage 12 trace 기준:

```text
oracle:
  PageNumPos/PageHide control marker 뒤에 TAB marker가 나온다.

generated:
  ColumnDef marker 뒤에 TAB marker가 먼저 나오고,
  그 뒤에 PageNumPos/PageHide/Header marker가 나온다.
```

record stream의 상위 `CTRL_HEADER` 순서는 이미 정답지와 같다.

```text
SectionDef -> ColumnDef -> PageNumPos -> PageHide -> Header
```

따라서 이번 단계는 record 순서를 바꾸지 않고, `PARA_TEXT` 안의 marker stream만 다룬다.

## 3. 현재 구현 위치

우선 검토할 소스:

```text
src/serializer/body_text.rs
  - serialize_para_text()
  - push_extended_ctrl()
  - control_char_code_and_id()

src/model/paragraph.rs
  - Paragraph::control_text_positions()

src/parser/hwpx/section.rs
  - HWPX 문단 text/control 파싱 순서
```

현재 `serialize_para_text()`는 `para.text`와 `para.char_offsets`의 갭을 기준으로 control marker를 삽입하고,
탭 문자는 `para.text` 내부 문자로 직접 처리한다. 이번 문제는 첫 문단에서 탭 문자가
PageNumPos/PageHide/Header marker보다 앞쪽으로 직렬화되는 현상이다.

## 4. 수행 절차

1. `mel-001.hwpx` 첫 문단 XML에서 TAB과 `hp:ctrl`의 원천 순서를 추출한다.
2. 현재 IR의 첫 문단 `text`, `char_offsets`, `controls`, `tab_extended`를 덤프한다.
3. 정답 HWP와 generated HWP의 `PARA_TEXT` payload를 byte/code-unit 단위로 나란히 비교한다.
4. 다음 후보를 분리한다.

```text
- HWPX parser가 TAB을 text에 먼저 넣고 control을 뒤에 넣는 문제
- char_offsets가 TAB 위치를 정답지와 다르게 materialize하는 문제
- serialize_para_text()가 control marker와 text tab marker의 우선순위를 잘못 적용하는 문제
```

5. 구현 후보는 첫 문단 특수 처리로 시작하지 않는다. 일반 규칙으로 다음을 검증한다.

```text
control marker stream은 record stream의 control 순서와 text/char_offsets의 위치 정보를 함께 따라야 한다.
동일 위치에 text tab과 page/header control marker가 충돌하면, 한컴 정답지의 우선순위를 적용한다.
```

## 5. 산출물

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/
```

분석 문서:

```text
output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/para_text_marker_contract.md
```

필요 시 판정용 HWP:

| file | 목적 |
|---|---|
| `01_stage14_baseline.hwp` | 기준 파일 |
| `02_oracle_para_text_guard.hwp` | Stage 14 `03` 재확인 guard |
| `03_marker_order_candidate.hwp` | marker stream 순서 후보 |
| `04_tab_position_candidate.hwp` | tab 위치/offset 후보 |

## 6. 성공 기준

```text
1. Stage 14의 03 성공을 구현 규칙으로 설명한다.
2. 고스트 라인을 제거한다.
3. PARA_HEADER/PARA_CHAR_SHAPE 전체 투영 없이 PARA_TEXT marker stream 규칙으로 해결한다.
4. 셀 대각선, gradient, 조직도 셀 높이 개선을 회귀시키지 않는다.
```

## 7. 승인 요청

이 계획으로 Stage 15 첫 문단 `PARA_TEXT` marker stream contract 추적을 진행한다.
