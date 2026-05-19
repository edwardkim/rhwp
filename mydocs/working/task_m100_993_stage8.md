# Task M100-993 Stage 8 Working Note

## 1. 목적

`tb-org-02` 조직도 셀에서 한컴 에디터가 셀 내부 텍스트를 1글자씩 줄나눔하는 문제를
HWP5 record 축으로 분리한다.

Stage 7에서 확인한 차이는 다음 두 계열이다.

```text
1. 셀 LIST_HEADER
   - bytes 6..7: oracle=0x0001, generated=0x0000
   - bytes 34..46: oracle에는 13바이트 extra 존재, generated에는 없음

2. 셀 내부 PARA_HEADER
   - oracle size=24
   - generated size=22
```

Stage 8은 production adapter를 바로 수정하지 않고, 정답 HWP의 raw contract만 generated HWP에
핀셋 graft한 판정 파일을 생성한다.

## 2. 입력

```text
oracle:
samples/hwpx/hancom-hwp/tb-org-02.hwp

generated baseline:
output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/tb-org-02-gradient-fixed.hwp
```

## 3. 생성 결과

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/
```

생성 파일:

```text
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/01_current.hwp
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/02_target_org_cell_list_width_ref_only.hwp
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/03_target_org_cell_list_extra_only.hwp
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/04_target_org_cell_list_width_ref_plus_extra.hwp
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/05_target_org_cell_para_header_tail_only.hwp
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/06_target_org_cell_list_bundle_plus_para_header_tail.hwp
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/07_all_org_cells_list_width_ref_plus_extra.hwp
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/08_all_org_cells_list_bundle_plus_para_header_tail.hwp
```

생성 로그:

```text
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/stage8_generation.md
```

## 4. 생성 파일 요약

| file | rhwp reload | list width_ref | list extra | para header tail | target cells | 비고 |
|---|---|---:|---:|---:|---:|---|
| `01_current.hwp` | ok, pages=1 | 0 | 0 | 0 | 0 | 기준 |
| `02_target_org_cell_list_width_ref_only.hwp` | ok, pages=1 | 5 | 0 | 0 | 5 | 대표 셀 width_ref만 보강 |
| `03_target_org_cell_list_extra_only.hwp` | ok, pages=1 | 0 | 5 | 0 | 5 | 대표 셀 LIST_HEADER extra만 보강 |
| `04_target_org_cell_list_width_ref_plus_extra.hwp` | ok, pages=1 | 5 | 5 | 0 | 5 | 대표 셀 LIST_HEADER bundle |
| `05_target_org_cell_para_header_tail_only.hwp` | ok, pages=1 | 0 | 0 | 21 | 5 | 대표 셀 내부 PARA_HEADER tail |
| `06_target_org_cell_list_bundle_plus_para_header_tail.hwp` | ok, pages=1 | 5 | 5 | 21 | 5 | 대표 셀 전체 후보 |
| `07_all_org_cells_list_width_ref_plus_extra.hwp` | ok, pages=1 | 186 | 240 | 0 | 0 | 조직도 전체 LIST_HEADER bundle |
| `08_all_org_cells_list_bundle_plus_para_header_tail.hwp` | ok, pages=1 | 186 | 240 | 306 | 0 | 조직도 전체 후보 |

## 5. 정정 사항

초기 생성 시 조직도 표 dimension을 `24x47`로 잘못 두어 patch count가 0으로 나왔다.
`tb-org-02`의 실제 조직도 TABLE payload는 다음과 같다.

```text
rows = 10
cols = 65
```

도구의 조직도 표 dimension을 `10x65`로 보정한 뒤 다시 생성했고, 위와 같이 실제 graft가 적용됐다.

## 6. 작업지시자 판정표

| variant | 한컴 판정 유형 | 조직도 셀 줄나눔 | 표 높이 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| `01_current.hwp` |  |  |  |  | 기준 |
| `02_target_org_cell_list_width_ref_only.hwp` |  |  |  |  |  |
| `03_target_org_cell_list_extra_only.hwp` |  |  |  |  |  |
| `04_target_org_cell_list_width_ref_plus_extra.hwp` |  |  |  |  |  |
| `05_target_org_cell_para_header_tail_only.hwp` |  |  |  |  |  |
| `06_target_org_cell_list_bundle_plus_para_header_tail.hwp` |  |  |  |  |  |
| `07_all_org_cells_list_width_ref_plus_extra.hwp` |  |  |  |  |  |
| `08_all_org_cells_list_bundle_plus_para_header_tail.hwp` |  |  |  |  |  |

## 7. 판정 메모

작업지시자 시각 판정:

```text
07_all_org_cells_list_width_ref_plus_extra.hwp
08_all_org_cells_list_bundle_plus_para_header_tail.hwp

위 두 파일에서 개선이 관찰됨.
```

해석:

```text
1. 대표 5개 셀만 보강하는 축보다, 조직도 전체 셀 LIST_HEADER 보강이 더 유효하다.
2. 한컴 에디터의 조직도 셀 줄나눔/표 높이 산정은 개별 셀 하나의 독립 계산이 아니라,
   같은 TABLE 안의 셀 LIST_HEADER contract 전체에 영향을 받는 것으로 보인다.
3. 07과 08이 모두 개선된다면 1차 핵심은 셀 내부 PARA_HEADER tail보다
   LIST_HEADER width_ref + extra bundle일 가능성이 높다.
4. 08이 07보다 더 좋다면 PARA_HEADER tail은 보조 contract로 분리한다.
```

다음 단계에서는 `07`과 `08`의 실제 차이를 기준으로 다음을 결정한다.

```text
- 07이 충분히 정상: adapter에는 조직도/표 셀 LIST_HEADER width_ref + extra materialization을 우선 구현한다.
- 08만 충분히 정상: LIST_HEADER bundle + 셀 내부 PARA_HEADER tail을 함께 구현 후보로 올린다.
```

## 8. 실행한 검증

```text
cargo check
cargo run --quiet --bin rhwp -- hwp5-cell-header-probe samples/hwpx/hancom-hwp/tb-org-02.hwp output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/tb-org-02-gradient-fixed.hwp --out-dir output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe --section 0
```

결과:

```text
cargo check: success
probe generation: success
all generated files: rhwp reload ok, pages=1
```
