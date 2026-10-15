// 각 작업자는 기존 캡처 프로그램과 Chrome 하나를 재사용한다.
import { spawn } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { createInterface } from 'node:readline';

export async function runParallelRasterJobs(jobs, workers, script, chrome) {
  const directory = mkdtempSync(join(dirname(resolve(jobs[0].output)), '.webfont-workers-'));
  const count = Math.min(workers, jobs.length);
  const indices = new Map(jobs.map((job, index) => [resolve(job.output), index]));
  const pending = new Map();
  const seen = new Set();
  const children = [];
  let next = 0;
  let stopping = false;
  const stop = () => {
    stopping = true;
    for (const { child } of children) {
      if (child.exitCode === null && !child.stdin.destroyed) {
        child.stdin.write('stop\n');
      }
    }
  };
  const interrupt = () => stop();
  let control = '';
  const onControl = chunk => {
    control += chunk.toString('utf8');
    if (control.includes('\n')) {
      if (control.trim() === 'stop') stop();
      control = '';
    }
  };
  process.once('SIGINT', interrupt);
  process.once('SIGTERM', interrupt);
  process.stdin.on('data', onControl);
  process.stdin.unref?.();
  try {
    for (let worker = 0; worker < count; worker += 1) {
      const assigned = jobs.filter((_, index) => index % count === worker);
      const manifest = join(directory, `${worker}.json`);
      writeFileSync(manifest, JSON.stringify(assigned));
      const child = spawn(process.execPath, [script, '--batch-json', manifest,
        '--workers', '1', '--chrome', chrome], { stdio: ['pipe', 'pipe', 'inherit'] });
      // 중단 직전에 이미 종료한 자식의 stdin 오류도 다른 작업을 건드리지 않는다.
      child.stdin.on('error', () => {});
      const lines = createInterface({ input: child.stdout });
      let protocolError = null;
      lines.on('line', line => {
        try {
          const record = JSON.parse(line);
          const index = indices.get(record.output);
          const job = jobs[index];
          if (!job || index % count !== worker || seen.has(index)
            || record.input !== resolve(job.input) || record.zoom !== job.zoom) {
            throw new Error('작업자의 쪽 소유·중복·입력·배율이 요청과 다릅니다.');
          }
          seen.add(index);
          pending.set(index, record);
          // 쪽별 checkpoint 소비자는 병렬 완료 순서 대신 원래 순서를 받는다.
          while (pending.has(next)) {
            console.log(JSON.stringify(pending.get(next)));
            pending.delete(next);
            next += 1;
          }
        } catch (error) {
          protocolError = error;
          stop();
        }
      });
      const done = new Promise((accept, reject) => {
        child.once('error', reject);
        child.once('close', code => {
          lines.close();
          if (protocolError) reject(protocolError);
          else if (code !== 0) reject(new Error(`캡처 작업자 ${worker + 1} 종료: ${code}`));
          else accept();
        });
      });
      children.push({ child, done });
      // 먼저 실패한 작업자가 있어도 다른 작업자의 rejection은 수집한다.
      done.catch(() => stop());
    }
    await Promise.all(children.map(({ done }) => done));
    if (stopping || next !== jobs.length) throw new Error('병렬 캡처가 중단되거나 쪽이 누락됐습니다.');
  } finally {
    stop();
    await Promise.allSettled(children.map(({ done }) => done));
    process.removeListener('SIGINT', interrupt);
    process.removeListener('SIGTERM', interrupt);
    process.stdin.removeListener('data', onControl);
    process.stdin.pause();
    rmSync(directory, { recursive: true, force: true });
  }
}
