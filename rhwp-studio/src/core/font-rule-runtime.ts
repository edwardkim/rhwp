import {
  FONT_RULE_CANVAS2D_PAINT_RULES,
  type GeneratedFontRuleProjection as Canvas2dPaintRule,
} from './generated/font-rule-projections/canvas2d-paint.ts';
import {
  FONT_RULE_CANVAS2D_WEBFONT_RULES,
} from './generated/font-rule-projections/webfont-supply.ts';
import {
  FONT_RULE_CANVASKIT_SFNT_RULES,
  type GeneratedFontRuleProjection as CanvasKitRule,
} from './generated/font-rule-projections/canvaskit-sfnt.ts';

export type ProjectedFontFormat = 'woff2' | 'woff' | 'truetype' | 'opentype';

export type ProjectedSubstitutionEntry = readonly [
  sourceFace: string,
  sourceAltType: number,
  targetFace: string,
  targetAltType: number,
  ruleId: string,
];

export interface ProjectedWebFontEntry {
  name: string;
  file: string;
  format: ProjectedFontFormat;
  unicodeRange?: string;
  ruleId: string;
}

export interface ProjectedCanvasKitSource {
  url: string;
  aliases: string[];
}

export interface ProjectedCanvasKitPlanOptions {
  disableExternalWebFonts?: boolean;
  localFontBaseUrl?: string;
  availableLocalFiles?: ReadonlySet<string>;
}

export interface ProjectedCanvasKitPlan {
  sources: ProjectedCanvasKitSource[];
  unavailableFonts: string[];
  ruleIds: string[];
}

const SUBSTITUTION_BOUNDARY = 'studio-substitution.substitution-tables';
const GOVERNMENT_SUCCESSOR_BOUNDARY = 'rust-paint-chain.installed-aliases';
const DISPLAY_CHAIN_BOUNDARY = 'studio-substitution.display-chain';
const WEBFONT_BOUNDARY = 'studio-supply.font-list';
const CANVASKIT_PLAN_BOUNDARY = 'studio-supply.canvaskit-plan';

function hasBoundary(
  rule: { sourceBoundaryId: string },
  boundary: string,
): boolean {
  return rule.sourceBoundaryId === boundary;
}

function requiredString(
  value: Readonly<Record<string, unknown>>,
  key: string,
  ruleId: string,
): string {
  const result = value[key];
  if (typeof result !== 'string' || result.length === 0) {
    throw new Error(`${ruleId}: generated font supply ${key} must be a non-empty string`);
  }
  return result;
}

function nullableString(
  value: Readonly<Record<string, unknown>>,
  key: string,
  ruleId: string,
): string | null {
  const result = value[key];
  if (result === null || result === undefined) return null;
  if (typeof result !== 'string') {
    throw new Error(`${ruleId}: generated font supply ${key} must be a string or null`);
  }
  return result;
}

function supplyObject(
  rule: { ruleId: string; supply: Readonly<Record<string, unknown>> | null },
): Readonly<Record<string, unknown>> {
  if (rule.supply === null) {
    throw new Error(`${rule.ruleId}: generated font supply payload is missing`);
  }
  return rule.supply;
}

function parseSubstitutionAltTypes(rule: Canvas2dPaintRule): readonly [number, number] {
  const match = /^source:(\d+)->target:(\d+)$/.exec(rule.conditions.altType ?? '');
  if (!match) throw new Error(`${rule.ruleId}: generated substitution altType is invalid`);
  return [Number.parseInt(match[1], 10), Number.parseInt(match[2], 10)];
}

export const FONT_RULE_SUBSTITUTION_TABLES: readonly (readonly ProjectedSubstitutionEntry[])[] =
  Object.freeze(Array.from({ length: 7 }, (_, languageSlot) => Object.freeze(
    FONT_RULE_CANVAS2D_PAINT_RULES
      .filter(rule => (
        hasBoundary(rule, SUBSTITUTION_BOUNDARY)
        && rule.conditions.languageSlot === String(languageSlot)
      ))
      .sort((left, right) => (left.order ?? 0) - (right.order ?? 0))
      .map(rule => {
        if (rule.sourceFace === null) {
          throw new Error(`${rule.ruleId}: generated substitution sourceFace is missing`);
        }
        const [sourceAltType, targetAltType] = parseSubstitutionAltTypes(rule);
        return Object.freeze([
          rule.sourceFace,
          sourceAltType,
          rule.targetFaceOrPolicy,
          targetAltType,
          rule.ruleId,
        ]) as ProjectedSubstitutionEntry;
      }),
  )));

