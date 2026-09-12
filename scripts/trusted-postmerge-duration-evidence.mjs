import { execFileSync } from "node:child_process";

const LIMIT = 8 * 1024 * 1024;
const LABELS = ["b", "c", "d"];
const ID = /^[1-9][0-9]*$/;
const SHA = /^[0-9a-f]{40}$/;
// JUnit preserves Rust Unicode identifiers. Keep this a data-only identifier
// path, not an arbitrary string: reject empty segments and invisible controls.
const NAME = /^(?!.*\p{Default_Ignorable_Code_Point})[_\p{XID_Start}][_\p{XID_Continue}]*(?:::[_\p{XID_Start}][_\p{XID_Continue}]*)*$/u;
const fail = (message) => { throw new Error(`duration-evidence: ${message}`); };
const record = (value) => value !== null && typeof value === "object" && !Array.isArray(value);

// The caller supplies GitHub API data, never values from the downloaded report.
export function selectDurationArtifacts(artifacts, run, pr, repositoryId) {
  if (!Array.isArray(artifacts) || !Number.isSafeInteger(run?.id) || run.id <= 0
    || !Number.isSafeInteger(run.run_attempt) || run.run_attempt <= 0
    || !Number.isSafeInteger(repositoryId) || repositoryId <= 0
    || pr?.base?.repo?.id !== repositoryId || run.repository?.id !== repositoryId
    || run.event !== "pull_request" || run.status !== "completed" || run.conclusion !== "success") {
    fail("invalid successful upstream run identity");
  }
  const started = Date.parse(run.run_started_at || run.created_at);
  const merged = Date.parse(pr.merged_at);
  if (!Number.isFinite(started) || !Number.isFinite(merged) || started > merged) fail("invalid run time");
  const prefix = `nextest-target-durations-${run.id}-attempt-${run.run_attempt}`;
  const useNew = artifacts.some((artifact) => artifact?.name?.startsWith(`${prefix}-`));
  const legacy = !useNew && pr.head?.repo?.id === repositoryId && run.run_attempt === 1;
  if (!useNew && !legacy) fail("attempt-bound B/C/D artifacts unavailable");
  const selected = LABELS.map((label) => {
    const name = `${legacy ? `nextest-target-durations-${run.id}` : prefix}-${label}`;
    const matches = artifacts.filter((artifact) => artifact?.name === name);
    if (matches.length !== 1) fail(`missing or duplicate artifact: ${label}`);
    const artifact = matches[0];
    const created = Date.parse(artifact.created_at);
    if (!Number.isSafeInteger(artifact.id) || artifact.id <= 0 || artifact.expired !== false
      || !Number.isSafeInteger(artifact.size_in_bytes) || artifact.size_in_bytes <= 0 || artifact.size_in_bytes > LIMIT
      || !Number.isFinite(created) || created < started || created > merged
      || (artifact.workflow_run && (artifact.workflow_run.id !== run.id
        || artifact.workflow_run.repository_id !== repositoryId
        || artifact.workflow_run.head_repository_id !== pr.head.repo.id
        || artifact.workflow_run.head_sha !== run.head_sha))) {
      fail(`expired, oversized, stale, or mismatched artifact: ${label}`);
    }
    return { label, artifact, legacy };
  });
  if (new Set(selected.map(({ artifact }) => artifact.id)).size !== 3) fail("duplicate artifact IDs");
  return selected;
}

// No extraction: reject additional entries, paths, links, encryption and zip bombs.
// object_pairs_hook also rejects duplicate JSON keys before JS can discard them.
export function decodeDurationArtifact(bytes, label) {
  if (!LABELS.includes(label)) fail("invalid archive label");
  const buffer = Buffer.from(bytes);
  if (buffer.length === 0 || buffer.length > LIMIT) fail("invalid ZIP size");
  const script = `
import io,json,stat,sys,zipfile
limit=${LIMIT}
def pairs(items):
    result={}
    for key,value in items:
        if key in result: raise ValueError('duplicate JSON key')
        result[key]=value
    return result
def constant(value): raise ValueError('non-finite JSON number')
with zipfile.ZipFile(io.BytesIO(sys.stdin.buffer.read(limit+1))) as archive:
    entries=archive.infolist()
    if len(entries)!=1: raise ValueError('expected one JSON entry')
    entry=entries[0]
    mode=(entry.external_attr >> 16) & 0xffff
    if (entry.filename!=sys.argv[1] or entry.is_dir() or entry.flag_bits & 1
        or stat.S_ISLNK(mode) or stat.S_IFMT(mode) not in (0,stat.S_IFREG)
        or entry.file_size<=0 or entry.file_size>limit
        or entry.compress_size<=0 or entry.compress_size>limit):
        raise ValueError('invalid ZIP entry')
    with archive.open(entry) as stream: data=stream.read(limit+1)
    if len(data)>limit or len(data)!=entry.file_size: raise ValueError('invalid expanded size')
    value=json.loads(data.decode('utf-8'),object_pairs_hook=pairs,parse_constant=constant)
    sys.stdout.write(json.dumps(value,allow_nan=False,separators=(',',':')))
`;
  return JSON.parse(execFileSync("python3", ["-c", script, `target-durations-${label}.json`], {
    input: buffer, timeout: 15000, maxBuffer: LIMIT * 4,
  }).toString("utf8"));
}

