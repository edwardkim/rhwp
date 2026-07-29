import type { CommandDef, CommandServices } from '../types';
import { PageSetupDialog } from '@/ui/page-setup-dialog';
import { AboutDialog } from '@/ui/about-dialog';
import { showSaveAs } from '@/ui/save-as-dialog';
import { showUnsavedChangesDialog } from '@/ui/unsaved-changes-dialog';
import { showHmlSaveFormatDialog } from '@/ui/hml-save-format-dialog';
import {
  fileNameForFormat,
  markConvertedHmlSaveHandle,
  requiresSaveFormatChoice,
  resolveSaveTarget,
  type SaveFormat,
} from '@/command/save-target';
import { SAVE_FORMAT_DETAILS } from '@/command/save-format';
import { exportDocumentForFormat } from '@/command/save-document-format';
import {
  readHmlSaveContext,
  resolveHmlSaveCapability,
} from '@/core/hml-save-capability';
import {
  appendPrintStyle,
  appendSvgPage,
  createPrintPage,
  pdfPrintTitle,
  printProgressText,
  printReadyText,
  type PrintIntent,
  type PrintPage,
} from '@/command/print-pages';
import {
  createPrintPreviewSurface,
  createPrintSurface,
  PrintPreviewBlockedError,
  waitForPrintSurfaceReady,
  type PrintPreviewSurface,
  type PrintSurface,
} from '@/command/print-surface';
import {
  canUseOpenFilePicker,
  pickOpenFileHandle,
  readFileFromHandle,
  saveDocumentToFileSystem,
  type FileSystemFileHandleLike,
  type SaveDocumentResult,
  type FileSystemWindowLike,
} from '@/command/file-system-access';
import { PdfPrintDialog } from '@/ui/pdf-print-dialog';
import { userSettings } from '@/core/user-settings';
import { showToast } from '@/ui/toast';
import { clearRecentDocs, listRecentDocs, removeRecentDoc } from '@/recent/recent-store';
import { openRecentEntry } from '@/recent/recent-open';

/**
 * 파일 열기 대화상자(File System Access picker, 미지원 시 숨김 input 폴백)를 열어
 * 문서를 로드한다. `file:open` 커맨드와 "최근 문서" 메타-only 항목 재열기가 공유한다.
 */
async function openFileViaPicker(services: CommandServices): Promise<void> {
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
}

/** 최근 문서 핸들의 읽기 권한을 확인/요청한다. 최종 'granted' 여부 반환. */
async function ensureReadPermission(handle: FileSystemFileHandleLike): Promise<boolean> {
  try {
    if (typeof handle.queryPermission === 'function') {
      if ((await handle.queryPermission({ mode: 'read' })) === 'granted') return true;
    }
    if (typeof handle.requestPermission === 'function') {
      return (await handle.requestPermission({ mode: 'read' })) === 'granted';
    }
    // 권한 API 미지원 브라우저 → getFile() 시도로 위임(여기선 통과).
    return true;
  } catch {
    return false;
  }
}

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

function saveBaseNameFor(fileName: string, format: SaveFormat): string {
  return fileNameForFormat(fileName, format).replace(/\.(hwp|hwpx|hml)$/i, '');
}

function flushDeferredPaginationBeforeExplicitOutput(
  services: CommandServices,
  reason: string,
): void {
  const inputHandler = services.getInputHandler();
  if (!inputHandler) return;
  inputHandler.flushDeferredPaginationIfNeeded(reason);
  if (inputHandler.hasDeferredPaginationPending()) {
    throw new Error(`출력 전 페이지네이션을 완료하지 못했습니다 (${reason})`);
  }
}

async function chooseSaveAsFormat(services: CommandServices): Promise<SaveFormat | null> {
  const sourceFormat = services.wasm.getSourceFormat();
  if (sourceFormat !== 'hml') return sourceFormat === 'hwpx' ? 'hwpx' : 'hwp';
  const context = getHmlSaveContext(services);
  return showHmlSaveFormatDialog(
    context.metadata,
    context.exporterAvailable,
  );
}

function createSaveBlob(services: CommandServices, format: SaveFormat): Blob {
  const bytes = exportDocumentForFormat(services.wasm, format);
  return new Blob([bytes as unknown as BlobPart], {
    type: SAVE_FORMAT_DETAILS[format].mimeType,
  });
}

