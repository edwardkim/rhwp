/**
 * API 키 보관 — 키를 만지는 유일한 모듈.
 *
 * 세 모드:
 * - 세션(기본): 이 모듈의 메모리에만 둔다. 새로고침하면 사라진다.
 * - 기억: 브라우저 저장소(IndexedDB)에 평문으로 둔다.
 * - 기억 + 암호: PBKDF2-SHA-256 으로 만든 키로 AES-GCM 암호화해 둔다. 쓸 때마다 암호로 연다.
 *
 * 브라우저에 둔 키는 같은 출처의 스크립트(XSS)가 읽을 수 있다. 암호 모드는 저장소 유출만
 * 막는다 — 잠금을 푼 뒤의 메모리까지 지키지는 못한다. 설정 화면이 이 한계를 알린다.
 *
 * 키는 로그·오류 메시지·전역 객체에 싣지 않는다.
 */
import type { AiProviderId } from '../provider/models.ts';

export const PBKDF2_ITERATIONS = 310_000;
const MAX_KEY_LENGTH = 512;

export type StoredKey =
  | { provider: AiProviderId; mode: 'plain'; key: string }
  | {
    provider: AiProviderId;
    mode: 'encrypted';
    ciphertext: string;
    salt: string;
    iv: string;
    iterations: number;
  };

/** 저장소 추상. 브라우저는 IndexedDB, 테스트는 메모리를 쓴다. */
export interface KeyBackend {
  get(provider: AiProviderId): Promise<StoredKey | null>;
  put(record: StoredKey): Promise<void>;
  delete(provider: AiProviderId): Promise<void>;
}

export type KeyStatus =
  /** 키가 없다 */
  | 'none'
  /** 쓸 수 있다(세션 메모리 또는 잠금 해제됨) */
  | 'ready'
  /** 암호로 잠긴 키가 저장돼 있다 */
  | 'locked';

export type KeyStoreErrorCode =
  | 'INVALID_KEY'
  | 'WRONG_PASSPHRASE'
  | 'EMPTY_PASSPHRASE'
  /** 암호화 API(`crypto.subtle`)가 없다 — https·localhost 가 아닌 http 출처에서 브라우저가 주지 않는다. */
  | 'CRYPTO_UNAVAILABLE';

export class KeyStoreError extends Error {
  readonly code: KeyStoreErrorCode;
  constructor(code: KeyStoreErrorCode, message: string) {
    super(message);
    this.name = 'KeyStoreError';
    this.code = code;
  }
}

export interface SetKeyResult {
  /** 요청대로 저장소에 남겼는가. 저장소가 막혔으면 `false` — 키는 세션 동안만 쓴다. */
  persisted: boolean;
}

/** 키 앞 7자와 끝 4자만 보인다. 짧은 키는 통째로 가린다. */
export function maskKey(key: string): string {
  if (key.length <= 16) return '••••';
  return `${key.slice(0, 7)}…${key.slice(-4)}`;
}

