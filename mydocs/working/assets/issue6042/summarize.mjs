// Stage 1 원본 DOM 관찰 JSON의 재집계. 브라우저를 실행하거나 제품 상태를 바꾸지 않는다.
// node mydocs/working/assets/issue6042/summarize.mjs
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
const folder = dirname(fileURLToPath(import.meta.url));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const statistics = values => {
  const a = values.filter(Number.isFinite).sort((x, y) => x - y);
  if (!a.length) return null;
  const mean = a.reduce((s, n) => s + n, 0) / a.length;
  return { n: a.length, median: (a[Math.floor((a.length - 1) / 2)] + a[Math.ceil((a.length - 1) / 2)]) / 2,
    p95: a[Math.ceil(a.length * .95) - 1], min: a[0], max: a.at(-1), mean,
    sd: Math.sqrt(a.reduce((s, n) => s + (n - mean) ** 2, 0) / a.length) };
};
const files = readdirSync(folder).filter(f => f.endsWith('.json') && !/^(discarded|intermediate|summary)/.test(f)).sort();
const runs = files.map(file => ({ file, data: JSON.parse(readFileSync(resolve(folder, file), 'utf8')) }));
const baseline = runs.filter(r => r.data.kind === 'baseline-scroll').map(({ file, data: d }) => {
  const traces = d.evidence.traces;
  const counters = {};
  for (const key of new Set(traces.flatMap(t => Object.keys(t.counters)))) {
    counters[key] = Object.fromEntries(['calls', 'inclusiveMs', 'maxMs', 'units'].map(field =>
      [field, statistics(traces.map(t => t.counters[key]?.[field] ?? 0))]));
  }
  return { file, pageCount: d.evidence.pageCount, columns: d.evidence.columns, zoom: d.evidence.zoom,
    visible: d.evidence.visible, retained: d.evidence.retained, totalPixels: d.evidence.totalAllocatedPixels,
    idlePoolPixels: d.evidence.idlePoolPixels,
    settledMs: statistics(d.samples.map(s => s.knownWorkNextFrameMs)),
    downMs: statistics(d.samples.filter(s => s.round % 2 === 0).map(s => s.knownWorkNextFrameMs)),
    backMs: statistics(d.samples.filter(s => s.round % 2 === 1).map(s => s.knownWorkNextFrameMs)),
    setterMs: statistics(d.samples.map(s => s.syncMs)),
    milestones: Object.fromEntries(['preview', 'visibleFirst', 'focusedSharp', 'visibleStable', 'retainedComplete'].map(m =>
      [m, statistics(traces.map(t => t.milestones[m]))])),
    counters, frames: statistics(traces.flatMap(t => t.frames)),
    statuses: traces.reduce((s, t) => ({ ...s, [t.status]: (s[t.status] ?? 0) + 1 }), {}),
    errors: d.evidence.errors };
});
const overhead = {};
for (const { file, data } of runs.filter(r => r.data.kind === 'observation-overhead')) {
  const key = file.replace(/-overhead-\d\.json$/, '');
  const group = overhead[key] ??= { files: [], off: [], on: [], pairedDifference: [] };
  group.files.push(file);
  for (const round of new Set(data.samples.filter(s => s.round >= 2).map(s => s.round))) {
    const off = data.samples.find(s => s.round === round && !s.enabled);
    const on = data.samples.find(s => s.round === round && s.enabled);
    if (!off || !on) throw Error('missing paired sample');
    group.off.push(off.knownWorkNextFrameMs); group.on.push(on.knownWorkNextFrameMs);
    group.pairedDifference.push(on.knownWorkNextFrameMs - off.knownWorkNextFrameMs);
  }
}
for (const group of Object.values(overhead)) {
  for (const field of ['off', 'on', 'pairedDifference']) group[field] = statistics(group[field]);
  group.medianChangePercent = (group.on.median / group.off.median - 1) * 100;
}
const zoom = runs.filter(r => r.file.includes('-zoom-')).map(({ file, data }) => ({ file,
  traces: data.traces.map(t => ({ source: t.source, status: t.status, milestones: t.milestones, reason: t.reason })),
  errors: data.errors, pages: data.pageCount, columns: data.columns, zoom: data.zoom }));
const hashes = Object.fromEntries(readdirSync(folder).filter(f => /\.(json|mjs|jpg|log)$/.test(f)
  && !f.startsWith('summary')).sort().map(f => [f, hash(readFileSync(resolve(folder, f)))]));
console.log(JSON.stringify({ baseline, overhead, zoom, hashes }, null, 2));
