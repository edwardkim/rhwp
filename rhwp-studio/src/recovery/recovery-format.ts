import type { AutosaveDraft } from './autosave-store.ts';

function baseNameWithoutKnownExtension(fileName: string): string {
  const trimmed = fileName.trim() || '문서.hwp';
  const dot = trimmed.lastIndexOf('.');
  if (dot <= 0) return trimmed;

  const ext = trimmed.slice(dot).toLowerCase();
  if (ext === '.hwp' || ext === '.hwpx') {
    return trimmed.slice(0, dot);
  }
  return trimmed;
}

export function recoveryFileName(fileName: string, sourceFormat = 'hwp'): string {
  const base = baseNameWithoutKnownExtension(fileName);
  // autosave draft는 exportHwp() 결과이므로 HWPX 출처도 복구본은 HWP로 연다.
  if (sourceFormat.toLowerCase() === 'hwpx') return `${base} 복구본.hwp`;
  return `${base} 복구본.hwp`;
}

export function formatDraftSavedAt(timestamp: number): string {
  if (!Number.isFinite(timestamp) || timestamp <= 0) return '저장 시각 알 수 없음';
  return new Date(timestamp).toLocaleString('ko-KR');
}

export function formatDraftSize(byteLength: number): string {
  if (!Number.isFinite(byteLength) || byteLength < 0) return '크기 알 수 없음';
  if (byteLength < 1024) return `${byteLength} B`;
  const kb = byteLength / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  return `${(kb / 1024).toFixed(1)} MB`;
}

export function describeDraft(draft: AutosaveDraft): string {
  const format = draft.sourceFormat.toUpperCase();
  const suffix = draft.sourceFormat.toLowerCase() === 'hwpx' ? ' → HWP 복구본' : '';
  return `${formatDraftSavedAt(draft.savedAt)} · ${formatDraftSize(draft.byteLength)} · ${format}${suffix}`;
}
