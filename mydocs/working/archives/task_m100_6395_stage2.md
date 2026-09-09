# Task M100 #6395 Stage 2 — PR #6396 최신 devel 충돌 해소

- **Issue**: [#6395](https://github.com/edwardkim/rhwp/issues/6395)
- **PR**: [#6396](https://github.com/edwardkim/rhwp/pull/6396)
- **브랜치**: `codex/issue-6395-page-break-caret-reveal`
- **이전 PR head**: `5481452f81c115007d793770c496ccec554d947d`
- **병합 기준**: `upstream/devel` `d3b40a3d7c3ecb5d0f014ce604b99fda17b2bd9b`
- **수행일**: 2026-08-30 KST
- **상태**: 충돌 해소·로컬 검증 완료, merge commit과 remote push 준비

## 1. 충돌 원인

PR #6396이 갈라진 `2deb3dd61` 뒤 최신 `devel`에 Studio 머리말/바닥글 선택, 셀 선택, WASM alias 전환과
오늘할일 기록이 병합됐다. #6395와 같은 파일의 인접 구간을 바꾼 결과 GitHub가 PR을
`CONFLICTING/DIRTY`로 판정했다.

실제 content conflict는 다음 두 파일뿐이었다.

- `mydocs/orders/20260830.md`
- `rhwp-studio/src/engine/input-handler.ts`

`rhwp-studio/e2e/MANIFEST.md`, `rhwp-studio/package.json`, `rhwp-studio/src/view/canvas-view.ts`는 Git이
자동 병합했다.

## 2. 해소 결과

### 오늘할일

최신 `devel`의 #4121, `test-caption`, HWPX 양식 컨트롤 기록을 모두 보존하고, 문서 끝에 #6395의
`M100 — v1.0.0` 표만 추가했다. `git diff upstream/devel -- mydocs/orders/20260830.md`에서 이 6줄만
PR 고유 diff로 남는 것을 확인했다.

### `InputHandler`

최신 `devel`의 다음 동작을 모두 보존했다.

- `emitHeaderFooterModeChanged`, `CellBlockLetterImeGuard` import
- HF 선택의 `viewport-scroll`·`viewport-resize` 투영
- redo 뒤 HF selection 복원
- `SubmodeSelectionSnapshotCommand`의 edit context·selection 복원

그 위에 #6395의 `CaretLayoutReveal` import·필드·layout 완료 listener와 undo/redo/snapshot 예약만 함께
배치했다. `git diff upstream/devel -- rhwp-studio/src/engine/input-handler.ts`는 #6395의 15줄 추가만 남겼다.

## 3. 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg` | PASS — 최신 devel WASM 생성 |
| `cd rhwp-studio && npx tsc --noEmit` | PASS |
| `cd rhwp-studio && npm test` | PASS — tests 1,313, pass 1,312, skip 1, fail 0 |
| `cd rhwp-studio && npm run build` | PASS — 245 modules, 기존 chunk-size 경고만 발생 |
| `CHROME_PATH=... npm run e2e:page-break-caret` | PASS — 실제 headless Chrome |
| `cd rhwp-studio && npm run e2e:manifest-check` | PASS — tracked 122개 / manifest 122행 |
| `git diff --cached --check` | PASS |

첫 WASM 패키징은 Rust release compile을 완료한 뒤 sandbox가 `wasm-bindgen` 설치를 `Operation not permitted`로
막아 종료됐다. 동일 wrapper를 승인된 호스트 권한으로 다시 실행해 12.80초에 패키징과 `wasm-opt`를
완료했다. 생성된 root `pkg/`와 `target/pr-review`는 로컬 검증 산출물이며 PR에 추가하지 않는다.

Chrome E2E는 병합 전과 같은 값을 확인했다.

- 새 커서: section 0, paragraph 1, offset 0
- 새 쪽: page index 1, page offset `1713.75`
- DOM 캐럿: `1912.2px`, 기대값 `1912.2px`
- 편집 영역: `scrollTop=1214`
- viewport 안 캐럿: `698.2..718.15px / 738px`

## 4. 판정과 남은 조건

충돌 해소는 최신 `devel` 기능을 버리지 않고 #6395의 PR 고유 diff를 그대로 보존했다. Rust source 변경은
base에서 온 것이며 PR 고유 diff에는 없다. merge commit push 뒤 최신 PR head의 Full/분류된 CI와
`MERGEABLE/CLEAN`을 다시 확인해야 한다. 병합은 작업지시자의 별도 승인 조건이다.
