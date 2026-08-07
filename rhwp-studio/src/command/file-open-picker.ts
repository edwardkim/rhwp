import {
  canUseOpenFilePicker,
  pickOpenFileHandle,
  readFileFromHandle,
  type FileSystemFileHandleLike,
  type FileSystemWindowLike,
} from './file-system-access.ts';

export interface OpenFilePickerDependencies {
  canReplace: () => Promise<boolean>;
  windowLike: FileSystemWindowLike;
  findFileInput: () => HTMLInputElement | null;
  emitOpenDocument: (payload: {
    bytes: Uint8Array;
    fileName: string;
    fileHandle: FileSystemFileHandleLike;
    skipUnsavedGuard: true;
  }) => void;
  warn: (message: string, error: unknown) => void;
  alert: (message: string) => void;
}

/**
 * File System Access picker로 문서를 열고, 지원하지 않거나 접근이 거부되면 숨김 file input으로
 * 전환한다. `AbortError`는 사용자가 picker를 취소한 경우이므로 fallback을 열지 않는다.
 */
export async function openDocumentViaPicker(deps: OpenFilePickerDependencies): Promise<void> {
  if (!await deps.canReplace()) return;

  let nativeOpenPickerAvailable = canUseOpenFilePicker(deps.windowLike);
  let handle: FileSystemFileHandleLike | null = null;
  if (nativeOpenPickerAvailable) {
    try {
      handle = await pickOpenFileHandle(deps.windowLike);
    } catch (error) {
      // 교차 출처 서브프레임 등에서 showOpenFilePicker 자체가 거부되는 경우(SecurityError 등).
      deps.warn('[file:open] File System Access API 실패, 폴백:', error);
      nativeOpenPickerAvailable = false;
    }
  }

  if (!handle) {
    // Native picker가 있었다면 null은 사용자 취소(예: Esc)다. fallback을 열면 안 된다.
    if (nativeOpenPickerAvailable) return;
    const fileInput = deps.findFileInput();
    if (fileInput) {
      fileInput.dataset.skipUnsavedGuard = 'true';
      fileInput.click();
    }
    return;
  }

  try {
    const { bytes, name } = await readFileFromHandle(handle);
    deps.emitOpenDocument({
      bytes,
      fileName: name,
      fileHandle: handle,
      skipUnsavedGuard: true,
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.error('[file:open] 열기 실패:', message);
    deps.alert(`파일 열기에 실패했습니다:\n${message}`);
  }
}
