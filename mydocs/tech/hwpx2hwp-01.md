---
kind: reference
status: active
canonical: mydocs/tech/hwpx2hwp-01.md
last_verified: 2026-07-17
---

# HWPX to HWP 저장 기술 노트 01

## 목적

이 문서는 `hwpx -> IR -> hwp` 저장 기능을 구현하면서 확인한 한컴 호환성 규칙을
정리한다. 특히 `samples/hwpx/hwpx-h-01.hwpx`를 중심으로 진행한 #903 probe에서
얻은 내용을 다음 작업자가 재사용할 수 있도록 남긴다.

관련 샘플:

```text
samples/hwpx/basic-table-01.hwpx
samples/hwpx/expense_report.hwpx
samples/hwpx/business_overview.hwpx
samples/hwpx/hwpx-h-01.hwpx
samples/hwpx/hancom-hwp/*.hwp
```

## 기본 원칙

HWPX 파싱과 rhwp-studio 렌더링이 정상이어도 HWP 저장이 정상이라는 뜻은 아니다.
한컴 에디터는 HWP `DocInfo`, `BodyText`, `BinData`, `CTRL_HEADER`, `TABLE` record의
암묵적 조합에 민감하다.

따라서 구현은 다음 순서를 따른다.

```text
1. HWPX 원본과 한컴 변환 정답 HWP를 IR/record 단위로 비교한다.
2. raw graft probe로 실패 축을 좁힌다.
3. 실제 구현은 raw graft가 아니라 IR 모델 기반 보강으로 한다.
4. 산출물은 output/poc/... 아래에 만들고 작업지시자 시각 판정을 받는다.
5. 한컴 판정, rhwp-studio 재로드, 기존 회귀 샘플을 모두 확인한다.
```

`파일 손상`과 `파일 읽기 오류`는 다른 문제다. 한컴이 일부 페이지를 출력한 뒤
`파일 손상`을 내는 경우는 이후 record 해석 중 깨지는 경우가 많고, `파일 읽기 오류`는
초기 stream/header/DocInfo 조합부터 잘못되었을 가능성이 더 크다.

## 발견 규칙 요약

이번 `hwpx -> IR -> hwp` 저장 POC에서 확인한 규칙은 다음과 같다.

### 규칙 1. HWPX 렌더링 성공은 HWP 저장 성공을 보장하지 않는다

rhwp-studio에서 HWPX를 정상 렌더링해도, 같은 IR을 HWP로 저장하면 한컴 에디터에서
파일 손상이나 파일 읽기 오류가 날 수 있다. HWP 저장은 렌더링 IR뿐 아니라 HWP5 record
관례까지 만족해야 한다.

판정 기준은 다음 세 가지를 모두 본다.

```text
1. rhwp-studio 원본 HWPX 렌더링
2. 저장된 HWP의 rhwp-studio 재로드/렌더링
3. 저장된 HWP의 한컴 에디터 시각 판정
```

### 규칙 2. HWPX 출처 문서는 HWP 저장 직전에 adapter 보정이 필요하다

HWPX parser가 만든 IR은 HWP5 raw record를 완전히 갖고 있지 않다. 따라서 HWP 저장
직전에 다음 값을 materialize해야 한다.

```text
FileHeader 압축 플래그
DocProperties.section_count
SectionDef Control
BinData metadata
Table CTRL_HEADER / TABLE row_sizes
필요한 BorderFill 정규화
```

단, HWP 출처 문서에는 이 보정을 적용해 기존 raw payload를 깨면 안 된다.

### 규칙 3. section_count는 BodyText section 수와 같아야 한다

`DocProperties.section_count`가 실제 section 수보다 작으면 한컴이 뒤 section을
읽지 않는다. 이 경우 파일은 정상으로 열려도 마지막 페이지가 사라질 수 있다.

```text
section_count 누락 현상: 9페이지 문서가 8페이지까지만 출력
해결 규칙: section_count = doc.sections.len()
```

### 규칙 4. ParaShape는 표/셀 텍스트 배치에 직접 영향을 준다

`paraPr/margin` 자식 요소와 `ParaShape.attr1` 일부 비트가 빠지면 셀 안 텍스트의
baseline이 위로 붙고, 윗부분이 클리핑될 수 있다.

Stage30 판정으로 분리된 축:

```text
section_count: 마지막 페이지 출력 회복
ParaShape: 표/셀 텍스트 배치 회복
section_count + ParaShape: 페이지와 배치 동시 회복
```

