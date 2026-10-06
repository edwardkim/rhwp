// Analysis-only inventory. Run from the repository root; writes JSON to stdout.
// Literal references are candidates, not a call graph or proof of execution.
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const baseline = '313bd4273dc0cc36a3f3f9b01425797636601b61';
const run = (cmd, args) => execFileSync(cmd, args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
const root = run('git', ['rev-parse', '--show-toplevel']).trim();
process.chdir(root);
const scope = ['src', 'Cargo.toml', 'Cargo.lock', 'gym', 'scripts', 'tests', '.github', 'crates', 'bindings', 'tools', 'npm'];
run('git', ['diff', '--exit-code', baseline, '--', ...scope]);
const tracked = run('git', ['ls-files', '-z', '--', ...scope]).split('\0').filter(Boolean);
const textFiles = new Map();
for (const file of tracked) {
  if (!/\.(rs|py|mjs|js|ts|json|yml|yaml|toml|md|sh|ps1)$/.test(file)) continue;
  if (file.includes('/submissions/') || file.includes('/generated/')) continue;
  const stat = fs.statSync(file);
  if (stat.size > 2 * 1024 * 1024) continue;
  const data = fs.readFileSync(file, 'utf8');
  if (!data.includes('\0')) textFiles.set(file, data);
}
const read = file => fs.readFileSync(file, 'utf8');
const lineAt = (text, offset) => text.slice(0, offset).split('\n').length;
const matches = (file, regex) => [...read(file).matchAll(regex)].map(m => ({
  name: m[1], source: file, line: lineAt(read(file), m.index),
}));
const unique = rows => [...new Map(rows.map(r => [r.name, r])).values()];
const names = rows => rows.map(r => r.name).sort();
const mainSource = read('src/main.rs');
const mainStart = mainSource.indexOf('fn main()');
const mainEnd = mainSource.indexOf('\nfn ', mainStart + 1);
const dispatch = unique([...mainSource.slice(mainStart, mainEnd).matchAll(/Some\("([a-z][a-z0-9-]*)"\)/g)].map(m => ({
  name: m[1], source: 'src/main.rs', line: lineAt(mainSource, mainStart + m.index),
})));
const catalog = matches('src/cli/catalog.rs', /\bspec\(\s*"([^"]+)"/g);
const agent = matches('src/bin/rhwp-agent/caps.rs', /\bname:\s*"([^"]+)"/g);
const kit = matches('src/bin/rhwp-q-kit/main.rs', /\bname:\s*"([^"]+)"/g);
const more = unique(matches('src/bin/rhwp-q-more/main.rs', /Some\("([a-z][a-z0-9-]*)"\)/g));
const pack = unique(matches('src/bin/rhwp-q-pack/main.rs', /Some\("([a-z][a-z0-9-]*)"\)/g));
const metadata = JSON.parse(run('cargo', ['metadata', '--no-deps', '--format-version', '1', '--offline']));
const pkg = metadata.packages.find(p => p.manifest_path === path.join(root, 'Cargo.toml'));
const references = (needle, exclude = '') => [...textFiles].filter(([f, t]) => !f.startsWith(exclude || '\0') && t.includes(needle)).map(([file, t]) => ({ file, line: lineAt(t, t.indexOf(needle)) }));
const targets = pkg.targets.filter(t => t.kind.includes('bin')).map(t => {
  const source = path.relative(root, t.src_path);
  const ownPrefix = source.endsWith('/main.rs') ? path.dirname(source) + '/' : source;
  return {
    name: t.name, source, requiredFeatures: t['required-features'] || [],
    singleCommand: source.startsWith('src/bin/') && !source.endsWith('/main.rs')
      ? (/const COMMAND:\s*&str\s*=\s*"([^"]+)"/.exec(read(source))?.[1]
        || /\benvelope\(\s*"([^"]+)"/.exec(read(source))?.[1] || null)
      : null,
    role: t.name === 'rhwp' ? 'product-cli' : t.name === 'rhwp-agent' ? 'experimental-cli' : 'developer-diagnostic',
    gymRelation: t.name === 'rhwp' ? 'gym-consumes-product' : 'no-direct-reference-observed',
    references: t.name === 'rhwp' ? [] : references(t.name, ownPrefix),
  };
});
const modules = tracked.filter(f => /^src\/(bin|agent|cli)\/.*\.rs$/.test(f)).map(file => {
  const source = read(file);
  const group = file.split('/')[1];
  return {
    file, lines: source.split('\n').length - (source.endsWith('\n') ? 1 : 0),
    role: group === 'cli' ? 'product-cli-adapter' : group === 'agent' ? 'public-dsel-library' : file.startsWith('src/bin/rhwp-agent/') ? 'experimental-cli' : 'developer-diagnostic',
    declaredModules: [...source.matchAll(/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([a-z0-9_]+)\s*;/gm)].map(m => m[1]),
  };
});
// Collect command heads only. Do not copy task solutions, arguments or answers.
// Both task checker cmd and baseline run/cmd arrays are structural evidence.
const gymCommands = new Map();
let gymJsonFiles = 0;
function visit(value, file) {
  if (!value || typeof value !== 'object') return;
  for (const [key, item] of Object.entries(value)) {
    if (['cmd', 'run'].includes(key) && Array.isArray(item) && typeof item[0] === 'string' && /^[a-z][a-z0-9-]*$/.test(item[0])) {
      const row = gymCommands.get(item[0]) || { command: item[0], occurrences: 0, files: new Set(), keys: new Set() };
      row.occurrences++;
      row.files.add(file);
      row.keys.add(key);
      gymCommands.set(item[0], row);
    }
    visit(item, file);
  }
}
for (const [file, source] of textFiles) {
  if (!/^gym\/packs\/.*\.json$/.test(file)) continue;
  gymJsonFiles++;
  visit(JSON.parse(source), file);
}
const gymRows = [...gymCommands.values()].map(r => ({
  command: r.command, occurrences: r.occurrences, fileCount: r.files.size, keys: [...r.keys].sort(),
  inMainDispatch: dispatch.some(d => d.name === r.command),
})).sort((a, b) => a.command.localeCompare(b.command, 'en'));
const result = {
  baseline, method: 'tracked static inventory; literal references are candidates; no benchmark execution',
  exclusions: 'untracked/generated/submissions/binary files; text >2 MiB; mydocs historical references; dynamic calls not resolved',
  scannedTextFiles: textFiles.size, targets, modules,
  commands: { mainDispatch: dispatch, mainCatalog: catalog, agentRegistry: agent, kitRegistry: kit, moreDispatch: more, packDispatch: pack },
  comparisons: {
    mainAgentSameNames: names(agent).filter(n => names(catalog).includes(n)),
    dispatchNotCatalog: names(dispatch).filter(n => !names(catalog).includes(n)),
    catalogNotDispatch: names(catalog).filter(n => !names(dispatch).includes(n)),
    morePackEnvelopeEqualAfterToolName: read('src/bin/rhwp-q-more/envelope.rs').replaceAll('rhwp-q-more', 'TOOL') === read('src/bin/rhwp-q-pack/envelope.rs').replaceAll('rhwp-q-pack', 'TOOL'),
  },
  gym: { scannedPackJsonFiles: gymJsonFiles, commandHeads: gymRows, directAuxiliaryReferences: [...textFiles].filter(([f, t]) => f.startsWith('gym/') && /rhwp-agent|rhwp-q-|agent::dsel/.test(t)).map(([f]) => f) },
  dselReferenceCandidates: [...textFiles].filter(([f, t]) => !f.startsWith('src/agent/') && /agent::dsel|agent::Selector|AGENT_KERNEL_VERSION|\bdsel\b/i.test(t)).map(([file]) => file),
};
console.log(JSON.stringify(result, null, 2));
