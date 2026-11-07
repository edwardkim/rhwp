#!/usr/bin/env node
// Root integration suites also depend on host-only proptest/wait-timeout.
// This harness compiles the same committed browser case without those dev deps.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = realpathSync(path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'));
const lock = readFileSync(path.join(root, 'Cargo.lock'), 'utf8');

function packages(text) {
  return text.split(/^\[\[package\]\]\s*$/m).slice(1).map((block) => ({
    name: /^name = "([^"]+)"$/m.exec(block)?.[1],
    version: /^version = "([^"]+)"$/m.exec(block)?.[1],
    source: /^source = "([^"]+)"$/m.exec(block)?.[1] ?? '',
  }));
}

const locked = packages(lock);
function version(name) {
  const versions = [...new Set(locked.filter((p) => p.name === name).map((p) => p.version))];
  if (versions.length !== 1) throw new Error(`Expected one locked version of ${name}`);
  return versions[0];
}

const channel = /^channel\s*=\s*"([^"]+)"/m.exec(
  readFileSync(path.join(root, 'rust-toolchain.toml'), 'utf8'),
)?.[1];
if (!channel) throw new Error('Missing repository Rust toolchain');
const env = { ...process.env, RUSTUP_TOOLCHAIN: channel };
const runner = env.CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER ?? 'wasm-bindgen-test-runner';
const runnerVersion = spawnSync(runner, ['--version'], { env, encoding: 'utf8' });
if (runnerVersion.error || runnerVersion.status !== 0
  || !runnerVersion.stdout.trim().endsWith(` ${version('wasm-bindgen')}`)) {
  throw new Error(`Use wasm-bindgen-test-runner ${version('wasm-bindgen')} from the matching wasm-pack cache`);
}

const sourceIdentity = createHash('sha256').update(root).digest('hex').slice(0, 12);
const target = path.resolve(env.CARGO_TARGET_DIR ?? path.join(root, 'target'));
const harness = path.join(target, `web-canvas-outline-harness-${sourceIdentity}`);
const name = `rhwp-web-canvas-outline-tests-${sourceIdentity}`;
const source = path.join(root, 'tests/cases/issue_536_web_canvas_outline.rs');
const manifest = `[package]
name = ${JSON.stringify(name)}
version = "0.0.0"
edition = "2021"
autotests = false

[workspace]

[[test]]
name = "canvas2d_outline"
path = ${JSON.stringify(source)}

[dependencies]
rhwp = { path = ${JSON.stringify(root)} }
image = { version = "=${version('image')}", default-features = false, features = ["png"] }
wasm-bindgen = "=${version('wasm-bindgen')}"
wasm-bindgen-test = "=${version('wasm-bindgen-test')}"
web-sys = "=${version('web-sys')}"
serde_json = "=${version('serde_json')}"
`;

mkdirSync(harness, { recursive: true });
writeFileSync(path.join(harness, 'Cargo.toml'), manifest);
writeFileSync(path.join(harness, 'Cargo.lock'), lock);
const manifestPath = path.join(harness, 'Cargo.toml');
const metadata = spawnSync('cargo', [
  'metadata', '--offline', '--format-version', '1', '--filter-platform',
  'wasm32-unknown-unknown', '--manifest-path', manifestPath,
], { cwd: root, env, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
if (metadata.error || metadata.status !== 0) {
  process.stderr.write(metadata.stderr ?? '');
  throw metadata.error ?? new Error(`Harness metadata failed: ${metadata.status}`);
}
const identity = (p) => `${p.name}\0${p.version}\0${p.source}`;
const originals = new Set(locked.map(identity));
const resolved = packages(readFileSync(path.join(harness, 'Cargo.lock'), 'utf8'));
for (const dependency of resolved) {
  if (dependency.name !== name && !originals.has(identity(dependency))) {
    throw new Error(`Harness dependency differs from root lock: ${dependency.name} ${dependency.version}`);
  }
}
console.log(`Canvas2D browser source: ${source}`);
console.log(`All ${resolved.length - 1} dependencies retain their root lock identities; Rust ${channel}`);
const result = spawnSync('cargo', [
  'test', '--locked', '--offline', '--manifest-path', manifestPath,
  '--target', 'wasm32-unknown-unknown', '--test', 'canvas2d_outline',
], { cwd: root, env: { ...env, CARGO_TARGET_DIR: target }, stdio: 'inherit' });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
