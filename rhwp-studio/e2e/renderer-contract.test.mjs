import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { PNG } from 'pngjs';

import { comparePngBuffers } from './helpers.mjs';

const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(studioRoot, '..');
const canvaskitPath = path.join(studioRoot, 'src/view/canvaskit-renderer.ts');
const canvaskitDirectory = path.join(studioRoot, 'src/view/canvaskit');
const canvaskitDiagnosticsPath = path.join(canvaskitDirectory, 'diagnostics.ts');
const layerTypesPath = path.join(studioRoot, 'src/core/types.ts');
const textIrV2DocPath = path.join(repoRoot, 'docs/text-ir-v2.md');
const canvaskitParityPlanDocPath = path.join(repoRoot, 'docs/canvaskit-parity-implementation.md');
const rendererBaselinePath = path.join(studioRoot, 'e2e/renderer-baseline.mjs');
const rendererBaselineManifestPath = path.join(repoRoot, 'scripts/renderer_baseline_manifest.json');
const mainPath = path.join(studioRoot, 'src/main.ts');
const embedRpcRouterPath = path.join(studioRoot, 'src/embed/rpc-router.ts');
const renderBackendPath = path.join(studioRoot, 'src/view/render-backend.ts');
const pageRendererPath = path.join(studioRoot, 'src/view/page-renderer.ts');
const canvasViewPath = path.join(studioRoot, 'src/view/canvas-view.ts');
const renderDiffWorkflowPath = path.join(repoRoot, '.github/workflows/render-diff.yml');

const canvaskitSource = fs.readFileSync(canvaskitPath, 'utf8');
const canvaskitDiagnosticsSource = fs.readFileSync(canvaskitDiagnosticsPath, 'utf8');
const layerTypesSource = fs.readFileSync(layerTypesPath, 'utf8');
const textIrV2DocSource = fs.readFileSync(textIrV2DocPath, 'utf8');
const canvaskitParityPlanDocSource = fs.readFileSync(canvaskitParityPlanDocPath, 'utf8');
const rendererBaselineSource = fs.readFileSync(rendererBaselinePath, 'utf8');
const rendererBaselineManifest = JSON.parse(fs.readFileSync(rendererBaselineManifestPath, 'utf8'));
const mainSource = fs.readFileSync(mainPath, 'utf8');
const embedRpcRouterSource = fs.readFileSync(embedRpcRouterPath, 'utf8');
const renderBackendSource = fs.readFileSync(renderBackendPath, 'utf8');
const pageRendererSource = fs.readFileSync(pageRendererPath, 'utf8');
const canvasViewSource = fs.readFileSync(canvasViewPath, 'utf8');
const renderDiffWorkflowSource = fs.readFileSync(renderDiffWorkflowPath, 'utf8');
const normalizedCanvaskitParityPlanDocSource = canvaskitParityPlanDocSource.replace(/\s+/g, ' ');

function extractBlockBody(source, signatureIndex, blockName) {
  let bodyStart = -1;
  for (let index = signatureIndex; index < source.length; index += 1) {
    if (source[index] === '{') {
      bodyStart = index;
      break;
    }
  }
  assert.notEqual(bodyStart, -1, `missing body for ${blockName}`);

  let depth = 0;
  for (let index = bodyStart; index < source.length; index += 1) {
    const char = source[index];
    if (char === '{') {
      depth += 1;
    } else if (char === '}') {
      depth -= 1;
      if (depth === 0) {
        return source.slice(bodyStart + 1, index);
      }
    }
  }

  throw new Error(`unterminated body for ${blockName}`);
}

function extractMethodBody(source, methodName) {
  let signatureIndex = source.indexOf(`private ${methodName}(`);
  if (signatureIndex === -1) {
    signatureIndex = source.indexOf(`${methodName}(`);
  }
  assert.notEqual(signatureIndex, -1, `missing method ${methodName}`);

  return extractBlockBody(source, signatureIndex, methodName);
}

function extractSwitchCaseClusterBody(methodBody, caseLabel) {
  const casePattern = new RegExp(`^\\s*case '${caseLabel}':`, 'm');
  const caseMatch = methodBody.match(casePattern);
  assert.notEqual(caseMatch, null, `missing switch case ${caseLabel}`);

  const startIndex = caseMatch.index;
  let cursor = startIndex + caseMatch[0].length;
  const labelPattern = /^\s*(case\s+'[^']+':|default:)/gm;
  labelPattern.lastIndex = cursor;
  for (
    let match = labelPattern.exec(methodBody);
    match !== null;
    match = labelPattern.exec(methodBody)
  ) {
    const betweenLabels = methodBody.slice(cursor, match.index).trim();
    if (betweenLabels !== '') {
      return methodBody.slice(startIndex, match.index);
    }
    cursor = match.index + match[0].length;
  }

  return methodBody.slice(startIndex);
}

function caseLabels(methodBody) {
  return [...methodBody.matchAll(/case\s+'([^']+)':/g)].map((match) => match[1]);
}

function tsFilesUnder(directory) {
  return fs.readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const entryPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        return tsFilesUnder(entryPath);
      }
      return entry.name.endsWith('.ts') ? [entryPath] : [];
    })
    .sort();
}

function layerPaintOpTypes() {
  const unionMatch = layerTypesSource.match(/export type LayerPaintOp =([\s\S]*?);/);
  assert.notEqual(unionMatch, null, 'missing LayerPaintOp union');
  const interfaceNames = [...unionMatch[1].matchAll(/\|\s*(Layer[A-Za-z0-9]+Op)\b/g)]
    .map((match) => match[1]);
  assert.ok(interfaceNames.length > 0, 'LayerPaintOp union has no variants');

  return interfaceNames.map((interfaceName) => {
    const interfacePattern = new RegExp(`export interface ${interfaceName} \\{[\\s\\S]*?type:\\s*'([^']+)'`);
    const interfaceMatch = layerTypesSource.match(interfacePattern);
    assert.notEqual(interfaceMatch, null, `missing literal type for ${interfaceName}`);
    return interfaceMatch[1];
  }).sort();
}

