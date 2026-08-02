/*
 * 대량 처리 — 부분 실패를 잃지 않고 집계한다.
 *
 * 배치는 NDJSON 한 줄 = 문서 하나다. 한 줄이 실패해도 나머지는 계속 나온다.
 * 그래서 실패는 예외로 올라오지 않는다 — 줄 안의 `error` 필드로 온다.
 * 이 필드를 안 보면 실패가 조용히 사라진다.
 *
 *   npx tsx examples/05-batch-pipeline.ts 폴더 [스레드수]
 */

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import * as rhwp from '../src/index.js';

/**
 * 배치 한 줄. 성공/실패가 같은 스트림에 섞여 온다.
 *
 * `rhwp.BatchRecord` 를 확장하면 `batch<T>()` 에 그대로 넘길 수 있다 — 반환값을
 * 캐스팅으로 우겨넣지 않는다. 캐스팅은 "이 모양일 것"이라는 짐작을 컴파일러의
 * 보증으로 둔갑시킨다. 확장은 `error`·`source` 같은 배치 공통 계약을 물려받으므로
 * 실패 판별을 빼먹을 수 없다.
 */
interface TextRecord extends rhwp.BatchRecord {
  readonly pageCount?: number;
}

/** 하위 폴더까지 훑는다. Node 버전별 `readdirSync` 옵션 차이를 피하려고 직접 돈다. */
function walk(dir: string): string[] {
  const out: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walk(full));
    else out.push(full);
  }
  return out;
}

async function main(folder: string, threads: number | undefined): Promise<number> {
  const paths = walk(folder)
    .filter((p) => ['.hwp', '.hwpx'].includes(path.extname(p).toLowerCase()))
    .sort();

  if (paths.length === 0) {
    console.log(`처리할 문서가 없습니다: ${folder}`);
    return 1;
  }
  console.log(`${paths.length}개 문서 처리 중…`);

  // batch 는 실패를 예외로 올리지 않는다. 첫 손상 파일에서 멈추면 나머지 999개를
  // 다시 돌려야 하기 때문이다. 대신 실패도 줄로 돌려준다.
  // 문서 수가 아주 많아 메모리에 다 담기 곤란하면 `rhwp.iterNdjson` 으로 흘려 받는다.
  //
  // `threads` 는 **낮추는 쪽으로** 쓰는 값이다. 기본은 CPU 코어 수인데, 공유 CI
  // 러너나 메모리가 빠듯한 컨테이너에서 코어 수만큼 문서를 동시에 펼치면 OOM 으로
  // 끝난다. 그때는 처리량보다 완주가 중요하다.
  const results = await rhwp.batch<TextRecord>('export-text', paths, { threads });

  const failures = results.filter((r) => r.error !== undefined);
  const successes = results.filter((r) => r.error === undefined);

  console.log(`\n성공 ${successes.length} / 실패 ${failures.length}`);
  console.log(`총 쪽수: ${successes.reduce((sum, r) => sum + (r.pageCount ?? 0), 0)}`);

  for (const r of failures.slice(0, 10)) {
    console.log(`  실패: ${r.source} — ${r.error}`);
  }
  if (failures.length > 10) {
    console.log(`  … 외 ${failures.length - 10}건`);
  }

  // 부분 실패는 파이프라인의 고장이 아니라 결과 데이터다. 종료 코드로 뭉개지 않고
  // 목록으로 돌려준다 — 어느 문서가 왜 실패했는지가 "몇 건 실패"보다 쓸모 있다.
  return 0;
}

const argv = process.argv.slice(2);
const [folder, threadArg] = argv;
if (argv.length < 1 || argv.length > 2 || folder === undefined) {
  console.error('사용법: npx tsx examples/05-batch-pipeline.ts 폴더 [스레드수]');
  process.exit(2);
}

const threads = threadArg === undefined ? undefined : Number(threadArg);
if (threads !== undefined && (!Number.isInteger(threads) || threads < 1)) {
  console.error(`스레드수는 1 이상의 정수여야 합니다: ${threadArg}`);
  process.exit(2);
}

process.exit(await main(folder, threads));