export interface ProjectedSubstituteTarget {
  /** 치환 대상 face 이름. 설치 face 이름인 경우가 많다(`한양중고딕` → `HY중고딕`). */
  face: string;
  altType: number;
  ruleId: string;
}

function substitutionKey(face: string, altType: number): string {
  return `${face}\u0000${altType}`;
}

const substitutionIndexes: (Map<string, ProjectedSubstituteTarget> | null)[] =
  Array.from({ length: 7 }, () => null);

function substitutionIndex(languageSlot: number): Map<string, ProjectedSubstituteTarget> {
  const cached = substitutionIndexes[languageSlot];
  if (cached) return cached;
  const index = new Map<string, ProjectedSubstituteTarget>();
  for (const [sourceFace, sourceAltType, targetFace, targetAltType, ruleId]
    of FONT_RULE_SUBSTITUTION_TABLES[languageSlot]) {
    const key = substitutionKey(sourceFace, sourceAltType);
    if (!index.has(key)) index.set(key, { face: targetFace, altType: targetAltType, ruleId });
  }
  substitutionIndexes[languageSlot] = index;
  return index;
}

/**
 * 등록 웹폰트 여부와 무관하게 paint 치환 대상 체인을 그대로 돌려준다.
 *
 * `resolveFont`는 요청 이름이 이미 등록 웹폰트면 체인을 타지 않는다. 그러나 legacy 이름의
 * 웹폰트 공급이 번들 stand-in(예: `한양중고딕` → `NotoSansKR-Regular.woff2`)뿐이면 호스트에
 * 설치된 실제 face(`HY중고딕`)를 찾기 위해 치환 대상 자체가 필요하다.
 */
export function projectedSubstituteTargets(
  fontName: string,
  altType = 0,
  langId = 0,
): ProjectedSubstituteTarget[] {
  if (!fontName) return [];
  const index = substitutionIndex(langId >= 0 && langId <= 6 ? langId : 0);
  let name = fontName;
  let type = altType || 0;
  if (type === 0) {
    if (index.has(substitutionKey(name, 1))) type = 1;
    else if (index.has(substitutionKey(name, 2))) type = 2;
    else return [];
  }

  const targets: ProjectedSubstituteTarget[] = [];
  const visited = new Set<string>();
  for (let step = 0; step < 15; step += 1) {
    const key = substitutionKey(name, type);
    if (visited.has(key)) break;
    visited.add(key);
    const target = index.get(key);
    if (!target) break;
    targets.push(target);
    name = target.face;
    type = target.altType;
  }
  return targets;
}

export interface ProjectedGovernmentSuccessor {
  sourceFace: string;
  targetFace: string;
  order: number;
  ruleId: string;
}

export const FONT_RULE_GOVERNMENT_SUCCESSORS: readonly ProjectedGovernmentSuccessor[] =
  Object.freeze(FONT_RULE_CANVAS2D_PAINT_RULES
    .filter(rule => hasBoundary(rule, GOVERNMENT_SUCCESSOR_BOUNDARY))
    .map(rule => {
      if (rule.sourceFace === null || rule.order === null) {
        throw new Error(`${rule.ruleId}: generated government successor is incomplete`);
      }
      return Object.freeze({
        sourceFace: rule.sourceFace,
        targetFace: rule.targetFaceOrPolicy,
        order: rule.order,
        ruleId: rule.ruleId,
      });
    }));

export const FONT_RULE_DISPLAY_CHAIN_POLICY_IDS: Readonly<Record<string, string>> =
  Object.freeze(Object.fromEntries(
    FONT_RULE_CANVAS2D_PAINT_RULES
      .filter(rule => hasBoundary(rule, DISPLAY_CHAIN_BOUNDARY))
      .map(rule => [rule.relationType, rule.ruleId]),
  ));

