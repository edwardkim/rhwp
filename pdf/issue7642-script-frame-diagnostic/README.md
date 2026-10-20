# 축소 글자·복합 본체 첨자의 독립 Print 대조군

Issue: #7642

한컴 정상 HWPX의 스타일·구역을 재사용한 수동 구조 입력이다. 개체 크기·기준선·저장 LineSeg를 비우고 한컴 2020에서 HWP로 정상 저장한 뒤 그 저장본을 Print 출력했다. 정식 회귀 추가를 승인하는 증거는 아니며 Native/fresh WASM과 확대 판독은 별도로 확인한다.

실제 실행 제품은 `11.0.0.9136`, PDF는 `pdf_print_method: 0` / `hancom2020_pdf_driver_one_up`이다. 서비스 profile 명칭만으로 실제 OS를 추정하지 않는다.

## 입력·산출 고정

- hwp: job `5f4eddb9-06c9-4556-8c02-4b5269b067cb`, SHA-256 `ac7b964913c72e4d8f7d675bf8b467ebbc09017dc68e1260eb62f26328cd30db`.
- pdf: job `c694ca53-3c07-4c5d-b106-b81ed47b9113`, SHA-256 `4e40c2ecf230cd95a2266d32fcc77e3bc9c3d7274386adfc4db659e3568af098`.
- 입력 HWPX SHA-256: `058668115c67a5e09e2d4556c7061f5accf5df663c22983c3723c7c154eacde0`.

## 페이지·스크립트·논리 프레임

| 쪽 | 이름 | 스크립트 | BaseUnit | 높이(HU) | 기준선(%) |
|---:|---|---|---:|---:|---:|
| 1 | line_612 | `x` | 612 | 600 | 86 |
| 2 | sup_612 | `x^z` | 612 | 750 | 89 |
| 3 | sub_612 | `x_z` | 612 | 750 | 69 |
| 4 | left_sup_612 | `x LSUP z` | 612 | 750 | 89 |
| 5 | line_1020 | `x` | 1020 | 1050 | 85 |
| 6 | sup_1020 | `x^z` | 1020 | 1213 | 87 |
| 7 | sub_1020 | `x_z` | 1020 | 1249 | 71 |
| 8 | left_sup_1020 | `x LSUP z` | 1020 | 1213 | 87 |
| 9 | line_1496 | `x` | 1496 | 1500 | 85 |
| 10 | sup_1496 | `x^z` | 1496 | 1800 | 88 |
| 11 | sub_1496 | `x_z` | 1496 | 1822 | 70 |
| 12 | left_sup_1496 | `x LSUP z` | 1496 | 1800 | 88 |
| 13 | root_LSUB | `{SQRT{x}} LSUB i` | 1500 | 2046 | 74 |
| 14 | root_LSUP | `{SQRT{x}} LSUP i` | 1500 | 1995 | 89 |
| 15 | root_SUB | `{SQRT{x}} SUB i` | 1500 | 2046 | 74 |
| 16 | root_SUP | `{SQRT{x}} SUP i` | 1500 | 1995 | 89 |
| 17 | fraction_LSUB | `{{1}OVER{2}} LSUB i` | 1500 | 3771 | 60 |
| 18 | fraction_LSUP | `{{1}OVER{2}} LSUP i` | 1500 | 3720 | 68 |
| 19 | fraction_SUB | `{{1}OVER{2}} SUB i` | 1500 | 3771 | 60 |
| 20 | fraction_SUP | `{{1}OVER{2}} SUP i` | 1500 | 3720 | 68 |
| 21 | sum_LSUB | `{SUM} LSUB i` | 1500 | 2346 | 66 |
| 22 | sum_LSUP | `{SUM} LSUP i` | 1500 | 2295 | 79 |
| 23 | sum_SUB | `{SUM} SUB i` | 1500 | 2346 | 66 |
| 24 | sum_SUP | `{SUM} SUP i` | 1500 | 2295 | 79 |
| 25 | integral_LSUB | `{INT} LSUB i` | 1500 | 3151 | 59 |
| 26 | integral_LSUP | `{INT} LSUP i` | 1500 | 3100 | 69 |
| 27 | integral_SUB | `{INT} SUB i` | 1500 | 3151 | 59 |
| 28 | integral_SUP | `{INT} SUP i` | 1500 | 3100 | 69 |
| 29 | nested_left | `x LSUB {y LSUP z}` | 1500 | 1963 | 65 |
| 30 | four_slots | `x LSUB i LSUP j SUB k SUP m` | 1500 | 2121 | 75 |

612/1020/1496HU의 일반 위첨자와 왼쪽 위첨자 프레임은 각 크기에서 일치했다. 근호·분수·합·적분의 그룹 바깥 왼쪽/오른쪽 명령형 첨자도 같은 본체별 높이·기준선을 보였다. 독립 메트릭을 미측정 크기나 다른 복합 본체로 보간하지 않는다.