function isHmlSaveEnabled(services: CommandServices): boolean {
  const context = getHmlSaveContext(services);
  return resolveHmlSaveCapability(
    context.metadata,
    context.exporterAvailable,
  ).hmlEnabled;
}

function getHmlSaveContext(services: CommandServices) {
  return readHmlSaveContext(
    () => services.wasm.getHmlOpenMetadata(),
    () => services.wasm.hasHmlExportCapability(),
  );
}

async function tryFileSystemSave(
  services: CommandServices,
  format: SaveFormat,
  blob: Blob,
  suggestedName: string,
  forceSaveAs: boolean,
  currentHandle: FileSystemFileHandleLike | null,
): Promise<SaveDocumentResult | 'cancelled'> {
  try {
    return await saveDocumentToFileSystem({
      blob,
      suggestedName,
      currentHandle,
      windowLike: window as FileSystemWindowLike,
      forceSaveAs,
      saveFormat: format,
    });
  } catch (error) {
    if (isUserCancelError(error)) return 'cancelled';
    console.warn('[file:save] File System Access API 실패, 폴백:', error);
    return { method: 'fallback', handle: null, fileName: suggestedName };
  }
}

function completeHandleSave(
  services: CommandServices,
  sourceFormat: string,
  result: SaveDocumentResult,
  reason: 'save' | 'save-as',
): void {
  if (sourceFormat === 'hml') markConvertedHmlSaveHandle(result.handle);
  services.wasm.currentFileHandle = result.handle;
  services.wasm.fileName = result.fileName;
  services.documentState.markClean(reason);
}

function downloadBlob(blob: Blob, fileName: string): void {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = fileName;
  anchor.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

async function promptFallbackName(
  suggestedName: string,
  format: SaveFormat,
): Promise<string | null> {
  const result = await showSaveAs(saveBaseNameFor(suggestedName, format), format);
  return result ? fileNameForFormat(result, format) : null;
}

async function saveAsFormat(services: CommandServices, format: SaveFormat): Promise<void> {
  try {
    flushDeferredPaginationBeforeExplicitOutput(services, 'save-as');
    const sourceFormat = services.wasm.getSourceFormat();
    const saveName = fileNameForFormat(services.wasm.fileName, format);
    const blob = createSaveBlob(services, format);
    const originalHandle = sourceFormat === 'hml' ? services.wasm.currentFileHandle : null;
    const result = await tryFileSystemSave(
      services,
      format,
      blob,
      saveName,
      true,
      originalHandle,
    );
    if (result === 'cancelled') return;
    if (result.method !== 'fallback') {
      completeHandleSave(services, sourceFormat, result, 'save-as');
      return;
    }
    const downloadName = await promptFallbackName(saveName, format);
    if (!downloadName) return;
    services.wasm.fileName = downloadName;
    downloadBlob(blob, downloadName);
    services.documentState.markClean('save-as');
  } catch (error) {
    reportSaveError('file:save-as', error);
  }
}

function reportSaveError(scope: string, error: unknown): void {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`[${scope}] 저장 실패:`, message);
  alert(`파일 저장에 실패했습니다:\n${message}`);
}

export type SaveCurrentDocumentResult = 'saved' | 'cancelled' | 'failed' | 'unsupported';

export async function saveCurrentDocument(services: CommandServices): Promise<SaveCurrentDocumentResult> {
  try {
    flushDeferredPaginationBeforeExplicitOutput(services, 'save');
    const sourceFormat = services.wasm.getSourceFormat();
    let target = resolveSaveTarget(
      sourceFormat,
      services.wasm.fileName,
      services.wasm.currentFileHandle,
    );
    const hmlEnabled = target.format !== 'hml' || isHmlSaveEnabled(services);
    if (requiresSaveFormatChoice(target, hmlEnabled)) {
      const format = await chooseSaveAsFormat(services);
      if (format === null) return 'cancelled';
      target = {
        ...target,
        format,
        forceSaveAs: target.forceSaveAs || format !== target.format,
        suggestedName: fileNameForFormat(services.wasm.fileName, format),
      };
    }
    const blob = createSaveBlob(services, target.format);
    const result = await tryFileSystemSave(
      services,
      target.format,
      blob,
      target.suggestedName,
      target.forceSaveAs,
      services.wasm.currentFileHandle,
    );
    if (result === 'cancelled') return 'cancelled';
    if (result.method !== 'fallback') {
      completeHandleSave(services, sourceFormat, result, 'save');
      return 'saved';
    }
    const downloadName = await fallbackNameForCurrentSave(services, target);
    if (!downloadName) return 'cancelled';
    downloadBlob(blob, downloadName);
    services.documentState.markClean('save');
    return 'saved';
  } catch (error) {
    reportSaveError('file:save', error);
    return 'failed';
  }
}