function layerNodeKinds() {
  const unionMatch = layerTypesSource.match(/export type LayerNode =([\s\S]*?);/);
  assert.notEqual(unionMatch, null, 'missing LayerNode union');
  const interfaceNames = unionMatch[1].split('|')
    .map((item) => item.trim().replace(/;$/, ''))
    .filter(Boolean);
  assert.ok(interfaceNames.length > 0, 'LayerNode union has no variants');

  return interfaceNames.map((interfaceName) => {
    const interfacePattern = new RegExp(`export interface ${interfaceName} \\{[\\s\\S]*?kind:\\s*'([^']+)'`);
    const interfaceMatch = layerTypesSource.match(interfacePattern);
    assert.notEqual(interfaceMatch, null, `missing kind literal for ${interfaceName}`);
    return interfaceMatch[1];
  }).sort();
}

function requireSnippet(source, pattern, message) {
  assert.match(source, pattern, message);
}

const renderOpBody = extractMethodBody(canvaskitSource, 'renderOp');
const renderNodeBody = extractMethodBody(canvaskitSource, 'renderNode');
const diagnosticsBody = extractMethodBody(canvaskitSource, 'diagnostics');
const makeSurfaceBody = extractMethodBody(canvaskitSource, 'makeSurface');
const renderPageCanvasKitBody = extractMethodBody(pageRendererSource, 'renderPageCanvasKit');
const renderOpCases = caseLabels(renderOpBody).sort();
const layerOpTypes = layerPaintOpTypes();
const layerNodeKindSet = layerNodeKinds();
const canvaskitSourceFiles = [
  { label: path.relative(studioRoot, canvaskitPath), source: canvaskitSource },
  ...tsFilesUnder(canvaskitDirectory).map((filePath) => ({
    label: path.relative(studioRoot, filePath),
    source: fs.readFileSync(filePath, 'utf8'),
  })),
];
const forbiddenCanvas2dApiPatterns = [
  [/document\s*\.\s*createElement\b/, 'document.createElement'],
  [/\.getContext\s*\(/, 'HTMLCanvasElement.getContext'],
  [/\bCanvasRenderingContext2D\b/, 'CanvasRenderingContext2D'],
  [/\bPath2D\b/, 'Path2D'],
  [/\.measureText\s*\(/, 'CanvasRenderingContext2D.measureText'],
  [/\bOffscreenCanvas\b/, 'OffscreenCanvas'],
  [/\bImageData\b/, 'ImageData'],
  [/\bcreateImageBitmap\s*\(/, 'createImageBitmap'],
  [/\bImageBitmap\b/, 'ImageBitmap'],
  [/\bHTMLImageElement\b/, 'HTMLImageElement'],
  [/\bnew\s+Image\s*\(/, 'new Image'],
  [/\bDOMParser\b/, 'DOMParser'],
  [/\bXMLSerializer\b/, 'XMLSerializer'],
  [/\bURL\s*\.\s*createObjectURL\s*\(/, 'URL.createObjectURL'],
  [/\bFileReader\b/, 'FileReader'],
  [/\bCanvas2DLayerRenderer\b/, 'Canvas2DLayerRenderer'],
  [/canvas2d-layer-renderer/, 'canvas2d-layer-renderer import'],
];
const canvaskitParityPlanTouchpoints = [
  { token: 'src/paint/text_v2.rs', path: path.join(repoRoot, 'src/paint/text_v2.rs'), kind: 'file' },
  {
    token: 'src/renderer/canvaskit_policy.rs',
    path: path.join(repoRoot, 'src/renderer/canvaskit_policy.rs'),
    kind: 'file',
  },
  {
    token: 'rhwp-studio/src/core/types.ts',
    path: path.join(studioRoot, 'src/core/types.ts'),
    kind: 'file',
  },
  {
    token: 'rhwp-studio/src/view/canvaskit-renderer.ts',
    path: canvaskitPath,
    kind: 'file',
  },
  {
    token: 'rhwp-studio/src/view/canvaskit/',
    path: canvaskitDirectory,
    kind: 'directory',
  },
  {
    token: 'rhwp-studio/src/view/glyph-outline-payload-status.ts',
    path: path.join(studioRoot, 'src/view/glyph-outline-payload-status.ts'),
    kind: 'file',
  },
  {
    token: 'rhwp-studio/e2e/renderer-contract.test.mjs',
    path: fileURLToPath(import.meta.url),
    kind: 'file',
  },
  {
    token: '.github/workflows/render-diff.yml',
    path: path.join(repoRoot, '.github/workflows/render-diff.yml'),
    kind: 'file',
  },
];
const canvaskitParityPlanRequiredTokens = [
  'PageLayerTree',
  'CanvasKit direct replay',
  'must not depend on Canvas2D',
  'unsupported operations stay visible',
  'TextRun compatibility',
  'GlyphRun',
  'GlyphOutline',
  'text.variantGroups',
  'ResourceArena',
  'render-diff CI',
  'passesRuntimeReadinessGate',
  'canvaskitReadinessGate',
];
const expectedUnsupportedSetMatch = canvaskitDiagnosticsSource.match(
  /const EXPECTED_CANVASKIT_UNSUPPORTED_OPS = new Set\(\[([\s\S]*?)\]\);/,
);
assert.notEqual(expectedUnsupportedSetMatch, null, 'missing CanvasKit expected unsupported op set');
const expectedUnsupportedSetBody = expectedUnsupportedSetMatch[1];
const expectedUnsupportedFunctionMatch = canvaskitDiagnosticsSource.match(
  /export function isExpectedCanvasKitUnsupportedOp\(op: string\): boolean \{([\s\S]*?)\n\}/,
);
assert.notEqual(expectedUnsupportedFunctionMatch, null, 'missing CanvasKit expected unsupported helper');
const expectedUnsupportedBody = expectedUnsupportedFunctionMatch[1];

assert.deepEqual(
  renderOpCases,
  layerOpTypes,
  'CanvasKit renderOp must explicitly mention every LayerPaintOp variant',
);
assert.deepEqual(
  layerNodeKindSet,
  ['clipRect', 'group', 'leaf'],
  'renderer contract guard should know every LayerNode kind',
);

requireSnippet(
  renderNodeBody,
  /node\.kind === 'group'[\s\S]*?for \(const child of node\.children\)[\s\S]*?this\.renderNode\(canvas, child,[\s\S]*?\}\s*return;/,
  'group nodes should recurse through children',
);
requireSnippet(
  renderNodeBody,
  /node\.kind === 'clipRect'[\s\S]*?this\.renderClipNode\(canvas, node,[\s\S]*?\);\s*return;/,
  'clipRect nodes should go through renderClipNode',
);
requireSnippet(
  renderNodeBody,
  /this\.renderLeaf\(canvas, node, replayPlane, activeLayer\);/,
  'leaf nodes should go through renderLeaf',
);
requireSnippet(
  diagnosticsBody,
  /const lastUnsupportedOps = \[\.\.\.this\.unsupportedOps\]\.sort\(\);[\s\S]*?const lastExpectedUnsupportedOps = lastUnsupportedOps\.filter\(isExpectedCanvasKitUnsupportedOp\);[\s\S]*?const lastUnexpectedUnsupportedOps = lastUnsupportedOps\.filter\([\s\S]*?!isExpectedCanvasKitUnsupportedOp\(op\)/,
  'CanvasKit diagnostics should split expected and unexpected unsupported operations',
);
requireSnippet(
  expectedUnsupportedBody,
  /return EXPECTED_CANVASKIT_UNSUPPORTED_OPS\.has\(op\);/,
  'CanvasKit expected unsupported helper should use exact diagnostics only',
);
assert.doesNotMatch(
  canvaskitDiagnosticsSource,
  /startsWith\(/,
  'CanvasKit readiness classification must not hide future diagnostic suffixes behind prefixes',
);
requireSnippet(
  diagnosticsBody,
  /if \(!this\.lastRenderCompleted\) readinessBlockers\.push\('renderNotCompleted'\);[\s\S]*?if \(this\.lastRenderError !== null\) readinessBlockers\.push\('renderError'\);[\s\S]*?if \(lastUnexpectedUnsupportedOps\.length > 0\) readinessBlockers\.push\('unexpectedUnsupportedOps'\);[\s\S]*?passesRuntimeReadinessGate: readinessBlockers\.length === 0/,
  'CanvasKit diagnostics should expose deterministic runtime readiness blockers',
);
requireSnippet(
  canvaskitSource,
  /this\.lastRenderCompleted = false;[\s\S]*?surface\.flush\(\);[\s\S]*?this\.lastRenderCompleted = true;/,
  'CanvasKit readiness should require a completed surface flush',
);
requireSnippet(
  makeSurfaceBody,
  /try \{[\s\S]*?MakeCanvasSurface\(targetCanvas\)[\s\S]*?this\.surfaceBackend = 'default'[\s\S]*?\} catch \{[\s\S]*?defaultSurfaceCreationFailed[\s\S]*?MakeSWCanvasSurface\(softwareCanvas\)[\s\S]*?this\.surfaceBackend = 'software'/,
  'CanvasKit auto surface creation should fall back to software after default surface exceptions',
);
requireSnippet(
  makeSurfaceBody,
  /targetCanvas\.parentElement !== originalParent[\s\S]*?this\.surfaceBackend = 'software';[\s\S]*?canvas: replacement/,
  'CanvasKit internal software fallback should expose its replacement canvas',
);
requireSnippet(
  makeSurfaceBody,
  /surfaceRequest\.preference === 'webgpu'[\s\S]*?surfaceFallbackReason = 'webgpuSurfaceUnsupported'[\s\S]*?reuseSoftwareFallbackCanvas/,
  'CanvasKit repeated software fallback should preserve the original WebGPU rejection reason',
);
requireSnippet(
  renderPageCanvasKitBody,
  /canvaskitDiagnosticsByPage\.delete\(pageIdx\);[\s\S]*?try \{[\s\S]*?getPageInfo\(pageIdx\)[\s\S]*?getPageLayerTreeObject\(pageIdx[\s\S]*?renderStarted = true;[\s\S]*?recordRenderFailure\(error, !renderStarted\)[\s\S]*?if \(!renderStarted\) throw error/,
  'CanvasKit page diagnostics should be cleared before page info or layer lowering can fail',
);
requireSnippet(
  pageRendererSource,
  /canvaskitDiagnosticsByPage = new Map<number, CanvasKitRenderDiagnostics>\(\)[\s\S]*?getCanvasKitRenderDiagnostics\(pageIdx: number\)[\s\S]*?this\.canvaskitDiagnosticsByPage\.get\(pageIdx\)[\s\S]*?this\.canvaskitDiagnosticsByPage\.set\(pageIdx, this\.canvaskitRenderer\.diagnostics\(\)\)/,
  'PageRenderer should retain CanvasKit diagnostics by page instead of global last-render state',
);
requireSnippet(
  canvasViewSource,
  /getCanvasKitRenderDiagnostics\(pageIndex: number\)[\s\S]*?this\.pageRenderer\.getCanvasKitRenderDiagnostics\(pageIndex\)/,
  'CanvasView should expose page-scoped CanvasKit diagnostics',
);
requireSnippet(
  mainSource,
  /async getRendererDiagnostics\(pageIndex\)[\s\S]*?initialized: rendererInitialized[\s\S]*?initializationError:[\s\S]*?effectiveBackend: rendererInitialized \?[\s\S]*?backendFallbackReason:[\s\S]*?getCanvasKitRenderDiagnostics\(pageIndex\)/,
  'Studio iframe API should expose backend selection and page-scoped renderer diagnostics',
);
requireSnippet(
  embedRpcRouterSource,
  /case 'getRendererDiagnostics':[\s\S]*?Number\(params\.page \?\? 0\)[\s\S]*?page must be a non-negative integer[\s\S]*?handlers\.getRendererDiagnostics\(page\)/,
  'Embed router should preserve renderer diagnostics and reject invalid page indexes',
);
assert.doesNotMatch(
  mainSource,
  /effectiveBackend: canvasView\?\.getRenderBackend\(\) \?\? 'canvas2d'/,
  'Studio diagnostics must not report Canvas2D when no renderer initialized',
);
requireSnippet(
  mainSource,
  /CanvasKit 초기화 실패[\s\S]*?renderBackend = 'canvas2d';[\s\S]*?renderBackendFallbackReason = 'canvaskitInitializationFailed';/,
  'CanvasKit initialization failure should remain an observable Canvas2D fallback',
);
assert.doesNotMatch(
  renderBackendSource,
  /rhwp\.renderBackend|persistRenderBackend/,
  'CanvasKit backend opt-in should stay URL-only',
);

const directReplayOps = [
  ['ellipse', 'renderEllipse'],
  ['footnoteMarker', 'renderTextRun'],
  ['formObject', 'renderFormObject'],
  ['image', 'renderImage'],
  ['line', 'renderLine'],
  ['pageBackground', 'renderPageBackground'],
  ['path', 'renderPath'],
  ['placeholder', 'renderPlaceholder'],
  ['rectangle', 'renderRectangle'],
  ['textRun', 'renderTextRun'],
];
const textRunFallbackOps = [
  'charOverlap',
  'glyphRun',
  'tabLeader',
  'textControlMark',
  'textDecoration',
];
const objectFragmentFallbackOps = [
  ['equation', 'equation:unsupportedDirectReplay'],
  ['rawSvg', 'rawSvg:unsupportedDirectReplay'],
];

for (const [op, renderMethod] of directReplayOps) {
  const caseBody = extractSwitchCaseClusterBody(renderOpBody, op);
  requireSnippet(
    caseBody,
    new RegExp(`this\\.${renderMethod}\\(canvas,`),
    `${op} should dispatch to a CanvasKit replay method`,
  );
  requireSnippet(caseBody, /\breturn;/, `${op} should terminate inside its own switch case`);
  assert.doesNotMatch(
    caseBody,
    /unsupportedOps\.add/,
    `${op} direct replay case should not mark the op unsupported`,
  );
}

for (const op of textRunFallbackOps) {
  const caseBody = extractSwitchCaseClusterBody(renderOpBody, op);
  requireSnippet(caseBody, new RegExp(`case '${op}':`), `${op} should remain in the fallback case group`);
  requireSnippet(
    caseBody,
    /this\.unsupportedOps\.add\(op\.type\);\s*return;/,
    `${op} should stay on the declared unsupported/TextRun fallback path`,
  );
  assert.doesNotMatch(
    caseBody,
    /this\.render[A-Za-z0-9]+\(/,
    `${op} fallback case should not direct-render before the fallback policy changes`,
  );
}

for (const [op, unsupportedReason] of objectFragmentFallbackOps) {
  const caseBody = extractSwitchCaseClusterBody(renderOpBody, op);
  requireSnippet(caseBody, new RegExp(`case '${op}':`), `${op} should have an explicit CanvasKit fallback case`);
  requireSnippet(
    caseBody,
    new RegExp(`this\\.unsupportedOps\\.add\\('${unsupportedReason}'\\);\\s*return;`),
    `${op} should report the declared direct replay gap`,
  );
  assert.doesNotMatch(
    caseBody,
    /this\.render[A-Za-z0-9]+\(/,
    `${op} fallback case should not direct-render before the fallback policy changes`,
  );
}
for (const expectedUnsupportedToken of [
  'charOverlap',
  'equation:unsupportedDirectReplay',
  'rawSvg:unsupportedDirectReplay',
  'glyphRun',
  'tabLeader',
  'textControlMark',
  'textDecoration',
  'textRunFont',
  'image:dataMissing',
  'image:invalidBounds',
  'image:dimensionUnavailable',
  'image:tileLimit',
  'glyphOutline:unsupportedColorGlyph',
  'imageEffect:grayScale',
  'textRun:verticalText',
  'textRun:scriptTextRequiresShaping',
]) {
  assert.ok(
    expectedUnsupportedSetBody.includes(`'${expectedUnsupportedToken}'`),
    `CanvasKit expected unsupported set should include ${expectedUnsupportedToken}`,
  );
}
assert.ok(
  !expectedUnsupportedSetBody.includes("'renderPage'"),
  'CanvasKit render failures should stay unexpected readiness diagnostics',
);
assert.ok(
  !expectedUnsupportedSetBody.includes("'unknown'"),
  'CanvasKit unknown op diagnostics should stay unexpected readiness diagnostics',
);
assert.ok(
  !expectedUnsupportedSetBody.includes("'glyphOutline:replayInvariant'"),
  'CanvasKit replay invariants should stay unexpected readiness diagnostics',
);

const glyphOutlineCaseBody = extractSwitchCaseClusterBody(renderOpBody, 'glyphOutline');
requireSnippet(
  glyphOutlineCaseBody,
  /const status = glyphOutlinePayloadStatus\(op,[\s\S]*?if \(status\.supported && op\.payloadKind === 'colorLayers'\) \{[\s\S]*?this\.renderGlyphOutline\(canvas, op\);\s*return;\s*\}[\s\S]*?this\.unsupportedOps\.add\(status\.reason \? `glyphOutline:\$\{status\.reason\}` : 'glyphOutline'\);\s*return;/,
  'glyphOutline should stay guarded by payload status before direct replay',
);

const renderRectangleBody = extractMethodBody(canvaskitSource, 'renderRectangle');
const renderEllipseBody = extractMethodBody(canvaskitSource, 'renderEllipse');
const renderPathBody = extractMethodBody(canvaskitSource, 'renderPath');
const renderLineBody = extractMethodBody(canvaskitSource, 'renderLine');
const renderFormObjectBody = extractMethodBody(canvaskitSource, 'renderFormObject');
const renderPlaceholderBody = extractMethodBody(canvaskitSource, 'renderPlaceholder');
const renderTextRunBody = extractMethodBody(canvaskitSource, 'renderTextRun');
const renderShapedScriptTextBody = extractMethodBody(canvaskitSource, 'renderShapedScriptText');
const renderGlyphOutlineBody = extractMethodBody(canvaskitSource, 'renderGlyphOutline');
const renderColorPaintGraphNodeBody = extractMethodBody(canvaskitSource, 'renderColorPaintGraphNode');
const recordTextRunCoverageGapsBody = extractMethodBody(canvaskitSource, 'recordTextRunCoverageGaps');

const vite = await createServer({
  root: studioRoot,
  server: { middlewareMode: true },
  appType: 'custom',
  logLevel: 'silent',
});
let CanvasKitLayerRendererRuntime;
try {
  ({ CanvasKitLayerRenderer: CanvasKitLayerRendererRuntime } = await vite.ssrLoadModule(
    '/src/view/canvaskit-renderer.ts',
  ));
} finally {
  await vite.close();
}

function runExecutableTextReplay(op, {
  glyphIds,
  drawGlyphsError,
  drawParagraphError,
  shapedTextAvailable = true,
} = {}) {
  const events = [];
  const unsupportedOps = new Set();
  const replayText = op.displayText ?? op.text;
  const resolvedGlyphIds = glyphIds
    ?? Array.from({ length: Array.from(replayText).length }, (_, index) => index + 1);

  class FakeFont {
    constructor(_typeface, size) {
      events.push({ type: 'font.create', size });
    }

    getGlyphIDs(text, count) {
      events.push({ type: 'font.getGlyphIDs', text, count });
      return Uint16Array.from(resolvedGlyphIds);
    }

    delete() {
      events.push({ type: 'font.delete' });
    }
  }

  class FakeParagraphStyle {
    constructor(style) {
      this.style = style;
      events.push({ type: 'paragraphStyle.create', style });
    }
  }

  const paragraph = {
    layout(width) {
      events.push({ type: 'paragraph.layout', width });
    },
    delete() {
      events.push({ type: 'paragraph.delete' });
    },
  };
  const paragraphBuilder = {
    addText(text) {
      events.push({ type: 'paragraphBuilder.addText', text });
    },
    build() {
      events.push({ type: 'paragraphBuilder.build' });
      return paragraph;
    },
    delete() {
      events.push({ type: 'paragraphBuilder.delete' });
    },
  };

  const paint = {
    setAntiAlias(value) {
      events.push({ type: 'paint.antiAlias', value });
    },
    delete() {
      events.push({ type: 'paint.delete' });
    },
  };
  const canvas = {
    save() {
      events.push({ type: 'canvas.save' });
    },
    concat(matrix) {
      events.push({ type: 'canvas.concat', matrix: Array.from(matrix) });
    },
    rotate(rotation, x, y) {
      events.push({ type: 'canvas.rotate', rotation, x, y });
    },
    drawGlyphs(ids, positions, x, y) {
      events.push({
        type: 'canvas.drawGlyphs',
        glyphIds: Array.from(ids),
        positions: Array.from(positions),
        x,
        y,
      });
      if (drawGlyphsError) throw drawGlyphsError;
    },
    drawText(text, x, y) {
      events.push({ type: 'canvas.drawText', text, x, y });
    },
    drawParagraph(_paragraph, x, y) {
      events.push({ type: 'canvas.drawParagraph', x, y });
      if (drawParagraphError) throw drawParagraphError;
    },
    restore() {
      events.push({ type: 'canvas.restore' });
    },
  };
  const renderer = new CanvasKitLayerRendererRuntime({
    Font: FakeFont,
    ParagraphStyle: FakeParagraphStyle,
    ParagraphBuilder: {
      Make(style, fontManager) {
        events.push({ type: 'paragraphBuilder.make', style, fontManager });
        return paragraphBuilder;
      },
    },
  }, 'default', {}, {}, shapedTextAvailable ? {} : null, 'Noto Sans KR');
  renderer.unsupportedOps = unsupportedOps;
  renderer.recordTextRunCoverageGaps = () => {
    events.push({ type: 'coverage.record' });
  };
  renderer.makeFillPaint = () => {
    events.push({ type: 'paint.create' });
    return paint;
  };
  renderer.color = (color) => color;

  let error = null;
  try {
    renderer.renderTextRun(canvas, op);
  } catch (caught) {
    error = caught;
  }
  return { error, events, unsupportedOps };
}

requireSnippet(
  renderRectangleBody,
  /this\.drawStyledShape\(canvas, op\.bbox, op\.style,[\s\S]*?drawRRect[\s\S]*?drawRect/,
  'rectangle replay should stay on drawStyledShape and handle rounded and plain rectangles',
);
requireSnippet(
  renderEllipseBody,
  /this\.drawStyledShape\(canvas, op\.bbox, op\.style,[\s\S]*?drawOval/,
  'ellipse replay should stay on drawStyledShape',
);
requireSnippet(
  renderPathBody,
  /new this\.canvasKit\.Path\(\)[\s\S]*?this\.applyPathCommand[\s\S]*?this\.drawStyledPath/,
  'path replay should build CanvasKit paths through applyPathCommand and drawStyledPath',
);
requireSnippet(
  renderLineBody,
  /this\.makeStrokePaint\(op\.style\?\.color[\s\S]*?canvas\.drawLine\(op\.x1, op\.y1, op\.x2, op\.y2, paint\)/,
  'line replay should draw a CanvasKit line with stroke paint',
);
requireSnippet(
  renderFormObjectBody,
  /op\.formType === 'checkbox' \|\| op\.formType === 'radio'[\s\S]*?canvas\.drawLine[\s\S]*?const label = op\.caption \|\| op\.text[\s\S]*?this\.renderTextRun/,
  'form object replay should keep checkbox/radio mark and caption text branches explicit',
);
for (const [label, body, baselinePattern] of [
  ['footnote marker', extractSwitchCaseClusterBody(renderOpBody, 'footnoteMarker'), /baseline: op\.fontSize \?\? 7/],
  ['form object', renderFormObjectBody, /baseline: Math\.max\(10, op\.bbox\.height \* 0\.68\)/],
  ['placeholder', renderPlaceholderBody, /baseline: Math\.max\(10, op\.bbox\.height \* 0\.65\)/],
]) {
  requireSnippet(body, baselinePattern, `${label} replay should declare its run-local baseline`);
  assert.doesNotMatch(
    body,
    /baseline:\s*op\.bbox\.y/,
    `${label} replay should pass a run-local baseline to renderTextRun`,
  );
}
requireSnippet(
  renderTextRunBody,
  /this\.recordTextRunCoverageGaps\(op\);[\s\S]*?canvas\.drawGlyphs\(glyphIds, glyphPositions, originX, originY, font, paint\)/,
  'textRun replay should record unsupported effect diagnostics before drawing positioned glyphs',
);
requireSnippet(
  renderTextRunBody,
  /const placementMatrix = this\.affineToCanvasKitMatrix\(op\.placement\?\.runToPage\);[\s\S]*?op\.bbox\.y \+ \(op\.baseline \?\? baseFontSize\)[\s\S]*?canvas\.concat\(placementMatrix\);[\s\S]*?canvas\.rotate\(rotation, originX, originY\);/,
  'textRun replay should use canonical run placement with a page-space fallback',
);
requireSnippet(
  renderTextRunBody,
  /let fontSize = baseFontSize;[\s\S]*?let baselineShift = 0;[\s\S]*?style\.superscript[\s\S]*?fontSize = baseFontSize \* 0\.7;[\s\S]*?baselineShift -= baseFontSize \* 0\.3;[\s\S]*?style\.subscript[\s\S]*?fontSize = baseFontSize \* 0\.7;[\s\S]*?baselineShift \+= baseFontSize \* 0\.15;/,
  'textRun replay should apply superscript/subscript offsets in run-local space',
);
requireSnippet(
  renderTextRunBody,
  /const replayText = op\.displayText \?\? op\.text;[\s\S]*?const replayPositions = op\.displayText !== undefined \? op\.displayPositions : op\.positions;[\s\S]*?const codePoints = Array\.from\(replayText\);[\s\S]*?const hasSimpleScriptText[\s\S]*?code >= 0x20 && code <= 0x7e[\s\S]*?needsPreservedAdvances && !hasSimpleScriptText[\s\S]*?this\.renderShapedScriptText\([\s\S]*?needsPreservedAdvances && hasLayoutPositions[\s\S]*?font\.getGlyphIDs\(replayText, codePoints\.length\)[\s\S]*?glyphIds\.every\(\(glyphId\) => glyphId !== 0\)[\s\S]*?glyphPositions\[index \* 2\] = replayPositions!\[index\];[\s\S]*?canvas\.drawGlyphs\(glyphIds, glyphPositions, originX, originY, font, paint\)/,
  'textRun replay should preserve serialized layout advances when glyph size changes',
);
requireSnippet(
  renderShapedScriptTextBody,
  /new this\.canvasKit\.ParagraphStyle[\s\S]*?this\.canvasKit\.ParagraphBuilder\.Make[\s\S]*?builder\.addText\(text\)[\s\S]*?paragraph\.layout\(CanvasKitLayerRenderer\.MAX_SHAPED_TEXT_WIDTH\)[\s\S]*?canvas\.drawParagraph\(paragraph, originX, originY - fontSize \+ baselineShift\)[\s\S]*?paragraph\.delete\?\.\(\)[\s\S]*?builder\.delete\?\.\(\)/,
  'non-ASCII script replay should use CanvasKit paragraph shaping and release native objects',
);
requireSnippet(
  renderTextRunBody,
  /textRun:scriptTextRequiresShaping[\s\S]*?textRun:glyphMapping[\s\S]*?textRun:layoutPositions/,
  'textRun replay should expose unavailable shaping and malformed positioned-text fallbacks',
);
requireSnippet(
  renderTextRunBody,
  /try \{[\s\S]*?canvas\.save\(\);[\s\S]*?\} finally \{[\s\S]*?if \(canvasSaved\) canvas\.restore\(\);[\s\S]*?font\?\.delete\?\.\(\);[\s\S]*?paint\.delete\?\.\(\);/,
  'textRun replay should restore CanvasKit state and delete native objects after failures',
);

const placedSuperscriptReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 10, y: 100, width: 30, height: 20 },
  text: 'AB',
  baseline: 15,
  rotation: 90,
  placement: {
    runToPage: { a: 0, b: 1, c: -1, d: 0, e: 50, f: 60 },
    baselineY: 0,
  },
  positions: [0, 12, 24],
  style: { fontSize: 20, superscript: true },
});
assert.equal(placedSuperscriptReplay.error, null);
assert.deepEqual(
  placedSuperscriptReplay.events.find((event) => event.type === 'canvas.concat')?.matrix,
  [0, -1, 50, 1, 0, 60, 0, 0, 1],
  'placement transform should use the serialized affine coefficient order',
);
assert.equal(
  placedSuperscriptReplay.events.some((event) => event.type === 'canvas.rotate'),
  false,
  'placement transform should suppress the legacy rotation fallback',
);
assert.deepEqual(
  placedSuperscriptReplay.events.find((event) => event.type === 'canvas.drawGlyphs'),
  {
    type: 'canvas.drawGlyphs',
    glyphIds: [1, 2],
    positions: [0, -6, 12, -6],
    x: 0,
    y: 0,
  },
  'superscript replay should keep producer advances and apply a run-local baseline shift',
);

const rotatedSubscriptReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 7, y: 100, width: 30, height: 20 },
  text: 'AB',
  baseline: 15,
  rotation: 90,
  positions: [0, 9, 18],
  style: { fontSize: 20, subscript: true },
});
assert.deepEqual(
  rotatedSubscriptReplay.events.find((event) => event.type === 'canvas.rotate'),
  { type: 'canvas.rotate', rotation: 90, x: 7, y: 115 },
  'legacy placement fallback should add the run-local baseline exactly once',
);
assert.deepEqual(
  rotatedSubscriptReplay.events.find((event) => event.type === 'canvas.drawGlyphs')?.positions,
  [0, 3, 9, 3],
  'subscript replay should apply its baseline shift in rotated run-local space',
);

const projectedTextReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: '\u{F012B}',
  displayText: '(인)',
  baseline: 15,
  positions: [0, 5],
  displayPositions: [0, 11, 22, 33],
  style: { fontSize: 20, superscript: true },
});
assert.deepEqual(
  projectedTextReplay.events.find((event) => event.type === 'paragraphBuilder.addText'),
  { type: 'paragraphBuilder.addText', text: '(인)' },
  'CanvasKit replay should shape the actual PUA display projection',
);
assert.equal(
  projectedTextReplay.events.some((event) => event.type === 'canvas.drawGlyphs'),
  false,
  'a non-ASCII PUA display projection should not enter direct glyph replay',
);

const shapedTextReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'e\u0301',
  baseline: 15,
  positions: [0, 8, 8],
  style: { fontSize: 20, superscript: true },
});
assert.equal(
  shapedTextReplay.events.some((event) => event.type === 'font.getGlyphIDs'),
  false,
  'text requiring shaping should not enter nominal glyph replay',
);
assert.equal(
  shapedTextReplay.events.some((event) => event.type === 'canvas.drawParagraph'),
  true,
  'text requiring shaping should use CanvasKit paragraph replay',
);
assert.equal(shapedTextReplay.unsupportedOps.has('textRun:scriptTextRequiresShaping'), false);

const unavailableShapingReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'e\u0301',
  baseline: 15,
  positions: [0, 8, 8],
  style: { fontSize: 20, superscript: true },
}, { shapedTextAvailable: false });
assert.equal(unavailableShapingReplay.unsupportedOps.has('textRun:scriptTextRequiresShaping'), true);
assert.equal(
  unavailableShapingReplay.events.some((event) => event.type === 'canvas.drawText'),
  false,
  'text requiring shaping must not silently fall back to CanvasKit drawText',
);

const missingGlyphReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'AB',
  baseline: 15,
  positions: [0, 8, 16],
  style: { fontSize: 20, superscript: true },
}, { glyphIds: [1, 0] });
assert.equal(missingGlyphReplay.unsupportedOps.has('textRun:glyphMapping'), true);
assert.equal(
  missingGlyphReplay.events.some((event) => event.type === 'canvas.drawGlyphs'),
  false,
  'glyph ID zero should reject positioned glyph replay',
);

const cleanupReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'AB',
  baseline: 15,
  positions: [0, 8, 16],
  style: { fontSize: 20, superscript: true },
}, { drawGlyphsError: new Error('draw failed') });
assert.equal(cleanupReplay.error?.message, 'draw failed');
for (const cleanupEvent of ['canvas.restore', 'font.delete', 'paint.delete']) {
  assert.equal(
    cleanupReplay.events.some((event) => event.type === cleanupEvent),
    true,
    `${cleanupEvent} should run after drawGlyphs throws`,
  );
}

const shapedCleanupReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'e\u0301',
  baseline: 15,
  positions: [0, 8, 8],
  style: { fontSize: 20, superscript: true },
}, { drawParagraphError: new Error('paragraph draw failed') });
assert.equal(shapedCleanupReplay.error?.message, 'paragraph draw failed');
for (const cleanupEvent of ['canvas.restore', 'paragraph.delete', 'paragraphBuilder.delete', 'paint.delete']) {
  assert.equal(
    shapedCleanupReplay.events.some((event) => event.type === cleanupEvent),
    true,
    `${cleanupEvent} should run after drawParagraph throws`,
  );
}
for (const expectedTextRunGap of [
  'textRun:verticalText',
  'textRun:textDecoration',
  'textRun:emphasisDot',
  'textRun:outlineTextEffect',
  'textRun:shadowTextEffect',
  'textRun:embossTextEffect',
  'textRun:engraveTextEffect',
  'textRun:shadeTextEffect',
  'textRun:ratioTextEffect',
]) {
  assert.ok(
    recordTextRunCoverageGapsBody.includes(`'${expectedTextRunGap}'`),
    `textRun runtime diagnostics should include ${expectedTextRunGap}`,
  );
}
requireSnippet(
  renderGlyphOutlineBody,
  /op\.colorLayers\?\.paintGraph[\s\S]*?graph\.rootNodeId[\s\S]*?this\.renderColorPaintGraphNode/,
  'glyphOutline replay should require a colorLayers paint graph root',
);
requireSnippet(
  renderColorPaintGraphNodeBody,
  /visited\.has\(nodeId\)[\s\S]*?replayInvariant[\s\S]*?return;[\s\S]*?visited\.add\(nodeId\);/,
  'glyphOutline color graph replay should record visited nodes before recursion',
);
requireSnippet(
  renderColorPaintGraphNodeBody,
  /node\.kind === 'transform'[\s\S]*?transformNode\?\.childNodeId[\s\S]*?this\.renderColorPaintGraphNode\(canvas, nodesById, transformNode\.childNodeId, visited\)/,
  'glyphOutline color graph replay should keep transform recursion explicit',
);
requireSnippet(
  renderColorPaintGraphNodeBody,
  /node\.solidPath \?\? node\.linearGradientPath \?\? node\.radialGradientPath \?\? node\.sweepGradientPath[\s\S]*?node\.kind === 'solidPath' && node\.solidPath\?\.fill[\s\S]*?node\.kind === 'linearGradientPath' && node\.linearGradientPath\?\.gradient[\s\S]*?node\.kind === 'radialGradientPath' && node\.radialGradientPath\?\.gradient[\s\S]*?node\.kind === 'sweepGradientPath' && node\.sweepGradientPath\?\.gradient/,
  'glyphOutline color graph replay should keep cycle guard and supported path families explicit',
);

