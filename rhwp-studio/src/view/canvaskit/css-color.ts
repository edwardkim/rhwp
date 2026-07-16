import type { CanvasKit } from 'canvaskit-wasm';

export const CSS_NAMED_COLORS: Record<string, string> = {
  aliceblue: '#f0f8ff',
  antiquewhite: '#faebd7',
  aqua: '#00ffff',
  aquamarine: '#7fffd4',
  azure: '#f0ffff',
  beige: '#f5f5dc',
  bisque: '#ffe4c4',
  black: '#000000',
  blanchedalmond: '#ffebcd',
  blue: '#0000ff',
  blueviolet: '#8a2be2',
  brown: '#a52a2a',
  burlywood: '#deb887',
  cadetblue: '#5f9ea0',
  chartreuse: '#7fff00',
  chocolate: '#d2691e',
  coral: '#ff7f50',
  cornflowerblue: '#6495ed',
  cornsilk: '#fff8dc',
  crimson: '#dc143c',
  cyan: '#00ffff',
  darkblue: '#00008b',
  darkcyan: '#008b8b',
  darkgoldenrod: '#b8860b',
  darkgray: '#a9a9a9',
  darkgreen: '#006400',
  darkgrey: '#a9a9a9',
  darkkhaki: '#bdb76b',
  darkmagenta: '#8b008b',
  darkolivegreen: '#556b2f',
  darkorange: '#ff8c00',
  darkorchid: '#9932cc',
  darkred: '#8b0000',
  darksalmon: '#e9967a',
  darkseagreen: '#8fbc8f',
  darkslateblue: '#483d8b',
  darkslategray: '#2f4f4f',
  darkslategrey: '#2f4f4f',
  darkturquoise: '#00ced1',
  darkviolet: '#9400d3',
  deeppink: '#ff1493',
  deepskyblue: '#00bfff',
  dimgray: '#696969',
  dimgrey: '#696969',
  dodgerblue: '#1e90ff',
  firebrick: '#b22222',
  floralwhite: '#fffaf0',
  forestgreen: '#228b22',
  fuchsia: '#ff00ff',
  gainsboro: '#dcdcdc',
  ghostwhite: '#f8f8ff',
  gold: '#ffd700',
  goldenrod: '#daa520',
  gray: '#808080',
  green: '#008000',
  greenyellow: '#adff2f',
  grey: '#808080',
  honeydew: '#f0fff0',
  hotpink: '#ff69b4',
  indianred: '#cd5c5c',
  indigo: '#4b0082',
  ivory: '#fffff0',
  khaki: '#f0e68c',
  lavender: '#e6e6fa',
  lavenderblush: '#fff0f5',
  lawngreen: '#7cfc00',
  lemonchiffon: '#fffacd',
  lightblue: '#add8e6',
  lightcoral: '#f08080',
  lightcyan: '#e0ffff',
  lightgoldenrodyellow: '#fafad2',
  lightgray: '#d3d3d3',
  lightgreen: '#90ee90',
  lightgrey: '#d3d3d3',
  lightpink: '#ffb6c1',
  lightsalmon: '#ffa07a',
  lightseagreen: '#20b2aa',
  lightskyblue: '#87cefa',
  lightslategray: '#778899',
  lightslategrey: '#778899',
  lightsteelblue: '#b0c4de',
  lightyellow: '#ffffe0',
  lime: '#00ff00',
  limegreen: '#32cd32',
  linen: '#faf0e6',
  magenta: '#ff00ff',
  maroon: '#800000',
  mediumaquamarine: '#66cdaa',
  mediumblue: '#0000cd',
  mediumorchid: '#ba55d3',
  mediumpurple: '#9370db',
  mediumseagreen: '#3cb371',
  mediumslateblue: '#7b68ee',
  mediumspringgreen: '#00fa9a',
  mediumturquoise: '#48d1cc',
  mediumvioletred: '#c71585',
  midnightblue: '#191970',
  mintcream: '#f5fffa',
  mistyrose: '#ffe4e1',
  moccasin: '#ffe4b5',
  navajowhite: '#ffdead',
  navy: '#000080',
  oldlace: '#fdf5e6',
  olive: '#808000',
  olivedrab: '#6b8e23',
  orange: '#ffa500',
  orangered: '#ff4500',
  orchid: '#da70d6',
  palegoldenrod: '#eee8aa',
  palegreen: '#98fb98',
  paleturquoise: '#afeeee',
  palevioletred: '#db7093',
  papayawhip: '#ffefd5',
  peachpuff: '#ffdab9',
  peru: '#cd853f',
  pink: '#ffc0cb',
  plum: '#dda0dd',
  powderblue: '#b0e0e6',
  purple: '#800080',
  rebeccapurple: '#663399',
  red: '#ff0000',
  rosybrown: '#bc8f8f',
  royalblue: '#4169e1',
  saddlebrown: '#8b4513',
  salmon: '#fa8072',
  sandybrown: '#f4a460',
  seagreen: '#2e8b57',
  seashell: '#fff5ee',
  sienna: '#a0522d',
  silver: '#c0c0c0',
  skyblue: '#87ceeb',
  slateblue: '#6a5acd',
  slategray: '#708090',
  slategrey: '#708090',
  snow: '#fffafa',
  springgreen: '#00ff7f',
  steelblue: '#4682b4',
  tan: '#d2b48c',
  teal: '#008080',
  thistle: '#d8bfd8',
  tomato: '#ff6347',
  transparent: '#00000000',
  turquoise: '#40e0d0',
  violet: '#ee82ee',
  wheat: '#f5deb3',
  white: '#ffffff',
  whitesmoke: '#f5f5f5',
  yellow: '#ffff00',
  yellowgreen: '#9acd32',
};

