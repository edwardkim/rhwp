'use strict';

const fs = require('node:fs');

const POLICY_VERSION = '5';
const POLICY_CONTEXT = 'CI Impact Policy';
const WORKFLOW_ORDER = ['CI', 'CodeQL', 'Render Diff'];
const WORKFLOW_PATHS = {
  CI: '.github/workflows/ci.yml',
  CodeQL: '.github/workflows/codeql.yml',
  'Render Diff': '.github/workflows/render-diff.yml',
};
const WRITER_PERMISSIONS = new Set(['admin', 'maintain', 'write']);
const FRONTEND_MODES = new Set(['none', 'unit', 'package']);
const BOOLEAN_VALUES = new Set(['true', 'false']);
const CLASSIFICATION_STATUSES = new Set(['classified', 'full']);
const CODEQL_LANGUAGE_ORDER = ['javascript-typescript', 'python', 'rust'];

const CI_PUSH_PATHS_IGNORE = [
  'mydocs/**',
  'docs/**',
  'samples/**',
  'pdf/**',
  'pdf-2020/**',
  'pdf-large/**',
  'assets/chrome/**',
  'assets/edge/**',
  'assets/logo/**',
  'assets/screenshots/**',
  '*.md',
  'LICENSE',
  '.github/ISSUE_TEMPLATE/**',
  '.github/FUNDING.yml',
  '.github/CODE_OF_CONDUCT.md',
  '.github/SECURITY.md',
  '.github/pull_request_template.md',
  '.github/dependabot.yml',
  'rhwp-logo.*',
];

const CI_PULL_REQUEST_PATHS_IGNORE = [
  'docs/**',
  'assets/chrome/**',
  'assets/edge/**',
  'assets/logo/**',
  'assets/screenshots/**',
  '*.md',
  'LICENSE',
  '.github/ISSUE_TEMPLATE/**',
  '.github/FUNDING.yml',
  '.github/CODE_OF_CONDUCT.md',
  '.github/SECURITY.md',
  '.github/pull_request_template.md',
  '.github/dependabot.yml',
  'rhwp-logo.*',
];

const CODEQL_PUSH_PATHS_IGNORE = [
  'mydocs/**',
  'docs/**',
  'samples/**',
  'pdf/**',
  'pdf-2020/**',
  'pdf-large/**',
  'assets/**',
  '*.md',
  'LICENSE',
  '.github/ISSUE_TEMPLATE/**',
  '.github/FUNDING.yml',
  '.github/CODE_OF_CONDUCT.md',
  '.github/SECURITY.md',
  '.github/pull_request_template.md',
  '.github/dependabot.yml',
  'rhwp-logo.*',
];

const CODEQL_PULL_REQUEST_PATHS_IGNORE = [
  'docs/**',
  'assets/**',
  '*.md',
  'LICENSE',
  '.github/ISSUE_TEMPLATE/**',
  '.github/FUNDING.yml',
  '.github/CODE_OF_CONDUCT.md',
  '.github/SECURITY.md',
  '.github/pull_request_template.md',
  '.github/dependabot.yml',
  'rhwp-logo.*',
];

const DEPLOY_PAGES_PUSH_PATHS_IGNORE = [
  'mydocs/**',
  'docs/**',
  'samples/**',
  'pdf/**',
  'pdf-2020/**',
  'pdf-large/**',
  'assets/**',
  '*.md',
  'LICENSE',
  '.github/ISSUE_TEMPLATE/**',
  '.github/FUNDING.yml',
  '.github/CODE_OF_CONDUCT.md',
  '.github/SECURITY.md',
  '.github/pull_request_template.md',
  '.github/dependabot.yml',
  'rhwp-logo.*',
];

const RENDER_DIFF_PULL_REQUEST_PATHS = [
  'Cargo.toml',
  'Cargo.lock',
  'src/paint/**',
  'src/model/**',
  'src/renderer/**',
  'src/document_core/queries/rendering.rs',
  'src/wasm_api.rs',
  'src/main.rs',
  'assets/fonts/**',
  'ttfs/**',
  'scripts/renderer_baseline.py',
  'scripts/renderer_baseline_manifest.json',
  'scripts/ci-impact-classifier.cjs',
  'scripts/generate_font_glyph_payload_fixture.py',
  'scripts/generate_exact_face_collection_fixture.py',
  'scripts/generate_font_native_hwpx_fixture.py',
  'scripts/requirements-font-fixtures.txt',
  'samples/render-p35-font-native-bitmap.hwpx',
  'tests/fixtures/fonts/**',
  'docs/canvaskit-parity-implementation.md',
  'docs/text-ir-v2.md',
  'rhwp-studio/**',
  '.github/workflows/render-diff.yml',
];

