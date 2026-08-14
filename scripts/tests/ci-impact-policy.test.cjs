'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const { classifyChanges } = require('../ci-impact-classifier.cjs');
const {
  CI_AUDITED_JOB_IDS,
  CI_FRONTEND_JOBS,
  CI_JOB_ALIASES,
  CI_NATIVE_JOB,
  CI_PULL_REQUEST_PATHS_IGNORE,
  CI_RUST_JOBS,
  CODEQL_JOBS,
  CODEQL_PULL_REQUEST_PATHS_IGNORE,
  RENDER_DIFF_PULL_REQUEST_PATHS,
  WORKFLOW_ORDER,
  WORKFLOW_PATHS,
  auditPolicyRuns,
  determinePolicy,
  parseStatusDescription,
  runCli,
  selectLatestWorkflowRun,
  workflowRunExpected,
} = require('../ci-impact-policy.cjs');

const CI_WORKFLOW = fs.readFileSync(
  path.join(__dirname, '../../.github/workflows/ci.yml'),
  'utf8',
);

const HEAD_SHA = 'a'.repeat(40);
const BASE_SHA = 'b'.repeat(40);
const HEAD_BRANCH = 'topic';

function classificationFor(files) {
  return classifyChanges({ eventName: 'pull_request', files });
}

function policyInput(overrides = {}) {
  const files = overrides.files || [
    { filename: 'rhwp-studio/src/command/shortcut-map.ts', status: 'modified' },
  ];
  return {
    repository: 'edwardkim/rhwp',
    pullRequest: {
      number: 123,
      baseRef: 'devel',
      baseSha: BASE_SHA,
      headSha: HEAD_SHA,
      headBranch: HEAD_BRANCH,
      headRepository: 'edwardkim/rhwp',
      authorPermission: 'write',
    },
    files,
    classification: classificationFor(files),
    controllerAvailable: true,
    ...overrides,
  };
}

function job(name, conclusion, steps = []) {
  return { name, status: 'completed', conclusion, steps };
}

function step(name, conclusion) {
  return { name, status: 'completed', conclusion };
}

function workflowRun(name, status = 'completed', conclusion = 'success') {
  return {
    name,
    path: WORKFLOW_PATHS[name],
    event: 'pull_request',
    status,
    conclusion,
    headSha: HEAD_SHA,
    headBranch: HEAD_BRANCH,
    headRepository: 'edwardkim/rhwp',
    pullNumbers: [123],
    baseShas: [BASE_SHA],
  };
}

function apiWorkflowRun(overrides = {}) {
  return {
    id: 1,
    name: 'CI',
    path: '.github/workflows/ci.yml',
    event: 'pull_request',
    head_sha: HEAD_SHA,
    head_branch: HEAD_BRANCH,
    head_repository: { full_name: 'external/rhwp' },
    run_attempt: 1,
    run_started_at: '2026-08-12T10:00:00Z',
    pull_requests: [],
    ...overrides,
  };
}

function workflowRunIdentity(overrides = {}) {
  return {
    name: 'CI',
    path: '.github/workflows/ci.yml',
    pullNumber: 123,
    baseRef: 'devel',
    baseSha: BASE_SHA,
    headSha: HEAD_SHA,
    headBranch: HEAD_BRANCH,
    headRepository: 'external/rhwp',
    ...overrides,
  };
}

function ciJobs(classification, fastPass = false) {
  const checkout = fastPass ? 'skipped' : 'success';
  const jobs = [
    job('CI preflight', 'success', [step('Check out trusted CI impact classifier', checkout)]),
    job('Build & Test', 'success'),
  ];
  if (fastPass) {
    return jobs.concat(
      [...CI_RUST_JOBS, CI_NATIVE_JOB, ...CI_FRONTEND_JOBS]
        .map((name) => job(name, 'skipped')),
    );
  }
  const rust = classification.rust_required === 'true' ? 'success' : 'skipped';
  jobs.push(...CI_RUST_JOBS.map((name) => {
    const aliases = CI_JOB_ALIASES[name] || [name];
    return job(rust === 'success' ? aliases.at(-1) : aliases[0], rust);
  }));
  jobs.push(job(
    CI_NATIVE_JOB,
    classification.native_skia_required === 'true' ? 'success' : 'skipped',
  ));
  const frontend = {
    none: ['skipped', 'skipped'],
    unit: ['success', 'skipped'],
    package: ['skipped', 'success'],
  }[classification.frontend_mode];
  jobs.push(job(CI_FRONTEND_JOBS[0], frontend[0]));
  jobs.push(job(CI_FRONTEND_JOBS[1], frontend[1]));
  return jobs;
}

