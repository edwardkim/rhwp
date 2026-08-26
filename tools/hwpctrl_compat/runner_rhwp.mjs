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
 *               `createHwpCtrl({ wasm, onSave, onReadFile })` 를 export 하면 된다.
 *
 * `onReadFile` 은 규격이 **경로**를 받는 API(`InsertPicture`)를 위한 호스트 고리다. 바탕화면
 * 컨트롤은 경로가 곧 파일이지만 이 층은 브라우저에서도 돌아 스스로 못 연다 — 하니스는 node 의
 * 파일 읽기를 그대로 준다.
 *
 * ## 대조가 성립하려면
 *
 * 반환값 정규화 규칙이 오라클 러너와 **같아야 한다**. 객체는 값을 그대로 싣고, 클래스
 * 인스턴스는 `{__type: 이름}` 으로 줄인다(`normalize`). 규칙을 한쪽만 바꾸면 diff 가
 * 구현 차이가 아니라 러너 차이가 된다.
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { dirname, isAbsolute, join, resolve } from 'node:path';
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
    // 신규 패키지는 **자기 손으로** 문서를 연다(규격의 `Open`). 하니스가 문서를 만들어
    // 넘겨 주면 그 API 가 대조에서 빠져 "구현했다"고 착각하게 된다.
    ownsOpen: true,
    make: ({ wasm, onSave, onReadFile, onCreatePageImage }) =>
      mod.createHwpCtrl({ wasm, onSave, onReadFile, onCreatePageImage }),
  };
}

/**
 * 메서드면 호출하고, 속성이면 읽는다 — 오라클 러너와 같은 규칙.
 *
 * 이름에 점을 찍으면 객체를 타고 들어간다(`CharShape.Item`). 서식은 ParameterSet 객체로
 * 오므로 점 표기 없이는 `{__type: …}` 만 대조하게 된다.
 */
const CALL_WITH_ARGS = /^([A-Za-z_]\w*)\((.*)\)$/;

/**
 * 점 표기 한 마디를 `[이름, 인자들]` 로 가른다.
 *
 * 중간 마디가 인자를 받는 메서드일 때 쓴다 — `HeadCtrl.GetAnchorPos(0).Item` 처럼.
 * 인자는 JSON 으로 읽는다. 파이썬 러너도 같은 규약이라 양쪽이 같은 호출을 한다.
 */
function splitCall(part) {
  const m = CALL_WITH_ARGS.exec(part);
  if (!m) return [part, []];
  const inner = m[2].trim();
  return [m[1], inner ? JSON.parse(`[${inner}]`) : []];
}

/** 점 표기 경로를 따라가 그 자리의 값을 준다 — `$obj` 인자를 푸는 데 쓴다. */
function resolvePath(ctrl, path) {
  let obj = ctrl;
  for (const raw of path.split('.')) {
    const [part, callArgs] = splitCall(raw);
    const next = obj[part];
    obj = typeof next === 'function' ? next.apply(obj, callArgs) : next;
  }
  return obj;
}

/**
 * 인자 중 `{"$path": "이름"}` 을 **이 플랫폼의 실제 경로**로 바꾼다 — `scenario_spec.py` 의
 * `resolve_args` 와 같은 규칙이다(규칙을 한쪽만 고치면 하니스 차이가 diff 로 샌다).
 *
 * 시나리오가 Windows 절대 경로를 박아 두면 Linux 에서는 그것이 "못 여는 경로"가 아니라
 * **그냥 그런 이름의 상대 경로**가 된다. `C:\없는폴더xyz\a.bmp` 가 멀쩡히 만들어져 `false`
 * 여야 할 자리가 `true` 가 됐고, 저장소 작업본에 그 이름의 파일까지 남았다(#4274 리뷰).
 */
function resolvePathArgs(args, scenario, outDir) {
  const table = scenario.paths ?? {};
  const key = process.platform === 'win32' ? 'win' : 'posix';
  return args.map((a) => {
    if (!a || typeof a !== 'object' || Array.isArray(a) || !('$path' in a)) return a;
    const variants = table[a.$path];
    if (!variants) throw new Error(`시나리오에 없는 경로 이름입니다: ${a.$path}`);
    if (!(key in variants)) throw new Error(`경로 '${a.$path}' 에 '${key}' 갈래가 없습니다`);
    return String(variants[key]).replaceAll('{repo}', REPO).replaceAll('{out}', outDir);
  });
}

/**
 * 인자 중 `{"$obj": "경로"}` 를 **그 자리의 객체**로 바꾼다.
 *
 * 파라미터셋이나 `Ctrl` 을 인자로 받는 API 를 시나리오가 부를 수 있게 하는 규약이다.
 * 파이썬 러너도 같은 규약이라 양쪽이 같은 것을 넘긴다.
 */
