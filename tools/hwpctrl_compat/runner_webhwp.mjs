/**
 * 기안기 러너 — 같은 시나리오를 **실물 웹한글 기안기**(헤드리스 브라우저) 위에서 실행한다.
 *
 * 오라클 이원화(계획서 §6.3.3·§9-6 — PR #4470)의 정본 축이다. `runner_ocx.py`(COM)·`runner_rhwp.mjs`
 * (rhwp WASM)와 **같은 모양의 returns.json** 을 내서 `compare.py`/`compare3.py` 가 세 산출물을
 * 그대로 대조할 수 있게 한다.
 *
 * ## 쓰임
 *
 *     node tools/hwpctrl_compat/runner_webhwp.mjs scenarios/doc-basic.json \
 *          --out output/poc/hwpctrl/webhwp [--url https://.../webhwpctrl/] [--chrome <경로>]
 *
 * ## 지켜야 하는 것
 *
 * - **저빈도 수동 실행 전용이다.** 기본 URL 은 한컴 공개 데모다 — CI 에 물리거나 반복 폭주로
 *   몰지 않는다(계획서 §6.3.3 — PR #4470). 자가 호스팅 웹한글 서버가 있으면 `--url` 로 그쪽을 쓴다.
 * - **버전 스탬프 없는 결과는 정답지 자격이 없다.** 이 러너는 매 실행 `oracle` 에
 *   URL·측정 시각·`HwpCtrl.Version` 을 남기고, `compare3.py` 는 스탬프 없는 산출물을 거부한다.
 * - 시나리오당 **브라우저 프로세스 하나**다. COM 의 "문서 하나당 프로세스 하나"와 같은 이유다 —
 *   상태가 새면 정답지를 못 믿는다.
 *
 * ## 실물과 하니스의 계약 차이 (r1 실측)
 *
 * - `Open` 은 브라우저 `File` + **완료 콜백**을 받는다(데모 페이지 실측:
 *   `HwpCtrl.Open(file, "", "include-format:hwpx;hwpjson20;", cb, userData)`). 이 러너는 표본
 *   바이트로 페이지 안에서 `File` 을 만들어 넘기고 콜백 결과를 기다린다.
 * - `SaveAs` 는 브라우저 다운로드 경로라 산출물 검증(L3)을 하지 않는다 — `saved: null` 로 남겨
 *   `compare.py` 가 L3 를 건너뛰게 한다. 파일 갈래는 이 축의 대조 대상이 아니다.
 * - `$path` 인자는 `posix` 갈래로 푼다. 실물 컨트롤에는 로컬 파일계가 없으므로 그 호출이
 *   어떻게 죽는지 자체가 관측이다 — 지어내지 않고 기록한다.
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync, readdirSync } from 'node:fs';
import { dirname, join, resolve, basename } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';
import os from 'node:os';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(HERE, '..', '..');
const DEFAULT_URL = 'https://webhwpctrl.cloud.hancom.com/webhwpctrl/';

// puppeteer-core 는 studio 가 이미 들고 있다 — 하니스가 의존성을 따로 늘리지 않는다.
const require = createRequire(join(REPO, 'rhwp-studio', 'package.json'));

function parseArgs(argv) {
  const out = { url: DEFAULT_URL, timeoutMs: 30000 };
  const rest = [];
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === '--out') { out.out = argv[i + 1]; i += 1; }
    else if (argv[i] === '--url') { out.url = argv[i + 1]; i += 1; }
    else if (argv[i] === '--chrome') { out.chrome = argv[i + 1]; i += 1; }
    else if (argv[i] === '--timeout-ms') { out.timeoutMs = Number(argv[i + 1]); i += 1; }
    else rest.push(argv[i]);
  }
  out.scenario = rest[0];
  return out;
}

/** studio e2e 의 해석 순서와 같다: 환경변수 → 시스템 → puppeteer 캐시. */
function resolveChromePath(explicit) {
  if (explicit && existsSync(explicit)) return explicit;
  const envPath = process.env.CHROME_PATH || process.env.PUPPETEER_EXECUTABLE_PATH;
  if (envPath && existsSync(envPath)) return envPath;
  const system = [
    '/usr/bin/google-chrome-stable', '/usr/bin/google-chrome',
    '/usr/bin/chromium-browser', '/usr/bin/chromium', '/snap/bin/chromium',
  ].find((c) => existsSync(c));
  if (system) return system;
  const cacheRoot = join(os.homedir(), '.cache', 'puppeteer');
  if (!existsSync(cacheRoot)) return '';
  const stack = [cacheRoot];
  const found = [];
  while (stack.length) {
    const current = stack.pop();
    let entries;
    try { entries = readdirSync(current, { withFileTypes: true }); } catch { continue; }
    for (const entry of entries) {
      const candidate = join(current, entry.name);
      if (entry.isDirectory()) stack.push(candidate);
      else if (entry.name === 'chrome') found.push(candidate);
    }
  }
  return found.sort().reverse()[0] ?? '';
}