/** 붙여 넣을 때 따라온 앞뒤 공백은 지우고, 안쪽 공백이나 제어 문자가 있으면 거절한다. */
export function normalizeKey(raw: string): string {
  const key = raw.trim();
  if (key.length === 0 || key.length > MAX_KEY_LENGTH || /[\s\u0000-\u001f\u007f]/u.test(key)) {
    throw new KeyStoreError('INVALID_KEY', 'API 키 형식이 올바르지 않습니다.');
  }
  return key;
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

function toBase64(bytes: Uint8Array): string {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function fromBase64(text: string): Uint8Array<ArrayBuffer> {
  const binary = atob(text);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

export class KeyStore {
  private readonly session = new Map<AiProviderId, string>();
  private readonly backend: KeyBackend | null;
  private readonly subtle: SubtleCrypto | null;
  private readonly iterations: number;

  /**
   * @param backend 저장소. `null` 이면 세션 모드만 가능하다.
   * @param subtle 암호화 API. 비보안 출처(http://IP 등)에서는 브라우저가 주지 않아 `null` 이다 —
   *   그때는 암호 모드만 막고 세션·기억 모드는 그대로 쓴다.
   * @param iterations PBKDF2 반복 수. 테스트만 낮춘다.
   */
  constructor(
    backend: KeyBackend | null,
    subtle: SubtleCrypto | null = globalThis.crypto?.subtle ?? null,
    iterations: number = PBKDF2_ITERATIONS,
  ) {
    this.backend = backend;
    this.subtle = subtle;
    this.iterations = iterations;
  }

  get canPersist(): boolean {
    return this.backend !== null;
  }

  /** 암호로 잠근 저장을 쓸 수 있는가 */
  get canEncrypt(): boolean {
    return this.subtle !== null;
  }

  /** 지금 쓸 수 있는 키. 없거나 잠겨 있으면 `null`. */
  getKey(provider: AiProviderId): string | null {
    return this.session.get(provider) ?? null;
  }

  /**
   * 저장소의 평문 키는 세션으로 올리고, 상태를 돌려준다.
   * 앱 시작 때 한 번, 그리고 설정 화면을 열 때 부른다.
   */
  async status(provider: AiProviderId): Promise<KeyStatus> {
    if (this.session.has(provider)) return 'ready';
    const stored = await this.readStored(provider);
    if (!stored) return 'none';
    if (stored.mode === 'plain') {
      this.session.set(provider, stored.key);
      return 'ready';
    }
    return 'locked';
  }

  async setKey(
    provider: AiProviderId,
    rawKey: string,
    options: { remember: boolean; passphrase?: string },
  ): Promise<SetKeyResult> {
    const key = normalizeKey(rawKey);
    if (options.remember && options.passphrase !== undefined && options.passphrase.length === 0) {
      throw new KeyStoreError('EMPTY_PASSPHRASE', '암호를 입력하세요.');
    }
    // 아무것도 바꾸기 전에 거절한다 — 세션 키만 바뀌고 저장은 실패한 어중간한 상태를 남기지 않는다.
    if (options.remember && options.passphrase && !this.subtle) {
      throw new KeyStoreError('CRYPTO_UNAVAILABLE', '이 연결에서는 암호 보호를 쓸 수 없습니다.');
    }
    this.session.set(provider, key);

    if (!options.remember) {
      await this.deleteStored(provider);
      return { persisted: false };
    }
    if (!this.backend) return { persisted: false };
    const record: StoredKey = options.passphrase
      ? { provider, mode: 'encrypted', ...await this.encrypt(key, options.passphrase) }
      : { provider, mode: 'plain', key };
    try {
      await this.backend.put(record);
      return { persisted: true };
    } catch {
      return { persisted: false };
    }
  }

  /** 저장 방식만 바꾼다(키는 그대로). 세션에 키가 있어야 한다. */
  async setRemember(
    provider: AiProviderId,
    options: { remember: boolean; passphrase?: string },
  ): Promise<SetKeyResult> {
    const key = this.session.get(provider);
    if (!key) throw new KeyStoreError('INVALID_KEY', '먼저 API 키를 입력하세요.');
    return this.setKey(provider, key, options);
  }

  async unlock(provider: AiProviderId, passphrase: string): Promise<void> {
    if (passphrase.length === 0) throw new KeyStoreError('EMPTY_PASSPHRASE', '암호를 입력하세요.');
    const stored = await this.readStored(provider);
    if (!stored) throw new KeyStoreError('INVALID_KEY', '저장된 API 키가 없습니다.');
    if (stored.mode === 'plain') {
      this.session.set(provider, stored.key);
      return;
    }
    if (!this.subtle) {
      throw new KeyStoreError('CRYPTO_UNAVAILABLE', '이 연결에서는 암호로 잠근 키를 열 수 없습니다.');
    }
    const key = await this.decrypt(stored, passphrase);
    this.session.set(provider, key);
  }

  /** 메모리와 저장소의 키를 모두 지운다. */
  async clear(provider: AiProviderId): Promise<void> {
    this.session.delete(provider);
    await this.deleteStored(provider);
  }

  private async readStored(provider: AiProviderId): Promise<StoredKey | null> {
    if (!this.backend) return null;
    try { return await this.backend.get(provider); } catch { return null; }
  }

  private async deleteStored(provider: AiProviderId): Promise<void> {
    if (!this.backend) return;
    try { await this.backend.delete(provider); } catch { /* 저장소가 막혔으면 남은 것도 없다 */ }
  }

  private requireSubtle(): SubtleCrypto {
    if (!this.subtle) throw new KeyStoreError('CRYPTO_UNAVAILABLE', '이 연결에서는 암호 보호를 쓸 수 없습니다.');
    return this.subtle;
  }

  private async deriveKey(passphrase: string, salt: Uint8Array<ArrayBuffer>, iterations: number): Promise<CryptoKey> {
    const subtle = this.requireSubtle();
    const material = await subtle.importKey('raw', encoder.encode(passphrase), 'PBKDF2', false, ['deriveKey']);
    return subtle.deriveKey(
      { name: 'PBKDF2', hash: 'SHA-256', salt, iterations },
      material,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt'],
    );
  }

  private async encrypt(key: string, passphrase: string) {
    const salt = globalThis.crypto.getRandomValues(new Uint8Array(16));
    const iv = globalThis.crypto.getRandomValues(new Uint8Array(12));
    const aes = await this.deriveKey(passphrase, salt, this.iterations);
    const ciphertext = new Uint8Array(await this.requireSubtle().encrypt({ name: 'AES-GCM', iv }, aes, encoder.encode(key)));
    return {
      ciphertext: toBase64(ciphertext),
      salt: toBase64(salt),
      iv: toBase64(iv),
      iterations: this.iterations,
    };
  }

  private async decrypt(stored: Extract<StoredKey, { mode: 'encrypted' }>, passphrase: string): Promise<string> {
    try {
      const aes = await this.deriveKey(passphrase, fromBase64(stored.salt), stored.iterations);
      const plain = await this.requireSubtle().decrypt(
        { name: 'AES-GCM', iv: fromBase64(stored.iv) }, aes, fromBase64(stored.ciphertext),
      );
      return decoder.decode(plain);
    } catch {
      // AES-GCM 인증 실패 = 암호가 틀렸거나 저장값이 손상됐다. 둘을 구분할 수 없다.
      throw new KeyStoreError('WRONG_PASSPHRASE', '암호가 맞지 않습니다.');
    }
  }
}

/** 메모리 저장소 — 테스트 전용. IndexedDB 를 못 쓰는 환경은 backend `null` 로 세션 모드가 된다. */
export function createMemoryKeyBackend(): KeyBackend & { records: Map<AiProviderId, StoredKey> } {
  const records = new Map<AiProviderId, StoredKey>();
  return {
    records,
    async get(provider) { return records.get(provider) ?? null; },
    async put(record) { records.set(record.provider, record); },
    async delete(provider) { records.delete(provider); },
  };
}

const DB_NAME = 'rhwp-ai';
const STORE_NAME = 'keys';

function request<T>(req: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

/**
 * IndexedDB 저장소. 열 수 없으면(사생활 보호 창·차단된 사이트 데이터) `null` —
 * 호출자는 세션 모드로 내려간다.
 */
export async function openIndexedDbKeyBackend(): Promise<KeyBackend | null> {
  let db: IDBDatabase;
  try {
    if (typeof indexedDB === 'undefined') return null;
    const open = indexedDB.open(DB_NAME, 1);
    open.onupgradeneeded = () => {
      if (!open.result.objectStoreNames.contains(STORE_NAME)) {
        open.result.createObjectStore(STORE_NAME, { keyPath: 'provider' });
      }
    };
    db = await request(open);
  } catch {
    return null;
  }
  const store = (mode: IDBTransactionMode) => db.transaction(STORE_NAME, mode).objectStore(STORE_NAME);
  return {
    async get(provider) { return (await request(store('readonly').get(provider))) as StoredKey | undefined ?? null; },
    async put(record) { await request(store('readwrite').put(record)); },
    async delete(provider) { await request(store('readwrite').delete(provider)); },
  };
}
