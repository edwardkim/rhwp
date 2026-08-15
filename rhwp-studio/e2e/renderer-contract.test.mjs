import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { PNG } from 'pngjs';
import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';

import { comparePngBuffers } from './helpers.mjs';
import { inspectCanvasKitRuntimeImageFailures } from './renderer-baseline-contract.mjs';

const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(studioRoot, '..');
const canvaskitPath = path.join(studioRoot, 'src/view/canvaskit-renderer.ts');
const canvaskitDirectory = path.join(studioRoot, 'src/view/canvaskit');
const canvaskitDiagnosticsPath = path.join(canvaskitDirectory, 'diagnostics.ts');
const layerTypesPath = path.join(studioRoot, 'src/core/types.ts');
const textIrV2DocPath = path.join(repoRoot, 'docs/text-ir-v2.md');
const canvaskitParityPlanDocPath = path.join(repoRoot, 'docs/canvaskit-parity-implementation.md');
const rendererBaselinePath = path.join(studioRoot, 'e2e/renderer-baseline.mjs');
const rendererBaselineNativeDiffPath = path.join(
  studioRoot,
  'e2e/renderer-baseline-native-diff.mjs',
);
const rendererBaselineDriverPath = path.join(repoRoot, 'scripts/renderer_baseline.py');
const rendererBaselineManifestPath = path.join(repoRoot, 'scripts/renderer_baseline_manifest.json');
const helpersPath = path.join(studioRoot, 'e2e/helpers.mjs');
const mainPath = path.join(studioRoot, 'src/main.ts');
const embedRpcRouterPath = path.join(studioRoot, 'src/embed/rpc-router.ts');
const renderBackendPath = path.join(studioRoot, 'src/view/render-backend.ts');
const rendererSessionPath = path.join(studioRoot, 'src/view/renderer-session.ts');
const pageRendererPath = path.join(studioRoot, 'src/view/page-renderer.ts');
const canvasViewPath = path.join(studioRoot, 'src/view/canvas-view.ts');
const vscodeViewerPath = path.join(repoRoot, 'rhwp-vscode/src/webview/viewer.ts');
const vscodeWebpackPath = path.join(repoRoot, 'rhwp-vscode/webpack.config.js');
const renderDiffWorkflowPath = path.join(repoRoot, '.github/workflows/render-diff.yml');
const fullRendererSweepWorkflowPath = path.join(
  repoRoot,
  '.github/workflows/full-renderer-sweep.yml',
);

