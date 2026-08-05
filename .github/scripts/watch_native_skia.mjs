#!/usr/bin/env node

import fs from "node:fs";
import { pathToFileURL } from "node:url";

export function nativeSkiaState(job) {
  if (!job || job.status !== "completed") return "pending";
  return job.conclusion === "success" ? "success" : "failure";
}

export function findNativeSkiaJob(jobs, name = "Native Skia tests") {
  return jobs.find((job) => job.name === name) ?? null;
}

function parseArgs(argv) {
  const args = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith("--") || value === undefined) {
      throw new Error("usage: watch_native_skia.mjs --process-group PID --abort-file FILE [--poll-ms N]");
    }
    args.set(key, value);
  }
  return args;
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

function processAlive(processGroup) {
  try {
    process.kill(-processGroup, 0);
    return true;
  } catch (error) {
    return error.code === "EPERM";
  }
}

async function fetchNativeSkiaState({ apiUrl, repository, runId, token }) {
  const response = await fetch(`${apiUrl}/repos/${repository}/actions/runs/${runId}/jobs?per_page=100`, {
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token}`,
      "X-GitHub-Api-Version": "2026-03-10",
    },
  });
  if (!response.ok) throw new Error(`GitHub Actions job query failed: HTTP ${response.status}`);
  const payload = await response.json();
  return nativeSkiaState(findNativeSkiaJob(payload.jobs ?? []));
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const processGroup = Number(args.get("--process-group"));
  const abortFile = args.get("--abort-file");
  const pollMs = Number(args.get("--poll-ms") ?? 15000);
  const repository = process.env.GITHUB_REPOSITORY;
  const runId = process.env.GITHUB_RUN_ID;
  const token = process.env.GITHUB_TOKEN;
  const apiUrl = process.env.GITHUB_API_URL ?? "https://api.github.com";
  if (!Number.isInteger(processGroup) || processGroup <= 0 || !abortFile || !repository || !runId || !token || !Number.isInteger(pollMs) || pollMs < 1000) {
    throw new Error("native Skia watcher requires a process group, abort file, GitHub Actions environment, and poll interval >= 1000 ms");
  }

  while (processAlive(processGroup)) {
    try {
      const state = await fetchNativeSkiaState({ apiUrl, repository, runId, token });
      if (state === "success") return;
      if (state === "failure") {
        fs.writeFileSync(abortFile, `${JSON.stringify({ reason: "native-skia-failed" })}\n`);
        process.kill(-processGroup, "SIGTERM");
        return;
      }
    } catch (error) {
      console.warn(`native Skia watcher retry: ${error.message}`);
    }
    await sleep(pollMs);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
