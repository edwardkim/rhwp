import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const REGISTRY = 'registry+https://github.com/rust-lang/crates.io-index';

export function dioxusCliSource(root = ROOT) {
  const manifest = readFileSync(path.join(root, 'Cargo.toml'), 'utf8');
  const fields = /^\s*subsecond\s*=\s*\{([^}]*)\}/m.exec(manifest)?.[1];
  if (!fields) throw new Error('Cargo.toml에서 subsecond 의존성을 찾지 못했다');
  const version = /\bversion\s*=\s*"([^"]+)"/.exec(fields)?.[1];
  const git = /\bgit\s*=\s*"([^"]+)"/.exec(fields)?.[1];
  const rev = /\brev\s*=\s*"([a-f0-9]{40})"/i.exec(fields)?.[1]?.toLowerCase();

  const block = readFileSync(path.join(root, 'Cargo.lock'), 'utf8')
    .split(/^\[\[package\]\]$/m)
    .find(candidate => /^\s*name = "subsecond"$/m.test(candidate));
  const lockedVersion = /^version = "([^"]+)"$/m.exec(block ?? '')?.[1];
  const lockedSource = /^source = "([^"]+)"$/m.exec(block ?? '')?.[1];
  if (!lockedVersion || !lockedSource) throw new Error('Cargo.lock의 subsecond source를 읽지 못했다');

  if (git && rev && !version) {
    const expected = `git+${git}?rev=${rev}#${rev}`;
    if (lockedSource !== expected) throw new Error(`subsecond rev(${rev})과 lock(${lockedSource})이 다르다`);
    return { kind: 'git', git, rev, version: lockedVersion };
  }
  if (version?.startsWith('=') && !git && lockedSource === REGISTRY) {
    const pinned = version.slice(1);
    if (pinned !== lockedVersion) throw new Error(`subsecond registry 핀(${pinned})과 lock(${lockedVersion})이 다르다`);
    return { kind: 'registry', version: pinned };
  }
  throw new Error('subsecond는 exact registry version 또는 git rev로 고정해야 한다');
}

export const dioxusCliVersion = (root = ROOT) => dioxusCliSource(root).version;

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.stdout.write(`${JSON.stringify(dioxusCliSource())}\n`);
}
