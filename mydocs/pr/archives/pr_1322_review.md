# PR #1322 검토 — 글상자 위 이미지 plane/히트테스트 + 삽입 글상자 floating 교정 (#1280)

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1322
- 작성자: `johndoekim`
- 상태: open / draft 아님
- base: `devel`
- head: `local/task1280` (`6e0390ea`)
- 이슈: #1280

## 2. 변경 요약

PR은 두 묶음을 포함한다.

1. #1280 본편
   - rhwp-studio에서 삽입한 글상자가 `text_box` 없는 Rectangle로 생성되어 커서 진입/타이핑/붙여넣기가 실패하던 문제 수정.
   - `enterTextboxPlacementMode()`가 `shapeType: 'textbox'`를 전달하도록 수정.
   - 백엔드 계약 테스트와 e2e 테스트 추가.

2. #1280 v2
   - 글상자 위 이미지 배치가 한컴과 다른 plane 문제를 수정.
   - 렌더 정렬키 `(plane, zOrder, stableIndex)`를 layout query에 노출하고, 프런트 hit-test가 겹침 후보 중 최상단 개체를 선택하도록 변경.
   - 삽입 글상자 기본값을 한컴 실측값에 맞춰 floating(`treatAsChar=false`) + `InFrontOfText`로 교정.
   - 권위 샘플 `samples/textbox-under-image.hwp`와 관련 e2e 추가.

## 3. GitHub 상태

- PR upstream base SHA: `bbad6b0f`
- 현재 로컬 `devel`: `ea234037`
- PR upstream branch는 현재 `devel`보다 오래된 base 위에 있어, 그대로 비교하면 #1133/#1314 관련 파일이 되돌아가는 diff가 생긴다.
- 실제 검토는 현재 `local/devel`에서 `local/pr1322-integration` 브랜치를 만들고 PR의 non-merge 작업 커밋 14개를 cherry-pick하여 진행했다.

GitHub Actions:

- CI: success (`27087492006`)
- Render Diff: success (`27087492008`)
- CodeQL: success (`27087492005`)

## 4. 로컬 적용 방식

검토 브랜치:

```text
local/pr1322-integration
```

적용 방식:

```text
local/devel 기준으로 PR 작업 커밋 14개 cherry-pick
```

현재 `local/devel` 기준 실제 변경 범위:

- 문서 14개 추가
- e2e 4개 추가
- `samples/textbox-under-image.hwp` 추가
- studio 타입/히트테스트/글상자 삽입 경로 수정
- Rust object_ops/rendering/layout 수정

## 5. 코드 검토 메모

확인된 핵심 정합:

- `src/renderer/layout.rs`의 `paper_node_sort_key()`를 `pub(crate)`로 열어 렌더 정렬키를 layout query와 공유한다.
- `src/document_core/queries/rendering.rs`가 `plane`, `zOrder`, `stableIndex`, `wrap`을 JSON에 노출한다.
- `rhwp-studio/src/engine/input-handler-picture.ts`가 1차 hit-test에서 첫 적중이 아니라 `(plane, zOrder, stableIndex)` 최댓값을 선택한다.
- `#1171` nested picture 우선 패스와 `#516` BehindText 폴백 패스는 유지된다.
- 삽입 글상자는 `shapeType: 'textbox'`, `treatAsChar: false`, `textWrap: 'InFrontOfText'`를 명시한다.

주의점:

- PR upstream base가 오래되어 merge 전에 최신 `devel` 기준 rebase 또는 maintainer-side integration merge가 필요하다.
- `BehindText` 2차 패스는 여러 behind 객체가 겹칠 때 topmost 비교를 하지 않고 기존 첫 적중 반환 방식을 유지한다. 이번 PR의 주 대상은 non-behind topmost hit-test이므로 보류 가능하지만 후속 정리 여지는 있다.
- contributor 작업 문서가 다수 포함되어 있으므로 최종 수용 시 문서 포함 정책 판단이 필요하다.

## 6. 로컬 검증

통과:

