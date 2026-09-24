import type { LayerNode, LayerTextStyle, PageLayerTree } from './types.ts';
import { resolveRendererLocalFont, localFontFaceKey, type LocalFontRecord } from './local-fonts.ts';

export function hostFontStyle(style?: LayerTextStyle): { weight: number; slant: 'normal' | 'italic' } {
  return { weight: style?.bold ? 700 : 400, slant: style?.italic ? 'italic' : 'normal' };
}

export function primaryHostFontFamily(family: string): string {
  return family.split(',')[0].trim().replace(/^['"]|['"]$/g, '');
}

/** Only text face consumers supported by the host CanvasKit/Canvas2D paths. No glyph-resource rewriting. */
export function collectHostFontRequests(tree: PageLayerTree): LocalFontRecord[] {
  const selected = new Map<string, LocalFontRecord>();
  const stack: LayerNode[] = [tree.root];
  while (stack.length) {
    const node = stack.pop()!;
    if (node.kind === 'group') stack.push(...node.children);
    else if (node.kind === 'clipRect') stack.push(node.child);
    else for (const op of node.ops) {
      if (op.type !== 'textRun' && op.type !== 'charOverlap') continue;
      const family = op.style?.fontFamily;
      if (!family) continue;
      const record = resolveRendererLocalFont(primaryHostFontFamily(family), hostFontStyle(op.style));
      if (record?.hostReference) selected.set(localFontFaceKey(record), record);
    }
  }
  return [...selected.values()];
}