### 규칙 5. HWP 무늬없음은 0이 아니라 -1이다

HWPX `winBrush`에 `faceColor`가 있고 `hatchStyle`이 없으면, HWP `pattern_type`은
`-1`이어야 한다. 이를 0 또는 1 기반 index처럼 다루면 한컴에서 배경 무늬가 잘못 표시된다.

```text
HWPX hatchStyle 없음 -> HWP pattern_type = -1
```

### 규칙 6. 문단/글자 배경 no-fill 정규화는 참조 범위를 제한해야 한다

투명 흰색 solid fill을 `FillType::None`으로 정규화하면 문단 배경 무늬 오류가 사라진다.
하지만 table/cell/page border가 같은 BorderFill을 참조할 수 있으므로, object 참조를
가진 BorderFill은 정규화 대상에서 제외한다.

```text
대상: ParaShape/CharShape만 참조하는 투명 흰색 solid fill
제외: table/cell/page border가 참조하는 BorderFill
```

### 규칙 7. BinData는 이미지 바이트와 HWP BIN_DATA metadata를 분리해서 봐야 한다

HWPX에는 이미지 파일이 package 안에 있어도, HWP `BIN_DATA` record metadata는 별도다.
한컴 HWP 저장 결과에서는 embedded image가 다음 값을 가져야 했다.

```text
attr = 0x0101
status = Success
```

`raw_data=None`은 이미지 바이트 제거가 아니라 HWPX 출처에는 HWP raw record가 없으므로
모델 기반으로 record를 재직렬화하라는 의미다.

### 규칙 8. Table CTRL_HEADER attr는 모든 표에 일괄 보존하면 안 된다

`CommonObjAttr`에서 pack한 table attr를 모든 table에 유지하면 일부 샘플에서 페이지 수가
변한다. 실제로 `hwpx-h-02`는 9페이지가 10페이지로 늘었다.

따라서 attr 보존은 성공이 확인된 guard 안에서만 한다.

```text
보존 대상:
- materialize_hancom_table
- materialize_tac_table

그 밖의 table:
- attr를 0으로 정규화
```

### 규칙 9. HWPX object id/zOrder/shapeComment는 CTRL_HEADER payload에 필요하다

HWPX table/picture의 `id`, `instid`, `zOrder`, `shapeComment`는 단순 보조 정보가 아니다.
HWP 저장 시 `CommonObjAttr`와 description payload에 반영되어 한컴의 object 해석에 영향을 준다.

확인된 매핑:

```text
table id 또는 instid -> table.common.instance_id
table zOrder -> table.common.z_order
picture shapeComment -> picture.common.description
```

### 규칙 10. raw graft 성공은 구현 성공이 아니라 원인 축 발견이다

정답 HWP record를 graft하면 성공 조합을 빨리 찾을 수 있다. 그러나 graft 자체를 구현으로
채택하면 샘플 과적합이 된다.

```text
raw graft 성공 -> 원인 축 확정
실제 구현 -> parser/model/serializer 필드 매핑
최종 판정 -> 기존 샘플 회귀 테스트까지 포함
```

### 규칙 11. 한컴 판정 파일은 output/poc 아래에 둔다

작업지시자 시각 판정 대상은 repository working docs가 아니라 `output/poc/...` 아래에
생성한다. 이렇게 해야 stage별 산출물과 판정 결과를 대응시키기 쉽다.

```text
output/poc/hwpx2hwp/task{issue}/stage{n}_.../*.hwp
```

## HWPX 출처 식별과 저장 어댑터

HWPX 출처 문서는 HWP 원본과 달리 HWP5 전용 raw record가 비어 있거나 기본값으로
남아 있다. 그래서 HWP 저장 직전에 `convert_hwpx_to_hwp_ir`에서 HWP 저장 관례를
materialize한다.

HWP 출처 문서에는 같은 어댑터를 적용해도 변경이 없어야 한다. HWP 원본은 이미
HWP record payload를 갖고 있으므로 raw 보존이 우선이다.

## FileHeader

HWPX에서 생성한 임시 `FileHeader`는 압축 플래그가 비어 있을 수 있다. HWP 저장기는
이 플래그를 기준으로 DocInfo/BodyText/BinData stream 압축 여부를 결정하므로,
HWPX 저장 경로에서는 압축 HWP5 헤더를 명시적으로 만든다.

