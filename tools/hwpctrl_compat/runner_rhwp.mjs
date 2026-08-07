/**
 * rhwp 측 러너 — 같은 시나리오를 rhwp WASM 위에서 실행한다 (P0).
 *
 * `runner_ocx.py` 와 **같은 모양의 returns.json** 을 낸다. 그래야 `compare.py` 가
 * 두 산출물을 그대로 대조할 수 있다.
 *
 * ## 쓰임
 *
 *     node tools/hwpctrl_compat/runner_rhwp.mjs scenarios/doc-basic.json \
 *          --out output/poc/hwpctrl/rhwp --impl legacy
 *
 * ## --impl
 *
 * - `legacy`  — 기존 `rhwp-studio/src/hwpctl/` 층. **P0 자체 검증 전용**이다.
 *               하니스가 "이미 아는 차이"를 실제로 잡아내는지 보는 데 쓴다.
 * - `<경로>`  — 신규 패키지의 엔트리(ESM). P1 부터 이쪽을 쓴다.
 *               `createHwpCtrl({ wasmModule })` 를 export 하면 된다.
 *
 * ## 대조가 성립하려면
 *
 * 반환값 정규화 규칙이 오라클 러너와 **같아야 한다**. 객체는 값을 그대로 싣고, 클래스
 * 인스턴스는 `{__type: 이름}` 으로 줄인다(`normalize`). 규칙을 한쪽만 바꾸면 diff 가
 * 구현 차이가 아니라 러너 차이가 된다.
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(HERE, '..', '..');

function parseArgs(argv) {
  const out = { impl: 'legacy' };
  const rest = [];
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === '--out') { out.out = argv[i + 1]; i += 1; }
    else if (argv[i] === '--impl') { out.impl = argv[i + 1]; i += 1; }
    else rest.push(argv[i]);
  }
  out.scenario = rest[0];
  return out;
}

/** 오라클 러너의 `normalize` 와 같은 규칙. 어긋나면 러너 차이가 diff 로 새어 나온다. */
function normalize(value) {
  if (value === undefined) return null;
  if (value === null) return null;
  const t = typeof value;
  if (t === 'boolean' || t === 'number' || t === 'string') return value;
  if (Array.isArray(value)) return value.map(normalize);
  if (value instanceof Uint8Array) return { __type: 'bytes', length: value.length };
  if (t === 'object') {
    if (value.constructor && value.constructor !== Object) {
      return { __type: value.constructor.name };
    }
    const out = {};
    for (const [k, v] of Object.entries(value)) out[k] = normalize(v);
    return out;
  }
  return { __type: t };
}

/**
 * 기존 studio hwpctl 층(TypeScript)을 Node 가 읽을 수 있게 CommonJS 로 옮긴다.
 *
 * Node 의 타입 스트리핑만으로는 안 된다 — 이 층은 확장자 없는 상대 import 를 쓰는데
 * ESM 은 그것을 해석하지 않는다. CJS 로 내리면 해석된다.
 */
function buildLegacyImpl() {
  const outDir = join(REPO, 'output', 'poc', 'hwpctrl', 'legacy-cjs');
  // `.bin/tsc.cmd` 는 쓰지 않는다 — Node 20+ 는 shell 없이 `.cmd` 실행을 막는다.
  const tsc = join(REPO, 'rhwp-studio', 'node_modules', 'typescript', 'bin', 'tsc');
  if (!existsSync(tsc)) {
    throw new Error(`tsc 없음: ${tsc} — rhwp-studio 에서 npm install 을 먼저 하라`);
  }
  const entry = join(outDir, 'index.js');
  try {
    execFileSync(
      process.execPath,
      [
        tsc,
        join(REPO, 'rhwp-studio', 'src', 'hwpctl', 'index.ts'),
        '--module', 'commonjs',
        '--target', 'es2022',
        '--skipLibCheck',
        '--outDir', outDir,
      ],
      { stdio: 'pipe' },
    );
  } catch (e) {
    // 이 층은 studio 경로 별칭(`@wasm/rhwp.js`)을 **지연 import** 안에서 쓴다. tsc 는 그것을
    // 해석하지 못해 TS2307 로 실패하지만 **JS 는 정상 방출한다**. 우리는 그 지연 경로를
    // 호출하지 않으므로(문서 객체를 직접 넘긴다) 방출물이 있으면 계속 간다.
    const emitted = existsSync(entry);
    const message = (e.stdout ?? Buffer.alloc(0)).toString('utf-8');
    if (!emitted) throw new Error(`tsc 실패:\n${message}`);
    const unexpected = message
      .split('\n')
      .filter((l) => l.includes('error TS') && !l.includes("'@wasm/rhwp.js'"));
    if (unexpected.length) throw new Error(`tsc 실패(예상 밖 오류):\n${unexpected.join('\n')}`);
  }
  return entry;
}

