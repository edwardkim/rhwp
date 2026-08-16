#!/usr/bin/env node

import {
  existsSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
export const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
export const MANIFEST_RELATIVE_PATH = 'tests/suites/unit-test-tiers.json';
const CFG_TEST = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/g;
const TEST_ATTRIBUTE =
  /#\s*\[\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*test\s*\]/g;

function walkRustFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      return walkRustFiles(entryPath);
    }
    return entry.isFile() && entry.name.endsWith('.rs') ? [entryPath] : [];
  });
}

function blank(masked, source, start, end) {
  for (let index = start; index < end; index += 1) {
    if (source[index] !== '\n' && source[index] !== '\r') {
      masked[index] = ' ';
    }
  }
}

export function maskRustNonCode(source) {
  // parser index는 JavaScript UTF-16 code unit 기준이므로 astral 문자가 있어도
  // source와 같은 index를 유지해야 한다.
  const masked = source.split('');
  let index = 0;
  while (index < source.length) {
    if (source.startsWith('//', index)) {
      const end = source.indexOf('\n', index + 2);
      const boundary = end === -1 ? source.length : end;
      blank(masked, source, index, boundary);
      index = boundary;
      continue;
    }
    if (source.startsWith('/*', index)) {
      let depth = 1;
      let cursor = index + 2;
      while (cursor < source.length && depth > 0) {
        if (source.startsWith('/*', cursor)) {
          depth += 1;
          cursor += 2;
        } else if (source.startsWith('*/', cursor)) {
          depth -= 1;
          cursor += 2;
        } else {
          cursor += 1;
        }
      }
      blank(masked, source, index, cursor);
      index = cursor;
      continue;
    }
    if (source[index] === 'r' || source[index] === 'b') {
      const raw = /^(?:b)?r(#{0,16})"/.exec(source.slice(index, index + 24));
      if (raw) {
        const closing = '"' + raw[1];
        const contentStart = index + raw[0].length;
        const closeAt = source.indexOf(closing, contentStart);
        const boundary =
          closeAt === -1 ? source.length : closeAt + closing.length;
        blank(masked, source, index, boundary);
        index = boundary;
        continue;
      }
    }
    if (source[index] === '"') {
      let cursor = index + 1;
      while (cursor < source.length) {
        if (source[cursor] === '\\') {
          cursor += 2;
        } else if (source[cursor] === '"') {
          cursor += 1;
          break;
        } else {
          cursor += 1;
        }
      }
      blank(masked, source, index, cursor);
      index = cursor;
      continue;
    }
    if (source[index] === "'") {
      let cursor = index + 1;
      if (source[cursor] === '\\') {
        cursor += 2;
      } else {
        cursor += 1;
      }
      if (source[cursor] === "'") {
        cursor += 1;
        blank(masked, source, index, cursor);
        index = cursor;
        continue;
      }
    }
    index += 1;
  }
  return masked.join('');
}

function skipWhitespace(source, start) {
  let cursor = start;
  while (/\s/.test(source[cursor] ?? '')) {
    cursor += 1;
  }
  return cursor;
}

function matchingDelimiter(source, start, open, close) {
  let depth = 0;
  for (let cursor = start; cursor < source.length; cursor += 1) {
    if (source[cursor] === open) {
      depth += 1;
    } else if (source[cursor] === close) {
      depth -= 1;
      if (depth === 0) {
        return cursor;
      }
    }
  }
  return -1;
}

function itemStartAfterAttributes(masked, start) {
  let cursor = skipWhitespace(masked, start);
  while (masked.startsWith('#[', cursor)) {
    const end = matchingDelimiter(masked, cursor + 1, '[', ']');
    if (end === -1) {
      break;
    }
    cursor = skipWhitespace(masked, end + 1);
  }
  const visibility = /^pub(?:\s*\([^)]*\))?\s+/.exec(masked.slice(cursor));
  if (visibility) {
    cursor += visibility[0].length;
  }
  return cursor;
}

function lineAt(source, index) {
  return source.slice(0, index).split('\n').length;
}

function countTestAttributes(masked) {
  return [...masked.matchAll(TEST_ATTRIBUTE)].length;
}

export function classifyTestModule(maskedBody) {
  if (/\bsuper\s*::|\buse\s+super\b/.test(maskedBody)) {
    return { tier: 'white_box', reasons: ['super_private_scope'] };
  }
  if (/\bcrate\s*::/.test(maskedBody)) {
    return { tier: 'test_support', reasons: ['crate_internal_path'] };
  }
  return { tier: 'integration_ready', reasons: ['no_parent_or_crate_path'] };
}

