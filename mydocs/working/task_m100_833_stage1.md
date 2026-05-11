# Task #833 Stage 1 (GREEN) 보고서

**브랜치**: `local/task833`
**선행**: 수행계획서 + 구현계획서 승인 완료
**목표**: (A) Save As 기능 + (B) 권한 cancel fallback 정정 통합 구현

## 수정 내용

### 1-1. `rhwp-studio/src/command/file-system-access.ts`

`SaveDocumentOptions.forceSaveAs?: boolean` 옵션 추가 + `saveDocumentToFileSystem` 분기 정정.

```diff
 export interface SaveDocumentOptions {
   blob: Blob;
   suggestedName: string;
   currentHandle: FileSystemFileHandleLike | null;
   windowLike: FileSystemWindowLike;
+  /** [Task #833] true 시 currentHandle 무시 + 항상 showSaveFilePicker 호출 (다른 이름으로 저장). */
+  forceSaveAs?: boolean;
 }

 export async function saveDocumentToFileSystem(options: SaveDocumentOptions): Promise<SaveDocumentResult> {
-  const { blob, suggestedName, currentHandle, windowLike } = options;
-
-  if (currentHandle) {
+  const { blob, suggestedName, currentHandle, windowLike, forceSaveAs } = options;
+
+  // [Task #833] forceSaveAs 시 currentHandle 우회 → 항상 picker (다른 이름으로 저장).
+  if (currentHandle && !forceSaveAs) {
     await writeBlobToHandle(currentHandle, blob);
```

### 1-2. `rhwp-studio/src/command/commands/file.ts`

3개 변경:

(a) `isUserCancelError` 모듈-레벨 helper 신규:
```typescript
function isUserCancelError(e: unknown): boolean {
  return e instanceof DOMException
      && (e.name === 'AbortError' || e.name === 'NotAllowedError');
}
```

(b) `file:save` catch 정정 — `AbortError` 단독 → `isUserCancelError`:
```diff
-          if (e instanceof DOMException && e.name === 'AbortError') return;
+          if (isUserCancelError(e)) return;
```

(c) `file:save-as` command 신규 — `forceSaveAs: true` + 동일 catch 패턴 적용. `Ctrl+Shift+S` shortcut label.

### 1-3. `rhwp-studio/src/command/shortcut-map.ts`

`Ctrl+Shift+S` (영문 `s` + 한글 IME `ㄴ`) 등록:
```diff
   [{ key: 's', ctrl: true }, 'file:save'],
+  // [Task #833] Ctrl+Shift+S → 다른 이름으로 저장 (한글 IME 'ㄴ' 도 함께).
+  [{ key: 's', ctrl: true, shift: true }, 'file:save-as'],
+  [{ key: 'ㄴ', ctrl: true, shift: true }, 'file:save-as'],
```

### 1-4. `rhwp-studio/index.html`

파일 메뉴 "저장" 항목 아래 "다른 이름으로 저장(A)..." 추가:
```diff
       <div class="md-item disabled" data-cmd="file:save"><span class="md-icon icon-save"></span><span class="md-label">저장</span><span class="md-shortcut">Ctrl+S</span></div>
+      <div class="md-item disabled" data-cmd="file:save-as"><span class="md-icon"></span><span class="md-label">다른 이름으로 저장(A)...</span><span class="md-shortcut">Ctrl+Shift+S</span></div>
       <div class="md-sep"></div>
```

> `disabled` class 는 초기 상태 (문서 미로드 시). 메뉴 빌드 시 `canExecute` 평가로 자동 enable/disable.

## 검증

```
$ cd rhwp-studio && npx tsc --noEmit
(clean)
```

## 다음 단계

Stage 2 (회귀 검증) — 전체 cargo test + clippy + tsc 재확인.