## 위첨자 본체·기호의 크기 대조

입력의 크기·기준선·저장 줄을 비우고 같은 한컴에서 정상 저장한 뒤 Print했다. 정식 회귀 fixture가 아니며 Native/fresh WASM·확대 검증 완료로 간주하지 않는다.

- `upper-frame-hancom-2020.hwp` SHA-256 `1751edcb64fb6da577acb66b52878e1b814801f1d4a104652131540f7469292a`.
- `upper-frame-input.hwpx` SHA-256 `6d91e3cd6661e1bbea973f54587bb6c1ac795204383df5f06c0b70aa8dfd8134`.
- `upper-frame-hancom-2020.pdf` SHA-256 `78bd8b692d991e321ab0910df221b8be4474518b1236a227714a14c7a74c96ed`.
- hwp job `6cad1122-de73-4eb9-891b-e2cd981fc100`.
- pdf job `3ee2cfd4-5a79-4cfc-bbee-82c14f5a6b55`.

| 쪽 | 이름 | 스크립트 | BaseUnit |
|---:|---|---|---:|
| 1 | plain_i_900 | `x^i` | 900 |
| 2 | plain_2_900 | `x^2` | 900 |
| 3 | plain_y_900 | `x^y` | 900 |
| 4 | root_i_900 | `{SQRT{x}} LSUP i` | 900 |
| 5 | root_2_900 | `{SQRT{x}} LSUP 2` | 900 |
| 6 | sum_i_900 | `{SUM} LSUP i` | 900 |
| 7 | sum_2_900 | `{SUM} LSUP 2` | 900 |
| 8 | int_i_900 | `{INT} LSUP i` | 900 |
| 9 | int_2_900 | `{INT} LSUP 2` | 900 |
| 10 | plain_i_1500 | `x^i` | 1500 |
| 11 | plain_2_1500 | `x^2` | 1500 |
| 12 | plain_y_1500 | `x^y` | 1500 |
| 13 | root_i_1500 | `{SQRT{x}} LSUP i` | 1500 |
| 14 | root_2_1500 | `{SQRT{x}} LSUP 2` | 1500 |
| 15 | sum_i_1500 | `{SUM} LSUP i` | 1500 |
| 16 | sum_2_1500 | `{SUM} LSUP 2` | 1500 |
| 17 | int_i_1500 | `{INT} LSUP i` | 1500 |
| 18 | int_2_1500 | `{INT} LSUP 2` | 1500 |
| 19 | plain_i_2200 | `x^i` | 2200 |
| 20 | plain_2_2200 | `x^2` | 2200 |
| 21 | plain_y_2200 | `x^y` | 2200 |
| 22 | root_i_2200 | `{SQRT{x}} LSUP i` | 2200 |
| 23 | root_2_2200 | `{SQRT{x}} LSUP 2` | 2200 |
| 24 | sum_i_2200 | `{SUM} LSUP i` | 2200 |
| 25 | sum_2_2200 | `{SUM} LSUP 2` | 2200 |
| 26 | int_i_2200 | `{INT} LSUP i` | 2200 |
| 27 | int_2_2200 | `{INT} LSUP 2` | 2200 |

## 원 요청 문서의 실제 BaseUnit 대조

입력의 크기·기준선·저장 줄을 비우고 같은 한컴에서 정상 저장한 뒤 Print했다. 정식 회귀 fixture가 아니며 Native/fresh WASM·확대 검증 완료로 간주하지 않는다.

- `observed-unit-frames-hancom-2020.pdf` SHA-256 `6a04038ee4374cf20917e5671daf225bcf175fff30e258edc9d0bde77235fd43`.
- `observed-unit-frames-hancom-2020.hwp` SHA-256 `5b440a7593d47d31830b38e7ac840c5124bd787b0a69fa694f9d55016b20e475`.
- `observed-unit-frames-input.hwpx` SHA-256 `d06a0711dcf683909e62f1d5bbf6ea401f61c2573ed5fa2bfaddc9a097a7a09f`.
- hwp job `bf84bf65-8c77-4cbd-9dac-8f664ed403e4`.
- pdf job `e174cdb1-a9ad-4498-a9a8-9cdbf7d8258b`.

