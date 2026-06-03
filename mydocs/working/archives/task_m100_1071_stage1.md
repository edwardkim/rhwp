# Task M100-1071 Stage 1 완료 보고서

## 1. 목표

HWPX `shape-001.hwpx` 의 TAC 도형 두 개가 HWP 정답지보다 서로 붙어 보이는 원인을
구현 전에 정량화하고, `paragraph_layout` 안에서 어느 정보가 어긋나는지 특정한다.

## 2. 입력 자료

```text
samples/shape-001.hwp
samples/hwpx/shape-001.hwpx
pdf-large/hwpx/shape-001.pdf
output/poc/issue_1067/hwp/shape-001.svg
output/poc/issue_1067/hwpx/shape-001.svg
```

## 3. 재현 결과

SVG 좌표 차이는 기존 이슈 기록과 동일하게 재현된다.

| 파일 | 첫 도형 중심 x | 두 번째 도형 중심 x | 간격 |
|---|---:|---:|---:|
| HWP `shape-001.hwp` | 132.0533 | 156.5867 | 24.5333 |
| HWPX `shape-001.hwpx` | 126.0533 | 137.5867 | 11.5333 |

산출물:

```text
output/poc/task1071/stage1_tac_shape_layout_trace/
```

## 4. 핵심 관찰

두 파일의 문단 텍스트와 `char_offsets` 는 동일하다.

```text
text="  "
char_offsets=[24, 25]
char_count=35
```

하지만 `para.controls` 개수가 다르다.

```text
HWP:
  controls=4
  [0] SectionDef
  [1] ColumnDef
  [2] Shape
  [3] Shape

HWPX:
  controls=3
  [0] ColumnDef
  [1] Shape
  [2] Shape
```

`char_offsets=[24,25]` 는 첫 글자 앞에 8 code unit 컨트롤 슬롯 3개가 있다는 뜻이다.
HWP는 그 3개 슬롯이 `SectionDef`, `ColumnDef`, 첫 번째 `Shape` 에 대응한다.
HWPX는 `SectionDef` 가 `para.controls` 에 없기 때문에 3개 슬롯이 `ColumnDef`, 첫 번째 `Shape`,
두 번째 `Shape` 에 대응해 버린다.

## 5. paragraph_layout TAC trace

`RHWP_DEBUG_PARA_TAC=1` 로 확인한 결과:

```text
HWP:
  run_char_end=6
  run_tacs=[(2, 4.533333333333333, 2), (5, 4.533333333333333, 3)]

HWPX:
  run_char_end=5
  run_tacs=[(1, 4.533333333333333, 1), (2, 4.533333333333333, 2)]
```

HWP는 두 도형이 `char position 2` 와 `char position 5` 로 배치된다.
HWPX는 두 도형이 `char position 1` 과 `char position 2` 로 배치되어 공백 2개의 advance 를
먹지 못한다.

## 6. XML 원문 확인

HWPX `Contents/section0.xml` 의 실제 순서는 다음과 같다.

```text
hp:secPr
hp:ctrl/hp:colPr
hp:polygon    # 첫 번째 TAC 도형
hp:t "  "
hp:polygon    # 두 번째 TAC 도형
hp:t empty
```

따라서 두 번째 도형은 XML 상으로도 공백 2개 뒤에 존재한다. 현재 HWPX layout 이 두 번째 도형을
공백 앞쪽으로 끌어오는 것은 XML 순서 문제가 아니라 parser 가 만든 `char_offsets` 와
`para.controls` 의 slot 불일치 문제다.

## 7. 원인 판단

`src/parser/hwpx/section.rs::parse_paragraph` 는 `hp:secPr` 를 처리할 때:

```text
1. sec_def 는 별도 반환값으로 저장한다.
2. text_parts 에 control marker(\u0002)를 추가한다.
3. colPr 은 para.controls 에 ColumnDef 로 추가한다.
4. secPr 자체는 para.controls 에 SectionDef 로 추가하지 않는다.
```

그 결과 `char_offsets` stream 에는 `secPr` 슬롯이 있지만, controls stream 에는 대응 컨트롤이 없다.

```text
char_offsets stream:
  [secPr][colPr][shape1][" "][" "][shape2]

para.controls:
  [colPr][shape1][shape2]
```

`Paragraph::control_text_positions()` 는 `char_offsets` 의 8 code unit gap 과 `para.controls`
순서를 매칭해 위치를 계산한다. 이 전제가 깨지면서 HWPX의 두 번째 TAC 도형이 공백 뒤가 아니라
첫 gap 안으로 잘못 들어간다.

## 8. 다음 단계 제안

Stage 2 에서는 HWPX `hp:secPr` 를 HWP5와 동일하게 `Control::SectionDef` 로도 materialize 하는
후보를 검증한다.

검증 조건:

```text
1. shape-001 HWP/HWPX control stream 이 [SectionDef, ColumnDef, Shape, Shape] 로 정합된다.
2. HWPX 두 번째 TAC 도형이 공백 2개 뒤로 배치된다.
3. 기존 section metadata 보존 및 serializer 중복 부작용이 없다.
4. issue_1067_shape_rotation 회귀 가드가 계속 통과한다.
```

## 9. 실행한 명령

```text
target/release/rhwp dump samples/shape-001.hwp
target/release/rhwp dump samples/hwpx/shape-001.hwpx
target/release/rhwp ir-diff samples/hwpx/shape-001.hwpx samples/shape-001.hwp --summary
cargo run --quiet --example dump_shape_para -- samples/shape-001.hwp samples/hwpx/shape-001.hwpx
cargo run --quiet --example dump_polygon_transform -- samples/shape-001.hwp samples/hwpx/shape-001.hwpx
RHWP_DEBUG_PARA_TAC=1 target/release/rhwp export-svg samples/shape-001.hwp -o output/poc/task1071/stage1_tac_shape_layout_trace/hwp_debug
RHWP_DEBUG_PARA_TAC=1 target/release/rhwp export-svg samples/hwpx/shape-001.hwpx -o output/poc/task1071/stage1_tac_shape_layout_trace/hwpx_debug
```

## 10. 결론

Stage 1은 구현 전 원인 특정 단계로 완료한다. 이번 증상은 `textWrap` 차이나 도형 회전 문제가 아니라,
HWPX `secPr` marker 와 `para.controls` slot 이 불일치하면서 TAC 도형의 character position 이
잘못 계산되는 문제다.
