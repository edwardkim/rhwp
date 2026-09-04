# 구현계획서 — task_m100_2048

- **이슈**: #2048 [vscode] macOS 웹뷰 WASM 로드 실패 (initSync 동기 컴파일)
- **브랜치**: `local/task2048`
- 수행계획서: `task_m100_2048.md`

## 변경 개요

`rhwp-vscode/src/webview/viewer.ts`의 동기 초기화(`initSync`)를 async 기본 export(`init`)로 교체한다. 초기화가 이미 `fetch().then((buf) => {...})` 콜백 안에 있으므로, 해당 콜백을 `async`로 바꾸고 `await init(...)`를 호출한다. 에러는 기존 `.catch`로 그대로 전파된다.

## 단계별 구현 (3단계)

### Stage 1 — 소스 수정 (viewer.ts)

1. import 교체
   - `import { initSync, HwpDocument } from "@rhwp-wasm/rhwp.js";`
   - → `import init, { HwpDocument } from "@rhwp-wasm/rhwp.js";`
2. 초기화 콜백 async 전환 + async init 호출
   - `.then((buf) => {` → `.then(async (buf) => {`
   - `initSync({ module: buf });` → `await init({ module_or_path: buf });`

나머지(`wasmReady = true`, `postMessage({ type: "ready" })`, `.catch`)는 그대로 유지.

산출: 소스 커밋 + `_stage1.md` 완료보고서.

### Stage 2 — 빌드 검증

1. `rhwp-vscode`에서 webpack 빌드 (`npm run` 스크립트 확인 후 실행)
2. TypeScript 타입 체크 통과 확인
3. `dist/` 산출물 정상 생성 확인

빌드 실패 시 원인 조사 후 수정. 산출: `_stage2.md` 완료보고서.

### Stage 3 — 최종 검증 및 보고

1. 변경 diff 최종 확인 (`viewer.ts` 2줄 한정, 무관 diff 없음)
2. macOS VSCode 실행 확인 방법 정리 (작업지시자 시각 판정용)
3. 최종 결과보고서 `report/task_m100_2048_report.md` 작성
4. `_stage{N}.md`, `_impl.md`, `_report.md` 커밋

## 검증 기준

- webpack 빌드 성공 + 타입 체크 통과
- macOS VSCode에서 오류 메시지 소멸 + 문서 렌더링 (작업지시자 시각 판정)
- Windows/Linux 회귀 없음 (async init은 전 플랫폼 동작)

## 비적용 범위

- Rust 코드 / WASM 바이너리(`pkg/`) 변경 없음
- `cargo fmt --all` 미실행 (기능 브랜치, 무관 diff 금지)
