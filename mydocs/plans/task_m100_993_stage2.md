# Task M100-993 Stage 2 계획서

## 1. 목적

Stage 1에서 분리한 `mel-001` 파일손상 후보를 독립 probe로 검증한다.

핵심 질문:

```text
한컴 에디터의 파일손상 판정이 다음 중 어느 contract 누락에서 발생하는가?

1. 예산 표 CTRL_HEADER(Table).common_attr bit 0x20000000 누락
2. 2페이지 8x12 표 TABLE tail materialization 불일치
3. 2페이지 10x65 표 CTRL_DATA 누락
4. 위 조건의 복합
```

## 2. 입력 파일

```text
oracle HWP: samples/hwpx/hancom-hwp/mel-001.hwp
generated HWP: output/poc/hwpx2hwp/task993/stage1_mel001_damage_trace/generated-mel-001.hwp
```

## 3. 산출물

Stage 2 산출물 디렉터리:

```text
output/poc/hwpx2hwp/task993/stage2_mel001_table_contract_probe/
```

생성할 판정 파일:

| variant | 목적 |
|---|---|
| `01_current_generated.hwp` | Stage 1 generated baseline |
| `02_budget_table_ctrl_common_attr_oracle.hwp` | `CTRL_HEADER(Table).common_attr`만 oracle 값으로 graft |
| `03_personnel_table_tail_oracle.hwp` | 8x12 표 `TABLE.tail_after_0x16`만 oracle 값으로 graft |
| `04_org_table_missing_ctrl_data_oracle.hwp` | 10x65 표 oracle-only `CTRL_DATA`만 삽입 |
| `05_budget_common_attr_plus_personnel_tail.hwp` | 후보 1+2 결합 |
| `06_all_three_candidates.hwp` | 후보 1+2+3 결합 |

## 4. 판정표

```markdown
| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 2페이지 중단 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_current_generated |  |  |  |  |  |  |
| 02_budget_table_ctrl_common_attr_oracle |  |  |  |  |  |  |
| 03_personnel_table_tail_oracle |  |  |  |  |  |  |
| 04_org_table_missing_ctrl_data_oracle |  |  |  |  |  |  |
| 05_budget_common_attr_plus_personnel_tail |  |  |  |  |  |  |
| 06_all_three_candidates |  |  |  |  |  |  |
```

## 5. 완료 조건

```text
- 한컴 파일손상 판정이 사라지는 최소 후보 조합을 찾는다.
- 성공 후보가 있으면 해당 후보를 adapter 구현 범위로 좁힌다.
- 모든 후보가 실패하면 budget table 주변 record window를 더 세분화하는 Stage 3 계획을 작성한다.
```

## 6. 주의점

이번 단계는 실제 adapter 구현이 아니라 판정용 probe 생성이다.
이미 한컴에서 실패한 generated baseline을 성공 후보로 착각하지 않고, Stage 1에서 확인한
record 차이만 oracle 값으로 최소 graft한다.
