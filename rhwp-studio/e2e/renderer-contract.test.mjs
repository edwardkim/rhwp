import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(studioRoot, '..');
const canvaskitPath = path.join(studioRoot, 'src/view/canvaskit-renderer.ts');
const canvaskitDirectory = path.join(studioRoot, 'src/view/canvaskit');
const layerTypesPath = path.join(studioRoot, 'src/core/types.ts');
const textIrV2DocPath = path.join(repoRoot, 'docs/text-ir-v2.md');
const canvaskitParityPlanDocPath = path.join(repoRoot, 'docs/canvaskit-parity-implementation.md');

const canvaskitSource = fs.readFileSync(canvaskitPath, 'utf8');
const layerTypesSource = fs.readFileSync(layerTypesPath, 'utf8');
const textIrV2DocSource = fs.readFileSync(textIrV2DocPath, 'utf8');
const canvaskitParityPlanDocSource = fs.readFileSync(canvaskitParityPlanDocPath, 'utf8');
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
];

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
  'equation',
  'glyphRun',
  'rawSvg',
  'tabLeader',
  'textControlMark',
  'textDecoration',
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
const renderTextRunBody = extractMethodBody(canvaskitSource, 'renderTextRun');
const renderGlyphOutlineBody = extractMethodBody(canvaskitSource, 'renderGlyphOutline');
const renderColorPaintGraphNodeBody = extractMethodBody(canvaskitSource, 'renderColorPaintGraphNode');
const recordTextRunCoverageGapsBody = extractMethodBody(canvaskitSource, 'recordTextRunCoverageGaps');

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
requireSnippet(
  renderTextRunBody,
  /this\.recordTextRunCoverageGaps\(op\);[\s\S]*?canvas\.drawText\(op\.text, x, y, paint, font\)/,
  'textRun replay should record unsupported effect diagnostics before drawing the compatibility text',
);
requireSnippet(
  renderTextRunBody,
  /const rotation = op\.rotation \?\? 0;[\s\S]*?if \(rotation !== 0\) \{[\s\S]*?canvas\.rotate\(rotation, x, y\);[\s\S]*?\}/,
  'textRun replay should keep rotation on the direct CanvasKit path',
);
for (const expectedTextRunGap of [
  'textRun:verticalText',
  'textRun:textDecoration',
  'textRun:emphasisDot',
  'textRun:outlineTextEffect',
  'textRun:shadowTextEffect',
  'textRun:embossTextEffect',
  'textRun:engraveTextEffect',
  'textRun:superscriptTextEffect',
  'textRun:subscriptTextEffect',
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
  /visited\.has\(nodeId\)[\s\S]*?unsupportedColorGlyph[\s\S]*?return;[\s\S]*?visited\.add\(nodeId\);/,
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

console.log('renderer backend contract guard passed');
