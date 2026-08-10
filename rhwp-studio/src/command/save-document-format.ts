import type { SaveFormat } from './save-format.ts';
import type {
  ContentLossReport,
  DocumentExportArtifact,
} from '../core/export-content-loss.ts';

export interface DocumentFormatExporter {
  exportHml(): Uint8Array;
  exportHwp(): Uint8Array;
  exportHwpx(): Uint8Array;
}

export interface DocumentFormatPasswordExporter extends DocumentFormatExporter {
  exportHwpWithPassword(password: string): Uint8Array;
  exportHwpxWithPassword(password: string): Uint8Array;
}

/** 명시적 저장에서 바이트와 같은 직렬화의 내용 손실을 함께 반환하는 표면. */
export interface ReportedDocumentFormatExporter extends DocumentFormatExporter {
  exportHwpWithReport(): DocumentExportArtifact;
  exportHwpxWithReport(): DocumentExportArtifact;
}

export interface ReportedDocumentFormatPasswordExporter
  extends ReportedDocumentFormatExporter, DocumentFormatPasswordExporter {
  exportHwpWithPasswordAndReport(password: string): DocumentExportArtifact;
  exportHwpxWithPasswordAndReport(password: string): DocumentExportArtifact;
}

export interface SaveExportArtifact {
  bytes: Uint8Array;
  /** HML은 손실을 보고하는 serializer 경계가 없으므로 null이다. */
  contentLoss: ContentLossReport | null;
}

export function exportDocumentForFormat(
  exporter: DocumentFormatExporter,
  format: SaveFormat,
): Uint8Array {
  if (format === 'hml') return exporter.exportHml();
  if (format === 'hwpx') return exporter.exportHwpx();
  return exporter.exportHwp();
}

/**
 * 사용자 명시 저장용 exporter를 고른다.
 *
 * HWP/HWPX는 반드시 reported API를 사용한다. byte-only API는 autosave, embed RPC,
 * history, compare, hwpctl, digest 같은 보조 소비자의 기존 계약으로 남으며, 이 흐름의
 * 손실 보고로 간주하지 않는다.
 */
export function exportDocumentWithReportForFormat(
  exporter: ReportedDocumentFormatExporter,
  format: SaveFormat,
): SaveExportArtifact {
  if (format === 'hml') {
    return { bytes: exporter.exportHml(), contentLoss: null };
  }
  return format === 'hwpx'
    ? exporter.exportHwpxWithReport()
    : exporter.exportHwpWithReport();
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

/** 비밀번호 명시 저장에서도 산출 바이트와 보고서를 한 transaction으로 받는다. */
export function exportPasswordProtectedDocumentWithReportForFormat(
  exporter: ReportedDocumentFormatPasswordExporter,
  format: Exclude<SaveFormat, 'hml'>,
  password: string,
): SaveExportArtifact {
  return format === 'hwpx'
    ? exporter.exportHwpxWithPasswordAndReport(password)
    : exporter.exportHwpWithPasswordAndReport(password);
}
