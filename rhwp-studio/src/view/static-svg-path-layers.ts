import type { LayerAffineTransform } from '@/core/types';
import { parseSupportedCssColor } from './canvaskit/css-color.ts';

type StaticSvgFillRule = 'evenodd' | 'nonzero';
type StaticSvgLineJoin = 'miter' | 'round' | 'bevel';
type StaticSvgLineCap = 'butt' | 'round' | 'square';

export type StaticSvgPathLayer = {
  pathData: string;
  fill: string | null;
  fillRule?: StaticSvgFillRule;
  opacity: number;
  stroke?: StaticSvgStrokeLayer;
  transform?: LayerAffineTransform;
};

export type StaticSvgStrokeLayer = {
  color: string;
  opacity: number;
  width: number;
  lineJoin: StaticSvgLineJoin;
  lineCap: StaticSvgLineCap;
  miterLimit: number;
  dashArray?: number[];
  dashOffset: number;
};

export type StaticSvgTextLayer = {
  text: string;
  x: number;
  y: number;
  fill: string;
  opacity: number;
  fontFamily: string;
  fontSize: number;
  fontWeight: 'normal' | 'bold';
  fontStyle: 'normal' | 'italic';
  textAnchor: 'start' | 'middle' | 'end';
  dominantBaseline: 'alphabetic' | 'middle';
  transform?: LayerAffineTransform;
};

type StaticSvgPaintState = {
  color: string;
  fill: string | null;
  fillRuleValue: string | null;
  fillOpacity: number;
  stroke: string | null;
  strokeOpacity: number;
  strokeWidth: number;
  strokeLineJoin: StaticSvgLineJoin;
  strokeLineCap: StaticSvgLineCap;
  strokeMiterLimit: number;
  strokeDashArray: number[] | null;
  strokeDashOffset: number;
  fontFamily: string;
  fontSize: number;
  fontWeight: 'normal' | 'bold';
  fontStyle: 'normal' | 'italic';
  textAnchor: 'start' | 'middle' | 'end';
  dominantBaseline: 'alphabetic' | 'middle';
  transform?: LayerAffineTransform;
};

const STATIC_SVG_UNSUPPORTED_INDIRECT_PAINT_VALUES = new Set([
  'context-fill',
  'context-stroke',
  'inherit',
  'initial',
  'revert',
  'revert-layer',
  'unset',
]);

export function parseStaticSvgPathLayers(
  fragment: string,
  currentColor = '#000000',
): StaticSvgPathLayer[] {
  return parseStaticSvgFragmentLayers(fragment, currentColor).paths;
}

export function parseStaticSvgTextLayers(fragment: string): StaticSvgTextLayer[] {
  return parseStaticSvgFragmentLayers(fragment, '#000000').texts;
}