export function parseCanvasKitCssColor(canvasKit: CanvasKit, color: string, opacity = 1): Float32Array {
  const parsed = parseSupportedCssColor(color);
  let rgba: ArrayLike<number> = parsed ?? [0, 0, 0, 1];
  if (!parsed) {
    try {
      rgba = canvasKit.parseColorString(color) as ArrayLike<number> | undefined ?? rgba;
    } catch {
      rgba = [0, 0, 0, 1];
    }
  }
  return Float32Array.of(
    clampCanvasKitUnit(rgba[0] ?? 0),
    clampCanvasKitUnit(rgba[1] ?? 0),
    clampCanvasKitUnit(rgba[2] ?? 0),
    clampCanvasKitUnit((rgba[3] ?? 1) * opacity),
  );
}

export function parseSupportedCssColor(color: string): [number, number, number, number] | null {
  const normalized = color.trim().toLowerCase();
  const named = CSS_NAMED_COLORS[normalized];
  if (named) {
    return parseHexColor(named);
  }
  const hex = parseHexColor(normalized);
  if (hex) {
    return hex;
  }
  const rgbMatch = normalized.match(/^rgba?\((.*)\)$/);
  if (rgbMatch) {
    return parseRgbColorFunction(rgbMatch[1]);
  }
  const hslMatch = normalized.match(/^hsla?\((.*)\)$/);
  if (hslMatch) {
    return parseHslColorFunction(hslMatch[1]);
  }
  const hwbMatch = normalized.match(/^hwb\((.*)\)$/);
  if (hwbMatch) {
    const [colorBody, slashAlpha] = hwbMatch[1].split('/').map((part) => part.trim());
    const parts = colorBody.includes(',')
      ? colorBody.split(',').map((part) => part.trim()).filter((part) => part.length > 0)
      : colorBody.split(/\s+/).filter((part) => part.length > 0);
    if (parts.length < 3 || parts.length > 4) {
      return null;
    }
    const hue = parseCssHue(parts[0]);
    const whiteness = parseCssPercent(parts[1]);
    const blackness = parseCssPercent(parts[2]);
    const alpha = parseCssAlpha(slashAlpha ?? parts[3] ?? '1');
    if (hue === null || whiteness === null || blackness === null || alpha === null) {
      return null;
    }
    const sum = whiteness + blackness;
    if (sum >= 1) {
      const gray = whiteness / sum;
      return [gray, gray, gray, alpha];
    }
    const [baseRed, baseGreen, baseBlue] = hslToRgb(hue, 1, 0.5);
    const multiplier = 1 - whiteness - blackness;
    return [
      (baseRed * multiplier) + whiteness,
      (baseGreen * multiplier) + whiteness,
      (baseBlue * multiplier) + whiteness,
      alpha,
    ];
  }
  const colorMatch = normalized.match(/^color\((.*)\)$/);
  if (colorMatch) {
    const [colorBody, slashAlpha] = colorMatch[1].split('/').map((part) => part.trim());
    const parts = colorBody.split(/\s+/).filter((part) => part.length > 0);
    if (
      parts.length !== 4
      || ![
        'srgb',
        'srgb-linear',
        'display-p3',
        'a98-rgb',
        'prophoto-rgb',
        'rec2020',
        'xyz',
        'xyz-d50',
        'xyz-d65',
      ].includes(parts[0])
    ) {
      return null;
    }
    const channels = parts.slice(1).map((part) => {
      if (part.endsWith('%')) {
        return parseCssPercent(part);
      }
      const number = Number(part);
      return Number.isFinite(number) ? clampCanvasKitUnit(number) : null;
    });
    const alpha = parseCssAlpha(slashAlpha ?? '1');
    if (channels.some((channel) => channel === null) || alpha === null) {
      return null;
    }
    const red = channels[0] ?? 0;
    const green = channels[1] ?? 0;
    const blue = channels[2] ?? 0;
    if (parts[0] === 'srgb') {
      return [red, green, blue, alpha];
    }
    if (parts[0] === 'srgb-linear') {
      return [
        linearSrgbToEncodedUnit(red),
        linearSrgbToEncodedUnit(green),
        linearSrgbToEncodedUnit(blue),
        alpha,
      ];
    }
    if (parts[0] === 'xyz-d50') {
      return [...xyzD65ToEncodedSrgb(...d50ToD65Xyz(red, green, blue)), alpha];
    }
    if (parts[0] === 'xyz' || parts[0] === 'xyz-d65') {
      return [...xyzD65ToEncodedSrgb(red, green, blue), alpha];
    }
    if (parts[0] === 'a98-rgb') {
      const linearA98Red = red ** (563 / 256);
      const linearA98Green = green ** (563 / 256);
      const linearA98Blue = blue ** (563 / 256);
      const xD65 = (0.5766690429 * linearA98Red)
        + (0.1855582379 * linearA98Green)
        + (0.1882286462 * linearA98Blue);
      const yD65 = (0.2973449753 * linearA98Red)
        + (0.6273635663 * linearA98Green)
        + (0.0752914585 * linearA98Blue);
      const zD65 = (0.0270313614 * linearA98Red)
        + (0.0706888525 * linearA98Green)
        + (0.9913375368 * linearA98Blue);
      return [...xyzD65ToEncodedSrgb(xD65, yD65, zD65), alpha];
    }
    if (parts[0] === 'prophoto-rgb') {
      const linearProPhotoRed = red <= 0.03125 ? red / 16 : red ** 1.8;
      const linearProPhotoGreen = green <= 0.03125 ? green / 16 : green ** 1.8;
      const linearProPhotoBlue = blue <= 0.03125 ? blue / 16 : blue ** 1.8;
      const xD50 = (0.7977666449 * linearProPhotoRed)
        + (0.1351812974 * linearProPhotoGreen)
        + (0.0313477341 * linearProPhotoBlue);
      const yD50 = (0.2880748288 * linearProPhotoRed)
        + (0.7118352342 * linearProPhotoGreen)
        + (0.0000899369 * linearProPhotoBlue);
      const zD50 = 0.8251046025 * linearProPhotoBlue;
      return [...xyzD65ToEncodedSrgb(...d50ToD65Xyz(xD50, yD50, zD50)), alpha];
    }
    if (parts[0] === 'rec2020') {
      const alphaRec2020 = 1.0992968268;
      const betaRec2020 = 0.0180539685;
      const linearRec2020Red = red < betaRec2020 * 4.5
        ? red / 4.5
        : ((red + alphaRec2020 - 1) / alphaRec2020) ** (1 / 0.45);
      const linearRec2020Green = green < betaRec2020 * 4.5
        ? green / 4.5
        : ((green + alphaRec2020 - 1) / alphaRec2020) ** (1 / 0.45);
      const linearRec2020Blue = blue < betaRec2020 * 4.5
        ? blue / 4.5
        : ((blue + alphaRec2020 - 1) / alphaRec2020) ** (1 / 0.45);
      const xD65 = (0.6369580483 * linearRec2020Red)
        + (0.1446169036 * linearRec2020Green)
        + (0.1688809752 * linearRec2020Blue);
      const yD65 = (0.262700212 * linearRec2020Red)
        + (0.6779980715 * linearRec2020Green)
        + (0.0593017165 * linearRec2020Blue);
      const zD65 = (0.028072693 * linearRec2020Green)
        + (1.0609850577 * linearRec2020Blue);
      return [...xyzD65ToEncodedSrgb(xD65, yD65, zD65), alpha];
    }
    const linearP3Red = red <= 0.04045 ? red / 12.92 : ((red + 0.055) / 1.055) ** 2.4;
    const linearP3Green = green <= 0.04045 ? green / 12.92 : ((green + 0.055) / 1.055) ** 2.4;
    const linearP3Blue = blue <= 0.04045 ? blue / 12.92 : ((blue + 0.055) / 1.055) ** 2.4;
    const xD65 = (0.4865709486 * linearP3Red) + (0.2656676932 * linearP3Green) + (0.1982172852 * linearP3Blue);
    const yD65 = (0.2289745641 * linearP3Red) + (0.6917385218 * linearP3Green) + (0.0792869141 * linearP3Blue);
    const zD65 = (0.0451133819 * linearP3Green) + (1.0439443689 * linearP3Blue);
    return [...xyzD65ToEncodedSrgb(xD65, yD65, zD65), alpha];
  }
  const labMatch = normalized.match(/^lab\((.*)\)$/);
  if (labMatch) {
    const [colorBody, slashAlpha] = labMatch[1].split('/').map((part) => part.trim());
    const parts = colorBody.split(/\s+/).filter((part) => part.length > 0);
    if (parts.length !== 3) {
      return null;
    }
    const lightness = parseCssLabLightness(parts[0]);
    const axisA = Number(parts[1]);
    const axisB = Number(parts[2]);
    const alpha = parseCssAlpha(slashAlpha ?? '1');
    if (lightness === null || !Number.isFinite(axisA) || !Number.isFinite(axisB) || alpha === null) {
      return null;
    }
    const [red, green, blue] = labToRgb(lightness, axisA, axisB);
    return [red, green, blue, alpha];
  }
  const lchMatch = normalized.match(/^lch\((.*)\)$/);
  if (lchMatch) {
    const [colorBody, slashAlpha] = lchMatch[1].split('/').map((part) => part.trim());
    const parts = colorBody.split(/\s+/).filter((part) => part.length > 0);
    if (parts.length !== 3) {
      return null;
    }
    const lightness = parseCssLabLightness(parts[0]);
    const chroma = Number(parts[1]);
    const hue = parseCssHue(parts[2]);
    const alpha = parseCssAlpha(slashAlpha ?? '1');
    if (lightness === null || !Number.isFinite(chroma) || hue === null || alpha === null) {
      return null;
    }
    const hueRadians = hue * (Math.PI / 180);
    const [red, green, blue] = labToRgb(
      lightness,
      chroma * Math.cos(hueRadians),
      chroma * Math.sin(hueRadians),
    );
    return [red, green, blue, alpha];
  }
  const oklabMatch = normalized.match(/^oklab\((.*)\)$/);
  if (oklabMatch) {
    const [colorBody, slashAlpha] = oklabMatch[1].split('/').map((part) => part.trim());
    const parts = colorBody.split(/\s+/).filter((part) => part.length > 0);
    if (parts.length !== 3) {
      return null;
    }
    const lightness = parts[0].endsWith('%') ? parseCssPercent(parts[0]) : Number(parts[0]);
    const axisA = Number(parts[1]);
    const axisB = Number(parts[2]);
    const alpha = parseCssAlpha(slashAlpha ?? '1');
    if (
      lightness === null
      || !Number.isFinite(lightness)
      || !Number.isFinite(axisA)
      || !Number.isFinite(axisB)
      || alpha === null
    ) {
      return null;
    }
    const [red, green, blue] = oklabToRgb(lightness, axisA, axisB);
    return [red, green, blue, alpha];
  }
  const oklchMatch = normalized.match(/^oklch\((.*)\)$/);
  if (oklchMatch) {
    const [colorBody, slashAlpha] = oklchMatch[1].split('/').map((part) => part.trim());
    const parts = colorBody.split(/\s+/).filter((part) => part.length > 0);
    if (parts.length !== 3) {
      return null;
    }
    const lightness = parts[0].endsWith('%') ? parseCssPercent(parts[0]) : Number(parts[0]);
    const chroma = Number(parts[1]);
    const hue = parseCssHue(parts[2]);
    const alpha = parseCssAlpha(slashAlpha ?? '1');
    if (
      lightness === null
      || !Number.isFinite(lightness)
      || !Number.isFinite(chroma)
      || hue === null
      || alpha === null
    ) {
      return null;
    }
    const hueRadians = hue * (Math.PI / 180);
    const [red, green, blue] = oklabToRgb(
      lightness,
      chroma * Math.cos(hueRadians),
      chroma * Math.sin(hueRadians),
    );
    return [red, green, blue, alpha];
  }
  return null;
}

