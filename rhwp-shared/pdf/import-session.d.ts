export interface PdfPageProgress {
  /** One-based original PDF page being converted. */
  sourcePage: number;
  sourcePageCount: number;
  /** Completed selected pages, not elapsed-time progress. */
  completedPages: number;
  totalPages: number;
}
export interface PdfRequestOptions {
  signal?: AbortSignal;
  timeoutMs?: number;
  onProgress?: (phase: 'initializing' | 'importing' | 'ready', progress?: PdfPageProgress) => void;
}
export interface PdfImportOptions {
  password?: string;
  layout?: 'fidelity' | 'editable';
  resourceProfile?: 'standard' | 'constrained';
  resourceLimits?: Record<string, unknown>;
  sourcePages?: number[];
  includeTableInference?: boolean;
}
export interface PdfImportResult {
  pageCount: number;
  diagnostics: Record<string, unknown>;
}
export class PdfImportSession {
  constructor(options?: { workerFactory?: () => Worker });
  importPdf(bytes: Uint8Array, options?: PdfImportOptions, request?: PdfRequestOptions): Promise<PdfImportResult>;
  inspect(options?: PdfRequestOptions): Promise<PdfImportResult>;
  renderPageSvg(page: number, options?: PdfRequestOptions & { outputBytes?: number }): Promise<string>;
  exportDocument(format?: 'hwp' | 'hwpx', options?: PdfRequestOptions & { outputBytes?: number }): Promise<Uint8Array>;
  getEditorReopenLimits(): { inputBytes: number; decodedBytes: number; options: string };
  dispose(): void;
}