async function fallbackNameForCurrentSave(
  services: CommandServices,
  target: ReturnType<typeof resolveSaveTarget>,
): Promise<string | null> {
  if (!services.wasm.isNewDocument && !target.forceSaveAs) return target.suggestedName;
  const downloadName = await promptFallbackName(target.suggestedName, target.format);
  if (!downloadName) return null;
  services.wasm.fileName = downloadName;
  if (target.forceSaveAs) services.wasm.currentFileHandle = null;
  return downloadName;
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

function setupPrintDocument(
  doc: Document,
  fileName: string,
  printPages: PrintPage[],
  previewWindow: Window | null = null,
): void {
  doc.documentElement.lang = 'ko';

  doc.head.replaceChildren();
  const meta = doc.createElement('meta');
  meta.setAttribute('charset', 'UTF-8');
  const viewport = doc.createElement('meta');
  viewport.name = 'viewport';
  viewport.content = 'width=device-width, initial-scale=1.0';
  doc.head.append(meta, viewport);
  doc.title = previewWindow
    ? `${fileName} — 인쇄 미리보기`
    : pdfPrintTitle(fileName);
  appendPrintStyle(doc, printPages);

  doc.body.replaceChildren();
  doc.body.className = previewWindow ? 'rhwp-print-preview' : '';
  if (previewWindow) {
    appendPrintPreviewBar(doc, previewWindow, fileName, printPages.length);
  }
  for (const printPage of printPages) {
    appendSvgPage(doc, doc.body, printPage);
  }
}

function appendPrintPreviewBar(
  doc: Document,
  printWindow: Window,
  fileName: string,
  pageCount: number,
): void {
  const bar = doc.createElement('div');
  bar.className = 'print-preview-bar';
  bar.setAttribute('role', 'toolbar');
  bar.setAttribute('aria-label', '인쇄 미리보기 도구');

  const printButton = doc.createElement('button');
  printButton.id = 'print-btn';
  printButton.type = 'button';
  printButton.className = 'print-preview-primary';
  printButton.textContent = '인쇄';
  printButton.addEventListener('click', () => printWindow.print());

  const closeButton = doc.createElement('button');
  closeButton.id = 'close-btn';
  closeButton.type = 'button';
  closeButton.textContent = '닫기';
  closeButton.addEventListener('click', () => printWindow.close());

  const title = doc.createElement('span');
  title.className = 'print-preview-title';
  title.textContent = `${fileName} — ${pageCount}쪽`;

  bar.append(printButton, closeButton, title);
  doc.body.appendChild(bar);
}

function waitForHostAnimationFrame(): Promise<void> {
  return new Promise((resolve) => {
    window.requestAnimationFrame(() => resolve());
  });
}

async function waitForHostPaint(): Promise<void> {
  await waitForHostAnimationFrame();
  await waitForHostAnimationFrame();
}

function setPrintPreviewLoading(
  surface: PrintPreviewSurface,
  message: string,
): void {
  const loadingMessage = surface.document.getElementById('print-loading-message');
  if (loadingMessage) loadingMessage.textContent = message;
}

async function preparePrintPages(
  services: CommandServices,
  intent: PrintIntent,
  onProgress: (currentPage: number, pageCount: number) => void,
): Promise<PrintPage[]> {
  const wasm = services.wasm;
  const pageCount = wasm.pageCount;
  const printPages: PrintPage[] = [];
  for (let i = 0; i < pageCount; i++) {
    onProgress(i + 1, pageCount);
    const svg = wasm.renderPageSvgWithProfile(i, 'print');
    const pageInfo = wasm.getPageInfo(i);
    printPages.push(createPrintPage(svg, pageInfo, i));
    if (i % 5 === 0) await new Promise(resolve => setTimeout(resolve, 0));
  }
  return printPages;
}

let printJobActive = false;

function beginPrintJob(): boolean {
  if (printJobActive) {
    showToast({ message: '인쇄 문서를 준비하고 있습니다.', durationMs: 2500 });
    return false;
  }
  printJobActive = true;
  return true;
}

async function runPdfPrint(services: CommandServices): Promise<void> {
  if (!beginPrintJob()) return;

  const wasm = services.wasm;
  const statusEl = document.getElementById('sb-message');
  const originalStatus = statusEl?.textContent || '';
  let dialog: PdfPrintDialog | null = null;
  let surface: PrintSurface | null = null;
  let restoreStatus = true;
  let dialogVisible = false;
  let originalDocumentTitle: string | null = null;

  try {
    flushDeferredPaginationBeforeExplicitOutput(services, 'print-pdf');
    const pageCount = wasm.pageCount;
    if (pageCount === 0) return;

    const showGuidance = userSettings.getShowPdfPrintGuidance();
    dialog = new PdfPrintDialog(pageCount, showGuidance);
    dialogVisible = true;
    const decision = await dialog.showAsync();
    if (!decision.confirmed) return;
    if (decision.hideFutureGuidance) {
      try {
        userSettings.setShowPdfPrintGuidance(false);
      } catch (error) {
        console.warn('[file:print-to-pdf] 안내 표시 설정을 저장하지 못했습니다:', error);
      }
    }

    const initialProgress = printProgressText('pdf', 0, pageCount);
    if (statusEl) statusEl.textContent = initialProgress;
    dialog.updateProgress(0);
    await waitForHostPaint();

    surface = await createPrintSurface();
    const printPages = await preparePrintPages(services, 'pdf', (current, total) => {
      if (statusEl) statusEl.textContent = printProgressText('pdf', current, total);
      dialog?.updateProgress(current);
    });

    setupPrintDocument(surface.document, wasm.fileName, printPages);
    await waitForPrintSurfaceReady(surface);
    if (statusEl) statusEl.textContent = printReadyText('pdf');

    dialog.closeBeforePrint();
    dialogVisible = false;
    await waitForHostPaint();

    console.info(
      `[file:print-to-pdf] 브라우저 인쇄 호출 `
      + `(surface=iframe, pages=${pageCount}, profile=print)`,
    );
    // Chromium/Edge는 iframe을 인쇄해도 최상위 문서 제목을 PDF 기본 파일명으로
    // 사용한다. native print()가 열린 동안에만 원본 파일의 basename을 노출한다.
    originalDocumentTitle = document.title;
    document.title = pdfPrintTitle(wasm.fileName);
    surface.window.print();
  } catch (err) {
    restoreStatus = false;
    const msg = err instanceof Error ? err.message : String(err);
    console.error('[file:print-to-pdf]', msg);
    if (statusEl) statusEl.textContent = `PDF 준비 실패: ${msg}`;
    if (dialogVisible && dialog) {
      dialog.showError(msg);
    } else {
      showToast({ message: `PDF 준비에 실패했습니다: ${msg}`, durationMs: 5000 });
    }
  } finally {
    if (originalDocumentTitle !== null) {
      document.title = originalDocumentTitle;
    }
    surface?.dispose();
    printJobActive = false;
    if (restoreStatus && statusEl) statusEl.textContent = originalStatus;
  }
}

async function runPrintPreview(services: CommandServices): Promise<void> {
  if (!beginPrintJob()) return;

  const wasm = services.wasm;
  const statusEl = document.getElementById('sb-message');
  const originalStatus = statusEl?.textContent || '';
  let surface: PrintPreviewSurface | null = null;
  let keepPreviewOpen = false;
  let restoreStatus = true;

  try {
    flushDeferredPaginationBeforeExplicitOutput(services, 'print');
    const pageCount = wasm.pageCount;
    if (pageCount === 0) return;

    // popup 허용을 위해 사용자 클릭에서 첫 await 전에 창을 연다.
    const surfacePromise = createPrintPreviewSurface();
    const initialProgress = printProgressText('print', 0, pageCount);
    if (statusEl) statusEl.textContent = initialProgress;

    surface = await surfacePromise;
    if (!surface) return;
    setPrintPreviewLoading(surface, initialProgress);

    const printPages = await preparePrintPages(services, 'print', (current, total) => {
      const progressText = printProgressText('print', current, total);
      if (statusEl) statusEl.textContent = progressText;
      if (surface) setPrintPreviewLoading(surface, progressText);
    });

    setupPrintDocument(surface.document, wasm.fileName, printPages, surface.window);
    await waitForPrintSurfaceReady(surface);
    if (statusEl) statusEl.textContent = printReadyText('print');
    keepPreviewOpen = true;

    console.info(
      `[file:print] 인쇄 미리보기 준비 완료 `
      + `(surface=window, pages=${pageCount}, profile=print)`,
    );
  } catch (err) {
    restoreStatus = false;
    const msg = err instanceof Error ? err.message : String(err);
    console.error('[file:print]', msg);
    if (statusEl) statusEl.textContent = `인쇄 미리보기 실패: ${msg}`;
    if (err instanceof PrintPreviewBlockedError) {
      alert('인쇄 미리보기 팝업이 차단되었습니다. 팝업 허용 후 다시 시도해주세요.');
    } else {
      showToast({ message: `인쇄 미리보기에 실패했습니다: ${msg}`, durationMs: 5000 });
    }
  } finally {
    if (!keepPreviewOpen) surface?.close();
    printJobActive = false;
    if (restoreStatus && statusEl) statusEl.textContent = originalStatus;
  }
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
    execute: openFileViaPicker,
  },
  {
    // 최근 문서 재열기 — 저장된 핸들 권한 재확인 후 라이브 파일 로드. params.id로 레코드 지정.
    // #2285 범위: 바이트 스냅샷 폴백 없음. 권한 거부는 항목 유지(다음에 다시 시도 가능),
    // 파일 이동/삭제(getFile 실패)는 항목 제거 + 안내. 결과 규칙은
    // recent-open.ts(openRecentEntry) — 테스트 가능한 순수 로직으로 분리.
    id: 'file:open-recent',
    label: '최근 문서 열기',
    async execute(services, params) {
      const id = typeof params?.id === 'string' ? params.id : undefined;
      if (!id) return;
      const recents = await listRecentDocs();
      const entry = recents.find((r) => r.id === id);
      if (!entry) {
        showToast({ message: '최근 문서 정보를 찾을 수 없습니다.', durationMs: 2500 });
        return;
      }

      await openRecentEntry(entry, {
        ensurePermission: ensureReadPermission,
        readFile: readFileFromHandle,
        remove: removeRecentDoc,
        toast: (message, durationMs) => showToast({ message, durationMs }),
        emitOpen: (payload) => services.eventBus.emit('open-document-bytes', payload),
        // 메타-only 항목: 핸들이 없어 자동 재열기 불가 → 열기 대화상자를 다시 연다.
        requestReopen: () => { void openFileViaPicker(services); },
      });
    },
  },
  {
    // 최근 문서 목록 전체 삭제.
    id: 'file:clear-recent',
    label: '최근 문서 목록 지우기',
    async execute() {
      if (!confirm('최근 문서 목록을 모두 지우시겠습니까?')) return;
      await clearRecentDocs();
      showToast({ message: '최근 문서 목록을 지웠습니다.', durationMs: 2200 });
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
      const format = await chooseSaveAsFormat(services);
      if (format !== null) await saveAsFormat(services, format);
    },
  },
  {
    // [#1613] HWP 형식으로 저장 — 출처 무관 HWP 출력.
    id: 'file:save-as-hwp',
    label: 'HWP 형식으로 저장',
    canExecute: (ctx) => ctx.hasDocument,
    async execute(services) {
      await saveAsFormat(services, 'hwp');
    },
  },
  {
    // [#1613] HWPX 형식으로 저장 — 출처 무관 HWPX 출력.
    id: 'file:save-as-hwpx',
    label: 'HWPX 형식으로 저장',
    canExecute: (ctx) => ctx.hasDocument,
    async execute(services) {
      await saveAsFormat(services, 'hwpx');
    },
  },
  {
    id: 'file:page-setup',
    label: '편집 용지',
    icon: 'icon-page-setup',
    shortcutLabel: 'F7',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const dialog = new PageSetupDialog(services.wasm, services.eventBus, 0, services);
      dialog.show();
    },
  },
  {
    id: 'file:print-to-pdf',
    label: 'PDF로 저장…',
    canExecute: (ctx) => ctx.hasDocument,
    async execute(services) {
      await runPdfPrint(services);
    },
  },
  {
    id: 'file:print',
    label: '인쇄',
    icon: 'icon-print',
    shortcutLabel: 'Ctrl+P',
    canExecute: (ctx) => ctx.hasDocument,
    async execute(services) {
      await runPrintPreview(services);
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
