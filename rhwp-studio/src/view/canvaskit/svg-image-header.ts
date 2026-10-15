import type { EncodedImageHeader } from './image-header.ts';

type SvgIntrinsicLength =
  | { kind: 'missing' | 'relative' }
  | { kind: 'absolute'; value: number };

export function parseSvgImageHeader(bytes: Uint8Array): EncodedImageHeader | null {
  if (bytes.byteLength > 4 * 1024 * 1024) return null;
  let source: string;
  try {
    source = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  } catch {
    return null;
  }
  if (source.charCodeAt(0) === 0xfeff) {
    source = source.slice(1);
  }
  source = source.replace(/<!--[\s\S]*?-->/g, '');
  if (source.includes('<!--') || /<\?(?!xml\s)/i.test(source)) return null;
  let elements = 0;
  let depth = 0;
  for (const tag of source.matchAll(/<\s*(\/?)\s*[A-Za-z_][^>]*>/g)) {
    if (tag[1] === '/') depth -= 1;
    else {
      if (++elements > 4096) return null;
      if (!/\/\s*>$/.test(tag[0]) && ++depth > 64) return null;
    }
    if (depth < 0) return null;
  }
  if (depth !== 0) return null;
  if (source.toLowerCase().includes('<!doctype')
    || /<(?:[\w.-]+:)?(?:script|animate\w*|set|foreignObject|image|feImage)\b/i.test(source)
    || /\bon\w+\s*=/i.test(source)
    || /@|\/\*|&|<!ENTITY/i.test(source)
    // CSS escapes / XML character references can hide url(), @import or a URI scheme.
    // The producer emits literal paths/colors; reject these spellings before browser decoding.
    || /\\|&#|\bxml:base\s*=/i.test(source)
    || /\b(?:href|xlink:href)\s*=\s*(['"])(?!#)[^'"]*\1/i.test(source)
    || [...source.matchAll(/url\(([^)]*)\)/gi)].some(match =>
      !match[1].trim().replace(/^(['"])(.*)\1$/, '$2').startsWith('#'))) {
    return null;
  }

  let cursor = 0;
  while (true) {
    cursor = skipXmlWhitespace(source, cursor);
    if (source.startsWith('<!--', cursor)) {
      const end = source.indexOf('-->', cursor + 4);
      if (end < 0) {
        return null;
      }
      cursor = end + 3;
      continue;
    }
    if (source.startsWith('<?', cursor)) {
      const end = source.indexOf('?>', cursor + 2);
      if (end < 0) {
        return null;
      }
      cursor = end + 2;
      continue;
    }
    break;
  }

  if (source[cursor] !== '<') {
    return null;
  }
  const tagEnd = findXmlStartTagEnd(source, cursor);
  if (tagEnd < 0) {
    return null;
  }
  const tag = source.slice(cursor + 1, tagEnd);
  const nameMatch = /^([A-Za-z_][A-Za-z0-9_.-]*:)?svg(?=\s|\/?$)/.exec(tag);
  if (nameMatch === null) {
    return null;
  }

  let width: SvgIntrinsicLength = { kind: 'missing' };
  let height: SvgIntrinsicLength = { kind: 'missing' };
  let viewBox: readonly [number, number] | null = null;
  const seen = new Set<string>();
  let attributeCursor = nameMatch[0].length;
  while (attributeCursor < tag.length) {
    attributeCursor = skipXmlWhitespace(tag, attributeCursor);
    if (attributeCursor >= tag.length || tag[attributeCursor] === '/') {
      break;
    }
    const attributeMatch = /^[A-Za-z_:][A-Za-z0-9_.:-]*/.exec(tag.slice(attributeCursor));
    if (attributeMatch === null) {
      return null;
    }
    const name = attributeMatch[0];
    attributeCursor += name.length;
    attributeCursor = skipXmlWhitespace(tag, attributeCursor);
    if (tag[attributeCursor] !== '=') {
      return null;
    }
    attributeCursor = skipXmlWhitespace(tag, attributeCursor + 1);
    const quote = tag[attributeCursor];
    if (quote !== '"' && quote !== "'") {
      return null;
    }
    const valueEnd = tag.indexOf(quote, attributeCursor + 1);
    if (valueEnd < 0) {
      return null;
    }
    const value = tag.slice(attributeCursor + 1, valueEnd);
    attributeCursor = valueEnd + 1;
    if (!['width', 'height', 'viewBox'].includes(name)) {
      continue;
    }
    if (seen.has(name)) {
      return null;
    }
    seen.add(name);
    if (name === 'width') {
      const parsed = parseSvgLength(value);
      if (parsed === null) {
        return null;
      }
      width = parsed;
    } else if (name === 'height') {
      const parsed = parseSvgLength(value);
      if (parsed === null) {
        return null;
      }
      height = parsed;
    } else {
      viewBox = parseSvgViewBox(value);
      if (viewBox === null) {
        return null;
      }
    }
  }

  const dimensions = resolveSvgDimensions(width, height, viewBox);
  return dimensions === null
    ? null
    : { format: 'svg', width: dimensions[0], height: dimensions[1] };
}

function skipXmlWhitespace(source: string, offset: number): number {
  while (offset < source.length && /[\t\n\r ]/.test(source[offset])) {
    offset += 1;
  }
  return offset;
}

function findXmlStartTagEnd(source: string, offset: number): number {
  let quote: '"' | "'" | null = null;
  for (let index = offset + 1; index < source.length; index += 1) {
    const character = source[index];
    if (quote !== null) {
      if (character === quote) {
        quote = null;
      }
    } else if (character === '"' || character === "'") {
      quote = character;
    } else if (character === '>') {
      return index;
    }
  }
  return -1;
}

function parseSvgLength(value: string): SvgIntrinsicLength | null {
  const normalized = value.trim();
  if (normalized.toLowerCase() === 'auto') {
    return { kind: 'relative' };
  }
  const match = /^([+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?)(px|pt|pc|in|cm|mm|q|%)?$/i
    .exec(normalized);
  if (match === null) {
    return null;
  }
  const number = Number(match[1]);
  if (!Number.isFinite(number) || number <= 0) {
    return null;
  }
  const unit = match[2]?.toLowerCase();
  if (unit === '%') {
    return { kind: 'relative' };
  }
  const scale = unit === 'pt'
    ? 96 / 72
    : unit === 'pc'
      ? 16
      : unit === 'in'
        ? 96
        : unit === 'cm'
          ? 96 / 2.54
          : unit === 'mm'
            ? 96 / 25.4
            : unit === 'q'
              ? 96 / 101.6
              : 1;
  return { kind: 'absolute', value: number * scale };
}

function parseSvgViewBox(value: string): readonly [number, number] | null {
  const values = value.trim().split(/[\s,]+/).map(Number);
  if (
    values.length !== 4
    || values.some((entry) => !Number.isFinite(entry))
    || values[2] <= 0
    || values[3] <= 0
  ) {
    return null;
  }
  return [values[2], values[3]];
}

function resolveSvgDimensions(
  width: SvgIntrinsicLength,
  height: SvgIntrinsicLength,
  viewBox: readonly [number, number] | null,
): readonly [number, number] | null {
  const absoluteWidth = width.kind === 'absolute' ? width.value : null;
  const absoluteHeight = height.kind === 'absolute' ? height.value : null;
  let resolvedWidth: number;
  let resolvedHeight: number;
  if (absoluteWidth !== null && absoluteHeight !== null) {
    [resolvedWidth, resolvedHeight] = [absoluteWidth, absoluteHeight];
  } else if (absoluteWidth !== null && viewBox !== null) {
    resolvedWidth = absoluteWidth;
    resolvedHeight = absoluteWidth * viewBox[1] / viewBox[0];
  } else if (absoluteHeight !== null && viewBox !== null) {
    resolvedWidth = absoluteHeight * viewBox[0] / viewBox[1];
    resolvedHeight = absoluteHeight;
  } else if (viewBox !== null) {
    [resolvedWidth, resolvedHeight] = viewBox;
  } else {
    resolvedWidth = absoluteWidth ?? 300;
    resolvedHeight = absoluteHeight ?? 150;
  }
  const finiteWidth = finiteSvgDimension(resolvedWidth);
  const finiteHeight = finiteSvgDimension(resolvedHeight);
  return finiteWidth === null || finiteHeight === null
    ? null
    : [finiteWidth, finiteHeight];
}

function finiteSvgDimension(value: number): number | null {
  return Number.isFinite(value) && value > 0 && value <= 0xffffffff ? Math.ceil(value) : null;
}
