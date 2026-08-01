import type { SaveFormat } from './save-format.ts';

export interface DocumentFormatExporter {
  exportHml(): Uint8Array;
  exportHwp(): Uint8Array;
  exportHwpx(): Uint8Array;
}

export interface DocumentFormatPasswordExporter extends DocumentFormatExporter {
  exportHwpWithPassword(password: string): Uint8Array;
  exportHwpxWithPassword(password: string): Uint8Array;
}

export function exportDocumentForFormat(
  exporter: DocumentFormatExporter,
  format: SaveFormat,
): Uint8Array {
  if (format === 'hml') return exporter.exportHml();
  if (format === 'hwpx') return exporter.exportHwpx();
  return exporter.exportHwp();
}

/** HML 이외의 출력 형식에 password serializer를 선택한다. */
export function exportPasswordProtectedDocumentForFormat(
  exporter: DocumentFormatPasswordExporter,
  format: Exclude<SaveFormat, 'hml'>,
  password: string,
): Uint8Array {
  if (format === 'hwpx') return exporter.exportHwpxWithPassword(password);
  return exporter.exportHwpWithPassword(password);
}
