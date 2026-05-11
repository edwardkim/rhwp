# Task #833 구현 계획서

**선행 문서**: [task_m100_833.md](task_m100_833.md) (수행계획서, 승인 완료)
**브랜치**: `local/task833`

## 개요

3단계 — (A) Save As + (B) 권한 cancel fallback fix 통합 구현 → 회귀 → 시각+PR.

## 단계별 상세

### 단계 1 — GREEN (Save As + cancel fallback fix)

**목적**: 두 결함 동시 정정 (TS 만 변경, RED 단계 생략 — 단일 GREEN).

**작업 순서**:

#### 1-1. `file-system-access.ts` — `forceSaveAs` 옵션
```typescript
export interface SaveDocumentOptions {
  blob: Blob;
  suggestedName: string;
  currentHandle: FileSystemFileHandleLike | null;
  windowLike: FileSystemWindowLike;
  /** [Task #833] true 시 currentHandle 무시하고 항상 showSaveFilePicker 호출. */
  forceSaveAs?: boolean;
}

export async function saveDocumentToFileSystem(options: SaveDocumentOptions): Promise<SaveDocumentResult> {
  const { blob, suggestedName, currentHandle, windowLike, forceSaveAs } = options;

  // [Task #833] forceSaveAs 시 currentHandle 우회 → 항상 picker.
  if (currentHandle && !forceSaveAs) {
    await writeBlobToHandle(currentHandle, blob);
    return { method: 'current-handle', handle: currentHandle, fileName: currentHandle.name };
  }
  ...
}
```

#### 1-2. `commands/file.ts` — `isUserCancelError` helper + `file:save-as` command + 양쪽 catch 정정

**helper 함수 (모듈 레벨)**:
```typescript
/** [Task #833] 사용자 명시 cancel 에러 검출.
 * AbortError: showSaveFilePicker / showOpenFilePicker 다이얼로그 취소
 * NotAllowedError: writeBlobToHandle 권한 거부 (Chrome "변경사항 저장" 프롬프트 취소) */
function isUserCancelError(e: unknown): boolean {
  return e instanceof DOMException
      && (e.name === 'AbortError' || e.name === 'NotAllowedError');
}
```

**기존 `file:save` catch 정정**:
```typescript
} catch (e) {
  // [Task #833] 사용자 명시 cancel 시 fallback download 우회.
  if (isUserCancelError(e)) return;
  console.warn('[file:save] File System Access API 실패, 폴백:', e);
}
```

**신규 `file:save-as` command**:
```typescript
{
  id: 'file:save-as',
  label: '다른 이름으로 저장',
  shortcutLabel: 'Ctrl+Shift+S',
  // HWPX 출처는 #196 정합 비활성 (file:save 와 동일).
  canExecute: (ctx) => ctx.hasDocument && ctx.sourceFormat !== 'hwpx',
  async execute(services) {
    try {
      const saveName = services.wasm.fileName;
      const sourceFormat = services.wasm.getSourceFormat();
      const isHwpx = sourceFormat === 'hwpx';
      const bytes = isHwpx ? services.wasm.exportHwpx() : services.wasm.exportHwp();
      const mimeType = isHwpx ? 'application/hwp+zip' : 'application/x-hwp';
      const blob = new Blob([bytes as unknown as BlobPart], { type: mimeType });
      console.log(`[file:save-as] format=${sourceFormat}, ${bytes.length} bytes`);

      try {
        const saveResult = await saveDocumentToFileSystem({
          blob,
          suggestedName: saveName,
          currentHandle: services.wasm.currentFileHandle,
          windowLike: window as FileSystemWindowLike,
          forceSaveAs: true,  // ← 핵심: 항상 picker
        });
        if (saveResult.method !== 'fallback') {
          services.wasm.currentFileHandle = saveResult.handle;
          services.wasm.fileName = saveResult.fileName;
          console.log(`[file:save-as] ${saveResult.fileName} (${(bytes.length / 1024).toFixed(1)}KB)`);
          return;
        }
      } catch (e) {
        if (isUserCancelError(e)) return;
        console.warn('[file:save-as] File System Access API 실패, 폴백:', e);
      }

      // 폴백: 파일명 입력 → blob download
      const baseName = saveName.replace(/\.hwp$/i, '');
      const result = await showSaveAs(baseName);
      if (!result) return;
      const downloadName = result;
      services.wasm.fileName = downloadName;

      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = downloadName;
      a.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);

      console.log(`[file:save-as] ${downloadName} (${(bytes.length / 1024).toFixed(1)}KB)`);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error('[file:save-as] 저장 실패:', msg);
      alert(`파일 저장에 실패했습니다:\n${msg}`);
    }
  },
},
```