const CI_RUST_JOBS = [
  'Lint (fmt, clippy, WASM check)',
  'build-test-archive-slow',
  'build-test-archive-a',
  'build-test-archive-b',
  'test-slow-shard',
  'test-regular-shard-1',
  'test-regular-shard-2',
  'test-regular-shard-3',
];
// Reusable workflow calls have two observed REST job names: a skipped call keeps only
// its caller job id, while an executed call is reported as "caller / called job name".
// Audit one logical lane across both representations without accepting duplicates.
const CI_JOB_ALIASES = {
  'Lint (fmt, clippy, WASM check)': ['Lint (fmt, clippy, WASM check)'],
  'build-test-archive-slow': [
    'build-test-archive-slow',
    'build-test-archive-slow / Build test archive (slow 2)',
  ],
  'build-test-archive-a': [
    'build-test-archive-a',
    'build-test-archive-a / Build test archive (1)',
  ],
  'build-test-archive-b': [
    'build-test-archive-b',
    'build-test-archive-b / Build test archive (3)',
  ],
  'test-slow-shard': [
    'test-slow-shard',
    'test-slow-shard / Default-feature tests (slow shard)',
  ],
  'test-regular-shard-1': [
    'test-regular-shard-1',
    'test-regular-shard-1 / Default-feature tests (shard 1/3)',
  ],
  'test-regular-shard-2': [
    'test-regular-shard-2',
    'test-regular-shard-2 / Default-feature tests (shard 2/3)',
  ],
  'test-regular-shard-3': [
    'test-regular-shard-3',
    'test-regular-shard-3 / Default-feature tests (shard 3/3)',
  ],
};
const CI_NATIVE_JOB = 'Native Skia tests';
const CI_FRONTEND_JOBS = ['Frontend unit gates', 'Frontend package gates'];
// Job ids are the stable YAML-side identities. Values are the REST job names audited
// below. Tests derive every impact-conditioned ci.yml job and require this map to stay
// complete, so adding a new selectable lane cannot silently escape the controller.
const CI_AUDITED_JOB_IDS = {
  'build-test-archive-slow': 'build-test-archive-slow',
  'build-test-archive-a': 'build-test-archive-a',
  'build-test-archive-b': 'build-test-archive-b',
  'test-slow-shard': 'test-slow-shard',
  'test-regular-shard-1': 'test-regular-shard-1',
  'test-regular-shard-2': 'test-regular-shard-2',
  'test-regular-shard-3': 'test-regular-shard-3',
  lint: 'Lint (fmt, clippy, WASM check)',
  'native-skia-tests': CI_NATIVE_JOB,
  'frontend-unit-gates': CI_FRONTEND_JOBS[0],
  'frontend-package-gates': CI_FRONTEND_JOBS[1],
  'build-and-test': 'Build & Test',
};
const CODEQL_JOBS = {
  'javascript-typescript': 'Analyze (javascript-typescript)',
  python: 'Analyze (python)',
  rust: 'Analyze (rust)',
};

function fullClassification(reason) {
  return {
    rust_required: 'true',
    frontend_mode: 'package',
    render_required: 'true',
    native_skia_required: 'true',
    codeql_languages: CODEQL_LANGUAGE_ORDER.join(','),
    classification_status: 'full',
    classifier_version: 'unavailable',
    reason: `fail-closed:${reason}`,
  };
}

function normalizedCodeqlLanguages(value) {
  if (value === 'none') return 'none';
  const languages = String(value || '').split(',').filter(Boolean);
  if (languages.length === 0 || languages.length !== new Set(languages).size) return '';
  if (languages.some((language) => !CODEQL_LANGUAGE_ORDER.includes(language))) return '';
  const canonical = CODEQL_LANGUAGE_ORDER.filter((language) => languages.includes(language));
  return canonical.join(',') === languages.join(',') ? canonical.join(',') : '';
}

function normalizeClassification(value, fallbackReason = 'classifier-unavailable') {
  if (!value || typeof value !== 'object') return fullClassification(fallbackReason);
  const normalized = {
    rust_required: String(value.rust_required || ''),
    frontend_mode: String(value.frontend_mode || ''),
    render_required: String(value.render_required || ''),
    native_skia_required: String(value.native_skia_required || ''),
    codeql_languages: normalizedCodeqlLanguages(String(value.codeql_languages || '')),
    classification_status: String(value.classification_status || ''),
    classifier_version: String(value.classifier_version || ''),
    reason: String(value.reason || ''),
  };
  if (
    !BOOLEAN_VALUES.has(normalized.rust_required)
    || !FRONTEND_MODES.has(normalized.frontend_mode)
    || !BOOLEAN_VALUES.has(normalized.render_required)
    || !BOOLEAN_VALUES.has(normalized.native_skia_required)
    || !CLASSIFICATION_STATUSES.has(normalized.classification_status)
    || !normalized.codeql_languages
    || !/^[1-9][0-9]*$/.test(normalized.classifier_version)
    || !normalized.reason
  ) {
    return fullClassification('classifier-output-invalid');
  }
  const fullAxes = normalized.rust_required === 'true'
    && normalized.frontend_mode === 'package'
    && normalized.render_required === 'true'
    && normalized.native_skia_required === 'true'
    && normalized.codeql_languages === CODEQL_LANGUAGE_ORDER.join(',');
  if (
    (normalized.classification_status === 'full'
      && (!fullAxes || !normalized.reason.startsWith('fail-closed:')))
    || (normalized.classification_status === 'classified'
      && !normalized.reason.startsWith('classified:'))
  ) {
    return fullClassification('classifier-output-invalid');
  }
  return normalized;
}

function normalizeFile(file) {
  if (typeof file === 'string') return { filename: file, previous_filename: '' };
  return {
    filename: String(file?.filename || file?.path || ''),
    previous_filename: String(file?.previous_filename || file?.previousPath || ''),
  };
}

function allFilePaths(files) {
  return (Array.isArray(files) ? files : [])
    .map(normalizeFile)
    .flatMap((file) => [file.filename, file.previous_filename])
    .filter(Boolean);
}

function changesEnforcementSurface(files) {
  return allFilePaths(files).some((filename) => (
    filename.startsWith('.github/workflows/')
    || filename.startsWith('.github/actions/')
    || filename === 'scripts/ci-impact-classifier.cjs'
    || filename === 'scripts/ci-impact-policy.cjs'
    || filename === 'scripts/verify_review_only_merge_resolution.py'
  ));
}

