import { execFileSync } from "node:child_process";

export function zipEntries(entries) {
  return execFileSync("python3", ["-c", `
import io,json,sys,zipfile
output=io.BytesIO()
with zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED) as archive:
    for entry in json.load(sys.stdin):
        info=zipfile.ZipInfo(entry['name'])
        info.compress_type=zipfile.ZIP_DEFLATED
        info.external_attr=entry.get('mode',0o100644)<<16
        archive.writestr(info,entry['content'])
sys.stdout.buffer.write(output.getvalue())
`], { input: JSON.stringify(entries), maxBuffer: 20 * 1024 * 1024 });
}

export function reportZip(report, label = report.archive_label) {
  return zipEntries([{ name: `target-durations-${label}.json`, content: JSON.stringify(report) }]);
}

export function durationFixture({ fork = true, legacy = false, attempt = legacy ? 1 : 2 } = {}) {
  const repositoryId = 10;
  const run = { id: 123, run_attempt: attempt, head_sha: "2".repeat(40),
    repository: { id: 10, full_name: "edwardkim/rhwp" },
    event: "pull_request", status: "completed", conclusion: "success",
    created_at: "2026-09-06T11:04:00Z", run_started_at: "2026-09-06T11:04:00Z",
    updated_at: "2026-09-06T11:27:30Z" };
  const pr = { number: 42, merged_at: "2026-09-06T11:29:16Z",
    base: { repo: run.repository, ref: "devel" },
    head: { ref: "feature", repo: { id: fork ? 20 : 10, full_name: fork ? "contributor/rhwp" : "edwardkim/rhwp" } } };
  const testedMergeSha = "3".repeat(40);
  const reports = ["b", "c", "d"].map(label => ({ schema_version: 2, archive_label: label,
    run_id: String(run.id), run_attempt: String(attempt), repository: run.repository.full_name,
    repository_id: "10", head_repository_id: String(pr.head.repo.id), pull_number: "42",
    head_sha: run.head_sha, head_ref: pr.head.ref, ref: "refs/pull/42/merge", sha: testedMergeSha,
    targets: { [`regression_suite_${label}`]: 1 }, cases: { [`case_${label}`]: 1 },
    test_cases: { [`regression_suite_${label}::case_${label}::works`]: 1 } }));
  const artifacts = reports.map((report, index) => ({ id: index + 100, expired: false,
    name: `nextest-target-durations-123${legacy ? "" : `-attempt-${attempt}`}-${report.archive_label}`,
    created_at: "2026-09-06T11:20:00Z", size_in_bytes: 1024,
    workflow_run: { id: run.id, repository_id: 10, head_repository_id: pr.head.repo.id, head_sha: run.head_sha } }));
  return { reports, artifacts, run, pr, repositoryId,
    context: { run, pr, repositoryId, testedMergeSha, legacy } };
}
