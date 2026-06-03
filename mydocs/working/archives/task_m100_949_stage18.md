# Task M100-949 Stage 18 작업 보고서: table-axis 성공 후보 확장 검증

## 1. 목적

Stage 17에서 `hwpx-h-01` sentinel이 성공했다. 이번 단계는 같은 adapter를 `hwpx-h-01`,
`hwpx-h-02`, `hwpx-h-03`에 적용해 다음 실패 축을 분리하는 것이다.

## 2. 산출 경로

```text
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/
```

## 3. 작업지시자 시각 판정표

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-01.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | Stage 17 guard |
| `output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-02.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 확장 검증 |
| `output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-03.hwp` | 파일손상 | 성공 | 성공 | - | 실패(2페이지까지만) | 1페이지 페이지네이션 실패 | 확장 검증 |

## 4. 생성 결과

생성 파일:

```text
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-03.hwp
```

파일 크기와 rhwp 재로드 정보:

| file | size | rhwp reload |
|---|---:|---|
| `hwpx-h-01.hwp` | 374,784 bytes | ok, sections=2, pages=9 |
| `hwpx-h-02.hwp` | 32,256 bytes | ok, sections=2, pages=10 |
| `hwpx-h-03.hwp` | 38,400 bytes | ok, sections=2, pages=9 |

생성 명령:

```text
cargo run --quiet --bin rhwp -- convert samples/hwpx/hwpx-h-01.hwpx output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-01.hwp
cargo run --quiet --bin rhwp -- convert samples/hwpx/hwpx-h-02.hwpx output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-02.hwp
cargo run --quiet --bin rhwp -- convert samples/hwpx/hwpx-h-03.hwpx output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-03.hwp
```

## 5. 정적 검증

각 샘플의 한컴 변환 정답지와 Stage18 산출물에 대해 TABLE field diff를 확인했다.

| sample | candidate_count | TABLE field diff count | report |
|---|---:|---:|---|
| `hwpx-h-01` | 0 | 0 | `output/poc/hwpx2hwp/task949/stage18_table_axis_regression/oracle_vs_hwpx-h-01_table_fields.md` |
| `hwpx-h-02` | 0 | 0 | `output/poc/hwpx2hwp/task949/stage18_table_axis_regression/oracle_vs_hwpx-h-02_table_fields.md` |
| `hwpx-h-03` | 0 | 0 | `output/poc/hwpx2hwp/task949/stage18_table_axis_regression/oracle_vs_hwpx-h-03_table_fields.md` |

판정 전 해석:

```text
1. Stage17에서 성공한 table-axis 계약은 Stage18 세 샘플 모두에서 oracle TABLE field와 일치한다.
2. 따라서 시각 판정에서 표 배치 실패가 남으면 TABLE record 단독 문제가 아니라,
   주변 control/common shape/group/picture contract 또는 조판 계산값 축으로 분리해야 한다.
3. hwpx-h-02/hwpx-h-03 실패가 발생하더라도 원인을 지금 단정하지 않는다.
   실패 샘플 하나를 기준으로 oracle/source contract 비교로 들어간다.
```

## 6. 시각 판정 해석

작업지시자 판정 결과:

```text
hwpx-h-01: 한컴/rhwp-studio 모두 성공
hwpx-h-02: 한컴/rhwp-studio 모두 성공
hwpx-h-03: 한컴 파일손상, 한컴 이미지는 출력 성공, 표/셀 배치 성공, 2페이지까지만 출력
           rhwp-studio는 1페이지 페이지네이션 실패
```

확정된 점:

```text
1. Stage17 table-axis 계약은 hwpx-h-01뿐 아니라 hwpx-h-02에서도 성공했다.
2. hwpx-h-03도 TABLE field diff는 oracle 대비 0이다.
3. 따라서 hwpx-h-03의 파일손상 원인을 TABLE record field 축으로 보면 안 된다.
4. hwpx-h-03은 이미지 출력과 표/셀 배치가 성공했으므로, 적어도 1페이지 표/이미지 축은
   이번 성공 후보의 직접 실패 원인이 아니다.
```

다음 단계의 핵심:

```text
hwpx-h-03 전용으로 2페이지 중단 지점과 rhwp-studio 1페이지 페이지네이션 실패를 동시에 설명하는
control contract를 찾아야 한다.

우선순위는 다음과 같다.

1. hwpx-h-03 oracle/generated의 2페이지 전후 control tree와 record stream 비교
2. page break, section boundary, shape/group control, caption/anchor, line segment 계산값 비교
3. hwpx-h-01/02 성공 케이스와 hwpx-h-03 실패 케이스의 source XML 구조 차이 확인
```
