# 구현계획 — task_m100_4272

- **Issue**: #4272
- **수행계획**: [task_m100_4272.md](task_m100_4272.md)
- **대상 브랜치**: `fix/issue-4272-nested-cell-text-selection`
- **기준 commit**: `828eabc19a4953a684e05d523a614256dae28b26`

## 1. Rust 선택 대상 일반화

`src/document_core/queries/cursor_nav.rs`에서 선택 대상 표현을 본문·평면 셀·경로 셀로 구분한다.

- 기존 `get_selection_rects_native()`는 호환 wrapper로 유지한다.
- 내부 공통 함수는 평면 셀 또는 `(parentPara, cellPath)`를 받아 동일한 페이지 계획과 사각형
  계산을 사용한다.
- 경로 셀은 중간 엔트리의 `cellParaIndex`까지 정확히 비교하고, 마지막 엔트리의 문단 인덱스만
  현재 선택 문단에 맞춘다.
- IR 문단 조회는 `resolve_paragraph_by_path()`를 사용한다.

## 2. WASM·Studio 경계

- `getSelectionRectsInCellByPath`와 page hint options 변형을 WASM에 노출한다.
- `selection-page-hints.ts`에 path query dispatch와 단위 테스트를 추가한다.
- `WasmBridge`에 경로 기반 메서드를 추가한다.
- `InputHandler.updateSelection()`은 시작·끝의 cell container path가 같은지 비교해 깊이 2 이상이면
  새 API를 호출한다. 깊이 1은 기존 API를 유지한다.

## 3. E2E·증적

- `issue-4272-nested-cell-text-selection.test.mjs`에서 물리 5쪽 `23,504`의 실제 bbox를 찾아
  브라우저 mouse drag를 수행한다.
- `23,504` 전체 selection offset `0 -> 6`, 깊이 3 경로, 한 drag event당 path API 최대 1회,
  highlight 1개 이상, 관련 console warning/error 0건을 고정한다.
- 결과 JSON과 screenshot은 `output/4272/`에 생성한다.

## 4. 검증 순서

1. focused Rust matcher/API 테스트
2. Studio selection dispatch 단위 테스트와 TypeScript 검사
3. `cargo fmt --all -- --check`, `git diff --check`
4. 프로젝트 표준 Docker WASM 빌드:
   `docker compose --env-file .env.docker run --rm wasm`
5. #4272 실제 browser E2E와 인접 #4252 E2E
6. 결과를 `mydocs/working/task_m100_4272_stage1.md`에 기록

## 5. Stage 2 — 선택 텍스트 복사·붙여넣기 완결

Stage 1 시각 검증에서 확인된 동일 사용자 여정을 별도 이슈로 분리하지 않고 #4272 안에서
완결한다.

- `copySelectionInCellByPath` native/WASM API를 추가해 선택 문단마다 마지막
  `cellParaIndex`만 바꿔 실제 안쪽 셀 문단을 복사한다.
- `exportSelectionInCellHtmlByPath`를 같은 경로 계약으로 추가해 시스템 클립보드의 HTML도
  바깥 셀로 퇴행하지 않게 한다.
- Studio `onCopy`는 중첩 셀에서 두 path API를 사용하고, 깊이 1은 기존 API를 유지한다.
- path API의 시작·끝 문단 인덱스는 평면 호환 필드가 아니라 `cellParaIndexOf()`로 전체 path의
  마지막 엔트리에서 읽는다.
- 실제 샘플 Rust 래칫은 plain text `23,504`와 HTML fragment를 검증한다.
- CDP E2E는 실제 mouse drag → Ctrl+C → Ctrl+V를 수행해 내부 클립보드와 최종 셀 텍스트가
  `23,504`로 보존되는지 확인한다.
- 물리 11쪽 자식 표 문단 22의 offset `66 -> 89` 선택을 별도 CDP 래칫으로 두어 평면
  `cellParaIndex=0` 퇴행을 차단한다.
- 복사 이벤트는 드래그 hot path가 아니며 문서 전체 순회·페이지네이션을 추가하지 않는다.

## 6. Stage 3 — 중첩 표 객체 복사

깊이 3 자식 표 객체 선택 참조의 `cellPath`는 표의 각 셀까지 내려가는 경로다. 선택된 표 control은
마지막 엔트리의 `controlIndex`이고, 그 control을 소유한 문단은 마지막 엔트리를 제외한 prefix
path가 가리킨다.

- `tableObjectClipboardTarget()` 순수 헬퍼로 선택 참조를
  `(ownerCellPathJson, selectedControlIndex)`로 변환한다.
- 키보드 Ctrl+C와 컨텍스트 메뉴·도구 상자의 `performCopy()`가 같은 헬퍼를 사용한다.
- 본문 표는 기존 빈 owner path와 `ref.ci` 계약을 유지한다.
- 실제 fixture의 깊이 3 경로에서 owner path depth 2, selected control index 0으로
  `copyControl`·`exportControlHtml`이 호출되는지 Rust·Studio·CDP 래칫으로 검증한다.
- 객체 선택 렌더링과 페이지네이션은 변경하지 않는다.
