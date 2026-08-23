#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from './font_rule_ledger.mjs';
import {
  REGISTERED_FONTS,
  getWebFontSupplySnapshot,
  loadWebFonts,
  resolveCanvasKitFontPlan,
} from '../rhwp-studio/src/core/font-loader.ts';
import {
  fontFamilyCandidatesForDisplay,
  resolveFont,
} from '../rhwp-studio/src/core/font-substitution.ts';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const CANDIDATES_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4939',
  'font_rule_candidates.json',
);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function rowsAndHash(rows) {
  return {
    count: rows.length,
    sha256: sha256Text(canonicalJson(rows)),
    rows,
  };
}

function substitutionRows(candidates) {
  return candidates
    .filter(candidate => (
      candidate.sourceBoundaryId === 'studio-substitution.substitution-tables'
    ))
    .map(candidate => {
      const languageSlot = Number.parseInt(candidate.conditions.languageSlot, 10);
      const altType = candidate.conditions.sourceAltType;
      return {
        candidateId: candidate.candidateId,
        sourceFace: candidate.sourceFace,
        altType,
        languageSlot,
        resolvedFace: resolveFont(candidate.sourceFace, altType, languageSlot),
        displayCandidates: fontFamilyCandidatesForDisplay(
          candidate.sourceFace,
          altType,
          languageSlot,
          { confirmedLocalFonts: [] },
        ),
      };
    });
}

function canvasKitRows(fontNames) {
  return fontNames.map(fontName => ({
    fontName,
    online: resolveCanvasKitFontPlan([fontName]),
    offline: resolveCanvasKitFontPlan(
      [fontName],
      { disableExternalWebFonts: true },
    ),
  }));
}

function governmentSuccessorRows() {
  const legacyNames = ['정부상징 부처명_16040911', 'government_16040911'];
  const successors = ['ROKG', 'ROKG R', '대한민국정부상징체', '대한민국정부상징체 R', 'ROKGR'];
  const subsets = Array.from({ length: 2 ** successors.length }, (_, mask) => (
    successors.filter((_, index) => (mask & (1 << index)) !== 0)
  ));
  const rows = legacyNames.flatMap(fontName => subsets.map(confirmedLocalFonts => ({
    fontName,
    confirmedLocalFonts,
    displayCandidates: fontFamilyCandidatesForDisplay(
      fontName,
      1,
      0,
      { confirmedLocalFonts },
    ),
  })));
  rows.push({
    fontName: '일반 제목체',
    confirmedLocalFonts: successors,
    displayCandidates: fontFamilyCandidatesForDisplay(
      '일반 제목체',
      1,
      0,
      { confirmedLocalFonts: successors },
    ),
  });
  return rows;
}

function displayFallbackProbeRows() {
  return [
    'serif',
    'sans-serif',
    'monospace',
    'KoPub바탕체 Medium',
    '굴림체',
    '휴먼명조',
    'HY중고딕',
    '없는글꼴',
  ].map(fontName => ({
    fontName,
    displayCandidates: fontFamilyCandidatesForDisplay(
      fontName,
      0,
      0,
      { confirmedLocalFonts: [] },
    ),
  }));
}

async function webfontRows(fontNames) {
  const styles = [];
  const requests = [];
  const previousDocument = globalThis.document;
  const previousFontFace = globalThis.FontFace;
  const previousLog = console.log;
  const previousDebug = console.debug;

  const fakeDocument = {
    head: {
      appendChild(element) {
        styles.push(element);
      },
    },
    createElement(tagName) {
      if (tagName !== 'style') throw new Error(`unexpected element in font snapshot: ${tagName}`);
      return { id: '', textContent: '' };
    },
    getElementById(id) {
      return styles.find(style => style.id === id) ?? null;
    },
    fonts: {
      check() {
        return false;
      },
      add() {
        // FontFace constructor records the deterministic request tuple.
      },
    },
  };

  class FakeFontFace {
    constructor(family, source) {
      this.family = family;
      this.source = source;
      requests.push({ family, source });
    }

    async load() {
      return this;
    }
  }

  Object.defineProperty(globalThis, 'document', {
    configurable: true,
    value: fakeDocument,
  });
  Object.defineProperty(globalThis, 'FontFace', {
    configurable: true,
    value: FakeFontFace,
  });
  console.log = () => {};
  console.debug = () => {};
  try {
    await loadWebFonts(fontNames);
    return {
      css: styles.map(style => ({ id: style.id, textContent: style.textContent })),
      requests,
    };
  } finally {
    Object.defineProperty(globalThis, 'document', {
      configurable: true,
      value: previousDocument,
    });
    Object.defineProperty(globalThis, 'FontFace', {
      configurable: true,
      value: previousFontFace,
    });
    console.log = previousLog;
    console.debug = previousDebug;
  }
}

export async function buildRuntimeSnapshot() {
  const approvedCandidates = readJson(CANDIDATES_PATH).ruleCandidates;
  const fontNames = [...REGISTERED_FONTS];
  const webfontSupply = fontNames.map(fontName => ({
    fontName,
    ...getWebFontSupplySnapshot(fontName),
  }));
  const webfontLoad = await webfontRows(fontNames);
  return {
    substitution: rowsAndHash(substitutionRows(approvedCandidates)),
    governmentSuccessor: rowsAndHash(governmentSuccessorRows()),
    displayFallbackProbes: rowsAndHash(displayFallbackProbeRows()),
    registeredFonts: rowsAndHash(fontNames),
    webfontSupply: rowsAndHash(webfontSupply),
    webfontLoad: {
      cssRuleSetCount: webfontLoad.css.length,
      requestCount: webfontLoad.requests.length,
      sha256: sha256Text(canonicalJson(webfontLoad)),
      ...webfontLoad,
    },
    canvasKitPlans: rowsAndHash(canvasKitRows(fontNames)),
  };
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : '';
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    process.stdout.write(canonicalJson(await buildRuntimeSnapshot()));
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
