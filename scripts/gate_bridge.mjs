#!/usr/bin/env node
/**
 * Studio Bridge 게이트 러너 — 자동화·플러그인·브리지 계약의 전 검증을 한 번에 돌린다.
 *
 * 이 기능의 검증은 여섯 군데에 흩어져 있다(타입, studio 단위, 패키지 단위, e2e 5종, 번들,
 * hwpctrl gate). 흩어진 채로 두면 "무엇까지 돌렸는지" 가 매번 사람 기억에 남고, 기억은 회귀를
 * 놓친다. 이 러너가 그 목록의 단일 권위다.
 *
 * 핵심은 **dev 서버 수명 관리**다 — e2e 는 vite 가 떠 있어야 하고, 손으로 띄우면 껐다 켜는 것을
 * 잊어 낡은 번들에 대고 통과하는 일이 생긴다. 여기서는 러너가 띄우고 반드시 내린다.
 *
 * 사용:
 *   node scripts/gate_bridge.mjs                 # 전부
 *   node scripts/gate_bridge.mjs --no-e2e        # 브라우저 없이 (타입·단위·번들)
 *   node scripts/gate_bridge.mjs --only=e2e      # e2e 만
 *   node scripts/gate_bridge.mjs --no-hwpctrl-gate
 *   CHROME_EXTRA_ARGS='--js-flags=--expose-gc' node scripts/gate_bridge.mjs
 *     └ 성능 게이트의 힙 측정을 정밀하게 한다(GC 강제). 없으면 느슨한 상한으로 판정한다.
 *
 * 종료 코드: 0 통과 / 1 실패. 계획: mydocs/plans/rhwp_studio_hwpctrl_plugin.md
 */
import { spawn } from 'node:child_process';
import { readdirSync } from 'node:fs';
import { createConnection } from 'node:net';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const STUDIO = resolve(ROOT, 'rhwp-studio');
const HWPCTRL = resolve(ROOT, 'npm/hwpctrl-ocx');

const DEV_PORT = Number(process.env.GATE_VITE_PORT || 7700);
const DEV_URL = `http://localhost:${DEV_PORT}`;

const args = process.argv.slice(2);
const has = (flag) => args.includes(flag);
const only = args.find((a) => a.startsWith('--only='))?.slice('--only='.length) ?? null;

/** e2e 목록 — MANIFEST 의 `npm gate:bridge` 배선 열과 짝이 맞아야 한다. */
const E2E_SUITES = [
  'automation-commands',
  'plugin-lifecycle',
  'hwpctrl-plugin',
  'bridge-lifecycle',
  'bridge-perf',
];

// ── 실행 유틸 ────────────────────────────────────────────────

function run(cmd, cmdArgs, options = {}) {
  return new Promise((done) => {
    const child = spawn(cmd, cmdArgs, {
      cwd: options.cwd ?? ROOT,
      env: { ...process.env, ...options.env },
      shell: false,
    });
    let out = '';
    child.stdout.on('data', (d) => { out += d; });
    child.stderr.on('data', (d) => { out += d; });
    child.on('close', (code) => done({ code, out }));
    child.on('error', (error) => done({ code: -1, out: `${out}\n${error.message}` }));
  });
}

const npx = (cmdArgs, options) => run('npx', cmdArgs, options);
const npm = (cmdArgs, options) => run('npm', cmdArgs, options);

function portOpen(port) {
  return new Promise((done) => {
    const socket = createConnection({ port, host: '127.0.0.1' });
    socket.on('connect', () => { socket.destroy(); done(true); });
    socket.on('error', () => done(false));
    setTimeout(() => { socket.destroy(); done(false); }, 800);
  });
}

async function waitForPort(port, timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await portOpen(port)) return true;
    await new Promise((r) => setTimeout(r, 300));
  }
  return false;
}

