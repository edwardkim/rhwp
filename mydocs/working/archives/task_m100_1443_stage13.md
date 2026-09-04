# Task M100 #1443 Stage 13 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `08b20bc7 task 1443: 셀 segment 리사이즈 한컴 동작 보정`

## 1. Stage 13 목표

`셀보호.hwp` / `셀보호.hwpx` 샘플의 셀 속성 중 `안 여백 지정` 체크 상태가 한컴과 다르게 표시되는 문제를 분석하고, 스펙 기준으로 보정한다.

사용자 확인 기준:

- 한컴에서 `samples/셀보호.hwp`와 `samples/셀보호.hwpx`의 모든 셀은 `안 여백 지정`이 꺼져 있다.
- rhwp Studio 표/셀 속성 대화상자는 일부 셀에서 `안 여백 지정`이 켜진 것처럼 보인다.
- 셀 여백 원값이 저장되어 있어도, 지정 플래그가 꺼져 있으면 셀 고유 여백으로 적용하면 안 된다.

## 2. 기술문서 기준

- HWP 5.0 공개 스펙 `표 65: 문단 리스트 헤더`는 공통 LIST_HEADER 6바이트만 설명한다.
- 저장소 errata/IR 문서 기준으로 표 셀 LIST_HEADER에는 추가 2바이트 `width_ref`가 붙고, 이 영역의 bit 0이 셀 `안 여백 지정`이다.
- 현재 모델 상수는 `CELL_FLAG_HAS_MARGIN = 0x0001`이다.
- HWPX에서는 `<hp:tc hasMargin="...">`가 같은 의미를 가진다. OWPML 기본값은 false로 본다.
- `cellMargin` / HWP 셀 padding 필드는 원값 보존 대상이다. 체크 여부나 렌더 적용 여부는 `hasMargin` 또는 HWP LIST_HEADER bit 0이 결정한다.

## 3. 샘플 분석

### HWPX

명령:

```sh
unzip -p samples/셀보호.hwpx Contents/section0.xml | rg -o 'hasMargin="[^"]+"' | sort | uniq -c
unzip -p samples/셀보호.hwpx Contents/section0.xml | rg -o '<hp:cellMargin [^>]+'
unzip -p samples/셀보호.hwpx Contents/section0.xml | rg -o '<hp:inMargin [^>]+'
```

결과:

- 25개 셀 모두 `hasMargin="0"`.
- 표 기본 `hp:inMargin`은 `left="510" right="510" top="141" bottom="141"`.
- 모든 셀의 `hp:cellMargin`도 같은 값으로 저장되어 있으나, `hasMargin="0"`이므로 셀 고유 여백 적용 상태가 아니다.

### HWP 바이너리

명령:

```sh
target/debug/rhwp dump-records samples/셀보호.hwp
target/debug/rhwp dump samples/셀보호.hwp --section 0 --para 0
```

결과:

- 첫 셀 LIST_HEADER 앞 8바이트 예: `01 00 00 00 20 00 02 01`
  - `n_para=1`
  - `list_attr=0x00200000`
  - `width_ref=0x0102`
  - bit 0은 0, bit 1은 셀 보호.
- 필드/양식 편집 셀은 `width_ref=0x010a`로 bit 1(셀 보호), bit 3(양식 모드 편집 가능)이 켜져 있고 bit 0은 0.
- 파서 덤프에서 25개 셀 모두 `aim=false`.
- HWP와 HWPX 샘플은 한컴 표시와 동일하게 `안 여백 지정`이 모두 꺼진 상태로 저장되어 있다.

## 4. 원인 후보

- Rust 모델/파서는 `apply_inner_margin=false`를 올바르게 복원한다.
- HWPX 직렬화도 `cell.apply_inner_margin`을 `hasMargin`으로 방출한다.
- 하지만 `get_cell_properties_native()`가 `apply_inner_margin`을 Studio로 전달하지 않는다.
- Studio `TableCellPropsDialog`는 셀 탭 생성 시 `cellPaddingCheck.checked = true`로 고정하고, `populateFields()`에서도 체크 상태를 실제 셀 플래그로 갱신하지 않는다.
- `set_cell_properties_native()`도 `applyInnerMargin`을 받지 않으므로 사용자가 체크를 꺼도 HWP/HWPX 저장 플래그를 갱신할 수 없다.

## 5. 수정 기준

