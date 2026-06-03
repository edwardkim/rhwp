# Task M100-1094 Stage 4 작업 기록

## 1. 단계 목표

작업지시자 판정:

```text
Stage 3 후보에서도 한컴 에디터의 "셀 안쪽 여백 지정"이 활성화되지 않는다.
2페이지 표의 "AI응용제품신속상용화지원사업" 텍스트가 포함된 셀 속성을 세부 조사해야 한다.
```

Stage 4는 해당 셀을 정답 HWP와 생성 HWP에서 record 단위로 비교해, 실제 활성화 bit가 어느
record/field에 있는지 확정한다.

## 2. 대상 셀

HWPX 원본 `samples/hwpx/aift.hwpx`의 Section 0 표에서 같은 텍스트가 2회 등장한다.

```text
AI응용제품신속상용화지원사업
```

원본 HWPX 셀 속성:

```text
hp:tc@hasMargin = "1"
hp:cellAddr = colAddr 19, rowAddr 2 / rowAddr 3
hp:cellSpan = colSpan 8, rowSpan 1 / rowSpan 2
hp:cellSz width = 11678
hp:cellMargin = left 141, right 141, top 113, bottom 113
```

핵심은 `hp:tc@hasMargin="1"`이다. 기존 HWPX parser가 이 값을 읽지 않았으므로, IR의
`Cell.apply_inner_margin`이 false로 남을 수 있었다.

## 3. 정답 HWP와 Stage 3 생성 HWP의 차이

정답 HWP에서 대상 셀 직전 `LIST_HEADER`:

```text
01 00 00 00 20 00 01 00 ...
```

Stage 3 생성 HWP에서 대상 셀 직전 `LIST_HEADER`:

```text
01 00 00 00 20 00 00 00 ...
```

해석:

```text
bytes 0-1: n_para = 1
bytes 2-5: list_attr = 0x00200000 (vertical center)
bytes 6-7: list_header_width_ref/property = 0x0001 / 0x0000
```

따라서 "셀 안쪽 여백 지정" 활성화는 TABLE record attr bit 26만으로 결정되지 않는다. 대상 셀에서는
셀 `LIST_HEADER`의 bytes 6-7, 즉 `list_header_width_ref/property bit 0`이 켜져야 한다.

## 4. 구현 내용

수정 파일:

```text
src/parser/hwpx/section.rs
src/parser/control.rs
src/document_core/converters/hwpx_to_hwp.rs
src/document_core/converters/diagnostics.rs
tests/hwpx_to_hwp_adapter.rs
```

구현 규칙:

```text
1. HWPX parser
   hp:tc@hasMargin="1" -> Cell.apply_inner_margin = true

2. HWP5 parser
   Cell.apply_inner_margin <- LIST_HEADER width_ref/property bit 0

3. HWPX to HWP adapter
   Cell.apply_inner_margin == true 이면 LIST_HEADER width_ref/property bit 0을 materialize
```

정정 사항:

```text
이전 가설: list_attr bit 16 또는 TABLE attr bit 26만으로 셀 안쪽 여백 지정 활성화
정정 결과: 대상 셀의 실제 활성화 bit는 셀 LIST_HEADER width_ref/property bit 0
```

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task1094/stage4_cell_has_margin_width_ref/aift-cell-has-margin-width-ref.hwp
```

`rhwp info` 결과:

```text
sections = 3
pages = 76
reload = ok
size = 4,605,952 bytes
```

## 6. 생성 후보 검증

Stage 4 생성 HWP에서 대상 셀 직전 `LIST_HEADER`:

```text
01 00 00 00 20 00 01 00 ...
```

정답 HWP와 동일하게 bytes 6-7이 `01 00`으로 저장된다.

생성 trace:

```text
output/poc/hwpx2hwp/task1094/stage4_cell_has_margin_width_ref/generated_anchor_trace.md
```

정답 trace:

```text
output/poc/hwpx2hwp/task1094/stage4_cell_has_margin_width_ref/oracle_anchor_trace.md
```

## 7. 판정표

| file | 한컴 판정 유형 | 셀 안쪽 여백 지정 | 메모 표시 | 2페이지 표/셀 배치 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1094/stage4_cell_has_margin_width_ref/aift-cell-has-margin-width-ref.hwp` |  |  |  |  |  |  | `hasMargin -> width_ref bit0` 후보 |

## 8. 실행한 검증

```text
cargo fmt --check
cargo check
cargo test parser::hwpx::section::tests::test_parse_table_cell_has_margin
cargo test document_core::converters::hwpx_to_hwp::tests::stage3_cell_with_inner_margin_gets_width_ref_bit0
cargo test --test hwpx_to_hwp_adapter task888_basic_table_materializes_hancom_table_attrs
target/debug/rhwp hwp5-anchor-trace ...
target/debug/rhwp info ...
```

결과:

```text
success
```

## 9. 다음 판정 기준

작업지시자 시각 판정에서 확인할 항목:

```text
1. 한컴 에디터에서 대상 셀의 "셀 안쪽 여백 지정"이 활성화되는가
2. 2페이지 표 높이와 페이지 배치가 정답 HWP에 가까워지는가
3. #1092 메모 표시가 유지되는가
```

만약 "셀 안쪽 여백 지정"은 활성화되지만 표 높이가 여전히 다르면, 남은 축은 셀 여백 활성화가 아니라
다른 셀/행 높이 contract 또는 line segment contract로 분리한다.