function parseHexColor(color: string): [number, number, number, number] | null {
  const match = color.match(/^#([0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})$/);
  if (!match) {
    return null;
  }
  const hex = match[1];
  const components = hex.length <= 4
    ? [...hex].map((component) => Number.parseInt(`${component}${component}`, 16))
    : hex.match(/../g)?.map((component) => Number.parseInt(component, 16));
  if (!components || components.some((component) => !Number.isFinite(component))) {
    return null;
  }
  return [
    components[0] / 255,
    components[1] / 255,
    components[2] / 255,
    (components[3] ?? 255) / 255,
  ];
}

function parseRgbColorFunction(body: string): [number, number, number, number] | null {
  const [colorBody, slashAlpha] = body.split('/').map((part) => part.trim());
  const parts = colorBody.includes(',')
    ? colorBody.split(',').map((part) => part.trim()).filter((part) => part.length > 0)
    : colorBody.split(/\s+/).filter((part) => part.length > 0);
  if (parts.length < 3 || parts.length > 4) {
    return null;
  }
  const red = parseRgbChannel(parts[0]);
  const green = parseRgbChannel(parts[1]);
  const blue = parseRgbChannel(parts[2]);
  const alpha = parseCssAlpha(slashAlpha ?? parts[3] ?? '1');
  if (red === null || green === null || blue === null || alpha === null) {
    return null;
  }
  return [red, green, blue, alpha];
}

function parseHslColorFunction(body: string): [number, number, number, number] | null {
  const [colorBody, slashAlpha] = body.split('/').map((part) => part.trim());
  const parts = colorBody.includes(',')
    ? colorBody.split(',').map((part) => part.trim()).filter((part) => part.length > 0)
    : colorBody.split(/\s+/).filter((part) => part.length > 0);
  if (parts.length < 3 || parts.length > 4) {
    return null;
  }
  const hue = parseCssHue(parts[0]);
  const saturation = parseCssPercent(parts[1]);
  const lightness = parseCssPercent(parts[2]);
  const alpha = parseCssAlpha(slashAlpha ?? parts[3] ?? '1');
  if (hue === null || saturation === null || lightness === null || alpha === null) {
    return null;
  }
  const [red, green, blue] = hslToRgb(hue, saturation, lightness);
  return [red, green, blue, alpha];
}

function parseCssHue(value: string): number | null {
  const trimmed = value.trim();
  const match = trimmed.match(/^([-+]?(?:\d+|\d*\.\d+))(deg|grad|rad|turn)?$/);
  if (!match) {
    return null;
  }
  const amount = Number(match[1]);
  if (!Number.isFinite(amount)) {
    return null;
  }
  switch (match[2]) {
    case undefined:
    case 'deg':
      return amount;
    case 'grad':
      return amount * 0.9;
    case 'rad':
      return amount * (180 / Math.PI);
    case 'turn':
      return amount * 360;
    default:
      return null;
  }
}

function parseRgbChannel(value: string): number | null {
  const trimmed = value.trim();
  if (trimmed.endsWith('%')) {
    return parseCssPercent(trimmed);
  }
  const number = Number(trimmed);
  return Number.isFinite(number) ? clampCanvasKitUnit(number / 255) : null;
}

function parseCssAlpha(value: string): number | null {
  const trimmed = value.trim();
  if (trimmed.endsWith('%')) {
    return parseCssPercent(trimmed);
  }
  const number = Number(trimmed);
  return Number.isFinite(number) ? clampCanvasKitUnit(number) : null;
}

function parseCssPercent(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed.endsWith('%')) {
    return null;
  }
  const number = Number(trimmed.slice(0, -1).trim());
  return Number.isFinite(number) ? clampCanvasKitUnit(number / 100) : null;
}