function parseStaticSvgFragmentLayers(fragment: string, currentColor: string): {
  paths: StaticSvgPathLayer[];
  texts: StaticSvgTextLayer[];
} {
  const parserFragment = staticSvgMarkupWithoutComments(fragment);
  if (parserFragment === null || hasStaticSvgUnsupportedMarkup(parserFragment)) {
    return { paths: [], texts: [] };
  }

  const paths: StaticSvgPathLayer[] = [];
  const texts: StaticSvgTextLayer[] = [];
  const paintStateStack: StaticSvgPaintState[] = [{
    color: currentColor,
    fill: null,
    fillRuleValue: null,
    fillOpacity: 1,
    stroke: null,
    strokeOpacity: 1,
    strokeWidth: 1,
    strokeLineJoin: 'miter',
    strokeLineCap: 'butt',
    strokeMiterLimit: 4,
    strokeDashArray: null,
    strokeDashOffset: 0,
    fontFamily: 'sans-serif',
    fontSize: 12,
    fontWeight: 'normal',
    fontStyle: 'normal',
    textAnchor: 'start',
    dominantBaseline: 'alphabetic',
  }];
  const tagPattern = /<\s*(\/?)\s*([A-Za-z][A-Za-z0-9:-]*)\b([^>]*)>/g;
  let ignoredElementDepth = 0;
  let openText: {
    attributes: Map<string, string>;
    state: StaticSvgPaintState;
    contentStart: number;
  } | null = null;
  for (const match of parserFragment.matchAll(tagPattern)) {
    const isClosingTag = match[1] === '/';
    const elementName = match[2].toLowerCase();
    const rawAttributes = match[3] ?? '';
    if (isClosingTag) {
      if (openText) {
        if (elementName !== 'text') {
          return { paths: [], texts: [] };
        }
        const content = parserFragment.slice(openText.contentStart, match.index);
        const textLayer = staticSvgTextLayer(
          content,
          openText.attributes,
          openText.state,
        );
        if (textLayer) {
          texts.push(textLayer);
        }
        openText = null;
        continue;
      }
      if (ignoredElementDepth > 0) {
        ignoredElementDepth -= 1;
        continue;
      }
      if ((elementName === 'svg' || elementName === 'g') && paintStateStack.length > 1) {
        paintStateStack.pop();
      }
      continue;
    }
    if (openText) {
      return { paths: [], texts: [] };
    }
    const supportedAttributes = staticSvgSupportedAttributes(elementName);
    if (!supportedAttributes) {
      return { paths: [], texts: [] };
    }
    const isSelfClosing = /\/\s*$/.test(rawAttributes);

    const attributes = new Map<string, string>();
    const attributePattern = /([A-Za-z_][A-Za-z0-9:._-]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+))/g;
    let remainingAttributes = rawAttributes;
    for (const attributeMatch of rawAttributes.matchAll(attributePattern)) {
      const rawName = attributeMatch[1].trim().toLowerCase();
      if (attributes.has(rawName)) {
        return { paths: [], texts: [] };
      }
      const value = attributeMatch[2] ?? attributeMatch[3] ?? attributeMatch[4] ?? '';
      const decodedValue = decodeStaticSvgXmlEntities(value);
      if (decodedValue === null) {
        return { paths: [], texts: [] };
      }
      attributes.set(rawName, decodedValue.trim());
      remainingAttributes = remainingAttributes.replace(attributeMatch[0], '');
    }
    if (remainingAttributes.replace(/\/\s*$/, '').trim().length > 0) {
      return { paths: [], texts: [] };
    }

    for (const [name, value] of attributes) {
      if (!isStaticSvgAttributeSupported(elementName, supportedAttributes, name, value)) {
        return { paths: [], texts: [] };
      }
    }
    if (
      ignoredElementDepth > 0
      || elementName === 'defs'
      || elementName === 'title'
      || elementName === 'desc'
      || elementName === 'metadata'
    ) {
      if (!isSelfClosing) {
        ignoredElementDepth += 1;
      }
      continue;
    }
    if (elementName === 'svg' || elementName === 'g') {
      if (!isSelfClosing) {
        paintStateStack.push(staticSvgPaintStateFromMap(
          paintStateStack[paintStateStack.length - 1],
          attributes,
          true,
        ));
      }
      continue;
    }
    if (elementName === 'text') {
      if (isSelfClosing) {
        continue;
      }
      openText = {
        attributes,
        state: paintStateStack[paintStateStack.length - 1],
        contentStart: (match.index ?? 0) + match[0].length,
      };
      continue;
    }

    let pathData: string | null = null;
    if (elementName === 'path') {
      pathData = attributes.get('d')?.trim() || null;
    } else if (elementName === 'circle') {
      const cx = svgNumber(attributes.get('cx') ?? '0') ?? 0;
      const cy = svgNumber(attributes.get('cy') ?? '0') ?? 0;
      const r = attributes.has('r') ? svgNumber(attributes.get('r') ?? '') : null;
      if (r !== null && r > 0) {
        pathData = `M${cx - r} ${cy}A${r} ${r} 0 1 0 ${cx + r} ${cy}A${r} ${r} 0 1 0 ${cx - r} ${cy}Z`;
      }
    } else if (elementName === 'ellipse') {
      const cx = svgNumber(attributes.get('cx') ?? '0') ?? 0;
      const cy = svgNumber(attributes.get('cy') ?? '0') ?? 0;
      const rx = attributes.has('rx') ? svgNumber(attributes.get('rx') ?? '') : null;
      const ry = attributes.has('ry') ? svgNumber(attributes.get('ry') ?? '') : null;
      if (rx !== null && ry !== null && rx > 0 && ry > 0) {
        pathData = `M${cx - rx} ${cy}A${rx} ${ry} 0 1 0 ${cx + rx} ${cy}A${rx} ${ry} 0 1 0 ${cx - rx} ${cy}Z`;
      }
    } else if (elementName === 'polygon' || elementName === 'polyline') {
      const points = svgPointList(attributes.get('points') ?? '');
      if (points.length >= 3) {
        const [first, ...rest] = points;
        const closePath = elementName === 'polygon' ? 'Z' : '';
        pathData = `M${first[0]} ${first[1]}${rest.map(([x, y]) => `L${x} ${y}`).join('')}${closePath}`;
      }
    } else if (elementName === 'line') {
      const x1 = svgNumber(attributes.get('x1') ?? '0') ?? 0;
      const y1 = svgNumber(attributes.get('y1') ?? '0') ?? 0;
      const x2 = svgNumber(attributes.get('x2') ?? '0') ?? 0;
      const y2 = svgNumber(attributes.get('y2') ?? '0') ?? 0;
      pathData = `M${x1} ${y1}L${x2} ${y2}`;
    } else if (elementName === 'rect') {
      const x = svgNumber(attributes.get('x') ?? '0') ?? 0;
      const y = svgNumber(attributes.get('y') ?? '0') ?? 0;
      const width = attributes.has('width') ? svgNumber(attributes.get('width') ?? '') : null;
      const height = attributes.has('height') ? svgNumber(attributes.get('height') ?? '') : null;
      const rx = attributes.has('rx') ? svgNumber(attributes.get('rx') ?? '') : null;
      const ry = attributes.has('ry') ? svgNumber(attributes.get('ry') ?? '') : null;
      pathData = staticSvgRectPathData(x, y, width, height, rx, ry);
    }
    if (!pathData) {
      continue;
    }

    const currentState = paintStateStack[paintStateStack.length - 1];
    const fill = staticSvgMapPresentationAttribute(attributes, 'fill');
    const strokeValue = staticSvgMapPresentationAttribute(attributes, 'stroke');
    const opacityValue = staticSvgMapPresentationAttribute(attributes, 'opacity');
    const fillOpacityValue = staticSvgMapPresentationAttribute(attributes, 'fill-opacity');
    const strokeOpacityValue = staticSvgMapPresentationAttribute(attributes, 'stroke-opacity');
    const shapeColor = staticSvgMapPresentationAttribute(attributes, 'color') ?? currentState.color;
    const strokeWidthValue = staticSvgMapPresentationAttribute(attributes, 'stroke-width');
    const strokeLineJoinValue = staticSvgMapPresentationAttribute(attributes, 'stroke-linejoin');
    const strokeLineCapValue = staticSvgMapPresentationAttribute(attributes, 'stroke-linecap');
    const strokeMiterLimitValue = staticSvgMapPresentationAttribute(attributes, 'stroke-miterlimit');
    const strokeDashArrayValue = staticSvgMapPresentationAttribute(attributes, 'stroke-dasharray');
    const strokeDashOffsetValue = staticSvgMapPresentationAttribute(attributes, 'stroke-dashoffset');
    const fillRuleValue = staticSvgMapPresentationAttribute(attributes, 'fill-rule') ?? currentState.fillRuleValue;
    const resolvedFill = resolveStaticSvgPaintValue(fill ?? currentState.fill ?? '#000000', shapeColor);
    const resolvedStroke = strokeValue ?? currentState.stroke;
    const shapeOpacity = svgOpacity(opacityValue);
    const stroke = staticSvgStrokeLayer(
      resolvedStroke,
      strokeWidthValue,
      strokeOpacityValue,
      strokeLineJoinValue,
      strokeLineCapValue,
      strokeMiterLimitValue,
      strokeDashArrayValue,
      strokeDashOffsetValue,
      currentState,
      shapeOpacity,
      shapeColor,
    );
    const shouldFill = elementName !== 'line' && resolvedFill.trim().toLowerCase() !== 'none';
    const transform = staticSvgComposeTransforms(
      currentState.transform,
      parseStaticSvgTransform(attributes.get('transform')),
    );
    paths.push({
      pathData,
      fill: shouldFill ? resolvedFill : null,
      fillRule: svgFillRule(fillRuleValue),
      opacity: shapeOpacity * (fillOpacityValue === null ? currentState.fillOpacity : svgOpacity(fillOpacityValue)),
      stroke,
      transform,
    });
  }
  if (openText) {
    return { paths: [], texts: [] };
  }
  return { paths, texts };
}

