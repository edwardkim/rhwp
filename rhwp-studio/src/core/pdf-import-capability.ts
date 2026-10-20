/** An older bundle can expose importPdf while PDF support is compiled out. */
export function hasPdfImportCapability(api: unknown): boolean {
  const candidate = api as { supportsPdfImport?: () => unknown } | null;
  return typeof candidate?.supportsPdfImport === 'function'
    && candidate.supportsPdfImport() === true;
}
