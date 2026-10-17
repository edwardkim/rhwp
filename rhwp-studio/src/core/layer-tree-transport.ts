import type { LayerRenderProfile } from './types.ts';
import { LayerResourceStore } from './layer-resource-store.ts';
import { normalizeTextV2Tree } from './text-v2-normalizer.ts';

export interface LayerTreeExportOptions {
  /** JSON remains the default; the object adapter is opt-in until measured with the same corpus. */
  transport?: 'json' | 'value';
  textProfile?: 'v1Compat' | 'v2Compat' | 'strictGlyphOutline' | 'strictGlyphRun';
}
type ExportMethod = (this: unknown, ...args: any[]) => unknown;

/** Keep optional binding detection and resource-hint lifetime independently testable from document UI. */
export function readLayerTreeTransport(
  document: object, page: number, profile: LayerRenderProfile,
  options: LayerTreeExportOptions, resources: LayerResourceStore,
): unknown {
  const methods = document as Record<string, ExportMethod | undefined>;
  const transport = options.transport ?? 'json';
  const textProfile = options.textProfile ?? 'v1Compat';
  if (!['json', 'value'].includes(transport)
    || !['v1Compat', 'v2Compat', 'strictGlyphOutline', 'strictGlyphRun'].includes(textProfile)) {
    throw new Error('unsupported layer tree export options');
  }
  const suffix = textProfile === 'v1Compat' ? '' : textProfile === 'v2Compat' ? 'V2Compat'
    : textProfile === 'strictGlyphOutline' ? 'V2StrictGlyphOutline' : 'V2StrictGlyphRun';
  let value: unknown;
  if (transport === 'value') {
    const name = `getPageLayerTreeValue${suffix}WithProfileAndResourceKeys`;
    const method = methods[name];
    if (typeof method !== 'function') throw new Error(`WASM does not support ${name}`);
    const args: unknown[] = [page, profile, resources.knownImageKeys(), resources.knownSvgKeys()];
    if (suffix === '') args.push(typeof methods.getSourceFontBytes === 'function', typeof methods.getSourceImageBytes === 'function');
    value = method.call(document, ...args);
  } else {
    const name = `getPageLayerTree${suffix}WithProfile`;
    const method = methods[name];
    let json: unknown;
    if (typeof method === 'function') {
      const args: unknown[] = [page, profile];
      if (suffix === '') args.push(false, typeof methods.getSourceFontBytes === 'function');
      json = method.call(document, ...args);
    } else if (suffix === '' && profile === 'screen' && typeof methods.getPageLayerTree === 'function') {
      json = methods.getPageLayerTree.call(document, page);
    } else throw new Error(`WASM does not support ${name}`);
    if (typeof json !== 'string') throw new Error('layer tree JSON export must return a string');
    try { value = JSON.parse(json); } catch (error) { throw new Error(`PageLayerTree JSON parse failed (page=${page}): ${error}`); }
  }
  const tree = normalizeTextV2Tree(value);
  if (transport === 'value') tree.resources = resources.hydrate(tree.resources);
  return tree;
}