function externalModulePath(file, moduleName, explicitPath = null) {
  const directory = path.dirname(file);
  if (explicitPath !== null) {
    const candidate = path.resolve(directory, explicitPath);
    return existsSync(candidate) ? candidate : undefined;
  }
  const stem = path.basename(file, '.rs');
  const candidates =
    stem === 'mod'
      ? [path.join(directory, moduleName + '.rs')]
      : [
          path.join(directory, stem, moduleName + '.rs'),
          path.join(directory, moduleName + '.rs'),
        ];
  return candidates.find((candidate) => existsSync(candidate));
}

export function analyzeSourceFile(file, root = ROOT) {
  const source = readFileSync(file, 'utf8');
  const masked = maskRustNonCode(source);
  const relative = path.relative(root, file).split(path.sep).join('/');
  const modules = [];
  const supportItems = [];
  const covered = [];
  const moduleOrdinals = new Map();
  const supportOrdinals = new Map();

  for (const match of masked.matchAll(CFG_TEST)) {
    const attributeStart = match.index;
    if (covered.some(([start, end]) => attributeStart > start && attributeStart < end)) {
      continue;
    }
    const itemStart = itemStartAfterAttributes(masked, attributeStart + match[0].length);
    const token = /^[A-Za-z_][A-Za-z0-9_]*!?/.exec(masked.slice(itemStart))?.[0] ?? 'unknown';
    const leading = masked.slice(attributeStart + match[0].length, itemStart);
    const line = lineAt(source, attributeStart);

    if (token === 'mod') {
      let cursor = skipWhitespace(masked, itemStart + token.length);
      const nameMatch = /^[A-Za-z_][A-Za-z0-9_]*/.exec(masked.slice(cursor));
      const moduleName = nameMatch?.[0] ?? 'unknown';
      cursor = skipWhitespace(masked, cursor + moduleName.length);
      let bodyMasked = '';
      let sourcePath = relative;
      if (masked[cursor] === '{') {
        const close = matchingDelimiter(masked, cursor, '{', '}');
        if (close === -1) {
          throw new Error('닫히지 않은 cfg(test) module: ' + relative + ':' + line);
        }
        bodyMasked = masked.slice(cursor + 1, close);
        covered.push([attributeStart, close]);
      } else if (masked[cursor] === ';') {
        const moduleAttributes = source.slice(
          attributeStart + match[0].length,
          itemStart,
        );
        const explicitPath =
          /#\s*\[\s*path\s*=\s*"([^"]+)"\s*\]/.exec(moduleAttributes)?.[1] ??
          null;
        const external = externalModulePath(file, moduleName, explicitPath);
        if (!external) {
          throw new Error('외부 cfg(test) module을 찾을 수 없습니다: ' + relative + ':' + line);
        }
        bodyMasked = maskRustNonCode(readFileSync(external, 'utf8'));
        sourcePath = path.relative(root, external).split(path.sep).join('/');
      }
      const ordinal = (moduleOrdinals.get(moduleName) ?? 0) + 1;
      moduleOrdinals.set(moduleName, ordinal);
      const classification = classifyTestModule(bodyMasked);
      modules.push({
        id: relative + '::' + moduleName + '#' + ordinal,
        file: relative,
        sourcePath,
        line,
        module: moduleName,
        testAttributes: countTestAttributes(bodyMasked),
        tier: classification.tier,
        reasons: classification.reasons,
      });
      continue;
    }

    const ordinal = (supportOrdinals.get(token) ?? 0) + 1;
    supportOrdinals.set(token, ordinal);
    supportItems.push({
      id: relative + '::cfg-' + token + '#' + ordinal,
      file: relative,
      line,
      kind: token,
      testAttributes: countTestAttributes(leading),
    });
  }
  return { modules, supportItems };
}

export function inventorySourceTests(root = ROOT) {
  const modules = [];
  const supportItems = [];
  for (const file of walkRustFiles(path.join(root, 'src'))) {
    const result = analyzeSourceFile(file, root);
    modules.push(...result.modules);
    supportItems.push(...result.supportItems);
  }
  modules.sort((left, right) => left.id.localeCompare(right.id));
  supportItems.sort((left, right) => left.id.localeCompare(right.id));

  const tiers = {
    integration_ready: { modules: 0, testAttributes: 0 },
    test_support: { modules: 0, testAttributes: 0 },
    white_box: { modules: 0, testAttributes: 0 },
  };
  for (const module of modules) {
    tiers[module.tier].modules += 1;
    tiers[module.tier].testAttributes += module.testAttributes;
  }
  const standaloneTests = supportItems.reduce(
    (total, item) => total + item.testAttributes,
    0,
  );
  return {
    summary: {
      files: new Set(
        [...modules, ...supportItems].map((entry) => entry.file),
      ).size,
      cfgTestModules: modules.length,
      cfgTestSupportItems: supportItems.length,
      staticTestAttributes:
        modules.reduce((total, entry) => total + entry.testAttributes, 0) +
        standaloneTests,
      standaloneTestAttributes: standaloneTests,
      tiers,
    },
    modules,
    supportItems,
  };
}

