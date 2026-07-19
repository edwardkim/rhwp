#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { lstatSync, mkdirSync, readFileSync, readlinkSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_FILE = fileURLToPath(import.meta.url);
const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const STUDIO_DIR = path.join(ROOT, 'rhwp-studio');
const METRICS_DIR = path.join(ROOT, 'scripts/frontend-metrics');
const metricsRequire = createRequire(path.join(METRICS_DIR, 'package.json'));

const { ESLint } = metricsRequire('eslint');
const sonarjs = metricsRequire('eslint-plugin-sonarjs');
const tsParser = metricsRequire('@typescript-eslint/parser');
const ts = metricsRequire('typescript');

const DEFAULT_OUT = 'output/frontend-metrics/metrics.json';
const DEFAULT_SUMMARY = 'output/frontend-metrics/summary.md';

const INCLUDE_GROUPS = [
  {
    id: 'studio-runtime',
    label: 'Studio runtime',
    patterns: ['rhwp-studio/src/**/*.{ts,tsx,js,mjs}'],
    test: (file) => file.startsWith('rhwp-studio/src/') && isCodeFile(file),
  },
  {
    id: 'chrome-extension',
    label: 'Chrome extension',
    patterns: ['rhwp-chrome/**/*.{js,mjs,ts}'],
    test: (file) => file.startsWith('rhwp-chrome/') && isCodeFile(file),
  },
  {
    id: 'firefox-extension',
    label: 'Firefox extension',
    patterns: ['rhwp-firefox/**/*.{js,mjs,ts}'],
    test: (file) => file.startsWith('rhwp-firefox/') && isCodeFile(file),
  },
  {
    id: 'safari-extension',
    label: 'Safari extension',
    patterns: ['rhwp-safari/src/**/*.{js,mjs,ts}'],
    test: (file) => file.startsWith('rhwp-safari/src/') && isCodeFile(file),
  },
  {
    id: 'shared-frontend',
    label: 'Shared frontend',
    patterns: ['rhwp-shared/**/*.{js,mjs,ts}'],
    test: (file) => file.startsWith('rhwp-shared/') && isCodeFile(file),
  },
  {
    id: 'vscode-extension',
    label: 'VS Code extension',
    patterns: ['rhwp-vscode/src/**/*.{ts,js}'],
    test: (file) => file.startsWith('rhwp-vscode/src/') && isCodeFile(file),
  },
  {
    id: 'npm-editor',
    label: 'npm editor wrapper',
    patterns: ['npm/editor/**/*.{js,ts}'],
    test: (file) => file.startsWith('npm/editor/') && isCodeFile(file),
  },
];

const EXCLUDE_RULES = [
  'node_modules/',
  'dist/',
  'pkg/',
  '*.min.js',
  'vendored/generated data',
  '**/{test,tests,e2e}/',
  '*.{test,spec}.{js,mjs,ts,tsx}',
  'assets/fonts/',
  'icons/',
  '_locales/',
  'certs/',
  'snapshot/output/cache files',
];

function parseArgs(argv) {
  const args = {
    out: DEFAULT_OUT,
    summary: DEFAULT_SUMMARY,
    compare: null,
    help: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--help' || arg === '-h') {
      args.help = true;
    } else if (arg === '--out') {
      args.out = argv[++i];
    } else if (arg === '--summary') {
      args.summary = argv[++i];
    } else if (arg === '--compare') {
      args.compare = argv[++i];
    } else {
      throw new Error(`Unknown option: ${arg}`);
    }
  }

  return args;
}

function printHelp() {
  console.log(`frontend metrics snapshot helper

Usage:
  node scripts/frontend-metrics.mjs [--out <metrics.json>] [--summary <summary.md>]
                                    [--compare <baseline-metrics.json>]

Default output:
  ${DEFAULT_OUT}
  ${DEFAULT_SUMMARY}

Notes:
  - Uses the private scripts/frontend-metrics package for analysis dependencies.
  - sonarjs/cognitive-complexity is advisory in Phase 0 and does not act as a fail gate.
  - --compare records aggregate and per-function cognitive-complexity deltas.
`);
}

function normalizePath(file) {
  return file.split(path.sep).join('/');
}

function isCodeFile(file) {
  return /\.(ts|tsx|js|mjs)$/i.test(file);
}

