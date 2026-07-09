# PR #2095 검토 — 새 문단/셀 문단 서식 상속

- PR: https://github.com/edwardkim/rhwp/pull/2095
- 작성자: physwkim
- base/head: `devel` <- `fix/insert-paragraph-inherit-shape`
- 검토 브랜치: `codex/pr2095-review-20260709`
- 검토 기준 SHA: `86b0be18b64cfc8cc7b9b6f86f20d6be2ffc6eb5`
- reviewer assign: `jangster77` 완료
- 작성일: 2026-07-09

## 결론

**수정 요청.** PR 이 지적한 문제 방향은 맞고, 로컬 Rust 단위 검증과 GitHub CI 는 통과했다. `wasm-pack build --target web --out-dir pkg` 후 실제 Studio 브라우저(`localhost:7700`)에서 지정 샘플 `issue1937_rowbreak_footnote_overpagination.hwp` 를 검증한 결과, 단일 글자모양 문단 뒤 새 문단 삽입과 4쪽 기존 표의 행 삽입은 정상 상속된다. 그러나 혼합 글자모양 문단의 cursor offset 에 표를 삽입하면 새 셀 문단이 cursor 위치 글자모양이 아니라 문단 첫 글자모양을 상속한다. 따라서 PR 의 핵심 주장 중 새 표 생성 시 셀 문단의 "현재 위치 서식 상속"은 아직 성립하지 않는다.

## 변경 요약

- `Paragraph::new_empty_like(template)` 추가.
- `insert_paragraph_native` 가 앞 문단, 또는 `para_idx == 0` 일 때 현재 0번 문단을 상속원으로 사용.
- `create_table_native` / `create_table_ex_native` 의 새 셀 문단에 삽입 문단의 `char_shapes.first()` 값을 설정.
- `Table::insert_row` / `insert_column` 의 병합 셀 fallback 을 표의 임의 셀까지 확장.

## 검증