function isReviewReferencePath(filename) {
  const pdfPrefixes = ['pdf/', 'pdf-2020/', 'pdf-large/'];
  return (
    filename.startsWith('samples/')
    && (
      filename.endsWith('.hwp')
      || filename.endsWith('.hwpx')
      || filename.endsWith('.pdf')
      || filename.endsWith('.png')
    )
  ) || (
    pdfPrefixes.some((prefix) => filename.startsWith(prefix))
    && filename.endsWith('.pdf')
  );
}

function isAllowedReviewFile(file) {
  const normalized = normalizeFile(file);
  if (normalized.filename.startsWith('mydocs/')) return true;
  return String(file?.status || '') === 'added'
    && isReviewReferencePath(normalized.filename);
}

function selectReviewOnlyCandidate(commits, currentBaseSha) {
  const entries = Array.isArray(commits) ? commits : [];
  if (entries.length === 0) {
    return { eligible: false, reason: 'no-pr-commits', candidateSha: '' };
  }
  let trailingCount = 0;
  let baseMergeBridge = null;
  let expectedSha = '';
  for (let index = entries.length - 1; index >= 0; index -= 1) {
    const commit = entries[index] || {};
    const sha = String(commit.sha || '');
    const parents = Array.isArray(commit.parents)
      ? commit.parents.map((parent) => String(parent?.sha || parent || '')).filter(Boolean)
      : [];
    const files = Array.isArray(commit.files) ? commit.files : [];
    if (!/^[0-9a-f]{40}$/.test(sha)) {
      return { eligible: false, reason: 'invalid-commit-sha', candidateSha: '' };
    }
    if (expectedSha && sha !== expectedSha) {
      return {
        eligible: false,
        reason: `non-linear-review-tail:${sha}`,
        candidateSha: '',
      };
    }
    const isCurrentBaseMerge = parents.length === 2
      && parents.filter((parent) => parent === currentBaseSha).length === 1;
    if (isCurrentBaseMerge) {
      if (baseMergeBridge) {
        return {
          eligible: false,
          reason: `multiple-current-base-update-merges:${sha}`,
          candidateSha: '',
        };
      }
      const sourceParentSha = parents.find((parent) => parent !== currentBaseSha) || '';
      if (!sourceParentSha) {
        return {
          eligible: false,
          reason: `current-base-source-parent-unavailable:${sha}`,
          candidateSha: '',
        };
      }
      baseMergeBridge = { mergeSha: sha, sourceParentSha };
      expectedSha = sourceParentSha;
      continue;
    }
    if (files.length === 0 || files.length >= 300) {
      return { eligible: false, reason: `unusable-file-list:${sha}`, candidateSha: '' };
    }
    if (files.every(isAllowedReviewFile)) {
      if (parents.length !== 1) {
        return { eligible: false, reason: `review-only-merge-not-reusable:${sha}`, candidateSha: '' };
      }
      trailingCount += 1;
      expectedSha = parents[0];
      continue;
    }
    if (trailingCount === 0 && !baseMergeBridge) {
      return { eligible: false, reason: 'no-trailing-review-only-commits', candidateSha: '' };
    }
    if (baseMergeBridge && baseMergeBridge.sourceParentSha !== sha) {
      return { eligible: false, reason: 'base-merge-source-parent-mismatch', candidateSha: '' };
    }
    return {
      eligible: true,
      reason: baseMergeBridge ? 'candidate-with-current-base-bridge' : 'candidate-with-review-tail',
      candidateSha: sha,
      trailingCount,
      baseMergeBridge,
    };
  }
  if (baseMergeBridge) {
    return {
      eligible: true,
      reason: 'direct-current-base-bridge',
      candidateSha: baseMergeBridge.sourceParentSha,
      trailingCount,
      baseMergeBridge,
    };
  }
  return { eligible: false, reason: 'no-code-candidate', candidateSha: '' };
}

function matchesPathPattern(filename, pattern) {
  if (pattern === '*.md') return !filename.includes('/') && filename.endsWith('.md');
  if (pattern === 'rhwp-logo.*') return /^rhwp-logo\.[^/]+$/.test(filename);
  if (pattern.endsWith('/**')) return filename.startsWith(pattern.slice(0, -2));
  return filename === pattern;
}

function workflowRunExpected(workflow, files) {
  const paths = allFilePaths(files);
  if (paths.length === 0) return true;
  if (workflow === 'CI') {
    return !paths.every((filename) => (
      CI_PULL_REQUEST_PATHS_IGNORE.some((pattern) => matchesPathPattern(filename, pattern))
    ));
  }
  if (workflow === 'CodeQL') {
    return !paths.every((filename) => (
      CODEQL_PULL_REQUEST_PATHS_IGNORE.some((pattern) => matchesPathPattern(filename, pattern))
    ));
  }
  if (workflow === 'Render Diff') {
    return paths.some((filename) => (
      RENDER_DIFF_PULL_REQUEST_PATHS.some((pattern) => matchesPathPattern(filename, pattern))
    ));
  }
  throw new Error(`unsupported workflow: ${workflow}`);
}

function expectedWorkflowMap(files, forceAll = false) {
  return Object.fromEntries(WORKFLOW_ORDER.map((workflow) => [
    workflow,
    forceAll || workflowRunExpected(workflow, files) ? 'true' : 'false',
  ]));
}

function workflowRunMatchesPull(run, identity) {
  const pulls = Array.isArray(run?.pull_requests) ? run.pull_requests : [];
  // GitHub can return an empty pull_requests array for fork runs. In that case the
  // repository + branch + exact SHA guards in selectLatestWorkflowRun remain the
  // authority. When association data exists, reject runs from another PR/base.
  if (pulls.length === 0) return true;
  return pulls.some((pull) => (
    Number(pull?.number || 0) === Number(identity.pullNumber || 0)
    && String(pull?.base?.ref || '') === String(identity.baseRef || '')
    && (
      identity.allowPriorBase === true
      || String(pull?.base?.sha || '') === String(identity.baseSha || '')
    )
    && String(pull?.head?.ref || '') === String(identity.headBranch || '')
    && String(pull?.head?.sha || '') === String(identity.headSha || '')
  ));
}