function staticSvgMarkupWithoutComments(fragment: string): string | null {
  let stripped = '';
  let cursor = 0;
  while (cursor < fragment.length) {
    const commentStart = fragment.indexOf('<!--', cursor);
    const strayCommentEnd = fragment.indexOf('-->', cursor);
    if (strayCommentEnd !== -1 && (commentStart === -1 || strayCommentEnd < commentStart)) {
      return null;
    }
    if (commentStart === -1) {
      return stripped + fragment.slice(cursor);
    }
    stripped += fragment.slice(cursor, commentStart);
    const commentEnd = fragment.indexOf('-->', commentStart + 4);
    if (commentEnd === -1) {
      return null;
    }
    if (fragment.slice(commentStart + 4, commentEnd).includes('--')) {
      return null;
    }
    cursor = commentEnd + 3;
  }
  return stripped;
}

function staticSvgStyleWithoutComments(style: string): string | null {
  let stripped = '';
  let cursor = 0;
  while (cursor < style.length) {
    const commentStart = style.indexOf('/*', cursor);
    const strayCommentEnd = style.indexOf('*/', cursor);
    if (strayCommentEnd !== -1 && (commentStart === -1 || strayCommentEnd < commentStart)) {
      return null;
    }
    if (commentStart === -1) {
      return stripped + style.slice(cursor);
    }
    stripped += `${style.slice(cursor, commentStart)} `;
    const commentEnd = style.indexOf('*/', commentStart + 2);
    if (commentEnd === -1) {
      return null;
    }
    cursor = commentEnd + 2;
  }
  return stripped;
}

function decodeStaticSvgXmlEntities(value: string): string | null {
  let decodedValue = '';
  let cursor = 0;
  while (cursor < value.length) {
    const entityStart = value.indexOf('&', cursor);
    if (entityStart < 0) {
      return decodedValue + value.slice(cursor);
    }
    decodedValue += value.slice(cursor, entityStart);
    const entityEnd = value.indexOf(';', entityStart + 1);
    if (entityEnd < 0) {
      return null;
    }
    const entity = value.slice(entityStart + 1, entityEnd);
    let decodedEntity: string | null = null;
    if (entity === 'amp') {
      decodedEntity = '&';
    } else if (entity === 'lt') {
      decodedEntity = '<';
    } else if (entity === 'gt') {
      decodedEntity = '>';
    } else if (entity === 'quot') {
      decodedEntity = '"';
    } else if (entity === 'apos') {
      decodedEntity = "'";
    } else {
      const decimalEntity = /^#([0-9]+)$/.exec(entity);
      const hexEntity = /^#x([0-9a-fA-F]+)$/.exec(entity);
      const codePoint = decimalEntity
        ? Number(decimalEntity[1])
        : hexEntity
          ? Number.parseInt(hexEntity[1], 16)
          : Number.NaN;
      const isXmlCharacter = codePoint === 0x09
        || codePoint === 0x0a
        || codePoint === 0x0d
        || (codePoint >= 0x20 && codePoint <= 0xd7ff)
        || (codePoint >= 0xe000 && codePoint <= 0xfffd)
        || (codePoint >= 0x10000 && codePoint <= 0x10ffff);
      if (Number.isInteger(codePoint) && isXmlCharacter) {
        decodedEntity = String.fromCodePoint(codePoint);
      }
    }
    if (decodedEntity === null) {
      return null;
    }
    decodedValue += decodedEntity;
    cursor = entityEnd + 1;
  }
  return decodedValue;
}

function isStaticSvgTextContentSupported(text: string): boolean {
  return !text.includes('<') && decodeStaticSvgXmlEntities(text) !== null;
}

function hasStaticSvgUnsupportedMarkup(fragment: string): boolean {
  if (/<\s*\?/.test(fragment) || /<\s*!(?!\s*--)/.test(fragment) || fragment.includes(']]>')) {
    return true;
  }
  const openElementStack: string[] = [];
  const tagPattern = /<\s*(\/?)\s*([A-Za-z][A-Za-z0-9:-]*)\b([^>]*)>/g;
  let cursor = 0;
  for (const match of fragment.matchAll(tagPattern)) {
    if (!isStaticSvgTextContentSupported(fragment.slice(cursor, match.index))) {
      return true;
    }
    cursor = (match.index ?? 0) + match[0].length;
    const isClosingTag = match[1] === '/';
    const elementName = match[2].toLowerCase();
    const trailingContent = match[3] ?? '';
    if (!staticSvgSupportedAttributes(elementName)) {
      return true;
    }
    if (isClosingTag) {
      if (trailingContent.trim().length > 0 || openElementStack.pop() !== elementName) {
        return true;
      }
      continue;
    }
    if (!/\/\s*$/.test(trailingContent)) {
      openElementStack.push(elementName);
    }
  }
  return openElementStack.length > 0 || !isStaticSvgTextContentSupported(fragment.slice(cursor));
}