```text
header.compressed = true
header.flags |= 0x01
header.raw_data = None
```

## DocProperties.section_count

`DocProperties.section_count`가 실제 BodyText section 수보다 작으면 마지막 section이
한컴에서 출력되지 않는다. `hwpx-h-01` 계열에서는 마지막 9페이지 미출력의 직접 원인이었다.

저장 직전 다음 값을 보장한다.

```text
doc.doc_properties.section_count = doc.sections.len()
doc.doc_properties.raw_data = None
doc.doc_info.raw_stream_dirty = true
```

Stage30의 핵심 판정:

```text
section_count 보정 단독: 마지막 페이지 회복
ParaShape 보정 단독: 표/셀 배치 회복, 마지막 페이지는 미회복
section_count + ParaShape: 마지막 페이지와 표/셀 배치 모두 회복
```

## SectionDef Control

HWPX 파서는 `<hp:secPr>`를 `Section.section_def`에 채우지만, HWP 직렬화기는
`paragraph.controls` 안의 `Control::SectionDef`를 순회해야 `PAGE_DEF`,
`FOOTNOTE_SHAPE`, `PAGE_BORDER_FILL` record를 쓴다.

따라서 각 section의 첫 문단 앞쪽에 `Control::SectionDef`가 없으면 삽입한다.
이미 있으면 no-op이어야 한다.

## BorderFill과 배경 무늬

HWP의 `pattern_type`에서 무늬없음은 `-1`이다. HWPX `winBrush`에 `faceColor`는 있지만
`hatchStyle`이 없으면 HWP 저장 시 `pattern_type=-1`로 보존해야 한다.

```text
HWPX hatchStyle 없음 -> HWP pattern_type = -1
HORIZONTAL -> 1
VERTICAL -> 2
BACK_SLASH -> 3
SLASH -> 4
CROSS -> 5
CROSS_DIAGONAL -> 6
```

문단/글자 모양이 참조하는 투명 흰색 solid fill은 한컴에서 배경 무늬로 오해될 수 있다.
다만 표, 셀, page border가 참조하는 BorderFill까지 무작정 바꾸면 안 된다.

정규화 조건:

```text
대상: ParaShape/CharShape가 참조하는 BorderFill
제외: table/cell/page border 등 object가 참조하는 BorderFill
조건: 모든 border line이 None, fill이 Solid, alpha=0, background=#FFFFFFFF
동작: FillType::None으로 정규화하고 raw_data 제거
```

## BinData

HWPX의 embedded image는 `content.hpf`와 package 내부 파일로 존재한다. 웹 canvas
렌더러가 BMP/WMF 등을 표시하기 위해 변환한 산출물은 HWP 저장용 원본이 아니다.
HWP 저장은 원본 bindata를 유지해야 한다.

HWPX 파서가 만든 `BinData` model은 HWP `BIN_DATA` record 전용 metadata가 비어 있을 수 있다.
한컴 HWP 로더는 embedded image에 대해 다음 조합을 기대한다.

```text
data_type = Embedding 또는 Storage
attr = 0x0101
status = Success
```

HWPX -> HWP 저장 어댑터에서는 다음처럼 보정한다.

```text
bin_data.attr = 0x0101
bin_data.status = Success
bin_data.raw_data = None
doc.doc_info.raw_stream_dirty = true
```

여기서 `raw_data=None`은 이미지 바이트를 버린다는 뜻이 아니다. HWPX 출처에는 HWP
`BIN_DATA` record raw payload가 없으므로, 모델 값으로 HWP record를 다시 쓰게 하기 위한
처리다. 실제 이미지 바이트는 BinData storage/package 경로를 통해 유지되어야 한다.

HWP -> HWP 저장 경로에서는 기존 raw record와 BinData stream 보존이 우선이다.

## Table CTRL_HEADER

HWPX table은 `raw_ctrl_data`가 비어 들어오므로 HWP 저장 전에 `CommonObjAttr` 기반
`CTRL_HEADER` payload를 합성해야 한다. 단, table attr는 모든 표에 일괄 적용하면 안 된다.

Stage54에서 확인한 회귀:

```text
모든 table packed attr 보존:
- hwpx-h-01: 성공
- hwpx-h-02: 9페이지가 10페이지로 증가
```

따라서 table attr 보존은 이미 성공 조건이 확인된 guard 범위 안에서만 한다.

