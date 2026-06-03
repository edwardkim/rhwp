# Task M100-993 Stage 12 계획서

## 1. 목적

Stage 11에서 셀 대각선 문제는 `BORDER_FILL` diagonal contract로 분리되고 해결 후보가 확인되었다.
그러나 1페이지 시작 부분의 고스트 선은 여전히 남아 있었다.

작업지시자의 한컴 에디터 조판부호 관찰에 따르면, 원본 HWPX와 rhwp 생성 HWP는 첫 문단 control
sequence가 다르다.

원본 `mel-001.hwpx`:

```text
[쪽 번호 위치][감추기][탭][머리말 양쪽]
```

rhwp 생성 HWP:

```text
[탭]{고스트 선}[쪽 번호 위치][감추기][머리말 양쪽]
```

Stage 12의 목적은 이 차이를 첫 문단 control contract 문제로 분리하는 것이다.

## 2. 현재 가설

고스트 선은 표/셀 대각선이 아니라 첫 문단 control 재구성 과정에서 생긴 drawing artifact로 본다.

후보:

```text
1. HWPX 첫 문단의 control order가 HWP 저장 시 바뀐다.
2. 탭 문자 또는 탭 관련 control char가 앞쪽으로 이동한다.
3. 쪽 번호 위치/감추기/머리말 control의 CTRL_HEADER 또는 payload 순서가 정답지와 다르다.
4. 정답지에는 없는 line/shape 계열 record가 첫 문단 근처에 materialize된다.
5. 특정 control을 paragraph text stream과 record stream 양쪽에서 중복 표현한다.
```

## 3. 비교 대상

HWPX 원본:

```text
samples/hwpx/mel-001.hwpx
```

정답 HWP:

```text
samples/hwpx/hancom-hwp/mel-001.hwp
```

현재 positive 기준:

```text
output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/04_diagonal_attr_payload.hwp
```

## 4. 수행 절차

1. 원본 HWPX의 첫 문단 XML을 추출한다.
2. 정답 HWP의 첫 문단 record bundle을 추출한다.
3. generated HWP의 첫 문단 record bundle을 추출한다.
4. 다음 항목을 나란히 비교한다.

```text
- PARA_HEADER
- PARA_TEXT
- CTRL_HEADER 순서와 ctrl id
- LIST_HEADER/CTRL_DATA 등 control 부속 record
- control char offset
- line/shape 계열 record 존재 여부
```

5. 정답지와 generated 사이에서 첫 문단 근처에만 나타나는 추가 record 또는 순서 차이를 찾는다.
6. 필요하면 control order만 정답지로 투영한 제한 probe를 생성한다.

## 5. 산출물

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage12_first_para_control_trace/
```

분석 문서:

```text
output/poc/hwpx2hwp/task993/stage12_first_para_control_trace/first_para_control_trace.md
```

필요 시 판정용 HWP:

| file | 목적 |
|---|---|
| `01_stage11_positive.hwp` | Stage 11 positive 기준 |
| `02_oracle_first_para_order.hwp` | 첫 문단 control order만 정답지 투영 |
| `03_remove_suspicious_line_shape.hwp` | 첫 문단 의심 line/shape record 제거 |
| `04_oracle_first_para_bundle.hwp` | 첫 문단 bundle 제한 정답지 투영 |

## 6. 성공 기준

```text
1. 1페이지 시작 부분의 고스트 선 원인을 특정한다.
2. 셀 대각선, 1x1 표 배경, pi22 표 높이 개선을 회귀시키지 않는다.
3. 첫 문단만 제한적으로 다룬다.
4. 전체 HWP를 정답지로 덮어쓰지 않는다.
```

## 7. 비목표

```text
- 전체 문서 control order 재작성
- 모든 머리말/쪽번호 control 일반화
- 표/셀 배치 문제와 다시 섞기
- 고스트 선 제거를 위한 임의 좌표 삭제
```

## 8. 승인 요청

이 계획으로 Stage 12 첫 문단 control sequence 및 고스트 선 contract 추적을 진행한다.