export const FONT_RULE_WEBFONT_ENTRIES: readonly ProjectedWebFontEntry[] = Object.freeze(
  FONT_RULE_CANVAS2D_WEBFONT_RULES.map(rule => {
    if (!hasBoundary(rule, WEBFONT_BOUNDARY)) {
      throw new Error(`${rule.ruleId}: unexpected webfont source boundary`);
    }
    const supply = supplyObject(rule);
    const format = requiredString(supply, 'format', rule.ruleId);
    if (!['woff2', 'woff', 'truetype', 'opentype'].includes(format)) {
      throw new Error(`${rule.ruleId}: generated webfont format is invalid`);
    }
    const unicodeRange = nullableString(supply, 'unicodeRange', rule.ruleId);
    return Object.freeze({
      name: requiredString(supply, 'fontFamily', rule.ruleId),
      file: requiredString(supply, 'sourceUrl', rule.ruleId),
      format: format as ProjectedFontFormat,
      ...(unicodeRange === null ? {} : { unicodeRange }),
      ruleId: rule.ruleId,
    });
  }),
);

interface CanvasKitSnapshot {
  sources: ProjectedCanvasKitSource[];
  unavailableFonts: string[];
}

interface CanvasKitSupply {
  sourceFace: string;
  declaredCapability: string;
  online: CanvasKitSnapshot;
  offline: CanvasKitSnapshot;
  ruleId: string;
}

function stringArray(value: unknown, location: string): string[] {
  if (!Array.isArray(value) || value.some(entry => typeof entry !== 'string')) {
    throw new Error(`${location} must be a string array`);
  }
  return [...value];
}

