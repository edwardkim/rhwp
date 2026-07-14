import type { HmlOpenMetadata } from '../core/wasm-bridge';
import { showToast } from './toast';
import { buildHmlImportWarningMessage } from './hml-import-warning-message';

export { buildHmlImportWarningMessage } from './hml-import-warning-message';

export function showHmlImportWarning(metadata: HmlOpenMetadata): void {
  showToast({
    message: buildHmlImportWarningMessage(metadata),
    durationMs: 0,
    confirmLabel: '확인',
  });
}
