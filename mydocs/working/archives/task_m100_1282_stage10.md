# Task 1282 Stage 10 - 쪽 영역 안으로 제한 속성 분석

## 목적

- 새 샘플 `쪽영역안제한`, `쪽영역안제한no`를 기준으로 `쪽 영역 안으로 제한` 속성이 왜 UI에 표시되지 않고 기능도 동작하지 않는지 확인한다.
- 코드에서 해당 설정을 읽고, 전달하고, 저장하고, 배치에 반영해야 하는 지점을 식별한다.

## 입력 샘플

- `samples/ta-pic-001-r-쪽영역안제한.hwp`
- `samples/ta-pic-001-r-쪽영역안제한.hwpx`
- `samples/ta-pic-001-r-쪽영역안제한no.hwp`
- `samples/ta-pic-001-r-쪽영역안제한no.hwpx`
- `pdf/ta-pic-001-r-쪽영역안제한-2024.pdf`
- `pdf/ta-pic-001-r-쪽영역안제한no-2024.pdf`

## 진행 기록

- Stage 9 변경분 커밋 후 새 Stage로 분리했다.
- 분석 전 상태: 샘플 파일은 작업트리에 있으나 아직 커밋하지 않았다.

## 샘플 비교 결과

### HWPX

- `쪽영역안제한.hwpx`
  - 첫 번째 그림 `<hp:pos ... flowWithText="1" ... horzRelTo="COLUMN" vertRelTo="PARA" ...>`
- `쪽영역안제한no.hwpx`
  - 첫 번째 그림 `<hp:pos ... flowWithText="0" ... horzRelTo="COLUMN" vertRelTo="PARA" ...>`

따라서 한컴 UI의 `쪽 영역 안으로 제한`은 HWPX `hp:pos@flowWithText`와 대응한다.

### HWP5

`target/debug/rhwp hwp5-anchor-trace ... --window 20`로 첫 번째 그림 `CTRL_HEADER`를 확인했다.

- `쪽영역안제한.hwp`: `properties=0x002a2210`, `flowWithText=true`
- `쪽영역안제한no.hwp`: `properties=0x002a0210`, `flowWithText=false`

두 값은 bit 13만 차이난다. HWP5 스펙 표 70의 `CommonObjAttr bit 13`이 같은 설정이다.

## 코드 경로 확인

### 파서/저장 경로는 대부분 보존됨

- `src/model/shape.rs`
  - `CommonObjAttr::flow_with_text`가 HWPX `hp:pos@flowWithText`를 보존한다.
- `src/parser/control/shape.rs`
  - HWP5 `CTRL_HEADER` attr bit 13을 `common.flow_with_text`로 파싱한다.
- `src/parser/hwpx/section.rs`
  - HWPX `flowWithText`를 `common.flow_with_text`로 파싱한다.
- `src/document_core/converters/common_obj_attr_writer.rs`
  - HWP5 저장 시 `common.flow_with_text`를 attr bit 13으로 다시 합성한다.
- `src/serializer/hwpx/shape.rs`
  - 일반 shape 계열은 `flowWithText`를 `common.flow_with_text` 값으로 쓴다.
- `src/serializer/control.rs`
  - HWP5 picture 저장은 `serialize_common_obj_attr(&pic.common)`를 사용하므로 attr bit 13 보존 경로가 있다.

### 끊기는 지점

- `src/document_core/commands/object_ops.rs`
  - 그림 속성 getter JSON에는 `restrictInPage` 또는 `flowWithText`가 없다.
  - `apply_picture_props_inner`와 `apply_common_obj_attr_from_json`도 `restrictInPage`를 읽지 않는다.
  - 결과적으로 프런트엔드가 값을 표시하거나 변경해도 Rust 모델까지 전달될 경로가 없다.
- `src/serializer/hwpx/picture.rs`
  - picture 전용 HWPX 저장기 `write_pos()`가 `flowWithText`를 `common.flow_with_text`가 아니라 `"1"`로 고정한다.
  - `쪽영역안제한no.hwpx`를 라운드트립하면 no 상태가 on으로 뒤집힐 수 있다.
- `rhwp-studio/src/ui/picture-props-dialog.ts`
  - `쪽 영역 안으로 제한(B)` 체크박스를 만들지만 `disabled = true`로 고정한다.
  - `applyPropsToUi()`에서 `this.props.restrictInPage`를 체크박스에 반영하지 않는다.
  - `handleOk()`에서도 체크박스 값을 `updated.restrictInPage`로 내보내지 않는다.
- `src/renderer/layout/picture_footnote.rs`
  - `compute_object_position()`은 기준 영역, 정렬, offset만 사용하고 `common.flow_with_text`를 보지 않는다.
  - 따라서 값이 true여도 좌표를 본문/쪽 영역 안으로 보정하지 않는다.
- `src/renderer/layout/table_layout.rs`, `src/renderer/layout/table_partial.rs`
  - 표 셀 내부 non-inline 그림은 `compute_object_position()`에 `inner_area`를 넘기지만,
    `flow_with_text` 기반의 위치 제한 보정은 호출 전후 어디에도 없다.

## 기술 문서 보완

- `mydocs/tech/한글문서파일형식_5.0_revision1.3.md`
  - 표 70 아래에 `쪽 영역 안으로 제한` ↔ HWP5 bit 13 ↔ HWPX `flowWithText` 매핑 주석을 추가했다.
- `mydocs/tech/hwp_spec_errata.md`
  - 같은 매핑과 샘플 기준을 errata 항목으로 추가했다.

## 원인 판단

1. 저장 포맷 파싱 자체는 이미 `flow_with_text`로 가능하다.
2. 그림 속성 API와 대화상자에서 한컴 UI 명칭인 `restrictInPage`로 노출하지 않아 표시가 되지 않는다.
3. setter 경로가 없어 사용자가 체크 상태를 바꿔도 모델에 반영할 수 없다.
4. HWPX picture 저장기가 `flowWithText`를 `"1"`로 고정해 off 상태를 보존하지 못한다.
5. 레이아웃 좌표 계산에서 `flow_with_text`를 사용하지 않아 기능도 동작하지 않는다.

## 다음 구현 방향

- 그림/일반 개체 속성 JSON에 `restrictInPage: common.flow_with_text`와
  `allowOverlap: common.allow_overlap`을 포함한다.
- setter에서 `restrictInPage`를 `common.flow_with_text` 및 attr bit 13으로 반영한다.
- picture 속성 대화상자의 체크박스를 활성화하고 props와 양방향 바인딩한다.
- `restrictInPage=true`일 때 `allowOverlap=false`로 취급하는 한컴 UI 규칙을 반영한다.
- HWPX picture 저장기의 `flowWithText`/`allowOverlap`도 `common` 값을 쓰게 정규화한다.
- `compute_object_position()` 또는 호출 직후 보정 함수에서 `flow_with_text`가 true인 비-TAC 개체의
  y 좌표를 본문/쪽 영역 안으로 제한하는 규칙을 구현한다.
