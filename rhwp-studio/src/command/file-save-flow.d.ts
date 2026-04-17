import type {
  FileSystemFileHandleLike,
  FileSystemWindowLike,
  SaveTraceEvent,
} from './file-system-access';

export interface PerformDocumentSaveOptions {
  bytes: Uint8Array;
  saveName: string;
  currentHandle: FileSystemFileHandleLike | null;
  isNewDocument: boolean;
  windowLike: FileSystemWindowLike;
  promptForName?: (baseName: string) => Promise<string | null>;
  onDownload: (blob: Blob, fileName: string) => void;
  onStatus?: (message: string) => void;
  onAlert?: (message: string) => void;
  onTrace?: (event: SaveTraceEvent) => void;
}

export type PerformDocumentSaveResult =
  | {
    ok: true;
    method: 'current-handle' | 'save-picker' | 'fallback';
    fileName: string;
    handle: FileSystemFileHandleLike | null;
  }
  | {
    ok: false;
    reason: 'aborted' | 'existing-save-failed';
    error?: unknown;
  };

export function performDocumentSave(options: PerformDocumentSaveOptions): Promise<PerformDocumentSaveResult>;
