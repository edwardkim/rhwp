import type { FileSystemFileHandleLike } from './file-system-access';

export interface FileHandlingLaunchParamsLike {
  files?: FileSystemFileHandleLike[];
}

export interface LaunchQueueLike {
  setConsumer(consumer: (params: FileHandlingLaunchParamsLike) => void): void;
}

export interface FileHandlingWindowLike {
  launchQueue?: LaunchQueueLike;
}

export interface OpenDocumentBytesPayload {
  bytes: Uint8Array;
  fileName: string;
  fileHandle: FileSystemFileHandleLike;
  skipUnsavedGuard?: boolean;
}

export interface PwaFileHandlingCallbacks {
  openDocumentBytes(payload: OpenDocumentBytesPayload): void;
  notifyUnsupportedFile(fileName: string): void;
  notifyError(error: unknown): void;
  notifyMultipleFiles?(count: number): void;
}

function isSupportedLaunchDocumentFileName(fileName: string): boolean {
  return /\.(hwp|hwpx)$/i.test(fileName.trim());
}

async function readLaunchFileFromHandle(handle: FileSystemFileHandleLike): Promise<{
  name: string;
  bytes: Uint8Array;
}> {
  const file = await handle.getFile();
  return {
    name: file.name,
    bytes: new Uint8Array(await file.arrayBuffer()),
  };
}

export async function handlePwaLaunchFiles(
  params: FileHandlingLaunchParamsLike,
  callbacks: PwaFileHandlingCallbacks,
): Promise<void> {
  const handles = params.files ?? [];
  if (handles.length === 0) return;
  if (handles.length > 1) callbacks.notifyMultipleFiles?.(handles.length);

  const handle = handles[0];
  if (!isSupportedLaunchDocumentFileName(handle.name)) {
    callbacks.notifyUnsupportedFile(handle.name);
    return;
  }

  try {
    const { bytes, name } = await readLaunchFileFromHandle(handle);
    if (!isSupportedLaunchDocumentFileName(name)) {
      callbacks.notifyUnsupportedFile(name);
      return;
    }
    callbacks.openDocumentBytes({
      bytes,
      fileName: name,
      fileHandle: handle,
      skipUnsavedGuard: false,
    });
  } catch (error) {
    callbacks.notifyError(error);
  }
}

export function installPwaFileHandling(
  windowLike: FileHandlingWindowLike,
  callbacks: PwaFileHandlingCallbacks,
): boolean {
  if (!windowLike.launchQueue?.setConsumer) return false;

  windowLike.launchQueue.setConsumer((params) => {
    void handlePwaLaunchFiles(params, callbacks);
  });
  return true;
}
