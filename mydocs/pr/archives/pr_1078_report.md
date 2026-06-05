# PR #1078 처리 결과 보고서

- PR: <https://github.com/edwardkim/rhwp/pull/1078>
- 제목: `fix: HTML 테이블 붙여넣기 raw_ctrl_data 오프셋 밀림 (#698 관련)`
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/698>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. 처리 결과

| 항목 | 결과 |
|---|---|
| PR 상태 | closed |
| 처리 방식 | 추가 cherry-pick 없이 close |
| 관련 이슈 #698 | 이미 closed, state_reason=`completed` |
| 코드 변경 | 없음 |
| 문서 변경 | `mydocs/pr/pr_1078_review.md`, `mydocs/pr/pr_1078_report.md` |

## 2. 판단 근거

PR #1078은 HTML table import 경로의 `raw_ctrl_data` CommonObjAttr 오프셋 밀림을 지적했다.
문제 제기는 타당하다.

하지만 동일 수정은 이미 `devel`에 반영되어 있었다.

```text
55783a41 fix: align HTML table ctrl data offsets
```

현재 `local/devel` 확인 결과:

```text
src/document_core/html_table_import.rs:
  raw_ctrl_data[0..4]   = table_attr
  raw_ctrl_data[12..16] = total_width
  raw_ctrl_data[16..20] = total_height
  raw_ctrl_data[24..32] = outer_margin
  raw_ctrl_data[32..36] = instance_id
```

테스트도 PR #1078보다 강한 방식으로 반영되어 있다.

```text
src/wasm_api/tests.rs:
  parse_common_obj_attr(&tbl.raw_ctrl_data)로 실제 parser layout 기준 검증
```

검증 항목:

```text
common.attr == tbl.attr
common.width/height == table column/row sum
common.margin == table outer margin
common.instance_id != 0
```

## 3. TypeScript 경로 확인

PR #1078에는 TypeScript 변경이 포함되지 않는다.

변경 파일:

```text
src/document_core/html_table_import.rs
src/wasm_api/tests.rs
```

다만 TypeScript는 외부 클립보드 HTML 붙여넣기의 진입점이다.

```text
rhwp-studio/src/engine/input-handler-keyboard.ts:
  ClipboardEvent에서 text/html 추출
  wasm.pasteHtml(...) / wasm.pasteHtmlInCell(...) 호출

src/document_core/commands/html_import.rs:
  paste_html_native(...)
  <table> 발견 시 parse_table_html(...)

src/document_core/html_table_import.rs:
  HWP Table control과 raw_ctrl_data 직접 생성
```

따라서 이 문제는 TypeScript가 HTML을 잘못 전달한 문제가 아니라,
Rust HTML table import가 HWP5 CommonObjAttr layout을 잘못 직렬화하던 문제다.

## 4. GitHub 처리

PR #1078에 다음 취지의 코멘트를 남기고 close 처리했다.

```text
문제 제기는 타당하지만 동일 수정은 55783a41로 devel에 이미 반영됨.
추가 cherry-pick 없이 중복 PR로 정리.
```

## 5. 후속 조치

추가 코드 변경은 필요하지 않다.

남은 작업:

```text
1. 이 보고서 승인
2. PR 검토/보고 문서 커밋
```