function resolveArgs(ctrl, args) {
  return args.map((a) =>
    a && typeof a === 'object' && !Array.isArray(a) && '$obj' in a ? resolvePath(ctrl, a.$obj) : a,
  );
}

function callOne(ctrl, name, rawArgs) {
  const args = resolveArgs(ctrl, rawArgs);
  const parts = name.split('.');
  let owner = ctrl;
  for (const raw of parts.slice(0, -1)) {
    const [part, midArgs] = splitCall(raw);
    if (owner == null || !(part in owner)) {
      const err = new Error(`구현에 없는 API: ${name}`);
      err.missing = true;
      throw err;
    }
    const next = owner[part];
    owner = typeof next === 'function' ? next.apply(owner, midArgs) : next;
  }
  const last = parts[parts.length - 1];
  const value = owner == null ? undefined : owner[last];
  if (typeof value === 'function') return normalize(value.apply(owner, args));
  if (owner != null && last in owner) return normalize(value);
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
    let ctrl;
    // 저장은 호스트가 받는다 — 규격상 브라우저에서는 다운로드다(v2.4 §2.2).
    let savedBytes = null;
    // 시나리오가 **중간에** `SaveAs` 를 걸면(앞뒤 저장본을 떠 액션의 자취를 재는 L3) 그
    // 자리에서 파일로 흘린다. 이름이 절대경로일 때만 그렇게 한다 — 시나리오 끝의
    // `saveAs` 는 이름만 주고 러너가 `outDir` 아래에 쓴다.
    const onSave = (bytes, fileName) => {
      savedBytes = bytes;
      if (typeof fileName === 'string' && isAbsolute(fileName)) {
        writeFileSync(fileName, bytes);
      }
    };
    // 규격이 **경로**를 받는 API(`InsertPicture`)를 위한 호스트 고리. 오라클은 바탕화면에서
    // 그 경로를 그대로 열므로 이쪽도 같은 경로를 읽어 준다.
    const onReadFile = (path) => new Uint8Array(readFileSync(path));
    // `CreatePageImage` 는 코어가 그린 쪽 SVG 를 호스트에 넘긴다 — 픽셀로 앉히는 것은 호스트
    // 일이다(studio 는 CanvasKit 으로 한다). 하니스에는 래스터라이저가 없으므로 그 SVG 를 그대로
    // 쓴다. **파일 갈래는 대조 대상이 아니다** — 대조하는 것은 반환값이다.
    const onCreatePageImage = ({ path, svg }) => {
      writeFileSync(path, svg, 'utf-8');
      return true;
    };

    if (impl.ownsOpen) {
      ctrl = impl.make({ wasm, onSave, onReadFile, onCreatePageImage });
      if (scenario.open) {
        const bytes = readFileSync(join(REPO, scenario.open));
        const opened = ctrl.Open(new Uint8Array(bytes), '', '');
        result.calls.push({ call: 'Open', args: [scenario.open], value: normalize(opened) });
      }
    } else {
      const doc = scenario.open
        ? new wasm.HwpDocument(readFileSync(join(REPO, scenario.open)))
        : wasm.HwpDocument.createEmpty();
      ctrl = impl.make(doc);
      if (scenario.open) {
        result.calls.push({ call: 'Open', args: [scenario.open], value: true });
      }
    }

    for (const [name, rawArgs = []] of scenario.calls ?? []) {
      const callArgs = resolvePathArgs(rawArgs, scenario, outDir);
      const record = { call: name, args: callArgs };
      try {
        record.value = callOne(ctrl, name, callArgs);
      } catch (e) {
        record.error = e.missing ? `MissingApi: ${name}` : `${e.constructor.name}: ${e.message}`;
      }
      result.calls.push(record);
    }

    if (scenario.saveAs) {
      // 규격의 `SaveAs` 를 실제로 태운다. 하니스가 문서를 직접 내보내면 그 API 가
      // 대조에서 빠진다. 옛 경로(문서를 넘겨받는 impl)만 직접 내보내기로 남긴다.
      let bytes = null;
      if (impl.ownsOpen) {
        ctrl.SaveAs(scenario.saveAs, '', '');
        bytes = savedBytes;
      } else {
        bytes = ctrl.getWasmDoc().exportHwp();
      }
      if (bytes) {
        const dst = join(outDir, scenario.saveAs);
        writeFileSync(dst, bytes);
        result.saved = { path: dst, ok: true };
      } else {
        result.saved = { path: null, ok: false };
      }
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
