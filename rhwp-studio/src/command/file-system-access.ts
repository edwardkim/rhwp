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

export function createHttpFileHandle(options: HttpFileHandleOptions): FileSystemFileHandleLike {
  const { fileName, fileUrl, saveUrl } = options;

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

          const response = await fetch(saveUrl, {
            method: 'PUT',
            body: pendingBlob,
            headers: {
              'content-type': pendingBlob.type || 'application/octet-stream',
            },
          });

          if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
          }
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
  const { blob, suggestedName, currentHandle, windowLike } = options;

  if (currentHandle) {
    await writeBlobToHandle(currentHandle, blob);
    return {
      method: 'current-handle',
      handle: currentHandle,
      fileName: currentHandle.name,
    };
  }

  if (windowLike.showSaveFilePicker) {
    const handle = await windowLike.showSaveFilePicker({
      suggestedName,
      types: HWP_PICKER_TYPES,
    });
    await writeBlobToHandle(handle, blob);
    return {
      method: 'save-picker',
      handle,
      fileName: handle.name,
    };
  }

  return {
    method: 'fallback',
    handle: null,
    fileName: suggestedName,
  };
}
