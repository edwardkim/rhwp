export type GridPattern = 'dots' | 'horizontal' | 'vertical' | 'both';
export type GridLayer = 'behindText' | 'inFrontOfText';
export type GridSnapMode = 'free' | 'magnetic' | 'gridOnly';
export type GridOrigin = 'page' | 'paper';

export interface GridOffsetMm {
  x: number;
  y: number;
}

export interface GridViewSettings {
  visible: boolean;
  pattern: GridPattern;
  layer: GridLayer;
  snapMode: GridSnapMode;
  horizontalMm: number;
  verticalMm: number;
  origin: GridOrigin;
  offsetXmm: number;
  offsetYmm: number;
}

const DEFAULT_SETTINGS: GridViewSettings = {
  visible: false,
  pattern: 'dots',
  layer: 'behindText',
  snapMode: 'free',
  horizontalMm: 3,
  verticalMm: 3,
  origin: 'page',
  offsetXmm: 0,
  offsetYmm: 0,
};

let currentSettings: GridViewSettings = { ...DEFAULT_SETTINGS };

export function getGridViewSettings(): GridViewSettings {
  return { ...currentSettings };
}

export function setGridViewSettings(next: Partial<GridViewSettings>): GridViewSettings {
  currentSettings = normalizeGridSettings({ ...currentSettings, ...next });
  return getGridViewSettings();
}

export function toggleGridVisibility(): GridViewSettings {
  return setGridViewSettings({ visible: !currentSettings.visible });
}

export function normalizeGridSettings(settings: GridViewSettings): GridViewSettings {
  return {
    visible: settings.visible,
    pattern: settings.pattern,
    layer: settings.layer,
    snapMode: settings.snapMode,
    horizontalMm: clampGridMm(settings.horizontalMm),
    verticalMm: clampGridMm(settings.verticalMm),
    origin: settings.origin,
    offsetXmm: clampOffsetMm(settings.offsetXmm),
    offsetYmm: clampOffsetMm(settings.offsetYmm),
  };
}

export function convertGridOffsetForOrigin(
  offset: GridOffsetMm,
  from: GridOrigin,
  to: GridOrigin,
  originBases: Record<GridOrigin, GridOffsetMm>,
): GridOffsetMm {
  const currentBase = originBases[from];
  const nextBase = originBases[to];
  const absoluteX = finiteOrZero(offset.x) + currentBase.x;
  const absoluteY = finiteOrZero(offset.y) + currentBase.y;
  return {
    x: roundMm(absoluteX - nextBase.x),
    y: roundMm(absoluteY - nextBase.y),
  };
}

function clampGridMm(value: number): number {
  if (!Number.isFinite(value)) return 3;
  return Math.min(50, Math.max(0.5, value));
}

function clampOffsetMm(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.min(500, Math.max(-500, value));
}

function finiteOrZero(value: number): number {
  return Number.isFinite(value) ? value : 0;
}

function roundMm(value: number): number {
  return Math.round(value * 100) / 100;
}
