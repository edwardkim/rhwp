# PR #1078 검토 문서

- PR: <https://github.com/edwardkim/rhwp/pull/1078>
- 제목: `fix: HTML 테이블 붙여넣기 raw_ctrl_data 오프셋 밀림 (#698 관련)`
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/698>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. PR 상태

| 항목 | 값 |
|---|---|
| 상태 | open |
| base | `devel` |
| head | `contrib/fix-html-table-import-offset` |
| head sha | `28268c054987cb449f3a311878ba2c33277b2147` |
| mergeable | false |
| 작성자 | `oksure` |
| 변경 파일 | 2개 |
| 변경 범위 | `src/document_core/html_table_import.rs`, 테스트 |

CI 확인:

| workflow | conclusion |
|---|---|
| CI | success |
| CodeQL | success |

## 2. PR 주장

PR #1078의 핵심 주장은 다음과 같다.

```text
HTML table import 경로도 HWP5 CommonObjAttr raw_ctrl_data를 직접 생성한다.
이 경로의 raw_ctrl_data는 PR #1077과 같은 4바이트 offset 밀림 문제가 있다.
따라서 [0..4]=attr, [12..16]=width, [16..20]=height,
[24..32]=outer_margin, [32..36]=instance_id 순서로 저장해야 한다.
```

이 주장은 코드 기준으로 타당하다.

`html_table_import.rs`는 HWPX adapter를 통하지 않고 `Table.raw_ctrl_data`를 직접 만든다.
따라서 `hwpx_to_hwp` adapter의 `raw_ctrl_data.is_empty()` 보정 대상이 아니며, 생성 지점에서
CommonObjAttr layout을 맞춰야 한다.

### 2.1 TypeScript 클립보드 경로와 PR 수정 범위

PR #1078의 변경 파일은 다음 2개뿐이다.

```text
src/document_core/html_table_import.rs
src/wasm_api/tests.rs
```

즉 TypeScript 소스 변경은 이 PR에 포함되어 있지 않다.

다만 TypeScript 경로는 이 기능의 진입점이다.

```text
rhwp-studio/src/engine/input-handler-keyboard.ts:
  ClipboardEvent에서 text/html을 읽음
  wasm.pasteHtml(...) 또는 wasm.pasteHtmlInCell(...) 호출

rhwp-studio/src/core/wasm-bridge.ts:
  pasteHtml(...) / pasteHtmlInCell(...)을 wasm API로 전달

src/wasm_api.rs:
  pasteHtml / pasteHtmlInCell wasm binding

src/document_core/commands/html_import.rs:
  paste_html_native(...)
  HTML 내부 <table> 발견 시 parse_table_html(...)

src/document_core/html_table_import.rs:
  Table과 raw_ctrl_data 직접 생성
```

따라서 이 문제는 "TS에서 HTML clipboard를 잘못 전달한 문제"가 아니라,
TS가 정상 전달한 `text/html` 표를 Rust `parse_table_html()`이 HWP table control로 만들 때
`raw_ctrl_data`의 CommonObjAttr layout을 잘못 쓴 문제다.

## 3. 현재 local/devel 반영 상태

현재 `local/devel`에는 다음 커밋이 이미 반영되어 있다.

```text
55783a41 fix: align HTML table ctrl data offsets
```

이 커밋은 PR #1078의 핵심 수정과 같은 문제를 처리한다.

현재 코드 확인:

```text
src/document_core/html_table_import.rs:
  table_attr = 0x082A2311
  raw_ctrl_data[0..4]   = table_attr
  raw_ctrl_data[12..16] = total_width
  raw_ctrl_data[16..20] = total_height
  raw_ctrl_data[24..32] = outer_margin
  raw_ctrl_data[32..36] = instance_id
```

추가로 현재 `local/devel`의 테스트는 PR #1078보다 강하다.

```text
src/wasm_api/tests.rs:
  parse_common_obj_attr(&tbl.raw_ctrl_data)로 실제 parser layout을 검증
  common.attr == tbl.attr
  common.width/height == table column/row sum
  common.margin == table outer margin
  common.instance_id != 0
```

즉 단순 byte slice 검증이 아니라, 실제 HWP5 CommonObjAttr parser가 읽는 결과를 기준으로
회귀를 막는다.

## 4. PR #1078 그대로 수용 시 문제

PR #1078은 현재 `devel`과 mergeable 상태가 아니다.
또한 현재 `local/devel`에는 같은 수정이 이미 더 넓은 검증과 함께 들어가 있으므로,
PR 커밋을 그대로 cherry-pick하면 다음 문제가 생긴다.

```text
1. 이미 반영된 수정과 중복된다.
2. 현재 테스트가 PR 테스트보다 더 강하므로 PR 버전으로 되돌릴 이유가 없다.
3. PR branch base가 오래되어 직접 merge/cherry-pick 이득이 없다.
```

## 5. 판정

PR #1078의 문제 제기는 타당하다.

하지만 구현은 이미 `local/devel`과 `origin/devel`에 반영되었다.
따라서 이 PR은 추가 cherry-pick 없이 "이미 반영됨"으로 정리하는 것이 적절하다.

## 6. 권장 처리

```text
1. PR #1078에는 "문제 제기는 맞고, 동일 수정이 55783a41로 devel에 반영되었다"는 코멘트를 남긴다.
2. PR #1078은 close 처리한다.
3. 관련 이슈 #698은 PR #1077 처리 때 이미 close된 상태라면 추가 조치하지 않는다.
4. 별도 코드 변경은 하지 않는다.
```

## 7. 확인한 검증

PR #1078 처리 전 이미 `55783a41` 반영 과정에서 다음 검증을 통과했다.

```text
cargo fmt --check
cargo check
cargo test test_paste_html_table_as_control
cargo test test_parse_table_html_save
cargo test --lib
```

`cargo test --lib` 결과:

```text
1398 passed; 0 failed; 6 ignored
```
