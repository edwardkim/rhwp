export interface FileSystemWritableFileStreamLike {
  write(data: Blob): Promise<void>;
  close(): Promise<void>;
}

export interface FileSystemFileHandleLike {
  kind?: 'file';
  name: string;
  getFile(): Promise<File>;
  createWritable(): Promise<FileSystemWritableFileStreamLike>;
}

export interface FileSystemWindowLike {
  showOpenFilePicker?: (options?: {
    excludeAcceptAllOption?: boolean;
    multiple?: boolean;
    types?: { description: string; accept: Record<string, string[]> }[];
  }) => Promise<FileSystemFileHandleLike[]>;
  showSaveFilePicker?: (options?: {
    suggestedName?: string;
    types?: { description: string; accept: Record<string, string[]> }[];
  }) => Promise<FileSystemFileHandleLike>;
  launchQueue?: LaunchQueueLike;
}

export interface LaunchParamsLike {
  files?: FileSystemFileHandleLike[];
}

export interface LaunchQueueLike {
  setConsumer(consumer: (launchParams: LaunchParamsLike) => Promise<void> | void): void;
}

export interface FileHandleReadResult {
  name: string;
  bytes: Uint8Array;
}

export interface SaveDocumentOptions {
  blob: Blob;
  suggestedName: string;
  currentHandle: FileSystemFileHandleLike | null;
  windowLike: FileSystemWindowLike;
  onTrace?: (event: SaveTraceEvent) => void;
}

export interface SaveDocumentResult {
  method: 'current-handle' | 'save-picker' | 'fallback';
  handle: FileSystemFileHandleLike | null;
  fileName: string;
}

export interface PwaLaunchPayload {
  bytes: Uint8Array;
  fileName: string;
  fileHandle: FileSystemFileHandleLike;
}

export interface HttpFileHandleOptions {
  fileName: string;
  fileUrl: string;
  saveUrl: string;
  onTrace?: (event: SaveTraceEvent) => void;
}

export interface SaveTraceEvent {
  stage:
  | 'save-start'
  | 'save-success'
  | 'save-error'
  | 'save-fallback'
  | 'http-save-request'
  | 'http-save-success'
  | 'http-save-error';
  method?: 'current-handle' | 'save-picker' | 'fallback';
  fileName: string;
  handleName?: string;
  token?: string;
  fileUrl?: string;
  saveUrl?: string;
  error?: string;
}

const HWP_PICKER_TYPES = [{
  description: 'HWP 문서',
  accept: { 'application/x-hwp': ['.hwp', '.hwpx'] },
}];

function isAbortError(error: unknown): boolean {
  return error instanceof DOMException && error.name === 'AbortError';
}

async function writeBlobToHandle(handle: FileSystemFileHandleLike, blob: Blob): Promise<void> {
  const writable = await handle.createWritable();
  await writable.write(blob);
  await writable.close();
}

function extractTokenFromSaveUrl(saveUrl: string): string | undefined {
  try {
    const url = new URL(saveUrl);
    const token = url.pathname.split('/').pop() ?? '';
    return token || undefined;
  } catch {
    return undefined;
  }
}

export function createHttpFileHandle(options: HttpFileHandleOptions): FileSystemFileHandleLike {
  const {
    fileName,
    fileUrl,
    saveUrl,
    onTrace,
  } = options;
  const token = extractTokenFromSaveUrl(saveUrl);

  return {
    kind: 'file',
    name: fileName,
    async getFile() {
      const response = await fetch(fileUrl);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const blob = await response.blob();
      return new File([blob], fileName, {
        type: blob.type || 'application/x-hwp',
      });
    },
    async createWritable() {
      let pendingBlob: Blob | null = null;

      return {
        async write(data: Blob) {
          pendingBlob = data;
        },
        async close() {
          if (!pendingBlob) return;

          onTrace?.({
            stage: 'http-save-request',
            fileName,
            token,
            fileUrl,
            saveUrl,
          });

          const response = await fetch(saveUrl, {
            method: 'PUT',
            body: pendingBlob,
            headers: {
              'content-type': pendingBlob.type || 'application/octet-stream',
            },
          });

          if (!response.ok) {
            onTrace?.({
              stage: 'http-save-error',
              fileName,
              token,
              fileUrl,
              saveUrl,
              error: `HTTP ${response.status}: ${response.statusText}`,
            });
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
          }

          onTrace?.({
            stage: 'http-save-success',
            fileName,
            token,
            fileUrl,
            saveUrl,
          });
        },
      };
    },
  };
}

export async function pickOpenFileHandle(windowLike: FileSystemWindowLike): Promise<FileSystemFileHandleLike | null> {
  if (!windowLike.showOpenFilePicker) return null;

  try {
    const handles = await windowLike.showOpenFilePicker({
      excludeAcceptAllOption: true,
      multiple: false,
      types: HWP_PICKER_TYPES,
    });
    return handles[0] ?? null;
  } catch (error) {
    if (isAbortError(error)) return null;
    throw error;
  }
}

export async function readFileFromHandle(handle: FileSystemFileHandleLike): Promise<FileHandleReadResult> {
  const file = await handle.getFile();
  return {
    name: file.name,
    bytes: new Uint8Array(await file.arrayBuffer()),
  };
}

export function setupPwaFileLaunch(
  windowLike: FileSystemWindowLike,
  onLaunch: (payload: PwaLaunchPayload) => Promise<void> | void,
  onError?: (error: unknown) => void,
): boolean {
  if (!windowLike.launchQueue) return false;

  windowLike.launchQueue.setConsumer(async (launchParams) => {
    const handle = launchParams.files?.[0];
    if (!handle) return;

    try {
      const { bytes, name } = await readFileFromHandle(handle);
      await onLaunch({
        bytes,
        fileName: name,
        fileHandle: handle,
      });
    } catch (error) {
      onError?.(error);
    }
  });

  return true;
}

export async function saveDocumentToFileSystem(options: SaveDocumentOptions): Promise<SaveDocumentResult> {
  const {
    blob,
    suggestedName,
    currentHandle,
    windowLike,
    onTrace,
  } = options;

  if (currentHandle) {
    onTrace?.({
      stage: 'save-start',
      method: 'current-handle',
      fileName: suggestedName,
      handleName: currentHandle.name,
    });

    try {
      await writeBlobToHandle(currentHandle, blob);
      onTrace?.({
        stage: 'save-success',
        method: 'current-handle',
        fileName: currentHandle.name,
        handleName: currentHandle.name,
      });
      return {
        method: 'current-handle',
        handle: currentHandle,
        fileName: currentHandle.name,
      };
    } catch (error) {
      onTrace?.({
        stage: 'save-error',
        method: 'current-handle',
        fileName: suggestedName,
        handleName: currentHandle.name,
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  if (windowLike.showSaveFilePicker) {
    onTrace?.({
      stage: 'save-start',
      method: 'save-picker',
      fileName: suggestedName,
    });
    const handle = await windowLike.showSaveFilePicker({
      suggestedName,
      types: HWP_PICKER_TYPES,
    });
    await writeBlobToHandle(handle, blob);
    onTrace?.({
      stage: 'save-success',
      method: 'save-picker',
      fileName: handle.name,
      handleName: handle.name,
    });
    return {
      method: 'save-picker',
      handle,
      fileName: handle.name,
    };
  }

  onTrace?.({
    stage: 'save-fallback',
    method: 'fallback',
    fileName: suggestedName,
  });
  return {
    method: 'fallback',
    handle: null,
    fileName: suggestedName,
  };
}