const canvaskitSource = fs.readFileSync(canvaskitPath, 'utf8');
const canvaskitDiagnosticsSource = fs.readFileSync(canvaskitDiagnosticsPath, 'utf8');
const layerTypesSource = fs.readFileSync(layerTypesPath, 'utf8');
const textIrV2DocSource = fs.readFileSync(textIrV2DocPath, 'utf8');
const canvaskitParityPlanDocSource = fs.readFileSync(canvaskitParityPlanDocPath, 'utf8');
const rendererBaselineSource = fs.readFileSync(rendererBaselinePath, 'utf8');
const rendererBaselineNativeDiffSource = fs.readFileSync(rendererBaselineNativeDiffPath, 'utf8');
const rendererBaselineDriverSource = fs.readFileSync(rendererBaselineDriverPath, 'utf8');
const rendererBaselineManifest = JSON.parse(fs.readFileSync(rendererBaselineManifestPath, 'utf8'));
const helpersSource = fs.readFileSync(helpersPath, 'utf8');
const mainSource = fs.readFileSync(mainPath, 'utf8');
const embedRpcRouterSource = fs.readFileSync(embedRpcRouterPath, 'utf8');
const renderBackendSource = fs.readFileSync(renderBackendPath, 'utf8');
const rendererSessionSource = fs.readFileSync(rendererSessionPath, 'utf8');
const pageRendererSource = fs.readFileSync(pageRendererPath, 'utf8');
const canvasViewSource = fs.readFileSync(canvasViewPath, 'utf8');
const vscodeViewerSource = fs.readFileSync(vscodeViewerPath, 'utf8');
const vscodeWebpackSource = fs.readFileSync(vscodeWebpackPath, 'utf8');
const renderDiffWorkflowSource = fs.readFileSync(renderDiffWorkflowPath, 'utf8');
const fullRendererSweepWorkflowSource = fs.readFileSync(fullRendererSweepWorkflowPath, 'utf8');
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
for (const sample of rendererBaselineManifest.samples.filter(
  (entry) => entry.canvaskitReadinessGate === true,
)) {
  assert.deepEqual(
    Object.keys(sample.canvaskitPerformanceBudget ?? {}).sort(),
    [
      'maxColdDocumentLoadAndInitialRenderMs',
      'maxImageCachePixels',
      'maxWarmRendererDurationMs',
      'maxWarmReplayMs',
    ],
    `readiness sample ${sample.id} should define the complete cold/warm performance budget`,
  );
}
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
  /this\.renderLeaf\(canvas, node, profile, replayPlane, activeLayer\);/,
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
  /if \(!this\.lastRenderCompleted\) readinessBlockers\.push\('renderNotCompleted'\);[\s\S]*?if \(this\.lastRenderError !== null\) readinessBlockers\.push\('renderError'\);[\s\S]*?if \(lastUnexpectedUnsupportedOps\.length > 0\) readinessBlockers\.push\('unexpectedUnsupportedOps'\);[\s\S]*?if \(this\.currentImageFailures\.size > 0\) readinessBlockers\.push\('imageReplayFailure'\);[\s\S]*?passesRuntimeReadinessGate: readinessBlockers\.length === 0/,
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
  vscodeViewerSource,
  /new RendererSession\([\s\S]*?backend: "canvas2d"[\s\S]*?import\("@\/view\/canvaskit-renderer"\)/,
  'VS Code should keep the compatibility default while retaining lazy CanvasKit infrastructure',
);
requireSnippet(
  vscodeViewerSource,
  /async function loadDocument\([\s\S]*?rendererSession\.beginDocument\(digest\)[\s\S]*?await rendererSession\.resolve\([\s\S]*?applyRendererSelection\(selection\)[\s\S]*?buildPageLayout\(\)/,
  'VS Code should resolve one backend before laying out and rendering the document',
);
requireSnippet(
  vscodeViewerSource,
  /resolveCanvasKitFontPlan[\s\S]*?transformCanvasKitPreflight[\s\S]*?withCanvasKitSurfaceBlockers[\s\S]*?prepareCanvasKitDocument[\s\S]*?prepareBundledFonts/,
  'VS Code auto selection should validate and prepare document fonts before first replay',
);
requireSnippet(
  vscodeViewerSource,
  /updateVisiblePages\(\);[\s\S]*?await Promise\.resolve\(\);[\s\S]*?const activeSelection = rendererSelection \?\? selection;[\s\S]*?renderer: activeSelection\.diagnostics/,
  'VS Code loaded diagnostics should report a first-render fallback instead of stale CanvasKit selection',
);
requireSnippet(
  vscodeViewerSource,
  /function scheduleRendererFallback\([\s\S]*?fallbackFromResourceFailure[\s\S]*?fallbackFromRuntimeFailure[\s\S]*?queueMicrotask[\s\S]*?releasePage\(pageNum\)[\s\S]*?updateVisiblePages\(\)/,
  'VS Code CanvasKit failures should trigger one whole-document Canvas2D replay',
);
requireSnippet(
  vscodeWebpackSource,
  /resourceQuery: \/url\/[\s\S]*?type: "asset\/resource"/,
  'VS Code should emit the lazily loaded CanvasKit WASM asset',
);
requireSnippet(
  vscodeWebpackSource,
  /path: path\.resolve\(__dirname, "dist", "webview"\)[\s\S]*?clean: true/,
  'VS Code webview builds should remove obsolete lazy chunks and assets',
);
requireSnippet(
  mainSource,
  /async getRendererDiagnostics\(pageIndex\)[\s\S]*?getRendererSessionDiagnostics\(\)[\s\S]*?request: rendererRuntimeRequest[\s\S]*?initialized: rendererInitialized[\s\S]*?initializationError:[\s\S]*?effectiveBackend: selection\?\.effectiveBackend[\s\S]*?backendFallbackReason:[\s\S]*?selection,[\s\S]*?getCanvasKitRenderDiagnostics\(pageIndex\)/,
  'Studio iframe API should expose backend selection and page-scoped renderer diagnostics',
);
requireSnippet(
  mainSource,
  /renderBackendRequest\.backend === 'auto'[\s\S]*?backend: 'canvas2d'[\s\S]*?backend: diagnosticsBackendRequest/,
  'Studio renderer diagnostics v1 should preserve its legacy request backend enum',
);
requireSnippet(
  rendererSessionSource,
  /beginDocument\(documentDigest: string \| null\)[\s\S]*?documentRevision \+= 1;[\s\S]*?resourceGeneration \+= 1;[\s\S]*?invalidateDocument\(\)[\s\S]*?decisionKey\(\)/,
  'RendererSession should invalidate document-scoped decisions by revision and resource generation',
);
requireSnippet(
  rendererSessionSource,
  /pinAutoMutationRevision\(\)[\s\S]*?invalidateDocument\(\)[\s\S]*?'autoRevisionPending'[\s\S]*?'canvaskitRevisionInvalidated'/,
  'Auto edits should pin an invalidated revision to Canvas2D without a synchronous document rescan',
);
requireSnippet(
  canvasViewSource,
  /scheduleAutoRendererReselection\(\)[\s\S]*?setTimeout\([\s\S]*?selectNextDocumentRevision\(\)\.then[\s\S]*?AUTO_RENDERER_RESELECTION_DELAY_MS/,
  'Auto edit revisions should coalesce one bounded capability re-evaluation after input settles',
);
requireSnippet(
  canvasViewSource,
  /prepareDocumentLoad\(\)[\s\S]*?rendererSelectionEpoch \+= 1;[\s\S]*?rendererSession\.beginDocument[\s\S]*?this\.reset\(\)/,
  'Document replacement should synchronously detach the previous renderer decision and canvases',
);
requireSnippet(
  helpersSource,
  /await window\.__canvasView\?\.loadDocument\?\.\(\)/,
  'Cold renderer timing should include preflight, lazy CanvasKit initialization, and initial replay',
);
requireSnippet(
  rendererSessionSource,
  /dispose\(\): void[\s\S]*?this\.canvaskitRenderer = null;[\s\S]*?renderer\?\.dispose\(\)/,
  'RendererSession should own and dispose the shared CanvasKit renderer',
);
assert.doesNotMatch(
  extractMethodBody(pageRendererSource, 'dispose'),
  /canvaskitRenderer\?\.dispose/,
  'PageRenderer must not dispose the RendererSession-owned CanvasKit instance',
);
requireSnippet(
  rendererSessionSource,
  /this\.request\.backend === 'auto'[\s\S]*?this\.readPreflight\(source\)[\s\S]*?!preflight\.complete \|\| preflight\.status === 'incomplete'[\s\S]*?!preflight\.eligible \|\| preflight\.status !== 'eligible'[\s\S]*?ensureCanvasKitRenderer\(\)/,
  'Auto backend selection should fail closed before lazily initializing CanvasKit',
);
requireSnippet(
  rendererSessionSource,
  /readPreflight\(source: RendererPreflightSource\)[\s\S]*?source\.getCanvasKitDocumentPreflight\([\s\S]*?preflight\.schemaVersion !== 1[\s\S]*?preflight\.mode !== this\.canvaskitMode\.mode[\s\S]*?preflight\.profile !== this\.renderProfile/,
  'RendererSession should validate the bounded document preflight contract',
);
requireSnippet(
  rendererSessionSource,
  /transformCanvasKitPreflight[\s\S]*?prepareCanvasKitDocument[\s\S]*?await this\.options\.prepareCanvasKitDocument\(renderer, preflight\)[\s\S]*?'canvaskitResourcePreparationFailed'/,
  'RendererSession should apply surface capability blockers before initialization and prepare resources before selection',
);
requireSnippet(
  rendererSessionSource,
  /invalidateDocument\(options:[\s\S]*?resetResources[\s\S]*?this\.canvaskitRenderer\?\.resetDocumentResources\(\)[\s\S]*?this\.decisionKey\(\) !== key[\s\S]*?'superseded'/,
  'RendererSession should cancel stale resource preparation and reset native resources on document mutations',
);
requireSnippet(
  canvaskitSource,
  /catch \(error\) \{[\s\S]*?!this\.disposed && generation === this\.documentGeneration[\s\S]*?this\.bundledTypefaceLoadFailures\.add\(source\.url\)/,
  'Document replacement cancellation must not poison the next CanvasKit font preparation attempt',
);
requireSnippet(
  rendererSessionSource,
  /fallbackFromResourceFailure\([\s\S]*?expectedDecisionKey: string[\s\S]*?'canvaskitResourcePreparationFailed'[\s\S]*?fallbackForCurrentDecision\([\s\S]*?'canvas2d'/,
  'CanvasKit resource preparation failures should pin the document to Canvas2D',
);
requireSnippet(
  rendererSessionSource,
  /fallbackFromRuntimeFailure\([\s\S]*?if \(!this\.isAutoRequest\(\)\) return null;[\s\S]*?'canvaskitRuntimeFailed'/,
  'Auto CanvasKit runtime failures should pin the document to Canvas2D without changing explicit requests',
);
requireSnippet(
  canvasViewSource,
  /activeRendererDecisionKey[\s\S]*?getCanvasKitRenderDiagnostics\(pageIdx\)[\s\S]*?!canvaskitDiagnostics\.passesRuntimeReadinessGate[\s\S]*?rendererSession\.isAutoRequest\(\)[\s\S]*?readinessBlockers\.join[\s\S]*?scheduleCanvasKitFallback\([\s\S]*?'runtime'[\s\S]*?fallbackFromRuntimeFailure\(error, expectedDecisionKey\)/,
  'CanvasView should promote failed auto CanvasKit readiness through the current document decision only',
);
requireSnippet(
  pageRendererSource,
  /invalidateDocumentRevision\(\)[\s\S]*?releaseAllPageDiagnostics\(\);[\s\S]*?layerSummaryCache\.clear\(\)/,
  'PageRenderer should drop revision-scoped diagnostics and layer summaries before replaying a new decision',
);
requireSnippet(
  renderBackendSource,
  /if \(!normalized\) return \{ backend: 'canvas2d', source: 'default' \}/,
  'Browser rendering should preserve Canvas2D unless auto is explicitly requested',
);
requireSnippet(
  rendererBaselineSource,
  /if \(options\.readinessOnly\) \{[\s\S]*?\?renderer=auto&canvaskitMode=default&renderProfile=[\s\S]*?runtime\.request\?\.backend\?\.backend !== 'canvas2d'[\s\S]*?runtime\.selection\?\.request\?\.backend !== 'auto'[\s\S]*?runtime\.selection\?\.request\?\.source !== 'url'[\s\S]*?runtime\.selection\?\.selectionReason !== 'autoEligible'[\s\S]*?autoPreflightNotEligible/,
  'Selected readiness should measure an explicit auto candidate and its preflight decision',
);
assert.doesNotMatch(
  mainSource,
  /viewOption:showParagraphMarks/,
  'Automatic selection should permit directly replayable text marks',
);
assert.match(
  mainSource,
  /viewOption:showControlCodes/,
  'Automatic selection should reject structural control markers until they have explicit ops',
);
requireSnippet(
  embedRpcRouterSource,
  /case 'getRendererDiagnostics':[\s\S]*?params\.page \?\? 0[\s\S]*?Number\.isSafeInteger\(page\)[\s\S]*?page must be a non-negative safe integer[\s\S]*?handlers\.getRendererDiagnostics\(page as number\)/,
  'Embed router should preserve renderer diagnostics and reject invalid page indexes',
);
assert.doesNotMatch(
  mainSource,
  /effectiveBackend: canvasView\?\.getRenderBackend\(\) \?\? 'canvas2d'/,
  'Studio diagnostics must not report Canvas2D when no renderer initialized',
);
requireSnippet(
  mainSource,
  /new RendererSession\([\s\S]*?async \(mode, surface\) => \{[\s\S]*?import\('\@\/view\/canvaskit-renderer'\)[\s\S]*?CanvasKitLayerRenderer\.create\(mode, surface,[\s\S]*?requirePreparedFontFamilies:[\s\S]*?transformCanvasKitPreflight[\s\S]*?prepareCanvasKitDocument[\s\S]*?loadStoredLocalFonts\(\)[\s\S]*?prepareLocalFonts\(report\.requiredFontFamilies\)[\s\S]*?prepareBundledFonts/,
  'Studio should prepare stored local faces and bundled fallback before first CanvasKit replay',
);
requireSnippet(
  canvaskitSource,
  /prepareBundledFonts\([\s\S]*?MAX_BUNDLED_FONT_BYTES[\s\S]*?bundledTypefaceAliases\.set[\s\S]*?CanvasKit font family가 준비되지 않았습니다/,
  'CanvasKit should bound bundled font parsing and reject unprepared explicit families',
);
requireSnippet(
  canvaskitSource,
  /private findPreparedTypeface\([\s\S]*?const local =[\s\S]*?const bundled =[\s\S]*?if \(local\) return local;[\s\S]*?if \(bundled\) return bundled;/,
  'CanvasKit should prefer an exact prepared local face over its bundled fallback alias',
);
requireSnippet(
  canvaskitSource,
  /requiresShapingManager[\s\S]*?OLD_HANGUL_FONT_FAMILY[\s\S]*?!prepared\.fontManager[\s\S]*?shaping font source 준비 실패/,
  'Old-Hangul font preparation should require a shaping-capable font manager',
);
assert.doesNotMatch(
  renderBackendSource,
  /rhwp\.renderBackend|persistRenderBackend/,
  'CanvasKit backend opt-in should stay URL-only',
);

const directReplayOps = [
  ['charOverlap', 'renderCharOverlap'],
  ['ellipse', 'renderEllipse'],
  ['equation', 'renderEquation'],
  ['footnoteMarker', 'renderTextRun'],
  ['formObject', 'renderFormObject'],
  ['glyphRun', 'renderGlyphRun'],
  ['image', 'renderImage'],
  ['line', 'renderLine'],
  ['pageBackground', 'renderPageBackground'],
  ['path', 'renderPath'],
  ['placeholder', 'renderPlaceholder'],
  ['rectangle', 'renderRectangle'],
  ['tabLeader', 'renderTabLeader'],
  ['textControlMark', 'renderTextControlMark'],
  ['textDecoration', 'renderTextDecoration'],
  ['textRun', 'renderTextRun'],
];
const textRunFallbackOps = [];
const objectFragmentFallbackOps = [
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
  'equation:unsupportedDirectReplay',
  'rawSvg:unsupportedDirectReplay',
  'textRunFont',
  'image:dataMissing',
  'image:invalidBounds',
  'image:dimensionUnavailable',
  'image:tileLimit',
  'glyphOutline:unsupportedColorGlyph',
  'imageEffect:grayScale',
  'textRun:scriptTextRequiresShaping',
]) {
  assert.ok(
    expectedUnsupportedSetBody.includes(`'${expectedUnsupportedToken}'`),
    `CanvasKit expected unsupported set should include ${expectedUnsupportedToken}`,
  );
}
for (const directTextVisualToken of [
  'charOverlap',
  'tabLeader',
  'textControlMark',
  'textDecoration',
  'textRun:glyphMapping',
  'textRun:textDecoration',
  'textRun:verticalText',
  'textRun:outlineTextEffect',
  'textRun:shadowTextEffect',
  'textRun:embossTextEffect',
  'textRun:engraveTextEffect',
  'textRun:shadeTextEffect',
  'textRun:ratioTextEffect',
]) {
  assert.equal(
    expectedUnsupportedSetBody.includes(`'${directTextVisualToken}'`),
    false,
    `CanvasKit direct text visual should not stay on the expected-unsupported allowlist: ${directTextVisualToken}`,
  );
}
assert.ok(
  !expectedUnsupportedSetBody.includes("'equation:invalidLayout'"),
  'malformed semantic equation layouts should block CanvasKit readiness',
);
assert.ok(
  !expectedUnsupportedSetBody.includes("'renderPage'"),
  'CanvasKit render failures should stay unexpected readiness diagnostics',
);
assert.ok(
  !expectedUnsupportedSetBody.includes("'unknown'"),
  'CanvasKit unknown op diagnostics should stay unexpected readiness diagnostics',
);
assert.ok(
  canvaskitSource.includes('MAX_FONT_SUBSTITUTION_DIAGNOSTICS = 4096')
    && canvaskitSource.includes('private readonly currentFontSubstitutions = new Map')
    && canvaskitSource.includes('unregisteredFontFallbacks: fontSubstitutions.filter(')
    && canvaskitSource.includes("candidateFontSources.push('missingGlyphDefault')")
    && canvaskitSource.includes("candidateFontSources.push('missingGlyphSymbol')")
    && canvaskitSource.includes("source: 'oldHangul'")
    && canvaskitSource.includes('this.currentFontSubstitutions.clear()'),
  'CanvasKit font substitutions should be bounded, structured, and reset with replay state',
);
assert.ok(
  !expectedUnsupportedSetBody.includes("'glyphOutline:replayInvariant'"),
  'CanvasKit replay invariants should stay unexpected readiness diagnostics',
);

const glyphOutlineCaseBody = extractSwitchCaseClusterBody(renderOpBody, 'glyphOutline');
requireSnippet(
  glyphOutlineCaseBody,
  /const status = glyphOutlinePayloadStatus\(op,[\s\S]*?allowBitmapGlyph: true[\s\S]*?allowSvgGlyph: true[\s\S]*?if \(status\.supported && this\.glyphOutlineVariantReplayable\(op\)\) \{[\s\S]*?this\.renderGlyphOutline\(canvas, op\);\s*return;\s*\}[\s\S]*?this\.unsupportedOps\.add\(status\.reason \? `glyphOutline:\$\{status\.reason\}` : 'glyphOutline'\);\s*return;/,
  'glyphOutline should stay guarded by payload status before direct replay',
);

const renderRectangleBody = extractMethodBody(canvaskitSource, 'renderRectangle');
const renderEllipseBody = extractMethodBody(canvaskitSource, 'renderEllipse');
const renderEquationBody = extractMethodBody(canvaskitSource, 'renderEquation');
const renderEquationBoxBody = extractMethodBody(canvaskitSource, 'renderEquationBox');
const renderPathBody = extractMethodBody(canvaskitSource, 'renderPath');
const renderLineBody = extractMethodBody(canvaskitSource, 'renderLine');
const drawStrokeWithDashBody = extractMethodBody(canvaskitSource, 'drawStrokeWithDash');
const renderFormObjectBody = extractMethodBody(canvaskitSource, 'renderFormObject');
const renderPlaceholderBody = extractMethodBody(canvaskitSource, 'renderPlaceholder');
const renderTextRunBody = extractMethodBody(canvaskitSource, 'renderTextRun');
const renderShapedScriptTextBody = extractMethodBody(canvaskitSource, 'renderShapedScriptText');
const renderGlyphRunBody = extractMethodBody(canvaskitSource, 'renderGlyphRun');
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
let glyphOutlinePayloadResourceKeyRuntime;
try {
  ({ CanvasKitLayerRenderer: CanvasKitLayerRendererRuntime } = await vite.ssrLoadModule(
    '/src/view/canvaskit-renderer.ts',
  ));
  ({ glyphOutlinePayloadResourceKey: glyphOutlinePayloadResourceKeyRuntime }
    = await vite.ssrLoadModule('/src/view/glyph-outline-payload-status.ts'));
} finally {
  await vite.close();
}

function runExecutableTextReplay(op, {
  glyphIds,
  fallbackGlyphIds,
  symbolGlyphIds,
  usePreparedTypeface = false,
  drawGlyphsError,
  drawParagraphError,
  fillPaintErrorAt = null,
  requirePreparedFontFamilies = false,
  shapedTextAvailable = true,
} = {}) {
  const events = [];
  const unsupportedOps = new Set();
  const replayText = op.displayText ?? op.text;
  const resolvedGlyphIds = glyphIds
    ?? Array.from({ length: Array.from(replayText).length }, (_, index) => index + 1);

  class FakeFont {
    constructor(typeface, size) {
      this.typeface = typeface;
      events.push({ type: 'font.create', face: typeface?.face ?? 'default', size });
    }

    getGlyphIDs(text, count) {
      events.push({ type: 'font.getGlyphIDs', text, count });
      return Uint16Array.from(
        this.typeface?.face === 'symbol' && symbolGlyphIds
          ? symbolGlyphIds
          : this.typeface?.face === 'fallback' && fallbackGlyphIds
            ? fallbackGlyphIds
            : resolvedGlyphIds,
      );
    }

    getGlyphWidths(ids) {
      return Array.from(ids, () => 8);
    }

    getGlyphBounds(ids) {
      return Float32Array.from(Array.from(ids).flatMap(() => [-4, -12, 4, 2]));
    }

    setScaleX(scale) {
      events.push({ type: 'font.scaleX', scale });
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

  const makePaint = (kind, color, width = null) => ({
    kind,
    color,
    width,
    setAntiAlias(value) {
      events.push({ type: 'paint.antiAlias', value });
    },
    delete() {
      events.push({ type: 'paint.delete', kind, color, width });
    },
  });
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
    translate(x, y) {
      events.push({ type: 'canvas.translate', x, y });
    },
    drawGlyphs(ids, positions, x, y, _font, paint) {
      events.push({
        type: 'canvas.drawGlyphs',
        glyphIds: Array.from(ids),
        positions: Array.from(positions),
        x,
        y,
        paint: paint ? { kind: paint.kind, color: paint.color, width: paint.width } : null,
      });
      if (drawGlyphsError) throw drawGlyphsError;
    },
    drawText(text, x, y, paint) {
      events.push({
        type: 'canvas.drawText',
        text,
        x,
        y,
        paint: paint ? { kind: paint.kind, color: paint.color, width: paint.width } : null,
      });
    },
    drawRect(rect, paint) {
      events.push({
        type: 'canvas.drawRect',
        rect,
        paint: paint ? { kind: paint.kind, color: paint.color, width: paint.width } : null,
      });
    },
    drawParagraph(_paragraph, x, y) {
      events.push({ type: 'canvas.drawParagraph', x, y });
      if (drawParagraphError) throw drawParagraphError;
    },
    restore() {
      events.push({ type: 'canvas.restore' });
    },
  };
  const fallbackTypeface = { face: 'fallback' };
  const symbolTypeface = symbolGlyphIds ? { face: 'symbol' } : null;
  const renderer = new CanvasKitLayerRendererRuntime({
    Font: FakeFont,
    ParagraphStyle: FakeParagraphStyle,
    ParagraphBuilder: {
      Make(style, fontManager) {
        events.push({ type: 'paragraphBuilder.make', style, fontManager });
        return paragraphBuilder;
      },
    },
    XYWHRect(x, y, width, height) {
      return { x, y, width, height };
    },
  }, 'default', {}, fallbackTypeface, symbolTypeface, shapedTextAvailable ? {} : null,
  'Noto Sans KR', 'fonts/NotoSansKR-Regular.woff2', requirePreparedFontFamilies);
  renderer.unsupportedOps = unsupportedOps;
  if (usePreparedTypeface) {
    renderer.findPreparedTypeface = (fontFamily) => ({
      typeface: fontFamily === 'Source Han Serif K Old Hangul'
        ? null
        : { face: 'primary' },
      fontManager: shapedTextAvailable ? {} : null,
      fontFamily,
    });
  }
  const recordTextRunCoverageGaps = renderer.recordTextRunCoverageGaps.bind(renderer);
  renderer.recordTextRunCoverageGaps = (textRun, codePoints) => {
    events.push({ type: 'coverage.record' });
    return recordTextRunCoverageGaps(textRun, codePoints);
  };
  let fillPaintCreates = 0;
  renderer.makeFillPaint = (color) => {
    fillPaintCreates += 1;
    if (fillPaintCreates === fillPaintErrorAt) throw new Error('synthetic fill paint failure');
    events.push({ type: 'paint.create', kind: 'fill', color });
    return makePaint('fill', color);
  };
  renderer.makeStrokePaint = (color, width) => {
    events.push({ type: 'paint.create', kind: 'stroke', color, width });
    return makePaint('stroke', color, width);
  };
  renderer.color = (color) => color;

  let error = null;
  try {
    renderer.renderTextRun(canvas, op);
  } catch (caught) {
    error = caught;
  }
  return { error, events, unsupportedOps, diagnostics: renderer.diagnostics(), renderer };
}

function runExecutableStrokeDashReplay() {
  const events = [];
  class FakePaint {
    setAntiAlias() {}
    setStyle() {}
    setColor(color) { this.color = color; }
    setStrokeWidth(width) { this.width = width; }
    setPathEffect() { events.push({ type: 'paint.pathEffect' }); }
    delete() { events.push({ type: 'paint.delete' }); }
  }
  class FakePath {
    moveTo() {}
    lineTo() {}
    delete() { events.push({ type: 'path.delete' }); }
  }
  const renderer = new CanvasKitLayerRendererRuntime({
    Paint: FakePaint,
    Path: FakePath,
    PaintStyle: { Fill: 0, Stroke: 1 },
    PathEffect: {
      MakeDash(intervals, phase) {
        events.push({ type: 'pathEffect.create', intervals: [...intervals], phase });
        return { delete() { events.push({ type: 'pathEffect.delete' }); } };
      },
    },
    Color: (r, g, b, a) => [r, g, b, a],
    XYWHRect: (x, y, width, height) => ({ x, y, width, height }),
  }, 'default', {}, {});
  const canvas = {
    drawLine(_x1, _y1, _x2, _y2, paint) {
      events.push({ type: 'canvas.drawLine', color: paint.color, width: paint.width });
    },
    drawRect(_rect, paint) {
      events.push({ type: 'canvas.drawRect', color: paint.color, width: paint.width });
    },
    drawPath(_path, paint) {
      events.push({ type: 'canvas.drawPath', color: paint.color, width: paint.width });
    },
  };
  renderer.unsupportedOps = new Set();
  renderer.renderLine(canvas, {
    type: 'line', x1: 0, y1: 0, x2: 10, y2: 0, style: { dash: 'dash' },
  });
  renderer.renderRectangle(canvas, {
    type: 'rectangle',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    style: { strokeColor: '#000000', strokeWidth: 1, strokeDash: 'dot' },
  });
  renderer.renderPath(canvas, {
    type: 'path',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    commands: [{ type: 'moveTo', x: 0, y: 0 }, { type: 'lineTo', x: 10, y: 10 }],
    style: { fillColor: null, strokeWidth: 0 },
    lineStyle: { color: '#123456', width: 2, dash: 'dashDot' },
  });
  renderer.renderLine(canvas, {
    type: 'line', x1: 0, y1: 0, x2: 10, y2: 0, style: { dash: 'dashDotDot' },
  });
  const drawCountBeforeInvalid = events.filter(event => event.type.startsWith('canvas.draw')).length;
  renderer.renderLine(canvas, {
    type: 'line', x1: 0, y1: 0, x2: 10, y2: 0, style: { dash: 'zigzag' },
  });

  return { events, renderer, drawCountBeforeInvalid };
}

function runExecutableTextSpecialReplay() {
  const events = [];
  class FakePaint {
    setAntiAlias() {}
    setStyle() {}
    setColor() {}
    setStrokeWidth() {}
    setStrokeCap() {}
    setPathEffect() { events.push({ type: 'paint.pathEffect' }); }
    delete() { events.push({ type: 'paint.delete' }); }
  }
  class FakeFont {
    constructor(_typeface, size) { this.size = size; }
    getGlyphIDs(text) { return Uint16Array.from(Array.from(text), (_, index) => index + 1); }
    getGlyphWidths(ids) { return Array.from(ids, () => this.size * 0.5); }
    setScaleX(scale) { events.push({ type: 'font.scaleX', scale }); }
    delete() { events.push({ type: 'font.delete' }); }
  }
  const canvasKit = {
    Font: FakeFont,
    Paint: FakePaint,
    PaintStyle: { Fill: 0, Stroke: 1 },
    StrokeCap: { Round: 0 },
    PathEffect: {
      MakeDash(dash) {
        events.push({ type: 'pathEffect.create', dash: [...dash] });
        return { delete() { events.push({ type: 'pathEffect.delete' }); } };
      },
    },
    Color: (r, g, b, a) => [r, g, b, a],
    XYWHRect: (x, y, width, height) => ({ x, y, width, height }),
  };
  const canvas = {
    save() { events.push({ type: 'canvas.save' }); },
    restore() { events.push({ type: 'canvas.restore' }); },
    translate(x, y) { events.push({ type: 'canvas.translate', x, y }); },
    rotate(rotation) { events.push({ type: 'canvas.rotate', rotation }); },
    drawOval(rect) { events.push({ type: 'canvas.drawOval', rect }); },
    drawRect(rect) { events.push({ type: 'canvas.drawRect', rect }); },
    drawText(text, x, y) { events.push({ type: 'canvas.drawText', text, x, y }); },
    drawLine(x1, y1, x2, y2) { events.push({ type: 'canvas.drawLine', x1, y1, x2, y2 }); },
    drawCircle(x, y, radius) { events.push({ type: 'canvas.drawCircle', x, y, radius }); },
  };
  const renderer = new CanvasKitLayerRendererRuntime(
    canvasKit,
    'default',
    {},
    { face: 'fallback' },
    null,
    null,
    'Noto Sans KR',
  );
  renderer.currentShowParagraphMarks = true;
  renderer.currentShowControlCodes = true;
  const incompleteOldHangulAlias = { typeface: { face: 'old-hangul-incomplete' }, fontManager: null };
  renderer.bundledTypefaceAliases.set('source han serif k old hangul', incompleteOldHangulAlias);
  const rejectedOldHangulAlias = renderer.findPreparedTypeface('Source Han Serif K Old Hangul');
  const oldHangulAlias = {
    typeface: { face: 'old-hangul-alias' },
    fontManager: { family: 'old-hangul-manager' },
  };
  renderer.bundledTypefaceAliases.set('source han serif k old hangul', oldHangulAlias);
  const resolvedOldHangulAlias = renderer.findPreparedTypeface('Source Han Serif K Old Hangul');

  renderer.renderOp(canvas, {
    type: 'charOverlap',
    bbox: { x: 10, y: 20, width: 16, height: 16 },
    text: '①',
    baseline: 12,
    rotation: 0,
    isVertical: false,
    style: { fontSize: 16, color: '#112233' },
    positions: [0, 16],
    positionsComplete: true,
    charOverlap: { borderType: 1, innerCharSize: 80 },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'charOverlap',
    bbox: { x: 30, y: 20, width: 16, height: 16 },
    text: '\u{F0289}\u{F0294}',
    baseline: 12,
    rotation: 0,
    isVertical: false,
    style: { fontSize: 16, color: '#112233' },
    positions: [0, 8, 16],
    positionsComplete: true,
    charOverlap: { borderType: 1, innerCharSize: 80 },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'textControlMark',
    bbox: { x: 10, y: 20, width: 40, height: 16 },
    fieldMarker: 'none',
    isParaEnd: true,
    isLineBreakEnd: false,
    baseline: 12,
    rotation: 0,
    isVertical: false,
    marks: [
      { kind: 'space', text: '∨', x: 8, y: 0, fontSize: 8 },
      { kind: 'paragraphEnd', text: '↵', x: 40, y: 0, fontSize: 16 },
    ],
    marksComplete: true,
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'tabLeader',
    bbox: { x: 10, y: 20, width: 40, height: 16 },
    leaders: [{ startX: 4, endX: 30, fillType: 2 }],
    color: '#000000',
    fontSize: 16,
    baseline: 12,
    rotation: 0,
    isVertical: false,
    leadersComplete: true,
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'textDecoration',
    bbox: { x: 10, y: 20, width: 40, height: 16 },
    decoration: {
      kind: 'emphasisDot',
      baseline: 12,
      rotation: 0,
      isVertical: false,
      fontSize: 16,
      ratio: 1,
      color: '#000000',
      shape: 0,
      underline: 'none',
      emphasisDot: 1,
      positions: [0, 12],
      positionsComplete: true,
    },
  }, 'screen');
  const beforeMirror = events.length;
  renderer.renderTextRun(canvas, {
    type: 'textRun',
    bbox: { x: 10, y: 20, width: 16, height: 16 },
    text: '①',
    style: { fontSize: 16 },
    charOverlap: { borderType: 1, innerCharSize: 80 },
    legacyVisuals: { charOverlap: 'mirror' },
  });
  const mirrorEvents = events.slice(beforeMirror);
  const beforeMalformed = events.length;
  renderer.renderOp(canvas, {
    type: 'textDecoration',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    decoration: {
      kind: 'underline',
      baseline: 8,
      rotation: 0,
      isVertical: false,
      fontSize: 10,
      ratio: 1,
      color: '#000000',
      shape: 0,
      underline: 'center',
      emphasisDot: 0,
      positions: [0, 10],
      positionsComplete: true,
    },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'charOverlap',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    text: 'A'.repeat(4097),
    baseline: 8,
    rotation: 0,
    isVertical: false,
    style: { fontSize: 10 },
    positions: [],
    positionsComplete: true,
    charOverlap: { borderType: 1, innerCharSize: 100 },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'charOverlap',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    text: 'A',
    baseline: 8,
    rotation: 0,
    isVertical: false,
    style: { fontSize: 10 },
    positions: [0, 10],
    positionsComplete: true,
    charOverlap: { borderType: 5, innerCharSize: 100 },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'charOverlap',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    text: 'A',
    baseline: 8,
    rotation: 0,
    isVertical: false,
    style: { fontSize: 10 },
    positions: [],
    positionsComplete: true,
    charOverlap: { borderType: 1, innerCharSize: 100 },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'textControlMark',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    baseline: 8,
    rotation: 0,
    isVertical: false,
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'tabLeader',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    fontSize: 10,
    baseline: 8,
    rotation: 0,
    isVertical: false,
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'charOverlap',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    text: 'A',
    baseline: 8,
    rotation: 0,
    isVertical: false,
    style: { fontSize: 10 },
    positions: [0, 10],
    charOverlap: { borderType: 1, innerCharSize: 100 },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'textControlMark',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    fieldMarker: 'none',
    isParaEnd: false,
    isLineBreakEnd: false,
    baseline: 8,
    rotation: 0,
    isVertical: false,
    marks: [],
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'tabLeader',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    leaders: [{ startX: 1, endX: 8, fillType: 1 }],
    color: '#000000',
    fontSize: 10,
    baseline: 8,
    rotation: 0,
    isVertical: false,
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'textDecoration',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    decoration: {
      kind: 'underline',
      baseline: 8,
      rotation: 0,
      isVertical: false,
      fontSize: 10,
      ratio: 1,
      color: '#000000',
      shape: 0,
      underline: 'bottom',
      emphasisDot: 0,
      positions: [0, 10],
    },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'textDecoration',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'textDecoration',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    decoration: {
      kind: 'underline',
      baseline: 8,
      rotation: 0,
      isVertical: false,
      fontSize: 10,
      ratio: 1,
      color: '#000000',
      shape: 0,
      underline: 'bottom',
      emphasisDot: 0,
      positions: [0, 10],
      positionsComplete: false,
    },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'charOverlap',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    text: 'A',
    baseline: 8,
    rotation: 0,
    isVertical: false,
    style: { fontSize: 10 },
    positions: [0, 10],
    positionsComplete: false,
    charOverlap: { borderType: 1, innerCharSize: 100 },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'tabLeader',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    leaders: [{ startX: 1, endX: 8, fillType: 1 }],
    leadersComplete: false,
    color: '#000000',
    fontSize: 10,
    baseline: 8,
    rotation: 0,
    isVertical: false,
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'tabLeader',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    leaders: [{ startX: 1, endX: 8, fillType: 1.5 }],
    leadersComplete: true,
    color: '#000000',
    fontSize: 10,
    baseline: 8,
    rotation: 0,
    isVertical: false,
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'tabLeader',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    leaders: [{ startX: 4, endX: 4, fillType: 1 }],
    leadersComplete: true,
    color: '#000000',
    fontSize: 10,
    baseline: 8,
    rotation: 0,
    isVertical: false,
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'textDecoration',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    decoration: {
      kind: 'underline',
      baseline: 8,
      rotation: 0,
      isVertical: false,
      fontSize: 10,
      ratio: 1,
      color: '#000000',
      shape: 0.5,
      underline: 'future',
      emphasisDot: 0,
      positions: [0, 10],
      positionsComplete: true,
    },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'charOverlap',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    text: 'A',
    baseline: 8,
    rotation: 15,
    isVertical: false,
    style: { fontSize: 10 },
    positions: [0, 10],
    positionsComplete: true,
    charOverlap: { borderType: 1, innerCharSize: 100 },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'textControlMark',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    baseline: 8,
    rotation: 15,
    isVertical: false,
    marks: [],
    marksComplete: true,
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'tabLeader',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    leaders: [{ startX: 1, endX: 8, fillType: 1 }],
    color: '#000000',
    fontSize: 10,
    baseline: 8,
    rotation: 15,
    isVertical: false,
    leadersComplete: true,
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'textDecoration',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    decoration: {
      kind: 'underline',
      baseline: 8,
      rotation: 15,
      isVertical: false,
      fontSize: 10,
      ratio: 1,
      color: '#000000',
      shape: 0,
      underline: 'bottom',
      emphasisDot: 0,
      positions: [0, 10],
      positionsComplete: true,
    },
  }, 'screen');
  renderer.renderOp(canvas, {
    type: 'tabLeader',
    bbox: { x: 0, y: 0, width: -1, height: 10 },
    leaders: [{ startX: 1, endX: 8, fillType: 1 }],
    color: '#000000',
    fontSize: 10,
    baseline: 8,
    rotation: 0,
    isVertical: false,
    leadersComplete: true,
  }, 'screen');
  return {
    events,
    mirrorEvents,
    malformedEvents: events.slice(beforeMalformed),
    oldHangulAlias,
    rejectedOldHangulAlias,
    resolvedOldHangulAlias,
    unsupportedOps: renderer.unsupportedOps,
  };
}

function runExecutableFontNativeGlyphReplay() {
  const events = [];
  class FakePaint {
    setAntiAlias() {}
    setStyle() {}
    setColor() {}
    setStrokeWidth() {}
    setStrokeJoin() {}
    setStrokeCap() {}
    setStrokeMiter() {}
    setPathEffect() {}
    delete() { events.push('paint.delete'); }
  }
  const fakeImage = {
    width: () => 1,
    height: () => 1,
    delete() { events.push('image.delete'); },
  };
  const exifOrientedImage = {
    width: () => 3,
    height: () => 2,
    delete() { events.push('image.delete:exif'); },
  };
  const fakePath = () => ({
    setFillType() {},
    delete() { events.push('path.delete'); },
  });
  const canvasKit = {
    MakeImageFromEncoded(bytes) {
      events.push(`image.decode:${bytes.byteLength}`);
      return bytes[0] === 0xff && bytes[1] === 0xd8 ? exifOrientedImage : fakeImage;
    },
    Path: {
      MakeFromSVGString(pathData) {
        events.push(`path.parse:${pathData}`);
        return fakePath();
      },
    },
    Paint: FakePaint,
    PaintStyle: { Fill: 0, Stroke: 1 },
    FillType: { EvenOdd: 0, Winding: 1 },
    StrokeJoin: { Round: 0, Bevel: 1, Miter: 2 },
    StrokeCap: { Round: 0, Square: 1, Butt: 2 },
    PathEffect: { MakeDash: () => null },
    ClipOp: { Intersect: 0 },
    Color: (r, g, b, a) => [r, g, b, a],
    XYWHRect: (x, y, width, height) => ({ x, y, width, height }),
  };
  const canvas = {
    save() { events.push('canvas.save'); },
    restore() { events.push('canvas.restore'); },
    concat() { events.push('canvas.concat'); },
    translate() { events.push('canvas.translate'); },
    scale() { events.push('canvas.scale'); },
    drawImageRect() { events.push('canvas.drawImageRect'); },
    drawPath() { events.push('canvas.drawPath'); },
  };
  const renderer = new CanvasKitLayerRendererRuntime(
    canvasKit,
    'default',
    { preference: 'software', requested: 'software' },
    null,
  );
  const textFallback = {
    type: 'textRun',
    bbox: { x: 0, y: 0, width: 16, height: 16 },
    text: '\ue100',
    variant: {
      equivalenceGroup: 'font-native-0',
      variantId: 'textRun',
      variantKind: 'textRun',
      isDefaultFallback: true,
    },
  };
  const bitmapBytes = new Uint8Array(33);
  bitmapBytes.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  const bitmapView = new DataView(bitmapBytes.buffer);
  bitmapView.setUint32(8, 13);
  bitmapBytes.set([0x49, 0x48, 0x44, 0x52], 12);
  bitmapView.setUint32(16, 1);
  bitmapView.setUint32(20, 1);
  bitmapBytes.set([8, 6, 0, 0, 0], 24);
  const bitmapBase64 = Buffer.from(bitmapBytes).toString('base64');
  const bitmapResourceKey = `img:blake3:${bitmapBytes.byteLength}:${bytesToHex(blake3(bitmapBytes))}`;
  const bitmap = {
    type: 'glyphOutline',
    bbox: { x: 0, y: 0, width: 16, height: 16 },
    payloadKind: 'bitmapGlyph',
    bitmapGlyph: {
      imageRef: 0,
      placement: { x: 0, y: 0, width: 16, height: 16 },
      scalingPolicy: 'sourceExact',
    },
    diagnostics: { strictVisualEligible: true },
    variant: {
      equivalenceGroup: 'font-native-0',
      variantId: 'glyphOutline',
      variantKind: 'glyphOutline',
      isDefaultFallback: false,
    },
  };
  bitmap.payloadResourceKey = `${glyphOutlinePayloadResourceKeyRuntime(bitmap)}:resource:${bitmapResourceKey}`;
  const svgFragment = '<svg viewBox="0 0 16 16"><path d="M0 0H16V16H0Z"/></svg>';
  const svgBytes = new TextEncoder().encode(svgFragment);
  const svgResourceKey = `svg:blake3:${svgBytes.byteLength}:${bytesToHex(blake3(svgBytes))}`;
  renderer.currentResources = {
    images: [bitmapBase64],
    imageKeys: [bitmapResourceKey],
    svgFragments: [svgFragment],
    svgKeys: [svgResourceKey],
  };
  renderer.selectTextVariants({
    kind: 'leaf',
    bounds: bitmap.bbox,
    ops: [textFallback, bitmap],
  });
  assert.equal(renderer.selectedTextVariantOps.has(bitmap), true);
  assert.equal(renderer.selectedTextVariantOps.has(textFallback), false);
  renderer.renderGlyphOutline(canvas, bitmap);

  const svg = {
    ...bitmap,
    payloadKind: 'svgGlyph',
    payloadResourceKey: undefined,
    bitmapGlyph: undefined,
    svgGlyph: {
      svgRef: 0,
      viewBox: { x: 0, y: 0, width: 16, height: 16 },
      staticSanitized: true,
      scriptAllowed: false,
      animationAllowed: false,
      externalResourcesAllowed: false,
      interactivityAllowed: false,
    },
  };
  svg.payloadResourceKey = `${glyphOutlinePayloadResourceKeyRuntime(svg)}:resource:${svgResourceKey}`;
  assert.equal(renderer.glyphOutlineVariantReplayable(svg), true);
  renderer.renderGlyphOutline(canvas, svg);

  const corrupt = { ...bitmap, payloadResourceKey: 'glyphPayload:bitmapGlyph:resource:img:missing' };
  assert.equal(renderer.glyphOutlineVariantReplayable(corrupt), false);
  const imageDecodeEventsBeforeRejection = events.filter(
    (event) => event.startsWith('image.decode:'),
  ).length;
  const rejectedImage = {
    type: 'image',
    bbox: { x: 0, y: 0, width: 16, height: 16 },
    sourceImageKey: 'bin:1:7:src',
    imageRef: 7,
    base64: Buffer.from([1, 2, 3]).toString('base64'),
  };
  assert.equal(renderer.imageForOp(rejectedImage), null);
  assert.equal(renderer.imageForOp(rejectedImage), null);
  assert.equal(
    events.filter((event) => event.startsWith('image.decode:')).length,
    imageDecodeEventsBeforeRejection,
    'malformed headers must not reach the CanvasKit decoder',
  );
  const imageFailureDiagnostics = renderer.diagnostics();
  assert.equal(imageFailureDiagnostics.imageFailureCacheHits, 1);
  assert.deepEqual(imageFailureDiagnostics.imageFailures, [{
    source: 'sourceKey',
    sourceImageKey: 'bin:1:7:src',
    imageRef: 7,
    reason: 'encodedImageRejected',
  }]);
  assert.equal(imageFailureDiagnostics.passesRuntimeReadinessGate, false);
  assert.ok(imageFailureDiagnostics.readinessBlockers.includes('imageReplayFailure'));
  const orientedJpeg = new Uint8Array([
    0xff, 0xd8,
    0xff, 0xe0, 0x00, 0x04, 0x00, 0x00,
    0xff, 0xc0, 0x00, 0x0b, 0x08,
    0x00, 0x03,
    0x00, 0x02,
    0x01, 0x01, 0x11, 0x00,
  ]);
  assert.equal(renderer.imageForOp({
    type: 'image',
    bbox: { x: 0, y: 0, width: 3, height: 2 },
    sourceImageKey: 'bin:1:8:src',
    imageRef: 8,
    base64: Buffer.from(orientedJpeg).toString('base64'),
  }), exifOrientedImage, 'EXIF-oriented JPEG decode may swap bounded source dimensions');
  renderer.recordRenderFailure(new Error('page preparation failed'), true);
  const resetFailureDiagnostics = renderer.diagnostics();
  assert.deepEqual(
    resetFailureDiagnostics.imageFailures,
    [],
    'pre-replay failures must not retain image diagnostics from the previous page',
  );
  assert.equal(
    resetFailureDiagnostics.readinessBlockers.includes('imageReplayFailure'),
    false,
  );
  renderer.lastRenderCompleted = true;
  renderer.localTypefacePending.set('pending:test-face', 1);
  assert.ok(renderer.diagnostics().readinessBlockers.includes('localFontsPending'));
  renderer.localTypefacePending.clear();
  return events;
}

function runExecutableEquationFallback() {
  const events = [];
  class FakeFont {
    getGlyphIDs(text) { return Uint16Array.from(Array.from(text), () => 1); }
    getGlyphWidths(glyphIds) { return Array.from(glyphIds, () => 8); }
    setScaleX(scale) { events.push(`font.scale:${scale}`); }
    setEmbolden() {}
    setSkewX() {}
    delete() { events.push('font.delete'); }
  }
  class FakePaint {
    setAntiAlias() {}
    setStyle() {}
    setColor() {}
    setStrokeWidth() {}
    delete() { events.push('paint.delete'); }
  }
  const recordingCanvas = {
    save() { events.push('recording.save'); },
    restore() { events.push('recording.restore'); },
    translate() { events.push('recording.translate'); },
    scale() { events.push('recording.scale'); },
    drawLine() { events.push('canvas.drawLine'); },
    drawText() { events.push('canvas.drawText'); },
  };
  const picture = { delete() { events.push('picture.delete'); } };
  class FakePictureRecorder {
    beginRecording() { return recordingCanvas; }
    finishRecordingAsPicture() { return picture; }
    delete() { events.push('recorder.delete'); }
  }
  const renderer = new CanvasKitLayerRendererRuntime({
    Font: FakeFont,
    Paint: FakePaint,
    PictureRecorder: FakePictureRecorder,
    PaintStyle: { Fill: 0, Stroke: 1 },
    Color: (r, g, b, a) => [r, g, b, a],
    XYWHRect: (x, y, width, height) => ({ x, y, width, height }),
  }, 'default', {}, {});
  const canvas = {
    drawPicture() { events.push('canvas.drawPicture'); },
  };
  renderer.unsupportedOps = new Set();
  renderer.renderEquation(canvas, {
    type: 'equation',
    bbox: { x: 10, y: 20, width: 100, height: 30 },
    svgContent: '<svg><script>invalid</script></svg>',
    color: '#000000',
    fontSize: 12,
    layoutBox: {
      x: 0,
      y: 0,
      width: 40,
      height: 20,
      baseline: 10,
      kind: {
        type: 'fraction',
        numer: { x: 2, y: 0, width: 8, height: 8, baseline: 7, kind: { type: 'text', text: 'x' } },
        denom: { x: 2, y: 12, width: 8, height: 8, baseline: 7, kind: { type: 'number', text: '2' } },
      },
    },
  });
  assert.equal(renderer.unsupportedOps.size, 0, 'invalid equation SVG should use the semantic layout fallback');
  assert.ok(events.includes('canvas.drawLine'));
  assert.equal(events.filter((event) => event === 'canvas.drawText').length, 2);

  renderer.renderEquation(canvas, {
    type: 'equation',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    layoutBox: {
      x: Number.NaN,
      y: 0,
      width: 10,
      height: 10,
      baseline: 8,
      kind: { type: 'text', text: 'x' },
    },
  });
  assert.ok(renderer.unsupportedOps.has('equation:invalidLayout'));
}

const fontNativeGlyphReplayEvents = runExecutableFontNativeGlyphReplay();
assert.ok(fontNativeGlyphReplayEvents.includes('canvas.drawImageRect'));
assert.ok(fontNativeGlyphReplayEvents.includes('canvas.drawPath'));
const strokeDashReplay = runExecutableStrokeDashReplay();
assert.deepEqual(
  strokeDashReplay.events
    .filter(event => event.type === 'pathEffect.create')
    .map(event => event.intervals),
  [[6, 3], [2, 2], [6, 3, 2, 3], [6, 3, 2, 3, 2, 3]],
  'line, shape, and path replay should preserve every serialized stroke dash pattern',
);
assert.equal(
  strokeDashReplay.events.filter(event => event.type === 'pathEffect.delete').length,
  4,
  'each CanvasKit dash path effect should be released after drawing',
);
assert.deepEqual(
  strokeDashReplay.renderer.diagnostics().replayFeatureCounts,
  {
    dashedStrokes: 4,
    glyphRuns: 0,
    verticalPresentationPunctuation: 0,
    verticalTextRuns: 0,
  },
  'dash readiness counts must include only completed native dash draws',
);
assert.deepEqual(
  strokeDashReplay.events.find(event => event.type === 'canvas.drawPath'),
  { type: 'canvas.drawPath', color: [18, 52, 86, 1], width: 2 },
  'path replay should merge lineStyle color, width, and dash into the serialized shape style',
);
assert.equal(
  strokeDashReplay.events.filter(event => event.type.startsWith('canvas.draw')).length,
  strokeDashReplay.drawCountBeforeInvalid,
  'unknown dash styles must fail closed before drawing',
);
assert.ok(strokeDashReplay.renderer.unsupportedOps.has('strokeDash:zigzag'));
runExecutableEquationFallback();

requireSnippet(
  renderEquationBody,
  /op\.layoutBox[\s\S]*?PictureRecorder[\s\S]*?this\.renderEquationBox[\s\S]*?equation:invalidLayout/,
  'equation replay should commit a bounded semantic layout only after complete recording',
);
requireSnippet(
  renderEquationBoxBody,
  /MAX_EQUATION_LAYOUT_DEPTH[\s\S]*?remainingNodes[\s\S]*?equationBoxIsFinite/,
  'equation layout replay should bound recursion and reject non-finite geometry',
);

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
  /this\.makeStrokePaint\(op\.style\?\.color[\s\S]*?this\.drawStrokeWithDash\(op\.style\?\.dash[\s\S]*?canvas\.drawLine\(op\.x1, op\.y1, op\.x2, op\.y2, paint\)/,
  'line replay should draw a CanvasKit line with its serialized stroke pattern',
);
requireSnippet(
  drawStrokeWithDashBody,
  /dash === 'dash'[\s\S]*?\[6, 3\][\s\S]*?dash === 'dot'[\s\S]*?\[2, 2\][\s\S]*?dash === 'dashDot'[\s\S]*?\[6, 3, 2, 3\][\s\S]*?dash === 'dashDotDot'[\s\S]*?\[6, 3, 2, 3, 2, 3\][\s\S]*?PathEffect\.MakeDash[\s\S]*?setPathEffect[\s\S]*?finally[\s\S]*?effect\.delete/,
  'stroke dash replay should map every producer enum and release its native path effect',
);
requireSnippet(
  renderFormObjectBody,
  /op\.formType === 'checkBox'[\s\S]*?op\.formType === 'radioButton'[\s\S]*?op\.formType === 'checkbox'[\s\S]*?op\.formType === 'radio'[\s\S]*?canvas\.drawLine[\s\S]*?const label = op\.caption \|\| op\.text[\s\S]*?this\.renderTextRun/,
  'form object replay should keep canonical and compatibility checkbox/radio mark names explicit',
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
  /MAX_TEXT_RUN_CODE_POINTS[\s\S]*?this\.recordTextRunCoverageGaps\(op, replayCodePoints\)[\s\S]*?const primaryGlyphIds = font\.getGlyphIDs[\s\S]*?const drawPass[\s\S]*?const runGlyphIds = new Uint16Array[\s\S]*?canvas\.drawGlyphs\(/,
  'textRun replay should validate coverage once and reuse positioned glyphs across paint passes',
);
requireSnippet(
  canvaskitSource,
  /this\.currentFontResources = tree\.fontResources;[\s\S]*?this\.glyphRunFonts\.registerResources\(tree\.fontResources, tree\.resources\);[\s\S]*?this\.selectTextVariants\(tree\.root\)/,
  'GlyphRun font blobs must be verified before text variant selection',
);
requireSnippet(
  renderGlyphRunBody,
  /this\.glyphRunFonts\.font\(op, this\.currentFontResources\)[\s\S]*?drawCanvasKitGlyphRun\(canvas, op, font, paint\)[\s\S]*?this\.currentReplayFeatureCounts\.glyphRuns \+= 1[\s\S]*?glyphRun:replayFailed[\s\S]*?finally[\s\S]*?paint\.delete/,
  'strict GlyphRun replay should use the verified exact font and release per-draw paint state',
);
requireSnippet(
  renderTextRunBody,
  /const baseline = op\.baseline \?\? baseFontSize;[\s\S]*?const placementMatrix = this\.affineToCanvasKitMatrix\(op\.placement\?\.runToPage\);[\s\S]*?op\.bbox\.y \+ baseline[\s\S]*?canvas\.concat\(placementMatrix\);[\s\S]*?canvas\.rotate\(rotation, originX, originY\);/,
  'textRun replay should use canonical run placement with a page-space fallback',
);
requireSnippet(
  renderTextRunBody,
  /let fontSize = baseFontSize;[\s\S]*?let baselineShift = 0;[\s\S]*?style\.superscript[\s\S]*?fontSize = baseFontSize \* 0\.7;[\s\S]*?baselineShift -= baseFontSize \* 0\.3;[\s\S]*?style\.subscript[\s\S]*?fontSize = baseFontSize \* 0\.7;[\s\S]*?baselineShift \+= baseFontSize \* 0\.15;/,
  'textRun replay should apply superscript/subscript offsets in run-local space',
);
requireSnippet(
  renderTextRunBody,
  /const replayText = op\.displayText \?\? op\.text;[\s\S]*?const replayPositions = op\.displayText !== undefined \? op\.displayPositions : op\.positions;[\s\S]*?for \(const character of replayText\)[\s\S]*?textRun:visualItemLimitExceeded[\s\S]*?const glyphReplayText = verticalPresentationText \?\? replayText;[\s\S]*?if \(hasLayoutPositions\)[\s\S]*?const primaryGlyphIds = font\.getGlyphIDs\(glyphReplayText, codePoints\.length\)[\s\S]*?const runPositions = new Float32Array[\s\S]*?runPositions\[\(index - runStart\) \* 2\] = replayPositions!\[index\];[\s\S]*?canvas\.drawGlyphs\(/,
  'textRun replay should bound display projection and preserve serialized advances',
);
requireSnippet(
  renderTextRunBody,
  /VERTICAL_PRESENTATION_BASE_TEXT\.get\(replayText\)[\s\S]*?getGlyphBounds[\s\S]*?canvas\.translate\(targetCenterX \+ offsetX, targetCenterY \+ offsetY\)[\s\S]*?canvas\.rotate\(90, 0, 0\)/,
  'vertical presentation forms should replay through centered rotated base glyphs',
);
requireSnippet(
  renderTextRunBody,
  /setScaleX\?\.\(ratio\)[\s\S]*?shadeColor !== '#ffffff' && shadeColor !== '#000000'[\s\S]*?style\.emboss \|\| style\.engrave[\s\S]*?shadowType > 0[\s\S]*?outlineType > 0/,
  'textRun replay should retain ratio, shade, shadow, outline, emboss, and engrave paint passes',
);
requireSnippet(
  renderTextRunBody,
  /OLD_HANGUL_FONT_FAMILY[\s\S]*?selectedFontIndices[\s\S]*?return -2;[\s\S]*?fontIndex === -2[\s\S]*?codePoints\.slice\(runStart, runEnd\)\.join\(''\)[\s\S]*?originX \+ replayPositions!\[runStart\][\s\S]*?oldHangulTypeface\?\.fontManager/,
  'old Hangul Jamo should shape as a bounded cluster at the producer position',
);
requireSnippet(
  renderShapedScriptTextBody,
  /new this\.canvasKit\.ParagraphStyle[\s\S]*?this\.canvasKit\.ParagraphBuilder\.Make[\s\S]*?builder\.addText\(text\)[\s\S]*?paragraph\.layout\(CanvasKitLayerRenderer\.MAX_SHAPED_TEXT_WIDTH\)[\s\S]*?canvas\.drawParagraph\(paragraph, originX, originY - fontSize \+ baselineShift\)[\s\S]*?paragraph\.delete\?\.\(\)[\s\S]*?builder\.delete\?\.\(\)/,
  'old Hangul cluster replay should use CanvasKit paragraph shaping and release native objects',
);
requireSnippet(
  renderTextRunBody,
  /textRun:glyphMapping[\s\S]*?textRun:layoutPositions/,
  'textRun replay should expose malformed positioned-text fallbacks',
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
    paint: { kind: 'fill', color: '#000000', width: null },
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
  projectedTextReplay.events.find((event) => event.type === 'canvas.drawGlyphs')?.positions,
  [0, -6, 11, -6, 22, -6],
  'a bounded CJK PUA display projection should retain its serialized positions',
);

for (const text of [
  'e\u0301',
  'к\u0483',
  '漢\u302A',
  'か\u3099',
  'a\u200Fb',
  'a\u2067b',
  '\u00AD',
  'سلام',
  'ສະບາຍດີ',
  'བོད',
  'မြန်မာ',
]) {
  const shapedTextReplay = runExecutableTextReplay({
    type: 'textRun',
    bbox: { x: 0, y: 20, width: 48, height: 20 },
    text,
    baseline: 15,
    positions: Array.from({ length: Array.from(text).length + 1 }, (_, index) => index * 8),
    style: { fontSize: 20 },
  });
  assert.equal(shapedTextReplay.unsupportedOps.has('textRun:scriptTextRequiresShaping'), true);
  assert.equal(
    shapedTextReplay.events.some((event) => event.type.startsWith('canvas.draw')),
    false,
    `text without positioned cluster authority must fail closed: ${JSON.stringify(text)}`,
  );
}

const complexEffectReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 48, height: 20 },
  text: 'سلام',
  baseline: 15,
  positions: [0, 10, 20, 30, 40],
  style: { fontSize: 20, shadowType: 1, shadowOffsetX: 2, shadowOffsetY: 2 },
});
assert.equal(complexEffectReplay.unsupportedOps.has('textRun:scriptTextRequiresShaping'), true);
assert.equal(
  complexEffectReplay.events.some((event) => event.type.startsWith('canvas.draw')),
  false,
  'unsupported complex-script effect combinations must fail before partial drawing',
);

const textEffectReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'AB',
  baseline: 15,
  positions: [0, 12, 24],
  style: {
    fontSize: 20,
    ratio: 0.8,
    shadeColor: '#ffeeaa',
    shadowType: 1,
    shadowColor: '#445566',
    shadowOffsetX: 2,
    shadowOffsetY: 3,
    outlineType: 1,
    color: '#112233',
  },
});
assert.equal(textEffectReplay.error, null);
assert.equal(
  textEffectReplay.events.some((event) => event.type === 'font.scaleX' && event.scale === 0.8),
  true,
  'ratio replay should scale glyph shapes while retaining producer positions',
);
assert.deepEqual(
  textEffectReplay.events.find((event) => event.type === 'canvas.drawRect'),
  {
    type: 'canvas.drawRect',
    rect: { x: 0, y: 15, width: 24, height: 24 },
    paint: { kind: 'fill', color: '#ffeeaa', width: null },
  },
  'shade replay should paint the producer-width text background',
);

const noShadeSentinelReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'AB',
  baseline: 15,
  positions: [0, 12, 24],
  style: { fontSize: 20, shadeColor: '#000000' },
});
assert.equal(
  noShadeSentinelReplay.events.some((event) => event.type === 'canvas.drawRect'),
  false,
  'the legacy zero shade sentinel must not paint a black text background',
);

const positionedCjkEffectReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: '가',
  baseline: 15,
  positions: [0, 14],
  style: { fontSize: 20, superscript: true, shadowType: 1 },
});
assert.equal(positionedCjkEffectReplay.unsupportedOps.size, 0);
assert.equal(
  positionedCjkEffectReplay.events.filter((event) => event.type === 'canvas.drawGlyphs').length,
  2,
  'positioned CJK superscript should retain both its shadow and authored fill passes',
);
assert.deepEqual(
  textEffectReplay.events
    .filter((event) => event.type === 'canvas.drawGlyphs')
    .map(({ x, y, paint }) => ({ x, y, paint })),
  [
    { x: 2, y: 38, paint: { kind: 'fill', color: '#445566', width: null } },
    { x: 0, y: 35, paint: { kind: 'fill', color: '#ffffff', width: null } },
    { x: 0, y: 35, paint: { kind: 'stroke', color: '#112233', width: 0.8 } },
  ],
  'shadow and outline should replay as positioned fill/stroke passes',
);

for (const [effect, expectedPasses] of [
  ['emboss', [
    { x: -1, y: 34, color: '#ffffff' },
    { x: 1, y: 36, color: '#808080' },
    { x: 0, y: 35, color: '#000000' },
  ]],
  ['engrave', [
    { x: -1, y: 34, color: '#808080' },
    { x: 1, y: 36, color: '#ffffff' },
    { x: 0, y: 35, color: '#000000' },
  ]],
]) {
  const reliefReplay = runExecutableTextReplay({
    type: 'textRun',
    bbox: { x: 0, y: 20, width: 30, height: 20 },
    text: 'AB',
    baseline: 15,
    positions: [0, 12, 24],
    style: { fontSize: 20, [effect]: true },
  });
  assert.deepEqual(
    reliefReplay.events
      .filter((event) => event.type === 'canvas.drawGlyphs')
      .map(({ x, y, paint }) => ({ x, y, color: paint.color })),
    expectedPasses,
    `${effect} should replay two relief passes plus the authored fill`,
  );
}

const reliefAllocationFailure = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'AB',
  baseline: 15,
  positions: [0, 12, 24],
  style: { fontSize: 20, emboss: true },
}, { fillPaintErrorAt: 3 });
assert.match(String(reliefAllocationFailure.error), /synthetic fill paint failure/);
assert.equal(
  reliefAllocationFailure.events.some(
    (event) => event.type === 'paint.delete' && event.color === '#ffffff',
  ),
  true,
  'a partially constructed relief pass should release the first native paint',
);

const verticalPresentationReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 10, y: 20, width: 20, height: 30 },
  text: '\uFE35',
  baseline: 15,
  rotation: 0,
  isVertical: true,
  orientation: 'vertical-upright',
  positions: [0, 20],
  style: { fontSize: 20 },
});
assert.equal(
  verticalPresentationReplay.events.some(
    (event) => event.type === 'font.getGlyphIDs' && event.text === '(',
  ),
  true,
  'vertical presentation punctuation should resolve a broadly available base glyph',
);
assert.equal(
  verticalPresentationReplay.events.some(
    (event) => event.type === 'font.getGlyphIDs' && event.text === '\uFE35',
  ),
  false,
  'vertical presentation punctuation should not require the compatibility glyph itself',
);
assert.deepEqual(
  verticalPresentationReplay.events.find((event) => event.type === 'canvas.translate'),
  { type: 'canvas.translate', x: 20, y: 35 },
  'vertical punctuation should rotate around the producer cell center',
);
assert.equal(
  verticalPresentationReplay.events.some(
    (event) => event.type === 'canvas.rotate' && event.rotation === 90,
  ),
  true,
  'vertical punctuation should rotate its base glyph by 90 degrees',
);
assert.deepEqual(
  verticalPresentationReplay.diagnostics.replayFeatureCounts,
  {
    dashedStrokes: 0,
    glyphRuns: 0,
    verticalPresentationPunctuation: 1,
    verticalTextRuns: 1,
  },
  'vertical readiness counts must be emitted only after the dedicated replay path completes',
);

const verticalSuperscriptReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 10, y: 20, width: 20, height: 30 },
  text: '\uFE35',
  baseline: 15,
  isVertical: true,
  orientation: 'vertical-upright',
  positions: [0, 20],
  style: { fontSize: 20, superscript: true },
});
assert.deepEqual(
  verticalSuperscriptReplay.events.find((event) => event.type === 'canvas.translate'),
  { type: 'canvas.translate', x: 20, y: 29 },
  'vertical superscript punctuation should apply its baseline shift in the same direction as text',
);

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

const oversizedTextReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'A'.repeat(4097),
  baseline: 15,
  style: { fontSize: 20 },
});
assert.equal(oversizedTextReplay.unsupportedOps.has('textRun:visualItemLimitExceeded'), true);
assert.equal(
  oversizedTextReplay.events.some((event) => event.type === 'font.create'),
  false,
  'forced CanvasKit replay should reject oversized text before native font allocation',
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
  true,
  'an unresolved glyph should retain its producer position while runtime diagnostics fail closed',
);

const fallbackGlyphReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'A①B',
  baseline: 15,
  positions: [0, 8, 17, 25],
  style: { fontFamily: 'Prepared', fontSize: 20 },
}, {
  glyphIds: [1, 0, 2],
  fallbackGlyphIds: [0, 7, 0],
  usePreparedTypeface: true,
});
assert.equal(fallbackGlyphReplay.unsupportedOps.has('textRun:glyphMapping'), false);
assert.deepEqual(
  fallbackGlyphReplay.events
    .filter((event) => event.type === 'canvas.drawGlyphs')
    .map(({ glyphIds: ids, positions }) => ({ glyphIds: ids, positions })),
  [
    { glyphIds: [1], positions: [0, 0] },
    { glyphIds: [7], positions: [8, 0] },
    { glyphIds: [2], positions: [17, 0] },
  ],
  'fallback glyphs should switch fonts per contiguous run without changing serialized positions',
);
assert.deepEqual(
  fallbackGlyphReplay.diagnostics.fontSubstitutions,
  [{
    requestedFamily: 'Prepared',
    resolvedFamily: 'Noto Sans KR',
    source: 'missingGlyphDefault',
    kind: 'glyphCoverageFallback',
  }],
  'prepared fonts with coverage gaps should expose the selected default fallback',
);

const symbolGlyphReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 30, height: 20 },
  text: 'A①B',
  baseline: 15,
  positions: [0, 8, 17, 25],
  style: { fontFamily: 'Prepared', fontSize: 20 },
}, {
  glyphIds: [1, 0, 2],
  fallbackGlyphIds: [0, 0, 0],
  symbolGlyphIds: [0, 9, 0],
  usePreparedTypeface: true,
});
assert.equal(symbolGlyphReplay.unsupportedOps.has('textRun:glyphMapping'), false);
assert.deepEqual(
  symbolGlyphReplay.events
    .filter((event) => event.type === 'canvas.drawGlyphs')
    .map(({ glyphIds: ids, positions }) => ({ glyphIds: ids, positions })),
  [
    { glyphIds: [1], positions: [0, 0] },
    { glyphIds: [9], positions: [8, 0] },
    { glyphIds: [2], positions: [17, 0] },
  ],
  'the bounded symbol face should be the final positioned fallback without moving surrounding text',
);
assert.deepEqual(
  symbolGlyphReplay.diagnostics.fontSubstitutions,
  [{
    requestedFamily: 'Prepared',
    resolvedFamily: 'CanvasKit symbol fallback',
    source: 'missingGlyphSymbol',
    kind: 'glyphCoverageFallback',
  }],
  'symbol coverage fallbacks should remain observable in renderer diagnostics',
);

const unregisteredFontReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 20, height: 20 },
  text: 'A',
  baseline: 15,
  positions: [0, 10],
  style: { fontFamily: 'Missing Family', fontSize: 20 },
});
assert.equal(unregisteredFontReplay.diagnostics.unregisteredFontFallbacks, 1);
assert.deepEqual(
  unregisteredFontReplay.diagnostics.fontSubstitutions,
  [{
    requestedFamily: 'Missing Family',
    resolvedFamily: 'Noto Sans KR',
    source: 'unregisteredDefault',
    kind: 'unregisteredFallback',
  }],
  'unregistered authored families should not silently disappear behind the default face',
);
const strictMissingFontReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 20, height: 20 },
  text: 'A',
  baseline: 15,
  positions: [0, 10],
  style: { fontFamily: 'Missing Family', fontSize: 20 },
}, { requirePreparedFontFamilies: true });
assert.match(String(strictMissingFontReplay.error), /font family가 준비되지 않았습니다/);
assert.equal(
  strictMissingFontReplay.events.some((event) => event.type === 'paint.create'),
  false,
  'strict font readiness failures must happen before allocating native paint',
);
for (let index = 0; index < 5000; index += 1) {
  unregisteredFontReplay.renderer.recordFontSubstitution({
    requestedFamily: `Missing Family ${index}`,
    resolvedFamily: 'Noto Sans KR',
    source: 'unregisteredDefault',
    kind: 'unregisteredFallback',
  });
}
assert.equal(
  unregisteredFontReplay.renderer.diagnostics().fontSubstitutions.length,
  unregisteredFontReplay.diagnostics.fontSubstitutionLimit,
  'font substitution diagnostics should stay within their advertised runtime limit',
);
unregisteredFontReplay.renderer.recordFontSubstitution({
  requestedFamily: 'Missing Family',
  resolvedFamily: 'Noto Sans KR',
  source: 'unregisteredDefault',
  kind: 'unregisteredFallback',
});
assert.equal(
  unregisteredFontReplay.renderer.diagnostics().fontSubstitutions.length,
  unregisteredFontReplay.diagnostics.fontSubstitutionLimit,
  'repeated substitutions should remain deduplicated after reaching the limit',
);
unregisteredFontReplay.renderer.resetDocumentResources();
assert.deepEqual(
  unregisteredFontReplay.renderer.diagnostics().fontSubstitutions,
  [],
  'document resource reset should clear prior font substitution diagnostics',
);

const oldHangulReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 10, y: 20, width: 40, height: 20 },
  text: 'A\u{F53A}B',
  displayText: 'A\u1112\u119E\u11ABB',
  baseline: 15,
  positions: [0, 8, 20, 28],
  displayPositions: [0, 8, 8, 8, 20, 28],
  style: { fontFamily: 'Prepared', fontSize: 20 },
}, { usePreparedTypeface: true });
assert.equal(oldHangulReplay.unsupportedOps.has('textRun:glyphMapping'), false);
assert.deepEqual(
  oldHangulReplay.events.find(event => event.type === 'paragraphBuilder.addText'),
  { type: 'paragraphBuilder.addText', text: '\u1112\u119E\u11AB' },
  'old Hangul PUA projection should shape its Jamo sequence as one cluster',
);
assert.deepEqual(
  oldHangulReplay.events.find(event => event.type === 'canvas.drawParagraph'),
  { type: 'canvas.drawParagraph', x: 18, y: 15 },
  'old Hangul shaping should begin at the serialized cluster position',
);
assert.deepEqual(
  oldHangulReplay.events
    .filter(event => event.type === 'canvas.drawGlyphs')
    .map(({ glyphIds: ids, positions }) => ({ glyphIds: ids, positions })),
  [
    { glyphIds: [1], positions: [0, 0] },
    { glyphIds: [5], positions: [20, 0] },
  ],
  'surrounding glyphs should retain their producer positions around shaped old Hangul',
);

const boxedPuaReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 20, height: 20 },
  text: '\u{F02B1}',
  baseline: 15,
  positions: [0, 18],
  style: { fontFamily: 'Prepared', fontSize: 20 },
}, {
  glyphIds: [0],
  fallbackGlyphIds: [0],
  symbolGlyphIds: [0],
  usePreparedTypeface: true,
});
assert.equal(boxedPuaReplay.unsupportedOps.has('textRun:glyphMapping'), false);
assert.equal(
  boxedPuaReplay.events.some(event => event.type === 'canvas.drawRect'),
  true,
  'Hancom boxed-number PUA should use a bounded vector box fallback',
);
assert.equal(
  boxedPuaReplay.events.some(event => event.type === 'canvas.drawText' && event.text === '1'),
  true,
  'Hancom boxed-number PUA should preserve the encoded number',
);

const textSpecialReplay = runExecutableTextSpecialReplay();
assert.equal(textSpecialReplay.events.some(event => event.type === 'canvas.drawOval'), true);
assert.equal(
  textSpecialReplay.events.some(event => event.type === 'canvas.drawText' && event.text === '1'),
  true,
  'circled overlap text should replay as a directly drawn border plus display digit',
);
assert.equal(
  textSpecialReplay.events.filter(event => event.type === 'canvas.drawLine').length >= 6,
  true,
  'control marks should replay as font-independent vectors at producer positions',
);
assert.equal(
  textSpecialReplay.events.some(event => event.type === 'canvas.drawText' && ['∨', '↵'].includes(event.text)),
  false,
  'control mark replay should not depend on optional symbol glyph coverage',
);
assert.equal(textSpecialReplay.events.some(event => event.type === 'pathEffect.create'), true);
assert.equal(textSpecialReplay.events.some(event => event.type === 'canvas.drawCircle'), true);
assert.equal(
  textSpecialReplay.events.some(event => event.type === 'font.scaleX' && event.scale === 0.7),
  true,
  'combined overlap numbers should use the Canvas2D digit-count compression formula',
);
assert.equal(
  textSpecialReplay.rejectedOldHangulAlias,
  null,
  'a typeface-only old-Hangul alias must not satisfy the shaping contract',
);
assert.equal(
  textSpecialReplay.resolvedOldHangulAlias,
  textSpecialReplay.oldHangulAlias,
  'a prepared old-Hangul alias must remain reachable when the dedicated subset is unavailable',
);
assert.deepEqual(textSpecialReplay.mirrorEvents, [], 'the TextRun char-overlap mirror must not double-paint');
assert.deepEqual(
  textSpecialReplay.malformedEvents,
  [],
  'malformed or over-limit text visuals must fail closed without drawing partial output',
);
for (const diagnostic of [
  'charOverlap:visualItemLimitExceeded',
  'charOverlap:invalidGeometry',
  'textControlMark:invalidGeometry',
  'textControlMark:visualItemLimitExceeded',
  'tabLeader:invalidGeometry',
  'tabLeader:visualItemLimitExceeded',
  'textDecoration:invalidGeometry',
  'textDecoration:visualItemLimitExceeded',
  'charOverlap:rotatedText',
  'textControlMark:rotatedText',
  'tabLeader:rotatedText',
  'textDecoration:rotatedText',
]) {
  assert.equal(
    textSpecialReplay.unsupportedOps.has(diagnostic),
    true,
    `malformed text visuals should report ${diagnostic}`,
  );
}

