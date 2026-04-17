import { saveDocumentToFileSystem } from './file-system-access.ts';

function isAbortError(error) {
  return error instanceof DOMException && error.name === 'AbortError';
}

function formatErrorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

export async function performDocumentSave(options) {
  const {
    bytes,
    saveName,
    currentHandle,
    isNewDocument,
    windowLike,
    promptForName,
    onDownload,
    onStatus,
    onAlert,
    onTrace,
  } = options;
  const blob = new Blob([bytes], { type: 'application/x-hwp' });

  try {
    const saveResult = await saveDocumentToFileSystem({
      blob,
      suggestedName: saveName,
      currentHandle,
      windowLike,
      onTrace,
    });

    if (saveResult.method !== 'fallback') {
      return {
        ok: true,
        method: saveResult.method,
        fileName: saveResult.fileName,
        handle: saveResult.handle,
      };
    }
  } catch (error) {
    if (isAbortError(error)) {
      return { ok: false, reason: 'aborted' };
    }

    if (currentHandle) {
      const message = `저장 실패: ${formatErrorMessage(error)}`;
      onStatus?.(message);
      onAlert?.(`파일 저장에 실패했습니다:\n${formatErrorMessage(error)}`);
      return {
        ok: false,
        reason: 'existing-save-failed',
        error,
      };
    }
  }

  let downloadName = saveName;
  if (isNewDocument) {
    const baseName = saveName.replace(/\.hwp$/i, '');
    const result = await promptForName?.(baseName);
    if (!result) return { ok: false, reason: 'aborted' };
    downloadName = result;
  }

  onDownload(blob, downloadName);
  onTrace?.({
    stage: 'save-fallback',
    method: 'fallback',
    fileName: downloadName,
  });

  return {
    ok: true,
    method: 'fallback',
    fileName: downloadName,
    handle: null,
  };
}
