/** Versioned Studio transport shared by native shells. Binary messages use bounded base64 chunks. */
const VERSION = 1;
const CHUNK_BYTES = 768 * 1024;
const MAX_DOCUMENT_BYTES = 128 * 1024 * 1024;

export type DesktopFormat = 'hwp' | 'hwpx';
export type DesktopTheme = 'light' | 'dark' | 'highContrast';
type FileCommand = 'open' | 'save' | 'saveAs';
type FileResult = 'saved' | 'completed' | 'cancelled' | 'failed';
type MessageId = number | string;

export interface DesktopFileCommands {
  request(command: FileCommand, format?: DesktopFormat, suggestedName?: string): Promise<FileResult>;
}

export interface DesktopTransport {
  version: number;
  postMessage(message: object): void;
  subscribe(listener: (message: unknown) => void): () => void;
}

export interface DesktopRevision {
  documentEpoch: number;
  changeSeq: number;
}

export interface DesktopState extends DesktopRevision {
  hasDocument: boolean;
  fileName: string;
  format: DesktopFormat | null;
  dirty: boolean;
  canNativeSave: boolean;
}

export interface DesktopEditor {
  state(): DesktopState;
  load(bytes: Uint8Array, fileName: string, validate: () => void): Promise<void>;
  export(format: DesktopFormat): Uint8Array;
  saved(fileName: string, format: DesktopFormat): Promise<void>;
  discard(): Promise<void>;
  theme(theme: DesktopTheme): void;
  lock(): () => void;
}

interface IncomingDocument {
  bytes: Uint8Array;
  fileName: string;
  offset: number;
  revision: DesktopRevision;
  unlock: () => void;
}

interface OutgoingDocument {
  bytes: Uint8Array;
  format: DesktopFormat;
  revision: DesktopRevision;
  unlock: () => void;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function isMessageId(value: unknown): value is MessageId {
  return typeof value === 'string' || isInteger(value);
}

function isInteger(value: unknown): value is number {
  return typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;
}

function readInteger(value: unknown, maximum: number, label: string): number {
  if (!isInteger(value) || value > maximum) throw new Error(`잘못된 ${label} 값입니다.`);
  return value;
}

function readFileName(value: unknown): string {
  if (
    typeof value !== 'string' || value.trim().length === 0 || value.length > 255
    || /[\\/\u0000-\u001f]/u.test(value)
  ) throw new Error('잘못된 문서 이름입니다.');
  return value;
}

function readRevision(params: Record<string, unknown>): DesktopRevision {
  return {
    documentEpoch: readInteger(params.documentEpoch, Number.MAX_SAFE_INTEGER, '문서 세대'),
    changeSeq: readInteger(params.changeSeq, Number.MAX_SAFE_INTEGER, '문서 변경'),
  };
}

function sameRevision(left: DesktopRevision, right: DesktopRevision): boolean {
  return left.documentEpoch === right.documentEpoch && left.changeSeq === right.changeSeq;
}

function encodeBase64(bytes: Uint8Array): string {
  let binary = '';
  for (let offset = 0; offset < bytes.length; offset += 8192) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + 8192));
  }
  return btoa(binary);
}

function decodeBase64(value: unknown): string {
  if (typeof value !== 'string' || value.length > 4 * Math.ceil(CHUNK_BYTES / 3) || value.length % 4 !== 0) {
    throw new Error('잘못된 문서 데이터 조각입니다.');
  }
  return atob(value);
}

export function isDesktopFormat(format: unknown): format is DesktopFormat {
  return format === 'hwp' || format === 'hwpx';
}

export function getDesktopTransport(window: Window): DesktopTransport | undefined {
  const transport = (window as Window & { rhwpDesktopHost?: DesktopTransport }).rhwpDesktopHost;
  if (transport?.version === VERSION && typeof transport.postMessage === 'function'
    && typeof transport.subscribe === 'function') return transport;
}

export function openDesktopWebLink(window: Window, url: string): boolean {
  if (!getDesktopTransport(window)) return false;
  window.open(url, '_blank', 'noopener');
  return true;
}

