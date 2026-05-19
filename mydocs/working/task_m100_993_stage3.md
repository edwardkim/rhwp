# Task M100-993 Stage 3 작업 기록

## 1. 목적

Stage 2 판정 결과를 실제 HWPX to HWP adapter에 반영한다.

Stage 2에서 확인된 파일손상 해소 조건은 다음 하나였다.

```text
2페이지 "󰊳  예산 현황" 직후 12x5 표의 CTRL_HEADER(Table).common_attr

oracle:    0x282a2311
generated: 0x082a2311
missing:   0x20000000
```

## 2. 중요한 정정

처음 구현 후보는 `0x20000000`을 모든 table `CTRL_HEADER.common_attr`에 적용하는 방식이었다.
하지만 정답 HWP와 비교하면 이 방식은 과적용이다.

과적용 시 정답지와 달라지는 지점:

```text
- 다수의 일반 표 CTRL_HEADER.common_attr가 0x082a2311 -> 0x282a2311로 바뀜
- Stage 2의 "02만 파일손상 해소"라는 판정 근거와 맞지 않음
```

따라서 Stage 3의 최종 구현은 다음 조건으로 좁힌다.

```text
table.caption.is_some() 인 table에만 0x20000000을 적용한다.
```

근거:

```text
- 문제 지점의 예산 현황 표는 HWPX에서 hp:caption을 가진다.
- 정답 HWP의 해당 표 CTRL_HEADER.common_attr는 0x282a2311이다.
- caption이 없는 일반 표들은 정답 HWP에서 0x082a2311을 유지한다.
```

## 3. 구현 위치

```text
src/document_core/converters/hwpx_to_hwp.rs
```

수정 함수:

```text
materialize_table_ctrl_header_attr
```

구현 규칙:

```text
base = pack_common_attr_bits(table.common)
     | 0x00002000
     | 0x08000000

if table.caption.is_some():
    base |= 0x20000000
```

## 4. 단위 테스트

추가/수정한 테스트:

```text
table_axis_materializes_hancom_record_contract
captioned_table_materializes_hancom_caption_common_attr_bit
```

의미:

```text
- caption 없는 표는 0x082a2311을 유지한다.
- caption 있는 표는 0x282a2311로 materialize한다.
```

## 5. 생성 결과

생성 파일:

```text
output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp
```

파일 정보:

```text
size: 160,768 bytes
sha256: 5e0a50b96a05ac5920e528fa82236052a67c24fdbd03b6040d02da4975baa765
rhwp reload: ok, pages=20
```

정답지 비교:

```text
output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/table_field_diff.md
```

요약:

```text
candidate_count: 3
common_attr diff: 없음
남은 diff: TABLE tail_after_0x16 1건
```

남은 TABLE tail 차이는 Stage 2에서 이미 분리한 별도 축이다.

## 6. 판정표

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 2페이지 첫 1x1 표 배경 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp` | 정상 열림 |  | 실패 |  |  |  | 1페이지 조직도 표 높이가 한컴보다 커져 다음 페이지로 넘어감. 1페이지 처음 부분에 정체 모를 선 출력 |

## 7. 실행한 검증

```text
cargo fmt --check
cargo test table_axis_materializes_hancom_record_contract --lib
cargo test captioned_table_materializes_hancom_caption_common_attr_bit --lib
cargo build
target/debug/rhwp convert samples/hwpx/mel-001.hwpx output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp
target/debug/rhwp hwp5-inventory-diff samples/hwpx/hancom-hwp/mel-001.hwp output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp --align lcs --report table-fields --out output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/table_field_diff.md
target/debug/rhwp info output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp
```

검증 결과:

```text
- fmt 통과
- 단위 테스트 2건 통과
- cargo build 통과
- generated HWP rhwp reload pages=20
- table common_attr 과적용 없음
```

## 8. 다음 판정 요청

작업지시자는 다음 파일을 한컴 에디터에서 확인한다.

```text
output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp
```

판정 포인트:

```text
1. 파일손상 해소 여부
2. 2페이지 "󰊳  예산 현황" 이후 계속 출력되는지
3. 2페이지 첫 1x1 표 셀 배경이 여전히 검게 나오는지
```

3번은 별도 BorderFill/Cell fill 축일 가능성이 높으므로, 파일손상 해소 여부와 분리해서 판정한다.

## 9. 작업지시자 시각 판정

한컴 에디터 판정 결과:

```text
- 파일은 정상적으로 열린다.
- 1페이지 표 배치가 이상하다.
- 1페이지 조직도 표 높이가 한컴 정답지보다 커져 다음 페이지로 넘어간다.
- 1페이지 처음 부분에 정체를 알 수 없는 선이 하나 그려진다.
```

따라서 Stage 3 구현은 원래 목표였던 파일손상 해소 조건은 만족했지만, 한컴 에디터 기준
1페이지 조직도 표 조판 회귀를 남겼다.

## 10. 추가 분석

정답 HWP와 Stage 3 생성 HWP를 `dump-pages`로 비교하면 rhwp 내부 조판 결과는 1페이지에서
동일하게 보인다.

```text
- body area 동일
- page 1 item count 동일
- 조직도 표: pi=8, 3x1, 635.0x142.5px, wrap=TopAndBottom, treat_as_char=true
```

즉 현재 문제는 rhwp IR 조판 단계에서 보이는 표 높이 차이가 아니라, 한컴 에디터가 HWP5
record를 해석하는 과정에서 발생하는 계약 차이로 본다.

정답 HWP와 Stage 3 생성 HWP의 HWP5 inventory diff에서 반복적으로 확인되는 차이는 다음이다.

```text
- 셀 LIST_HEADER: 정답 size=47, 생성 size=34
- 셀 내부 PARA_HEADER: 정답 size=24, 생성 size=22
```

특히 1페이지 조직도 표 구간에서도 동일한 패턴이 나타난다.

```text
BodyText.Section0#103 PARA_HEADER: 정답 size=24, 생성 size=22
BodyText.Section0#109 LIST_HEADER: 정답 size=47, 생성 size=34
조직도 표 내부 셀 LIST_HEADER/PARA_HEADER에서도 같은 차이 반복
```

현 시점의 다음 검증 축은 표 자체 `TABLE`/`CTRL_HEADER`가 아니라, 표 셀의 `LIST_HEADER`
tail 13바이트와 셀 내부 `PARA_HEADER` tail/flag 2바이트 계열이다.

1페이지 첫 부분의 정체 불명 선은 아직 별도 원인으로 확정하지 않는다. Stage 4에서는 먼저
조직도 표 높이 회귀와 직접 관련 가능성이 높은 `LIST_HEADER`/`PARA_HEADER` 계약을 분리한다.
