/**
 * Display/save name at the viewer boundary (#6961).
 * DownloadItem.filename is an absolute local path, not a basename.
 * Treat both slash styles as path separators, including literal backslashes in names.
 * Preserve the leaf verbatim: percent sequences here are filename characters,
 * not URL encoding (URLSearchParams handles transport encoding separately).
 */
export function documentFilename(filename) {
  if (typeof filename !== 'string') return '';
  return filename.split(/[/\\]/).pop() || '';
}