function staticSvgMapPresentationAttribute(attributes: Map<string, string>, name: string): string | null {
  const style = attributes.get('style');
  if (style) {
    const normalizedStyle = staticSvgStyleWithoutComments(style);
    let styleValue: string | null = null;
    for (const declaration of (normalizedStyle ?? '').split(';')) {
      const separator = declaration.indexOf(':');
      if (separator < 0) {
        continue;
      }
      if (declaration.slice(0, separator).trim().toLowerCase() === name.toLowerCase()) {
        styleValue = declaration.slice(separator + 1).trim();
      }
    }
    if (styleValue !== null) {
      return styleValue;
    }
  }
  return attributes.get(name) ?? null;
}

function staticSvgPaintStateFromMap(
  parent: StaticSvgPaintState,
  attributes: Map<string, string>,
  allowTransform: boolean,
): StaticSvgPaintState {
  const fillOpacityValue = staticSvgMapPresentationAttribute(attributes, 'fill-opacity');
  const strokeDashArray = svgStrokeDashArray(staticSvgMapPresentationAttribute(attributes, 'stroke-dasharray'));
  const strokeDashOffset = svgStrokeDashOffset(staticSvgMapPresentationAttribute(attributes, 'stroke-dashoffset'));
  return {
    color: staticSvgMapPresentationAttribute(attributes, 'color') ?? parent.color,
    fill: staticSvgMapPresentationAttribute(attributes, 'fill') ?? parent.fill,
    fillRuleValue: staticSvgMapPresentationAttribute(attributes, 'fill-rule') ?? parent.fillRuleValue,
    fillOpacity: fillOpacityValue === null ? parent.fillOpacity : svgOpacity(fillOpacityValue),
    stroke: staticSvgMapPresentationAttribute(attributes, 'stroke') ?? parent.stroke,
    strokeOpacity: staticSvgMapPresentationAttribute(attributes, 'stroke-opacity') === null
      ? parent.strokeOpacity
      : svgOpacity(staticSvgMapPresentationAttribute(attributes, 'stroke-opacity')),
    strokeWidth: svgNonNegativeNumber(staticSvgMapPresentationAttribute(attributes, 'stroke-width')) ?? parent.strokeWidth,
    strokeLineJoin: svgStrokeLineJoin(staticSvgMapPresentationAttribute(attributes, 'stroke-linejoin'))
      ?? parent.strokeLineJoin,
    strokeLineCap: svgStrokeLineCap(staticSvgMapPresentationAttribute(attributes, 'stroke-linecap'))
      ?? parent.strokeLineCap,
    strokeMiterLimit: svgPositiveNumber(staticSvgMapPresentationAttribute(attributes, 'stroke-miterlimit'))
      ?? parent.strokeMiterLimit,
    strokeDashArray: strokeDashArray === undefined ? parent.strokeDashArray : strokeDashArray,
    strokeDashOffset: strokeDashOffset === undefined ? parent.strokeDashOffset : strokeDashOffset,
    fontFamily: svgFontFamily(staticSvgMapPresentationAttribute(attributes, 'font-family')) ?? parent.fontFamily,
    fontSize: svgPositiveNumber(staticSvgMapPresentationAttribute(attributes, 'font-size')) ?? parent.fontSize,
    fontWeight: svgFontWeight(staticSvgMapPresentationAttribute(attributes, 'font-weight')) ?? parent.fontWeight,
    fontStyle: svgFontStyle(staticSvgMapPresentationAttribute(attributes, 'font-style')) ?? parent.fontStyle,
    textAnchor: svgTextAnchor(staticSvgMapPresentationAttribute(attributes, 'text-anchor')) ?? parent.textAnchor,
    dominantBaseline: svgDominantBaseline(staticSvgMapPresentationAttribute(attributes, 'dominant-baseline'))
      ?? parent.dominantBaseline,
    transform: allowTransform
      ? staticSvgComposeTransforms(parent.transform, parseStaticSvgTransform(attributes.get('transform')))
      : parent.transform,
  };
}

function staticSvgTextLayer(
  rawText: string,
  attributes: Map<string, string>,
  parentState: StaticSvgPaintState,
): StaticSvgTextLayer | null {
  const decodedText = decodeStaticSvgXmlEntities(rawText);
  if (decodedText === null) {
    return null;
  }
  const text = decodedText.replace(/\s+/g, ' ').trim();
  if (text.length === 0) {
    return null;
  }
  const state = staticSvgPaintStateFromMap(parentState, attributes, true);
  const x = svgNumber(attributes.get('x') ?? '0') ?? 0;
  const y = svgNumber(attributes.get('y') ?? '0') ?? 0;
  const opacity = svgOpacity(staticSvgMapPresentationAttribute(attributes, 'opacity'))
    * svgOpacity(staticSvgMapPresentationAttribute(attributes, 'fill-opacity'));
  if (!(opacity > 0)) {
    return null;
  }
  const fillValue = staticSvgMapPresentationAttribute(attributes, 'fill') ?? state.fill ?? '#000000';
  const fill = resolveStaticSvgPaintValue(fillValue, state.color);
  if (fill.trim().toLowerCase() === 'none') {
    return null;
  }
  return {
    text,
    x,
    y,
    fill,
    opacity,
    fontFamily: state.fontFamily,
    fontSize: state.fontSize,
    fontWeight: state.fontWeight,
    fontStyle: state.fontStyle,
    textAnchor: state.textAnchor,
    dominantBaseline: state.dominantBaseline,
    transform: state.transform,
  };
}

