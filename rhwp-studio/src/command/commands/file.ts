import type { CommandDef, CommandServices } from '../types';
import { PageSetupDialog } from '@/ui/page-setup-dialog';
import { AboutDialog } from '@/ui/about-dialog';
import { showSaveAs } from '@/ui/save-as-dialog';
import { showUnsavedChangesDialog } from '@/ui/unsaved-changes-dialog';
import {
  appendPrintStyle,
  appendSvgPage,
  createPrintPage,
  type PrintPage,
} from '@/command/print-pages';
import {
  canUseOpenFilePicker,
  pickOpenFileHandle,
  readFileFromHandle,
  saveDocumentToFileSystem,
  type FileSystemFileHandleLike,
  type FileSystemWindowLike,
} from '@/command/file-system-access';

/** [Task #833] 사용자 명시 cancel 에러 검출.
 * - AbortError: showSaveFilePicker / showOpenFilePicker 다이얼로그 취소
 * - NotAllowedError: writeBlobToHandle 권한 거부 (Chrome "변경사항 저장" 프롬프트 취소)
 *
 * 두 케이스 모두 fallback download 우회 — 사용자가 명시적으로 취소했으므로
 * 의도하지 않은 Downloads 폴더 저장 + chrome-extension viewer 자동 연결 차단. */
function isUserCancelError(e: unknown): boolean {
  return e instanceof DOMException
      && (e.name === 'AbortError' || e.name === 'NotAllowedError');
}

/// 출처 포맷에 맞춘 저장 파일명(.hwp / .hwpx). HWPX 직접 저장 활성화용.
function saveFileNameFor(fileName: string, isHwpx: boolean): string {
  const ext = isHwpx ? '.hwpx' : '.hwp';
  const trimmed = fileName.trim() || `document${ext}`;
  if (/\.(hwp|hwpx)$/i.test(trimmed)) {
    return trimmed.replace(/\.(hwp|hwpx)$/i, ext);
  }
  return `${trimmed}${ext}`;
}

function saveBaseNameFor(fileName: string, isHwpx: boolean): string {
  return saveFileNameFor(fileName, isHwpx).replace(/\.(hwp|hwpx)$/i, '');
}

function hwpSaveCurrentHandle(
  sourceFormat: string,
  handle: FileSystemFileHandleLike | null,
): FileSystemFileHandleLike | null {
  if (sourceFormat === 'hwpx' && handle && !handle.name.toLowerCase().endsWith('.hwp')) {
    return null;
  }
  return handle;
}

function flushDeferredPaginationBeforeExplicitOutput(
  services: CommandServices,
  reason: string,
): void {
  services.getInputHandler()?.flushDeferredPaginationIfNeeded(reason);
}

/**
 * 출력 포맷을 명시 받아 "다른 이름으로 저장"한다 (#1613).
 *
 * 출처(getSourceFormat)가 아닌 호출자가 지정한 isHwpx 로 export(`exportHwpx`/`exportHwp`)·
 * 파일명 확장자·MIME 를 결정한다. WASM 은 출처 무관 양방향 export 를 지원하므로, HWP 문서를
 * HWPX 로, HWPX 문서를 HWP 로 저장할 수 있다. FS Access picker → 폴백 download 흐름은
 * file:save-as 와 동일.
 */
async function saveAsFormat(services: CommandServices, isHwpx: boolean): Promise<void> {
  try {
    flushDeferredPaginationBeforeExplicitOutput(services, 'save-as');
    const saveName = saveFileNameFor(services.wasm.fileName, isHwpx);
    const bytes = isHwpx ? services.wasm.exportHwpx() : services.wasm.exportHwp();
    const blob = new Blob([bytes as unknown as BlobPart], {
      type: isHwpx ? 'application/hwp+zip' : 'application/x-hwp',
    });
    console.log(`[file:save-as] format=${isHwpx ? 'hwpx' : 'hwp'}, ${bytes.length} bytes`);

    try {
      const saveResult = await saveDocumentToFileSystem({
        blob,
        suggestedName: saveName,
        currentHandle: null,
        windowLike: window as FileSystemWindowLike,
        forceSaveAs: true,
        saveAsHwpx: isHwpx,
      });
      if (saveResult.method !== 'fallback') {
        services.wasm.currentFileHandle = saveResult.handle;
        services.wasm.fileName = saveResult.fileName;
        services.documentState.markClean('save-as');
        console.log(`[file:save-as] ${saveResult.fileName} (${(bytes.length / 1024).toFixed(1)}KB)`);
        return;
      }
    } catch (e) {
      if (isUserCancelError(e)) return;
      console.warn('[file:save-as] File System Access API 실패, 폴백:', e);
    }

    // 폴백: 파일명 입력 → blob download
    const baseName = saveBaseNameFor(saveName, isHwpx);
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

    services.documentState.markClean('save-as');
    console.log(`[file:save-as] ${downloadName} (${(bytes.length / 1024).toFixed(1)}KB)`);
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    console.error('[file:save-as] 저장 실패:', msg);
    alert(`파일 저장에 실패했습니다:\n${msg}`);
  }
}