- GitHub Actions: PR #2095 head 기준 `CI` / `CodeQL` 모두 통과. 워크플로 조건상 `WASM Build` job 은 skipped.
- 로컬 target cleanup: `/Users/tsjang/rhwp/target` 하위 항목 삭제 후 focused test 재실행.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib insert_paragraph_ -- --nocapture`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib new_empty_like -- --nocapture`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib cells_inherit_char_shape -- --nocapture`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib inherits_shape_when -- --nocapture`: 통과.
- `git diff --check devel...HEAD`: 통과.
- `cargo fmt --check`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib`: 2163 passed, 0 failed, 7 ignored.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과.
- `wasm-pack build --target web --out-dir pkg`: 통과, `/Users/tsjang/rhwp/pkg` 갱신.
- `localhost:7700`: 기존 Vite dev server 재사용, HTTP 200.
- Browser plugin: 대상 페이지 접속은 가능했으나 DOM snapshot 이 `incrementalAriaSnapshot` 런타임 오류로 실패했고, page evaluate 에서 module import 가 허용되지 않아 WASM API 직접 검증에는 Puppeteer fallback 사용.
- Puppeteer headless Chrome: 동일한 `localhost:7700` Studio 앱에서 `window.__wasm` 로 지정 샘플 시나리오 실행.

## 지정 샘플 검증

- 입력: `samples/issue1937_rowbreak_footnote_overpagination.hwp`
- 입력 SHA-256: `a3a075594994e4741a7fbe973bc230d95ae75fdee6578ac6017cec0d52844990`
- Studio load 결과: 52쪽, 1구역, 415문단.

### 단일 글자모양 문단 뒤 새 문단 삽입

절차:

1. 샘플 로드.
2. 7번 문단 `소상공인ㆍ취약계층에 대한 금전납부의무 부담완화 법령 정비방안 연구` 확인.
3. 7번 문단 첫/마지막 글자모양 확인: 둘 다 `charShapeId=87`.
4. `doc.insertParagraph(0, 8)` 실행.
5. 8번 문단에 `PR2095_SAMPLE_INSERT` 입력.
6. HWPX export.

결과:

```text
loadPageCount=52
afterEditPageCount=49
sourceFirstCharShapeId=87
sourceLastCharShapeId=87
insertedCharShapeId=87
insertResult={"ok":true,"paraIdx":8,"newParagraphCount":416}
```

export XML 에서도 marker run 은 `charPrIDRef="87"` 이다. 즉 문단 전체가 같은 글자모양인 샘플 지점에서는 PR 변경이 의도대로 동작한다.

증적: `mydocs/pr/assets/pr_2095_issue1937_insert_uniform_browser_export.hwpx`

- SHA-256: `b37ae2d62c5f9bf7fa2c9a8190faa8d691c6f0176f8f52084eeab05474d5be04`

### 4쪽 기존 표 행 삽입

절차:

1. 샘플 로드.
2. `getPositionOfPage(3)` 으로 4쪽 시작 위치 확인: `sec=0`, `para=37`, `charOffset=0`.
3. 4쪽 표 확인: parent para 37, control 0, 2행 3열.
4. 삽입 전 row 0/header 셀 글자모양 확인: `26, 26, 26`.
5. 삽입 전 row 1/body 셀 글자모양 확인: `25, 25, 24`.
6. `doc.insertTableRow(0, 37, 0, 0, true)` 로 row 0 아래에 행 삽입.
7. 새 row 1 및 기존 row 1이 밀린 row 2의 셀 글자모양 확인.
8. HWPX export.

결과:

```text
loadPageCount=52
afterEditPageCount=53
beforeDims={"rowCount":2,"colCount":3,"cellCount":6}
beforeRows=row0 [26,26,26], row1 [25,25,24]
insertResult={"ok":true,"rowCount":3,"colCount":3}
afterDims={"rowCount":3,"colCount":3,"cellCount":9}
insertedRow=row1 [25,25,24]
shiftedOriginalRow=row2 [25,25,24]
```

4쪽 기존 표의 row insert 경로는 PR 의 의도대로 통과한다. 새 row 1은 기존 body row 의 글자모양 `25,25,24` 를 보존했고, export XML 에서도 table `rowCnt="3"` 및 새 body row의 `charPrIDRef="25"`, `25`, `24` 가 확인된다.

증적: `mydocs/pr/assets/pr_2095_issue1937_page4_row_insert_browser_export.hwpx`

- SHA-256: `77d38c0a63906a09942050b0a5f1188267d4255143cd8e421f87a0415b6ce1a8`

### 혼합 글자모양 문단의 cursor offset 에 표 삽입

절차:

1. 샘플 로드.
2. 179번 문단 `ㅇ 부담금은 인적 ...` 확인.
3. 같은 문단의 글자모양 확인:
   - offset 0: `charShapeId=34`
   - offset 10: `charShapeId=37`
   - 마지막: `charShapeId=28`
4. `doc.createTable(0, 179, 10, 2, 2)` 실행.
5. 반환된 표의 첫 셀 `getCellCharPropertiesAt(0, paraIdx, controlIdx, 0, 0, 0)` 확인.
6. HWPX export.

결과:

```text
loadPageCount=52
afterEditPageCount=53
sourceFirstCharShapeId=34
sourceOffsetCharShapeId=37
sourceLastCharShapeId=28
tableCellCharShapeId=34
tableResult={"ok":true,"paraIdx":180,"controlIdx":0}
```

이 샘플은 PR 의 `char_shapes.first()` 문제가 실제 문서에서도 재현됨을 보여준다. 표 삽입 API 는 cursor `char_offset=10` 을 받지만, 새 셀 문단은 cursor 위치의 `37` 이 아니라 문단 첫 엔트리 `34` 를 상속한다.

증적: `mydocs/pr/assets/pr_2095_issue1937_table_mixed_offset_browser_export.hwpx`

- SHA-256: `d53960927da2f01486b0a3c326874b65be4c9c6316590e27d7353e5d9993d9b2`

## Blocking finding

### [P1] 표 삽입 시 cursor offset 의 글자모양이 아니라 문단 첫 글자모양이 셀 문단에 상속됨

관련 위치:

- `src/document_core/commands/object_ops/table.rs:458` — `create_table_native` 가 `current_para.char_shapes.first()` 를 기본 글자모양으로 사용.
- `src/document_core/commands/object_ops/table.rs:874` — `create_table_ex_native` 도 동일.

지정 샘플의 179번 문단은 한 문단 안에 여러 글자모양이 섞여 있다. 실제 Studio 브라우저에서 `doc.createTable(0, 179, 10, 2, 2)` 를 실행하면 API 는 cursor `char_offset=10` 을 받으며, 이 위치의 글자모양은 `charShapeId=37` 이다. 그러나 생성된 첫 셀 문단의 `getCellCharPropertiesAt(...)` 결과는 `34` 로, 문단 첫 글자모양을 상속한다.

추가된 Rust 테스트들은 템플릿 문단을 직접 `char_shapes=[(0, 7)]` 로 구성하므로 cursor offset 과 `char_shapes.first()` 가 다른 케이스를 잡지 못한다. 단일 글자모양 문단에서는 통과하지만, 실제 문서의 혼합 글자모양 문단에서는 "현재 위치 서식"이 아니라 첫 엔트리가 복사된다.

권장 보완:

- `create_table_native` / `create_table_ex_native` 도 현재 문단의 첫 글자모양이 아니라 커서/삽입 offset 기준 글자모양을 상속해야 한다.
- 회귀 테스트에 "문단 첫 글자모양과 cursor offset 글자모양이 다른 경우"를 추가해야 한다.
- 가능하면 Studio/WASM export 경로의 최소 검증도 추가해야 한다.

## GitHub 리뷰 초안

```text
핵심 문제 방향은 맞고, CI 및 로컬 Rust 검증은 통과했습니다. 다만 merge 전 수정이 필요합니다.

`wasm-pack build --target web --out-dir pkg` 후 실제 `localhost:7700` Studio 브라우저에서 `samples/issue1937_rowbreak_footnote_overpagination.hwp` 로 확인했습니다.

- 단일 글자모양인 7번 제목 문단 뒤 새 문단 삽입은 `charShapeId 87 -> 87` 로 정상 상속됩니다.
- 4쪽 기존 표(parent para 37)에서 row 0 아래에 행을 삽입하면 새 row 1은 기존 body row 의 `25,25,24` 를 그대로 상속합니다.
- 그러나 179번 혼합 글자모양 문단에서 `doc.createTable(0, 179, 10, 2, 2)` 를 실행하면 cursor offset 10 의 글자모양은 `37` 인데 새 셀 문단은 문단 첫 엔트리 `34` 를 상속합니다.

원인은 table 생성 경로가 `char_shapes.first()` 를 상속 기준으로 쓰는 점입니다. 현재 테스트는 템플릿 문단을 `[(0, 7)]` 로 직접 구성해 cursor offset 과 첫 글자모양이 다른 케이스를 놓칩니다.

`create_table_native` / `create_table_ex_native` 가 커서/삽입 offset 기준 글자모양을 상속하도록 보완하고, 문단 첫 글자모양과 cursor offset 글자모양이 다른 회귀 테스트를 추가해 주세요.
```

## 판단

현재 상태로는 approve/merge 하지 않는다. 작성자 수정 후 rebuilt WASM + 지정 샘플 브라우저 export 를 다시 확인한다.