function staticSvgStrokeLayer(
  strokeValue: string | null,
  widthValue: string | null,
  opacityValue: string | null,
  lineJoinValue: string | null,
  lineCapValue: string | null,
  miterLimitValue: string | null,
  dashArrayValue: string | null,
  dashOffsetValue: string | null,
  currentState: StaticSvgPaintState,
  shapeOpacity: number,
  currentColor: string,
): StaticSvgStrokeLayer | undefined {
  const stroke = strokeValue ?? currentState.stroke;
  if (!stroke || stroke.trim().toLowerCase() === 'none') {
    return undefined;
  }
  const width = svgNonNegativeNumber(widthValue) ?? currentState.strokeWidth;
  if (!(width > 0)) {
    return undefined;
  }
  const opacity = shapeOpacity * (opacityValue === null ? currentState.strokeOpacity : svgOpacity(opacityValue));
  if (!(opacity > 0)) {
    return undefined;
  }
  const dashArray = svgStrokeDashArray(dashArrayValue);
  const dashOffset = svgStrokeDashOffset(dashOffsetValue);
  return {
    color: resolveStaticSvgPaintValue(stroke, currentColor),
    opacity,
    width,
    lineJoin: svgStrokeLineJoin(lineJoinValue) ?? currentState.strokeLineJoin,
    lineCap: svgStrokeLineCap(lineCapValue) ?? currentState.strokeLineCap,
    miterLimit: svgPositiveNumber(miterLimitValue) ?? currentState.strokeMiterLimit,
    dashArray: dashArray === undefined
      ? currentState.strokeDashArray ?? undefined
      : dashArray ?? undefined,
    dashOffset: dashOffset === undefined ? currentState.strokeDashOffset : dashOffset,
  };
}

function staticSvgComposeTransforms(
  parent: LayerAffineTransform | undefined,
  child: LayerAffineTransform | undefined,
): LayerAffineTransform | undefined {
  if (!parent) {
    return child;
  }
  if (!child) {
    return parent;
  }
  return {
    a: parent.a * child.a + parent.c * child.b,
    b: parent.b * child.a + parent.d * child.b,
    c: parent.a * child.c + parent.c * child.d,
    d: parent.b * child.c + parent.d * child.d,
    e: parent.a * child.e + parent.c * child.f + parent.e,
    f: parent.b * child.e + parent.d * child.f + parent.f,
  };
}

function staticSvgRectPathData(
  x: number,
  y: number,
  width: number | null,
  height: number | null,
  rxValue: number | null,
  ryValue: number | null,
): string | null {
  if (width === null || height === null || width <= 0 || height <= 0) {
    return null;
  }
  const rx = Math.min(Math.max(rxValue ?? ryValue ?? 0, 0), width / 2);
  const ry = Math.min(Math.max(ryValue ?? rxValue ?? 0, 0), height / 2);
  if (rx > 0 && ry > 0) {
    return `M${x + rx} ${y}H${x + width - rx}A${rx} ${ry} 0 0 1 ${x + width} ${y + ry}V${y + height - ry}A${rx} ${ry} 0 0 1 ${x + width - rx} ${y + height}H${x + rx}A${rx} ${ry} 0 0 1 ${x} ${y + height - ry}V${y + ry}A${rx} ${ry} 0 0 1 ${x + rx} ${y}Z`;
  }
  return `M${x} ${y}H${x + width}V${y + height}H${x}Z`;
}

function staticSvgSupportedAttributes(elementName: string): Set<string> | null {
  const paintAttributes = [
    'fill',
    'color',
    'fill-rule',
    'opacity',
    'fill-opacity',
    'stroke',
    'stroke-opacity',
    'stroke-width',
    'stroke-linejoin',
    'stroke-linecap',
    'stroke-miterlimit',
    'stroke-dasharray',
    'stroke-dashoffset',
    'style',
    'transform',
  ];
  if (elementName === 'path') {
    return new Set(['id', 'class', 'd', ...paintAttributes]);
  }
  if (elementName === 'rect') {
    return new Set(['id', 'class', 'x', 'y', 'width', 'height', 'rx', 'ry', ...paintAttributes]);
  }
  if (elementName === 'circle') {
    return new Set(['id', 'class', 'cx', 'cy', 'r', ...paintAttributes]);
  }
  if (elementName === 'ellipse') {
    return new Set(['id', 'class', 'cx', 'cy', 'rx', 'ry', ...paintAttributes]);
  }
  if (elementName === 'polygon' || elementName === 'polyline') {
    return new Set(['id', 'class', 'points', ...paintAttributes]);
  }
  if (elementName === 'line') {
    return new Set(['id', 'class', 'x1', 'y1', 'x2', 'y2', ...paintAttributes]);
  }
  if (elementName === 'text') {
    return new Set([
      'id',
      'class',
      'x',
      'y',
      'font-family',
      'font-size',
      'font-weight',
      'font-style',
      'text-anchor',
      'dominant-baseline',
      ...paintAttributes,
    ]);
  }
  if (elementName === 'svg') {
    return new Set([
      'id',
      'class',
      'xmlns',
      'xmlns:xlink',
      'xml:space',
      'viewbox',
      'width',
      'height',
      'x',
      'y',
      'version',
      ...paintAttributes,
    ]);
  }
  if (elementName === 'g') {
    return new Set(['id', 'class', 'xml:space', ...paintAttributes]);
  }
  if (elementName === 'title' || elementName === 'desc' || elementName === 'metadata') {
    return new Set(['id', 'class', 'xml:space']);
  }
  if (elementName === 'defs') {
    return new Set(['id', 'class', 'xml:space']);
  }
  return null;
}

