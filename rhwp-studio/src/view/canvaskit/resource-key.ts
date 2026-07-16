import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';

const RESOURCE_KEY_PATTERN = /^(img|svg):blake3:(0|[1-9][0-9]*):([0-9a-f]{64})$/;

export function layerResourceKeyMatches(
  expectedKind: 'img' | 'svg',
  resourceKey: string,
  bytes: Uint8Array,
): boolean {
  const match = RESOURCE_KEY_PATTERN.exec(resourceKey);
  if (!match || match[1] !== expectedKind) return false;
  const byteLength = Number(match[2]);
  return Number.isSafeInteger(byteLength)
    && byteLength === bytes.byteLength
    && bytesToHex(blake3(bytes)) === match[3];
}