function parseCssLabLightness(value: string): number | null {
  const trimmed = value.trim();
  if (trimmed.endsWith('%')) {
    const percent = parseCssPercent(trimmed);
    return percent === null ? null : percent * 100;
  }
  const number = Number(trimmed);
  return Number.isFinite(number) ? number : null;
}

function hslToRgb(hueDegrees: number, saturation: number, lightness: number): [number, number, number] {
  const hue = ((hueDegrees % 360) + 360) % 360;
  const chroma = (1 - Math.abs((2 * lightness) - 1)) * saturation;
  const second = chroma * (1 - Math.abs(((hue / 60) % 2) - 1));
  const match = lightness - (chroma / 2);
  let red = 0;
  let green = 0;
  let blue = 0;
  if (hue < 60) {
    red = chroma;
    green = second;
  } else if (hue < 120) {
    red = second;
    green = chroma;
  } else if (hue < 180) {
    green = chroma;
    blue = second;
  } else if (hue < 240) {
    green = second;
    blue = chroma;
  } else if (hue < 300) {
    red = second;
    blue = chroma;
  } else {
    red = chroma;
    blue = second;
  }
  return [red + match, green + match, blue + match];
}

function labToRgb(lightness: number, axisA: number, axisB: number): [number, number, number] {
  const normalizedY = (lightness + 16) / 116;
  const normalizedX = normalizedY + (axisA / 500);
  const normalizedZ = normalizedY - (axisB / 200);
  const epsilon = 216 / 24389;
  const kappa = 24389 / 27;
  const xD50 = 0.96422 * (
    (normalizedX ** 3) > epsilon
      ? normalizedX ** 3
      : ((116 * normalizedX) - 16) / kappa
  );
  const yD50 = 1 * (
    (normalizedY ** 3) > epsilon
      ? normalizedY ** 3
      : ((116 * normalizedY) - 16) / kappa
  );
  const zD50 = 0.82521 * (
    (normalizedZ ** 3) > epsilon
      ? normalizedZ ** 3
      : ((116 * normalizedZ) - 16) / kappa
  );
  const [xD65, yD65, zD65] = d50ToD65Xyz(xD50, yD50, zD50);
  return xyzD65ToEncodedSrgb(xD65, yD65, zD65);
}