function canvasKitSnapshot(value: unknown, location: string): CanvasKitSnapshot {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${location} must be an object`);
  }
  const record = value as Readonly<Record<string, unknown>>;
  if (!Array.isArray(record.sources)) throw new Error(`${location}.sources must be an array`);
  const sources = record.sources.map((source, index) => {
    if (source === null || typeof source !== 'object' || Array.isArray(source)) {
      throw new Error(`${location}.sources[${index}] must be an object`);
    }
    const sourceRecord = source as Readonly<Record<string, unknown>>;
    return {
      url: requiredString(sourceRecord, 'url', location),
      aliases: stringArray(sourceRecord.aliases, `${location}.sources[${index}].aliases`),
    };
  });
  return {
    sources,
    unavailableFonts: stringArray(record.unavailableFonts, `${location}.unavailableFonts`),
  };
}

const canvasKitSupplyRules = FONT_RULE_CANVASKIT_SFNT_RULES.filter(rule => (
  hasBoundary(rule, WEBFONT_BOUNDARY)
));

const canvasKitSupplies = new Map<string, CanvasKitSupply>(canvasKitSupplyRules.map(rule => {
  if (rule.sourceFace === null) {
    throw new Error(`${rule.ruleId}: generated CanvasKit font family is missing`);
  }
  const supply = supplyObject(rule);
  return [normalizeProjectedFontFamily(rule.sourceFace), {
    sourceFace: rule.sourceFace,
    declaredCapability: requiredString(supply, 'declaredCapability', rule.ruleId),
    online: canvasKitSnapshot(supply.online, `${rule.ruleId}.online`),
    offline: canvasKitSnapshot(supply.offline, `${rule.ruleId}.offline`),
    ruleId: rule.ruleId,
  }];
}));

const canvasKitSubstitutes = new Map<string, { target: string; ruleId: string }>(
  FONT_RULE_CANVASKIT_SFNT_RULES
    .filter(rule => hasBoundary(rule, CANVASKIT_PLAN_BOUNDARY) && rule.sourceFace !== null)
    .map(rule => [normalizeProjectedFontFamily(rule.sourceFace ?? ''), {
      target: normalizeProjectedFontFamily(rule.targetFaceOrPolicy),
      ruleId: rule.ruleId,
    }]),
);

const canvasKitPlanPolicyRule = FONT_RULE_CANVASKIT_SFNT_RULES.find(rule => (
  hasBoundary(rule, CANVASKIT_PLAN_BOUNDARY) && rule.sourceFace === null
));
if (!canvasKitPlanPolicyRule) throw new Error('generated CanvasKit plan policy is missing');
const canvasKitPlanPolicyRuleId = canvasKitPlanPolicyRule.ruleId;

export function normalizeProjectedFontFamily(value: string): string {
  return value
    .replace(/\u0000/g, '')
    .normalize('NFC')
    .replace(/\s+/g, ' ')
    .trim()
    .toLocaleLowerCase('en-US');
}

function isExternalFontFile(file: string): boolean {
  return /^https?:\/\//i.test(file);
}

function projectedCanvasKitFontUrl(file: string, localFontBaseUrl?: string): string {
  if (isExternalFontFile(file) || !localFontBaseUrl) return file;
  const base = localFontBaseUrl.replace(/\/+$/, '');
  return `${base}/${file.replace(/^fonts\//, '')}`;
}

export function isCanvasKitSfntPlanned(fontName: string): boolean {
  const supply = canvasKitSupplies.get(normalizeProjectedFontFamily(fontName));
  return supply?.online.sources.some(source => (
    /\.(?:ttf|otf|ttc)(?:$|[?#])/i.test(source.url)
  )) ?? false;
}

export function resolveProjectedCanvasKitFontPlan(
  requiredFontFamilies: readonly string[],
  options: ProjectedCanvasKitPlanOptions = {},
): ProjectedCanvasKitPlan {
  const sourcesByUrl = new Map<string, Set<string>>();
  const unavailableFonts = new Map<string, string>();
  const ruleIds: string[] = [canvasKitPlanPolicyRuleId];

  for (const requestedValue of requiredFontFamilies) {
    const requested = requestedValue.trim();
    const normalized = normalizeProjectedFontFamily(requestedValue);
    if (!normalized) continue;
    const substitute = canvasKitSubstitutes.get(normalized);
    const supply = canvasKitSupplies.get(normalized)
      ?? canvasKitSupplies.get(substitute?.target ?? '');
    if (substitute && supply) ruleIds.push(substitute.ruleId);
    if (!supply) {
      unavailableFonts.set(normalized, requested);
      continue;
    }
    ruleIds.push(supply.ruleId);

    const snapshot = options.disableExternalWebFonts === true ? supply.offline : supply.online;
    if (snapshot.sources.length === 0 || snapshot.unavailableFonts.length > 0) {
      unavailableFonts.set(normalized, requested);
      continue;
    }

    let sourceUnavailable = false;
    for (const source of snapshot.sources) {
      const localFile = source.url.startsWith('fonts/')
        ? source.url.slice('fonts/'.length)
        : null;
      if (localFile !== null
          && options.availableLocalFiles !== undefined
          && !options.availableLocalFiles.has(localFile)) {
        sourceUnavailable = true;
        break;
      }
    }
    if (sourceUnavailable) {
      unavailableFonts.set(normalized, requested);
      continue;
    }

    for (const source of snapshot.sources) {
      const url = projectedCanvasKitFontUrl(source.url, options.localFontBaseUrl);
      const aliases = sourcesByUrl.get(url) ?? new Set<string>();
      aliases.add(requested);
      for (const alias of source.aliases) aliases.add(alias);
      sourcesByUrl.set(url, aliases);
    }
  }

  return {
    sources: [...sourcesByUrl.entries()].map(([url, aliases]) => ({
      url,
      aliases: [...aliases].sort((left, right) => left.localeCompare(right, 'ko')),
    })),
    unavailableFonts: [...unavailableFonts.values()]
      .sort((left, right) => left.localeCompare(right, 'ko')),
    ruleIds: [...new Set(ruleIds)],
  };
}

export function getProjectedWebFontRuleIds(fontName: string): string[] {
  const normalized = normalizeProjectedFontFamily(fontName);
  return FONT_RULE_WEBFONT_ENTRIES
    .filter(entry => normalizeProjectedFontFamily(entry.name) === normalized)
    .map(entry => entry.ruleId);
}

export function getProjectedCanvasKitRuleIds(fontName: string): string[] {
  return resolveProjectedCanvasKitFontPlan([fontName]).ruleIds;
}

export function generatedCanvasKitRuleCount(): number {
  return FONT_RULE_CANVASKIT_SFNT_RULES.length;
}

export function generatedCanvas2dPaintRuleCount(): number {
  return FONT_RULE_CANVAS2D_PAINT_RULES.length;
}

export type { CanvasKitRule };