- `cargo fmt --all -- --check`
- `cargo test issue_1280 -- --nocapture`
- `cargo test --test issue_1171_textbox_picture_cellpath -- --nocapture`
- `cargo test issue1151 -- --nocapture`
- `cargo test task1280 -- --nocapture`
- `cargo test --lib` — 1610 passed, 0 failed, 6 ignored
- `cargo clippy -- -D warnings`
- `cd rhwp-studio && npx tsc --noEmit`
- `docker compose --env-file .env.docker run --rm wasm`

e2e 검증:

- `VITE_URL=http://localhost:7701 node e2e/topmost-hittest.test.mjs --mode=headless`
- `VITE_URL=http://localhost:7701 node e2e/issue-1280-textbox-text-input.test.mjs --mode=headless`
- `VITE_URL=http://localhost:7701 node e2e/textbox-insert-floating-1280v2.test.mjs --mode=headless`
- `VITE_URL=http://localhost:7701 node e2e/topmost-lifecycle.test.mjs --mode=headless`
- `VITE_URL=http://localhost:7701 node e2e/textbox-picture-1171.test.mjs --mode=headless`
- `VITE_URL=http://localhost:7701 node e2e/textbox-picture-ops-1273.test.mjs --mode=headless`

비고:

- 최초 `topmost-hittest`는 기존 public WASM이 Rust layout query 변경을 반영하지 않아 `plane/zOrder/stableIndex`가 `undefined`로 내려오며 실패했다.
- WASM 재빌드 후 `pkg/` 산출물을 `rhwp-studio/public/`에 동기화하자 동일 e2e가 통과했다.
- Vite 서버는 sandbox 포트 바인딩 제한 때문에 외부 실행으로 띄웠고, 7700이 사용 중이라 `http://localhost:7701`에서 검증했다.

## 7. 메인테이너 추가 확인 및 보강

메인테이너 동작 테스트 중 추가 확인:

- 예제 파일의 글상자 안에 커서를 위치시킨 뒤 이미지 삽입 영역을 마우스로 지정하고 릴리즈해도 이미지가 삽입되지 않음.
- 웹 콘솔에는 오류가 출력되지 않음.

원인:

- studio `finishImagePlacement()`가 `hit.isTextBox`인 경우 `cellPath`를 의도적으로 비워 body paragraph sibling floating으로 전달하던 예전 분기를 유지하고 있었다.
- Rust `insert_picture_native()`도 `cellPath`가 있으면 `resolve_cell_by_path()`로 표 전용 검증을 수행하여, 글상자 sentinel path를 내부 삽입 경로로 사용할 수 없었다.

보강:

- `insert_picture_native()`에서 `cellPath` 마지막 엔트리가 `Shape(text_box)`인지 먼저 판정한다.
- 표 셀 path는 기존 #1151 계약대로 parent paragraph sibling floating을 유지한다.
- 글상자 path는 `resolve_cell_paragraph_mut()`로 text_box 내부 paragraph를 찾아 `Control::Picture`를 직접 추가한다.
- studio는 글상자 hit에서도 `cellPath`를 전달하고, drag 시작점을 글상자 bbox + text box margin 기준 내부 좌표로 변환해 전달한다.

추가 검증:

- `cargo test issue_1280 -- --nocapture` — 새 `insert_picture_into_textbox_uses_textbox_paragraph_control` 포함 8 passed
- `cargo test --test issue_1171_textbox_picture_cellpath -- --nocapture` — 3 passed
- `cargo test issue1151 -- --nocapture` — 3 passed
- `cd rhwp-studio && npx tsc --noEmit`
- `docker compose --env-file .env.docker run --rm wasm`
- `pkg/` 산출물을 `rhwp-studio/public/`에 동기화

메인테이너 동작 판정:

- rhwp-studio에서 글상자 안 이미지 삽입 drag-release 동작 테스트 통과.

## 8. 현재 권장

로컬 코드 검증, headless e2e, 메인테이너 동작 판정이 통과했다.

다음 권장 절차:

1. PR comment로 로컬 검증 결과와 수용 방침을 남긴다.
2. PR #1322를 merge하고, 현재 `devel` 기준 통합 커밋을 원격 `devel`에 반영한다.
