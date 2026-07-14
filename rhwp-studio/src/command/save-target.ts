import { SAVE_FORMAT_DETAILS, type SaveFormat } from './save-format.ts';

export type { SaveFormat } from './save-format.ts';

export interface SaveTarget {
  format: SaveFormat;
  forceSaveAs: boolean;
  suggestedName: string;
}

export interface NamedSaveHandle {
  name: string;
}

const convertedHmlSaveHandles = new WeakSet<object>();

export function markConvertedHmlSaveHandle(handle: NamedSaveHandle | null): void {
  if (handle) convertedHmlSaveHandles.add(handle);
}

export function forgetConvertedHmlSaveHandle(handle: NamedSaveHandle | null): void {
  if (handle) convertedHmlSaveHandles.delete(handle);
}

function saveFormatForFileName(fileName: string): SaveFormat | null {
  const normalized = fileName.trim().toLowerCase();
  if (normalized.endsWith('.hml')) return 'hml';
  if (normalized.endsWith('.hwpx')) return 'hwpx';
  if (normalized.endsWith('.hwp')) return 'hwp';
  return null;
}

export function fileNameForFormat(fileName: string, format: SaveFormat): string {
  const extension = SAVE_FORMAT_DETAILS[format].extension;
  const trimmed = fileName.trim() || `document${extension}`;
  if (/\.(hwp|hwpx|hml)$/i.test(trimmed)) {
    return trimmed.replace(/\.(hwp|hwpx|hml)$/i, extension);
  }
  return `${trimmed}${extension}`;
}

export function requiresSaveFormatChoice(target: SaveTarget, hmlEnabled: boolean): boolean {
  return target.forceSaveAs || (target.format === 'hml' && !hmlEnabled);
}

export function resolveSaveTarget(
  sourceFormat: string,
  fileName: string,
  currentHandle?: NamedSaveHandle | null,
): SaveTarget {
  const hasConvertedHmlTarget = sourceFormat === 'hml'
    && currentHandle != null
    && convertedHmlSaveHandles.has(currentHandle);

  if (sourceFormat === 'hml' && !hasConvertedHmlTarget) {
    return {
      format: 'hml',
      forceSaveAs: true,
      suggestedName: fileNameForFormat(fileName, 'hml'),
    };
  }

  const convertedTarget = currentHandle?.name ?? fileName;
  const convertedFormat = hasConvertedHmlTarget
    ? saveFormatForFileName(convertedTarget)
    : null;
  const format: SaveFormat = convertedFormat
    ?? (sourceFormat === 'hwpx' ? 'hwpx' : 'hwp');

  return {
    format,
    forceSaveAs: false,
    suggestedName: fileNameForFormat(fileName, format),
  };
}