for (const { label, source } of canvaskitSourceFiles) {
  for (const [pattern, name] of forbiddenCanvas2dApiPatterns) {
    assert.doesNotMatch(
      source,
      pattern,
      `CanvasKit direct replay source ${label} must not depend on ${name}`,
    );
  }
}

for (const { token, path: touchpointPath, kind } of canvaskitParityPlanTouchpoints) {
  assert.ok(
    canvaskitParityPlanDocSource.includes(token),
    `CanvasKit parity plan should mention touchpoint ${token}`,
  );

  const stat = fs.statSync(touchpointPath);
  if (kind === 'directory') {
    assert.ok(stat.isDirectory(), `CanvasKit parity plan touchpoint ${token} should be a directory`);
  } else {
    assert.ok(stat.isFile(), `CanvasKit parity plan touchpoint ${token} should be a file`);
  }
}

for (const token of canvaskitParityPlanRequiredTokens) {
  assert.ok(
    normalizedCanvaskitParityPlanDocSource.includes(token),
    `CanvasKit parity plan should keep guard token: ${token}`,
  );
}

assert.ok(
  textIrV2DocSource.includes('docs/canvaskit-parity-implementation.md'),
  'Text IR v2 contract should link to the CanvasKit parity implementation plan',
);

const shiftedInkExpected = new PNG({ width: 3, height: 1 });
shiftedInkExpected.data.fill(255);
shiftedInkExpected.data.set([0, 0, 0, 255], 0);
const shiftedInkActual = new PNG({ width: 3, height: 1 });
shiftedInkActual.data.fill(255);
shiftedInkActual.data.set([0, 0, 0, 255], 4);
const shiftedInkDiff = await comparePngBuffers(
  PNG.sync.write(shiftedInkExpected),
  PNG.sync.write(shiftedInkActual),
  {
    inkMaskNeighborhoodRadius: 1,
    inkMaskMaxDiffPixels: 0,
    nonInkMaxDiffPixels: 0,
    solidInkMaxDiffPixels: 0,
  },
);
assert.equal(shiftedInkDiff.passed, true, 'nearby rasterized ink should pass the ink-mask gate');
assert.equal(shiftedInkDiff.hasVisualBudget, true);
assert.equal(shiftedInkDiff.passMetric, 'rasterOnly');