function isStaticSvgAttributeSupported(
  elementName: string,
  supportedAttributes: Set<string>,
  name: string,
  value: string,
): boolean {
  if (!supportedAttributes.has(name)) {
    return isStaticSvgNonVisualAttributeSupported(name, value);
  }
  if (name === 'xmlns') {
    return value.trim() === 'http://www.w3.org/2000/svg';
  }
  if (name === 'xmlns:xlink') {
    return value.trim() === 'http://www.w3.org/1999/xlink';
  }
  if (name === 'xml:space') {
    const trimmedValue = value.trim();
    return trimmedValue === 'default' || trimmedValue === 'preserve';
  }
  if (name === 'id' || name === 'class') {
    return !/[<>`]/.test(value);
  }
  if (name === 'viewbox') {
    return isStaticSvgViewBoxValueSupported(value);
  }
  if (name === 'version') {
    const version = Number(value.trim());
    return Number.isFinite(version);
  }
  if (name === 'd') {
    return value.trim().length > 0;
  }
  if (name === 'fill' || name === 'stroke') {
    return isStaticSvgPaintValueSupported(value);
  }
  if (name === 'color') {
    return isStaticSvgColorValueSupported(value);
  }
  if (name === 'stroke-opacity') {
    return isStaticSvgOpacityValueSupported(value);
  }
  if (name === 'stroke-width') {
    return isStaticSvgNonNegativeNumericValueSupported(value);
  }
  if (name === 'stroke-miterlimit') {
    return isStaticSvgPositiveNumericValueSupported(value);
  }
  if (name === 'stroke-dasharray') {
    return svgStrokeDashArray(value) !== undefined;
  }
  if (name === 'stroke-dashoffset') {
    return svgStrokeDashOffset(value) !== undefined;
  }
  if (name === 'stroke-linejoin') {
    return svgStrokeLineJoin(value) !== null;
  }
  if (name === 'stroke-linecap') {
    return svgStrokeLineCap(value) !== null;
  }
  if (name === 'font-family') {
    return svgFontFamily(value) !== null;
  }
  if (name === 'font-size') {
    return isStaticSvgPositiveNumericValueSupported(value);
  }
  if (name === 'font-weight') {
    return svgFontWeight(value) !== null;
  }
  if (name === 'font-style') {
    return svgFontStyle(value) !== null;
  }
  if (name === 'text-anchor') {
    return svgTextAnchor(value) !== null;
  }
  if (name === 'dominant-baseline') {
    return svgDominantBaseline(value) !== null;
  }
  if (name === 'opacity') {
    return elementName === 'svg' || elementName === 'g'
      ? isStaticSvgIdentityOpacityValueSupported(value)
      : isStaticSvgOpacityValueSupported(value);
  }
  if (name === 'fill-opacity') {
    return isStaticSvgOpacityValueSupported(value);
  }
  if (name === 'fill-rule') {
    return isStaticSvgFillRuleValueSupported(value);
  }
  if (name === 'style') {
    return isStaticSvgStyleSupported(
      value,
      elementName !== 'svg' && elementName !== 'g',
      elementName === 'svg' || elementName === 'g',
    );
  }
  if (name === 'transform') {
    return parseStaticSvgTransform(value) !== undefined;
  }
  if (
    name === 'x'
    || name === 'y'
    || name === 'width'
    || name === 'height'
    || name === 'x1'
    || name === 'y1'
    || name === 'x2'
    || name === 'y2'
  ) {
    return isStaticSvgNumericValueSupported(value);
  }
  if (elementName === 'rect' && (name === 'rx' || name === 'ry')) {
    return isStaticSvgNonNegativeNumericValueSupported(value);
  }
  if (name === 'cx' || name === 'cy' || name === 'r' || name === 'rx' || name === 'ry') {
    return isStaticSvgNumericValueSupported(value);
  }
  if (name === 'points') {
    return svgPointList(value).length >= 3;
  }
  return elementName === 'g';
}

function isStaticSvgNonVisualAttributeSupported(name: string, value: string): boolean {
  const normalizedName = name.trim().toLowerCase();
  if (/[<>`]/.test(value)) {
    return false;
  }
  if (/^aria-[a-z0-9_-]+$/.test(normalizedName)) {
    return true;
  }
  if (/^data-[a-z0-9_.:-]+$/.test(normalizedName)) {
    return true;
  }
  if (normalizedName === 'role') {
    const trimmedValue = value.trim();
    return /^[a-z][a-z0-9_-]*(?:\s+[a-z][a-z0-9_-]*)*$/i.test(trimmedValue);
  }
  if (normalizedName === 'focusable') {
    return ['true', 'false', 'auto'].includes(value.trim().toLowerCase());
  }
  return false;
}

function isStaticSvgNumericValueSupported(value: string): boolean {
  return svgNumber(value) !== null;
}

function isStaticSvgNonNegativeNumericValueSupported(value: string): boolean {
  const number = svgNumber(value);
  return number !== null && number >= 0;
}

function isStaticSvgPositiveNumericValueSupported(value: string): boolean {
  const number = svgNumber(value);
  return number !== null && number > 0;
}

function isStaticSvgViewBoxValueSupported(value: string): boolean {
  const numbers = value
    .trim()
    .split(/[\s,]+/)
    .filter((part) => part.length > 0)
    .map((part) => Number(part));
  return numbers.length === 4
    && numbers.every((number) => Number.isFinite(number))
    && numbers[2] > 0
    && numbers[3] > 0;
}

function svgPointList(value: string): Array<[number, number]> {
  const numbers = value
    .trim()
    .split(/[\s,]+/)
    .filter((part) => part.length > 0)
    .map((part) => Number(part));
  if (numbers.length < 6 || numbers.length % 2 !== 0 || numbers.some((number) => !Number.isFinite(number))) {
    return [];
  }
  const points: Array<[number, number]> = [];
  for (let index = 0; index < numbers.length; index += 2) {
    points.push([numbers[index], numbers[index + 1]]);
  }
  return points;
}

