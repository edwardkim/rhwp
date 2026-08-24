/**
 * 설치할 Dioxus CLI의 정본을 `subsecond` 의존성과 Cargo.lock에서 유도한다.
 *
 * registry exact version과 official git exact revision을 모두 지원한다. 어느 경우든
 * 매니페스트 요구와 lock이 실제로 해결한 source가 다르면 설치 전에 멈춘다.
 */
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const CRATE = 'subsecond';
const CRATES_IO_SOURCE = 'registry+https://github.com/rust-lang/crates.io-index';

function dependencyFields(manifest) {
  const match = new RegExp(`^\\s*${CRATE}\\s*=\\s*\\{([^}]*)\\}`, 'm').exec(manifest);
  if (!match) throw new Error(`Cargo.toml 에서 \`${CRATE}\` 의존성을 찾지 못했다`);
  return match[1];
}

function manifestSource(manifest) {
  const fields = dependencyFields(manifest);
  const version = /\bversion\s*=\s*"([^"]+)"/.exec(fields)?.[1];
  const registry = /\bregistry\s*=\s*"([^"]+)"/.exec(fields)?.[1];
  const git = /\bgit\s*=\s*"([^"]+)"/.exec(fields)?.[1];
  const rev = /\brev\s*=\s*"([^"]+)"/.exec(fields)?.[1];

  if (version && !git && !rev) {
    if (registry && registry !== 'crates-io') {
      throw new Error(`\`${CRATE}\` custom registry는 dioxus-cli source로 지원하지 않는다: ${registry}`);
    }
    if (!version.startsWith('=')) {
      throw new Error(
        `\`${CRATE}\` 가 정확 핀이 아니다(${version}). dx 와 맞출 수 없으므로 정확 version 또는 git rev로 고정해야 한다.`,
      );
    }
    return { kind: 'registry', version: version.slice(1) };
  }
  if (!version && git && rev && /^[0-9a-f]{40}$/i.test(rev)) {
    return { kind: 'git', git, rev: rev.toLowerCase() };
  }
  throw new Error(
    `\`${CRATE}\` 는 정확 registry version 또는 40자리 git rev 하나로 고정해야 한다: ${fields.trim()}`,
  );
}

function resolvedPackage(lockfile) {
  const packages = lockfile
    .split(/^\[\[package\]\]$/m)
    .filter(block => /^\s*name = "subsecond"$/m.test(block));
  if (packages.length !== 1) {
    throw new Error(`Cargo.lock 에서 \`${CRATE}\` 를 정확히 한 번 찾지 못했다(${packages.length}건)`);
  }
  const block = packages[0];
  const version = /^version = "([^"]+)"$/m.exec(block)?.[1];
  const source = /^source = "([^"]+)"$/m.exec(block)?.[1];
  if (!version || !source) throw new Error(`Cargo.lock 의 \`${CRATE}\` source를 읽지 못했다`);
  return { version, source };
}

export function dioxusCliSource(root = ROOT) {
  const requested = manifestSource(readFileSync(path.join(root, 'Cargo.toml'), 'utf8'));
  const resolved = resolvedPackage(readFileSync(path.join(root, 'Cargo.lock'), 'utf8'));

  if (requested.kind === 'registry') {
    if (requested.version !== resolved.version || resolved.source !== CRATES_IO_SOURCE) {
      throw new Error(
        `\`${CRATE}\` registry 핀(${requested.version})과 lock(${resolved.version}, ${resolved.source})이 다르다.`,
      );
    }
    return requested;
  }

  const expected = `git+${requested.git}?rev=${requested.rev}#${requested.rev}`;
  if (resolved.source !== expected) {
    throw new Error(
      `\`${CRATE}\` git rev(${requested.rev})과 Cargo.lock source(${resolved.source})가 다르다.`,
    );
  }
  return { ...requested, version: resolved.version };
}

/** 기존 소비자가 version 문자열만 필요할 때의 호환 API. 설치 source 판정에는 쓰지 않는다. */
export function dioxusCliVersion(root = ROOT) {
  return dioxusCliSource(root).version;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.stdout.write(`${JSON.stringify(dioxusCliSource())}\n`);
}