function selectLatestWorkflowRun(runs, identity = {}) {
  const candidates = (Array.isArray(runs) ? runs : []).filter((run) => (
    String(run?.name || '') === String(identity.name || '')
    && String(run?.path || '').split('@')[0] === String(identity.path || '')
    && String(run?.event || '') === 'pull_request'
    && String(run?.head_sha || '') === String(identity.headSha || '')
    && String(run?.head_branch || '') === String(identity.headBranch || '')
    && String(run?.head_repository?.full_name || '') === String(identity.headRepository || '')
    && workflowRunMatchesPull(run, identity)
  ));
  return candidates.sort((left, right) => {
    const rightStarted = Date.parse(right.run_started_at || right.created_at || 0) || 0;
    const leftStarted = Date.parse(left.run_started_at || left.created_at || 0) || 0;
    if (rightStarted !== leftStarted) return rightStarted - leftStarted;
    const runId = Number(right.id || 0) - Number(left.id || 0);
    if (runId !== 0) return runId;
    return Number(right.run_attempt || 0) - Number(left.run_attempt || 0);
  })[0] || null;
}

function determineHeadTrust(repository, pullRequest = {}) {
  const headRepository = String(pullRequest.headRepository || '');
  const authorPermission = String(pullRequest.authorPermission || '').toLowerCase();
  if (headRepository && headRepository === repository) return 'same-repository';
  if (WRITER_PERMISSIONS.has(authorPermission)) return 'collaborator-fork';
  return 'controller-mediated-fork';
}

function encodeCodeqlLanguages(value) {
  if (value === 'none') return 'none';
  return String(value).split(',').map((language) => ({
    'javascript-typescript': 'js',
    python: 'py',
    rust: 'rs',
  })[language] || 'all').join(',');
}

function statusDescription(policy) {
  const classification = policy.classification;
  const workflowBits = WORKFLOW_ORDER
    .map((workflow) => policy.expected_workflows[workflow] === 'true' ? '1' : '0')
    .join('');
  return [
    `v=${POLICY_VERSION}`,
    `cv=${classification.classifier_version}`,
    `mode=${policy.decision}`,
    `rfp=${policy.trusted_review_fast_pass === 'true' ? '1' : '0'}`,
    `wf=${workflowBits}`,
    `rust=${classification.rust_required === 'true' ? '1' : '0'}`,
    `fe=${classification.frontend_mode}`,
    `render=${classification.render_required === 'true' ? '1' : '0'}`,
    `skia=${classification.native_skia_required === 'true' ? '1' : '0'}`,
    `ql=${encodeCodeqlLanguages(classification.codeql_languages)}`,
    `b=${policy.base_sha}`,
  ].join(';');
}

function parseStatusDescription(description) {
  const fields = new Map();
  for (const part of String(description || '').split(';')) {
    const separator = part.indexOf('=');
    if (separator <= 0) throw new Error('malformed policy status description');
    const key = part.slice(0, separator);
    const value = part.slice(separator + 1);
    if (fields.has(key)) throw new Error(`duplicate policy status field: ${key}`);
    fields.set(key, value);
  }
  const expected = [
    'v', 'cv', 'mode', 'rfp', 'wf', 'rust', 'fe', 'render', 'skia', 'ql', 'b',
  ];
  if (fields.size !== expected.length || expected.some((key) => !fields.has(key))) {
    throw new Error('policy status fields are incomplete');
  }
  if (fields.get('v') !== POLICY_VERSION) throw new Error('unsupported policy version');
  if (!/^(?:[1-9][0-9]*|unavailable)$/.test(fields.get('cv'))) {
    throw new Error('invalid classifier version');
  }
  if (!new Set(['blocked', 'full', 'selective']).has(fields.get('mode'))) {
    throw new Error('invalid policy mode');
  }
  if (!new Set(['0', '1']).has(fields.get('rfp'))) {
    throw new Error('invalid trusted review fast-pass axis');
  }
  if (!/^[01]{3}$/.test(fields.get('wf'))) throw new Error('invalid workflow axes');
  if (!new Set(['0', '1']).has(fields.get('rust'))) throw new Error('invalid rust axis');
  if (!FRONTEND_MODES.has(fields.get('fe'))) throw new Error('invalid frontend axis');
  if (!new Set(['0', '1']).has(fields.get('render'))) throw new Error('invalid render axis');
  if (!new Set(['0', '1']).has(fields.get('skia'))) throw new Error('invalid skia axis');
  if (!/^(none|all|(?:js|py|rs)(?:,(?:js|py|rs))*)$/.test(fields.get('ql'))) {
    throw new Error('invalid CodeQL axis');
  }
  const languages = fields.get('ql').split(',');
  if (languages.length !== new Set(languages).size) throw new Error('duplicate CodeQL axis');
  if (!/^(?:[0-9a-f]{40}|unavailable)$/.test(fields.get('b'))) {
    throw new Error('invalid base SHA');
  }
  return Object.fromEntries(fields);
}

