import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';
import type { LayerResources } from './types.ts';

type ResourcePayload = Uint8Array | string;
interface ResourceEntry { payload: ResourcePayload; bytes: number }

/** Content-key hints never depend on page-local indices or on mutable caller-owned byte arrays. */
export class LayerResourceStore {
  private readonly entries = new Map<string, ResourceEntry>();
  private retainedBytes = 0;
  private static readonly MAX_ENTRIES = 256;
  private static readonly MAX_BYTES = 64 * 1024 * 1024;

  clear(): void { this.entries.clear(); this.retainedBytes = 0; }

  knownImageKeys(): string[] { return [...this.entries.keys()].filter(key => key.startsWith('img:')); }
  knownSvgKeys(): string[] { return [...this.entries.keys()].filter(key => key.startsWith('svg:')); }

  diagnostics(): { entries: number; bytes: number } { return { entries: this.entries.size, bytes: this.retainedBytes }; }

  hydrate(resources: LayerResources | undefined): LayerResources | undefined {
    if (!resources) return resources;
    const known = new Map(this.entries);
    let hydratedBytes = 0;
    const resolve = (kind: 'img' | 'svg', payloads: LayerResources['images'], keys: string[] | undefined) => {
      if (!payloads) return undefined;
      if (!Array.isArray(payloads) || payloads.length > 4096) throw new Error('layer resource table exceeds bounds');
      return Array.from({ length: payloads.length }, (_, index) => {
        const key = keys?.[index];
        const payload = payloads[index];
        if (payload === undefined) {
          const cached = typeof key === 'string' ? known.get(key) : undefined;
          if (!cached || !key?.startsWith(`${kind}:`)) throw new Error(`missing ${kind} resource hint at index ${index}`);
          hydratedBytes += cached.bytes;
          if (hydratedBytes > LayerResourceStore.MAX_BYTES) throw new Error('hydrated layer resources exceed byte budget');
          this.retain(key, cached.payload, cached.bytes);
          return this.copy(cached.payload);
        }
        let bytes: Uint8Array;
        let value: ResourcePayload;
        if (kind === 'svg') {
          if (typeof payload !== 'string' || payload.length > 4 * 1024 * 1024) throw new Error('invalid SVG resource');
          bytes = new TextEncoder().encode(payload); value = payload;
          if (bytes.length > 4 * 1024 * 1024) throw new Error('SVG resource exceeds bounds');
        } else {
          if (typeof payload === 'string') {
            if (payload.length > 24 * 1024 * 1024) throw new Error('image resource exceeds bounds');
            const binary = atob(payload);
            bytes = Uint8Array.from(binary, character => character.charCodeAt(0));
          } else if (payload instanceof Uint8Array) bytes = payload;
          else {
            if (!Array.isArray(payload) || payload.length > 18 * 1024 * 1024
              || !payload.every(byte => Number.isInteger(byte) && byte >= 0 && byte <= 255)) throw new Error('invalid image resource');
            bytes = Uint8Array.from(payload);
          }
          if (bytes.length > 18 * 1024 * 1024) throw new Error('image resource exceeds bounds');
          value = bytes;
        }
        if (key !== undefined) {
          const expected = `${kind}:blake3:${bytes.length}:${bytesToHex(blake3(bytes))}`;
          if (key !== expected) throw new Error(`invalid ${kind} content-addressed resource key`);
          this.retain(key, value, bytes.length);
        }
        hydratedBytes += bytes.length;
        if (hydratedBytes > LayerResourceStore.MAX_BYTES) throw new Error('hydrated layer resources exceed byte budget');
        return this.copy(value);
      });
    };
    return { ...resources,
      images: resolve('img', resources.images, resources.imageKeys),
      svgFragments: resolve('svg', resources.svgFragments, resources.svgKeys) as Array<string | undefined> | undefined,
    };
  }

  private copy(payload: ResourcePayload): ResourcePayload { return typeof payload === 'string' ? payload : payload.slice(); }

  private retain(key: string, payload: ResourcePayload, bytes: number): void {
    const previous = this.entries.get(key);
    if (previous) { this.entries.delete(key); this.entries.set(key, previous); return; }
    while (this.entries.size >= LayerResourceStore.MAX_ENTRIES || this.retainedBytes + bytes > LayerResourceStore.MAX_BYTES) {
      const oldest = this.entries.keys().next().value;
      if (oldest === undefined) break;
      this.retainedBytes -= this.entries.get(oldest)!.bytes;
      this.entries.delete(oldest);
    }
    this.entries.set(key, { payload: this.copy(payload), bytes });
    this.retainedBytes += bytes;
  }
}
