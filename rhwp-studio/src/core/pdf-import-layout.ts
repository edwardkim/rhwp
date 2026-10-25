/**
 * PDF 가져오기 배치 선택 (#8220).
 *
 * - `fidelity`(기본): 인식한 본문 영역마다 위치 고정 글상자를 둔다. 쪽 모양을 원본 그대로 따른다.
 * - `editable`: 본문 문단을 구역의 본문 흐름(다단·단/쪽 나누기)으로 가져온다.
 *   단·쪽 경계에서 끊긴 문단을 하나로 잇는다.
 *
 * 문서를 여는 경로는 하나다. PDF가 아니거나 기본 배치이면 기존 생성자를 그대로 쓴다.
 */
export type PdfImportLayout = 'fidelity' | 'editable';

export const PDF_IMPORT_LAYOUTS: readonly PdfImportLayout[] = ['fidelity', 'editable'];

export function normalizePdfImportLayout(value: unknown): PdfImportLayout {
  return value === 'editable' ? 'editable' : 'fidelity';
}

/** 엔진의 형식 판별(`parser::detect_format`)과 같이 `%PDF-`로 시작하는 바이트만 PDF다. */
export function isPdfBytes(data: Uint8Array): boolean {
  return data.length >= 5
    && data[0] === 0x25 && data[1] === 0x50 && data[2] === 0x44
    && data[3] === 0x46 && data[4] === 0x2d;
}

/** 문서 생성에 필요한 WASM 정적 API의 최소 모양. */
export interface DocumentFactory<T> {
  new (data: Uint8Array): T;
  importPdf?: (data: Uint8Array, options: unknown) => T;
}

/**
 * 바이트와 배치 설정에 맞는 생성 경로로 문서를 연다.
 * 편집 우선 배치를 고른 PDF만 `importPdf(data, { layout: 'editable' })`를 쓴다.
 * 옛 WASM처럼 `importPdf`가 없으면 기본 배치로 연다.
 */
export function createDocumentForBytes<T>(
  factory: DocumentFactory<T>,
  data: Uint8Array,
  layout: PdfImportLayout,
): T {
  if (layout === 'editable' && typeof factory.importPdf === 'function' && isPdfBytes(data)) {
    return factory.importPdf(data, { layout: 'editable' });
  }
  return new factory(data);
}