function codeqlJobs(classification, fastPass = false) {
  const jobs = [job('CodeQL preflight', 'success', [
    step('Check out trusted CodeQL impact classifier', fastPass ? 'skipped' : 'success'),
  ])];
  if (fastPass) {
    jobs.push(job('Analyze (${{ matrix.language }})', 'skipped'));
    return jobs;
  }
  const selected = new Set(
    classification.codeql_languages === 'none'
      ? []
      : classification.codeql_languages.split(','),
  );
  for (const [language, name] of Object.entries(CODEQL_JOBS)) {
    jobs.push(job(name, 'success', [
      step('Skip unselected language', selected.has(language) ? 'skipped' : 'success'),
      step('Perform CodeQL Analysis', selected.has(language) ? 'success' : 'skipped'),
    ]));
  }
  return jobs;
}

function renderJobs(classification, fastPass = false) {
  return [
    job('Render Diff preflight', 'success', [
      step('Check out trusted CI impact classifier', fastPass ? 'skipped' : 'success'),
    ]),
    job(
      'Canvas visual diff',
      fastPass || classification.render_required !== 'true' ? 'skipped' : 'success',
    ),
  ];
}

function workflowEvidence(policy, options = {}) {
  const evidence = {};
  if (policy.expected_workflows.CI === 'true' && !options.omitCi) {
    evidence.CI = {
      run: workflowRun('CI'),
      jobsCollected: true,
      jobs: ciJobs(policy.classification, options.fastPass === true),
    };
  }
  if (policy.expected_workflows.CodeQL === 'true' && !options.omitCodeql) {
    evidence.CodeQL = {
      run: workflowRun('CodeQL'),
      jobsCollected: true,
      jobs: codeqlJobs(policy.classification, options.fastPass === true),
    };
  }
  if (policy.expected_workflows['Render Diff'] === 'true' && !options.omitRender) {
    evidence['Render Diff'] = {
      run: workflowRun('Render Diff'),
      jobsCollected: true,
      jobs: renderJobs(policy.classification, options.fastPass === true),
    };
  }
  return evidence;
}

test('same-repository Studio unit PR gets selective three-workflow policy', () => {
  const policy = determinePolicy(policyInput());
  assert.equal(policy.decision, 'selective');
  assert.equal(policy.head_trust, 'same-repository');
  assert.deepEqual(policy.expected_workflows, {
    CI: 'true',
    CodeQL: 'true',
    'Render Diff': 'true',
  });
  assert.equal(policy.classification.frontend_mode, 'unit');
  assert.equal(policy.classification.codeql_languages, 'javascript-typescript');
});

test('external fork is mediated unless it changes the enforcement surface', () => {
  const input = policyInput();
  input.pullRequest = {
    ...input.pullRequest,
    headRepository: 'external/rhwp',
    authorPermission: 'read',
  };
  assert.equal(determinePolicy(input).decision, 'selective');

  const files = [{ filename: '.github/workflows/ci.yml', status: 'modified' }];
  const blocked = determinePolicy(policyInput({
    files,
    classification: classificationFor(files),
    pullRequest: input.pullRequest,
  }));
  assert.equal(blocked.decision, 'blocked');
  assert.equal(blocked.reason, 'fail-closed:untrusted-enforcement-change');
});

test('collaborator workflow change is full and requires CI plus CodeQL', () => {
  const files = [{ filename: '.github/workflows/ci.yml', status: 'modified' }];
  const policy = determinePolicy(policyInput({ files, classification: classificationFor(files) }));
  assert.equal(policy.decision, 'full');
  assert.equal(policy.enforcement_surface_changed, 'true');
  assert.deepEqual(policy.expected_workflows, {
    CI: 'true',
    CodeQL: 'true',
    'Render Diff': 'false',
  });
});

