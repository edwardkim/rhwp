import type { CommandDef } from '../types';
import { PageSetupDialog } from '@/ui/page-setup-dialog';
import { AboutDialog } from '@/ui/about-dialog';
import { showConfirm } from '@/ui/confirm-dialog';
import { showSaveAs } from '@/ui/save-as-dialog';
import {
  pickOpenFileHandle,
  readFileFromHandle,
  saveDocumentToFileSystem,
  type FileSystemWindowLike,
} from '@/command/file-system-access';

export const fileCommands: CommandDef[] = [
  {
    id: 'file:new-doc',
    label: '새로 만들기',
    icon: 'icon-new-doc',
    shortcutLabel: 'Alt+N',
    canExecute: () => true,
    async execute(services) {
      const ctx = services.getContext();
      if (ctx.hasDocument) {
        const ok = await showConfirm(
          '새로 만들기',
          '현재 문서를 닫고 새 문서를 만드시겠습니까?\n저장하지 않은 내용은 사라집니다.',
        );
        if (!ok) return;
      }
      services.eventBus.emit('create-new-document');
    },
  },
  {
    id: 'file:open',
    label: '열기',
    async execute(services) {
      try {
        const handle = await pickOpenFileHandle(window as FileSystemWindowLike);
        if (!handle) {
          document.getElementById('file-input')?.click();
          return;
        }

        const { bytes, name } = await readFileFromHandle(handle);
        services.eventBus.emit('open-document-bytes', {
          bytes,
          fileName: name,
          fileHandle: handle,
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
    // #196: HWPX 출처는 저장 비활성화 (베타 단계, #197 완전 변환기 완료 시까지)
    canExecute: (ctx) => ctx.hasDocument && ctx.sourceFormat !== 'hwpx',
    async execute(services) {
      try {
        const saveName = services.wasm.fileName;
        const sourceFormat = services.wasm.getSourceFormat();
        const isHwpx = sourceFormat === 'hwpx';
        const bytes = isHwpx ? services.wasm.exportHwpx() : services.wasm.exportHwp();
        const mimeType = isHwpx ? 'application/hwp+zip' : 'application/x-hwp';
        const blob = new Blob([bytes as unknown as BlobPart], { type: mimeType });
        console.log(`[file:save] format=${sourceFormat}, isHwpx=${isHwpx}, ${bytes.length} bytes`);

        // 1) 기존 파일 handle이 있으면 같은 파일에 저장, 없으면 save picker 시도
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
            console.log(`[file:save] ${saveResult.fileName} (${(bytes.length / 1024).toFixed(1)}KB)`);
            return;
          }
        } catch (e) {
          // 사용자가 취소하면 AbortError 발생 — 무시
          if (e instanceof DOMException && e.name === 'AbortError') return;
          // 그 외 오류는 폴백으로 진행
          console.warn('[file:save] File System Access API 실패, 폴백:', e);
        }

        // 2) 폴백: 새 문서인 경우 자체 파일이름 대화상자 표시
        let downloadName = saveName;
        if (services.wasm.isNewDocument) {
          const baseName = saveName.replace(/\.hwp$/i, '');
          const result = await showSaveAs(baseName);
          if (!result) return;
          downloadName = result;
          services.wasm.fileName = downloadName;
        }

        // 3) Blob 다운로드
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = downloadName;
        a.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000);

        console.log(`[file:save] ${downloadName} (${(bytes.length / 1024).toFixed(1)}KB)`);
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        console.error('[file:save] 저장 실패:', msg);
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
        const svgPages: string[] = [];
        for (let i = 0; i < pageCount; i++) {
          if (statusEl) statusEl.textContent = `인쇄 준비 중... (${i + 1}/${pageCount})`;
          const svg = wasm.renderPageSvg(i);
          svgPages.push(svg);
          // UI 갱신을 위한 양보
          if (i % 5 === 0) await new Promise(r => setTimeout(r, 0));
        }

        // 첫 페이지 정보로 용지 크기 결정
        const pageInfo = wasm.getPageInfo(0);
        const widthMm = Math.round(pageInfo.width * 25.4 / 96);
        const heightMm = Math.round(pageInfo.height * 25.4 / 96);

        // 인쇄 전용 창 생성
        const printWin = window.open('', '_blank');
        if (!printWin) {
          alert('팝업이 차단되었습니다. 팝업 허용 후 다시 시도해주세요.');
          return;
        }

        printWin.document.write(`<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>${wasm.fileName} — 인쇄</title>
<style>
  @page { size: ${widthMm}mm ${heightMm}mm; margin: 0; }
  * { margin: 0; padding: 0; }
  body { background: #fff; }
  .page { page-break-after: always; width: ${widthMm}mm; height: ${heightMm}mm; overflow: hidden; }
  .page:last-child { page-break-after: auto; }
  .page svg { width: 100%; height: 100%; }
  @media screen {
    body { background: #e5e7eb; display: flex; flex-direction: column; align-items: center; gap: 16px; padding: 16px; }
    .page { background: #fff; box-shadow: 0 2px 8px rgba(0,0,0,0.15); }
    .print-bar { position: fixed; top: 0; left: 0; right: 0; background: #1e293b; color: #fff; padding: 8px 16px; display: flex; align-items: center; gap: 12px; font: 14px sans-serif; z-index: 100; }
    .print-bar button { padding: 6px 16px; background: #2563eb; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; }
    .print-bar button:hover { background: #1d4ed8; }
    body { padding-top: 56px; }
  }
  @media print { .print-bar { display: none; } }
</style>
</head>
<body>
<div class="print-bar">
  <button id="print-btn">인쇄</button>
  <button id="close-btn" style="background:#475569">닫기</button>
  <span>${wasm.fileName} — ${pageCount}페이지</span>
</div>
${svgPages.map(svg => `<div class="page">${svg}</div>`).join('\n')}

</body>
</html>`);
        printWin.document.close();

        // CSP 안전: DOM API로 이벤트 바인딩 (인라인 스크립트 사용 안 함)
        printWin.document.getElementById('print-btn')?.addEventListener('click', () => {
          printWin.print();
        });
        printWin.document.getElementById('close-btn')?.addEventListener('click', () => {
          printWin.close();
        });

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