| 쪽 | 이름 | 스크립트 | BaseUnit |
|---:|---|---|---:|
| 1 | line_800 | `x` | 800 |
| 2 | plain_sup_800 | `x^i` | 800 |
| 3 | plain_sub_800 | `x_i` | 800 |
| 4 | root_800 | `SQRT{x}` | 800 |
| 5 | root_sup_800 | `{SQRT{x}} LSUP i` | 800 |
| 6 | root_sub_800 | `{SQRT{x}} LSUB i` | 800 |
| 7 | sum_800 | `SUM` | 800 |
| 8 | sum_sup_800 | `{SUM} LSUP i` | 800 |
| 9 | sum_sub_800 | `{SUM} LSUB i` | 800 |
| 10 | int_800 | `INT` | 800 |
| 11 | int_sup_800 | `{INT} LSUP i` | 800 |
| 12 | int_sub_800 | `{INT} LSUB i` | 800 |
| 13 | line_900 | `x` | 900 |
| 14 | plain_sup_900 | `x^i` | 900 |
| 15 | plain_sub_900 | `x_i` | 900 |
| 16 | root_900 | `SQRT{x}` | 900 |
| 17 | root_sup_900 | `{SQRT{x}} LSUP i` | 900 |
| 18 | root_sub_900 | `{SQRT{x}} LSUB i` | 900 |
| 19 | sum_900 | `SUM` | 900 |
| 20 | sum_sup_900 | `{SUM} LSUP i` | 900 |
| 21 | sum_sub_900 | `{SUM} LSUB i` | 900 |
| 22 | int_900 | `INT` | 900 |
| 23 | int_sup_900 | `{INT} LSUP i` | 900 |
| 24 | int_sub_900 | `{INT} LSUB i` | 900 |
| 25 | line_1000 | `x` | 1000 |
| 26 | plain_sup_1000 | `x^i` | 1000 |
| 27 | plain_sub_1000 | `x_i` | 1000 |
| 28 | root_1000 | `SQRT{x}` | 1000 |
| 29 | root_sup_1000 | `{SQRT{x}} LSUP i` | 1000 |
| 30 | root_sub_1000 | `{SQRT{x}} LSUB i` | 1000 |
| 31 | sum_1000 | `SUM` | 1000 |
| 32 | sum_sup_1000 | `{SUM} LSUP i` | 1000 |
| 33 | sum_sub_1000 | `{SUM} LSUB i` | 1000 |
| 34 | int_1000 | `INT` | 1000 |
| 35 | int_sup_1000 | `{INT} LSUP i` | 1000 |
| 36 | int_sub_1000 | `{INT} LSUB i` | 1000 |
| 37 | line_1100 | `x` | 1100 |
| 38 | plain_sup_1100 | `x^i` | 1100 |
| 39 | plain_sub_1100 | `x_i` | 1100 |
| 40 | root_1100 | `SQRT{x}` | 1100 |
| 41 | root_sup_1100 | `{SQRT{x}} LSUP i` | 1100 |
| 42 | root_sub_1100 | `{SQRT{x}} LSUB i` | 1100 |
| 43 | sum_1100 | `SUM` | 1100 |
| 44 | sum_sup_1100 | `{SUM} LSUP i` | 1100 |
| 45 | sum_sub_1100 | `{SUM} LSUB i` | 1100 |
| 46 | int_1100 | `INT` | 1100 |
| 47 | int_sup_1100 | `{INT} LSUP i` | 1100 |
| 48 | int_sub_1100 | `{INT} LSUB i` | 1100 |
| 49 | line_1200 | `x` | 1200 |
| 50 | plain_sup_1200 | `x^i` | 1200 |
| 51 | plain_sub_1200 | `x_i` | 1200 |
| 52 | root_1200 | `SQRT{x}` | 1200 |
| 53 | root_sup_1200 | `{SQRT{x}} LSUP i` | 1200 |
| 54 | root_sub_1200 | `{SQRT{x}} LSUB i` | 1200 |
| 55 | sum_1200 | `SUM` | 1200 |
| 56 | sum_sup_1200 | `{SUM} LSUP i` | 1200 |
| 57 | sum_sub_1200 | `{SUM} LSUB i` | 1200 |
| 58 | int_1200 | `INT` | 1200 |
| 59 | int_sup_1200 | `{INT} LSUP i` | 1200 |
| 60 | int_sub_1200 | `{INT} LSUB i` | 1200 |
| 61 | line_1600 | `x` | 1600 |
| 62 | plain_sup_1600 | `x^i` | 1600 |
| 63 | plain_sub_1600 | `x_i` | 1600 |
| 64 | root_1600 | `SQRT{x}` | 1600 |
| 65 | root_sup_1600 | `{SQRT{x}} LSUP i` | 1600 |
| 66 | root_sub_1600 | `{SQRT{x}} LSUB i` | 1600 |
| 67 | sum_1600 | `SUM` | 1600 |
| 68 | sum_sup_1600 | `{SUM} LSUP i` | 1600 |
| 69 | sum_sub_1600 | `{SUM} LSUB i` | 1600 |
| 70 | int_1600 | `INT` | 1600 |
| 71 | int_sup_1600 | `{INT} LSUP i` | 1600 |
| 72 | int_sub_1600 | `{INT} LSUB i` | 1600 |