export function buildTierManifest(
  inventory,
  existing = null,
  { acceptBaseline = false } = {},
) {
  const initial = existing === null;
  const previousModules = new Map(
    (existing?.modules ?? []).map((entry) => [entry.id, entry]),
  );
  const previousSupport = new Set(existing?.policy?.allowedSupportItems ?? []);
  const violations = [];
  const modules = inventory.modules.map((entry) => {
    const previous = previousModules.get(entry.id);
    const maximumTestAttributes =
      initial || acceptBaseline
        ? entry.testAttributes
        : (previous?.maximumTestAttributes ?? 0);
    if (!initial && !acceptBaseline && !previous) {
      violations.push('신규 cfg(test) module 금지: ' + entry.id);
    } else if (entry.testAttributes > maximumTestAttributes) {
      violations.push(
        'source unit test 증가 금지: ' +
          entry.id +
          ' (' +
          entry.testAttributes +
          ' > ' +
          maximumTestAttributes +
          ')',
      );
    }
    return { ...entry, maximumTestAttributes };
  });

  const allowedSupportItems = (
    initial || acceptBaseline
      ? inventory.supportItems.map((entry) => entry.id)
      : [...previousSupport]
  ).sort((left, right) => left.localeCompare(right));
  if (!initial && !acceptBaseline) {
    for (const item of inventory.supportItems) {
      if (!previousSupport.has(item.id)) {
        violations.push('신규 cfg(test) support item 금지: ' + item.id);
      }
    }
  }

  const maximumStaticTestAttributes =
    initial || acceptBaseline
      ? inventory.summary.staticTestAttributes
      : existing.policy.maximumStaticTestAttributes;
  if (inventory.summary.staticTestAttributes > maximumStaticTestAttributes) {
    violations.push(
      'source-side test 총량 증가 금지: ' +
        inventory.summary.staticTestAttributes +
        ' > ' +
        maximumStaticTestAttributes,
    );
  }

  return {
    manifest: {
      version: 1,
      policy: {
        newCfgTestModules: 'forbidden',
        newCfgTestSupportItems: 'forbidden',
        maximumStaticTestAttributes,
        allowedSupportItems,
      },
      summary: inventory.summary,
      modules,
      supportItems: inventory.supportItems,
    },
    violations,
  };
}

function manifestText(manifest) {
  return JSON.stringify(manifest, null, 2) + '\n';
}

function loadManifest(root = ROOT) {
  const file = path.join(root, MANIFEST_RELATIVE_PATH);
  return existsSync(file) ? JSON.parse(readFileSync(file, 'utf8')) : null;
}

function summaryLine(manifest) {
  const tiers = manifest.summary.tiers;
  return (
    manifest.summary.staticTestAttributes +
    ' tests / ' +
    manifest.summary.cfgTestModules +
    ' modules / ready ' +
    tiers.integration_ready.testAttributes +
    ' / support ' +
    tiers.test_support.testAttributes +
    ' / white-box ' +
    tiers.white_box.testAttributes +
    ' / cfg support items ' +
    manifest.summary.cfgTestSupportItems
  );
}

function run() {
  const option = process.argv[2];
  if (!['--generate', '--check', '--accept-baseline'].includes(option)) {
    throw new Error(
      '사용법: node scripts/rust-unit-test-tiers.mjs ' +
        '[--generate|--check|--accept-baseline]',
    );
  }
  const existing = loadManifest();
  const inventory = inventorySourceTests();
  const result = buildTierManifest(inventory, existing, {
    acceptBaseline: option === '--accept-baseline',
  });
  if (result.violations.length > 0) {
    throw new Error(result.violations.join('\n'));
  }
  const expected = manifestText(result.manifest);
  const manifestPath = path.join(ROOT, MANIFEST_RELATIVE_PATH);
  if (option === '--check') {
    if (!existing || readFileSync(manifestPath, 'utf8') !== expected) {
      throw new Error(
        'unit test tier manifest가 현재 source와 다릅니다. --generate를 실행하세요.',
      );
    }
    process.stdout.write('[RustUnitTier] 확인 완료: ' + summaryLine(result.manifest) + '\n');
    return;
  }
  writeFileSync(manifestPath, expected);
  process.stdout.write('[RustUnitTier] 생성 완료: ' + summaryLine(result.manifest) + '\n');
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    run();
  } catch (error) {
    process.stderr.write(
      '[RustUnitTier] 오류: ' +
        (error instanceof Error ? error.message : String(error)) +
        '\n',
    );
    process.exitCode = 1;
  }
}