test('invalid classifier output and API collection failure both close to full', () => {
  const missing = determinePolicy(policyInput({ classification: null }));
  assert.equal(missing.decision, 'full');
  assert.equal(missing.reason, 'fail-closed:classifier-unavailable');

  const invalid = determinePolicy(policyInput({
    classification: {
      ...classificationFor([{ filename: 'src/lib.rs', status: 'modified' }]),
      codeql_languages: 'rust,javascript-typescript',
    },
  }));
  assert.equal(invalid.reason, 'fail-closed:classifier-output-invalid');

  const weakFull = determinePolicy(policyInput({
    classification: {
      ...classificationFor([{ filename: 'src/lib.rs', status: 'modified' }]),
      classification_status: 'full',
      rust_required: 'false',
      reason: 'fail-closed:forged-weak-full',
    },
  }));
  assert.equal(weakFull.reason, 'fail-closed:classifier-output-invalid');

  const failed = determinePolicy(policyInput({ forceFullReason: 'collection-error', files: [] }));
  assert.equal(failed.decision, 'full');
  assert.deepEqual(failed.expected_workflows, Object.fromEntries(
    WORKFLOW_ORDER.map((workflow) => [workflow, 'true']),
  ));
});

test('compact status description round-trips workflow and impact axes', () => {
  const policy = determinePolicy(policyInput());
  assert.ok(policy.status_description.length <= 140);
  assert.deepEqual(parseStatusDescription(policy.status_description), {
    v: '2',
    cv: '2',
    mode: 'selective',
    wf: '111',
    rust: '0',
    fe: 'unit',
    render: '0',
    skia: '0',
    ql: 'js',
  });
  assert.throws(
    () => parseStatusDescription(`${policy.status_description};rust=1`),
    /duplicate policy status field|incomplete/,
  );
});

test('mirrored trigger contracts match CI, CodeQL, and Render Diff workflows', () => {
  const workflows = path.join(__dirname, '..', '..', '.github', 'workflows');
  function triggerItems(filename, start, end) {
    const workflow = fs.readFileSync(path.join(workflows, filename), 'utf8');
    const block = workflow.split(start, 2)[1].split(end, 1)[0];
    return Array.from(
      block.matchAll(/^      - ['"]?([^'"\n]+)['"]?$/gm),
      (match) => match[1],
    );
  }
  assert.deepEqual(
    triggerItems('ci.yml', '  pull_request:\n', '  workflow_dispatch:\n'),
    CI_PULL_REQUEST_PATHS_IGNORE,
  );
  assert.deepEqual(
    triggerItems('codeql.yml', '  pull_request:\n', '  schedule:\n'),
    CODEQL_PULL_REQUEST_PATHS_IGNORE,
  );
  assert.deepEqual(
    triggerItems('render-diff.yml', '  pull_request:\n', '  workflow_dispatch:\n'),
    RENDER_DIFF_PULL_REQUEST_PATHS,
  );
  assert.equal(workflowRunExpected('CI', [{ filename: 'README.md' }]), false);
  assert.equal(workflowRunExpected('CodeQL', [{ filename: 'assets/logo/icon.svg' }]), false);
  assert.equal(workflowRunExpected('Render Diff', [{ filename: 'rhwp-studio/tests/a.ts' }]), true);
});

test('every impact-conditioned CI job is covered by the audit allowlist', () => {
  const conditioned = [...CI_WORKFLOW.matchAll(
    /^  ([A-Za-z0-9_-]+):\n([\s\S]*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)/gm,
  )].filter(([, , body]) => (
    /needs\.preflight\.outputs\.(?:rust_required|native_skia_required|frontend_mode)/.test(body)
  )).map(([, jobId]) => jobId).sort();
  assert.deepEqual(conditioned, Object.keys(CI_AUDITED_JOB_IDS).sort());

  const auditedNames = new Set([
    ...CI_RUST_JOBS,
    CI_NATIVE_JOB,
    ...CI_FRONTEND_JOBS,
    'Build & Test',
  ]);
  assert.deepEqual(
    [...new Set(Object.values(CI_AUDITED_JOB_IDS))].sort(),
    [...auditedNames].sort(),
  );
});

test('workflow selection accepts fork runs without PR associations but keeps exact head identity', () => {
  const matching = apiWorkflowRun();
  const wrongBranch = apiWorkflowRun({ id: 2, head_branch: 'other' });
  const wrongRepository = apiWorkflowRun({
    id: 3,
    head_repository: { full_name: 'another/rhwp' },
  });
  assert.equal(
    selectLatestWorkflowRun([wrongBranch, wrongRepository, matching], workflowRunIdentity()),
    matching,
  );
});

