import type {
  HostSaveAcknowledgment,
  HostSaveResult,
  PasswordProtectedDocumentExport,
  PasswordProtectedDocumentRequest,
  SaveProtectionState,
} from './index.js';

export const PASSWORD_SAVE_CAPABILITY: 'password-save-v1';
export const PASSWORD_PROTECTION: 'password-protected';
export type PasswordSaveMethod = 'getSaveProtectionState' | 'exportPasswordProtectedDocument' | 'acknowledgeHostSave';

export class PasswordSaveError extends Error {
  constructor(code: unknown);
  readonly code: string;
}

export function isPasswordSaveMethod(method: unknown): method is PasswordSaveMethod;
export function validateSaveProtectionQuery(value: unknown): void;
export function validatePasswordExportRequest(value: unknown): PasswordProtectedDocumentRequest;
export function validateHostSaveRequest(value: unknown): HostSaveAcknowledgment;
export function validateSaveProtectionState(value: unknown): SaveProtectionState;
export function validatePasswordProtectedExport(
  value: unknown, request: Omit<PasswordProtectedDocumentRequest, 'password' | 'schemaVersion'>,
): PasswordProtectedDocumentExport;
export function validateHostSaveResult(value: unknown): HostSaveResult;
