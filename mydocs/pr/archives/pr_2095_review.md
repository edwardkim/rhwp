# PR #2095 검토 — 새 문단/셀 문단 서식 상속

- PR: https://github.com/edwardkim/rhwp/pull/2095
- 작성자: `physwkim`
- base/head: `devel` <- `fix/insert-paragraph-inherit-shape`
- 최초 검토 브랜치: `codex/pr2095-review-20260709`
- 최초 검토 기준 SHA: `86b0be18b64cfc8cc7b9b6f86f20d6be2ffc6eb5`
- 최신 재검토 기준 SHA: `5d4ca0f33fca099a238436994f5d735f70e872c7`
- reviewer assign: `jangster77` 완료
- GitHub review: `APPROVED` 제출
- merge: `27847162ca5edf999960ef45aa5d4f504e9bc626` (2026-07-09 16:15:23 UTC)
- 작성일: 2026-07-09
- 최신 갱신: 2026-07-10 KST

## 결론

**승인.** 최초 검토에서 확인한 문제 방향은 타당했으나, 당시 head `86b0be18`에서는 `create_table_native` /
`create_table_ex_native`가 혼합 글자모양 문단의 cursor offset이 아니라 `char_shapes.first()`를 상속해
blocking finding을 남겼다.

작성자가 후속 보완 커밋을 반영한 최신 head에서는 이 문제가 해소되었다. `create_table_native` /
`create_table_ex_native`는 `char_shape_id_at(char_offset)` 기준으로 바뀌었고, 같은 결함 계열인 글상자,
붙여넣기 빈 이웃 문단, copy control, 각주/미주 fallback도 cursor/control anchor 위치의 글자모양 기준으로
보강되었다. `Paragraph::new_empty_like`는 문단 끝 Enter 의미론에 맞게 마지막 글자모양을 상속한다.

최신 GitHub Actions 본 체크와 CodeQL이 모두 통과했고, PR #2095는 merge commit
`27847162ca5edf999960ef45aa5d4f504e9bc626`로 `devel`에 병합되었다.

## 변경 요약

- `Paragraph::new_empty_like(template)` 추가 및 `insert_paragraph_native`의 이웃 문단 서식 상속 적용.
- 표 뒤 빈 문단 생성 경로를 `new_empty_like`로 통합.
- `create_table_native` / `create_table_ex_native`의 셀 문단에 삽입 위치 글자모양을 명시.
- `Table::insert_row` / `Table::insert_column`의 병합 셀 fallback을 표의 임의 셀까지 확장.
- 리뷰 보완 후 `char_shapes.first()` 기반 상속을 cursor/control anchor 위치 기반으로 교정.
- cursor offset과 문단 첫 글자모양이 다른 혼합 글자모양 회귀 테스트 추가.

## 검증

- GitHub Actions: 최신 head 기준 CI/CodeQL 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib char_shape_inherit -- --nocapture`: 5 passed.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib new_empty_like -- --nocapture`: 2 passed.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib create_table -- --nocapture`: 17 passed.
- `wasm-pack build --target web --out-dir pkg`: 통과, `/Users/tsjang/rhwp/pkg` 갱신.
- `localhost:7700`: 기존 Vite dev server 재사용, HTTP 200.
- Puppeteer headless Chrome: `localhost:7700` Studio 앱에서 `window.__wasm` 경로로 지정 샘플 재검증.

## 지정 샘플

- 입력: `samples/issue1937_rowbreak_footnote_overpagination.hwp`
- 입력 SHA-256: `a3a075594994e4741a7fbe973bc230d95ae75fdee6578ac6017cec0d52844990`
- Studio load 결과: 52쪽.

## 최초 검토 결과

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

증적: `mydocs/pr/assets/pr_2095_issue1937_insert_uniform_browser_export.hwpx`

- SHA-256: `b37ae2d62c5f9bf7fa2c9a8190faa8d691c6f0176f8f52084eeab05474d5be04`

### 4쪽 기존 표 행 삽입

절차:

1. 샘플 로드.
2. `getPositionOfPage(3)`으로 4쪽 시작 위치 확인: `sec=0`, `para=37`, `charOffset=0`.
3. 4쪽 표 확인: parent para 37, control 0, 2행 3열.
4. 삽입 전 row 0/header 셀 글자모양 확인: `26, 26, 26`.
5. 삽입 전 row 1/body 셀 글자모양 확인: `25, 25, 24`.
6. `doc.insertTableRow(0, 37, 0, 0, true)` 실행.
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

증적: `mydocs/pr/assets/pr_2095_issue1937_page4_row_insert_browser_export.hwpx`

- SHA-256: `77d38c0a63906a09942050b0a5f1188267d4255143cd8e421f87a0415b6ce1a8`

### 최초 blocking finding

최초 head `86b0be18`에서 179번 혼합 글자모양 문단에 `doc.createTable(0, 179, 10, 2, 2)`를 실행하면,
cursor offset 10의 글자모양은 `37`인데 새 셀 문단은 문단 첫 엔트리 `34`를 상속했다.

```text
loadPageCount=52
afterEditPageCount=53
sourceFirstCharShapeId=34
sourceOffsetCharShapeId=37
sourceLastCharShapeId=28
tableCellCharShapeId=34
tableResult={"ok":true,"paraIdx":180,"controlIdx":0}
```

증적: `mydocs/pr/assets/pr_2095_issue1937_table_mixed_offset_browser_export.hwpx`

- SHA-256: `d53960927da2f01486b0a3c326874b65be4c9c6316590e27d7353e5d9993d9b2`

## 최신 재검증

최신 head 보완 후 같은 시나리오를 rebuilt WASM + `localhost:7700` Studio headless Chrome에서 재검증했다.

절차:

1. 샘플 로드.
2. 179번 문단의 글자모양 확인.
3. `doc.createTable(0, 179, 10, 2, 2)` 실행.
4. 반환된 표의 첫 셀 `getCellCharPropertiesAt(0, paraIdx, controlIdx, 0, 0, 0)` 확인.
5. HWPX export.

결과:

```text
pageCount=53
sourceFirstCharShapeId=34
sourceOffsetCharShapeId=37
sourceLastCharShapeId=28
tableResult={"ok":true,"paraIdx":180,"controlIdx":0}
tableCellCharShapeId=37
exported=117699 bytes
```

증적: `mydocs/pr/assets/pr_2095_issue1937_table_mixed_offset_browser_export_fixed.hwpx`

- SHA-256: `6ac5bec65eebccb0ad0eac34674f985d44f7cb4249cfd02e80d40f647901faf5`

## GitHub 리뷰 제출 내용

GitHub에 `APPROVED` review를 제출했다. 핵심 내용:

- 이전 blocking finding은 최신 head에서 해소됨.
- `create_table_native` / `create_table_ex_native`가 `char_shape_id_at(char_offset)` 기준으로 동작함.
- 글상자, 붙여넣기, copy control, 각주/미주 fallback도 같은 기준으로 보강됨.
- `samples/issue1937_rowbreak_footnote_overpagination.hwp` 179번 혼합 글자모양 문단에서 `34 / 37 / 28` 구간을 확인했고, `createTable(0, 179, 10, 2, 2)` 후 새 셀 문단이 `37`을 상속함을 확인.
- 최신 GitHub Actions 통과 후 merge 완료.

## 판단

Approve 및 merge 완료. 최초 blocking finding은 최신 head에서 해소되었고, 로컬 focused 검증,
WASM build, Studio headless Chrome 검증, GitHub Actions를 모두 통과했다.
