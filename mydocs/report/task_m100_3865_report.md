---
kind: report
status: active
canonical: mydocs/report/task_m100_3865_report.md
last_verified: 2026-08-03
---

# #3865 처리 기록 — 표 셀 안 텍스트가 찾기에서 잡히지 않는다

- Issue: [#3865](https://github.com/edwardkim/rhwp/issues/3865) — 표 내부 텍스트 검색 불가 (제보: @anemochore)
- 브랜치 `task/3865-find-in-table-cells`

## 증상

웹 데모에서 1쪽 표 안의 문장(`배포 즉시 보도해 주시기 바랍니다.`)을 어떤 단어로 찾아도
"검색 결과 없음"이 나온다. 모든 페이지에서 재현된다.

## 근인 — 파싱이 아니라 찾기 경로의 필터

먼저 파싱·코어는 정상임을 확인했다.

- `search_all`(`src/document_core/queries/search_query.rs:131`)은 `Control::Table` 을 열어
  `table.cells → cell.paragraphs` 까지 내려가고 매치에 `cell_context` 를 붙인다.
- CLI `grep`(`src/document_core/queries/grep.rs:245`)도 같은 순회를 한다. 그래서
  `rhwp search 문서.hwp --json -- "배포 즉시"` 는 지금도 표 안을 찾는다.

문제는 웹 찾기 대화상자(`rhwp-studio/src/ui/find-dialog.ts`)가 쓰는 `search_text_native` 였다.
이 함수가 셀·글상자 매치를 **옵션 없이** 걸러냈다.

```rust
// 본문 결과만 필터 (셀/글상자 내부 제외 — 커서 이동 불가)
let body_hits: Vec<&SearchHit> = all_hits
    .iter()
    .filter(|h| h.cell_context.is_none() && h.equation_control.is_none())
    .collect();
```

주석이 밝힌 제외 사유는 "커서 이동 불가"다. **그 사유가 지금은 성립하지 않는다.** 그 뒤
편집기가 셀 좌표를 다루게 됐다.

- `DocumentPosition` 에 `parentParaIndex`·`controlIndex`·`cellIndex`·`cellParaIndex`·`cellPath`
  (`rhwp-studio/src/core/types.ts:836`)
- 셀 인지 커서 사각형 `getCursorRectInCell`(`rhwp-studio/src/core/wasm-bridge.ts:1253`)이 있고
  `cursor.ts:1175` 가 이미 쓴다
- `rhwpDev.goto`(`rhwp-studio/src/core/rhwp-dev.ts:118`)는 이미 검색 히트의 `cellContext` 를
  셀 좌표로 바꿔 커서를 옮긴다 — 즉 경로가 실재하고 동작한다

`format_search_hit` 은 원래부터 `cellContext` 를 실어 보내고 있었다. 막고 있던 것은 필터 하나뿐이었다.

## 수정

**코어 — 옵트인으로 연다(무회귀).**

- `search_text_native` 에 `include_cells: bool` 을 더하고 필터를 조건부로 바꾼다.
- wasm 바인딩 `searchText` 는 `Option<bool>` 로 받는다. 인자를 6개만 넘기던 기존 호출자는
  `None → false` 로 종전 동작 그대로다(외부 호환 유지).

**스튜디오 — 셀 좌표로 이동·치환한다.**

- `wasm-bridge.searchText` 에 `includeCells = false` 기본값 인자를 더한다.
- 찾기 대화상자는 `true` 로 부른다.
- `navigateToHit` 은 `cellContext` 가 있으면 셀 좌표 `DocumentPosition` 을 만든다. 이때
  `moveCursorTo` 는 우회한다 — 그 사전 검증이 본문 좌표만 보는 `getCursorRect` 라서 셀 위치를
  넘기면 바깥 문단을 검사해 엉뚱하게 거절될 수 있다. `rhwpDev.goto` 와 같은 경로(`cursor.moveTo`)를 쓴다.
- **치환도 함께 갈랐다.** `replaceText` 는 본문 좌표만 받으므로, 찾기가 셀 매치를 돌려주기
  시작한 뒤에도 그대로 두면 셀 매치를 바꿀 때 **표가 놓인 바깥 문단**이 고쳐진다. 셀 매치는
  `replaceTextInCellDeferredPagination` 으로 보낸다. 이 갈래가 없으면 이번 수정이 문서 손상
  경로를 새로 여는 셈이라, 같은 커밋에 넣어야 한다.

## 검증

- `tests/issue_3865_search_text_in_table_cells.rs` 추가 — 셀에만 있는 단어와 본문에만 있는
  단어를 각각 둔 HML 픽스처로 세 가지를 고정한다.
  1. 끄면 본문은 찾히고 셀은 안 찾힌다(종전 동작 유지 — 이게 무너지면 나머지 판정이 공허하다)
  2. 켜면 셀 단어가 찾히고 `cellContext` 의 네 좌표가 모두 실린다
  3. 켜도 본문 매치 결과는 글자 그대로 동일하다
- `rustfmt` 로 변경한 Rust 파일 포맷 확인. 작성 중 `br#"..."#`(바이트 문자열)에 한글을 넣어
  컴파일이 깨지는 것을 rustfmt 파싱이 잡아내 `&str` + `as_bytes()` 로 고쳤다.
- 이 PC 는 MSVC 링커(`dbghelp.lib`) 손상과 GNU `dlltool` 부재로 `cargo test` 가 아예 돌지
  않고, `rhwp-studio/node_modules` 도 없어 `tsc` 도 못 돌린다. **Rust 테스트와 프런트 타입
  검사는 CI 가 유일한 판정자다.**
- 브라우저 실동작(셀로 스크롤·캐럿 표시·치환 후 재검색)은 확인하지 못했다. 리뷰 시 그 부분을
  봐 주셔야 한다.

## 찾기 경로 두 개를 함께 고친 이유

찾기 진입점은 둘이다 — 대화상자(Ctrl+F)와 대화상자 없는 "찾기 다음"(F3, `edit.ts:153`).
처음엔 대화상자만 고치고 F3 는 후속으로 남기려 했는데, 그러면 **같은 문서에서 Ctrl+F 는
표 안을 찾는데 F3 는 못 찾는** 상태가 된다. 사용자가 원인을 짐작할 수 없는 종류의 불일치라
같이 고치는 것이 맞다.

이동 규칙이 양쪽에 복제돼 있던 것이 애초에 이 갈림의 원인이었다. `navigateToSearchHit` 로
빼내 `find-dialog` 와 `edit.ts` 가 같이 쓴다 — 앞으로 이동 규칙이 바뀌어도 한쪽만 바뀌는
일이 없다.
