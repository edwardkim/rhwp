export const PDF_TWIN_LOOKUP_PATH = '/__rhwp_harness/pdf-twin';
export const DOCUMENT_ERROR_LOG_PATH = '/__rhwp_harness/pdf-diff';
export const PDF_PAGE_PATH_PREFIX = '/__rhwp_harness/pdf-page/';
export const DOCUMENT_ERROR_CAPABILITY_HEADER = 'x-rhwp-harness-capability';

export interface PdfTwinFound {
  status: 'found';
  pdfName: string;
  pdfPageUrl: string;
  pdfPageWidth: number;
  pdfPageCount: number | null;
  relativeDirectory: string;
  errorLogCapability: string;
}

export type PdfTwinLookupResponse =
  | PdfTwinFound
  | { status: 'none' }
  | { status: 'ambiguous' };
