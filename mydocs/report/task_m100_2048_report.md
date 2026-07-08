# 최종 결과보고서 — task_m100_2048

- **이슈**: #2048 [vscode] macOS 웹뷰 WASM 로드 실패 — 메인 스레드 동기 컴파일(initSync) 차단
- **브랜치**: `local/task2048` (분기 기준: `upstream/devel`)
- **마일스톤**: M100 (v1.0.0)

## 1. 문제

macOS VSCode 확장에서 HWP 문서를 열면 다음 오류로 렌더링 중단:

> WASM 로드 실패: WebAssembly.Compile is disallowed on the main thread, if the buffer size is larger than 4KB.

Windows/Linux 미재현, macOS 전용.

## 2. 원인

`rhwp-vscode/src/webview/viewer.ts`가 wasm-bindgen 동기 초기화 `initSync({ module: buf })`를 사용. 내부에서 `new WebAssembly.Module(bytes)`를 **메인 스레드에서 동기 실행**한다. `rhwp_bg.wasm`(약 5.9MB)이 4KB를 초과하여 메인 스레드 동기 WASM 컴파일 금지 규칙에 위반. macOS 웹뷰는 규칙을 엄격 강제하여 차단, Windows/Linux(Electron/V8)는 느슨하여 통과했다.

## 3. 해결

동일 wasm-bindgen 패키지의 async 기본 export(`__wbg_init`)로 교체. 이 함수는 내부 `__wbg_load`에서 `WebAssembly.instantiateStreaming`/`instantiate`(async)를 사용하여 메인 스레드 동기 컴파일을 회피한다.

| 구분 | 변경 전 | 변경 후 |
|------|--------|--------|
| import | `import { initSync, HwpDocument }` | `import init, { HwpDocument }` |
| 초기화 | `.then((buf) => { initSync({ module: buf }); ... })` | `.then(async (buf) => { await init({ module_or_path: buf }); ... })` |

## 4. 변경 파일

- `rhwp-vscode/src/webview/viewer.ts` — import 1줄 + 초기화 콜백 async 전환 + 설명 주석
- Rust 코드 / WASM 바이너리(`pkg/`) 변경 없음

## 5. 검증

- ✅ webpack production 빌드 성공, TypeScript 타입 체크 통과
- ✅ 산출 번들(`dist/webview/viewer.js`)이 async `WebAssembly.instantiate(Streaming)` 경로 사용 확인 (동기 `Module()` 실행 경로 제거)
- ⏳ macOS VSCode 실제 실행 시각 판정 — 작업지시자 확인 필요

### 작업지시자 확인 방법
1. `rhwp-vscode`에서 `npm run compile` (또는 `npm run package`로 vsix 생성)
2. macOS VSCode에 확장 적용 후 HWP 문서 열기
3. 오류 메시지 소멸 + 문서 렌더링 확인

## 6. 리스크 / 회귀

- 낮음. async init은 Windows/Linux 포함 전 플랫폼에서 표준 동작 → 기존 플랫폼 회귀 위험 없음.
- 초기화 순서(`init` 완료 후 `wasmReady=true`) 및 에러 전파(`.catch`) 기존과 동일 유지.

## 7. 상태

구현·빌드 검증 완료. macOS 실기기 시각 판정 후 이슈 클로즈(작업지시자 승인 필요).