function determinePolicy(input = {}) {
  const repository = String(input.repository || '');
  const pullRequest = input.pullRequest || {};
  let classification = normalizeClassification(input.classification);
  let forceFullReason = String(input.forceFullReason || '');
  const baseShaValue = String(pullRequest.baseSha || '').toLowerCase();
  const baseSha = /^[0-9a-f]{40}$/.test(baseShaValue) ? baseShaValue : 'unavailable';
  if (baseSha === 'unavailable' && !forceFullReason) forceFullReason = 'base-sha-unavailable';
  const controllerAvailable = input.controllerAvailable !== false;
  if (forceFullReason) classification = fullClassification(forceFullReason);
  else if (!controllerAvailable) classification = fullClassification('trusted-controller-unavailable');

  const headTrust = determineHeadTrust(repository, pullRequest);
  const enforcementChanged = changesEnforcementSurface(input.files);
  let decision = classification.classification_status === 'classified' ? 'selective' : 'full';
  if (
    headTrust === 'controller-mediated-fork'
    && (enforcementChanged || forceFullReason)
  ) {
    decision = 'blocked';
    classification = fullClassification(
      enforcementChanged
        ? 'untrusted-enforcement-change'
        : `untrusted-incomplete-file-list:${forceFullReason}`,
    );
  }
  const forceAllWorkflows = Boolean(forceFullReason || !controllerAvailable);
  const policy = {
    policy_version: POLICY_VERSION,
    policy_context: POLICY_CONTEXT,
    base_sha: baseSha,
    decision,
    skip_eligible: decision === 'selective' ? 'true' : 'false',
    head_trust: headTrust,
    enforcement_surface_changed: enforcementChanged ? 'true' : 'false',
    trusted_review_fast_pass: 'false',
    review_candidate_sha: '',
    expected_workflows: expectedWorkflowMap(input.files, forceAllWorkflows),
    reason: classification.reason,
    classification,
  };
  const reviewAudit = auditReviewReuse({ ...input, policy });
  if (reviewAudit.eligible === true) {
    policy.trusted_review_fast_pass = 'true';
    policy.review_candidate_sha = reviewAudit.candidateSha;
  }
  policy.review_reuse_reason = reviewAudit.reason;
  policy.ci_run_expected = policy.expected_workflows.CI;
  policy.status_description = statusDescription(policy);
  return policy;
}

function normalizedJobs(jobs) {
  const byName = new Map();
  for (const job of Array.isArray(jobs) ? jobs : []) {
    const name = String(job?.name || '');
    if (!name) continue;
    const entries = byName.get(name) || [];
    entries.push({
      status: String(job?.status || ''),
      conclusion: String(job?.conclusion || ''),
      steps: Array.isArray(job?.steps) ? job.steps.map((step) => ({
        name: String(step?.name || ''),
        status: String(step?.status || ''),
        conclusion: String(step?.conclusion || ''),
      })) : [],
    });
    byName.set(name, entries);
  }
  return byName;
}

function exactJob(byName, name) {
  const entries = byName.get(name) || [];
  if (entries.length !== 1) {
    return { error: entries.length === 0 ? `missing-job:${name}` : `duplicate-job:${name}` };
  }
  return { job: entries[0] };
}

function requireJobConclusion(byName, name, conclusion) {
  const resolved = exactJob(byName, name);
  if (resolved.error) return resolved.error;
  const job = resolved.job;
  if (job.status !== 'completed' || job.conclusion !== conclusion) {
    return `job-not-${conclusion}:${name}:${job.status || 'unknown'}:${job.conclusion || 'unknown'}`;
  }
  return '';
}

function safeConclusions(expected) {
  return expected === 'skipped' ? new Set(['skipped', 'success']) : new Set([expected]);
}

function requireSafeJobConclusion(byName, name, expected) {
  const resolved = exactJob(byName, name);
  if (resolved.error) return resolved.error;
  const job = resolved.job;
  const allowed = safeConclusions(expected);
  if (job.status !== 'completed' || !allowed.has(job.conclusion)) {
    return `job-not-${[...allowed].join('-or-')}:${name}:`
      + `${job.status || 'unknown'}:${job.conclusion || 'unknown'}`;
  }
  return '';
}

function requireSafeAliasedJobConclusion(byName, logicalName, expected) {
  const aliases = CI_JOB_ALIASES[logicalName] || [logicalName];
  const matches = aliases.flatMap((name) => (
    (byName.get(name) || []).map((job) => ({ name, job }))
  ));
  if (matches.length !== 1) {
    return matches.length === 0
      ? `missing-job:${logicalName}`
      : `duplicate-job:${logicalName}`;
  }
  const { name, job } = matches[0];
  const allowed = safeConclusions(expected);
  if (job.status !== 'completed' || !allowed.has(job.conclusion)) {
    return `job-not-${[...allowed].join('-or-')}:${name}:`
      + `${job.status || 'unknown'}:${job.conclusion || 'unknown'}`;
  }
  return '';
}

function exactStep(job, name) {
  const entries = job.steps.filter((step) => step.name === name);
  if (entries.length !== 1) {
    return { error: entries.length === 0 ? `missing-step:${name}` : `duplicate-step:${name}` };
  }
  return { step: entries[0] };
}

function requireStepConclusion(job, name, conclusion) {
  const entries = job.steps.filter((step) => step.name === name);
  if (entries.length !== 1) {
    return entries.length === 0 ? `missing-step:${name}` : `duplicate-step:${name}`;
  }
  const step = entries[0];
  if (step.status !== 'completed' || step.conclusion !== conclusion) {
    return `step-not-${conclusion}:${name}:${step.status || 'unknown'}:${step.conclusion || 'unknown'}`;
  }
  return '';
}

