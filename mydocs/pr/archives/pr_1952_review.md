# PR #1952 리뷰 - Task #1918

**PR**: https://github.com/edwardkim/rhwp/pull/1952
**작성자**: postmelee
**base**: `devel`
**문서 작성일**: 2026-07-05

## 메타

| 항목 | 내용 |
|---|---|
| 제목 | Task #1918: 워터마크 포함 문서 표 편집 지연 개선 |
| 관련 이슈 | #1918 |
| merge commit | `dc9e899d7ac27392d4457edf27ba6b191fa2eab3` |
| 후속 이슈 | #1964 raw IME/iOS 입력 경로 page-local refresh 안전 가드 보강 |
| 변경 규모 | 25 files, +2655/-71 |

## 변경 범위

- Studio text-edit fast path에서 page-local invalidation을 사용한다.
- `PageRenderer`가 overlay summary를 먼저 사용하고, text-edit 시 static overlay/static flow canvas를 재사용한다.
- WASM Canvas2D filtered render layer kind에 `flow-dynamic`, `flow-static`을 추가한다.
- Rust overlay summary API가 `hasBehind`, `hasFront`, `rawSvgCount`, `flowImageCount`, `flowRawSvgCount`를 반환한다.
- 성능 검증 및 stage/report 문서가 추가되었다.

## 렌더 영향 판정

렌더 영향 있음.

- `rhwp-studio/src/view/page-renderer.ts`, `canvas-view.ts`, `src/renderer/web_canvas.rs`, `src/wasm_api.rs`가 변경된다.
- PR 목적이 Canvas 렌더 경로와 입력 후 재렌더 성능 개선이다.
- 사용자 실검증에서 기존 렉 문제가 해소된 것을 확인했다.
- GitHub Actions `Canvas visual diff`, `Native Skia tests`, `Build default-feature tests`, `Build & Test`, CodeQL 계열 체크가 모두 통과했다.

## 코드 리뷰 결과

### 1. raw IME/iOS 입력 경로 안전 가드 보강 후보

`rhwp-studio/src/engine/input-handler.ts`의 `afterTextInputEdit()`는 raw IME/iOS 입력 후 `shouldUsePageLocalRefresh('insertText', beforePos, afterPos)`만 호출한다.
이 경로는 command 기반 입력 경로와 달리 `insertedText`, `beforePageIndex`, `afterPageIndex`를 넘기지 않는다.

PR의 핵심 성능 문제는 사용자 검증에서 해소되었으므로 merge blocker로 보지는 않았다. 다만 text-edit fast path의 장기 안전성을 위해 별도 후속 이슈로 분리했다.

- 후속 이슈: https://github.com/edwardkim/rhwp/issues/1964
- 권장 방향: raw 입력 경로도 command 입력과 같은 page-local 판정 힌트를 넘기거나, raw IME/iOS fallback 경로를 보수적으로 full refresh 처리한다.

관련 위치:

- `rhwp-studio/src/engine/input-handler.ts:2252`
- `rhwp-studio/src/engine/input-handler-text.ts:458`
- `rhwp-studio/src/engine/input-handler-text.ts:516`
- `rhwp-studio/src/engine/input-edit-invalidation.ts:41`

### 2. 문서 trailing whitespace

원 PR diff의 일부 stage/plan 문서에 trailing whitespace가 있었다. 코드 동작 문제는 아니며, 후속 docs-only PR에서 함께 정리한다.

정리 대상:

- `mydocs/plans/task_m100_1918.md`
- `mydocs/plans/task_m100_1918_impl.md`
- `mydocs/working/task_m100_1918_stage1.md`
- `mydocs/working/task_m100_1918_stage7.md`
- `mydocs/working/task_m100_1918_stage8.md`

## 검증 메모

GitHub Actions:

- `CI preflight`: success
- `CodeQL preflight`: success
- `Render Diff preflight`: success
- `Canvas visual diff`: success
- `Native Skia tests`: success
- `Build default-feature tests`: success
- `Build & Test`: success
- CodeQL 분석: success

로컬에서 추가 확인한 항목:

```bash
cargo fmt --all -- --check
env CARGO_INCREMENTAL=0 cargo build --profile release-test --verbose
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests --verbose
env CARGO_INCREMENTAL=0 cargo check --target wasm32-unknown-unknown --lib
env CARGO_INCREMENTAL=0 cargo clippy -- -D warnings
env CARGO_INCREMENTAL=0 cargo test --profile release-test --features native-skia skia --lib --verbose
env CARGO_INCREMENTAL=0 cargo build --release
env CARGO_INCREMENTAL=0 cargo test --release --lib
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
env CARGO_INCREMENTAL=0 cargo test --doc
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
wasm-pack build --target web --out-dir pkg
```

`cargo test` 전체 명령도 추가로 시작했으나, 사용자가 GitHub CI와 실사용 검증 통과를 확인한 뒤 merge 진행을 지시하여 장기 실행 중 중단했다.

## 최종 권고

merge 수용 완료.

PR #1952는 사용자 실검증에서 기존 표 입력 렉 문제가 해소되었고, GitHub Actions도 모두 통과했다. 코드 리뷰에서 확인한 raw IME/iOS fast path 안전 가드 보강 후보는 merge blocker가 아니라 후속 이슈 #1964로 추적한다.