```text
보존 대상:
- materialize_hancom_table
- materialize_tac_table

그 밖의 table:
- raw_ctrl_data[0..4] = 0
- table.attr = 0
```

`TABLE` record의 `row_sizes`도 HWPX 출처에서는 비어 있을 수 있다. 각 행의 cell 수를
계산해 `table.row_sizes`를 materialize해야 한컴이 표 record를 안정적으로 해석한다.

## HWPX table/picture 속성 파싱

HWPX table/object의 일부 속성은 HWP `CommonObjAttr`와 직접 연결된다. 누락되면
`CTRL_HEADER` payload가 정답 HWP와 달라지고, 이미지 출력이나 표 배치가 무너질 수 있다.

확인된 매핑:

```text
table id 또는 instid -> table.common.instance_id
table zOrder -> table.common.z_order
picture instid -> picture.common.instance_id
picture zOrder -> picture.common.z_order
picture shapeComment -> picture.common.description
```

`shapeComment`는 화면에 보이는 설명 문구가 아니더라도 HWP `CTRL_HEADER` description
payload 길이에 영향을 준다.

## ParaShape

HWPX `paraPr/margin`의 자식 요소형 값은 ParaShape margin/indent/spacing 계열에
누락 없이 매핑되어야 한다. Stage30에서 ParaShape 보정은 셀 내부 텍스트가 위로
붙어 클리핑되는 현상을 회복하는 축으로 확인되었다.

Stage51에서는 `ParaShape.attr1`의 세로 정렬 bits 20..21도 확인했다.
다만 #903 Stage53 기준에서는 해당 vertical bits가 이미 current 산출물에 들어가 있었고,
남은 실패의 직접 원인은 아니었다.

결론:

```text
ParaShape margin/attr 계열은 표/셀 텍스트 배치에 중요하다.
하지만 #903 최종 실패 원인은 BinData + CTRL_HEADER 조합이었다.
```

## raw graft probe 해석

정답 HWP에서 record를 통째로 graft하면 빠르게 성공 조건을 찾을 수 있다. 그러나 그 상태를
그대로 구현하면 특정 샘플에 과적합되고 다른 샘플을 깨뜨릴 가능성이 높다.

사용 원칙:

```text
raw graft는 원인 축 탐색용이다.
실제 구현은 parser/model/serializer 경로에 필드를 매핑한다.
성공한 probe라도 기존 샘플 회귀를 반드시 확인한다.
```

Stage53/54의 대표 사례:

```text
BIN_DATA + CTRL_HEADER raw graft: 한컴/rhwp-studio 성공
실제 구현: BinData metadata 보강 + CommonObjAttr 필드 파싱 + 기존 table attr guard 유지
```

## 검증 게이트

한컴 시각 판정 전후로 다음 자동 테스트를 최소 게이트로 사용한다.

```bash
cargo test --test hwpx_to_hwp_adapter task888 -- --nocapture
cargo test --test hwpx_to_hwp_adapter task899 -- --nocapture
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01 -- --nocapture
cargo test --test hwpx_to_hwp_adapter stage4_page_count_recovered -- --nocapture
cargo test --test hwpx_to_hwp_adapter stage5_all_three_samples_recover_via_unified_entry_point -- --nocapture
cargo test --test hwpx_to_hwp_adapter stage6_verify_recovered_for_all_three_samples -- --nocapture
cargo test --test hwpx_to_hwp_adapter task903_stage54_generate_minimal_impl_candidate -- --nocapture
```

작업지시자 시각 판정 파일은 반드시 `output/poc/...` 아래에 생성한다.

## 구현 소스 경로

현재 `hwpx -> IR -> hwp` 저장 호환 로직의 핵심 구현 위치는 다음이다.

### 저장 adapter

```text
src/document_core/converters/hwpx_to_hwp.rs
```

주요 진입점:

```text
convert_hwpx_to_hwp_ir()
convert_if_hwpx_source()
```

담당 로직:

```text
FileHeader compressed flag 보정
DocProperties.section_count 보정
HWPX embedded BinData metadata materialize
SectionDef Control 삽입
paragraph/char BorderFill no-fill 정규화
table CTRL_HEADER raw_ctrl_data 합성
table TABLE row_sizes materialize
table attr 보존 guard 적용
cell list_attr 보강
```

### CommonObjAttr 직렬화

```text
src/document_core/converters/common_obj_attr_writer.rs
```

