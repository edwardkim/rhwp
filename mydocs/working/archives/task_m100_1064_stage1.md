# Task M100-1064 Stage 1 — 진단 도구 + ParameterSet 정밀 분석

- 이슈: [#1064](https://github.com/edwardkim/rhwp/issues/1064)
- 단계: Stage 1
- 브랜치: `local/task1064`
- 일시: 2026-05-22

## 1. 진단 도구 작성

`examples/dump_table_ctrl_data.rs` 신규 — 정답지/저장본 표 control 직후의
HWPTAG_CTRL_DATA (tag 87) payload 를 ParameterSet 구조로 파싱하여 출력.

hwplib `ForParameterSet.java` 정합 — outer.id (UINT2) + outer.count (SINT2) + skip 2 + items.

## 2. 정답지 CTRL_DATA payload 정밀 분석

`samples/el-school-001.hwp` 의 두 CTRL_DATA (동일 payload, 104 byte):

```
ParameterSet id=0x021B count=1
  [0] id=0x0242 type=ParameterSet (recurse):
    ParameterSet id=0x0242 count=11
      [0] id=0x4000 type=I4 value=3826         ← vertical_offset (페이지 상단 ~)
      [1] id=0x4001 type=I4 value=1048         ← horizontal_offset
      [2] id=0x4002 type=I4 value=28346        ← table width (HWPUNIT)
      [3] id=0x4003 type=I4 value=8475         ← table height (HWPUNIT)
      [4] id=0x4004 type=I4 value=708          ← row height
      [5] id=0x4005 type=I4 value=0            ← flag
      [6] id=0x4006 type=I4 value=2            ← unknown (row count?)
      [7] id=0x4007 type=I4 value=9            ← unknown (col count?)
      [8] id=0x4008 type=I4 value=0            ← flag
      [9] id=0x4009 type=I4 value=59528        ← page width (210mm A4)
      [10] id=0x400A type=I4 value=84188       ← page height (297mm A4)
```

**핵심 발견**:
- `0x4009 = 59528` (A4 width 210mm = 59528 HWPUNIT)
- `0x400A = 84188` (A4 height 297mm = 84188 HWPUNIT)
- 두 표 가 동일 payload → 페이지 메타데이터 (표 별 다른 값 없음)

## 3. hwplib 권위 자료 분석

hwplib 검색 (`grep 0x021B`):
- `ControlField.java`: `ctrlData.getParameterSet().getId() == 0x021B` → **FieldName 영역**
- `ParameterSet.createForFieldName()`: id=0x021B, item.id=0x4000 type=String

→ hwplib 가 0x021B 를 **FieldName** 으로만 처리. **표 ParameterSet (id 0x0242, 11 I4)** 은
hwplib 에 명시적 처리 없음 — 한컴 자체 메타데이터 영역 (공식 contract 미정의).

## 4. 본질 확정

표 control 의 ctrl_data_records 슬롯에 다음 ParameterSet payload 합성 필요:

```
outer: id=0x021B count=1
  item[0]: id=0x0242 type=ParameterSet
    inner: id=0x0242 count=11
      items[0~10]: id=0x4000~0x400A type=I4 (페이지/표 위치/크기 메타)
```

값 합성 후보:
- **(a)** HWPX `<hp:tbl>` attribute + section 의 `pagePr` 에서 계산
- **(b)** 모든 표에 정답지 값 hardcoded (페이지 동일 시 안전)
- **(c)** 셀 안 도형 있는 표만 적용 + 페이지 정보는 section 에서 동적 계산

권장: **(c) 안전 적용**. 본 case 의 정답지 2 표 모두 셀 안 도형 가지며 동일 페이지 →
condition + 페이지 정보 동적 계산.

## 5. Stage 2 정정 방향

`src/document_core/converters/hwpx_to_hwp.rs::adapt_table`:
- 신규 함수 `adapt_table_ctrl_data(table, ctrl_data_slot, section_page_def, report)`
- 조건: 셀 안 그림/도형이 있는 표만 적용 (회귀 가드)
- payload 합성:
  - 0x4000 = table.common.vertical_offset
  - 0x4001 = table.common.horizontal_offset
  - 0x4002 = table.common.width
  - 0x4003 = table.common.height
  - 0x4004~0x4008 = 정답지 default (정확한 의미 미식별, 안전 default)
  - 0x4009 = page width (section 의 paper width)
  - 0x400A = page height (section 의 paper height)

대안: 처음에는 **(b) hardcoded** 로 시도하여 한컴 시각 판정 통과 확인 후 (c) 로 일반화.

## 6. CI 확인

- cargo build --release --example dump_table_ctrl_data: success
- 진단 도구 실행 성공 (정답지 + 저장본 둘 다 정상 파싱)

## 7. 메모리 룰 정합

- `feedback_diagnosis_layer_attribution` — record-level diff + ParameterSet 정밀 분석으로 본질 식별
- `reference_hwp2hwpx_library` — hwplib 검증 (0x021B 는 FieldName 영역, 표 ParameterSet 은 공식 contract 미정의)
- `project_hwpx_to_hwp_adapter_limit` 정합

## 8. 작업지시자 승인 요청

Stage 2 정정 방향 (조건 (c) — 셀 안 도형 있는 표 + payload 동적 합성) 진행 승인 여부.

또는 **(b) hardcoded** 로 우선 시도하여 한컴 시각 판정 후 (c) 일반화 결정 권장 — 안전한
점진 검증 (Task #1058/#1061 패턴).