#### 1-3. `shortcut-map.ts` — Ctrl+Shift+S 등록
```typescript
[{ key: 's', ctrl: true }, 'file:save'],
// [Task #833] Ctrl+Shift+S → 다른 이름으로 저장 (영문 + 한글 IME 'ㄴ').
[{ key: 'S', ctrl: true, shift: true }, 'file:save-as'],
[{ key: 'ㄴ', ctrl: true, shift: true }, 'file:save-as'],
```

> shortcut-map 의 키 비교 규칙 확인 필요 — 대문자 'S' 인지 'shift: true + s' 인지 기존 패턴 따라 결정.

#### 1-4. `index.html` — 메뉴 항목
```html
<div class="md-item" data-cmd="file:save"><span class="md-icon icon-save"></span><span class="md-label">저장</span><span class="md-shortcut">Ctrl+S</span></div>
<div class="md-item" data-cmd="file:save-as"><span class="md-icon"></span><span class="md-label">다른 이름으로 저장(A)...</span><span class="md-shortcut">Ctrl+Shift+S</span></div>
```

**산출물**: 4 파일 수정, `_stage1.md`

**커밋**: `Task #833 Stage 1 (GREEN): file:save-as command + 권한 cancel fallback 우회`

---

### 단계 2 — 회귀 검증

**목적**: TS 변경의 회귀 영향 확인 (Rust/WASM 무영향 예상).

**작업**:
1. `npx tsc --noEmit` (rhwp-studio) clean
2. `cargo test --release` 영향 부재 확인 (Rust 변경 없음)
3. `cargo clippy --release -- -D warnings` clean
4. WASM 재빌드 — **불필요** (TS 만 변경)
5. `mydocs/working/task_m100_833_stage2.md` 작성
6. 커밋: `Task #833 Stage 2 (회귀): tsc + cargo test 회귀 부재`

**산출물**: `_stage2.md`

---

### 단계 3 — 시각 검증 + 최종 보고서

**목적**: 작업지시자 시각 판정 + PR.

**작업**:
1. Vite hot-reload 즉시 적용 (또는 브라우저 새로고침)
2. 작업지시자 시각 검증 요청:

**A. Save As**
- 임의 HWP 문서 로드 → 파일 메뉴 → "다른 이름으로 저장(A)..." 항목 표시
- 클릭 → showSaveFilePicker 다이얼로그 표시
- 새 파일명 입력 + 저장 → 새 파일 생성 + currentFileHandle 갱신
- Ctrl+Shift+S 단축키 동작 (영문 + 한글 IME)
- Ctrl+S 회귀 부재

**B. 권한 cancel 정정**
- Ctrl+S → "변경사항 저장" 프롬프트 → "취소" → download 미발현
- showSaveFilePicker → 취소 → download 미발현
- 정상 저장 path 회귀 부재

3. 시각 통과 후:
   - 최종 보고서 `mydocs/report/task_m100_833_report.md`
   - 오늘할일 `mydocs/orders/20260511.md` 갱신
4. 커밋: `Task #833 Stage 3 (최종): 시각 판정 통과 + 보고서 + closes #833`
5. PR 생성 — `closes #833`

**산출물**: `_report.md`, orders 갱신, PR

---

## 단계별 commit 계획 요약

| 단계 | commit 메시지 | 변경 파일 |
|---|---|---|
| 1 | `Task #833 Stage 1 (GREEN): file:save-as command + 권한 cancel fallback 우회` | file-system-access.ts, file.ts, shortcut-map.ts, index.html, `_stage1.md` |
| 2 | `Task #833 Stage 2 (회귀): tsc + cargo test 회귀 부재` | `_stage2.md` |
| 3 | `Task #833 Stage 3 (최종): 시각 판정 통과 + 보고서 + closes #833` | `_report.md`, orders, body 의 `closes #833` |

## 위험 / 가정

- **가정**: shortcut-map 의 Ctrl+Shift+S 표현 — 기존 코드 패턴 (대문자 / shift 옵션) 따라 결정 (Stage 1 에서 확인)
- **가정**: HWPX 출처 비활성 정합 (`sourceFormat !== 'hwpx'`) `file:save` 와 동일 적용
- **위험**: `NotAllowedError` 가 다른 의도하지 않은 cancel 케이스를 잘못 swallow 할 가능성 — Stage 3 에서 정상 저장 path 회귀 검증
- **위험**: `forceSaveAs` 시 사용자가 같은 파일을 다시 선택하면 자동 overwrite — 이는 Chrome 의 기본 동작 (덮어쓰기 확인 프롬프트 표시) 으로 처리됨, 본 task 추가 처리 불필요
