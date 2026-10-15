import type { CanvasKit, Typeface, TypefaceFontProvider } from 'canvaskit-wasm';

/** Only prepared fonts enter this bounded provider; paint never fetches font data. */
export class CanvasKitParagraphFonts {
  private readonly entries = new Map<Typeface, { bytes: Uint8Array; alias: string }>();
  private provider: TypefaceFontProvider | null = null;
  private readonly families = new Map<string, { alias: string; members: string[]; bytes: number }>();
  private familyMembers = 0;
  private familyBytes = 0;
  private failed = false;
  private retainedBytes = 0;
  private nextAlias = 0;
  private builds = 0;
  private rejections = 0;
  private readonly kit: CanvasKit;
  private readonly maxEntries: number;
  private readonly maxBytes: number;

  constructor(kit: CanvasKit, maxEntries = 128, maxBytes = 64 * 1024 * 1024) {
    this.kit = kit; this.maxEntries = maxEntries; this.maxBytes = maxBytes;
  }

  remember(face: Typeface | null, bytes: ArrayBuffer | Uint8Array): boolean {
    if (!face) return false;
    if (this.entries.has(face)) return true;
    if (!bytes.byteLength || this.entries.size >= this.maxEntries || this.retainedBytes + bytes.byteLength > this.maxBytes) {
      this.rejections++; return false;
    }
    // A provider can be rebuilt after local-font removal without retaining caller-owned views.
    const copy = bytes instanceof Uint8Array ? Uint8Array.from(bytes) : new Uint8Array(bytes.slice(0));
    this.entries.set(face, { bytes: copy, alias: `RHWP-prepared-${this.nextAlias++}` });
    this.retainedBytes += copy.byteLength; this.invalidate();
    return true;
  }

  retain(faces: ReadonlySet<Typeface | null>): void {
    let changed = false;
    for (const [face, entry] of this.entries) if (!faces.has(face)) {
      this.entries.delete(face); this.retainedBytes -= entry.bytes.byteLength; changed = true;
    }
    if (changed) {
      const live = new Set([...this.entries.values()].map(entry => entry.alias));
      for (const [key, family] of this.families) if (family.members.some(alias => !live.has(alias))) {
        this.families.delete(key); this.familyMembers -= family.members.length; this.familyBytes -= family.bytes;
      }
      this.invalidate();
    }
  }

  source(faces: readonly (Typeface | null | readonly (Typeface | null)[])[]): {
    provider: TypefaceFontProvider; families: string[];
  } | null {
    const families: string[] = [];
    for (const candidate of faces) {
      const members = [...new Set((Array.isArray(candidate) ? candidate : [candidate])
        .flatMap(face => face && this.entries.has(face) ? [this.entries.get(face)!.alias] : []))].sort();
      if (!members.length) continue;
      if (members.length === 1) { families.push(members[0]); continue; }
      // Keep the producer's bundled family together, so Paragraph chooses its
      // physical weight/slant instead of treating Regular and Bold as fallbacks.
      const key = members.join(':');
      let family = this.families.get(key);
      if (!family) {
        const bytes = [...this.entries.values()].filter(entry => members.includes(entry.alias))
          .reduce((total, entry) => total + entry.bytes.byteLength, 0);
        if (this.familyMembers + members.length > this.maxEntries || this.familyBytes + bytes > this.maxBytes) {
          this.rejections++; return null;
        }
        family = { alias: `RHWP-family-${this.nextAlias++}`, members, bytes };
        this.families.set(key, family); this.familyMembers += members.length; this.familyBytes += bytes; this.invalidate();
      }
      families.push(family.alias);
    }
    const uniqueFamilies = [...new Set(families)];
    if (!families.length || this.failed || !this.kit.TypefaceFontProvider || !this.kit.ParagraphBuilder?.MakeFromFontProvider) return null;
    if (!this.provider) {
      let provider: TypefaceFontProvider | null = null;
      try {
        provider = this.kit.TypefaceFontProvider.Make();
        for (const entry of this.entries.values()) provider.registerFont(entry.bytes, entry.alias);
        const byAlias = new Map([...this.entries.values()].map(entry => [entry.alias, entry]));
        for (const family of this.families.values()) for (const alias of family.members) {
          provider.registerFont(byAlias.get(alias)!.bytes, family.alias);
        }
        this.provider = provider; this.builds++;
      } catch { provider?.delete(); this.failed = true; return null; }
    }
    return { provider: this.provider, families: uniqueFamilies };
  }

  diagnostics() { return { entries: this.entries.size, bytes: this.retainedBytes, builds: this.builds, rejections: this.rejections }; }
  clear(): void {
    this.entries.clear(); this.families.clear(); this.familyMembers = 0; this.familyBytes = 0; this.retainedBytes = 0; this.invalidate();
  }
  private invalidate(): void { this.provider?.delete(); this.provider = null; this.failed = false; }
}
