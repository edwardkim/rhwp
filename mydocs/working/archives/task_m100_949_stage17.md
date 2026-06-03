# Task M100-949 Stage 17 작업 보고서: hwpx-h-01 table-axis sentinel 회복

## 1. 목적

Stage 16은 성공 후보 검증이 아니라 current adapter의 실패 baseline snapshot이었다.
따라서 Stage 17은 `hwpx-h-01` 하나를 sentinel로 두고, Stage 9 성공 산출물의 TABLE axis 계약을
adapter 구현에 정확히 반영하는 단계로 재정의했다.

## 2. 입력 기준

Oracle:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 9 성공 기준:

```text
output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/08_all_table_axes.hwp
```

Stage 16 실패 baseline:

```text
output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-01.hwp
```

## 3. 실패 baseline 분석

Stage 9 성공 파일과 Stage 16 current adapter 산출물의 TABLE field diff:

```text
output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/stage9_success_vs_current_table_fields.md
output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/stage9_success_vs_current_table_probe_plan.md
```

요약:

```text
ctrl_outer_margin: 25건 차이
ctrl_common_attr: 9건 차이
table_attr: 18건 차이
table_tail: 25건 차이
```

이 결과는 current adapter가 Stage 9 성공 계약을 아직 구현하지 않았다는 뜻이다.

## 4. 구현 내용

수정 파일:

```text
src/parser/hwpx/section.rs
src/serializer/hwpx/table.rs
src/document_core/converters/hwpx_to_hwp.rs
```

반영한 계약:

```text
1. HWPX hp:tbl@noAdjust를 TABLE record attr bit 3으로 보존한다.
2. TABLE CTRL_HEADER의 outer margin을 HWPX hp:outMargin에서 materialize한다.
3. TABLE CTRL_HEADER common attr를 CommonObjAttr 기반으로 재계산하고,
   한컴 HWP 저장 관례의 flow-with-text/numbering bit를 materialize한다.
4. HWPX pageBreak=CELL은 HWP TABLE record에서는 RowBreak bit로 materialize한다.
5. TABLE record attr는 pageBreak/repeatHeader/noAdjust 세 축으로 재구성한다.
6. TABLE record row_sizes는 행별 실제 셀 수로 재구성한다.
7. TABLE record tail payload는 최소 zone-count 0(`00 00`)을 materialize한다.
```

중요한 제외:

```text
CTRL_HEADER height 재계산은 Stage 9 성공 계약이 아니므로 적용하지 않는다.
```

## 5. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/hwpx-h-01-stage17.hwp
```

rhwp reload:

```text
sections=2
pages=9
size=374,784 bytes
```

## 6. 정적 검증

Oracle vs Stage17:

```text
output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/oracle_vs_stage17_table_fields.md
```

결과:

```text
candidate_count=0
TABLE field diff=0
```

Stage9 success vs Stage17:

```text
output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/stage9_success_vs_stage17_table_fields.md
```

결과:

```text
candidate_count=0
TABLE field diff=0
```

## 7. 작업지시자 판정 대상

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/hwpx-h-01-stage17.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | `hwpx-h-01` sentinel |

## 8. 실행한 검증

```bash
cargo check --quiet
cargo test --quiet table_axis_materializes_hancom_record_contract
cargo test --quiet picture_href_ctrl_data
cargo run --quiet --bin rhwp -- convert samples/hwpx/hwpx-h-01.hwpx output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/hwpx-h-01-stage17.hwp
cargo run --quiet --bin rhwp -- hwp5-inventory-diff samples/hwpx/hancom-hwp/hwpx-h-01.hwp output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/hwpx-h-01-stage17.hwp --align lcs --report table-fields --focus table --format md --section 0 --out output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/oracle_vs_stage17_table_fields.md
cargo run --quiet --bin rhwp -- hwp5-inventory-diff output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/08_all_table_axes.hwp output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/hwpx-h-01-stage17.hwp --align lcs --report table-fields --focus table --format md --section 0 --out output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/stage9_success_vs_stage17_table_fields.md
```

테스트 참고:

```text
cargo test 실행 중 기존 경고가 출력되었지만, 신규 테스트와 picture href 관련 테스트는 통과했다.
```

## 9. 다음 단계

작업지시자 시각 판정에서 `hwpx-h-01-stage17.hwp`가 성공하면 그때만 `hwpx-h-02`, `hwpx-h-03`로
확장한다. 실패하면 추가 샘플로 가지 않고 `hwpx-h-01` sentinel의 남은 contract 차이를 먼저
분리한다.

## 10. 판정 결론

작업지시자 판정 결과 `hwpx-h-01` sentinel은 한컴 에디터와 rhwp-studio 모두 성공했다.

따라서 Stage 17에서 구현한 TABLE axis 계약은 `hwpx-h-01` 기준으로 성공 후보로 확정한다.
다음 단계에서는 같은 adapter를 `hwpx-h-02`, `hwpx-h-03`에 적용하되, 실패가 발생하면 다시
샘플을 넓히지 않고 해당 샘플의 oracle record bundle과 source contract 차이로 좁힌다.