export class DesktopHost implements DesktopFileCommands {
  private nextCommandId = 0;
  private commands = new Map<number, (result: FileResult) => void>();
  private pendingLoad: IncomingDocument | undefined;
  private pendingExport: OutgoingDocument | undefined;
  private committing = false;
  private lastState = '';
  private ready = false;
  private transport: DesktopTransport;
  private editor: DesktopEditor;

  constructor(transport: DesktopTransport, editor: DesktopEditor) {
    this.transport = transport;
    this.editor = editor;
    transport.subscribe(message => void this.receive(message));
  }

  notifyReady(): void {
    this.ready = true;
    this.postState('ready');
  }

  notifyFailed(): void {
    this.transport.postMessage({ version: VERSION, kind: 'event', event: 'failed' });
  }

  notifyChanged(): void {
    if (this.ready) this.postState('changed');
  }

  request(command: FileCommand, format?: DesktopFormat, suggestedName?: string): Promise<FileResult> {
    const id = ++this.nextCommandId;
    return new Promise(resolve => {
      this.commands.set(id, resolve);
      try {
        this.transport.postMessage({ version: VERSION, kind: 'command', id, command, format, suggestedName });
      } catch (error) {
        this.commands.delete(id);
        console.error('[desktop] File command failed', error);
        resolve('failed');
      }
    });
  }

  private postState(event: 'ready' | 'changed'): void {
    const value = this.editor.state();
    const snapshot = JSON.stringify(value);
    if (event === 'changed' && snapshot === this.lastState) return;
    this.lastState = snapshot;
    this.transport.postMessage({ version: VERSION, kind: 'event', event, value });
  }

  private async receive(message: unknown): Promise<void> {
    if (!isRecord(message) || message.version !== VERSION || !isMessageId(message.id)) return;
    const { id } = message;
    if (message.kind === 'commandResult' && typeof id === 'number') {
      const resolve = this.commands.get(id);
      if (!resolve) return;
      this.commands.delete(id);
      const result = message.ok === true ? message.result : 'failed';
      resolve(result === 'saved' || result === 'completed' || result === 'cancelled' ? result : 'failed');
      return;
    }
    if (message.kind !== 'request' || typeof message.method !== 'string' || !isRecord(message.params)) return;
    try {
      const result = await this.handle(message.method, message.params);
      this.transport.postMessage({ version: VERSION, kind: 'response', id, ok: true, result });
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      this.transport.postMessage({ version: VERSION, kind: 'response', id, ok: false, error: detail });
    }
  }

  private assertRevision(revision: DesktopRevision): void {
    if (!sameRevision(this.editor.state(), revision)) {
      throw new Error('파일 작업 중 문서가 변경되었습니다. 다시 시도해 주세요.');
    }
  }

  private assertIdle(): void {
    if (this.pendingLoad || this.pendingExport || this.committing) {
      throw new Error('다른 파일 작업이 진행 중입니다.');
    }
  }

  private clearTransfers(): void {
    this.pendingLoad?.unlock();
    this.pendingExport?.unlock();
    this.pendingLoad = undefined;
    this.pendingExport = undefined;
  }

  private beginLoad(params: Record<string, unknown>): object {
    this.assertIdle();
    const revision = params.documentEpoch === undefined && params.changeSeq === undefined
      ? this.editor.state() : readRevision(params);
    this.assertRevision(revision);
    const fileName = readFileName(params.fileName);
    const byteLength = readInteger(params.byteLength, MAX_DOCUMENT_BYTES, '문서 크기');
    if (byteLength === 0) throw new Error('빈 파일은 열 수 없습니다.');
    const bytes = new Uint8Array(byteLength);
    this.pendingLoad = { bytes, fileName, offset: 0, revision, unlock: this.editor.lock() };
    return {};
  }

  private loadChunk(params: Record<string, unknown>): object {
    const load = this.pendingLoad;
    if (!load || params.offset !== load.offset) throw new Error('문서 데이터 조각의 순서가 잘못되었습니다.');
    const binary = decodeBase64(params.bytes);
    if (binary.length === 0 || binary.length > CHUNK_BYTES || binary.length > load.bytes.length - load.offset) {
      throw new Error('문서 데이터 조각의 크기가 잘못되었습니다.');
    }
    for (let i = 0; i < binary.length; i++) load.bytes[load.offset + i] = binary.charCodeAt(i);
    load.offset += binary.length;
    return {};
  }