function preflightFastPass(byName, jobName, checkoutStepName) {
  const resolved = exactJob(byName, jobName);
  if (resolved.error) return { error: resolved.error };
  const failure = requireJobConclusion(byName, jobName, 'success');
  if (failure) return { error: failure };
  const checkout = resolved.job.steps.filter((step) => step.name === checkoutStepName);
  if (checkout.length !== 1) {
    return { error: checkout.length === 0
      ? `missing-step:${checkoutStepName}`
      : `duplicate-step:${checkoutStepName}` };
  }
  if (checkout[0].status !== 'completed') {
    return { error: `step-not-completed:${checkoutStepName}:${checkout[0].status || 'unknown'}` };
  }
  if (!new Set(['success', 'failure', 'skipped']).has(checkout[0].conclusion)) {
    return { error: `step-invalid:${checkoutStepName}:${checkout[0].conclusion || 'unknown'}` };
  }
  return { fastPass: checkout[0].conclusion === 'skipped' };
}

function auditCi(policy, jobs) {
  const byName = normalizedJobs(jobs);
  const preflight = preflightFastPass(
    byName,
    'CI preflight',
    'Check out trusted CI impact classifier',
  );
  if (preflight.error) return preflight.error;
  const aggregateFailure = requireJobConclusion(byName, 'Build & Test', 'success');
  if (aggregateFailure) return aggregateFailure;

  const laneJobs = [...CI_RUST_JOBS, CI_NATIVE_JOB, ...CI_FRONTEND_JOBS];
  if (preflight.fastPass) {
    if (
      policy.enforcement_surface_changed === 'true'
      && policy.trusted_review_fast_pass !== 'true'
    ) return 'fast-pass-with-enforcement-change:CI';
    for (const name of laneJobs) {
      const failure = requireSafeAliasedJobConclusion(byName, name, 'skipped');
      if (failure) return failure;
    }
    return '';
  }

  const rustConclusion = policy.classification.rust_required === 'true' ? 'success' : 'skipped';
  for (const name of CI_RUST_JOBS) {
    const failure = requireSafeAliasedJobConclusion(byName, name, rustConclusion);
    if (failure) return failure;
  }
  const nativeConclusion = policy.classification.native_skia_required === 'true' ? 'success' : 'skipped';
  const nativeFailure = requireSafeJobConclusion(byName, CI_NATIVE_JOB, nativeConclusion);
  if (nativeFailure) return nativeFailure;

  const frontendExpected = {
    none: ['skipped', 'skipped'],
    unit: ['success', 'skipped'],
    package: ['skipped', 'success'],
  }[policy.classification.frontend_mode];
  for (let index = 0; index < CI_FRONTEND_JOBS.length; index += 1) {
    const failure = requireSafeJobConclusion(
      byName,
      CI_FRONTEND_JOBS[index],
      frontendExpected[index],
    );
    if (failure) return failure;
  }
  return '';
}

function auditCodeql(policy, jobs) {
  const byName = normalizedJobs(jobs);
  const preflight = preflightFastPass(
    byName,
    'CodeQL preflight',
    'Check out trusted CodeQL impact classifier',
  );
  if (preflight.error) return preflight.error;
  if (preflight.fastPass) {
    if (
      policy.enforcement_surface_changed === 'true'
      && policy.trusted_review_fast_pass !== 'true'
    ) return 'fast-pass-with-enforcement-change:CodeQL';
    const templateName = 'Analyze (${{ matrix.language }})';
    const templateEntries = byName.get(templateName) || [];
    const expandedCount = Object.values(CODEQL_JOBS)
      .reduce((count, name) => count + (byName.get(name) || []).length, 0);
    if (templateEntries.length === 1 && expandedCount === 0) {
      const failure = requireJobConclusion(byName, templateName, 'skipped');
      if (failure) return failure;
    } else if (templateEntries.length === 0 && expandedCount === CODEQL_LANGUAGE_ORDER.length) {
      for (const name of Object.values(CODEQL_JOBS)) {
        const failure = requireJobConclusion(byName, name, 'skipped');
        if (failure) return failure;
      }
    } else {
      return `invalid-fast-pass-matrix:template=${templateEntries.length}:expanded=${expandedCount}`;
    }
    return '';
  }

  const selected = new Set(
    policy.classification.codeql_languages === 'none'
      ? []
      : policy.classification.codeql_languages.split(','),
  );
  for (const language of CODEQL_LANGUAGE_ORDER) {
    const name = CODEQL_JOBS[language];
    const failure = requireJobConclusion(byName, name, 'success');
    if (failure) return failure;
    const job = exactJob(byName, name).job;
    if (selected.has(language)) {
      const analyzeFailure = requireStepConclusion(job, 'Perform CodeQL Analysis', 'success');
      if (analyzeFailure) return `${name}:${analyzeFailure}`;
      const skipFailure = requireStepConclusion(job, 'Skip unselected language', 'skipped');
      if (skipFailure) return `${name}:${skipFailure}`;
    } else {
      const skipResolved = exactStep(job, 'Skip unselected language');
      if (skipResolved.error) return `${name}:${skipResolved.error}`;
      const analyzeResolved = exactStep(job, 'Perform CodeQL Analysis');
      if (analyzeResolved.error) return `${name}:${analyzeResolved.error}`;
      const skipStep = skipResolved.step;
      const analyzeStep = analyzeResolved.step;
      if (skipStep.status !== 'completed' || analyzeStep.status !== 'completed') {
        return `${name}:unselected-steps-not-completed:`
          + `${skipStep.status || 'unknown'}:${analyzeStep.status || 'unknown'}`;
      }
      const noOp = skipStep.conclusion === 'success' && analyzeStep.conclusion === 'skipped';
      const safeFull = skipStep.conclusion === 'skipped' && analyzeStep.conclusion === 'success';
      if (!noOp && !safeFull) {
        return `${name}:unsafe-unselected-execution:`
          + `${skipStep.conclusion || 'unknown'}:${analyzeStep.conclusion || 'unknown'}`;
      }
    }
  }
  return '';
}

