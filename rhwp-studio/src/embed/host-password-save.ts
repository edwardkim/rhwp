import {
  PASSWORD_PROTECTION,
  PasswordSaveError,
  validatePasswordProtectedExport,
} from '../../../npm/editor/password-save-contract.js';
import type {
  HostSaveAcknowledgment,
  HostSaveResult,
  PasswordProtectedDocumentExport,
  PasswordProtectedDocumentRequest,
  SaveProtectionState,
} from '../../../npm/editor/index.js';
import { exportPasswordProtectedDocumentWithReportForFormat } from '../command/save-document-format.ts';
import type { WasmBridge } from '../core/wasm-bridge.ts';

type Revision = Pick<SaveProtectionState, 'documentEpoch' | 'changeSeq'>;
type ExportReceipt = Pick<HostSaveAcknowledgment, keyof Revision | 'exportId' | 'format'>;

interface PasswordSaveDependencies {
  wasm: Pick<WasmBridge,
    'getLoadedDocumentFormat' | 'requiresPasswordForSave'
    | 'exportHwpWithPasswordAndReport' | 'exportHwpxWithPasswordAndReport'>;
  revision: () => Revision;
  completeHostSave: (fileName?: string) => Promise<HostSaveResult>;
}

export class HostPasswordSave {
  private pending: ExportReceipt | null = null;

  private readonly deps: PasswordSaveDependencies;

  constructor(deps: PasswordSaveDependencies) {
    this.deps = deps;
  }

  getState(): SaveProtectionState {
    const loadedFormat = this.deps.wasm.getLoadedDocumentFormat();
    if (loadedFormat === null) throw new PasswordSaveError('NO_DOCUMENT');
    if (loadedFormat !== 'hwp3' && loadedFormat !== 'hwp5' && loadedFormat !== 'hwpx') {
      throw new PasswordSaveError('UNSUPPORTED_SAVE_FORMAT');
    }
    const requiresPasswordForSave = this.deps.wasm.requiresPasswordForSave;
    return {
      schemaVersion: 1,
      persistedProtection: requiresPasswordForSave ? PASSWORD_PROTECTION : 'plain',
      requiresPasswordForSave,
      loadedFormat,
      ...this.deps.revision(),
    };
  }

  export(request: PasswordProtectedDocumentRequest): PasswordProtectedDocumentExport {
    const state = this.getState();
    this.checkRevision(state, request.expectedDocumentEpoch, request.expectedChangeSeq);
    const artifact = exportPasswordProtectedDocumentWithReportForFormat(
      this.deps.wasm, request.format, request.password,
    );
    this.checkRevision(this.deps.revision(), state.documentEpoch, state.changeSeq);

    const exportId = Array.from(crypto.getRandomValues(new Uint8Array(16)),
      byte => byte.toString(16).padStart(2, '0')).join('');
    const { documentEpoch, changeSeq } = state;
    const receipt = { exportId, format: request.format, documentEpoch, changeSeq };
    const result = validatePasswordProtectedExport({
      schemaVersion: 1,
      ...receipt,
      protection: PASSWORD_PROTECTION,
      bytes: artifact.bytes,
      byteCount: artifact.bytes.byteLength,
      contentLoss: artifact.contentLoss,
    }, request);
    this.pending = receipt;
    return result;
  }

  acknowledge(request: HostSaveAcknowledgment): Promise<HostSaveResult> {
    this.checkRevision(this.deps.revision(), request.documentEpoch, request.changeSeq);
    const pending = this.pending;
    if (!pending || pending.exportId !== request.exportId || pending.format !== request.format
      || pending.documentEpoch !== request.documentEpoch || pending.changeSeq !== request.changeSeq) {
      throw new PasswordSaveError('INVALID_SAVE_EXPORT');
    }

    // Consume the receipt and commit the state in the same turn as the revision check.
    this.pending = null;
    this.deps.wasm.requiresPasswordForSave = true;
    return this.deps.completeHostSave(request.fileName);
  }

  private checkRevision(revision: Revision, documentEpoch: number, changeSeq: number): void {
    if (revision.documentEpoch !== documentEpoch || revision.changeSeq !== changeSeq) {
      throw new PasswordSaveError('STALE_SAVE_REVISION');
    }
  }
}