async function loadWasm() {
  const mod = await import(pathToFileURL(join(REPO, 'pkg', 'rhwp.js')).href);
  await mod.default({ module_or_path: readFileSync(join(REPO, 'pkg', 'rhwp_bg.wasm')) });
  return mod;
}

async function loadImpl(impl, wasm) {
  if (impl === 'legacy') {
    const entry = buildLegacyImpl();
    const { createRequire } = await import('node:module');
    const require = createRequire(import.meta.url);
    const { HwpCtrl } = require(entry);
    return {
      name: 'legacy-studio-hwpctl',
      make: (doc) => new HwpCtrl(doc),
    };
  }
  const mod = await import(pathToFileURL(resolve(REPO, impl)).href);
  return {
    name: impl,
    make: (doc) => mod.createHwpCtrl({ wasmModule: doc }),
  };
}

/** 메서드면 호출하고, 속성이면 읽는다 — 오라클 러너와 같은 규칙. */
function callOne(ctrl, name, args) {
  const value = ctrl[name];
  if (typeof value === 'function') return normalize(value.apply(ctrl, args));
  if (name in ctrl) return normalize(value);
  // 대소문자만 다른 별칭(예: AddEventListener ↔ addEventListener)도 규격 위반이다.
  // 조용히 맞춰 주면 L1 차이가 가려진다 — 없는 것은 없는 것으로 기록한다.
  const err = new Error(`구현에 없는 API: ${name}`);
  err.missing = true;
  throw err;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (!args.scenario || !args.out) {
    console.error('사용법: runner_rhwp.mjs <시나리오.json> --out <디렉터리> [--impl legacy|<경로>]');
    process.exit(2);
  }
  const scenario = JSON.parse(readFileSync(resolve(args.scenario), 'utf-8'));
  const outDir = resolve(args.out);
  mkdirSync(outDir, { recursive: true });

  const wasm = await loadWasm();
  const impl = await loadImpl(args.impl, wasm);

  const result = {
    scenario: scenario.id,
    runner: 'rhwp',
    impl: impl.name,
    calls: [],
    saved: null,
    fatal: null,
  };

  try {
    let doc;
    if (scenario.open) {
      const bytes = readFileSync(join(REPO, scenario.open));
      doc = new wasm.HwpDocument(bytes);
      result.calls.push({ call: 'Open', args: [scenario.open], value: true });
    } else {
      doc = wasm.HwpDocument.createEmpty();
    }
    const ctrl = impl.make(doc);

    for (const [name, callArgs = []] of scenario.calls ?? []) {
      const record = { call: name, args: callArgs };
      try {
        record.value = callOne(ctrl, name, callArgs);
      } catch (e) {
        record.error = e.missing ? `MissingApi: ${name}` : `${e.constructor.name}: ${e.message}`;
      }
      result.calls.push(record);
    }

    if (scenario.saveAs) {
      const bytes = ctrl.getWasmDoc ? ctrl.getWasmDoc().exportHwp() : doc.exportHwp();
      const dst = join(outDir, scenario.saveAs);
      writeFileSync(dst, bytes);
      result.saved = { path: dst, ok: true };
    }
  } catch (e) {
    result.fatal = `${e.constructor.name}: ${e.message}`;
  }

  const dst = join(outDir, `${scenario.id}.returns.json`);
  writeFileSync(dst, `${JSON.stringify(result, null, 2)}\n`, 'utf-8');
  console.log(`${scenario.id}: 호출 ${result.calls.length}건 → ${dst}`);
  process.exit(result.fatal ? 1 : 0);
}

main();
