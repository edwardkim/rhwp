# Task M100-493 구현계획서 — 셀 속성 bit 계약 정합화

- 이슈: #493
- 수행계획서: `mydocs/plans/task_m100_493.md`
- 작성일: 2026-06-19

## 1. 현재 코드 계약

HWP5 LIST_HEADER의 bytes 6-7은 `Cell::list_header_width_ref`에 보존된다.

| bit | 의미 |
|---:|---|
| 0 | 안 여백 지정 |
| 1 | 셀 보호 |
| 2 | 제목 셀 |
| 3 | 양식 모드에서 편집 가능 |

기존 HWP5 경로는 이 값을 보존하고, `getCellProperties()`는 bit 1을 `cellProtect`로 노출한다.
HWPX 경로는 `hasMargin`, `header`, `name`만 의미 있게 읽고 `protect`, `editable`은 누락한다.

## 2. 설계 선택

`Cell`에 별도 bool 필드를 크게 늘리지 않고, 기존 `list_header_width_ref` bit 0~3을
HWP5/HWPX 공통 저장소로 쓴다.

필요하면 `Cell` helper를 추가한다.

```rust
const CELL_FLAG_HAS_MARGIN: u16 = 0x0001;
const CELL_FLAG_PROTECT: u16 = 0x0002;
const CELL_FLAG_HEADER: u16 = 0x0004;
const CELL_FLAG_EDITABLE_IN_FORM: u16 = 0x0008;
```

이 방식은 HWP5 serializer가 이미 `list_header_width_ref`를 그대로 쓰는 구조와 맞고,
HWPX→HWP 저장에서도 별도 materialize 필드를 최소화할 수 있다.

## 3. Stage 1 — 파서/serializer/API 회귀 봉인

### 3.1 모델 헬퍼

대상:

- `src/model/table.rs`

작업:

- 셀 속성 bit 상수와 helper를 추가한다.
- `cell_protect`, `editable_in_form` 읽기/쓰기 helper를 제공한다.
- 기존 `apply_inner_margin`, `is_header` 필드와 bit가 어긋나지 않도록 파서/설정 경로에서 동기화한다.

### 3.2 HWPX 파서

대상:

- `src/parser/hwpx/section.rs::parse_table_cell`

작업:

- `header`, `hasMargin` 파싱 시 기존 bool 필드와 `list_header_width_ref` bit를 함께 맞춘다.
- `protect`를 bit 1에 반영한다.
- `editable`을 bit 3에 반영한다.
- `name=""`는 현재처럼 `None`을 유지한다.

### 3.3 HWPX serializer

대상:

- `src/serializer/hwpx/table.rs::write_cell`

작업:

- `protect` 고정값을 `cell.cell_protect()` 기반으로 바꾼다.
- `editable` 고정값을 `cell.editable_in_form()` 기반으로 바꾼다.
- `header`, `hasMargin`도 helper/필드 동기화 결과를 사용한다.

### 3.4 셀 속성 API

대상:

- `src/document_core/commands/table_ops.rs`
- `src/wasm_api.rs`
- `rhwp-studio/src/core/types.ts`
- `rhwp-studio/src/ui/table-cell-props-dialog.ts`

작업:

- `getCellProperties()` 결과에 `fieldName`, `editableInForm`을 추가한다.
- `setCellProperties()` 입력에 `fieldName`, `editableInForm`을 허용한다.
- 기존 `cellProtect`는 bit 1 helper를 사용한다.
- 표/셀 속성 UI의 필드 이름과 양식 모드 편집 가능 컨트롤을 활성화하고 값을 채운다.

### 3.5 필드 목록 API

대상:

- `src/document_core/queries/field_query.rs`

작업:

- 셀 `field_name` 기반 가상 ClickHere 필드를 만들 때, cell bit 3이 켜져 있으면
  `properties` bit 0을 켠다.
- `getFieldList()`의 `editableInForm`이 HWPX `hp:tc editable="1"`과 일치하도록 한다.

## 4. Stage 1 테스트

신규:

- `tests/issue_493_cell_attrs.rs`

검증 항목:

1. `samples/셀보호.hwpx` 로드 시 셀[0], 셀[1], 셀[2]는 `protect=true`다.
2. 셀[2]는 `field_name == "name"`이고 `editableInForm=true`다.
3. `serialize_hwpx()` 결과에서 셀[0..2]의 `protect="1"`과 셀[2]의 `editable="1"`이 보존된다.
4. `samples/셀보호.hwp`와 `samples/셀보호.hwpx`가 같은 셀 속성 bit 의미를 갖는다.
5. `getFieldList()`에서 셀 필드 `"name"`의 `editableInForm`이 true다.

기존 회귀:

- `tests/issue_493_hwpx_cell_field_name.rs`
- `set_cell_field_text_updates_text_metadata`

## 5. Stage 2 — 필요 시 HWP 저장 검증 보강

Stage 1 후 HWPX→HWP 저장 또는 `exportHwp` 경로에서 셀 보호/양식 모드 bit가 빠지는 정황이 있으면,
`hwpx_to_hwp` adapter에 bit materialize 테스트를 추가한다. 현재 설계상 `list_header_width_ref`
자체를 HWPX 파서에서 세우므로 별도 adapter 수정은 없을 가능성이 높다.

## 6. 리스크

- 기존 HWPX roundtrip baseline은 IR diff가 protect/editable을 비교하지 않아 결함을 놓쳤다.
  신규 테스트는 XML 속성 또는 모델 bit를 직접 확인해야 한다.
- 필드 이름을 빈 문자열로 설정하는 경우는 `None`으로 유지한다. 빈 필드를 삭제 의미로 처리할지는
  Stage 1에서 명확히 테스트한다.
- 셀 보호 UX 차단까지 넓히면 #493 범위를 넘어가므로 이번 Stage에서는 속성 보존과 API 노출에 집중한다.
