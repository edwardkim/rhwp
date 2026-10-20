import type { LayerEquationLayoutBox, LayerNode, LayerTextStyle, PageLayerTree } from './types.ts';
import { resolveRendererLocalFont, localFontFaceKey, type LocalFontRecord } from './local-fonts.ts';
import boxedDigitMetrics from '../../../assets/fonts/boxed-digit-metrics.json' with { type: 'json' };

export function hostFontStyle(style?: LayerTextStyle): { weight: number; slant: 'normal' | 'italic' } {
  return { weight: style?.bold ? 700 : 400, slant: style?.italic ? 'italic' : 'normal' };
}

export function primaryHostFontFamily(family: string): string {
  return family.split(',')[0].trim().replace(/^['"]|['"]$/g, '');
}

/** 실제 수식 글리프 런마다 측정에 사용한 face를 준비한다. */
function hancomEquationFamilies(root: LayerEquationLayoutBox | undefined): Set<string> {
  const families = new Set<string>();
  const stack = root ? [root] : [];
  const seen = new Set<LayerEquationLayoutBox>();
  while (stack.length) {
    const box = stack.pop()!;
    if (seen.has(box)) continue;
    seen.add(box);
    const kind = box.kind;
    switch (kind.type) {
      case 'hancomGlyphs': families.add(kind.fontFamily ?? 'HyhwpEQ'); break;
      case 'row': stack.push(...kind.children); break;
      case 'fraction': stack.push(kind.numer, kind.denom); break;
      case 'atop': stack.push(kind.top, kind.bottom); break;
      case 'sqrt':
        stack.push(kind.body);
        if (kind.index) stack.push(kind.index);
        break;
      case 'superscript': stack.push(kind.base, kind.sup); break;
      case 'subscript': stack.push(kind.base, kind.sub); break;
      case 'subSup': stack.push(kind.base, kind.sub, kind.sup); break;
      case 'bigOp':
        if (kind.sub) stack.push(kind.sub);
        if (kind.sup) stack.push(kind.sup);
        break;
      case 'limit':
        if (kind.name) stack.push(kind.name);
        if (kind.sub) stack.push(kind.sub);
        break;
      case 'matrix': stack.push(...kind.cells.flat()); break;
      case 'rel':
        stack.push(kind.arrow, kind.over);
        if (kind.under) stack.push(kind.under);
        break;
      case 'eqAlign':
        for (const row of kind.rows) stack.push(row.left, row.right);
        break;
      case 'paren':
      case 'decoration':
      case 'fontStyle': stack.push(kind.body); break;
    }
  }
  return families;
}

/** 실제 text/수식 face 소비자를 준비한다. glyph ID나 자원 내용은 다시 쓰지 않는다. */
export function collectHostFontRequests(tree: PageLayerTree): LocalFontRecord[] {
  const selected = new Map<string, LocalFontRecord>();
  const stack: LayerNode[] = [tree.root];
  while (stack.length) {
    const node = stack.pop()!;
    if (node.kind === 'group') stack.push(...node.children);
    else if ((node.kind === 'clipRect' || node.kind === 'clipPath')) stack.push(node.child);
    else for (const op of node.ops) {
      if (op.type === 'equation') {
        for (const family of hancomEquationFamilies(op.layoutBox)) {
          const record = resolveRendererLocalFont(family, { weight: 400, slant: 'normal' });
          if (record?.hostReference) selected.set(localFontFaceKey(record), record);
        }
        continue;
      }
      if (op.type !== 'textRun' && op.type !== 'charOverlap') continue;
      const family = op.style?.fontFamily;
      if (!family) continue;
      const record = resolveRendererLocalFont(primaryHostFontFamily(family), hostFontStyle(op.style));
      if (record?.hostReference) selected.set(localFontFaceKey(record), record);
      if ([...op.text].some(character => {
        const code = character.codePointAt(0) ?? 0;
        return code >= 0xf02b0 && code <= 0xf02b9;
      })) {
        const numberFace = resolveRendererLocalFont(boxedDigitMetrics.numberFace,
          { weight: 400, slant: 'normal' });
        if (numberFace?.hostReference) selected.set(localFontFaceKey(numberFace), numberFace);
      }
    }
  }
  return [...selected.values()];
}
