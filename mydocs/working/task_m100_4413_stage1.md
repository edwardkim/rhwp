# task_m100_4413 Stage 1 — 문서 간 복사에서 셀 안 중첩 표·이미지 소실

- **이슈**: [#4413](https://github.com/edwardkim/rhwp/issues/4413)
- **브랜치**: `fix/issue-4413-clipboard-cell-controls`
- **분기 기준**: `upstream/devel` (0 behind)
- **상태**: 게이트 전부 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 결함

클립보드 HTML 내보내기가 셀 내용을 `para.text` 만으로 만든다. `para.controls` 를 보지 않으므로
**셀 안의 중첩 표와 이미지가 통째로 사라진다.** 사용자는 붙여넣기 결과를 보고서야 안다.

같은 결함이 세 진입점에 있었다 — `export_selection_in_cell_html_native`,
`export_selection_in_cell_html_by_path_native`(#4272 로 나중에 추가된 두 번째 사례),
`table_to_html` 의 셀 루프.

## 2. import 쪽은 더 나빴다 — 조용한 손실이 아니라 패닉

`src/document_core/html_table_import.rs:48` 의 `<td>`/`<th>` 내용 경계 추출이 순진한 `.find()` 라
같은 태그가 중첩되면 **범위를 벗어난 슬라이스로 패닉한다.**

```
thread panicked at src/document_core/html_table_import.rs:48:46:
byte index 18 is out of bounds of `<tr><td>inner`
```

이미 같은 파일에 `<tr>` 경계용으로 쓰는 깊이 추적 헬퍼 `find_closing_tag` 가 있었다. 그것을
재사용해 닫았다.

## 3. 고친 것

- `cell_paragraph_to_html` 신설 — 텍스트와 `para.controls` 를 함께 처리하고, 위 세 진입점에 연결.
- 중첩 표 재귀에 `MAX_NEST_DEPTH = 8` — `table_extract`/`explain`/`hidden_text` 와 같은 값·같은 모양.
- 지원하지 않는 셀 컨트롤 변형은 조용히 사라지는 대신 컨트롤 종류를 밝힌 HTML 주석 경고를 남긴다.
- `html_table_import.rs` 의 경계 추출을 `find_closing_tag` 로 교체.

## 4. 검증 (완료)

RED 확인은 패치를 떼어내고 했다 — `clipboard.rs`/`html_table_import.rs` 를 `upstream/devel` 로
되돌리고 테스트만 얹어 돌렸다.

```
nested_table_in_cell_is_exported_to_html ... FAILED
  실제 HTML: <html><body>\n<!--StartFragment-->\n<!--EndFragment-->\n</body></html>
picture_in_cell_is_exported_to_html ... FAILED
nested_table_in_cell_round_trips_through_import ... FAILED (패닉)
```

대조군 2건(`internal_clipboard_preserves_nested_table_in_cell_control_group`,
`body_level_picture_export_via_control_html_was_already_fine`)은 수정 전에도 통과했다 — 결함
범위가 셀 경로에 한정됨을 확인한 것이다.

- 신규 회귀 테스트 8건, 전부 RED → GREEN.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` **exit 0**,
  `test result:` 블록 497개, `FAILED` 0건. 로그 8,530줄.
  상습 타이밍 flake 인 `scan_cost_stays_linear_as_input_grows` 도 이번 회차는 통과.
  (첫 시도에서 `| tail -400` 을 걸어 exit code 가 `tail` 것이 되는 실수를 했고, 파이프 없이 다시
  돌려 얻은 결과다.)
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과.

## 5. 이 작업에서 고치지 않은 것

전부 [#4426](https://github.com/edwardkim/rhwp/issues/4426) 으로 열었다.

1. `table_to_html`(`clipboard.rs:1705`)이 `table.caption` 을 방출하지 않는다 — 최상위·중첩 모두.
2. `picture_to_html`(`:1816`)이 `pic.caption` 을 방출하지 않는다.
3. `export_selection_html_native`(`:1315`, 루프 `:1347`)는 여전히 `.controls` 를 안 보는
   `paragraph_to_html` 을 부른다 — 본문 텍스트 범위 선택이 인라인 앵커를 걸치면 같은 손실.
4. `picture_to_html` 이 `bin_data_id == 0` 이거나 `BinDataContent` 가 없으면 경고 없이 빈 문자열.

`MAX_NEST_DEPTH = 8` 을 또 한 벌 추가한 것 자체가 #4422(소유자 집합 6벌 중복, 이제 7벌)의
사례를 하나 늘린 것이다. 이 PR 에서 합치지 않았다 — 합치는 것은 #4422 의 일이다.

## 6. 미처리

GitHub Actions, 작업지시자 승인, merge. `clipboard.rs` 를 #4412(PR #4416)·#4414 와 공유하므로
머지 순서에 따라 리베이스가 필요하다.