function hasExcludedSegment(file, segment) {
  return file === segment || file.startsWith(`${segment}/`) || file.includes(`/${segment}/`);
}

function isTestFile(file) {
  return /(^|\/)(tests?|e2e)(\/|$)/i.test(file)
    || /\.(test|spec)\.(js|mjs|ts|tsx)$/i.test(file);
}

function isExcluded(file) {
  if (hasExcludedSegment(file, 'node_modules')) return true;
  if (hasExcludedSegment(file, 'dist')) return true;
  if (hasExcludedSegment(file, 'pkg')) return true;
  if (hasExcludedSegment(file, 'output')) return true;
  if (hasExcludedSegment(file, 'cache')) return true;
  if (hasExcludedSegment(file, 'snapshots')) return true;
  if (file.endsWith('.min.js')) return true;
  if (isTestFile(file)) return true;
  if (file.startsWith('assets/fonts/')) return true;
  if (file.includes('/icons/') || file.startsWith('icons/')) return true;
  if (file.includes('/_locales/') || file.startsWith('_locales/')) return true;
  if (file.includes('/certs/') || file.startsWith('certs/')) return true;
  return false;
}

function gitFiles() {
  return execFileSync(
    'git',
    ['ls-files', '--cached', '--others', '--exclude-standard'],
    { cwd: ROOT, encoding: 'utf8' },
  )
    .split(/\r?\n/)
    .filter(Boolean)
    .map(normalizePath)
    .sort();
}

function gitValue(args) {
  return execFileSync('git', args, { cwd: ROOT, encoding: 'utf8' }).trim();
}

function optionalGitValue(args) {
  try {
    return gitValue(args);
  } catch {
    return undefined;
  }
}

function canonicalDevelCommit() {
  return optionalGitValue(['rev-parse', '--verify', 'refs/remotes/upstream/devel'])
    ?? optionalGitValue(['rev-parse', '--verify', 'refs/remotes/origin/devel']);
}

function dirtyPaths() {
  return execFileSync(
    'git',
    ['status', '--porcelain=v1', '--untracked-files=all'],
    { cwd: ROOT, encoding: 'utf8' },
  )
    .split(/\r?\n/)
    .filter(Boolean)
    .map((line) => line.slice(3))
    .map((file) => file.includes(' -> ') ? file.split(' -> ').at(-1) : file)
    .map((file) => file.replace(/^"|"$/g, ''))
    .map(normalizePath)
    .sort();
}

function measuredSourceDirtyPaths(paths) {
  return paths.filter((file) => {
    if (isExcluded(file)) return false;
    return INCLUDE_GROUPS.some((group) => group.test(file));
  });
}

function classifyFiles(files) {
  const included = [];
  const excluded = [];

  for (const file of files) {
    if (isExcluded(file)) {
      excluded.push(file);
      continue;
    }

    const group = INCLUDE_GROUPS.find((candidate) => candidate.test(file));
    if (group) {
      included.push({ file, group: group.id, label: group.label, legacy: Boolean(group.legacy) });
    }
  }

  return { included, excluded };
}

function readText(file) {
  return readFileSync(path.join(ROOT, file), 'utf8');
}

function lineCounts(text) {
  const lines = text.length === 0 ? [] : text.split(/\r?\n/);
  if (lines.at(-1) === '') lines.pop();
  return {
    lines: lines.length,
    codeLines: lines.filter((line) => line.trim().length > 0).length,
  };
}

function scriptKindFor(file) {
  if (file.endsWith('.tsx')) return ts.ScriptKind.TSX;
  if (file.endsWith('.ts')) return ts.ScriptKind.TS;
  return ts.ScriptKind.JS;
}

function functionName(node) {
  if (node.name && ts.isIdentifier(node.name)) return node.name.text;
  const parent = node.parent;
  if (parent && ts.isVariableDeclaration(parent) && ts.isIdentifier(parent.name)) return parent.name.text;
  if (parent && ts.isPropertyAssignment(parent)) return parent.name.getText();
  if (parent && ts.isAssignmentExpression?.(parent)) return parent.left.getText();
  if (parent && ts.isMethodDeclaration(parent) && parent.name) return parent.name.getText();
  return '<anonymous>';
}

