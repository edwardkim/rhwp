# Task M100-1094 Stage 1 작업 기록

## 1. 단계 목표

#1092 최종 구현이 `local/devel`에 병합된 현재 코드 기준으로 `samples/hwpx/aift.hwpx`를 다시 HWP로
저장하고, 정답지 `samples/aift.hwp`와 TABLE record 차이가 Issue #1094에 기록된 상태와 같은지
재확인한다.

## 2. 산출물

출력 위치:

```text
output/poc/hwpx2hwp/task1094/stage1_current_table_attr_trace/
```

생성 파일:

```text
current_aift.hwp
oracle_section0_inventory.md
oracle_section1_inventory.md
current_section0_inventory.md
current_section1_inventory.md
section0_table_field_diff.md
section1_table_field_diff.md
```

생성 HWP:

```text
output/poc/hwpx2hwp/task1094/stage1_current_table_attr_trace/current_aift.hwp
```

파일 정보:

```text
size = 4,605,952 bytes
rhwp info = ok, sections=3, pages=76
```

## 3. 실행 명령

```text
cargo run --quiet --bin rhwp -- convert \
  samples/hwpx/aift.hwpx \
  output/poc/hwpx2hwp/task1094/stage1_current_table_attr_trace/current_aift.hwp

cargo run --quiet --bin rhwp -- hwp5-inventory-diff \
  samples/aift.hwp \
  output/poc/hwpx2hwp/task1094/stage1_current_table_attr_trace/current_aift.hwp \
  --align lcs --report table-fields --focus table --format md --section 0 \
  --out output/poc/hwpx2hwp/task1094/stage1_current_table_attr_trace/section0_table_field_diff.md

cargo run --quiet --bin rhwp -- hwp5-inventory-diff \
  samples/aift.hwp \
  output/poc/hwpx2hwp/task1094/stage1_current_table_attr_trace/current_aift.hwp \
  --align lcs --report table-fields --focus table --format md --section 1 \
  --out output/poc/hwpx2hwp/task1094/stage1_current_table_attr_trace/section1_table_field_diff.md
```

결과:

```text
success
```

## 4. Section 0 비교

`section0_table_field_diff.md` 기준 diff field는 `table_attr`뿐이다.

| table | field | oracle | generated | 기타 필드 |
|---|---|---:|---:|---|
| `BodyText.Section0#13` | `table_attr` | `0x04000006` | `0x00000006` | 모두 동일 |
| `BodyText.Section0#26~BodyText.Section0#25` | `table_attr` | `0x0600000e` | `0x0000000e` | 모두 동일 |

동일한 필드:

```text
record_size
payload_len
rows
cols
cell_spacing
in_margin_left/right/top/bottom
row_count_hint
col_count_hint
tail_after_0x16
```

## 5. Section 1 비교

`section1_table_field_diff.md` 기준 diff field도 `table_attr`뿐이다.

| table | field | oracle | generated | 기타 필드 |
|---|---|---:|---:|---|
| `BodyText.Section1#13` | `table_attr` | `0x0400000e` | `0x0000000e` | 모두 동일 |
| `BodyText.Section1#221~BodyText.Section1#220` | `table_attr` | `0x06000004` | `0x00000004` | 모두 동일 |

동일한 필드:

```text
record_size
payload_len
rows
cols
cell_spacing
in_margin_left/right/top/bottom
row_count_hint
col_count_hint
tail_after_0x16
```

## 6. 해석

현재 #1092 최종 구현 기준에서도 Issue #1094의 선행 관찰은 유지된다.

```text
1. HWPX -> HWP 저장 결과는 TABLE record 하위 attr만 생성한다.
2. 한컴 정답지는 같은 하위 attr 위에 0x04000000 또는 0x06000000 상위 비트를 추가한다.
3. TABLE payload에서 관찰 가능한 다른 필드는 정답지와 같다.
4. 따라서 다음 검증 후보는 TABLE attr 상위 비트 materialization이다.
```

상위 비트 패턴:

```text
0x04000000
0x06000000
```

현재 확인한 네 TABLE에서 하위 비트와 상위 비트의 결합:

| generated low attr | oracle high bits | oracle attr |
|---:|---:|---:|
| `0x00000006` | `0x04000000` | `0x04000006` |
| `0x0000000e` | `0x06000000` | `0x0600000e` |
| `0x0000000e` | `0x04000000` | `0x0400000e` |
| `0x00000004` | `0x06000000` | `0x06000004` |

즉 단순히 하위 attr만으로는 `0x04`/`0x06` 상위 비트를 결정할 수 없다.
Stage 2에서는 다음 축을 분리해 판정용 HWP를 만든다.

```text
1. 모든 TABLE에 0x04000000 부여
2. 모든 TABLE에 0x06000000 부여
3. Section 0/1 oracle projection 적용
4. row_count_hint/col_count_hint 또는 table class 기준 추론 후보
```

## 7. 다음 단계

Stage 2에서는 판정용 HWP 후보를 생성한다.

우선순위:

```text
1. 정답지 table_attr projection 후보로 한컴 표 높이/페이지 배치가 복구되는지 확인한다.
2. 복구되면 0x04000000/0x06000000 결정 규칙을 좁힌다.
3. 복구되지 않으면 TABLE attr 외 다른 한컴 내부 조판 contract가 필요하다고 판단한다.
```
