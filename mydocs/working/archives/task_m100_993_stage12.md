# Task M100-993 Stage 12 작업 기록

## 1. 목적

Stage 11에서 셀 대각선은 `BORDER_FILL` diagonal contract로 분리되었다. 그러나 한컴 에디터에서
1페이지 시작 부분에 고스트 선이 남아 있었다.

작업지시자 관찰:

```text
원본 HWPX:
  [쪽 번호 위치][감추기][탭][머리말 양쪽]

rhwp 생성 HWP:
  [탭]{고스트 선}[쪽 번호 위치][감추기][머리말 양쪽]
```

Stage 12는 이 차이를 첫 문단 record bundle에서 직접 분리한다.

## 2. 입력

정답지:

```text
samples/hwpx/hancom-hwp/mel-001.hwp
```

기준 생성물:

```text
output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/04_diagonal_attr_payload.hwp
```

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage12_first_para_control_trace/
```

상세 trace:

```text
output/poc/hwpx2hwp/task993/stage12_first_para_control_trace/first_para_control_trace.md
```

## 3. 생성 파일

| file | 한컴 판정 유형 | 1페이지 고스트 선 | 조판부호 표시 순서 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage12_first_para_control_trace/01_stage11_positive.hwp` |  |  |  |  |  | Stage 11 positive 기준 |
| `output/poc/hwpx2hwp/task993/stage12_first_para_control_trace/02_oracle_first_para_char_shape.hwp` |  |  |  |  |  | 첫 문단 `PARA_CHAR_SHAPE`만 정답지 투영 |
| `output/poc/hwpx2hwp/task993/stage12_first_para_control_trace/03_oracle_first_para_head_text_char_shape.hwp` |  | 제거 |  |  |  | 첫 문단 `PARA_HEADER/PARA_TEXT/PARA_CHAR_SHAPE` 투영 |
| `output/poc/hwpx2hwp/task993/stage12_first_para_control_trace/04_oracle_top_level_ctrl_headers.hwp` |  |  |  |  |  | 첫 문단 상위 `CTRL_HEADER` payload 투영 |
| `output/poc/hwpx2hwp/task993/stage12_first_para_control_trace/05_oracle_header_subtree.hwp` |  |  |  |  |  | 첫 문단 머리말 subtree 투영 |
| `output/poc/hwpx2hwp/task993/stage12_first_para_control_trace/06_oracle_first_para_bundle.hwp` |  | 제거 |  |  |  | 첫 문단 bundle 전체 투영 |

## 4. 핵심 비교

첫 문단의 상위 `CTRL_HEADER` record 순서는 정답지와 생성물이 같다.

```text
SectionDef -> ColumnDef -> PageNumPos -> PageHide -> Header
```

따라서 고스트 선/조판부호 순서 차이는 단순한 `para.controls` 순서 문제가 아니다.

정답지와 생성물의 첫 문단 `PARA_TEXT` marker 위치가 다르다.

```text
oracle:
  PageNumPos/PageHide control marker 뒤에 TAB marker가 나온다.

generated:
  ColumnDef marker 뒤에 TAB marker가 먼저 나오고,
  그 뒤에 PageNumPos/PageHide/Header marker가 나온다.
```

또한 첫 문단 `PARA_CHAR_SHAPE` run도 다르다.

```text
oracle:
  pos=0:shape=11
  pos=40:shape=7

generated:
  pos=0:shape=11
  pos=16:shape=11
  pos=24:shape=7
```

## 5. 판정 포인트

```text
02가 고스트 선을 제거하면:
  원인은 첫 문단 PARA_CHAR_SHAPE offset/run materialization이다.

03이 고스트 선을 제거하고 02가 실패하면:
  원인은 PARA_TEXT control char stream 또는 PARA_HEADER tail과 결합된 문제다.

04가 고스트 선을 제거하면:
  원인은 상위 CTRL_HEADER payload 차이다.

05가 고스트 선을 제거하면:
  원인은 Header CTRL_HEADER/LIST_HEADER/하위 문단 subtree materialization이다.

06에서만 고스트 선이 제거되면:
  첫 문단 bundle 내부의 복합 계약 문제로 보고 다음 stage에서 06과 01 사이를 다시 분해한다.
```

## 5.1 한컴 판정 결과 해석

작업지시자 판정:

```text
03, 06 케이스에서 고스트 라인 제거 확인
```

이 결과는 다음과 같이 해석한다.