function parseStaticSvgTransform(value: string | null | undefined): LayerAffineTransform | undefined {
  if (value === null || value === undefined) {
    return undefined;
  }
  const source = value.trim();
  if (source.length === 0) {
    return undefined;
  }

  let transform: LayerAffineTransform = { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 };
  let cursor = 0;
  const transformPattern = /([A-Za-z][A-Za-z0-9]*)\s*\(([^)]*)\)/g;
  for (const match of source.matchAll(transformPattern)) {
    const prefix = source.slice(cursor, match.index);
    if (!/^[\s,]*$/.test(prefix)) {
      return undefined;
    }
    cursor = (match.index ?? 0) + match[0].length;

    const rawArguments = match[2].trim();
    const numbers = rawArguments.length === 0
      ? []
      : rawArguments
        .split(/[\s,]+/)
        .filter((part) => part.length > 0)
        .map((part) => Number(part));
    if (numbers.some((number) => !Number.isFinite(number))) {
      return undefined;
    }

    const operation = match[1].toLowerCase();
    let next: LayerAffineTransform | undefined;
    if (operation === 'matrix' && numbers.length === 6) {
      next = { a: numbers[0], b: numbers[1], c: numbers[2], d: numbers[3], e: numbers[4], f: numbers[5] };
    } else if (operation === 'translate' && (numbers.length === 1 || numbers.length === 2)) {
      next = { a: 1, b: 0, c: 0, d: 1, e: numbers[0], f: numbers[1] ?? 0 };
    } else if (operation === 'scale' && (numbers.length === 1 || numbers.length === 2)) {
      next = { a: numbers[0], b: 0, c: 0, d: numbers[1] ?? numbers[0], e: 0, f: 0 };
    } else if (operation === 'rotate' && (numbers.length === 1 || numbers.length === 3)) {
      const radians = numbers[0] * Math.PI / 180;
      const cos = Math.cos(radians);
      const sin = Math.sin(radians);
      if (numbers.length === 3) {
        const cx = numbers[1];
        const cy = numbers[2];
        next = {
          a: cos,
          b: sin,
          c: -sin,
          d: cos,
          e: cx - cos * cx + sin * cy,
          f: cy - sin * cx - cos * cy,
        };
      } else {
        next = { a: cos, b: sin, c: -sin, d: cos, e: 0, f: 0 };
      }
    } else if (operation === 'skewx' && numbers.length === 1) {
      next = { a: 1, b: 0, c: Math.tan(numbers[0] * Math.PI / 180), d: 1, e: 0, f: 0 };
    } else if (operation === 'skewy' && numbers.length === 1) {
      next = { a: 1, b: Math.tan(numbers[0] * Math.PI / 180), c: 0, d: 1, e: 0, f: 0 };
    } else {
      return undefined;
    }
    if (!Object.values(next).every((number) => Number.isFinite(number))) {
      return undefined;
    }
    transform = staticSvgComposeTransforms(transform, next) ?? transform;
  }
  if (!/^[\s,]*$/.test(source.slice(cursor)) || cursor === 0) {
    return undefined;
  }
  return transform;
}

function isStaticSvgStyleSupported(style: string, allowOpacity: boolean, allowIdentityOpacity = false): boolean {
  const normalizedStyle = staticSvgStyleWithoutComments(style);
  if (normalizedStyle === null) {
    return false;
  }
  const supportedProperties = new Set([
    'fill',
    'color',
    'fill-rule',
    'fill-opacity',
    'stroke',
    'stroke-opacity',
    'stroke-width',
    'stroke-linejoin',
    'stroke-linecap',
    'stroke-miterlimit',
    'stroke-dasharray',
    'stroke-dashoffset',
    'font-family',
    'font-size',
    'font-weight',
    'font-style',
    'text-anchor',
    'dominant-baseline',
  ]);
  if (allowOpacity || allowIdentityOpacity) {
    supportedProperties.add('opacity');
  }
  for (const declaration of normalizedStyle.split(';')) {
    const separator = declaration.indexOf(':');
    if (separator < 0) {
      if (declaration.trim().length > 0) {
        return false;
      }
      continue;
    }
    const property = declaration.slice(0, separator).trim().toLowerCase();
    const value = declaration.slice(separator + 1).trim();
    if (!supportedProperties.has(property)) {
      return false;
    }
    if ((property === 'fill' || property === 'stroke') && !isStaticSvgPaintValueSupported(value)) {
      return false;
    }
    if (property === 'color' && !isStaticSvgColorValueSupported(value)) {
      return false;
    }
    if (property === 'opacity' && allowIdentityOpacity && !allowOpacity) {
      if (!isStaticSvgIdentityOpacityValueSupported(value)) {
        return false;
      }
      continue;
    }
    if (
      (property === 'opacity' || property === 'fill-opacity' || property === 'stroke-opacity')
      && !isStaticSvgOpacityValueSupported(value)
    ) {
      return false;
    }
    if (property === 'stroke-width' && !isStaticSvgNonNegativeNumericValueSupported(value)) {
      return false;
    }
    if (property === 'stroke-miterlimit' && !isStaticSvgPositiveNumericValueSupported(value)) {
      return false;
    }
    if (property === 'stroke-dasharray' && svgStrokeDashArray(value) === undefined) {
      return false;
    }
    if (property === 'stroke-dashoffset' && svgStrokeDashOffset(value) === undefined) {
      return false;
    }
    if (property === 'stroke-linejoin' && svgStrokeLineJoin(value) === null) {
      return false;
    }
    if (property === 'stroke-linecap' && svgStrokeLineCap(value) === null) {
      return false;
    }
    if (property === 'fill-rule' && !isStaticSvgFillRuleValueSupported(value)) {
      return false;
    }
    if (property === 'font-family' && svgFontFamily(value) === null) {
      return false;
    }
    if (property === 'font-size' && !isStaticSvgPositiveNumericValueSupported(value)) {
      return false;
    }
    if (property === 'font-weight' && svgFontWeight(value) === null) {
      return false;
    }
    if (property === 'font-style' && svgFontStyle(value) === null) {
      return false;
    }
    if (property === 'text-anchor' && svgTextAnchor(value) === null) {
      return false;
    }
    if (property === 'dominant-baseline' && svgDominantBaseline(value) === null) {
      return false;
    }
  }
  return true;
}