  private async commitLoad(): Promise<DesktopState> {
    const load = this.pendingLoad;
    if (!load || load.offset !== load.bytes.length) throw new Error('문서 데이터가 모두 도착하지 않았습니다.');
    this.assertRevision(load.revision);
    this.pendingLoad = undefined;
    this.committing = true;
    load.unlock();
    try {
      await this.editor.load(load.bytes, load.fileName, () => this.assertRevision(load.revision));
      return this.editor.state();
    } finally {
      this.committing = false;
      this.notifyChanged();
    }
  }

  private exportDocument(format: unknown): DesktopRevision & { byteLength: number; format: DesktopFormat } {
    this.assertIdle();
    if (!isDesktopFormat(format)) throw new Error('지원하지 않는 문서 저장 형식입니다.');
    if (!this.editor.state().canNativeSave) throw new Error('스튜디오 파일 메뉴에서 이 문서를 저장해 주세요.');
    const unlock = this.editor.lock();
    try {
      const bytes = this.editor.export(format);
      if (bytes.length === 0 || bytes.length > MAX_DOCUMENT_BYTES) {
        throw new Error('내보낸 문서 크기가 지원 범위를 벗어났습니다.');
      }
      const { documentEpoch, changeSeq } = this.editor.state();
      const revision = { documentEpoch, changeSeq };
      this.pendingExport = { bytes, format, revision, unlock };
      return { ...revision, byteLength: bytes.length, format };
    } catch (error) {
      unlock();
      throw error;
    }
  }

  private exportChunk(params: Record<string, unknown>): { bytes: string } {
    const exported = this.pendingExport;
    if (!exported) throw new Error('내보낼 문서가 없습니다.');
    const offset = readInteger(params.offset, exported.bytes.length, '데이터 시작 위치');
    const count = readInteger(params.count, CHUNK_BYTES, '데이터 조각 크기');
    if (count === 0 || offset + count > exported.bytes.length) throw new Error('문서 데이터 범위가 잘못되었습니다.');
    this.assertRevision(exported.revision);
    return { bytes: encodeBase64(exported.bytes.subarray(offset, offset + count)) };
  }

  private async finishSave(params: Record<string, unknown>): Promise<DesktopState> {
    const exported = this.pendingExport;
    const revision = readRevision(params);
    if (!exported || !sameRevision(exported.revision, revision)) throw new Error('저장할 문서가 바뀌었습니다.');
    this.assertRevision(revision);
    const fileName = readFileName(params.fileName);
    this.pendingExport = undefined;
    this.committing = true;
    try {
      await this.editor.saved(fileName, exported.format);
      return this.editor.state();
    } finally {
      this.committing = false;
      exported.unlock();
      this.notifyChanged();
    }
  }

  private async discard(params: Record<string, unknown>): Promise<DesktopState> {
    this.assertIdle();
    const revision = readRevision(params);
    this.assertRevision(revision);
    const unlock = this.editor.lock();
    this.committing = true;
    try {
      this.assertRevision(revision);
      await this.editor.discard();
      return this.editor.state();
    } finally {
      this.committing = false;
      unlock();
      this.notifyChanged();
    }
  }

  private handle(method: string, params: Record<string, unknown>): object | Promise<object> {
    switch (method) {
      case 'load.begin': return this.beginLoad(params);
      case 'load.chunk': return this.loadChunk(params);
      case 'load.commit': return this.commitLoad();
      case 'export': return this.exportDocument(params.format);
      case 'export.chunk': return this.exportChunk(params);
      case 'saved': return this.finishSave(params);
      case 'discard': return this.discard(params);
      case 'release': this.clearTransfers(); return {};
      case 'state': return this.editor.state();
      case 'theme': {
        const { theme } = params;
        if (theme !== 'light' && theme !== 'dark' && theme !== 'highContrast') {
          throw new Error('지원하지 않는 화면 테마입니다.');
        }
        this.editor.theme(theme);
        return {};
      }
      default: throw new Error('지원하지 않는 데스크톱 요청입니다.');
    }
  }
}
