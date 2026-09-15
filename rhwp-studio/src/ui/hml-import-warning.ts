import type { HmlOpenMetadata } from '../core/wasm-bridge';
import { showToast } from './toast';
import { buildHmlImportWarningMessage } from './hml-import-warning-message';

import { t } from '../i18n/index.ts';
export { buildHmlImportWarningMessage } from './hml-import-warning-message';

export function showHmlImportWarning(metadata: HmlOpenMetadata): void {
  showToast({
    message: buildHmlImportWarningMessage(metadata),
    durationMs: 0,
    confirmLabel: t('dialog.hmlImportWarning.message'),
  });
}