담당 로직:

```text
CommonObjAttr -> HWP CTRL_HEADER payload 직렬화
table/object attr packing
z_order, margin, instance_id, description payload 작성
```

### HWPX section/body parser

```text
src/parser/hwpx/section.rs
```

담당 로직:

```text
table id/instid -> table.common.instance_id
table zOrder -> table.common.z_order
picture instid/zOrder 파싱
picture shapeComment -> picture.common.description
SectionDef/PageDef/PageBorderFill 파싱
table/cell/shape/body paragraph 파싱
```

### HWPX header parser

```text
src/parser/hwpx/header.rs
```

담당 로직:

```text
DocInfo 계열 파싱
BorderFill/winBrush 파싱
faceColor/hatchStyle 처리
ParaShape/CharShape 파싱
BinData 목록 파싱
```

### HWPX utility

```text
src/parser/hwpx/utils.rs
```

담당 로직:

```text
parse_hatch_style()
HWPX hatchStyle -> HWP pattern_type 매핑
hatchStyle 없음 -> pattern_type=-1 규칙
```

### HWP serializer

```text
src/serializer/control.rs
src/serializer/mod.rs
```

담당 로직:

```text
Control/Table/Picture/Shape HWP record 직렬화
raw_ctrl_data가 있으면 CTRL_HEADER payload로 사용
BinData/DocInfo/BodyText stream 작성
```

### 통합 진입점

```text
src/document_core/mod.rs
```

관련 API:

```text
DocumentCore::export_hwp_with_adapter()
DocumentCore::serialize_hwp_with_verify()
```

HWPX 출처 저장은 이 통합 진입점을 통해 adapter 적용 후 HWP 직렬화로 이어져야 한다.

### 회귀 테스트와 probe

```text
tests/hwpx_to_hwp_adapter.rs
```

담당 범위:

```text
#888 basic-table / expense_report 회귀
#899 business_overview 배경색 + 무늬없음 회귀
#903 hwpx-h-01 저장 회귀
Stage54 산출물 생성
hwpx-h-01/02/03 페이지 회복 검증
```

## 구현 규칙과 소스 대응표

| 규칙 | 구현 위치 |
|---|---|
| FileHeader 압축 보정 | `src/document_core/converters/hwpx_to_hwp.rs::normalize_file_header_for_hwp` |
| section_count 보정 | `src/document_core/converters/hwpx_to_hwp.rs::normalize_doc_properties_for_hwp` |
| SectionDef Control 삽입 | `src/document_core/converters/hwpx_to_hwp.rs::insert_section_def_control` |
| BorderFill no-fill 정규화 | `src/document_core/converters/hwpx_to_hwp.rs::normalize_paragraph_char_border_fills` |
| BinData metadata 보강 | `src/document_core/converters/hwpx_to_hwp.rs::normalize_bin_data_for_hwp` |
| table CTRL_HEADER 합성 | `src/document_core/converters/hwpx_to_hwp.rs::adapt_table` |
| table row_sizes 보강 | `src/document_core/converters/hwpx_to_hwp.rs::materialize_table_record_row_sizes` |
| CommonObjAttr payload 생성 | `src/document_core/converters/common_obj_attr_writer.rs::serialize_common_obj_attr` |
| table id/zOrder 파싱 | `src/parser/hwpx/section.rs::parse_table` |
| picture shapeComment 파싱 | `src/parser/hwpx/section.rs::parse_picture` |
| hatchStyle -> pattern_type | `src/parser/hwpx/utils.rs::parse_hatch_style` |
| 회귀 테스트 | `tests/hwpx_to_hwp_adapter.rs` |

## #903 최종 적용 범위

`samples/hwpx/hwpx-h-01.hwpx` 저장 성공을 위해 확정한 최소 구현 범위는 다음이다.

```text
1. HWPX embedded BIN_DATA metadata materialize
2. 기존 table CTRL_HEADER attr 보존 guard 유지
3. HWPX table id/zOrder 파싱
4. HWPX picture shapeComment 파싱
```

Stage54 산출물:

```text
output/poc/hwpx2hwp/task903/stage54_minimal_impl_candidate/hwpx-h-01.hwp
```

작업지시자 판정:

```text
한컴: 성공
이미지 출력: 성공
표/셀 배치: 성공
셀 텍스트 클리핑: 성공
마지막 페이지 출력: 성공
rhwp-studio: 성공
```
