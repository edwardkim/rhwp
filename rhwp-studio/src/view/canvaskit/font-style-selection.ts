import type { CanvasKit, FontStyle, Typeface, TypefaceFontProvider } from 'canvaskit-wasm';

/** Select actual faces sharing one alias instead of keeping the last loaded URL. */
export class CanvasKitBundledFontFamily {
  private readonly provider: TypefaceFontProvider;
  private readonly matches = new Map<string, Typeface>();
  readonly family: string;
  constructor(kit: CanvasKit, family: string, bytes: readonly ArrayBuffer[]) {
    this.family = family;
    this.provider = kit.TypefaceFontProvider.Make();
    try {
      for (const data of bytes) this.provider.registerFont(data, family);
    } catch (error) {
      this.provider.delete();
      throw error;
    }
  }
  private match(bold: boolean, italic: boolean): Typeface {
    const key = `${bold}:${italic}`;
    let face = this.matches.get(key);
    if (!face) {
      // The value-object binding consumes numeric SkFontStyle fields. Embind
      // enum objects do not select the requested weight in matchFamilyStyle.
      const style = { weight: bold ? 700 : 400, width: 5, slant: italic ? 1 : 0 } as unknown as FontStyle;
      face = this.provider.matchFamilyStyle(this.family, style);
      this.matches.set(key, face);
    }
    return face;
  }
  select(bold: boolean, italic: boolean) {
    const typeface = this.match(bold, italic);
    return {
      typeface,
      fontManager: this.provider,
      fontFamily: this.family,
      syntheticStyle: {
        bold: bold && typeface.isAliasOf(this.match(false, italic)),
        italic: italic && typeface.isAliasOf(this.match(bold, false)),
      },
    };
  }
  dispose(): void {
    for (const face of this.matches.values()) face.delete();
    this.matches.clear();
    this.provider.delete();
  }
}
