# 구현 계획서: Task M100 #126

**이슈**: [#126 rhwp-studio TypeScript 빌드 에러 수정 — tsc strict 모드 위반 3종](https://github.com/edwardkim/rhwp/issues/126)  
**작성일**: 2026-04-13

---

## 단계 구성

### Stage 1: TS2551 — `create_empty` 오탈자 수정

**파일**: `rhwp-studio/src/hwpctl/index.ts:351`

```typescript
// 수정 전
wasmDoc = HwpDocument.create_empty();

// 수정 후
wasmDoc = HwpDocument.createEmpty();
```

단순 오탈자 1건 수정. 빌드 확인.

---

### Stage 2: TS2304 — `@types/chrome` 추가

**파일**: `rhwp-studio/package.json`, `rhwp-studio/tsconfig.json`

1. devDependency 추가:
```bash
npm install --save-dev @types/chrome
```

2. `tsconfig.json`의 `lib` 배열은 변경 없음 — `@types/chrome`은 `lib` 추가 없이 자동 인식됨.

빌드 확인 — `chrome` 관련 TS2304 3건 해소.

---

### Stage 3: TS2341 — `WasmBridge.doc` private 접근 수정

**원인 분석**:
- `WasmBridge.doc`는 `private`으로 선언됨
- 두 곳에서 `wasm.doc!.renderPageSvg(i)` / `wasm.doc?.renderPageSvg(...)` 형태로 직접 접근
- `WasmBridge`에 `renderPageSvg()` public 메서드가 없음

**수정 방향**: `WasmBridge`에 `renderPageSvg(pageNum: number): string` public 메서드를 추가하고,
`file.ts`와 `main.ts`의 호출부를 이 메서드를 통해 접근하도록 수정.

**파일 1**: `rhwp-studio/src/core/wasm-bridge.ts`
```typescript
renderPageSvg(pageNum: number): string {
  if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
  return this.doc.renderPageSvg(pageNum);
}
```

**파일 2**: `rhwp-studio/src/command/commands/file.ts:136`
```typescript
// 수정 전
const svg = wasm.doc!.renderPageSvg(i);

// 수정 후
const svg = wasm.renderPageSvg(i);
```

**파일 3**: `rhwp-studio/src/main.ts:532`
```typescript
// 수정 전
reply(wasm.doc?.renderPageSvg(params.page ?? 0));

// 수정 후
reply(wasm.renderPageSvg(params.page ?? 0));
```

> `main.ts`의 경우 `wasm.doc?.` (optional chaining)으로 null 안전하게 접근하던 것을
> `wasm.renderPageSvg()` 내부에서 null 체크로 처리하므로 동등한 안전성 보장.

빌드 확인 — TS2341 2건 해소, `npm run build` 전체 성공.

---

## 완료 기준

`npm run build` 출력에 `error TS` 없이 vite build까지 정상 완료.