const alternatingGlyphText = 'A'.repeat(4098);
const alternatingGlyphReplay = runExecutableTextReplay({
  type: 'textRun',
  bbox: { x: 0, y: 20, width: 4098, height: 20 },
  text: alternatingGlyphText,
  baseline: 15,
  positions: Array.from({ length: 4099 }, (_, index) => index),
  style: { fontSize: 12 },
}, {
  glyphIds: Array.from({ length: 4098 }, (_, index) => index % 2 === 0 ? 1 : 0),
  symbolGlyphIds: Array.from({ length: 4098 }, (_, index) => index % 2 === 0 ? 0 : 1),
});
assert.equal(
  alternatingGlyphReplay.unsupportedOps.has('textRun:visualItemLimitExceeded'),
  true,
  'alternating fallback coverage must hit the text bound before native draw-call amplification',
);
assert.equal(
  alternatingGlyphReplay.events.some(event => event.type === 'canvas.drawGlyphs'),
  false,
  'over-limit fallback segmentation must not draw a partial text run',
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
  text: '\u1112\u119E\u11AB',
  baseline: 15,
  positions: [0, 0, 0, 8],
  style: { fontSize: 20 },
}, {
  drawParagraphError: new Error('paragraph draw failed'),
  usePreparedTypeface: true,
});
assert.equal(shapedCleanupReplay.error?.message, 'paragraph draw failed');
for (const cleanupEvent of ['canvas.restore', 'paragraph.delete', 'paragraphBuilder.delete', 'paint.delete']) {
  assert.equal(
    shapedCleanupReplay.events.some((event) => event.type === cleanupEvent),
    true,
    `${cleanupEvent} should run after drawParagraph throws`,
  );
}
for (const closedTextRunGap of [
  'textRun:verticalText',
  'textRun:outlineTextEffect',
  'textRun:shadowTextEffect',
  'textRun:embossTextEffect',
  'textRun:engraveTextEffect',
  'textRun:shadeTextEffect',
  'textRun:ratioTextEffect',
]) {
  assert.equal(
    recordTextRunCoverageGapsBody.includes(`'${closedTextRunGap}'`),
    false,
    `textRun runtime diagnostics should no longer report ${closedTextRunGap}`,
  );
}
assert.equal(
  recordTextRunCoverageGapsBody.includes("'textRun:scriptTextRequiresShaping'"),
  true,
  'complex scripts and shaped fallback effect combinations must remain fail-closed',
);
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
  [
    'font-batang-hancom',
    'font-native-bitmap',
    'image-crop',
    'paragraph-line-basic',
    'paragraph-text-marks',
    'pua-special-glyphs',
    'table-border-style',
    'table-core',
  ],
  'CanvasKit readiness gate should cover text visuals, positioned fallbacks, and core resources',
);
const textMarkReadinessSample = rendererBaselineManifest.samples
  .find((sample) => sample.id === 'paragraph-text-marks');
assert.deepEqual(
  textMarkReadinessSample?.viewOptions,
  { showParagraphMarks: true, showControlCodes: false },
  'text-mark readiness must exercise the directly replayable paragraph-mark mode only',
);
assert.ok(
  rendererBaselineSource.includes('applySampleViewOptions(page, sample.viewOptions)'),
  'browser baseline capture must apply manifest view options before the selected-page replay',
);
assert.match(
  rendererBaselineSource,
  /viewOptions:\s*\{\s*showParagraphMarks:\s*false,\s*showControlCodes:\s*false,\s*\}/,
  'every baseline sample must reset view options so one marked sample cannot contaminate the next',
);
const fontNativeReadinessSample = rendererBaselineManifest.samples
  .find((sample) => sample.id === 'font-native-bitmap');
assert.equal(
  fontNativeReadinessSample?.browserParityThresholds?.minimumInkPixels,
  40,
  'font-native readiness must retain a positive anti-blank budget calibrated to intrinsic capture',
);
const tableReadinessSample = rendererBaselineManifest.samples
  .find((sample) => sample.id === 'table-core');
assert.equal(
  tableReadinessSample?.browserParityThresholds?.maxDiffRatio,
  0.047,
  'table readiness must keep the calibrated tolerant pixel budget bounded',
);
assert.equal(
  tableReadinessSample?.browserParityThresholds?.inkMaskMaxDiffRatio,
  0.0185,
  'table readiness must keep the calibrated ink-mask budget bounded',
);
const verticalTextReadinessSample = rendererBaselineManifest.samples
  .find((sample) => sample.id === 'table-border-style');
