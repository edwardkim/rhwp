// Repository product Worker: root WASM, common IR, no POC/HWPX roundtrip.
import init, * as product from '../../pkg/rhwp.js';
const { HwpDocument } = product;

let ready;
let initialized = false;
let document;
let prepared;
let reservedInputLength;
let queue = Promise.resolve();

self.onmessage = ({ data }) => {
  // Initialization is async; serialize messages so a later request cannot overtake import.
  queue = queue.then(async () => {
    let candidate;
    try {
      if (!initialized) self.postMessage({ id: data.id, phase: 'initializing' });
      await (ready ??= init());
      initialized = true;
      let result;
      let limits;
      let encoding;
      switch (data.method) {
        case 'policy':
          result = JSON.parse(HwpDocument.getPdfImportPolicyWithOptions(data.options));
          if (!result || !product.PdfImportRequest?.prepare) throw new Error('PDF_IMPORT_DISABLED: bounded PDF preparation is unavailable');
          prepared?.free();
          prepared = product.PdfImportRequest.prepare(data.options);
          reservedInputLength = undefined;
          limits = JSON.parse(prepared.getTransferLimits());
          break;
        case 'reserveInput':
          if (!prepared || !Number.isSafeInteger(data.byteLength) || data.byteLength < 0 || data.byteLength > 0xffffffff) throw new Error('PDF_RESOURCE_POLICY: invalid input reservation');
          prepared.reserveInput(data.byteLength);
          reservedInputLength = data.byteLength;
          result = true;
          break;
        case 'importPdf': {
          self.postMessage({ id: data.id, phase: 'importing' });
          if (!product.PdfImportRequest) throw new Error('PDF_IMPORT_DISABLED: bounded PDF request API is unavailable');
          if (!prepared || !(data.bytes instanceof Uint8Array) || data.bytes.byteLength !== reservedInputLength) throw new Error('PDF_WORKER_PROTOCOL: reserve the input before import');
          candidate = prepared;
          prepared = undefined;
          if (typeof candidate.openWithProgress === 'function') {
            candidate.openWithProgress(data.bytes, (sourcePage, sourcePageCount, completedPages, totalPages) => {
              self.postMessage({ id: data.id, phase: 'importing',
                progress: { sourcePage, sourcePageCount, completedPages, totalPages } });
            });
          } else {
            // Older bundles still open, without inventing page progress.
            candidate.open(data.bytes);
          }
          result = candidate.inspect();
          encoding = 'json';
          self.postMessage({ id: data.id, phase: 'ready' });
          const previous = document;
          document = candidate;
          candidate = undefined;
          previous?.free();
          break;
        }
        case 'inspect':
          requireDocument();
          result = document.inspect();
          encoding = 'json';
          break;
        case 'render':
          requireDocument();
          if (!Number.isInteger(data.page) || data.page < 0 || data.page >= document.pageCount()) {
            throw new Error('PDF_PAGE_SELECTION: output page is out of range');
          }
          {
            const limit = JSON.parse(document.getTransferLimits()).outputBytes;
            const maximum = data.maximumBytes ?? limit;
            if (!Number.isSafeInteger(maximum) || maximum <= 0 || maximum > limit) throw new Error('PDF_RESOURCE_POLICY: invalid SVG byte limit');
            result = document.renderPageSvgLimited(data.page, maximum);
          }
          break;
        case 'export':
          requireDocument();
          if (data.format !== 'hwp' && data.format !== 'hwpx') {
            throw new Error('PDF_EXPORT_FORMAT: expected hwp or hwpx');
          }
          {
            const limit = JSON.parse(document.getTransferLimits()).outputBytes;
            const maximum = data.maximumBytes ?? limit;
            if (!Number.isSafeInteger(maximum) || maximum <= 0 || maximum > limit) {
              throw new Error('PDF_RESOURCE_POLICY: invalid output byte limit');
            }
            result = document.exportDocument(data.format, maximum);
          }
          break;
        default:
          throw new Error('PDF_WORKER_PROTOCOL: unknown method');
      }
      // Structured clone gives the owner a separate UTF-16 response backing.
      if (typeof result === 'string') document.reserveResponseCopy(result.length * 2);
      self.postMessage({ id: data.id, result, limits, encoding, remainingDecodedBytes: (prepared ?? document)?.remainingDecodedBytes() }, result instanceof Uint8Array ? [result.buffer] : []);
    } catch (error) {
      candidate?.free();
      const message = String(error);
      const code = error instanceof RangeError ? 'PDF_WORKER_MEMORY'
        : message.includes('SERIALIZE_RESOURCE_LIMIT:') ? 'PDF_RESOURCE_LIMIT'
        : message.match(/\bPDF_[A-Z_]+\b/)?.[0] ?? 'PDF_WORKER_FAILED';
      self.postMessage({ id: data.id, error: {
        code,
        message: message.slice(0, 8192),
        fatal: !initialized || error instanceof WebAssembly.RuntimeError || code === 'PDF_WORKER_MEMORY'
          || (['reserveInput', 'export', 'render', 'inspect'].includes(data.method)
            && !['PDF_RESOURCE_POLICY', 'PDF_EXPORT_FORMAT', 'PDF_PAGE_SELECTION', 'PDF_NO_DOCUMENT'].includes(code)),
      } });
    }
  });
};

function requireDocument() {
  if (!document) throw new Error('PDF_NO_DOCUMENT: import a PDF first');
}
