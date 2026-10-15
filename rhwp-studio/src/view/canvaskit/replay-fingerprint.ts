import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';

/** Mutating a node or a typed resource cannot retain a stale native picture. */
export function replayFingerprint(value: unknown): string | null {
  const hash = blake3.create();
  const encoder = new TextEncoder();
  const active = new Set<object>();
  let remainingBytes = 64 * 1024 * 1024;
  let remainingValues = 100_000;
  const append = (bytes: Uint8Array) => {
    remainingBytes -= bytes.byteLength;
    if (remainingBytes < 0) throw new Error('fingerprint byte budget');
    hash.update(bytes);
  };
  const text = (source: string) => {
    if (source.length > remainingBytes) throw new Error('fingerprint string budget');
    append(encoder.encode(source));
  };
  const visit = (item: unknown, depth: number): void => {
    if (--remainingValues < 0 || depth > 128) throw new Error('fingerprint work budget');
    if (item === null || typeof item !== 'object') {
      if (typeof item === 'function' || typeof item === 'symbol') throw new Error('non-data replay dependency');
      const source = String(item);
      text(`${typeof item}:${source.length}:${source};`);
      return;
    }
    if (active.has(item)) throw new Error('cyclic replay dependency');
    active.add(item);
    try {
      if (ArrayBuffer.isView(item) || item instanceof ArrayBuffer) {
        const bytes = item instanceof ArrayBuffer ? new Uint8Array(item)
          : new Uint8Array(item.buffer, item.byteOffset, item.byteLength);
        text(`bytes:${bytes.length}:`); append(bytes);
      } else if (Array.isArray(item)) {
        if (item.length > remainingValues) throw new Error('fingerprint array budget');
        text(`array:${item.length}:`);
        for (const child of item) visit(child, depth + 1);
      } else {
        const keys = Object.keys(item).sort();
        if (keys.length > remainingValues) throw new Error('fingerprint object budget');
        text(`object:${keys.length}:`);
        for (const key of keys) { text(`${key.length}:${key}:`); visit((item as Record<string, unknown>)[key], depth + 1); }
      }
    } finally { active.delete(item); }
  };
  try { visit(value, 0); return bytesToHex(hash.digest()); } catch { return null; }
}
