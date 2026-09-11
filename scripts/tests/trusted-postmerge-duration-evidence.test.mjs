import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { selectDurationArtifacts, decodeDurationArtifact, validateDurationReports } from "../trusted-postmerge-duration-evidence.mjs";
import { refreshDurationPolicy } from "../refresh-nextest-target-duration-policy.mjs";
import { durationFixture, reportZip, zipEntries } from "./helpers/postmerge-duration-fixtures.mjs";

for (const options of [{ fork: true }, { fork: false }, { fork: false, legacy: true }]) {
  test(`duration provenance 정상 발행·소비: ${JSON.stringify(options)}`, () => {
    const f = durationFixture(options);
    const selected = selectDurationArtifacts(f.artifacts, f.run, f.pr, f.repositoryId);
    assert.equal(selected.length, 3);
    assert.ok(selected.every(s => s.legacy === Boolean(options.legacy)));
    const reports = selected.map(s => decodeDurationArtifact(reportZip(f.reports.find(r => r.archive_label === s.label)), s.label));
    const normalized = validateDurationReports(reports, f.context);
    assert.deepEqual(normalized.map(r => r.archive_label), ["b", "c", "d"]);
    assert.ok(normalized.every(r => r.run_id === "123" && r.sha === f.context.testedMergeSha));
  });
}

for (const [name, mutate] of Object.entries({
  "archive 누락": f => f.artifacts.pop(),
  "archive 중복": f => f.artifacts.push({ ...f.artifacts[0] }),
  "ID 중복": f => { f.artifacts[1].id = f.artifacts[0].id; },
  "만료": f => { f.artifacts[0].expired = true; },
  "만료 여부 누락": f => { delete f.artifacts[0].expired; },
  "크기 초과": f => { f.artifacts[0].size_in_bytes = 8 * 1024 * 1024 + 1; },
  "음수 크기": f => { f.artifacts[0].size_in_bytes = -1; },
  "다른 run": f => { f.artifacts[0].workflow_run.id++; },
  "다른 head": f => { f.artifacts[0].workflow_run.head_sha = "f".repeat(40); },
  "다른 fork": f => { f.artifacts[0].workflow_run.head_repository_id++; },
  "다른 upstream": f => { f.artifacts[0].workflow_run.repository_id++; },
  "fork 자체 Actions": f => { f.run.repository.id = 20; },
  "실패 run": f => { f.run.conclusion = "failure"; },
  "취소 run": f => { f.run.conclusion = "cancelled"; },
  "pending run": f => { f.run.status = "in_progress"; },
  "이전 attempt 이름": f => { f.run.run_attempt++; },
  "이전 attempt 생성 시각": f => { f.artifacts[0].created_at = "2020-01-01T00:00:00Z"; },
  "merge 이후 artifact": f => { f.artifacts[0].created_at = "2027-01-01T00:00:00Z"; },
})) {
  test(`duration artifact 거부: ${name}`, () => {
    const f = durationFixture(); mutate(f);
    assert.throws(() => selectDurationArtifacts(f.artifacts, f.run, f.pr, f.repositoryId));
  });
}

test("fork legacy와 same-repository 이전 attempt는 허용하지 않는다", () => {
  for (const options of [{ fork: true, legacy: true }, { fork: false, legacy: true, attempt: 2 }]) {
    const f = durationFixture(options);
    assert.throws(() => selectDurationArtifacts(f.artifacts, f.run, f.pr, f.repositoryId));
  }
});

for (const [name, mutate] of Object.entries({
  "run": f => { f.reports[0].run_id = "124"; },
  "attempt": f => { f.reports[0].run_attempt = "1"; },
  "PR": f => { f.reports[0].pull_number = "43"; },
  "upstream ID": f => { f.reports[0].repository_id = "11"; },
  "upstream 이름": f => { f.reports[0].repository = "attacker/rhwp"; },
  "fork ID": f => { f.reports[0].head_repository_id = "21"; },
  "head SHA": f => { f.reports[0].head_sha = "a".repeat(40); },
  "head branch": f => { f.reports[0].head_ref = "other"; },
  "테스트 merge SHA": f => { f.reports[0].sha = f.run.head_sha; },
  "merge ref": f => { f.reports[0].ref = "refs/heads/devel"; },
  "누락 archive": f => f.reports.pop(),
  "중복 archive": f => { f.reports[1].archive_label = "b"; },
  "없는 schema": f => { f.reports[0].schema_version = 99; },
  "빈 targets": f => { f.reports[0].targets = {}; },
  "음수 duration": f => { f.reports[0].targets.regression_suite_b = -1; },
  "무한 duration": f => { f.reports[0].targets.regression_suite_b = Infinity; },
  "NaN duration": f => { f.reports[0].targets.regression_suite_b = NaN; },
  "초과 duration": f => { f.reports[0].targets.regression_suite_b = 86401; },
  "문자 duration": f => { f.reports[0].targets.regression_suite_b = "1"; },
  "경로 식별자": f => { f.reports[0].cases = { "../../evil": 1 }; },
  "prototype 식별자": f => { f.reports[0].cases = { constructor: 1 }; },
  "알 수 없는 target": f => { f.reports[0].test_cases = { "other::test": 1 }; },
  "알 수 없는 case": f => { f.reports[0].test_cases = { "regression_suite_b::other::test": 1 }; },
  "서로 중복 target": f => { f.reports[1].targets.regression_suite_b = 1; },
  "서로 중복 case": f => { f.reports[1].cases.case_b = 1; },
})) {
  test(`duration JSON 거부: ${name}`, () => {
    const f = durationFixture(); mutate(f);
    assert.throws(() => validateDurationReports(f.reports, f.context));
  });
}