function auditRenderDiff(policy, jobs) {
  const byName = normalizedJobs(jobs);
  const preflight = preflightFastPass(
    byName,
    'Render Diff preflight',
    'Check out trusted CI impact classifier',
  );
  if (preflight.error) return preflight.error;
  if (
    preflight.fastPass
    && policy.enforcement_surface_changed === 'true'
    && policy.trusted_review_fast_pass !== 'true'
  ) {
    return 'fast-pass-with-enforcement-change:Render Diff';
  }
  const conclusion = preflight.fastPass || policy.classification.render_required !== 'true'
    ? 'skipped'
    : 'success';
  return requireSafeJobConclusion(byName, 'Canvas visual diff', conclusion);
}

function auditReviewReuse(input = {}) {
  const policy = input.policy;
  const reuse = input.reviewReuse || {};
  const candidateSha = String(reuse.candidateSha || '');
  if (!policy || policy.enforcement_surface_changed !== 'true') {
    return { eligible: false, candidateSha: '', reason: 'review-reuse-not-required' };
  }
  if (String(input.forceFullReason || '')) {
    return { eligible: false, candidateSha: '', reason: 'review-reuse-trusted-input-incomplete' };
  }
  if (policy.head_trust !== 'same-repository') {
    return { eligible: false, candidateSha: '', reason: 'review-reuse-requires-same-repository' };
  }
  if (policy.decision === 'blocked') {
    return { eligible: false, candidateSha: '', reason: 'review-reuse-policy-blocked' };
  }
  if (!reuse.lineageEligible || !/^[0-9a-f]{40}$/.test(candidateSha)) {
    return {
      eligible: false,
      candidateSha: '',
      reason: String(reuse.reason || 'review-reuse-lineage-unavailable'),
    };
  }
  if (reuse.baseMergeBridge && reuse.mergeTreeVerified !== true) {
    return { eligible: false, candidateSha: '', reason: 'review-reuse-merge-tree-unverified' };
  }

  for (const workflow of WORKFLOW_ORDER) {
    if (policy.expected_workflows[workflow] !== 'true') continue;
    const evidence = reuse.workflows?.[workflow];
    if (!evidence?.run) {
      return { eligible: false, candidateSha: '', reason: `review-reuse-missing-workflow:${workflow}` };
    }
    const run = evidence.run;
    const runCreatedAt = Date.parse(run.createdAt || 0);
    const pullCreatedAt = Date.parse(input.pullRequest?.createdAt || 0);
    if (
      String(run.name || '') !== workflow
      || String(run.path || '') !== WORKFLOW_PATHS[workflow]
      || String(run.event || '') !== 'pull_request'
      || String(run.headSha || '') !== candidateSha
      || String(run.headBranch || '') !== String(input.pullRequest?.headBranch || '')
      || String(run.headRepository || '') !== String(input.pullRequest?.headRepository || '')
      || !Array.isArray(run.pullNumbers)
      || !run.pullNumbers.map(Number).includes(Number(input.pullRequest?.number || 0))
      || !Number.isFinite(runCreatedAt)
      || !Number.isFinite(pullCreatedAt)
      || runCreatedAt < pullCreatedAt
      || String(run.status || '') !== 'completed'
      || String(run.conclusion || '') !== 'success'
      || evidence.jobsCollected !== true
    ) {
      return { eligible: false, candidateSha: '', reason: `review-reuse-invalid-workflow:${workflow}` };
    }
    const failure = workflow === 'CI'
      ? auditCi(policy, evidence.jobs)
      : workflow === 'CodeQL'
        ? auditCodeql(policy, evidence.jobs)
        : auditRenderDiff(policy, evidence.jobs);
    if (failure) {
      return { eligible: false, candidateSha: '', reason: `review-reuse-${workflow}:${failure}` };
    }
    if (workflow === 'CodeQL') {
      const check = evidence.securityCheck;
      const runStartedAt = Date.parse(run.runStartedAt || 0);
      const checkStartedAt = Date.parse(check?.startedAt || 0);
      if (
        check?.appSlug !== 'github-advanced-security'
        || check?.name !== 'CodeQL'
        || check?.headSha !== candidateSha
        || check?.status !== 'completed'
        || !new Set(['success', 'neutral']).has(check?.conclusion)
        || !Number.isFinite(runStartedAt)
        || !Number.isFinite(checkStartedAt)
        || checkStartedAt < runStartedAt
      ) {
        return { eligible: false, candidateSha: '', reason: 'review-reuse-invalid-security-check:CodeQL' };
      }
    }
  }
  return { eligible: true, candidateSha, reason: 'trusted-review-only-full-candidate' };
}