function d50ToD65Xyz(xD50: number, yD50: number, zD50: number): [number, number, number] {
  return [
    (0.9555766 * xD50) - (0.0230393 * yD50) + (0.0631636 * zD50),
    (-0.0282895 * xD50) + (1.0099416 * yD50) + (0.0210077 * zD50),
    (0.0122982 * xD50) - (0.020483 * yD50) + (1.3299098 * zD50),
  ];
}

function xyzD65ToEncodedSrgb(xD65: number, yD65: number, zD65: number): [number, number, number] {
  const linearRed = (3.2404542 * xD65) - (1.5371385 * yD65) - (0.4985314 * zD65);
  const linearGreen = (-0.969266 * xD65) + (1.8760108 * yD65) + (0.041556 * zD65);
  const linearBlue = (0.0556434 * xD65) - (0.2040259 * yD65) + (1.0572252 * zD65);
  return [linearRed, linearGreen, linearBlue].map(linearSrgbToEncodedUnit) as [number, number, number];
}

function oklabToRgb(lightness: number, axisA: number, axisB: number): [number, number, number] {
  const long = lightness + (0.3963377774 * axisA) + (0.2158037573 * axisB);
  const medium = lightness - (0.1055613458 * axisA) - (0.0638541728 * axisB);
  const short = lightness - (0.0894841775 * axisA) - (1.291485548 * axisB);
  const longCubed = long * long * long;
  const mediumCubed = medium * medium * medium;
  const shortCubed = short * short * short;
  const linearRed = (4.0767416621 * longCubed) - (3.3077115913 * mediumCubed) + (0.2309699292 * shortCubed);
  const linearGreen = (-1.2684380046 * longCubed) + (2.6097574011 * mediumCubed) - (0.3413193965 * shortCubed);
  const linearBlue = (-0.0041960863 * longCubed) - (0.7034186147 * mediumCubed) + (1.707614701 * shortCubed);
  return [linearRed, linearGreen, linearBlue].map(linearSrgbToEncodedUnit) as [number, number, number];
}

function linearSrgbToEncodedUnit(channel: number): number {
  const encoded = channel <= 0.0031308
    ? 12.92 * channel
    : (1.055 * (channel ** (1 / 2.4))) - 0.055;
  return clampCanvasKitUnit(encoded);
}

export function clampCanvasKitUnit(value: number): number {
  return Number.isFinite(value) ? Math.max(0, Math.min(1, value)) : 0;
}
