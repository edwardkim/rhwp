'use strict';

const {spawnSync} = require('node:child_process');
const path = require('node:path');

const MAX_BYTES = 2 * 1024 * 1024;
const REPORTS = [
  ['Run canvas visual diff and PDF report', [
    ['direct', '직접 PDF ↔ 호환 PDF', 'pdf-results.json'],
    ['browser', '브라우저 ↔ 호환 PDF', 'pdf-results.json'],
    ['canvas', 'Canvas 시각 비교', 'results.json'],
  ]],
  ['Run selected CanvasKit readiness gate', [['readiness', 'CanvasKit 전환 검사', 'baseline-report.json']]],
];

function fromUpload(artifacts, name, run, step) {
  if (!step) return null;
  const matches = artifacts.filter(a => a.name === name && !a.expired &&
    a.workflow_run?.id === run.id && a.workflow_run.head_sha === run.head_sha &&
    Date.parse(a.created_at) >= Date.parse(step.started_at) &&
    Date.parse(a.created_at) <= Date.parse(step.completed_at));
  return matches.length === 1 ? matches[0] : null;
}

async function readIndex(api, params, artifact, run, runUrl) {
  if (!artifact || !(artifact.size_in_bytes > 0 && artifact.size_in_bytes <= MAX_BYTES)) return null;
  let response;
  try {
    response = await api.downloadArtifact({...params, artifact_id:artifact.id, archive_format:'zip'});
  } catch (error) {
    if ([404,410].includes(error.status)) return null;
    throw error;
  }
  const bytes = Buffer.from(response.data);
  if (bytes.length > MAX_BYTES) return null;
  const read = spawnSync('python3', ['-I', path.join(__dirname, 'pr_ci_render_reports.py')], {
    input:bytes, timeout:10000, maxBuffer:MAX_BYTES,
  });
  if (read.status !== 0) return null;
  let index;
  try {
    index = JSON.parse(read.stdout);
  } catch {
    return null;
  }
  if (index?.schema_version !== 1 || index.run_id !== run.id || index.run_attempt !== run.run_attempt ||
      index.run_url !== runUrl + '/attempts/' + run.run_attempt || index.pr_head !== run.head_sha ||
      !/^[0-9a-f]{40}$/.test(index.tested_sha)) return null;
  return index;
}

async function renderEvidence(github, params, run, jobs, base) {
  const job = jobs.find(j => j.name === 'Canvas visual diff');
  if (!job) return [];
  const expected = REPORTS.filter(([name]) => job.steps?.some(s => s.name === name &&
    ['success', 'failure'].includes(s.conclusion))).flatMap(([, reports]) => reports);
  const runUrl = base + '/actions/runs/' + run.id;
  if (!expected.length) {
    const skipped = REPORTS.every(([name]) => job.steps?.some(s => s.name === name && s.conclusion === 'skipped'));
    if (!skipped) return [];
    return [{name:'Render Diff', result:'측정 생략', point:'시각 비교 0건', rank:1,
      link:'[작업 로그](' + runUrl + '/job/' + job.id + ')'}];
  }
  const uploadStep = name => job.steps.find(s => s.name === name && s.conclusion === 'success');
  const upload = uploadStep('Upload render diff index');
  const api = github.rest.actions;
  const artifacts = upload ? await github.paginate(api.listWorkflowRunArtifacts,
    {...params, run_id:run.id, per_page:100}) : [];
  const artifact = fromUpload(artifacts, 'render-diff-index-' + run.id + '-' + run.run_attempt, run, upload);
  const index = await readIndex(api, params, artifact, run, runUrl);
  const data = index?.comparisons || {};
  const full = fromUpload(artifacts, 'render-diff-artifacts', run, uploadStep('Upload render diff artifacts'));
  for (const [key, item] of Object.entries(data)) {
    if (!key.startsWith('print:') || !full || item?.role !== 'print' ||
        ![index.pr_head, index.tested_sha].includes(item.metrics?.measured_commit)) continue;
    const tsv = Array.isArray(item.evidence) && item.evidence.find(p => typeof p === 'string' && p.endsWith('/silhouette.tsv'));
    if (tsv && /^[0-9a-f]{64}$/.test(index.files?.[tsv])) {
      expected.push([key, 'Visual Sweep ' + String(item.metrics.backend).replace(/[^a-zA-Z0-9_-]/g, '').slice(0,40),
        'silhouette.tsv']);
    }
  }

  return expected.map(([key,name,file]) => {
    const row = data[key];
    if (typeof row?.result !== 'string' || typeof row.point !== 'string' || ![0,1,2,3].includes(row.priority)) {
      return {name, result:'결과 미확인', point:file, rank:1, link:'[측정 작업](' + runUrl + '/job/' + job.id + ')'};
    }
    const result = row.result.replace(/[^\p{L}\p{N} ,/%.-]/gu, ' ').slice(0, 100);
    const link = '[상세 표](' + runUrl + '/attempts/' + run.run_attempt + ') / [산출물](' +
      runUrl + '/artifacts/' + (full || artifact).id + ')';
    return {name,result,point:row.point,rank:row.priority,link};
  });
}
module.exports = {renderEvidence};