const collapsedInkExpected = new PNG({ width: 3, height: 1 });
collapsedInkExpected.data.fill(255);
collapsedInkExpected.data.set([0, 0, 0, 255], 0);
collapsedInkExpected.data.set([0, 0, 0, 255], 8);
const collapsedInkActual = new PNG({ width: 3, height: 1 });
collapsedInkActual.data.fill(255);
collapsedInkActual.data.set([0, 0, 0, 255], 4);
const collapsedInkDiff = await comparePngBuffers(
  PNG.sync.write(collapsedInkExpected),
  PNG.sync.write(collapsedInkActual),
  { inkMaskNeighborhoodRadius: 1, inkMaskMaxDiffPixels: 0 },
);
assert.equal(
  collapsedInkDiff.passed,
  false,
  'one actual ink pixel must not satisfy multiple expected ink pixels',
);
assert.equal(collapsedInkDiff.inkMaskDiffPixels, 1);

const augmentingInkExpected = new PNG({ width: 3, height: 2 });
augmentingInkExpected.data.fill(255);
augmentingInkExpected.data.set([0, 0, 0, 255], 4);
augmentingInkExpected.data.set([0, 0, 0, 255], 12);
const augmentingInkActual = new PNG({ width: 3, height: 2 });
augmentingInkActual.data.fill(255);
augmentingInkActual.data.set([0, 0, 0, 255], 0);
augmentingInkActual.data.set([0, 0, 0, 255], 8);
const augmentingInkDiff = await comparePngBuffers(
  PNG.sync.write(augmentingInkExpected),
  PNG.sync.write(augmentingInkActual),
  { inkMaskNeighborhoodRadius: 1, inkMaskMaxDiffPixels: 0 },
);
assert.equal(
  augmentingInkDiff.inkMaskDiffPixels,
  0,
  'one-to-one ink matching should find an augmenting path instead of depending on scan order',
);