assert.ok(
  [0.005, 0.006].includes(
    verticalTextReadinessSample?.browserParityThresholds?.inkMaskMaxDiffRatio,
  ),
  'vertical text readiness must use an approved calibrated font-raster ink-mask tolerance',
);
assert.equal(
  verticalTextReadinessSample?.browserParityThresholds?.nonInkMaxDiffPixels,
  4,
  'vertical punctuation readiness must keep non-ink raster changes tightly bounded',
);
assert.equal(
  verticalTextReadinessSample?.browserParityThresholds?.minimumInkPixels,
  50000,
  'vertical text readiness must reject blank or substantially incomplete output',
);
assert.equal(
  verticalTextReadinessSample?.diagnosticAxes?.includes('vertical-text'),
  true,
  'the real HWP vertical-text gate must retain its diagnostic axis',
);
assert.deepEqual(
  verticalTextReadinessSample?.canvaskitReadinessExpectations?.minLayerFeatureCounts,
  {
    dashedStrokes: 12,
    verticalPresentationPunctuation: 2,
    verticalTextRuns: 14,
  },
  'the real HWP gate must prove its vertical text, punctuation, and dash features are present',
);
assert.equal(rendererBaselineManifest.schemaVersion, 1, 'renderer baseline manifest schema must be explicit');
assert.ok(
  rendererBaselineManifest.samples.length >= 120,
  'renderer baseline manifest must keep the refreshed cross-backend corpus',
);
for (const sample of rendererBaselineManifest.samples) {
  assert.ok(
    Array.isArray(sample.diagnosticAxes)
      && sample.diagnosticAxes.length > 0
      && new Set(sample.diagnosticAxes).size === sample.diagnosticAxes.length,
    `renderer baseline sample ${sample.id} must declare unique diagnostic axes`,
  );
  assert.ok(
    sample.baselineTier === 'representative' || sample.baselineTier === 'extended',
    `renderer baseline sample ${sample.id} must declare its corpus tier`,
  );
  assert.ok(
    Number.isInteger(sample.page ?? 0) && (sample.page ?? 0) >= 0,
    `renderer baseline sample ${sample.id} must declare a valid page`,
  );
}
assert.equal(
  rendererBaselineManifest.samples.filter((sample) => sample.baselineTier === 'representative').length,
  24,
  'the default renderer baseline tier must remain bounded',
);
for (const sampleId of [
  'chart-line-markers-hwp',
  'chart-line-markers-hwpx',
  'chart-stock-hwp',
  'chart-stock-hwpx',
  'table-cell-image-clip-page-1',
  'missing-picture-profile',
  'local-font-nanumsquare-bold',
  'malformed-lineseg-reflow',
]) {
  assert.ok(
    rendererBaselineManifest.samples.some((sample) => sample.id === sampleId),
    `renderer baseline manifest must keep recent regression sample ${sampleId}`,
  );
}
assert.equal(
  rendererBaselineManifest.samples.some((sample) => Number(sample.page) > 0),
  true,
  'renderer baseline manifest must keep non-zero page coverage',
);
assert.deepEqual(
  rendererBaselineManifest.samples
    .filter((sample) => sample.id.startsWith('table-diagonal-cell-'))
    .map((sample) => sample.file)
    .sort(),
  ['대각선샘플.hwp', '대각선샘플.hwpx'],
  'renderer baseline manifest must keep the paired HWP/HWPX diagonal-cell corpus',
);
assert(
  rendererBaselineSource.includes('pageRenderer.renderPage(capturePageIndex, canvas, 1.0, 1.0, 1.0)')
    && rendererBaselineSource.includes('pageRenderer?.cancelAll?.()')
    && rendererBaselineSource.includes('BASELINE_CAPTURE_CONTAINER_SELECTOR')
    && rendererBaselineSource.includes('canvas2dRenderer?.domImageCache')
    && rendererBaselineSource.includes("container.querySelectorAll('img')")
    && rendererBaselineSource.includes("typeof image.decode === 'function'")
    && rendererBaselineSource.includes('localTypefacePendingCount')
    && rendererBaselineSource.includes('selectedPageRenderMs')
    && rendererBaselineDriverSource.includes('averageSelectedPageRenderMs')
    && helpersSource.includes('selector = CANVAS_SELECTOR')
    && !rendererBaselineSource.includes('browser baseline currently supports only page=0 samples'),
  'browser baseline must settle resources and capture the requested page at intrinsic scale',
);
assert(
  rendererBaselineSource.includes('getCanvasKitReplayPlan?.(')
    && rendererBaselineSource.includes('targetProfile,')
    && rendererBaselineSource.includes("code: 'replayPlanUnavailable'")
    && rendererBaselineSource.includes("code: 'replayPlanEmpty'")
    && rendererBaselineSource.includes("code: 'replayPlanContractMismatch'")
    && rendererBaselineSource.includes("code: 'runtimeDiagnosticsUnavailable'")
    && rendererBaselineSource.includes("code: 'runtimeRenderIncomplete'")
    && rendererBaselineSource.includes("code: 'runtimeRenderError'")
    && rendererBaselineSource.includes("code: 'runtimeUnexpectedUnsupportedOps'")
    && rendererBaselineSource.includes("code: 'runtimeImageDiagnosticsUnavailable'")
    && rendererBaselineSource.includes("code: 'runtimeImageReplayFailure'")
    && rendererBaselineSource.includes("code: 'runtimeBackendMismatch'")
    && rendererBaselineSource.includes("code: 'runtimeProfileMismatch'")
    && rendererBaselineSource.includes('contractGateAndReportInventory')
    && rendererBaselineSource.includes('planReasonCounts')
    && rendererBaselineSource.includes('planFeatureCounts')
    && rendererBaselineSource.includes('imageFailureReasonCounts')
    && rendererBaselineSource.includes('imageFailureSourceCounts'),
  'browser baseline must gate replay-plan/runtime contract failures and inventory known gaps',
);
assert.deepEqual(inspectCanvasKitRuntimeImageFailures(null), {
  available: false,
  failures: [],
  hasFailures: false,
});
assert.deepEqual(inspectCanvasKitRuntimeImageFailures({ imageFailures: {} }), {
  available: false,
  failures: [],
  hasFailures: false,
});
assert.deepEqual(inspectCanvasKitRuntimeImageFailures({ imageFailures: [] }), {
  available: true,
  failures: [],
  hasFailures: false,
});
const runtimeImageFailure = { source: 'inline', reason: 'imageDecodeFailed' };
assert.deepEqual(inspectCanvasKitRuntimeImageFailures({
  imageFailures: [runtimeImageFailure],
}), {
  available: true,
  failures: [runtimeImageFailure],
  hasFailures: true,
});
assert(
  rendererBaselineDriverSource.includes('CanvasKit Replay Diagnostics')
    && rendererBaselineDriverSource.includes('Replay Diagnostic Inventory')
    && rendererBaselineDriverSource.includes('expectedUnsupportedOpCounts')
    && rendererBaselineDriverSource.includes('unexpectedUnsupportedOpCounts')
    && rendererBaselineDriverSource.includes('runtimeImageReplayFailures')
    && rendererBaselineDriverSource.includes('imageFailureReasonCounts'),
  'renderer baseline report must preserve replay-plan and runtime diagnostic inventories',
);
assert(
  rendererBaselineSource.includes("createHash('sha256')")
    && rendererBaselineSource.includes('comparisonIdentity')
    && rendererBaselineSource.includes("status: 'identityMismatch'")
    && rendererBaselineSource.includes('summaryByDiagnosticAxis')
    && rendererBaselineDriverSource.includes('documentDigest')
    && rendererBaselineDriverSource.includes('comparisonIdentity')
    && rendererBaselineDriverSource.includes('Diagnostic Axis Summary')
    && rendererBaselineSource.includes('fs.realpathSync(samplePath)')
    && rendererBaselineSource.includes('baseline sample page must be a non-negative integer')
    && rendererBaselineNativeDiffSource.includes("status: 'identityMismatch'")
    && rendererBaselineNativeDiffSource.includes("createHash('sha256')")
    && rendererBaselineNativeDiffSource.includes('nativeArtifactSha256')
    && rendererBaselineNativeDiffSource.includes('nativeArtifactSizeBytes')
    && rendererBaselineDriverSource.includes('native Skia ({profile}) baseline export did not create a non-empty artifact')
    && rendererBaselineNativeDiffSource.includes('summaryByDiagnosticAxis'),
  'cross-backend comparisons must bind document/page/profile/artifact provenance and diagnostic axes',
);
assert(
  rendererBaselineDriverSource.includes('--include-pdf')
    && rendererBaselineDriverSource.includes('"export-pdf"')
    && rendererBaselineDriverSource.includes('"--profile"')
    && rendererBaselineDriverSource.includes('"backend": "pdf"')
    && rendererBaselineDriverSource.includes('PDF baseline export did not create a non-empty artifact')
    && fullRendererSweepWorkflowSource.includes('--include-pdf'),
  'full renderer baseline must collect verified print-profile PDF artifacts',
);
assert.ok(
  renderDiffWorkflowSource.includes('node --check e2e/renderer-baseline-native-diff.mjs')
    && renderDiffWorkflowSource.includes(
      'node e2e/renderer-baseline-native-diff.mjs --self-test',
    ),
  'Render Diff preflight must syntax-check and self-test the native parity comparator',
);
assert.ok(
  renderDiffWorkflowSource.includes('actions: read')
    && renderDiffWorkflowSource.includes('github.rest.actions.listWorkflowRuns')
    && renderDiffWorkflowSource.includes("workflow_id: 'render-diff.yml'")
    && renderDiffWorkflowSource.includes('head_sha: candidateSha')
    && renderDiffWorkflowSource.includes("run.path === '.github/workflows/render-diff.yml'")
    && renderDiffWorkflowSource.includes("renderDiffRun.conclusion !== 'success'")
    && renderDiffWorkflowSource.includes('renderDiffRun.head_repository?.id !== pr.head.repo?.id')
    && renderDiffWorkflowSource.includes('github.rest.actions.listJobsForWorkflowRun')
    && renderDiffWorkflowSource.includes("job.name === 'Canvas visual diff'")
    && renderDiffWorkflowSource.includes("renderDiffJob.conclusion !== 'success'")
    && renderDiffWorkflowSource.includes('`Render Diff identity PR #${pr.number} base ${pr.base.sha}`')
    && renderDiffWorkflowSource.includes("identityStep.conclusion !== 'success'")
    && renderDiffWorkflowSource.includes('Render Diff identity PR #${{ github.event.pull_request.number }} base ${{ github.event.pull_request.base.sha }}')
    && !renderDiffWorkflowSource.includes('github.rest.checks.listForRef')
    && !renderDiffWorkflowSource.includes('allowedConclusions'),
  'Render Diff fast-pass must trust only the exact successful workflow and Canvas job',
);
assert.ok(
  renderDiffWorkflowSource.includes("- 'src/model/**'")
    && renderDiffWorkflowSource.includes("- 'ttfs/**'"),
  'Render Diff must rerun for model and bundled-font changes',
);
assert.ok(
  rendererBaselineDriverSource.includes('--scope')
    && rendererBaselineSource.includes("scope: 'representative'")
    && fullRendererSweepWorkflowSource.includes('corpus:')
    && fullRendererSweepWorkflowSource.includes('--scope ${{ inputs.corpus }}')
    && fullRendererSweepWorkflowSource.includes('--scope representative'),
  'the default workflow must bound the corpus while retaining an explicit full sweep',
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
  'legacyRequestProjectionMismatch',
  'autoSelectionMismatch',
  'autoPreflightNotEligible',
  'autoDocumentDigestMissing',
  'autoDecisionGenerationMissing',
  'canvaskitModeRequestMismatch',
  'canvaskitSurfaceRequestMismatch',
  'canvaskitModeMismatch',
  'canvaskitSurfacePreferenceMismatch',
  'canvasOwnershipMismatch',
  'diagnosticsUnavailable',
  'runtime:readinessGateFailed',
  'visualThresholdMissing',
  'visualParityFailed',
  'performanceBudgetMissing',
  'performanceColdExceeded',
  'performanceWarmExceeded',
  'performanceRendererWarmExceeded',
  'imageCachePixelBudgetExceeded',
  'warmReplayMissing',
  'glyphOutlinePayloadMissing:',
  'warmImageCacheHitMissing',
  'layerFeatureMinimumMissing:',
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
assert.ok(
  rendererBaselineSource.includes('measureWarmCanvasKitReplay')
    && rendererBaselineSource.includes('requireColdAndWarmPerformanceBudget')
    && rendererBaselineSource.includes('readLayerFeatureProbe')
    && rendererBaselineSource.includes('?.replayFeatureCounts')
    && rendererBaselineSource.includes('minLayerFeatureCounts')
    && rendererBaselineSource.includes('layerFeatureMinimumMissing:'),
  'CanvasKit readiness should gate cold/warm replay and declared layer features',
);
requireSnippet(
  rendererBaselineSource,
  /getCurrentCanvasKitRenderDiagnostics\?\.\(\)[\s\S]*?rerenderPageForDiagnostics\?\.\(targetPageIndex\)[\s\S]*?getCurrentCanvasKitRenderDiagnostics\?\.\(\)[\s\S]*?renderCountDelta/,
  'CanvasKit warm replay should report whether the existing page canvas was rerendered',
);
requireSnippet(
  canvasViewSource,
  /rerenderPageForDiagnostics\(pageIdx: number\)[\s\S]*?canvasPool\.getCanvas\(pageIdx\)[\s\S]*?this\.renderCanvas\(pageIdx, canvas\)/,
  'diagnostic warm replay should reuse the canvas already owned by the pool',
);
assert.doesNotMatch(
  rendererBaselineSource,
  /view\?\.renderPage\?\.\(targetPageIndex\)/,
  'warm replay must not acquire a second canvas for an already rendered page',
);
requireSnippet(
  renderDiffWorkflowSource,
  /Run selected CanvasKit readiness gate[\s\S]*?scripts\/renderer_baseline\.py[\s\S]*?--readiness-only/,
  'render-diff CI should run the selected CanvasKit readiness gate',
);
assert.ok(
  renderDiffWorkflowSource.includes("RHWP_CHROMIUM_BUILD_ID: '1660786'")
    && renderDiffWorkflowSource.includes('chromium@"${RHWP_CHROMIUM_BUILD_ID}"'),
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
