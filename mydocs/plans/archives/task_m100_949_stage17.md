# Task M100-949 Stage 17 계획서: hwpx-h-01 table-axis sentinel 회복

## 1. 목적

Stage 16의 판단 오류를 끊고, 이미 실패한 current adapter를 다른 샘플에 확장하지 않는다.

이번 단계의 목적은 하나다.

```text
hwpx-h-01을 sentinel로 두고,
Stage 9에서 성공 판정을 받은 TABLE axis 계약을 실제 adapter 구현에 정확히 반영한다.
```

## 2. 기준

성공 기준:

```text
1. samples/hwpx/hancom-hwp/hwpx-h-01.hwp 와 새 adapter 산출물의 TABLE field diff가 0이어야 한다.
2. Stage 9의 08_all_table_axes.hwp 와 새 adapter 산출물의 TABLE field diff가 0이어야 한다.
3. 이 조건을 만족하기 전에는 hwpx-h-02, hwpx-h-03로 확장하지 않는다.
```

## 3. 구현 범위

Stage 9에서 확인된 네 축만 반영한다.

```text
- CTRL_HEADER(Table) outer margin
- CTRL_HEADER(Table) common attr
- TABLE record attr
- TABLE record tail payload
```

구현 위치:

```text
src/parser/hwpx/section.rs
src/serializer/hwpx/table.rs
src/document_core/converters/hwpx_to_hwp.rs
```

## 4. 하지 않는 것

```text
- hwpx-h-02/hwpx-h-03 regression 확장
- shape/group contract 추정
- CTRL_HEADER height 재계산
- 실패 baseline을 성공 후보로 간주하는 판정
```

## 5. 산출물

```text
output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/
```