const missingInkExpected = new PNG({ width: 3, height: 1 });
missingInkExpected.data.fill(255);
const missingInkActual = new PNG({ width: 3, height: 1 });
missingInkActual.data.fill(255);
missingInkActual.data.set([0, 0, 0, 255], 8);
const missingInkDiff = await comparePngBuffers(
  PNG.sync.write(missingInkExpected),
  PNG.sync.write(missingInkActual),
  { inkMaskMaxDiffPixels: 0 },
);
assert.equal(missingInkDiff.passed, false, 'new unmatched ink should fail the ink-mask gate');
assert.equal(missingInkDiff.inkMaskDiffPixels, 1);
const noBudgetDiff = await comparePngBuffers(
  PNG.sync.write(missingInkExpected),
  PNG.sync.write(missingInkExpected),
);
assert.equal(noBudgetDiff.hasVisualBudget, false, 'readiness requires an explicit visual budget');
const blankInkDiff = await comparePngBuffers(
  PNG.sync.write(missingInkExpected),
  PNG.sync.write(missingInkExpected),
  { maxDiffPixels: 0, minimumInkPixels: 1 },
);
assert.equal(blankInkDiff.passed, false, 'matching blank captures must not pass readiness');
assert.equal(blankInkDiff.minimumInkBudgetPassed, false);
assert.match(
  comparePngBuffers.toString(),
  /MAX_INK_MASK_MATCH_EDGES/,
  'ink-mask maximum matching should stop before allocating an unbounded edge graph',
);