function isStaticSvgPaintValueSupported(value: string): boolean {
  const trimmed = value.trim();
  const normalized = trimmed.toLowerCase();
  if (
    normalized.length === 0
    || STATIC_SVG_UNSUPPORTED_INDIRECT_PAINT_VALUES.has(normalized)
    || /\burl\s*\(/.test(normalized)
    || /\bvar\s*\(/.test(normalized)
  ) {
    return false;
  }
  if (normalized === 'none' || normalized === 'currentcolor') {
    return true;
  }
  return parseSupportedCssColor(trimmed) !== null;
}

function isStaticSvgColorValueSupported(value: string): boolean {
  const trimmed = value.trim();
  const normalized = trimmed.toLowerCase();
  return normalized !== 'none'
    && normalized !== 'currentcolor'
    && isStaticSvgPaintValueSupported(trimmed);
}

function resolveStaticSvgPaintValue(value: string, currentColor: string): string {
  return value.trim().toLowerCase() === 'currentcolor' ? currentColor : value;
}

function isStaticSvgOpacityValueSupported(value: string): boolean {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return false;
  }
  const numeric = trimmed.endsWith('%') ? trimmed.slice(0, -1).trim() : trimmed;
  return numeric.length > 0 && Number.isFinite(Number(numeric));
}

function isStaticSvgIdentityOpacityValueSupported(value: string): boolean {
  const trimmed = value.trim();
  if (!isStaticSvgOpacityValueSupported(trimmed)) {
    return false;
  }
  const numeric = trimmed.endsWith('%') ? Number(trimmed.slice(0, -1).trim()) / 100 : Number(trimmed);
  return numeric === 1;
}

function isStaticSvgFillRuleValueSupported(value: string): boolean {
  const normalized = value.trim().toLowerCase();
  return normalized === 'nonzero' || normalized === 'evenodd';
}

function svgOpacity(value: string | null): number {
  if (!value) {
    return 1;
  }
  const trimmed = value.trim();
  const parsed = Number.parseFloat(trimmed);
  if (!Number.isFinite(parsed)) {
    return 1;
  }
  const unitValue = trimmed.endsWith('%') ? parsed / 100 : parsed;
  return Math.max(0, Math.min(1, unitValue));
}

function svgNumber(value: string): number | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return null;
  }
  const normalized = trimmed.toLowerCase();
  const numeric = normalized.endsWith('px') ? trimmed.slice(0, -2).trim() : trimmed;
  if (numeric.length === 0) {
    return null;
  }
  const number = Number(numeric);
  return Number.isFinite(number) ? number : null;
}

function svgPositiveNumber(value: string | null): number | null {
  if (value === null) {
    return null;
  }
  const number = svgNumber(value);
  return number !== null && number > 0 ? number : null;
}

function svgNonNegativeNumber(value: string | null): number | null {
  if (value === null) {
    return null;
  }
  const number = svgNumber(value);
  return number !== null && number >= 0 ? number : null;
}

function svgStrokeLineJoin(value: string | null): StaticSvgLineJoin | null {
  const normalized = value?.trim().toLowerCase();
  if (normalized === 'miter' || normalized === 'round' || normalized === 'bevel') {
    return normalized;
  }
  return null;
}

function svgStrokeLineCap(value: string | null): StaticSvgLineCap | null {
  const normalized = value?.trim().toLowerCase();
  if (normalized === 'butt' || normalized === 'round' || normalized === 'square') {
    return normalized;
  }
  return null;
}

function svgFontFamily(value: string | null): string | null {
  const trimmed = value?.trim();
  if (!trimmed || /[<>`]/.test(trimmed)) {
    return null;
  }
  return trimmed.split(',')[0].trim().replace(/^['"]|['"]$/g, '') || null;
}

function svgFontWeight(value: string | null): 'normal' | 'bold' | null {
  if (value === null) {
    return null;
  }
  const normalized = value?.trim().toLowerCase();
  if (
    normalized === ''
    || normalized === 'normal'
    || normalized === '400'
  ) {
    return 'normal';
  }
  if (normalized === 'bold' || normalized === '700' || normalized === '600') {
    return 'bold';
  }
  return null;
}

function svgFontStyle(value: string | null): 'normal' | 'italic' | null {
  if (value === null) {
    return null;
  }
  const normalized = value?.trim().toLowerCase();
  if (normalized === '' || normalized === 'normal') {
    return 'normal';
  }
  if (normalized === 'italic' || normalized === 'oblique') {
    return 'italic';
  }
  return null;
}

function svgTextAnchor(value: string | null): 'start' | 'middle' | 'end' | null {
  if (value === null) {
    return null;
  }
  const normalized = value?.trim().toLowerCase();
  if (
    normalized === ''
    || normalized === 'start'
    || normalized === 'middle'
    || normalized === 'end'
  ) {
    return (normalized || 'start') as 'start' | 'middle' | 'end';
  }
  return null;
}

function svgDominantBaseline(value: string | null): 'alphabetic' | 'middle' | null {
  if (value === null) {
    return null;
  }
  const normalized = value?.trim().toLowerCase();
  if (
    normalized === ''
    || normalized === 'auto'
    || normalized === 'alphabetic'
    || normalized === 'baseline'
  ) {
    return 'alphabetic';
  }
  if (normalized === 'middle' || normalized === 'central') {
    return 'middle';
  }
  return null;
}

function svgStrokeDashArray(value: string | null): number[] | null | undefined {
  if (value === null) {
    return undefined;
  }
  const trimmed = value.trim();
  if (trimmed.toLowerCase() === 'none') {
    return null;
  }
  const values = trimmed.split(/[\s,]+/).filter((part) => part.length > 0);
  if (values.length === 0) {
    return undefined;
  }
  const parsed = values.map((part) => svgNumber(part));
  if (parsed.some((part) => part === null || part < 0) || !parsed.some((part) => part !== null && part > 0)) {
    return undefined;
  }
  const dashValues = parsed as number[];
  return dashValues.length % 2 === 0 ? dashValues : [...dashValues, ...dashValues];
}

function svgStrokeDashOffset(value: string | null): number | undefined {
  if (value === null) {
    return undefined;
  }
  return svgNumber(value) ?? undefined;
}

function svgFillRule(value: string | null): StaticSvgFillRule | undefined {
  return value?.trim().toLowerCase() === 'evenodd' ? 'evenodd' : undefined;
}
