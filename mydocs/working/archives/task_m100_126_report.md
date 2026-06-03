# 최종 결과 보고서: Task M100 #126

**이슈**: [#126 rhwp-studio TypeScript 빌드 에러 수정 — tsc strict 모드 위반 3종](https://github.com/edwardkim/rhwp/issues/126)  
**완료일**: 2026-04-13

---

## 수정 내용

### Stage 1: TS2551 — `create_empty` 오탈자 수정
- `rhwp-studio/src/hwpctl/index.ts:351`
- `HwpDocument.create_empty()` → `HwpDocument.createEmpty()`

### Stage 2: TS2304 — `@types/chrome` 추가
- `rhwp-studio/package.json` devDependencies에 `@types/chrome@0.1.40` 추가
- `src/main.ts`의 `chrome.*` API 전역 타입 3건 해소

### Stage 3: TS2341 — `WasmBridge.renderPageSvg()` public 메서드 추가
- `rhwp-studio/src/core/wasm-bridge.ts`: `renderPageSvg(pageNum)` public 메서드 추가
- `rhwp-studio/src/command/commands/file.ts:136`: `wasm.doc!.renderPageSvg(i)` → `wasm.renderPageSvg(i)`
- `rhwp-studio/src/main.ts:532`: `wasm.doc?.renderPageSvg(...)` → `wasm.renderPageSvg(...)`

## 결과

```
> rhwp-studio@0.7.0 build
> tsc && vite build

✓ 79 modules transformed.
✓ built in 1.44s
```

`npm run build` (tsc + vite build) 에러 없이 정상 완료.  
bypass(`@ts-ignore`, `any` 캐스팅, strict 완화) 없이 코드 수정으로 해결.
