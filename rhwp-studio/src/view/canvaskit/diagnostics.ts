const EXPECTED_CANVASKIT_UNSUPPORTED_OPS = new Set([
  'equation:unsupportedDirectReplay',
  'glyphOutline',
  'glyphOutline:glyphOutlineStrokeStyleUnsupported',
  'glyphOutline:unsupportedBitmapGlyph',
  'glyphOutline:unsupportedColorGlyph',
  'glyphOutline:unsupportedOutlinePayload',
  'glyphOutline:unsupportedSvgGlyph',
  'image:dataMissing',
  'image:dimensionUnavailable',
  'image:invalidBounds',
  'image:tileLimit',
  'imageEffect:blackWhite',
  'imageEffect:brightnessContrast',
  'imageEffect:grayScale',
  'imageEffect:pattern8x8',
  'rawSvg:unsupportedDirectReplay',
  'textRun:emphasisDot',
  'textRun:layoutPositions',
  'textRun:scriptTextRequiresShaping',
  'textRunFont',
]);

export function isExpectedCanvasKitUnsupportedOp(op: string): boolean {
  return EXPECTED_CANVASKIT_UNSUPPORTED_OPS.has(op);
}
