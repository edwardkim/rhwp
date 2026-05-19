# Task M100-993 Stage 14 계획서

## 1. 목적

Stage 12 판정에서 `03_oracle_first_para_head_text_char_shape.hwp`와
`06_oracle_first_para_bundle.hwp`에서 1페이지 고스트 라인이 제거되었다.

`06`은 첫 문단 bundle 전체 투영이므로 상위 양성 케이스다. `03`은 첫 문단의
`PARA_HEADER/PARA_TEXT/PARA_CHAR_SHAPE`만 투영한 제한 양성 케이스이므로, 다음 원인 범위는
이 세 record의 단독 또는 결합 차이다.

Stage 14의 목적은 `03`을 다시 분해해 고스트 라인 제거에 필요한 최소 record 조합을 찾는 것이다.

## 2. 입력

정답지:

```text
samples/hwpx/hancom-hwp/mel-001.hwp
```

기준 생성물:

```text
output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/04_diagonal_attr_payload.hwp
```

## 3. 수행 절차

첫 문단의 다음 record를 정답지에서 제한 투영한다.

```text
1. PARA_HEADER 단독
2. PARA_TEXT 단독
3. PARA_CHAR_SHAPE 단독
4. PARA_HEADER + PARA_TEXT
5. PARA_HEADER + PARA_CHAR_SHAPE
6. PARA_TEXT + PARA_CHAR_SHAPE
7. PARA_HEADER + PARA_TEXT + PARA_CHAR_SHAPE
```

기존 Stage 12 진단 도구에 `--triplet-matrix` 모드를 추가해 같은 비교 로직을 재사용한다.

## 4. 산출물

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/
```

분석 문서:

```text
output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/first_para_control_trace.md
```

판정용 HWP:

| file | 목적 |
|---|---|
| `01_stage11_positive.hwp` | 기준 파일 |
| `02_oracle_para_header.hwp` | 첫 문단 `PARA_HEADER` 단독 |
| `03_oracle_para_text.hwp` | 첫 문단 `PARA_TEXT` 단독 |
| `04_oracle_para_char_shape.hwp` | 첫 문단 `PARA_CHAR_SHAPE` 단독 |
| `05_oracle_para_header_text.hwp` | `PARA_HEADER + PARA_TEXT` |
| `06_oracle_para_header_char_shape.hwp` | `PARA_HEADER + PARA_CHAR_SHAPE` |
| `07_oracle_para_text_char_shape.hwp` | `PARA_TEXT + PARA_CHAR_SHAPE` |
| `08_oracle_para_header_text_char_shape.hwp` | `PARA_HEADER + PARA_TEXT + PARA_CHAR_SHAPE` |

## 5. 성공 기준

```text
1. 고스트 라인 제거에 필요한 최소 record 조합을 확인한다.
2. 첫 문단 bundle 전체 투영으로 후퇴하지 않는다.
3. Header subtree, 셀 대각선, gradient, 표 높이 문제와 섞지 않는다.
```
