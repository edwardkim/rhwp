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

function hwpSaveFileName(fileName: string): string {
  const trimmed = fileName.trim() || 'document.hwp';
  if (/\.(hwp|hwpx)$/i.test(trimmed)) {
    return trimmed.replace(/\.(hwp|hwpx)$/i, '.hwp');
  }
  return `${trimmed}.hwp`;
}

function hwpSaveBaseName(fileName: string): string {
  return hwpSaveFileName(fileName).replace(/\.hwp$/i, '');
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

export type SaveCurrentDocumentResult = 'saved' | 'cancelled' | 'failed' | 'unsupported';

export async function saveCurrentDocument(services: CommandServices): Promise<SaveCurrentDocumentResult> {
  try {
    const saveName = services.wasm.fileName;
    const sourceFormat = services.wasm.getSourceFormat();
    const isHwpx = sourceFormat === 'hwpx';
    if (isHwpx) {
      alert('HWPX 형식은 현재 베타 단계라 직접 저장이 비활성화되어 있습니다.');
      return 'unsupported';
    }

    const bytes = services.wasm.exportHwp();
    const blob = new Blob([bytes as unknown as BlobPart], { type: 'application/x-hwp' });
    console.log(`[file:save] format=${sourceFormat}, isHwpx=${isHwpx}, ${bytes.length} bytes`);

    try {
      const saveResult = await saveDocumentToFileSystem({
        blob,
        suggestedName: saveName,
        currentHandle: services.wasm.currentFileHandle,
        windowLike: window as FileSystemWindowLike,
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
      const baseName = saveName.replace(/\.hwp$/i, '');
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
    canSave: ctx.sourceFormat !== 'hwpx',
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

        const handle = await pickOpenFileHandle(window as FileSystemWindowLike);
        if (!handle) {
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
    id: 'file:save-as',
    label: '다른 이름으로 저장',
    shortcutLabel: 'Ctrl+Shift+S',
    canExecute: (ctx) => ctx.hasDocument,
    async execute(services) {
      try {
        const sourceFormat = services.wasm.getSourceFormat();
        const isHwpx = sourceFormat === 'hwpx';
        const saveName = hwpSaveFileName(services.wasm.fileName);
        const bytes = services.wasm.exportHwp();
        const blob = new Blob([bytes as unknown as BlobPart], { type: 'application/x-hwp' });
        console.log(`[file:save-as] format=${sourceFormat}, hwpExport=${isHwpx}, ${bytes.length} bytes`);

        try {
          const saveResult = await saveDocumentToFileSystem({
            blob,
            suggestedName: saveName,
            currentHandle: null,
            windowLike: window as FileSystemWindowLike,
            forceSaveAs: true,
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
        const baseName = hwpSaveBaseName(saveName);
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