test('workflow selection prefers the newest run and rejects a mismatched PR association', () => {
  const olderRerun = apiWorkflowRun({
    id: 10,
    run_attempt: 2,
    run_started_at: '2026-08-12T10:00:00Z',
  });
  const newerRun = apiWorkflowRun({
    id: 11,
    run_attempt: 1,
    run_started_at: '2026-08-12T11:00:00Z',
  });
  const wrongPull = apiWorkflowRun({
    id: 12,
    run_started_at: '2026-08-12T12:00:00Z',
    pull_requests: [{
      number: 999,
      base: { ref: 'devel', sha: BASE_SHA },
      head: { ref: HEAD_BRANCH, sha: HEAD_SHA },
    }],
  });
  assert.equal(
    selectLatestWorkflowRun([olderRerun, newerRun, wrongPull], workflowRunIdentity()),
    newerRun,
  );
});

test('workflow selection uses run id before attempt when timestamps tie', () => {
  const olderRerun = apiWorkflowRun({ id: 10, run_attempt: 2 });
  const newerRun = apiWorkflowRun({ id: 11, run_attempt: 1 });
  assert.equal(
    selectLatestWorkflowRun([olderRerun, newerRun], workflowRunIdentity()),
    newerRun,
  );
});

test('aggregate audit accepts Stage 3-5 selective truth table', () => {
  const input = policyInput();
  const policy = determinePolicy(input);
  assert.deepEqual(auditPolicyRuns({
    ...input,
    policy,
    currentHeadSha: HEAD_SHA,
    workflows: workflowEvidence(policy),
  }), {
    publish: 'true',
    conclusion: 'success',
    reason: 'audit:stage3-5-truth-table',
  });
});

test('aggregate audit accepts executed reusable-workflow REST aliases', () => {
  const files = [{ filename: 'src/lib.rs', status: 'modified' }];
  const input = policyInput({ files, classification: classificationFor(files) });
  const policy = determinePolicy(input);
  const audit = auditPolicyRuns({
    ...input,
    policy,
    currentHeadSha: HEAD_SHA,
    workflows: workflowEvidence(policy),
  });
  assert.equal(audit.conclusion, 'success');
});

test('aggregate audit rejects duplicate reusable-workflow aliases', () => {
  const files = [{ filename: 'src/lib.rs', status: 'modified' }];
  const input = policyInput({ files, classification: classificationFor(files) });
  const policy = determinePolicy(input);
  const workflows = workflowEvidence(policy);
  workflows.CI.jobs.push(job('build-test-archive-a', 'skipped'));
  const audit = auditPolicyRuns({ ...input, policy, currentHeadSha: HEAD_SHA, workflows });
  assert.equal(audit.conclusion, 'failure');
  assert.match(audit.reason, /duplicate-job:build-test-archive-a/);
});

test('aggregate audit stays pending until every expected workflow is present', () => {
  const input = policyInput();
  const policy = determinePolicy(input);
  const audit = auditPolicyRuns({
    ...input,
    policy,
    currentHeadSha: HEAD_SHA,
    workflows: workflowEvidence(policy, { omitRender: true }),
  });
  assert.equal(audit.conclusion, 'pending');
  assert.equal(audit.reason, 'missing-workflow:Render Diff');
});

test('aggregate audit stays pending when job evidence collection is unavailable', () => {
  const input = policyInput();
  const policy = determinePolicy(input);
  const workflows = workflowEvidence(policy);
  workflows.CI.jobsCollected = false;
  workflows.CI.jobs = [];
  const audit = auditPolicyRuns({ ...input, policy, currentHeadSha: HEAD_SHA, workflows });
  assert.deepEqual(audit, {
    publish: 'true',
    conclusion: 'pending',
    reason: 'job-evidence-unavailable:CI',
  });
});

test('aggregate audit rejects missing jobs after successful evidence collection', () => {
  const input = policyInput();
  const policy = determinePolicy(input);
  const workflows = workflowEvidence(policy);
  workflows.CI.jobs = [];
  const audit = auditPolicyRuns({ ...input, policy, currentHeadSha: HEAD_SHA, workflows });
  assert.equal(audit.conclusion, 'failure');
  assert.equal(audit.reason, 'CI:missing-job:CI preflight');
});