export type SaveCurrentDocumentResult = 'saved' | 'cancelled' | 'failed' | 'unsupported';

export async function saveCurrentDocument(services: CommandServices): Promise<SaveCurrentDocumentResult> {
  try {
    flushDeferredPaginationBeforeExplicitOutput(services, 'save');
    const saveName = services.wasm.fileName;
    const sourceFormat = services.wasm.getSourceFormat();
    const isHwpx = sourceFormat === 'hwpx';

    // HWPX 출처는 HWPX 로 직접 저장(직렬화 충실도 확보 후 활성화). 그 외는 HWP.
    const bytes = isHwpx ? services.wasm.exportHwpx() : services.wasm.exportHwp();
    const blob = new Blob([bytes as unknown as BlobPart], {
      type: isHwpx ? 'application/hwp+zip' : 'application/x-hwp',
    });
    console.log(`[file:save] format=${sourceFormat}, isHwpx=${isHwpx}, ${bytes.length} bytes`);

    try {
      const saveResult = await saveDocumentToFileSystem({
        blob,
        suggestedName: saveName,
        currentHandle: services.wasm.currentFileHandle,
        windowLike: window as FileSystemWindowLike,
        saveAsHwpx: isHwpx,
      });

      if (saveResult.method !== 'fallback') {
        services.wasm.currentFileHandle = saveResult.handle;
        services.wasm.fileName = saveResult.fileName;
        services.documentState.markClean('save');
        console.log(`[file:save] ${saveResult.fileName} (${(bytes.length / 1024).toFixed(1)}KB)`);
        return 'saved';
      }
    } catch (e) {
      if (isUserCancelError(e)) return 'cancelled';
      console.warn('[file:save] File System Access API 실패, 폴백:', e);
    }

    let downloadName = saveName;
    if (services.wasm.isNewDocument) {
      const baseName = saveBaseNameFor(saveName, isHwpx);
      const result = await showSaveAs(baseName);
      if (!result) return 'cancelled';
      downloadName = result;
      services.wasm.fileName = downloadName;
    }

    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = downloadName;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);

    services.documentState.markClean('save');
    console.log(`[file:save] ${downloadName} (${(bytes.length / 1024).toFixed(1)}KB)`);
    return 'saved';
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    console.error('[file:save] 저장 실패:', msg);
    alert(`파일 저장에 실패했습니다:\n${msg}`);
    return 'failed';
  }
}

export async function confirmSaveBeforeReplacingDocument(
  services: CommandServices,
): Promise<boolean> {
  const ctx = services.getContext();
  if (!ctx.hasDocument || !ctx.isDirty) return true;

  const choice = await showUnsavedChangesDialog({
    fileName: services.wasm.fileName,
    canSave: true, // HWPX 직접 저장 활성화로 모든 출처 저장 가능
  });

  if (choice === 'cancel') return false;
  if (choice === 'discard') return true;

  const result = await saveCurrentDocument(services);
  return result === 'saved';
}

function createPrintButton(doc: Document, id: string, label: string, background?: string): HTMLButtonElement {
  const button = doc.createElement('button');
  button.id = id;
  button.type = 'button';
  button.textContent = label;
  if (background) button.style.background = background;
  return button;
}

function setupPrintDocument(
  printWin: Window,
  fileName: string,
  pageCount: number,
  printPages: PrintPage[],
): void {
  const doc = printWin.document;
  doc.documentElement.lang = 'ko';
  doc.title = `${fileName} — 인쇄`;

  doc.head.replaceChildren();
  const meta = doc.createElement('meta');
  meta.setAttribute('charset', 'UTF-8');
  doc.head.appendChild(meta);
  appendPrintStyle(doc, printPages);

  const printBar = doc.createElement('div');
  printBar.className = 'print-bar';
  const printButton = createPrintButton(doc, 'print-btn', '인쇄');
  const closeButton = createPrintButton(doc, 'close-btn', '닫기', '#475569');
  const title = doc.createElement('span');
  title.textContent = `${fileName} — ${pageCount}페이지`;
  printBar.append(printButton, closeButton, title);

  doc.body.replaceChildren(printBar);
  for (const printPage of printPages) {
    appendSvgPage(doc, doc.body, printPage);
  }

  printButton.addEventListener('click', () => {
    printWin.print();
  });
  closeButton.addEventListener('click', () => {
    printWin.close();
  });
}

