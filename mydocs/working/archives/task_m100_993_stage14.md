# Task M100-993 Stage 14 작업 기록

## 1. 목적

Stage 12 판정에서 다음 두 파일이 1페이지 고스트 라인을 제거했다.

```text
03_oracle_first_para_head_text_char_shape.hwp
06_oracle_first_para_bundle.hwp
```

`06`은 첫 문단 전체 bundle 투영이므로 상위 양성 케이스다. `03`은 첫 문단의
`PARA_HEADER/PARA_TEXT/PARA_CHAR_SHAPE`만 투영한 제한 케이스이므로, Stage 14에서는 이 세 record를
다시 조합별로 분해한다.

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
output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/
```

상세 trace:

```text
output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/first_para_control_trace.md
```

## 3. 생성 파일

| file | 한컴 판정 유형 | 1페이지 고스트 선 | 조판부호 표시 순서 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/01_stage11_positive.hwp` |  |  |  |  |  | 기준 파일 |
| `output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/02_oracle_para_header.hwp` |  |  |  |  |  | 첫 문단 `PARA_HEADER` 단독 |
| `output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/03_oracle_para_text.hwp` | 성공 | 제거 |  |  |  | 첫 문단 `PARA_TEXT` 단독 |
| `output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/04_oracle_para_char_shape.hwp` |  |  |  |  |  | 첫 문단 `PARA_CHAR_SHAPE` 단독 |
| `output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/05_oracle_para_header_text.hwp` | 성공 | 제거 |  |  |  | `PARA_HEADER + PARA_TEXT` |
| `output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/06_oracle_para_header_char_shape.hwp` |  |  |  |  |  | `PARA_HEADER + PARA_CHAR_SHAPE` |
| `output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/07_oracle_para_text_char_shape.hwp` | 성공 | 제거 |  |  |  | `PARA_TEXT + PARA_CHAR_SHAPE` |
| `output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix/08_oracle_para_header_text_char_shape.hwp` | 성공 | 제거 |  |  |  | `PARA_HEADER + PARA_TEXT + PARA_CHAR_SHAPE` positive |

파일 크기와 rhwp 재로드 정보:

| file | size | rhwp reload |
|---|---:|---|
| `01_stage11_positive.hwp` | 162,304 bytes | ok, pages=20 |
| `02_oracle_para_header.hwp` | 162,304 bytes | ok, pages=20 |
| `03_oracle_para_text.hwp` | 162,304 bytes | ok, pages=20 |
| `04_oracle_para_char_shape.hwp` | 162,304 bytes | ok, pages=20 |
| `05_oracle_para_header_text.hwp` | 162,304 bytes | ok, pages=20 |
| `06_oracle_para_header_char_shape.hwp` | 162,304 bytes | ok, pages=20 |
| `07_oracle_para_text_char_shape.hwp` | 162,304 bytes | ok, pages=20 |
| `08_oracle_para_header_text_char_shape.hwp` | 162,304 bytes | ok, pages=20 |

## 4. 구현 변경

기존 Stage 12 진단 도구에 `--triplet-matrix` 모드를 추가했다.

```text
rhwp hwp5-first-para-control-probe ... --triplet-matrix
```

기본 모드는 Stage 12 산출물 구성을 유지하고, `--triplet-matrix`를 지정할 때만 Stage 14 조합 파일을
생성한다.

## 5. 실행한 검증

```text
cargo check
cargo run --bin rhwp -- hwp5-first-para-control-probe \
  samples/hwpx/hancom-hwp/mel-001.hwp \
  output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/04_diagonal_attr_payload.hwp \
  --out-dir output/poc/hwpx2hwp/task993/stage14_first_para_triplet_matrix \
  --section 0 \
  --triplet-matrix
```

결과:

```text
success
```

## 6. 판정 포인트

```text
02만 고스트 선을 제거하면:
  원인은 PARA_HEADER size/tail 또는 first paragraph flag 계열이다.

03만 고스트 선을 제거하면:
  원인은 PARA_TEXT control marker stream이다.

04만 고스트 선을 제거하면:
  원인은 PARA_CHAR_SHAPE run offset이다.

05가 양성이고 02/03이 음성이면:
  PARA_HEADER와 PARA_TEXT 결합 문제다.

06이 양성이고 02/04가 음성이면:
  PARA_HEADER와 PARA_CHAR_SHAPE 결합 문제다.

07이 양성이고 03/04가 음성이면:
  PARA_TEXT control marker와 PARA_CHAR_SHAPE run offset 결합 문제다.

08에서만 양성이면:
  세 record 모두의 결합 계약으로 보고, byte-level 차이를 다시 분해한다.
```

## 7. 한컴 판정 결과 해석

작업지시자 판정:

```text
03, 05, 07, 08 케이스에서 성공
```

성공한 모든 케이스는 `PARA_TEXT`를 포함한다.

```text
03 = PARA_TEXT
05 = PARA_HEADER + PARA_TEXT
07 = PARA_TEXT + PARA_CHAR_SHAPE
08 = PARA_HEADER + PARA_TEXT + PARA_CHAR_SHAPE
```

반대로 `PARA_TEXT`를 포함하지 않는 `02`, `04`, `06`은 성공 케이스로 보고되지 않았다.
따라서 고스트 라인의 최소 원인은 첫 문단 `PARA_TEXT` payload로 확정한다.

Stage 12 trace에서 확인한 핵심 차이는 다음이다.

```text
oracle:
  PageNumPos/PageHide control marker 뒤에 TAB marker가 나온다.

generated:
  ColumnDef marker 뒤에 TAB marker가 먼저 나오고,
  그 뒤에 PageNumPos/PageHide/Header marker가 나온다.
```

즉 다음 구현 후보는 `PARA_HEADER`나 `PARA_CHAR_SHAPE`가 아니라 첫 문단 `PARA_TEXT`의
control marker stream materialization이다.

다음 단계:

```text
1. HWPX 첫 문단에서 TAB과 PageNumPos/PageHide/Header control의 원천 순서를 확인한다.
2. HWP 저장 시 PARA_TEXT marker stream을 정답지와 같은 순서로 직렬화한다.
3. record stream의 CTRL_HEADER 순서는 이미 동일하므로, record 순서가 아니라 text marker stream만 다룬다.
4. Stage 14의 03을 guard로 사용한다.
```