test('aggregate audit rejects a skipped required Rust job despite green aggregate', () => {
  const files = [{ filename: 'src/lib.rs', status: 'modified' }];
  const input = policyInput({ files, classification: classificationFor(files) });
  const policy = determinePolicy(input);
  const workflows = workflowEvidence(policy);
  workflows.CI.jobs.find((entry) => entry.name === CI_RUST_JOBS[0]).conclusion = 'skipped';
  const audit = auditPolicyRuns({ ...input, policy, currentHeadSha: HEAD_SHA, workflows });
  assert.equal(audit.conclusion, 'failure');
  assert.match(audit.reason, /CI:job-not-success:Lint/);
});

test('aggregate audit accepts safe full execution when worker classification degrades', () => {
  const input = policyInput();
  const policy = determinePolicy(input);
  const workflows = workflowEvidence(policy);

  for (const evidence of Object.values(workflows)) {
    const preflight = evidence.jobs.find((entry) => entry.name.endsWith('preflight'));
    preflight.steps.find((entry) => entry.name.startsWith('Check out trusted')).conclusion = 'failure';
  }
  for (const name of [...CI_RUST_JOBS, CI_NATIVE_JOB, ...CI_FRONTEND_JOBS]) {
    const aliases = CI_JOB_ALIASES[name] || [name];
    const lane = workflows.CI.jobs.find((entry) => aliases.includes(entry.name));
    lane.conclusion = 'success';
  }
  for (const [language, name] of Object.entries(CODEQL_JOBS)) {
    if (policy.classification.codeql_languages.split(',').includes(language)) continue;
    const lane = workflows.CodeQL.jobs.find((entry) => entry.name === name);
    lane.steps.find((entry) => entry.name === 'Skip unselected language').conclusion = 'skipped';
    lane.steps.find((entry) => entry.name === 'Perform CodeQL Analysis').conclusion = 'success';
  }
  workflows['Render Diff'].jobs.find(
    (entry) => entry.name === 'Canvas visual diff',
  ).conclusion = 'success';

  const audit = auditPolicyRuns({ ...input, policy, currentHeadSha: HEAD_SHA, workflows });
  assert.equal(audit.conclusion, 'success');
});

test('safe supersets do not permit failed optional lanes or skipped required analysis', () => {
  const input = policyInput();
  const policy = determinePolicy(input);

  const failedOptional = workflowEvidence(policy);
  failedOptional.CI.jobs.find((entry) => entry.name === CI_RUST_JOBS[0]).conclusion = 'failure';
  const failedCiAudit = auditPolicyRuns({
    ...input,
    policy,
    currentHeadSha: HEAD_SHA,
    workflows: failedOptional,
  });
  assert.equal(failedCiAudit.conclusion, 'failure');
  assert.match(failedCiAudit.reason, /job-not-skipped-or-success:Lint/);

  const skippedRequired = workflowEvidence(policy);
  const javascript = skippedRequired.CodeQL.jobs.find(
    (entry) => entry.name === CODEQL_JOBS['javascript-typescript'],
  );
  javascript.steps.find(
    (entry) => entry.name === 'Perform CodeQL Analysis',
  ).conclusion = 'skipped';
  const failedCodeqlAudit = auditPolicyRuns({
    ...input,
    policy,
    currentHeadSha: HEAD_SHA,
    workflows: skippedRequired,
  });
  assert.equal(failedCodeqlAudit.conclusion, 'failure');
  assert.match(failedCodeqlAudit.reason, /step-not-success:Perform CodeQL Analysis/);

  const failedRender = workflowEvidence(policy);
  failedRender['Render Diff'].jobs.find(
    (entry) => entry.name === 'Canvas visual diff',
  ).conclusion = 'failure';
  const failedRenderAudit = auditPolicyRuns({
    ...input,
    policy,
    currentHeadSha: HEAD_SHA,
    workflows: failedRender,
  });
  assert.equal(failedRenderAudit.conclusion, 'failure');
  assert.match(failedRenderAudit.reason, /job-not-skipped-or-success:Canvas visual diff/);
});

test('CodeQL audit accepts safe unselected analysis but rejects inconsistent step pairs', () => {
  const input = policyInput();
  const policy = determinePolicy(input);
  const workflows = workflowEvidence(policy);
  const rust = workflows.CodeQL.jobs.find((entry) => entry.name === CODEQL_JOBS.rust);
  rust.steps.find((entry) => entry.name === 'Skip unselected language').conclusion = 'skipped';
  rust.steps.find((entry) => entry.name === 'Perform CodeQL Analysis').conclusion = 'success';
  const safeFull = auditPolicyRuns({ ...input, policy, currentHeadSha: HEAD_SHA, workflows });
  assert.equal(safeFull.conclusion, 'success');

  rust.steps.find((entry) => entry.name === 'Skip unselected language').conclusion = 'success';
  const inconsistent = auditPolicyRuns({ ...input, policy, currentHeadSha: HEAD_SHA, workflows });
  assert.equal(inconsistent.conclusion, 'failure');
  assert.match(inconsistent.reason, /Analyze \(rust\).*unsafe-unselected-execution/);
});