function isFunctionLike(node) {
  return ts.isFunctionDeclaration(node)
    || ts.isFunctionExpression(node)
    || ts.isArrowFunction(node)
    || ts.isMethodDeclaration(node)
    || ts.isConstructorDeclaration(node)
    || ts.isGetAccessorDeclaration(node)
    || ts.isSetAccessorDeclaration(node);
}

function functionMetrics(file, text) {
  if (!isCodeFile(file)) return [];

  const sourceFile = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, scriptKindFor(file));
  const lines = text.split(/\r?\n/);
  const functions = [];

  function visit(node) {
    if (isFunctionLike(node)) {
      const start = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile)).line + 1;
      const end = sourceFile.getLineAndCharacterOfPosition(node.getEnd()).line + 1;
      const physicalLoc = Math.max(1, end - start + 1);
      const codeLoc = lines.slice(start - 1, end).filter((line) => line.trim().length > 0).length;
      functions.push({
        file,
        name: functionName(node),
        line: start,
        endLine: end,
        loc: physicalLoc,
        codeLoc,
        kind: ts.SyntaxKind[node.kind] ?? String(node.kind),
      });
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return functions;
}

function assignFunctionIds(functions) {
  const occurrences = new Map();
  return functions.map((fn) => {
    const key = `${fn.file}::${fn.kind}:${fn.name}`;
    const occurrence = (occurrences.get(key) ?? 0) + 1;
    occurrences.set(key, occurrence);
    return { ...fn, id: `${key}#${occurrence}` };
  });
}

function hasModifier(node, kind) {
  return Boolean(node.modifiers?.some((modifier) => modifier.kind === kind));
}

function bindingNames(name) {
  if (ts.isIdentifier(name)) return [name.text];
  if (ts.isObjectBindingPattern(name) || ts.isArrayBindingPattern(name)) {
    return name.elements.flatMap((element) => {
      if (!ts.isBindingElement(element)) return [];
      return bindingNames(element.name);
    });
  }
  return [name.getText()];
}

function exportedSurface(sourceFile) {
  const entries = [];

  function add(node, name, kind, source = null) {
    entries.push({
      name,
      kind,
      source,
      line: sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile)).line + 1,
    });
  }

  for (const statement of sourceFile.statements) {
    if (ts.isExportDeclaration(statement)) {
      const source = statement.moduleSpecifier && ts.isStringLiteral(statement.moduleSpecifier)
        ? statement.moduleSpecifier.text
        : null;
      if (!statement.exportClause) {
        add(statement, '*', 'ExportDeclaration', source);
      } else if (ts.isNamedExports(statement.exportClause)) {
        for (const element of statement.exportClause.elements) {
          add(element, element.name.text, 'ExportSpecifier', source);
        }
      } else {
        add(statement.exportClause, statement.exportClause.name.text, 'NamespaceExport', source);
      }
      continue;
    }

    if (ts.isExportAssignment(statement)) {
      add(statement, 'default', 'ExportAssignment');
      continue;
    }

    if (!hasModifier(statement, ts.SyntaxKind.ExportKeyword)) continue;
    const isDefault = hasModifier(statement, ts.SyntaxKind.DefaultKeyword);

    if (ts.isVariableStatement(statement)) {
      for (const declaration of statement.declarationList.declarations) {
        for (const name of bindingNames(declaration.name)) {
          add(declaration, name, 'VariableDeclaration');
        }
      }
      continue;
    }

    const name = statement.name?.getText(sourceFile) ?? (isDefault ? 'default' : '<anonymous>');
    add(statement, name, ts.SyntaxKind[statement.kind] ?? String(statement.kind));
  }

  return entries;
}