/** node --test 산출에서 pass/fail 을 읽는다. */
function parseNodeTest(out) {
  // Node 22 이하는 '# pass', Node 24는 'ℹ pass' 요약을 출력한다.
  const pass = Number(/^(?:#|\u2139) pass (\d+)$/m.exec(out)?.[1] ?? -1);
  const fail = Number(/^(?:#|\u2139) fail (\d+)$/m.exec(out)?.[1] ?? -1);
  return { pass, fail };
}

/** e2e 산출에서 PASS/FAIL 수를 센다. 하니스가 assert 마다 한 줄씩 찍는다. */
function parseE2e(out) {
  const pass = (out.match(/^\s*PASS:/gm) ?? []).length;
  const fail = (out.match(/^\s*FAIL:/gm) ?? []).length
    + (out.match(/^테스트 오류:/gm) ?? []).length;
  return { pass, fail };
}

// ── 단계 정의 ────────────────────────────────────────────────

const steps = [];
const wants = (group) => (only ? only === group : true);

if (wants('types')) {
  steps.push({
    group: 'types',
    name: 'tsc (main, ci-unit)',
    async run() {
      const main = await npx(['tsc', '--noEmit', '-p', 'tsconfig.json'], { cwd: STUDIO });
      const ci = await npx(['tsc', '--noEmit', '-p', 'tsconfig.ci-unit.json'], { cwd: STUDIO });
      const ok = main.code === 0 && ci.code === 0;
      return { ok, detail: ok ? 'OK' : (main.out || ci.out).split('\n').slice(0, 3).join(' / ') };
    },
  });
}

if (wants('unit')) {
  steps.push({
    group: 'unit',
    name: 'studio 단위',
    async run() {
      const r = await npm(['test'], { cwd: STUDIO });
      const { pass, fail } = parseNodeTest(r.out);
      return { ok: r.code === 0 && fail === 0, detail: `${pass} pass / ${fail} fail` };
    },
  });
  steps.push({
    group: 'unit',
    name: 'hwpctrl 패키지 단위',
    async run() {
      // `node --test test/` 는 이 버전에서 디렉터리를 모듈로 해석해 실패한다. 파일을 직접 센다 —
      // 그래야 새 테스트 파일이 조용히 빠지지 않는다.
      const files = readdirSync(resolve(HWPCTRL, 'test'))
        .filter((name) => name.endsWith('.test.mjs'))
        .map((name) => `test/${name}`);
      const r = await run('node', ['--test', ...files], { cwd: HWPCTRL });
      const { pass, fail } = parseNodeTest(r.out);
      return {
        ok: r.code === 0 && fail === 0,
        detail: `${pass} pass / ${fail} fail (파일 ${files.length}종)`,
        out: fail > 0 ? r.out.slice(-1500) : undefined,
      };
    },
  });
}

if (wants('e2e') && !has('--no-e2e')) {
  for (const suite of E2E_SUITES) {
    steps.push({
      group: 'e2e',
      needsDevServer: true,
      name: `e2e ${suite}`,
      async run() {
        const r = await run('node', [`e2e/${suite}.test.mjs`, '--mode=headless'], {
          cwd: STUDIO,
          env: { VITE_URL: DEV_URL },
        });
        const { pass, fail } = parseE2e(r.out);
        return {
          ok: fail === 0 && pass > 0,
          detail: `${pass} PASS / ${fail} FAIL`,
          out: fail > 0 ? r.out : undefined,
        };
      },
    });
  }
}

if (wants('build')) {
  steps.push({
    group: 'build',
    name: 'build + 플러그인 청크 분리',
    async run() {
      const r = await npm(['run', 'build'], { cwd: STUDIO });
      if (r.code !== 0) return { ok: false, detail: '빌드 실패', out: r.out.slice(-1500) };
      // 플러그인이 엔트리에 섞이면 "안 올려도 로드되지 않는다" 는 계약이 깨진다.
      const chunk = /studio-plugin-[\w-]+\.js\s+([\d.]+\s*kB)/.exec(r.out);
      return chunk
        ? { ok: true, detail: `studio-plugin 청크 ${chunk[1].replace(/\s+/g, '')}` }
        : { ok: false, detail: 'studio-plugin 청크가 없다 — 엔트리에 섞였을 수 있다' };
    },
  });
}

if (wants('hwpctrl-gate') && !has('--no-hwpctrl-gate')) {
  steps.push({
    group: 'hwpctrl-gate',
    name: 'hwpctrl standalone gate',
    async run() {
      const r = await npm(['--prefix', 'npm/hwpctrl-ocx', 'run', 'gate']);
      const problems = /실행 문제: (\{.*\})/.exec(r.out)?.[1];
      const okScenarios = (r.out.match(/: OK$/gm) ?? []).length;
      return {
        ok: r.code === 0 && !problems,
        detail: problems ? `실행 문제 있음 — ${problems.slice(0, 120)}` : `시나리오 ${okScenarios}건 OK`,
      };
    },
  });
}

// ── 실행 ─────────────────────────────────────────────────────

async function main() {
  const started = Date.now();
  const needsDev = steps.some((s) => s.needsDevServer);
  let dev = null;
  let devWasAlreadyUp = false;

  console.log('\n  Studio Bridge 게이트\n');

  if (needsDev) {
    devWasAlreadyUp = await portOpen(DEV_PORT);
    if (devWasAlreadyUp) {
      // 남의 서버를 내리지 않는다. 다만 낡은 번들일 수 있으므로 알린다.
      console.log(`  [dev] ${DEV_URL} 이미 떠 있음 — 그대로 사용 (러너가 내리지 않는다)\n`);
    } else {
      dev = spawn('npm', ['run', 'dev'], { cwd: STUDIO, env: process.env, detached: true });
      dev.stdout?.on('data', () => {});
      dev.stderr?.on('data', () => {});
      if (!await waitForPort(DEV_PORT)) {
        console.error(`  [dev] ${DEV_URL} 기동 실패`);
        try { process.kill(-dev.pid); } catch { /* noop */ }
        process.exit(1);
      }
      console.log(`  [dev] ${DEV_URL} 기동\n`);
    }
  }

  const results = [];
  try {
    for (const step of steps) {
      const at = Date.now();
      const r = await step.run();
      const secs = ((Date.now() - at) / 1000).toFixed(1);
      results.push({ ...step, ...r, secs });
      const mark = r.ok ? 'ok  ' : 'FAIL';
      console.log(`  ${mark} ${step.name.padEnd(34)} ${r.detail}  (${secs}s)`);
      if (!r.ok && r.out) console.log(`\n${r.out.split('\n').slice(-25).join('\n')}\n`);
    }
  } finally {
    if (dev) {
      try { process.kill(-dev.pid); } catch { /* noop */ }
      console.log('\n  [dev] 종료');
    }
  }

  const failed = results.filter((r) => !r.ok);
  const total = ((Date.now() - started) / 1000).toFixed(0);
  console.log(
    `\n  결과: ${failed.length === 0 ? '통과' : `실패 ${failed.length}건`} `
      + `(${results.length}단계, ${total}s)`,
  );
  if (failed.length) {
    console.log(`  실패한 단계: ${failed.map((r) => r.name).join(', ')}`);
  }
  if (devWasAlreadyUp) {
    console.log('  주: 이미 떠 있던 dev 서버를 썼다. 낡은 번들이 의심되면 내리고 다시 돌린다.');
  }
  process.exit(failed.length ? 1 : 0);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
