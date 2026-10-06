# Stage 1 완료보고서 — task_m100_2048

- **이슈**: #2048  **브랜치**: `local/task2048`

## 작업 내용

`rhwp-vscode/src/webview/viewer.ts` 동기 초기화 → async 초기화 교체.

### 변경 1: import (line 1)
```diff
-import { initSync, HwpDocument } from "@rhwp-wasm/rhwp.js";
+import init, { HwpDocument } from "@rhwp-wasm/rhwp.js";
```

### 변경 2: 초기화 콜백 (line 43~44)
```diff
-  .then((buf) => {
-    initSync({ module: buf });
+  .then(async (buf) => {
+    // (주석: macOS 웹뷰 메인 스레드 동기 컴파일 금지 회피)
+    await init({ module_or_path: buf });
     wasmReady = true;
```

에러 전파(`.catch`), `wasmReady`, `postMessage({ type: "ready" })` 등 나머지 로직 불변.

## 결과
- 소스 변경 2곳 + 설명 주석. 무관 diff 없음.
- `package-lock.json` 부수 변경(버전 필드 동기화)은 되돌림.

## 상태: ✅ 완료