export const fileCommands: CommandDef[] = [
  {
    id: 'file:new-doc',
    label: '새로 만들기',
    icon: 'icon-new-doc',
    shortcutLabel: 'Alt+N',
    canExecute: () => true,
    execute(services) {
      services.eventBus.emit('create-new-document');
    },
  },
  {
    id: 'file:open',
    label: '열기',
    async execute(services) {
      try {
        const canReplace = await confirmSaveBeforeReplacingDocument(services);
        if (!canReplace) return;

        const windowLike = window as FileSystemWindowLike;
        const nativeOpenPickerAvailable = canUseOpenFilePicker(windowLike);
        const handle = await pickOpenFileHandle(windowLike);
        if (!handle) {
          // File System Access API picker가 있었다면 null은 사용자 취소(예: Esc)다.
          // 이때 숨김 input fallback을 다시 열면 파일 선택창이 곧바로 재오픈된다.
          if (nativeOpenPickerAvailable) return;
          const fileInput = document.getElementById('file-input') as HTMLInputElement | null;
          if (fileInput) {
            fileInput.dataset.skipUnsavedGuard = 'true';
            fileInput.click();
          }
          return;
        }

        const { bytes, name } = await readFileFromHandle(handle);
        services.eventBus.emit('open-document-bytes', {
          bytes,
          fileName: name,
          fileHandle: handle,
          skipUnsavedGuard: true,
        });
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        console.error('[file:open] 열기 실패:', msg);
        alert(`파일 열기에 실패했습니다:\n${msg}`);
      }
    },
  },
  {
    id: 'file:save',
    label: '저장',
    icon: 'icon-save',
    shortcutLabel: 'Ctrl+S',
    canExecute: (ctx) => ctx.hasDocument,
    async execute(services) {
      await saveCurrentDocument(services);
    },
  },
  {
    // [Task #833] 다른 이름으로 저장 — currentFileHandle 무시 + 항상 picker.
    // 출처 포맷 유지(HWPX→HWPX, HWP→HWP).
    id: 'file:save-as',
    label: '다른 이름으로 저장',
    shortcutLabel: 'Ctrl+Shift+S',
    canExecute: (ctx) => ctx.hasDocument,
    async execute(services) {
      await saveAsFormat(services, services.wasm.getSourceFormat() === 'hwpx');
    },
  },
  {
    // [#1613] HWP 형식으로 저장 — 출처 무관 HWP 출력.
    id: 'file:save-as-hwp',
    label: 'HWP 형식으로 저장',
    canExecute: (ctx) => ctx.hasDocument,
    async execute(services) {
      await saveAsFormat(services, false);
    },
  },
  {
    // [#1613] HWPX 형식으로 저장 — 출처 무관 HWPX 출력.
    id: 'file:save-as-hwpx',
    label: 'HWPX 형식으로 저장',
    canExecute: (ctx) => ctx.hasDocument,
    async execute(services) {
      await saveAsFormat(services, true);
    },
  },
  {
    id: 'file:page-setup',
    label: '편집 용지',
    icon: 'icon-page-setup',
    shortcutLabel: 'F7',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const dialog = new PageSetupDialog(services.wasm, services.eventBus, 0);
      dialog.show();
    },
  },
  {
    id: 'file:print',
    label: '인쇄',
    icon: 'icon-print',
    shortcutLabel: 'Ctrl+P',
    canExecute: (ctx) => ctx.hasDocument,
    async execute(services) {
      flushDeferredPaginationBeforeExplicitOutput(services, 'print');
      const wasm = services.wasm;
      const pageCount = wasm.pageCount;
      if (pageCount === 0) return;

      // 진행률 표시
      const statusEl = document.getElementById('sb-message');
      const origStatus = statusEl?.textContent || '';

      try {
        // SVG 페이지 생성
        const printPages: PrintPage[] = [];
        for (let i = 0; i < pageCount; i++) {
          if (statusEl) statusEl.textContent = `인쇄 준비 중... (${i + 1}/${pageCount})`;
          const svg = wasm.renderPageSvg(i);
          const pageInfo = wasm.getPageInfo(i);
          printPages.push(createPrintPage(svg, pageInfo, i));
          // UI 갱신을 위한 양보
          if (i % 5 === 0) await new Promise(r => setTimeout(r, 0));
        }

        // 인쇄 전용 창 생성
        const printWin = window.open('', '_blank');
        if (!printWin) {
          alert('팝업이 차단되었습니다. 팝업 허용 후 다시 시도해주세요.');
          return;
        }

        setupPrintDocument(printWin, wasm.fileName, pageCount, printPages);

        if (statusEl) statusEl.textContent = origStatus;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        console.error('[file:print]', msg);
        if (statusEl) statusEl.textContent = `인쇄 실패: ${msg}`;
      }
    },
  },
  {
    id: 'file:about',
    label: '제품 정보',
    icon: 'icon-help',
    execute() {
      new AboutDialog().show();
    },
  },
];