export function validateDurationReports(reports, { run, pr, repositoryId, testedMergeSha, legacy = false }) {
  if (!Array.isArray(reports) || reports.length !== 3 || !SHA.test(testedMergeSha || "")
    || !SHA.test(run?.head_sha || "") || !ID.test(String(run?.id || ""))
    || !Number.isSafeInteger(pr?.number) || pr.number <= 0
    || !Number.isSafeInteger(run.run_attempt) || run.run_attempt <= 0
    || (legacy && (pr.head?.repo?.id !== repositoryId || run.run_attempt !== 1))) fail("invalid report context");
  const labels = new Set();
  const keys = { targets: new Set(), cases: new Set(), test_cases: new Set() };
  const schemas = new Set();
  const output = reports.map((report) => {
    if (!record(report) || !LABELS.includes(report.archive_label) || labels.has(report.archive_label)
      || ![1, 2].includes(report.schema_version) || (!legacy && report.schema_version !== 2)) fail("invalid report schema or label");
    labels.add(report.archive_label);
    schemas.add(report.schema_version);
    if (report.run_id !== String(run.id) || report.ref !== `refs/pull/${pr.number}/merge`
      || report.sha !== testedMergeSha) fail("report run/ref/tested merge SHA mismatch");
    if (!legacy && (report.run_attempt !== String(run.run_attempt)
      || report.repository_id !== String(repositoryId) || report.repository !== run.repository.full_name
      || report.pull_number !== String(pr.number) || report.head_repository_id !== String(pr.head.repo.id)
      || report.head_sha !== run.head_sha || report.head_ref !== pr.head.ref)) fail("report PR/repository/attempt/head mismatch");
    const normalized = { schema_version: report.schema_version, archive_label: report.archive_label,
      run_id: report.run_id, ref: report.ref, sha: report.sha };
    if (!legacy) Object.assign(normalized, { run_attempt: report.run_attempt, repository_id: report.repository_id,
      repository: report.repository, pull_number: report.pull_number, head_repository_id: report.head_repository_id,
      head_sha: report.head_sha, head_ref: report.head_ref });
    for (const field of report.schema_version === 2 ? ["targets", "cases", "test_cases"] : ["targets"]) {
      if (!record(report[field])) fail(`invalid ${field} dictionary`);
      const entries = Object.entries(report[field]);
      if (!entries.length || entries.length > 50000) fail(`invalid ${field} count`);
      for (const [name, seconds] of entries) {
        if (name.length > 512 || !NAME.test(name) || ["__proto__", "constructor", "prototype"].includes(name)
          || !Number.isFinite(seconds) || seconds <= 0 || seconds > 86400 || keys[field].has(name)) {
          fail(`invalid or duplicate ${field} entry`);
        }
        keys[field].add(name);
      }
      normalized[field] = Object.fromEntries(entries);
    }
    if (report.schema_version === 2) {
      for (const name of Object.keys(normalized.test_cases)) {
        const separator = name.indexOf("::");
        if (separator < 1 || !Object.hasOwn(normalized.targets, name.slice(0, separator))) fail("unknown testcase target");
        const target = name.slice(0, separator);
        const test = name.slice(separator + 2);
        const sourceCase = target.startsWith("regression_suite_") ? test.split("::")[0] : target;
        if (!Object.hasOwn(normalized.cases, sourceCase)) fail("unknown testcase source case");
      }
    }
    return normalized;
  });
  if (labels.size !== 3 || schemas.size !== 1) fail("incomplete or mixed B/C/D schemas");
  return output.sort((a, b) => a.archive_label.localeCompare(b.archive_label));
}