assert.deepEqual(
  rendererBaselineManifest.samples
    .filter((sample) => sample.canvaskitReadinessGate === true)
    .map((sample) => sample.id)
    .sort(),
  ['image-crop', 'paragraph-line-basic', 'table-core'],
  'CanvasKit readiness gate should stay limited to the representative paragraph/table/image corpus',
);
requireSnippet(
  rendererBaselineSource,
  /getCanvasKitRenderDiagnostics\?\.\(targetPageIndex\)[\s\S]*?canvasPool\?\.getCanvas\?\.\(targetPageIndex\)[\s\S]*?activeBackend: window\.__renderBackend[\s\S]*?request: window\.__rendererRuntimeRequest[\s\S]*?canvasOwnershipTracked/,
  'CanvasKit baseline should read page-scoped diagnostics and effective backend selection',
);
requireSnippet(
  rendererBaselineSource,
  /readinessGateRequired: options\.readinessOnly[\s\S]*?backend\.key === 'canvaskit-default'[\s\S]*?profile === 'screen'[\s\S]*?options\.canvaskitSurface === 'auto'/,
  'CanvasKit readiness gate should be explicit and limited to default screen/auto captures',
);
for (const readinessGuard of [
  'backendNotActive',
  'explicitCanvasKitRequestMissing',
  'canvaskitModeRequestMismatch',
  'canvaskitSurfaceRequestMismatch',
  'canvaskitModeMismatch',
  'canvaskitSurfacePreferenceMismatch',
  'canvasOwnershipMismatch',
  'diagnosticsUnavailable',
  'runtime:readinessGateFailed',
  'visualThresholdMissing',
  'visualParityFailed',
]) {
  assert.ok(
    rendererBaselineSource.includes(readinessGuard),
    `CanvasKit readiness baseline should keep guard ${readinessGuard}`,
  );
}
requireSnippet(
  rendererBaselineSource,
  /canvaskitReadinessGate\.summary\.failed > 0[\s\S]*?process\.exitCode = 1/,
  'CanvasKit readiness baseline should fail after writing its JSON report',
);
requireSnippet(
  rendererBaselineSource,
  /catch \(error\) \{[\s\S]*?captureError =[\s\S]*?writeFileSync\([\s\S]*?captureError/,
  'CanvasKit readiness baseline should preserve a JSON report after browser capture failures',
);
assert.ok(
  rendererBaselineSource.includes("--readiness-only cannot be combined with --filter"),
  'CanvasKit readiness should reject partial filtered corpus runs',
);
assert.ok(
  rendererBaselineSource.includes('BROWSER_PARITY_ALLOWED_THRESHOLDS'),
  'CanvasKit readiness should validate visual threshold keys and ranges',
);
assert.ok(
  rendererBaselineSource.includes('requires a positive minimumInkPixels threshold'),
  'CanvasKit readiness samples should require an explicit positive ink floor',
);
requireSnippet(
  renderDiffWorkflowSource,
  /Run selected CanvasKit readiness gate[\s\S]*?scripts\/renderer_baseline\.py[\s\S]*?--readiness-only/,
  'render-diff CI should run the selected CanvasKit readiness gate',
);
assert.ok(
  renderDiffWorkflowSource.includes("RHWP_CHROMIUM_BUILD_ID: '1660786'")
    && renderDiffWorkflowSource.includes('chromium@${RHWP_CHROMIUM_BUILD_ID}'),
  'render-diff CI should pin the Chromium revision used by hard visual gates',
);
assert.ok(
  !renderDiffWorkflowSource.includes('chromium@latest'),
  'hard visual gates must not follow a moving Chromium revision',
);
assert.ok(
  rendererBaselineSource.includes('chromiumBuildId'),
  'CanvasKit readiness artifacts should identify the pinned Chromium snapshot',
);

console.log('renderer backend contract guard passed');
