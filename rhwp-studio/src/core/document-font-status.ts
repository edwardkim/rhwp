import { REGISTERED_FONTS } from './font-loader.ts';
import { resolveFont } from './font-substitution.ts';
import {
  getDetectedLocalFonts,
  getLocalFontDetectionMethod,
  getLocalFontState,
  type LocalFontDetectionSource,
} from './local-fonts.ts';

export type DocumentFontAvailability =
  | 'available'
  | 'needs-local-check'
  | 'web-substitute'
  | 'missing';

export type DocumentFontSource = 'local' | 'web' | 'generic' | 'unknown';

export interface DocumentFontStatusItem {
  fontName: string;
  status: DocumentFontAvailability;
  source: DocumentFontSource;
  substituteFont: string | null;
}

export interface DocumentFontStatusSummary {
  available: number;
  needsLocalCheck: number;
  webSubstitute: number;
  missing: number;
}

export interface DocumentFontStatusReport {
  fonts: DocumentFontStatusItem[];
  summary: DocumentFontStatusSummary;
  total: number;
  localSupported: boolean;
  localSnapshotLoaded: boolean;
  localSnapshotStored: boolean;
  localSnapshotComplete: boolean;
  localSnapshotSource: LocalFontDetectionSource | null;
  localCheckedFonts: string[];
  detectionMethod: LocalFontDetectionSource | null;
  shouldPromptLocalAccess: boolean;
}

export interface AnalyzeDocumentFontsOptions {
  localFonts?: string[];
  localSupported?: boolean;
  localSnapshotLoaded?: boolean;
  localSnapshotStored?: boolean;
  localSnapshotComplete?: boolean;
  localSnapshotSource?: LocalFontDetectionSource | null;
  localCheckedFonts?: string[];
  detectionMethod?: LocalFontDetectionSource | null;
}

const GENERIC_FONTS = new Set(['serif', 'sans-serif', 'monospace']);

function normalizeDocumentFonts(fonts: readonly string[] | undefined): string[] {
  const seen = new Set<string>();
  for (const font of fonts ?? []) {
    const name = font.trim();
    if (name) seen.add(name);
  }
  return Array.from(seen).sort((a, b) => a.localeCompare(b, 'ko'));
}

function resolveWebSubstitute(fontName: string): string | null {
  if (REGISTERED_FONTS.has(fontName)) return fontName;

  const resolved = resolveFont(fontName, 0, 0);
  if (resolved && resolved !== fontName && REGISTERED_FONTS.has(resolved)) {
    return resolved;
  }
  return null;
}

export function analyzeDocumentFonts(
  docFonts: readonly string[] | undefined,
  options: AnalyzeDocumentFontsOptions = {},
): DocumentFontStatusReport {
  const localState = getLocalFontState();
  const localFonts = options.localFonts ?? getDetectedLocalFonts();
  const localSet = new Set(localFonts);
  const localSupported = options.localSupported ?? localState.supported;
  const localSnapshotLoaded = options.localSnapshotLoaded ?? localState.loaded;
  const localSnapshotStored = options.localSnapshotStored ?? localState.stored;
  const localSnapshotComplete = options.localSnapshotComplete
    ?? (options.localSnapshotStored !== undefined ? options.localSnapshotStored : localState.complete);
  const localSnapshotSource = options.localSnapshotSource ?? localState.source;
  const localCheckedFonts = options.localCheckedFonts ?? localState.checkedFamilies;
  const localCheckedSet = new Set(localCheckedFonts);
  const detectionMethod = options.detectionMethod ?? getLocalFontDetectionMethod();

  const summary: DocumentFontStatusSummary = {
    available: 0,
    needsLocalCheck: 0,
    webSubstitute: 0,
    missing: 0,
  };

  const fonts = normalizeDocumentFonts(docFonts).map((fontName): DocumentFontStatusItem => {
    if (localSet.has(fontName)) {
      summary.available++;
      return { fontName, status: 'available', source: 'local', substituteFont: null };
    }

    if (GENERIC_FONTS.has(fontName) || REGISTERED_FONTS.has(fontName)) {
      summary.available++;
      return { fontName, status: 'available', source: GENERIC_FONTS.has(fontName) ? 'generic' : 'web', substituteFont: null };
    }

    const substituteFont = resolveWebSubstitute(fontName);

    const needsLocalCheck = localSupported
      && (!localSnapshotStored || (!localSnapshotComplete && !localCheckedSet.has(fontName)));

    if (needsLocalCheck) {
      summary.needsLocalCheck++;
      return {
        fontName,
        status: 'needs-local-check',
        source: 'unknown',
        substituteFont,
      };
    }

    if (substituteFont) {
      summary.webSubstitute++;
      return { fontName, status: 'web-substitute', source: 'web', substituteFont };
    }

    summary.missing++;
    return { fontName, status: 'missing', source: 'unknown', substituteFont: null };
  });

  const shouldPromptLocalAccess = localSupported && summary.needsLocalCheck > 0;

  return {
    fonts,
    summary,
    total: fonts.length,
    localSupported,
    localSnapshotLoaded,
    localSnapshotStored,
    localSnapshotComplete,
    localSnapshotSource,
    localCheckedFonts,
    detectionMethod,
    shouldPromptLocalAccess,
  };
}

export function shouldPromptLocalFontAccess(report: DocumentFontStatusReport): boolean {
  return report.shouldPromptLocalAccess;
}