function typeSurfaceMetrics(file, text) {
  if (!isCodeFile(file)) {
    return {
      file,
      any: 0,
      asAny: 0,
      thisAny: 0,
      exportCount: 0,
      exports: [],
    };
  }

  const sourceFile = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, scriptKindFor(file));
  const counts = { any: 0, asAny: 0, thisAny: 0 };

  function visit(node) {
    if (node.kind === ts.SyntaxKind.AnyKeyword) counts.any += 1;
    if (ts.isAsExpression(node) && node.type.kind === ts.SyntaxKind.AnyKeyword) counts.asAny += 1;
    if (ts.isParameter(node)
      && node.name.getText(sourceFile) === 'this'
      && node.type?.kind === ts.SyntaxKind.AnyKeyword) {
      counts.thisAny += 1;
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  const exports = exportedSurface(sourceFile);
  return {
    file,
    ...counts,
    exportCount: exports.length,
    exports,
  };
}

function closestFunction(functions, file, line) {
  const candidates = functions
    .filter((fn) => fn.file === file && fn.line <= line && fn.endLine >= line)
    .sort((a, b) => (a.endLine - a.line) - (b.endLine - b.line));
  return candidates[0] ?? null;
}

async function cognitiveComplexity(files, functions) {
  if (files.length === 0) return { entries: [], errors: [] };

  const eslint = new ESLint({
    cwd: ROOT,
    overrideConfigFile: true,
    allowInlineConfig: false,
    ignore: false,
    overrideConfig: [
      {
        files: ['**/*.{js,mjs,ts,tsx}'],
        languageOptions: {
          ecmaVersion: 'latest',
          sourceType: 'module',
          parser: tsParser,
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
        plugins: {
          sonarjs,
        },
        rules: {
          'sonarjs/cognitive-complexity': ['warn', 0],
        },
      },
    ],
  });

  const results = await eslint.lintFiles(files);
  const entries = [];
  const errors = [];

  for (const result of results) {
    const file = normalizePath(path.relative(ROOT, result.filePath));
    for (const message of result.messages) {
      if (message.ruleId !== 'sonarjs/cognitive-complexity') {
        if (message.fatal || message.severity === 2) {
          errors.push({
            file,
            line: message.line,
            column: message.column,
            ruleId: message.ruleId,
            message: message.message,
          });
        }
        continue;
      }

      const complexity = extractComplexity(message.message);
      const fn = closestFunction(functions, file, message.line);
      entries.push({
        file,
        line: message.line,
        column: message.column,
        complexity,
        functionId: fn?.id ?? `${file}::<unknown>@${message.line}`,
        function: fn?.name ?? '<unknown>',
        functionKind: fn?.kind ?? null,
        functionLine: fn?.line ?? null,
        functionLoc: fn?.loc ?? null,
      });
    }
  }

  entries.sort((a, b) => (b.complexity ?? -1) - (a.complexity ?? -1));
  return { entries, errors };
}

function extractComplexity(message) {
  const fromMatch = message.match(/\bfrom\s+(\d+)\s+to\b/i);
  if (fromMatch) return Number(fromMatch[1]);
  const numberMatch = message.match(/\b(\d+)\b/);
  return numberMatch ? Number(numberMatch[1]) : null;
}

function cognitiveComplexitySummary(entries) {
  const values = entries
    .map((entry) => entry.complexity)
    .filter((value) => typeof value === 'number')
    .sort((a, b) => b - a);
  const over25 = values.filter((value) => value > 25);
  const over100 = values.filter((value) => value > 100);
  return {
    reportedFunctions: values.length,
    total: values.reduce((sum, value) => sum + value, 0),
    top20Sum: values.slice(0, 20).reduce((sum, value) => sum + value, 0),
    over25Count: over25.length,
    over25Sum: over25.reduce((sum, value) => sum + value, 0),
    over100Count: over100.length,
    over100Sum: over100.reduce((sum, value) => sum + value, 0),
    max: values[0] ?? 0,
  };
}

function groupTotals(included, fileMetrics, functions, complexityEntries, typeMetrics) {
  const totals = {};
  for (const group of INCLUDE_GROUPS) {
    totals[group.id] = {
      label: group.label,
      files: 0,
      lines: 0,
      codeLines: 0,
      functions: 0,
      filesOver1200: 0,
      filesOver2000: 0,
      ccOver25: 0,
      ccOver100: 0,
      cognitiveComplexityTotal: 0,
      cognitiveComplexityTop20Sum: 0,
      cognitiveComplexityOver25Sum: 0,
      maxCognitiveComplexity: 0,
      any: 0,
      asAny: 0,
      thisAny: 0,
      exportCount: 0,
      _complexities: [],
    };
  }

  const groupByFile = new Map(included.map((entry) => [entry.file, entry.group]));

  for (const metric of fileMetrics) {
    const total = totals[metric.group];
    total.files += 1;
    total.lines += metric.lines;
    total.codeLines += metric.codeLines;
    if (metric.lines > 1200) total.filesOver1200 += 1;
    if (metric.lines > 2000) total.filesOver2000 += 1;
  }

  for (const fn of functions) {
    const group = groupByFile.get(fn.file);
    if (group) totals[group].functions += 1;
  }

  for (const entry of complexityEntries) {
    const group = groupByFile.get(entry.file);
    if (!group) continue;
    const total = totals[group];
    if (entry.complexity !== null) {
      total._complexities.push(entry.complexity);
      total.maxCognitiveComplexity = Math.max(total.maxCognitiveComplexity, entry.complexity);
      if (entry.complexity > 25) total.ccOver25 += 1;
      if (entry.complexity > 100) total.ccOver100 += 1;
    }
  }

  for (const metric of typeMetrics) {
    const group = groupByFile.get(metric.file);
    if (!group) continue;
    totals[group].any += metric.any;
    totals[group].asAny += metric.asAny;
    totals[group].thisAny += metric.thisAny;
    totals[group].exportCount += metric.exportCount;
  }

  for (const total of Object.values(totals)) {
    const summary = cognitiveComplexitySummary(
      total._complexities.map((complexity) => ({ complexity })),
    );
    total.cognitiveComplexityTotal = summary.total;
    total.cognitiveComplexityTop20Sum = summary.top20Sum;
    total.cognitiveComplexityOver25Sum = summary.over25Sum;
    delete total._complexities;
  }

  return totals;
}

function sha256(data) {
  return createHash('sha256').update(data).digest('hex');
}

function fileFingerprint(file) {
  const full = path.join(ROOT, file);
  const stat = lstatSync(full);
  const content = readFileSync(full);
  return {
    bytes: content.byteLength,
    sha256: sha256(content),
    symlinkTarget: stat.isSymbolicLink() ? normalizePath(readlinkSync(full)) : null,
  };
}

function browserDuplicateCandidates(files) {
  const browserFiles = files
    .map((entry) => entry.file)
    .filter((file) => /^rhwp-(chrome|firefox)\//.test(file) || file.startsWith('rhwp-safari/src/'));

  const byRelativePath = new Map();
  for (const file of browserFiles) {
    const browser = file.startsWith('rhwp-chrome/')
      ? 'chrome'
      : file.startsWith('rhwp-firefox/')
        ? 'firefox'
        : 'safari';
    const relative = file
      .replace(/^rhwp-chrome\//, '')
      .replace(/^rhwp-firefox\//, '')
      .replace(/^rhwp-safari\/src\//, '');
    if (!byRelativePath.has(relative)) byRelativePath.set(relative, []);
    byRelativePath.get(relative).push({ browser, file, ...fileFingerprint(file) });
  }

  return [...byRelativePath.entries()]
    .filter(([, refs]) => new Set(refs.map((ref) => ref.browser)).size > 1)
    .map(([relative, refs]) => ({
      relative,
      identicalContent: new Set(refs.map((ref) => ref.sha256)).size === 1,
      refs,
    }))
    .sort((a, b) => a.relative.localeCompare(b.relative));
}

function fontReferences(files) {
  const candidates = files.filter((file) => {
    if (isExcluded(file)) return false;
    if (!/^(rhwp-studio|rhwp-chrome|rhwp-firefox|rhwp-safari|rhwp-shared|rhwp-vscode|npm|scripts|\.github)\//.test(file)
      && !file.startsWith('mydocs/manual/')
      && file !== 'mydocs/tech/font_fallback_strategy.md'
      && file !== 'mydocs/tech/task_m100_2023_frontend_contract_guardrails.md'
      && file !== 'README.md'
      && file !== 'THIRD_PARTY_LICENSES.md') return false;
    if (/\.(png|jpg|jpeg|gif|svg|ico|woff2?|ttf|otf|wasm|pdf)$/i.test(file)) return false;
    if (file.endsWith('package-lock.json')) return false;
    return true;
  });

  const refs = [];
  const patterns = [
    /assets\/fonts/g,
    /fonts\//g,
    /\.woff2\b/g,
    /FONTS\.md/g,
    /THIRD_PARTY_LICENSES/g,
  ];

  for (const file of candidates) {
    let text;
    try {
      text = readText(file);
    } catch {
      continue;
    }
    const lines = text.split(/\r?\n/);
    for (let index = 0; index < lines.length; index += 1) {
      if (patterns.some((pattern) => {
        pattern.lastIndex = 0;
        return pattern.test(lines[index]);
      })) {
        refs.push({
          file,
          line: index + 1,
          text: lines[index].trim().slice(0, 220),
        });
      }
    }
  }

  return refs;
}

function fontAssetInventory(files) {
  const fontFiles = files
    .filter((file) => file.startsWith('assets/fonts/') && file.endsWith('.woff2'))
    .map((file) => ({ file, ...fileFingerprint(file) }));
  const licenseFiles = [
    'assets/fonts/FONTS.md',
    'THIRD_PARTY_LICENSES.md',
    'assets/fonts/SourceHanSerifK-OFL.txt',
  ].map((file) => ({ file, ...fileFingerprint(file) }));
  const studioLink = path.join(ROOT, 'rhwp-studio/public/fonts');

  return {
    canonicalDirectory: 'assets/fonts',
    studioPublicLink: {
      file: 'rhwp-studio/public/fonts',
      target: normalizePath(readlinkSync(studioLink)),
    },
    files: fontFiles,
    fileCount: fontFiles.length,
    totalBytes: fontFiles.reduce((sum, file) => sum + file.bytes, 0),
    licenseFiles,
  };
}

function complexityEntriesFromSnapshot(snapshot) {
  return snapshot.cognitiveComplexityEntries
    ?? snapshot.cognitiveComplexityTop
    ?? snapshot.cognitive_complexity
    ?? [];
}

function complexityEntryId(entry) {
  return entry.functionId
    ?? `${entry.file}::${entry.function ?? '<unknown>'}@${entry.functionLine ?? entry.line ?? 0}`;
}

function summaryDelta(before, after) {
  return Object.fromEntries(
    Object.keys(after).map((key) => [key, after[key] - (before[key] ?? 0)]),
  );
}

function compareComplexity(currentEntries, baselineFile) {
  const baselinePath = path.resolve(ROOT, baselineFile);
  const baseline = JSON.parse(readFileSync(baselinePath, 'utf8'));
  if (baseline.schemaVersion !== 2 || !Array.isArray(baseline.cognitiveComplexityEntries)) {
    throw new Error(`--compare requires a schemaVersion 2 frontend snapshot: ${baselineFile}`);
  }
  const baselineEntries = complexityEntriesFromSnapshot(baseline);
  const beforeById = new Map(baselineEntries.map((entry) => [complexityEntryId(entry), entry]));
  const afterById = new Map(currentEntries.map((entry) => [complexityEntryId(entry), entry]));
  const ids = new Set([...beforeById.keys(), ...afterById.keys()]);
  const functionDiff = [];

  for (const functionId of ids) {
    const before = beforeById.get(functionId);
    const after = afterById.get(functionId);
    const beforeComplexity = before?.complexity ?? 0;
    const afterComplexity = after?.complexity ?? 0;
    const delta = afterComplexity - beforeComplexity;
    if (delta === 0 && before && after) continue;
    functionDiff.push({
      functionId,
      file: after?.file ?? before?.file ?? null,
      function: after?.function ?? before?.function ?? '<unknown>',
      before: before ? beforeComplexity : null,
      after: after ? afterComplexity : null,
      delta,
      status: !before ? 'added' : !after ? 'removed' : 'changed',
    });
  }

  functionDiff.sort((a, b) => Math.abs(b.delta) - Math.abs(a.delta)
    || a.functionId.localeCompare(b.functionId));
  const beforeSummary = baseline.cognitiveComplexity
    ?? cognitiveComplexitySummary(baselineEntries);
  const afterSummary = cognitiveComplexitySummary(currentEntries);

  return {
    baselineFile: normalizePath(path.relative(ROOT, baselinePath)),
    baselineCommit: baseline.git?.headCommit ?? baseline.git?.commit ?? null,
    before: beforeSummary,
    after: afterSummary,
    delta: summaryDelta(beforeSummary, afterSummary),
    functionDiff,
  };
}

function writeJson(file, data) {
  const full = path.resolve(ROOT, file);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, `${JSON.stringify(data, null, 2)}\n`);
}

function writeSummary(file, data) {
  const full = path.resolve(ROOT, file);
  mkdirSync(path.dirname(full), { recursive: true });

  const groupRows = Object.entries(data.groupTotals)
    .map(([, total]) => `| ${total.label} | ${total.files} | ${total.lines} | ${total.functions} | ${total.cognitiveComplexityTotal} | ${total.cognitiveComplexityTop20Sum} | ${total.ccOver25} | ${total.cognitiveComplexityOver25Sum} | ${total.maxCognitiveComplexity} | ${total.any} | ${total.exportCount} |`)
    .join('\n');

  const topComplexity = data.cognitiveComplexityEntries
    .slice(0, 20)
    .map((entry) => `| ${entry.complexity ?? ''} | ${entry.functionLoc ?? ''} | \`${entry.function}\` | \`${entry.file}:${entry.line}\` |`)
    .join('\n');

  const topFunctionLoc = data.functionLocTop
    .slice(0, 20)
    .map((entry) => `| ${entry.loc} | \`${entry.name}\` | \`${entry.file}:${entry.line}\` |`)
    .join('\n');

  const comparison = data.comparison
    ? `## 직전 snapshot 대비\n\n- 기준: \`${data.comparison.baselineFile}\`\n- CC 총합 delta: ${data.comparison.delta.total}\n- 상위 20 합 delta: ${data.comparison.delta.top20Sum}\n- CC>25 합 delta: ${data.comparison.delta.over25Sum}\n\n| Delta | Before | After | Function | File |\n|------:|-------:|------:|----------|------|\n${data.comparison.functionDiff.slice(0, 20).map((entry) => `| ${entry.delta} | ${entry.before ?? ''} | ${entry.after ?? ''} | \`${entry.function}\` | \`${entry.file}\` |`).join('\n')}\n\n`
    : '';

  const body = `# 프론트 metrics 요약

- generatedAt: ${data.generatedAt}
- commit: ${data.git.headCommit}
- measured source clean: ${data.git.measuredSourceClean}
- advisory: Phase 0 기준선 고정용 snapshot이며 CI fail gate가 아니다.

## Cognitive Complexity 총량

| Reported functions | Total CC | Top 20 sum | CC>25 count | CC>25 sum | CC>100 count | Max CC |
|-------------------:|---------:|-----------:|------------:|----------:|-------------:|-------:|
| ${data.cognitiveComplexity.reportedFunctions} | ${data.cognitiveComplexity.total} | ${data.cognitiveComplexity.top20Sum} | ${data.cognitiveComplexity.over25Count} | ${data.cognitiveComplexity.over25Sum} | ${data.cognitiveComplexity.over100Count} | ${data.cognitiveComplexity.max} |

## Group 합계

| Group | Files | Lines | Functions | Total CC | Top 20 sum | CC>25 | CC>25 sum | Max CC | any | exports |
|------|------:|------:|----------:|---------:|-----------:|------:|----------:|-------:|----:|--------:|
${groupRows}

## Cognitive Complexity 상위

| CC | Function LOC | Function | Location |
|---:|-------------:|----------|----------|
${topComplexity}

## 함수 LOC 상위

| LOC | Function | Location |
|----:|----------|----------|
${topFunctionLoc}

${comparison}## 비고

- 이 결과는 Phase 0 advisory snapshot이며 CI fail gate가 아니다.
- JSON 전체 경로: \`${data.snapshotPath}\`.
- \`any\`/\`as any\`/\`this: any\`와 export 수는 TypeScript AST 기준이며 일부 항목은 서로 포함 관계다.
- ESLint parse/fatal diagnostics는 \`metrics.json\`에 보존한다.
`;

  writeFileSync(full, body);
}

function topBy(list, field, limit = 100) {
  return [...list]
    .filter((entry) => typeof entry[field] === 'number')
    .sort((a, b) => b[field] - a[field])
    .slice(0, limit);
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    printHelp();
    return;
  }

  const allFiles = gitFiles();
  const { included, excluded } = classifyFiles(allFiles);
  const includedCode = included.filter((entry) => isCodeFile(entry.file)).map((entry) => entry.file);

  const fileMetrics = [];
  const rawFunctions = [];
  const typeMetrics = [];
  for (const entry of included) {
    const text = readText(entry.file);
    const counts = lineCounts(text);
    fileMetrics.push({ ...entry, ...counts });
    rawFunctions.push(...functionMetrics(entry.file, text));
    typeMetrics.push(typeSurfaceMetrics(entry.file, text));
  }

  const functions = assignFunctionIds(rawFunctions);
  const { entries: cognitiveEntries, errors: eslintErrors } = await cognitiveComplexity(includedCode, functions);
  const groupTotalsData = groupTotals(included, fileMetrics, functions, cognitiveEntries, typeMetrics);
  const complexitySummary = cognitiveComplexitySummary(cognitiveEntries);
  const statusPaths = dirtyPaths();
  const sourceDirtyPaths = measuredSourceDirtyPaths(statusPaths);
  const develCommit = canonicalDevelCommit();
  const packageVersions = {
    eslint: metricsRequire('eslint/package.json').version,
    eslintPluginSonarjs: metricsRequire('eslint-plugin-sonarjs/package.json').version,
    typescriptEslintParser: metricsRequire('@typescript-eslint/parser/package.json').version,
    typescript: metricsRequire('typescript/package.json').version,
  };

  const data = {
    schemaVersion: 2,
    generatedAt: new Date().toISOString(),
    advisory: 'Phase 0 snapshot only. Do not use as a fail gate before baseline approval.',
    snapshotPath: normalizePath(args.out),
    git: {
      headCommit: gitValue(['rev-parse', 'HEAD']),
      ...(develCommit ? { upstreamDevelCommit: develCommit } : {}),
      clean: statusPaths.length === 0,
      dirtyPaths: statusPaths,
      measuredSourceClean: sourceDirtyPaths.length === 0,
      measuredSourceDirtyPaths: sourceDirtyPaths,
    },
    tools: {
      ...packageVersions,
      node: process.version,
      platform: process.platform,
      arch: process.arch,
      osRelease: os.release(),
      scriptSha256: sha256(readFileSync(SCRIPT_FILE)),
      metricsPackageLockSha256: sha256(readFileSync(path.join(METRICS_DIR, 'package-lock.json'))),
      studioPackageLockSha256: sha256(readFileSync(path.join(STUDIO_DIR, 'package-lock.json'))),
    },
    thresholds: {
      maxFileLines: 1200,
      largeFileLines: 2000,
      cognitiveComplexityWarn: 25,
      cognitiveComplexityExtreme: 100,
    },
    includeGroups: INCLUDE_GROUPS.map((group) => ({
      id: group.id,
      label: group.label,
      patterns: group.patterns,
      legacy: Boolean(group.legacy),
    })),
    excludeRules: EXCLUDE_RULES,
    includedFiles: included,
    excludedTrackedFilesConsidered: excluded,
    groupTotals: groupTotalsData,
    fileMetrics,
    fileLinesTop: topBy(fileMetrics, 'lines', 100),
    functionLocTop: topBy(functions, 'loc', 100),
    cognitiveComplexity: complexitySummary,
    cognitiveComplexityEntries: cognitiveEntries,
    cognitiveComplexityCounts: {
      reportedFunctions: complexitySummary.reportedFunctions,
      over25: complexitySummary.over25Count,
      over100: complexitySummary.over100Count,
    },
    typeSurfaceTop: topBy(typeMetrics.map(({ exports: _exports, ...metric }) => ({
      ...metric,
      score: metric.any + metric.asAny + metric.thisAny + metric.exportCount,
    })), 'score', 100),
    exportSurface: typeMetrics
      .filter((metric) => metric.exportCount > 0)
      .map(({ file, exportCount, exports }) => ({ file, exportCount, exports })),
    browserDuplicateCandidates: browserDuplicateCandidates(included),
    fontReferenceMap: fontReferences(allFiles),
    fontAssets: fontAssetInventory(allFiles),
    eslintDiagnostics: eslintErrors,
  };

  if (args.compare) data.comparison = compareComplexity(cognitiveEntries, args.compare);

  writeJson(args.out, data);
  writeSummary(args.summary, data);

  console.log(`metrics: ${args.out}`);
  console.log(`summary: ${args.summary}`);
  console.log(`included files: ${included.length}`);
  console.log(`CC total: ${data.cognitiveComplexity.total}`);
  console.log(`CC top 20 sum: ${data.cognitiveComplexity.top20Sum}`);
  console.log(`CC>25 sum: ${data.cognitiveComplexity.over25Sum}`);
  console.log(`CC>25: ${data.cognitiveComplexityCounts.over25}`);
  console.log(`CC>100: ${data.cognitiveComplexityCounts.over100}`);
  if (eslintErrors.length > 0) {
    console.log(`eslint diagnostics: ${eslintErrors.length}`);
  }
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exitCode = 1;
});