test('trailing review-only fast pass is accepted only on unchanged enforcement surface', () => {
  const input = policyInput();
  const policy = determinePolicy(input);
  const success = auditPolicyRuns({
    ...input,
    policy,
    currentHeadSha: HEAD_SHA,
    workflows: workflowEvidence(policy, { fastPass: true }),
  });
  assert.equal(success.conclusion, 'success');

  const files = [{ filename: '.github/workflows/ci.yml', status: 'modified' }];
  const changedInput = policyInput({ files, classification: classificationFor(files) });
  const changedPolicy = determinePolicy(changedInput);
  const failure = auditPolicyRuns({
    ...changedInput,
    policy: changedPolicy,
    currentHeadSha: HEAD_SHA,
    workflows: workflowEvidence(changedPolicy, { fastPass: true }),
  });
  assert.equal(failure.conclusion, 'failure');
  assert.match(failure.reason, /fast-pass-with-enforcement-change/);
});

test('CodeQL fast pass accepts exactly one matrix skip representation', () => {
  const input = policyInput();
  const policy = determinePolicy(input);
  const workflows = workflowEvidence(policy, { fastPass: true });
  workflows.CodeQL.jobs = workflows.CodeQL.jobs.filter(
    (entry) => entry.name !== 'Analyze (${{ matrix.language }})',
  );
  workflows.CodeQL.jobs.push(
    ...Object.values(CODEQL_JOBS).map((name) => job(name, 'skipped')),
  );
  const expanded = auditPolicyRuns({
    ...input,
    policy,
    currentHeadSha: HEAD_SHA,
    workflows,
  });
  assert.equal(expanded.conclusion, 'success');

  workflows.CodeQL.jobs.push(job('Analyze (${{ matrix.language }})', 'skipped'));
  const duplicate = auditPolicyRuns({
    ...input,
    policy,
    currentHeadSha: HEAD_SHA,
    workflows,
  });
  assert.equal(duplicate.conclusion, 'failure');
  assert.match(duplicate.reason, /invalid-fast-pass-matrix/);
});

test('stale workflow identity cannot overwrite the current policy status', () => {
  const input = policyInput();
  const policy = determinePolicy(input);
  const workflows = workflowEvidence(policy);
  workflows.CI.run.headSha = 'b'.repeat(40);
  const audit = auditPolicyRuns({ ...input, policy, currentHeadSha: HEAD_SHA, workflows });
  assert.equal(audit.conclusion, 'failure');
  assert.equal(audit.reason, 'workflow-identity-mismatch:CI');
});

test('stale trigger head is not publishable even when workflow evidence matches the live PR', () => {
  const input = policyInput();
  const policy = determinePolicy(input);
  const audit = auditPolicyRuns({
    ...input,
    policy,
    currentHeadSha: 'c'.repeat(40),
    workflows: workflowEvidence(policy),
  });
  assert.deepEqual(audit, {
    publish: 'false',
    conclusion: 'failure',
    reason: 'stale-or-unresolved-head',
  });
});

test('CLI writes policy and aggregate audit outputs', (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-ci-policy-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const input = policyInput();
  const policy = determinePolicy(input);
  const inputPath = path.join(directory, 'input.json');
  const outputPath = path.join(directory, 'github-output.txt');
  const resultPath = path.join(directory, 'result.json');
  fs.writeFileSync(inputPath, JSON.stringify({
    ...input,
    currentHeadSha: HEAD_SHA,
    workflows: workflowEvidence(policy),
  }));
  const result = runCli([
    '--input', inputPath,
    '--github-output', outputPath,
    '--output', resultPath,
  ]);
  const outputs = fs.readFileSync(outputPath, 'utf8');
  assert.equal(result.audit.conclusion, 'success');
  assert.match(outputs, /^codeql_run_expected=true$/m);
  assert.match(outputs, /^audit_conclusion=success$/m);
  assert.equal(JSON.parse(fs.readFileSync(resultPath, 'utf8')).policy.policy_version, '2');
});