- `getCellProperties()` JSON에 `applyInnerMargin`을 포함한다.
- `setCellProperties()` JSON에서 `applyInnerMargin`을 받아 `cell.set_apply_inner_margin()`으로 반영한다.
- Studio 셀 속성 대화상자는 `cp.applyInnerMargin`에 맞춰 `안 여백 지정` 체크 상태를 표시한다.
- 확인을 누르면 체크 상태를 항상 저장한다. 체크가 꺼져 있으면 padding 원값은 보존하되 셀 고유 여백 적용 플래그만 꺼야 한다.
- 렌더/편집 경로에서 셀 고유 padding 적용 여부를 판단할 때 원칙적으로 `apply_inner_margin`을 기준으로 삼는다.
- `안 여백 지정`을 켠 셀의 좌우 여백은 사용자 지정값이므로, 텍스트 오버플로우 방어용 padding 축소 휴리스틱으로 깎지 않는다.

## 6. 검증 계획

- `samples/셀보호.hwp`와 `samples/셀보호.hwpx` 로드 후 `getCellProperties()`의 `applyInnerMargin=false` 확인.
- Studio 셀 속성 대화상자에서 `안 여백 지정`이 꺼져 보이는지 확인.
- `안 여백 지정`을 켜고 저장하면 HWPX `hasMargin="1"` 또는 HWP LIST_HEADER bit 0이 켜지는지 확인.
- `안 여백 지정`을 끄면 셀 padding 원값이 있어도 렌더/편집에서 표 기본 padding을 사용하는지 확인.

## 7. 구현 내용

- `src/document_core/commands/table_ops.rs`
  - `get_cell_properties_native()`가 `applyInnerMargin`을 JSON으로 반환하게 했다.
  - `set_cell_properties_native()`가 `applyInnerMargin`을 받아 `cell.set_apply_inner_margin()`으로 LIST_HEADER bit 0과 IR 값을 함께 갱신하게 했다.
- `src/wasm_api.rs`
  - `getCellProperties` 반환 JSON 주석에 `applyInnerMargin`을 추가했다.
- `rhwp-studio/src/core/types.ts`
  - `CellProperties.applyInnerMargin` 타입을 추가했다.
- `rhwp-studio/src/ui/table-cell-props-dialog.ts`
  - 셀 탭 생성 시 `셀 크기 적용`을 무조건 체크하던 초기값을 제거했다.
  - `셀 크기 적용`이 꺼져 있으면 크기 입력칸을 비활성화하고, 확인 시 셀 width/height를 저장하지 않는다.
  - 셀 탭 생성 시 `안 여백 지정`을 무조건 체크하던 초기값을 제거했다.
  - `populateFields()`에서 실제 `cp.applyInnerMargin`으로 체크 상태를 채운다.
  - 확인 시 체크 상태를 항상 `applyInnerMargin`으로 저장한다.
  - 체크 해제 시 여백 입력칸을 비활성화한다.
- `rhwp-studio/src/engine/input-handler.ts`
  - 모양복사 셀 속성 키에 `applyInnerMargin`을 포함했다.
- `tests/issue_493_cell_attrs.rs`
  - `셀보호.hwp/hwpx`의 모든 셀 `apply_inner_margin=false` 회귀 가드를 추가했다.
  - `getCellProperties()`의 `applyInnerMargin=false` 반환을 확인한다.
  - HWPX 라운드트립에서 `hasMargin="0"` 25개가 유지되는지 확인한다.
  - `setCellProperties()`로 `applyInnerMargin`을 켰다 끄는 API 회귀 테스트를 추가했다.
- `src/renderer/layout/table_layout.rs`, `table_partial.rs`, `table_cell_content.rs`
  - `shrink_cell_padding_for_overflow()`에 사용자 지정 셀 여백 보존 조건을 추가했다.
  - `cell.apply_inner_margin=true`인 셀은 한컴처럼 입력한 좌우 안 여백을 그대로 보존하고, 좁아진 내부 폭에서 텍스트를 줄바꿈/클리핑하게 했다.
  - 일반 셀에는 기존 오버플로우 방어 휴리스틱을 유지한다.
  - 명시 안 여백 셀에서 padding 축소가 발생하지 않는 단위 테스트를 추가했다.

## 8. 검증 결과

- `git diff --check` 통과.
- `cargo test -p rhwp --lib renderer::layout::table_layout::row_cut_tests::test_shrink_cell_padding_preserves_explicit_cell_margin -- --nocapture` 통과.
- `cd rhwp-studio && npx tsc --noEmit` 통과.
- `cargo check --test issue_493_cell_attrs` 통과.
- `cargo test --test issue_493_cell_attrs -- --nocapture` 통과: 7개 테스트 통과.
- `wasm-pack build --target web --out-dir pkg` 통과. Rust/WASM API 및 렌더링 변경을 로컬 Studio 실행에 반영했다.

