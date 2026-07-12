# PR #2039 리뷰 기록

작성일: 2026-07-08
대상 PR: https://github.com/edwardkim/rhwp/pull/2039
관련 이슈: https://github.com/edwardkim/rhwp/issues/2037

## 1. 메타

| 항목 | 내용 |
| --- | --- |
| 제목 | Task #2037: 찾아 바꾸기/모두 바꾸기를 편집 라우터 snapshot 으로 기록 |
| 작성자 | `lpaiu-cs` |
| 작성자 상태 | GitHub 기준 `FIRST_TIME_CONTRIBUTOR` |
| base | `devel` |
| head | `lpaiu-cs:fix/issue-2037` |
| head SHA | 문서 작성 시점 참고값: `4ff9aaf74a6fa5f506ae0264a9a37210b1228806` |
| 규모 | 2 files, +80/-9 |
| maintainer can modify | true |
| reviewer assign | `jangster77` 요청 등록 완료 |
| merge 결과 | `7649e2180c6f9b823327081e48d897ad50806301` 로 merge 완료 |

## 2. 변경 범위

- `rhwp-studio/src/ui/find-dialog.ts`
  - `doReplace()` / `doReplaceAll()` 의 직접 `wasm.replaceText` / `wasm.replaceAll` 호출을 `InputHandler.executeOperation({ kind: 'snapshot' })` 경로로 우회시킨다.
  - `InputHandler` 가 없는 환경에서는 기존 직접 호출 + `document-changed` emit fallback 을 유지한다.
  - 라우터 경로에서는 `executeOperation` 의 `refreshAfterOperation(..., 'full', ...)` 이 `afterEdit()` 를 호출하므로 별도 `document-changed` emit 을 하지 않는다.

- `rhwp-studio/tests/find-replace-undo.test.ts`
  - find dialog 소스를 검사해 `replaceText` / `replaceAll` 이 snapshot 라우터를 통과하는지 확인하는 회귀 테스트 2건을 추가한다.

## 3. 관련 이슈 판단

#2037 의 핵심 문제는 찾아 바꾸기/모두 바꾸기가 문서를 mutate 하면서도 undo stack 에 기록되지 않는 것이다.
현재 PR 은 기존 `objectProps`, `pasteImage` 계열과 같은 snapshot 기반 라우터 경로를 사용하므로 #1320 편집 라우터 계약에 맞는 방향이다.

`doReplaceAll()` 의 0건 치환에서도 snapshot no-op 이 남을 수 있다는 PR 본문의 tradeoff 는 기존 snapshot 경로의 동작과 일관되며, 이번 PR 의 merge blocker 로 보지 않는다.

## 4. 렌더 영향 및 visual sweep

- 렌더러, Canvas paint, PDF/SVG 출력, pagination 경로 변경 없음.
- `rhwp-studio` find/replace UI 편집 라우팅 변경이며, PR 의 사용자-visible 핵심은 undo 기록 여부다.
- 기준 PDF, HWP 2020 MCP 산출, visual sweep 은 필요하지 않다.

## 5. 코드 리뷰 결과

블로커 발견 없음.

확인한 근거:

- `InputHandler.executeOperation` 의 snapshot 경로는 `SnapshotCommand` 를 만들고 `history.execute()` 를 호출하므로 undo/redo stack 에 기록된다.
- snapshot 경로는 `refreshAfterOperation(..., 'full', ...)` 을 거쳐 `afterEdit()` 를 호출하고, 이 경로에서 `document-changed` emit 이 발생한다.
- `find-dialog.ts` 는 `getInputHandler()` 가 null 인 환경에서만 기존 직접 wasm 호출 fallback 을 유지하므로 테스트/특수 주입 환경 호환성을 유지한다.
- form mode 에서는 snapshot 작업이 `isOperationAllowedInEditMode()` 에서 차단된다. 찾아 바꾸기의 문서 mutation 도 form mode 제한을 따르는 것이 현재 편집 라우터 계약과 맞다.

커버리지 보완 메모:

- 신규 테스트는 실제 DOM/WASM 통합 동작을 실행하지 않고 소스 패턴을 검사한다. 다만 이 저장소의 `picture-props-undo.test.ts` 와 같은 계열의 회귀 방지 방식이며, 이번 작은 라우팅 변경에는 수용 가능한 수준으로 판단한다.
- 비차단 보완 후보: `getInputHandler()` 가 null 인 fallback 경로에서는 기존 코드와 달리 `result.ok` 확인 전 `document-changed` 를 emit 한다. 일반 사용자 경로는 `InputHandler` 가 존재하므로 영향이 제한적이지만, 특수 주입/테스트 환경의 dirty 상태를 더 정확히 유지하려면 fallback emit 을 `result.ok` 안으로 되돌리는 편이 더 깔끔하다.

## 6. 로컬 검증

브랜치: `pr2039-merge-test`

- `git merge upstream/devel --no-commit --no-ff`
  - 결과: `Already up to date`
  - `upstream/devel` 이 PR head 의 ancestor 임을 `git merge-base --is-ancestor upstream/devel HEAD` 로 확인.
- `npm test` in `rhwp-studio`
  - 결과: 177 passed, 0 failed.
- `npx tsc --noEmit` in `rhwp-studio`
  - 결과: 통과.
- `npm run build` in `rhwp-studio`
  - 결과: 통과.
  - Vite chunk size warning 과 `canvaskit-wasm` 의 `fs`/`path` browser externalize warning 은 기존 빌드 경고 성격이며 이번 변경의 blocker 로 보지 않는다.
- `git diff --check upstream/devel...HEAD`
  - 결과: 통과.

Cargo/WASM 별도 검증:

- Rust/WASM 소스 및 renderer 출력 경로 변경이 없으므로 `cargo test --profile release-test --tests`, `wasm-pack build` 는 이번 PR 의 필수 검증으로 보지 않는다.

## 7. GitHub Actions

최신 head `4ff9aaf74a6fa5f506ae0264a9a37210b1228806` 기준 최종 확인:

- `CI preflight`: success.
- `CodeQL preflight`: success.
- `Render Diff preflight`: success.
- `WASM Build`: skipped.
- `Analyze (javascript-typescript)`: success.
- `Analyze (python)`: success.
- `Analyze (rust)`: success.
- `Canvas visual diff`: success.
- `Native Skia tests`: success.
- `Build default-feature tests`: success.
- `Build & Test`: success.
- `CodeQL`: success.

PR 상태: `MERGEABLE` / `CLEAN`.

## 8. 최종 권고

최종 판단: **merge 완료**.

처리 결과:

- 작업지시자 옵션 2 지시에 따라 원 코드 PR 을 먼저 merge 했다.
- PR #2039 는 `7649e2180c6f9b823327081e48d897ad50806301` 로 `devel` 에 반영됐다.
- PR 본문이 closing keyword 를 사용하지 않아 #2037 은 merge 직후 open 상태다.
- review 문서는 별도 docs-only 후속 PR 로 보존한 뒤 #2037 close/comment 및 PR 감사 코멘트를 처리한다.