```text
1. 06은 첫 문단 bundle 전체 투영이므로 양성 결과가 나올 수 있는 상위 케이스다.
2. 03은 첫 문단 PARA_HEADER/PARA_TEXT/PARA_CHAR_SHAPE만 투영한 제한 케이스다.
3. 따라서 현재 확인된 최소 양성 후보는 03이다.
4. 고스트 라인의 직접 원인은 상위 CTRL_HEADER 순서나 Header subtree 단독 문제가 아니라,
   첫 문단의 PARA_HEADER/PARA_TEXT/PARA_CHAR_SHAPE 결합부에 있다.
```

특히 trace에서 확인한 차이와 맞물려 보면 다음 축이 우선 후보가 된다.

```text
- PARA_TEXT 안의 control marker stream에서 TAB 위치가 앞당겨진 점
- PARA_CHAR_SHAPE run offset이 정답지와 다르게 materialize된 점
- PARA_HEADER size/tail이 정답지와 다른 점
```

다음 단계에서는 03을 다시 쪼갠다.

```text
1. PARA_HEADER 단독
2. PARA_TEXT 단독
3. PARA_CHAR_SHAPE 단독은 Stage 12의 02 결과와 대조
4. PARA_TEXT + PARA_CHAR_SHAPE
5. PARA_HEADER + PARA_TEXT
6. PARA_HEADER + PARA_CHAR_SHAPE
```

목표는 첫 문단 전체 bundle을 덮어쓰는 것이 아니라, HWPX -> HWP 저장기에서 재현해야 하는
정확한 control marker/materialization 규칙을 찾는 것이다.

## 6. 실행한 검증

```text
cargo check
cargo run --bin rhwp -- hwp5-first-para-control-probe \
  samples/hwpx/hancom-hwp/mel-001.hwp \
  output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/04_diagonal_attr_payload.hwp \
  --out-dir output/poc/hwpx2hwp/task993/stage12_first_para_control_trace \
  --section 0
```

결과:

```text
success
```

## 7. 추가 관찰: 글자 그림자 효과 저장 누락

작업지시자 추가 관찰:

```text
글자의 그림자 효과가 HWP 저장 결과에 반영되지 않는다.
```

이 문제는 1페이지 고스트 선, 셀 대각선, 표 높이 문제와 다른 `CharShape` 축으로 분리한다.

현재 코드 확인 결과:

```text
src/model/style.rs
  - CharShape에 shadow_type, shadow_offset_x, shadow_offset_y, shadow_color 필드가 있다.

src/parser/doc_info.rs
  - HWP5 CHAR_SHAPE parser는 shadow_type, shadow offset, shadow color를 읽는다.

src/serializer/doc_info.rs
  - HWP5 CHAR_SHAPE serializer는 shadow_type bit와 shadow offset/color를 쓴다.

src/parser/hwpx/header.rs
  - HWPX <hh:shadow>에서 type과 color는 읽지만 offsetX/offsetY는 읽지 않는다.
```

`mel-001.hwpx`에는 실제로 그림자 offset이 명시되어 있다.

```xml
<hh:shadow type="CONTINUOUS" color="#CCCCCC" offsetX="8" offsetY="8"/>
```

따라서 현재까지의 1차 원인 후보는 다음과 같다.

```text
HWPX header parser에서 shadow offsetX/offsetY가 IR로 들어오지 않는다.
그 결과 HWP5 serializer는 shadow offset을 0으로 저장하고,
한컴 에디터에서는 원본과 같은 그림자 효과가 나오지 않는다.
```

단, `type="CONTINUOUS"`가 HWP5 `shadow_type`에서 어떤 값으로 저장되어야 하는지는 아직
정답지 비교가 필요하다. 현재 HWPX parser는 `DROP`과 `CONTINUOUS`를 모두 `1`로 매핑한다.
이 매핑이 정답인지도 다음 단계에서 확인해야 한다.

다음 단계에서는 `CharShape shadow`만 독립적으로 비교한다.

```text
1. HWPX에서 <hh:shadow>가 있는 charPr ID를 수집한다.
2. 같은 charPr ID에 대응하는 정답 HWP CHAR_SHAPE payload를 비교한다.
3. shadow_type, shadow_offset_x, shadow_offset_y, shadow_color의 HWP5 저장값을 확인한다.
4. HWPX parser에 offsetX/offsetY 매핑을 추가하고, CONTINUOUS type 매핑을 검증한다.
```