function auditPolicyRuns(input = {}) {
  const policy = input.policy || determinePolicy(input);
  const currentHeadSha = String(input.currentHeadSha || input.pullRequest?.headSha || '');
  if (!currentHeadSha || currentHeadSha !== String(input.pullRequest?.headSha || '')) {
    return { publish: 'false', conclusion: 'failure', reason: 'stale-or-unresolved-head' };
  }
  if (policy.decision === 'blocked') {
    return { publish: 'true', conclusion: 'failure', reason: policy.reason };
  }

  const pending = [];
  for (const workflow of WORKFLOW_ORDER) {
    if (policy.expected_workflows[workflow] !== 'true') continue;
    const evidence = input.workflows?.[workflow];
    if (!evidence?.run) {
      pending.push(`missing-workflow:${workflow}`);
      continue;
    }
    const run = evidence.run;
    if (
      String(run.name || '') !== workflow
      || String(run.path || '') !== WORKFLOW_PATHS[workflow]
      || String(run.event || '') !== 'pull_request'
      || String(run.headSha || '') !== currentHeadSha
      || String(run.headBranch || '') !== String(input.pullRequest?.headBranch || '')
      || String(run.headRepository || '') !== String(input.pullRequest?.headRepository || '')
      || (
        Array.isArray(run.pullNumbers)
        && run.pullNumbers.length > 0
        && !run.pullNumbers.map(Number).includes(Number(input.pullRequest?.number || 0))
      )
      || (
        Array.isArray(run.baseShas)
        && run.baseShas.length > 0
        && !run.baseShas.map(String).includes(String(input.pullRequest?.baseSha || ''))
      )
    ) {
      return { publish: 'true', conclusion: 'failure', reason: `workflow-identity-mismatch:${workflow}` };
    }
    if (String(run.status || '') !== 'completed') {
      pending.push(`workflow-not-completed:${workflow}:${run.status || 'unknown'}`);
      continue;
    }
    if (String(run.conclusion || '') !== 'success') {
      return {
        publish: 'true',
        conclusion: 'failure',
        reason: `workflow-not-success:${workflow}:${run.conclusion || 'unknown'}`,
      };
    }
    if (evidence.jobsCollected !== true) {
      pending.push(`job-evidence-unavailable:${workflow}`);
      continue;
    }
    const failure = workflow === 'CI'
      ? auditCi(policy, evidence.jobs)
      : workflow === 'CodeQL'
        ? auditCodeql(policy, evidence.jobs)
        : auditRenderDiff(policy, evidence.jobs);
    if (failure) {
      return { publish: 'true', conclusion: 'failure', reason: `${workflow}:${failure}` };
    }
  }
  if (pending.length > 0) {
    return { publish: 'true', conclusion: 'pending', reason: pending.join('|') };
  }
  return { publish: 'true', conclusion: 'success', reason: 'audit:stage3-5-truth-table' };
}

function parseCliArgs(argv) {
  const args = { input: '', githubOutput: '', output: '' };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--input') args.input = argv[++index] || '';
    else if (argument === '--github-output') args.githubOutput = argv[++index] || '';
    else if (argument === '--output') args.output = argv[++index] || '';
    else throw new Error(`unknown argument: ${argument}`);
  }
  if (!args.input) throw new Error('--input is required');
  return args;
}

function flatOutputs(policy, audit) {
  const output = {
    policy_version: policy.policy_version,
    policy_context: policy.policy_context,
    base_sha: policy.base_sha,
    decision: policy.decision,
    skip_eligible: policy.skip_eligible,
    head_trust: policy.head_trust,
    enforcement_surface_changed: policy.enforcement_surface_changed,
    trusted_review_fast_pass: policy.trusted_review_fast_pass,
    review_candidate_sha: policy.review_candidate_sha,
    review_reuse_reason: policy.review_reuse_reason,
    ci_run_expected: policy.expected_workflows.CI,
    codeql_run_expected: policy.expected_workflows.CodeQL,
    render_diff_run_expected: policy.expected_workflows['Render Diff'],
    reason: policy.reason,
    status_description: policy.status_description,
    rust_required: policy.classification.rust_required,
    frontend_mode: policy.classification.frontend_mode,
    render_required: policy.classification.render_required,
    native_skia_required: policy.classification.native_skia_required,
    codeql_languages: policy.classification.codeql_languages,
    classification_status: policy.classification.classification_status,
    classifier_version: policy.classification.classifier_version,
  };
  if (audit) {
    output.audit_publish = audit.publish;
    output.audit_conclusion = audit.conclusion;
    output.audit_reason = audit.reason;
  }
  return output;
}

function runCli(argv) {
  const args = parseCliArgs(argv);
  const input = JSON.parse(fs.readFileSync(args.input, 'utf8'));
  const policy = determinePolicy(input);
  const audit = input.workflows
    ? auditPolicyRuns({ ...input, policy })
    : null;
  const result = { policy, audit };
  const output = flatOutputs(policy, audit);
  if (args.githubOutput) {
    fs.appendFileSync(
      args.githubOutput,
      `${Object.entries(output).map(([key, value]) => `${key}=${value}`).join('\n')}\n`,
      'utf8',
    );
  }
  if (args.output) fs.writeFileSync(args.output, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
  if (!args.githubOutput && !args.output) process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
  return result;
}

if (require.main === module) {
  try {
    runCli(process.argv.slice(2));
  } catch (error) {
    process.stderr.write(`ci-impact-policy: ${error.message}\n`);
    process.exitCode = 1;
  }
}

module.exports = {
  CI_AUDITED_JOB_IDS,
  CI_FRONTEND_JOBS,
  CI_JOB_ALIASES,
  CI_NATIVE_JOB,
  CI_PUSH_PATHS_IGNORE,
  CI_PULL_REQUEST_PATHS_IGNORE,
  CI_RUST_JOBS,
  CODEQL_JOBS,
  CODEQL_PUSH_PATHS_IGNORE,
  CODEQL_PULL_REQUEST_PATHS_IGNORE,
  DEPLOY_PAGES_PUSH_PATHS_IGNORE,
  POLICY_CONTEXT,
  POLICY_VERSION,
  RENDER_DIFF_PULL_REQUEST_PATHS,
  WORKFLOW_ORDER,
  WORKFLOW_PATHS,
  auditPolicyRuns,
  auditReviewReuse,
  changesEnforcementSurface,
  determinePolicy,
  expectedWorkflowMap,
  fullClassification,
  parseStatusDescription,
  runCli,
  selectReviewOnlyCandidate,
  selectLatestWorkflowRun,
  statusDescription,
  workflowRunExpected,
};
