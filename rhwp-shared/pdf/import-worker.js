// Repository product Worker: root WASM, common IR, no POC/HWPX roundtrip.
import init, { HwpDocument } from '../../pkg/rhwp.js';

let ready;
let initialized = false;
let document;
let queue = Promise.resolve();

self.onmessage = ({ data }) => {
  // Initialization is async; serialize messages so a later request cannot overtake import.
  queue = queue.then(async () => {
    let candidate;
    try {
      await (ready ??= init());
      initialized = true;
      let result;
      switch (data.method) {
        case 'policy':
          result = JSON.parse(HwpDocument.getPdfImportPolicy());
          if (!result) throw new Error('PDF_IMPORT_DISABLED: PDF import is not compiled in');
          break;
        case 'importPdf': {
          candidate = HwpDocument.importPdf(data.bytes, data.options);
          result = {
            pageCount: candidate.pageCount(),
            diagnostics: JSON.parse(candidate.getPdfImportDiagnostics()),
          };
          const previous = document;
          document = candidate;
          candidate = undefined;
          previous?.free();
          break;
        }
        case 'inspect':
          requireDocument();
          result = { pageCount: document.pageCount(), diagnostics: JSON.parse(document.getPdfImportDiagnostics()) };
          break;
        case 'render':
          requireDocument();
          if (!Number.isInteger(data.page) || data.page < 0 || data.page >= document.pageCount()) {
            throw new Error('PDF_PAGE_SELECTION: output page is out of range');
          }
          result = document.renderPageSvg(data.page);
          break;
        case 'export':
          requireDocument();
          if (data.format !== 'hwp' && data.format !== 'hwpx') {
            throw new Error('PDF_EXPORT_FORMAT: expected hwp or hwpx');
          }
          result = data.format === 'hwp' ? document.exportHwp() : document.exportHwpx();
          break;
        default:
          throw new Error('PDF_WORKER_PROTOCOL: unknown method');
      }
      self.postMessage({ id: data.id, result }, result instanceof Uint8Array ? [result.buffer] : []);
    } catch (error) {
      candidate?.free();
      const message = String(error);
      self.postMessage({ id: data.id, error: {
        code: message.match(/\bPDF_[A-Z_]+\b/)?.[0] ?? 'PDF_WORKER_FAILED',
        message,
        fatal: !initialized || error instanceof WebAssembly.RuntimeError,
      } });
    }
  });
};

function requireDocument() {
  if (!document) throw new Error('PDF_NO_DOCUMENT: import a PDF first');
}
