import test from 'node:test';
import assert from 'node:assert/strict';
import { CanvasMetricSession, withPortableMetrics } from '../src/core/canvas-metric-session.ts';

function fixture() {
  let active = false;
  const events: string[] = [];
  const requests = [{ key: 'emoji', cluster: '😀', font: '20px sans-serif' }];
  const doc = {
    collectCanvasMetricRequests() { events.push('collect'); return JSON.stringify({ revision: 1, requests }); },
    registerCanvasMetricReplies(_d: number, _f: number, _r: number, json: string) {
      events.push('register');
      assert.equal(JSON.parse(json)[0].measuredAdvancePx, 23.75);
      return false;
    },
    selectCanvasMetrics(value: boolean) { events.push(`select:${value}`); const changed = active !== value; active = value; return changed; },
    canvasMetricsActive() { return active; },
    getCanvasPageLayerTree() { return '{}'; },
  };
  let measures = 0;
  const context = { font: '', measureText() { measures++; return { width: 23.75 } as TextMetrics; } };
  const session = new CanvasMetricSession();
  const generation = { document: 1, fonts: 1 };
  return { session, doc, generation, events, context, measures: () => measures };
}

test('prepares only after fonts, registers then selects, and warm revision reuses measurements', async () => {
  const f = fixture();
  const prepare = () => f.session.prepare(f.doc, f.generation, 'canvas2d', () => true,
    async () => { f.events.push('fonts'); }, () => f.context);
  assert.equal(await prepare(), true);
  assert.deepEqual(f.events, ['fonts', 'collect', 'fonts', 'register', 'select:true']);
  assert.equal(await prepare(), false);
  assert.equal(f.measures(), 1);
});

test('late view or released document never touches a freed WASM handle', async () => {
  const f = fixture();
  let current = true;
  let ready!: () => void;
  const waiting = new Promise<void>(resolve => { ready = resolve; });
  const task = f.session.prepare(f.doc, f.generation, 'canvas2d', () => current, () => waiting, () => f.context);
  current = false;
  ready();
  assert.equal(await task, false);
  assert.deepEqual(f.events, []);
});

test('font generation invalidation discards pending preparation', async () => {
  const f = fixture();
  let ready!: () => void;
  const waiting = new Promise<void>(resolve => { ready = resolve; });
  const task = f.session.prepare(f.doc, f.generation, 'canvas2d', () => true, () => waiting, () => f.context);
  f.session.invalidate(); ready();
  assert.equal(await task, false);
  assert.deepEqual(f.events, []);
});

test('CanvasKit selection restores portable mode without requesting Canvas metrics', async () => {
  const f = fixture(); f.doc.selectCanvasMetrics(true); f.events.length = 0;
  assert.equal(await f.session.prepare(f.doc, f.generation, 'canvaskit', () => true,
    async () => { throw Error('not expected'); }, () => null), true);
  assert.deepEqual(f.events, ['select:false']);
});

test('measurement failure is diagnostic and cannot retain active stale positions', async () => {
  const f = fixture(); f.doc.selectCanvasMetrics(true);
  assert.equal(await f.session.prepare(f.doc, f.generation, 'canvas2d', () => true,
    async () => {}, () => null), true);
  assert.equal(f.doc.canvasMetricsActive(), false);
  assert.match(f.session.lastError!, /unavailable/);
});

test('portable transaction restores Canvas on success and on exception; nested export is safe', () => {
  const f = fixture(); f.doc.selectCanvasMetrics(true);
  assert.equal(withPortableMetrics(f.doc, () => {
    assert.equal(f.doc.canvasMetricsActive(), false);
    return withPortableMetrics(f.doc, () => 2);
  }), 2);
  assert.equal(f.doc.canvasMetricsActive(), true);
  assert.throws(() => withPortableMetrics(f.doc, () => { throw Error('export failed'); }), /export failed/);
  assert.equal(f.doc.canvasMetricsActive(), true);
});