test("정규화 report는 알 수 없는 필드를 버리고 policy에 provenance를 보존한다", () => {
  const f = durationFixture(); f.reports[0].command = "do not execute";
  const normalized = validateDurationReports(f.reports, f.context);
  assert.equal(normalized[0].command, undefined);
  const policy = JSON.parse(readFileSync(new URL("../../tests/suites/nextest-target-duration-policy.json", import.meta.url), "utf8"));
  const updated = refreshDurationPolicy(policy, normalized);
  assert.equal(updated.measurement_sources.b.run_attempt, "2");
  assert.equal(updated.measurement_sources.b.head_repository_id, "20");
  assert.equal(updated.measurement_sources.b.pull_number, "42");
});

for (const [name, entries] of [
  ["추가 파일", [{ name: "target-durations-b.json", content: "{}" }, { name: "extra", content: "x" }]],
  ["경로 탈출", [{ name: "../target-durations-b.json", content: "{}" }]],
  ["절대 경로", [{ name: "/tmp/target-durations-b.json", content: "{}" }]],
  ["심볼릭 링크", [{ name: "target-durations-b.json", content: "elsewhere", mode: 0o120777 }]],
  ["디렉터리", [{ name: "target-durations-b.json/", content: "" }]],
  ["중복 JSON key", [{ name: "target-durations-b.json", content: '{"run_id":"123","run_id":"456"}' }]],
  ["JSON NaN", [{ name: "target-durations-b.json", content: '{"number":NaN}' }]],
  ["잘못된 JSON", [{ name: "target-durations-b.json", content: "not JSON" }]],
]) {
  test(`원본 ZIP 거부: ${name}`, () => assert.throws(() => decodeDurationArtifact(zipEntries(entries), "b")));
}
test("ZIP 암호화 flag, 크기 제한, 큰 압축 해제 결과를 거부한다", () => {
  const encrypted = reportZip(durationFixture().reports[0]);
  const central = encrypted.indexOf(Buffer.from([0x50, 0x4b, 0x01, 0x02]));
  encrypted.writeUInt16LE(encrypted.readUInt16LE(central + 8) | 1, central + 8);
  assert.throws(() => decodeDurationArtifact(encrypted, "b"));
  assert.throws(() => decodeDurationArtifact(Buffer.alloc(8 * 1024 * 1024 + 1), "b"));
  assert.throws(() => decodeDurationArtifact(zipEntries([{ name: "target-durations-b.json", content: "x".repeat(8 * 1024 * 1024 + 1) }]), "b"));
});

const workflow = readFileSync(new URL("../../.github/workflows/trusted-postmerge-ci-reuse.yml", import.meta.url), "utf8");
const finalize = workflow.split("- name: Finalize reuse after validated evidence publication")[1]
  .split("run: |\n")[1].split("\n").map(line => line.replace(/^ {10}/, "")).join("\n");
for (const failed of [null, "B", "C", "D"]) {
  test(`정규화 artifact 발행 ${failed || "모두 성공"} 최종 reuse 계약`, t => {
    const directory = mkdtempSync(path.join(tmpdir(), "duration-finalize-"));
    t.after(() => rmSync(directory, { recursive: true, force: true }));
    const output = path.join(directory, "output");
    execFileSync("bash", ["-e", "-c", finalize], { env: { ...process.env, GITHUB_OUTPUT: output,
      REUSE: "true", REASON: "verified", SOURCE_RUN_ID: "123", REFRESH: "true",
      PUBLISH_B: failed === "B" ? "failure" : "success", PUBLISH_C: failed === "C" ? "skipped" : "success",
      PUBLISH_D: failed === "D" ? "cancelled" : "success" } });
    const result = Object.fromEntries(readFileSync(output, "utf8").trim().split("\n").map(line => line.split("=")));
    assert.equal(result.reuse, failed ? "false" : "true");
    assert.equal(result.refresh_duration_data, failed ? "false" : "true");
    assert.equal(result.source_run_id, failed ? "" : "123");
  });
}
