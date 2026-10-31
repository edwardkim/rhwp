import type { AutosaveDraft } from './autosave-store.ts';

// Web Crypto AES-GCM/PBKDF2; OWASP's PBKDF2-HMAC-SHA256 work factor.
export const RECOVERY_KDF_ITERATIONS = 600_000;
const MAX_BYTES = 64 * 1024 * 1024;
export interface RecoveryEnvelope {
  version: 1;
  scheme: 'PBKDF2-SHA256/AES-256-GCM';
  iterations: number;
  salt: Uint8Array;
  iv: Uint8Array;
  epoch: string;
  revision: number;
}

function failure(): Error { return new Error('RECOVERY_AUTH_FAILED: 암호가 다르거나 복구본이 변경되었습니다.'); }
function buffer(bytes: Uint8Array): ArrayBuffer { return new Uint8Array(bytes).buffer; }
function validEnvelope(value: RecoveryEnvelope): void {
  if (!value || value.version !== 1 || value.scheme !== 'PBKDF2-SHA256/AES-256-GCM'
    || value.iterations !== RECOVERY_KDF_ITERATIONS || !(value.salt instanceof Uint8Array)
    || value.salt.length !== 16 || !(value.iv instanceof Uint8Array) || value.iv.length !== 12
    || typeof value.epoch !== 'string' || !/^[0-9a-f-]{36}$/i.test(value.epoch)
    || !Number.isSafeInteger(value.revision) || value.revision < 1) throw failure();
}
function aad(draft: AutosaveDraft, envelope: RecoveryEnvelope): ArrayBuffer {
  if (![draft.id, draft.fileName, draft.sourceFormat, draft.dirtyReason ?? ''].every(
    value => typeof value === 'string' && value.length <= 4096)
    || !Number.isSafeInteger(draft.savedAt) || draft.savedAt < 0) throw failure();
  return buffer(new TextEncoder().encode(JSON.stringify([
    'rhwp-protected-recovery', envelope.version, envelope.scheme, envelope.iterations,
    Array.from(envelope.salt), Array.from(envelope.iv), envelope.epoch, envelope.revision,
    draft.id, draft.fileName, draft.sourceFormat, draft.savedAt, draft.dirtyReason ?? '',
  ])));
}

async function derive(password: string, salt: Uint8Array): Promise<CryptoKey> {
  if (!crypto?.subtle || typeof password !== 'string' || !password || password.length > 4096) throw failure();
  const encoded = new TextEncoder().encode(password);
  try {
    const material = await crypto.subtle.importKey('raw', buffer(encoded), 'PBKDF2', false, ['deriveKey']);
    return await crypto.subtle.deriveKey({ name: 'PBKDF2', salt: buffer(salt),
      iterations: RECOVERY_KDF_ITERATIONS, hash: 'SHA-256' }, material,
    { name: 'AES-GCM', length: 256 }, false, ['encrypt', 'decrypt']);
  } finally { encoded.fill(0); }
}

/** Session-only, non-extractable key. Passwords and keys never enter the draft. */
export class RecoveryCipher {
  #key: CryptoKey | null;
  #salt: Uint8Array;
  #epoch: string;
  #revision = 0;
  private constructor(key: CryptoKey, salt: Uint8Array, epoch: string, revision = 0) {
    this.#key = key; this.#salt = new Uint8Array(salt); this.#epoch = epoch; this.#revision = revision;
  }
  static async create(password: string): Promise<RecoveryCipher> {
    const salt = crypto.getRandomValues(new Uint8Array(16));
    return new RecoveryCipher(await derive(password, salt), salt, crypto.randomUUID());
  }
  dispose(): void { this.#key = null; this.#salt.fill(0); }
  async protect(draft: AutosaveDraft): Promise<AutosaveDraft> {
    if (!this.#key || draft.data.byteLength > MAX_BYTES) throw failure();
    const key = this.#key;
    const protection: RecoveryEnvelope = { version: 1, scheme: 'PBKDF2-SHA256/AES-256-GCM',
      iterations: RECOVERY_KDF_ITERATIONS, salt: new Uint8Array(this.#salt),
      iv: crypto.getRandomValues(new Uint8Array(12)), epoch: this.#epoch, revision: ++this.#revision };
    validEnvelope(protection);
    const data = new Uint8Array(await crypto.subtle.encrypt({ name: 'AES-GCM', iv: buffer(protection.iv),
      additionalData: aad(draft, protection), tagLength: 128 }, key, buffer(draft.data)));
    if (this.#key !== key) throw failure();
    return { ...draft, data, byteLength: data.byteLength, protection };
  }
  static async unlock(draft: AutosaveDraft, password: string): Promise<{ data: Uint8Array; cipher: RecoveryCipher }> {
    const envelope = draft.protection;
    if (!envelope || draft.data.byteLength < 16 || draft.data.byteLength > MAX_BYTES + 16) throw failure();
    validEnvelope(envelope);
    let cipher: RecoveryCipher | null = null;
    try {
      const key = await derive(password, envelope.salt);
      const data = new Uint8Array(await crypto.subtle.decrypt({ name: 'AES-GCM', iv: buffer(envelope.iv),
        additionalData: aad(draft, envelope), tagLength: 128 }, key, buffer(draft.data)));
      cipher = new RecoveryCipher(key, envelope.salt, envelope.epoch, envelope.revision);
      return { data, cipher };
    } catch { cipher?.dispose(); throw failure(); }
  }
}

/** Detect deletion/replacement while a password dialog or KDF was pending. */
export function sameRecoveryDraft(a: AutosaveDraft, b: AutosaveDraft | null): boolean {
  return b !== null && a.id === b.id && a.savedAt === b.savedAt && a.fileName === b.fileName
    && a.sourceFormat === b.sourceFormat && a.dirtyReason === b.dirtyReason
    && JSON.stringify(a.protection) === JSON.stringify(b.protection)
    && a.data.length === b.data.length && a.data.every((byte, index) => byte === b.data[index]);
}