## 9. 추가 시각 이슈 원인

사용자 시각 확인 중 셀 안 텍스트가 작아 보이는 현상이 있었다. 실제 글자 크기 변경보다는 `안 여백 지정`이 켜진 상태에서 좌우 여백이 적용되어, 셀 내부 텍스트 폭이 좁아지고 줄바꿈/클리핑되며 작게 보이는 현상으로 판단했다.

추가 확인에서 좌우 10mm처럼 내부 폭이 매우 좁아지는 셀에서 기존 `shrink_cell_padding_for_overflow()`가 텍스트 오버플로우 방어를 위해 좌우 padding을 줄이고 있었다. 이는 일반 셀에는 필요한 방어지만, 사용자가 `안 여백 지정`을 켠 셀에서는 한컴과 다르게 입력 여백을 무시하는 결과가 된다. 따라서 `apply_inner_margin=true` 셀은 padding 축소 대상에서 제외했다.

같은 대화상자에서 `셀 크기 적용`도 기본 체크 상태였기 때문에 확인만 눌러도 셀 width/height 저장과 재배치가 함께 발생할 수 있었다. 한컴처럼 명시 체크한 경우에만 셀 크기를 저장하도록 기본값을 꺼서 보정했다.

## 10. `셀보호2` 추가 기준 반영

사용자가 `samples/셀보호2.hwp`, `samples/셀보호2.hwpx`, `pdf/셀보호2-2024.pdf`를 추가했다.

- `pdf/셀보호2-2024.pdf`는 현재 로컬 파일 내용이 0바이트 패턴인 `data` 파일이라 Poppler(`pdfinfo`, `pdftoppm`)로 직접 렌더링할 수 없었다.
- 사용자 제공 PDF 스크린샷과 `셀보호2.hwpx`의 저장 XML을 기준으로 정합성을 확인했다.
- `셀보호2.hwpx` 기준:
  - 마지막 행 첫 셀만 `hasMargin="1"`.
  - 해당 셀은 `cellSz width="5951" height="3733"`.
  - 해당 셀은 `cellMargin left="2834" right="2834" top="0" bottom="0"`.
  - 해당 셀 텍스트 `12345`는 `LINE_SEG textpos=0,2,4`로 저장되어 한컴처럼 `12 / 34 / 5` 세 줄로 배치된다.
- `셀보호2.hwp`와 `셀보호2.hwpx`를 `rhwp dump -s 0 -p 1`로 확인했을 때 같은 IR로 파싱된다.

추가 구현:

- `src/document_core/commands/table_ops.rs`
  - width, 좌우 padding, `applyInnerMargin` 변경 시 해당 셀 문단을 즉시 `reflow_cell_paragraph()`로 재배치한다.
- `src/document_core/commands/text_editing.rs`
  - 셀 문단 리플로우 폭 계산에서도 `apply_inner_margin=true`인 셀의 0mm 값을 표 기본 여백으로 되살리지 않도록 했다.
- `src/renderer/layout/table_layout.rs`
  - `resolve_cell_padding()`에서 `apply_inner_margin=true`이면 0mm도 명시 셀 여백으로 그대로 사용한다.
- `src/renderer/layout/paragraph_layout.rs`
  - 셀 내부 가용 폭이 글자 자연 폭보다 좁아도 한컴처럼 글자를 수평 압축하지 않는다.
  - 줄바꿈은 `LINE_SEG`/리플로우 결과를 따르고, 렌더링된 글자는 셀 경계에서만 클리핑한다.
- `tests/issue_493_cell_attrs.rs`
  - `셀보호2.hwp/hwpx`에서 명시 안 여백 셀이 정확히 1개인지 확인한다.
  - 마지막 행 첫 셀의 좌우 10mm, 상하 0mm, `LINE_SEG` 0/2/4 저장값을 확인한다.
  - HWPX 라운드트립에서 `hasMargin="1"` 1개, `hasMargin="0"` 24개가 유지되는지 확인한다.

시각 확인:

- `cargo run --quiet --bin rhwp -- export-svg samples/셀보호2.hwp -o output/poc/task_m100_1443_stage13/cellprotect2_hwp_after -p 0`
- `cargo run --quiet --bin rhwp -- export-svg samples/셀보호2.hwpx -o output/poc/task_m100_1443_stage13/cellprotect2_hwpx_after -p 0`
- `rsvg-convert`로 PNG 변환 후 확인.
- HWP/HWPX 렌더 PNG는 byte 단위로 동일했다.
- 마지막 행 첫 셀은 사용자 제공 PDF 스크린샷처럼 `12 / 34 / 5`가 자연 폭으로 표시된다.
