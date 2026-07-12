# PR #2095 검토 처리 계획

- 대상 PR: https://github.com/edwardkim/rhwp/pull/2095
- 최초 검토 기준 SHA: `86b0be18b64cfc8cc7b9b6f86f20d6be2ffc6eb5`
- 최신 재검토 기준 SHA: `5d4ca0f33fca099a238436994f5d735f70e872c7`
- 상태: `MERGED`
- merge commit: `27847162ca5edf999960ef45aa5d4f504e9bc626`
- 작성일: 2026-07-09
- 최신 갱신: 2026-07-10 KST

## Stage 1 — 최초 리뷰

- reviewer assign 완료.
- PR diff 및 설명 확인 완료.
- 렌더/PDF visual sweep 대상은 아님.
- WASM/Studio 편집 저장 결과에 직접 영향을 주므로 브라우저 런타임 검증 대상으로 분류.

## Stage 2 — 최초 로컬/브라우저 검증

- Rust focused test, lib test, fmt, clippy 완료.
- `wasm-pack build --target web --out-dir pkg` 완료.
- 기존 `localhost:7700` Vite dev server 재사용.
- Puppeteer headless Chrome으로 실제 Studio 앱의 `window.__wasm` 경로 검증 완료.
- 지정 샘플 `samples/issue1937_rowbreak_footnote_overpagination.hwp` 검증 완료.
- 결과: 단일 글자모양 문단 삽입과 4쪽 표 row insert는 정상, 혼합 글자모양 문단 offset 10 표 삽입은 `37` 대신 `34`를 상속해 blocking finding으로 판정.

## Stage 3 — 작성자 보완 확인

- 작성자 보완으로 `create_table_native` / `create_table_ex_native`가 cursor offset 기준 글자모양을 상속하도록 수정됨.
- 같은 결함 계열의 글상자, 붙여넣기 빈 이웃 문단, copy control, 각주/미주 fallback도 보강됨.
- `Paragraph::new_empty_like`는 문단 끝 Enter 의미론에 맞게 마지막 글자모양 상속으로 조정됨.
- cursor offset과 문단 첫 글자모양이 다른 회귀 테스트가 추가됨.

## Stage 4 — 최신 재검증

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib char_shape_inherit -- --nocapture`: 5 passed.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib new_empty_like -- --nocapture`: 2 passed.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib create_table -- --nocapture`: 17 passed.
- `wasm-pack build --target web --out-dir pkg`: 통과.
- `localhost:7700` Studio headless Chrome 재검증:
  - 입력: `samples/issue1937_rowbreak_footnote_overpagination.hwp`
  - `sourceFirstCharShapeId=34`
  - `sourceOffsetCharShapeId=37`
  - `sourceLastCharShapeId=28`
  - `doc.createTable(0, 179, 10, 2, 2)` 후 `tableCellCharShapeId=37`
  - 증적: `mydocs/pr/assets/pr_2095_issue1937_table_mixed_offset_browser_export_fixed.hwpx`

## Stage 5 — GitHub review

- 작업지시자 승인 후 GitHub `APPROVED` review 제출 완료.
- review 본문에는 이전 blocking finding 해소, 최신 로컬/WASM/브라우저 검증 결과, 최신 CI 완료 후 merge 조건을 기록.

## Stage 6 — merge 및 후속 기록

- 최신 GitHub Actions 본 체크와 CodeQL 통과 확인.
- PR #2095 merge 완료: `27847162ca5edf999960ef45aa5d4f504e9bc626`.
- 옵션 2 경로에 따라 review 문서와 검증 asset은 별도 docs-only fast-pass PR로 보존한다.
- PR #2095 본문은 별도 `Closes #N` 이슈를 지정하지 않아 추가 issue close 대상은 없다.
