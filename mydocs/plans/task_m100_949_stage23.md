# Task M100 #949 Stage 23 계획서 — #974 글상자 TAC 계약을 HWP5 저장 계약으로 연결

## 1. 배경

Stage 22에서 `hwpx-h-03` 파일손상 후보는 `hp:rect > hp:drawText` 내부 HWP5 record contract로
좁혀졌다.

핵심 차이:

```text
#826 LIST_HEADER: oracle 33B / generated 20B
#827 PARA_HEADER: oracle 24B / generated 22B
```

이미 닫힌 계약:

```text
hp:shapeComment -> CommonObjAttr.description
hp:subList@vertAlign=CENTER -> LIST_HEADER list_attr bit 5
hp:pic@href -> CTRL_DATA #833
```

## 2. #974와의 연결

#974에서 처리한 문제:

```text
글상자 내부 TAC 그림 + space 문자열 + TAC 그림 조판
```

#974의 결론:

```text
1. 글상자 내부 TAC 그림은 일반 그림 배치가 아니라 inline 흐름 안에서 위치를 얻어야 한다.
2. 그림 사이 space 문자열은 run charPr만으로 폭을 계산하면 한컴과 달라진다.
3. 한컴식 결과는 lineSeg.horzsize와 스타일(바탕글)을 반영한 조판 계약을 따른다.
```

Stage 23의 해석:

```text
#974는 렌더링 계층에서 발견한 글상자 TAC 암묵 계약이다.
#949 Stage 22는 같은 계열의 계약을 HWP5 직렬화 계층에서 완성해야 하는 문제다.
```

따라서 이번 단계는 `drawText` 내부를 단순하게 "렌더링 가능한 IR"로 두지 않고,
한컴 에디터가 받아들이는 HWP5 record envelope까지 materialize하는 방향으로 진행한다.

## 3. 목적

`hwpx-h-03` 파일손상을 해결하기 위해 다음 두 축을 구현 후보로 분리한다.

```text
1. drawText TextBox LIST_HEADER 13B tail materialization
2. drawText 내부 paragraph PARA_HEADER 24B tail materialization
```

단, 후보는 반드시 다음 guard를 통과해야 한다.

```text
1. hwpx-h-01 성공 유지
2. hwpx-h-02 성공 유지
3. #974 hy-001 글상자 TAC 그림 렌더링 테스트 유지
4. hwpx-h-03 파일손상 해소 여부 판정
```

## 4. 구현 원칙

### 4.1 serializer를 직접 확장하지 않는다

현 구조는 HWPX to HWP 저장 전용 어댑터가 IR을 HWP5 저장 가능 형태로 정규화한다.

수정 대상:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

수정하지 않는 대상:

```text
src/serializer/control.rs
src/serializer/body_text.rs
```

이유:

```text
HWP 원본 roundtrip은 raw_* 필드 보존으로 이미 동작한다.
문제는 HWPX-origin IR에 HWP5 저장 계약 필드가 없다는 점이므로, HWPX adapter에서 materialize한다.
```

### 4.2 HWPX/HWP3 출처에만 적용한다

`convert_if_hwpx_source()` 가드 아래에서만 동작한다. HWP 출처는 no-op 유지한다.

### 4.3 넓은 payload graft 금지

Oracle payload를 통째로 복사하지 않는다.

허용하는 materialization:

```text
LIST_HEADER base 20B 이후 13B zero tail
PARA_HEADER raw_header_extra를 serializer가 24B header를 쓰도록 보정
```

## 5. 후보 구현 상세

### 5.1 AdapterReport 카운터 추가

추가 후보:

```rust
text_box_list_header_tail_materialized: u32
text_box_para_header_tail_materialized: u32
```

`changed_anything()`에도 반영한다.

### 5.2 TextBox LIST_HEADER tail

대상:

```text
drawing.text_box.raw_list_header_extra
```

후보 규칙:

```text
if text_box has paragraphs
and raw_list_header_extra is empty
then raw_list_header_extra = 13 zero bytes
```

예상 효과:

```text
serializer/control.rs::serialize_text_box_if_present()
20B base + raw_list_header_extra(13B) = 33B LIST_HEADER
```

### 5.3 TextBox 내부 paragraph header tail

대상:

```text
text_box.paragraphs[*].raw_header_extra
```

후보 규칙:

```text
if paragraph is inside HWPX-origin drawText text box
and raw_header_extra is too short
then materialize raw_header_extra so serializer emits a 24B PARA_HEADER
```

후보 byte shape:

```text
raw_header_extra[0..2]  = numCharShapes
raw_header_extra[2..4]  = numRangeTags
raw_header_extra[4..6]  = numLineSegs
raw_header_extra[6..10] = instanceId/tail base, oracle pattern 00 00 00 80
raw_header_extra[10..12]= trailing 00 00
```

주의:

```text
serializer/body_text.rs는 raw_header_extra[6..]만 쓴다.
따라서 raw_header_extra 길이를 12B로 맞추면 header tail 6B가 출력되어 24B PARA_HEADER가 된다.
```

## 6. 검증 계획

### 6.1 단위 테스트

추가/보강:

```text
cargo test --quiet hwpx_h_03_drawtext_hwp5_textbox_contract_from_source
cargo test --quiet test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx
```

검증 항목:

```text
1. hwpx-h-03 HWPX parse 후 adapter 적용 시 TextBox raw_list_header_extra == 13B zero tail
2. drawText 내부 paragraph raw_header_extra가 12B contract shape로 materialize됨
3. #974 hy-001 렌더링 회귀 없음
```

### 6.2 HWP 생성

생성 경로:

```text
output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/
```

생성 파일:

```text
hwpx-h-01.hwp
hwpx-h-02.hwp
hwpx-h-03.hwp
hy-001.hwp
```

생성 명령:

```text
cargo run --quiet --bin rhwp -- convert samples/hwpx/hwpx-h-01.hwpx output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-01.hwp
cargo run --quiet --bin rhwp -- convert samples/hwpx/hwpx-h-02.hwpx output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-02.hwp
cargo run --quiet --bin rhwp -- convert samples/hwpx/hwpx-h-03.hwpx output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-03.hwp
cargo run --quiet --bin rhwp -- convert samples/hwpx/hy-001.hwpx output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hy-001.hwp
```

### 6.3 정적 record 확인

대상:

```text
hwpx-h-03 #826 LIST_HEADER
hwpx-h-03 #827 PARA_HEADER
```

기대:

```text
#826 LIST_HEADER size=33
#827 PARA_HEADER size=24
```

### 6.4 작업지시자 시각 판정 요청

판정표:

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-01.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-02.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-03.hwp` |  |  |  |  |  |  | target |
| `output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hy-001.hwp` |  |  |  |  |  |  | #974 guard |

## 7. 산출물

완료 보고서:

```text
mydocs/working/task_m100_949_stage23.md
```

정적 trace:

```text
output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/
```

## 8. 승인 요청

Stage 23은 #974에서 확인한 글상자 TAC 계약을 HWP5 저장 계약으로 연결하는 첫 구현 단계다.

승인 후 `src/document_core/converters/hwpx_to_hwp.rs`에 한정해 구현하고, h01/h02/h03/hy-001
guard 파일을 생성한다.