/**
 * 페이지 안에서 도는 실행기 — `runner_rhwp.mjs` 의 `normalize`/`splitCall`/`callOne` 과
 * **같은 규칙**이다. 규칙을 한쪽만 바꾸면 diff 가 계약 차이가 아니라 러너 차이가 된다.
 * 문자열로 주입하는 이유: page.evaluate 는 함수 직렬화라 모듈 import 를 끌고 갈 수 없다.
 */
const PAGE_RUNTIME = String.raw`
(() => {
  function normalize(value) {
    if (value === undefined || value === null) return null;
    const t = typeof value;
    if (t === 'boolean' || t === 'number' || t === 'string') return value;
    if (Array.isArray(value)) return value.map(normalize);
    if (value instanceof Uint8Array) return { __type: 'bytes', length: value.length };
    if (t === 'object') {
      // 실물 컨트롤은 iframe(다른 realm)에 산다 — 평범한 객체도 top 의 Object 와 정체성이
      // 달라 \`constructor !== Object\` 로는 클래스 인스턴스와 못 가른다. 이름으로 가른다.
      // runner_rhwp 는 같은 realm 이라 정체성 비교로 충분했다 — 규칙의 뜻은 같다.
      const ctorName = value.constructor ? value.constructor.name : 'Object';
      if (ctorName !== 'Object') {
        return { __type: ctorName };
      }
      const out = {};
      for (const [k, v] of Object.entries(value)) out[k] = normalize(v);
      return out;
    }
    return { __type: t };
  }
  const CALL_WITH_ARGS = /^([A-Za-z_]\w*)\((.*)\)$/;
  function splitCall(part) {
    const m = CALL_WITH_ARGS.exec(part);
    if (!m) return [part, []];
    const inner = m[2].trim();
    return [m[1], inner ? JSON.parse('[' + inner + ']') : []];
  }
  function resolvePath(ctrl, path) {
    let obj = ctrl;
    for (const raw of path.split('.')) {
      const [part, callArgs] = splitCall(raw);
      const next = obj[part];
      obj = typeof next === 'function' ? next.apply(obj, callArgs) : next;
    }
    return obj;
  }
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
        const err = new Error('구현에 없는 API: ' + name);
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
    const err = new Error('구현에 없는 API: ' + name);
    err.missing = true;
    throw err;
  }
  window.__harness = { normalize, callOne };
})();
`;

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (!args.scenario || !args.out) {
    console.error('사용법: runner_webhwp.mjs <시나리오.json> --out <디렉터리> [--url <데모>] [--chrome <경로>]');
    process.exit(2);
  }
  const scenario = JSON.parse(readFileSync(resolve(args.scenario), 'utf-8'));
  const outDir = resolve(args.out);
  mkdirSync(outDir, { recursive: true });

  const chromePath = resolveChromePath(args.chrome);
  if (!chromePath) {
    console.error('Chrome 을 찾지 못했다 — CHROME_PATH 를 지정하라');
    process.exit(2);
  }

  const result = {
    scenario: scenario.id,
    runner: 'webhwp',
    oracle: null,
    calls: [],
    saved: null,
    fatal: null,
  };

  const puppeteer = require('puppeteer-core');
  const browser = await puppeteer.launch({
    headless: true,
    executablePath: chromePath,
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
  });
  try {
    const page = await browser.newPage();
    page.setDefaultTimeout(args.timeoutMs);
    await page.goto(args.url, { waitUntil: 'networkidle2', timeout: args.timeoutMs });
    await page.waitForFunction(() => typeof window.HwpCtrl !== 'undefined', {
      timeout: args.timeoutMs,
    });
    await page.evaluate(PAGE_RUNTIME);

    // 버전 스탬프 — 없으면 compare3 가 이 산출물을 거부한다. 지어내지 않는다.
    result.oracle = await page.evaluate(() => {
      let version = null;
      try { version = window.HwpCtrl.Version ?? null; } catch { version = null; }
      return { kind: 'webhwp', version, userAgent: navigator.userAgent };
    });
    result.oracle.url = args.url;
    result.oracle.measuredAt = new Date().toISOString();

    if (scenario.open) {
      const bytes = readFileSync(join(REPO, scenario.open));
      const opened = await page.evaluate(
        ({ b64, name, timeoutMs }) =>
          new Promise((resolveOpen) => {
            const bin = atob(b64);
            const buf = new Uint8Array(bin.length);
            for (let i = 0; i < bin.length; i += 1) buf[i] = bin.charCodeAt(i);
            const file = new File([buf], name);
            const timer = setTimeout(
              () => resolveOpen({ error: `TimeoutError: Open 콜백이 ${timeoutMs}ms 안에 안 왔다` }),
              timeoutMs,
            );
            try {
              // 데모 페이지 실측 계약: Open(File, format, arg, callback, userData)
              window.HwpCtrl.Open(file, '', '', (res) => {
                clearTimeout(timer);
                resolveOpen({ value: window.__harness.normalize(res) });
              });
            } catch (e) {
              clearTimeout(timer);
              resolveOpen({ error: `${e.constructor.name}: ${e.message}` });
            }
          }),
        { b64: bytes.toString('base64'), name: basename(scenario.open), timeoutMs: args.timeoutMs },
      );
      const record = { call: 'Open', args: [scenario.open] };
      if (opened.error) record.error = opened.error;
      else record.value = opened.value;
      result.calls.push(record);
    }

    for (const [name, callArgs = []] of scenario.calls ?? []) {
      // 경로 인자는 posix 갈래로 푼다 — 실물에는 로컬 파일계가 없으니 그 호출이 어떻게
      // 답하는지 자체가 관측이다.
      const resolved = callArgs.map((a) =>
        a && typeof a === 'object' && !Array.isArray(a) && '$path' in a
          ? String(
              (scenario.paths?.[a.$path]?.posix ?? '')
                .replaceAll('{repo}', REPO)
                .replaceAll('{out}', outDir),
            )
          : a,
      );
      const record = { call: name, args: resolved };
      try {
        const r = await page.evaluate(
          ({ n, a }) => {
            try {
              return { value: window.__harness.callOne(window.HwpCtrl, n, a) };
            } catch (e) {
              return { error: e.missing ? `MissingApi: ${n}` : `${e.constructor.name}: ${e.message}` };
            }
          },
          { n: name, a: resolved },
        );
        if (r.error) record.error = r.error;
        else record.value = r.value;
      } catch (e) {
        record.error = `${e.constructor.name}: ${e.message}`;
      }
      result.calls.push(record);
    }
    // scenario.saveAs 는 태우지 않는다 — 브라우저 다운로드 경로라 L3 대조 대상이 아니다.
  } catch (e) {
    result.fatal = `${e.constructor.name}: ${e.message}`;
  } finally {
    await browser.close();
  }

  const dst = join(outDir, `${scenario.id}.returns.json`);
  writeFileSync(dst, `${JSON.stringify(result, null, 2)}\n`, 'utf-8');
  console.log(`${scenario.id}: 호출 ${result.calls.length}건 → ${dst}`);
  process.exit(result.fatal ? 1 : 0);
}

main();
