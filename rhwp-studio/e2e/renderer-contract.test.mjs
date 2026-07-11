import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

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
