import type { SaveFormat } from './save-format.ts';

export interface DocumentFormatExporter {
  exportHml(): Uint8Array;
  exportHwp(): Uint8Array;
  exportHwpx(): Uint8Array;
}

export function exportDocumentForFormat(
  exporter: DocumentFormatExporter,
  format: SaveFormat,
): Uint8Array {
  if (format === 'hml') return exporter.exportHml();
  if (format === 'hwpx') return exporter.exportHwpx();
  return exporter.exportHwp();
}
