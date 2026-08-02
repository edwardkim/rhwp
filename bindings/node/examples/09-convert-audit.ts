/*
 * 변환 감사 — 폴더 전체를 반대 포맷으로 바꾸고 손실을 집계한다.
 *
 * 판정은 예외가 아니라 데이터다. 실패한 개수만 세는 게 아니라 **무엇이
 * 달라졌는지**를 범주별로 모은다. "997/1000 성공"은 나머지 3개를 고칠 단서를
 * 하나도 주지 않는다.
 *
 *   npx tsx examples/09-convert-audit.ts 입력폴더 출력폴더
 */

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import * as rhwp from '../src/index.js';

/** 감사 결과 버킷. 이름이 곧 사람이 볼 보고서의 항목이다. */
type Bucket = '통과' | '차이' | '재파싱실패' | '오류';
const BUCKETS: Bucket[] = ['통과', '차이', '재파싱실패', '오류'];
/** 보고서에서 먼저 봐야 할 순서 — 나쁜 것부터. */
const PROBLEM_BUCKETS: Bucket[] = ['오류', '재파싱실패', '차이'];

function walk(dir: string): string[] {
  const out: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walk(full));
    else out.push(full);
  }
  return out;
}

/**
 * 문서 하나를 반대 포맷으로 옮긴다.
 *
 * 두 축 모두 산출 경로가 **위치 인자**로 넘어간다 — 이 명령들은 `-o` 를 모른다
 * ("알 수 없는 옵션: -o", exit 2). 바인딩이 `out` 옵션을 받아 위치 인자로 옮겨
 * 주므로 호출부는 다른 명령과 같은 모양을 유지한다.
 *
 * 다른 점 하나: `exportHwpx` 는 `out` 을 생략하면 `<입력 stem>.hwpx` 로 떨어지지만
 * `convert` 는 **기본 경로가 없어 필수**다. 빠뜨리면 바인딩이 프로세스를 띄우기
 * 전에 `UsageError` 로 무엇이 빠졌는지 이름으로 알려 준다. 감사에서는 산출물을
 * 한곳에 모아야 하므로 어느 쪽이든 항상 명시한다.
 */
async function convertOne(source: string, targetDir: string): Promise<{
  readonly target: string;
  readonly result: rhwp.Envelope;
}> {
  const stem = path.basename(source, path.extname(source));
  const toHwpx = path.extname(source).toLowerCase() === '.hwp';
  const target = path.join(targetDir, `${stem}.${toHwpx ? 'hwpx' : 'hwp'}`);

  // throwOnVerdict 를 켜지 않는다. 검증 실패는 이 감사의 **결과물**이지 중단 사유가 아니다.
  const result = toHwpx
    ? await rhwp.exportHwpx(source, { out: target, verify: true })
    : await rhwp.convert(source, { out: target, verify: true });
  return { target, result };
}

/**
 * 차이가 난 문서에서 **어느 구역**이 원인인지 좁힌다.
 *
 * 문서 전체 비교는 "몇 건 다르다"까지만 알려준다. `section` 으로 범위를 좁히면
 * 차이가 한 구역에 몰려 있는지 전역인지가 드러나고, 그게 곧 이분법의 첫 갈래다.
 * (구역이 아니라 문단으로 더 좁히려면 `paragraph` 를 쓴다 — 쪽이 아니라 문단이다.)
 *
 * 비용이 구역 수만큼 늘어나므로 감사에서는 **첫 문제 문서 한 건에만** 쓴다.
 */
async function narrowBySection(a: string, b: string): Promise<void> {
  const sections = (await rhwp.info(a)).getOr<number>('sections', 0);
  if (sections <= 1) return;

  console.log(`  구역별 차이 (${path.basename(a)}):`);
  for (let section = 0; section < sections; section += 1) {
    const scoped = await rhwp.irDiff(a, b, { section });
    console.log(`    구역 ${section}: ${scoped.getOr<number>('diffCount', 0)}건`);
  }
}

async function audit(sourceDir: string, targetDir: string): Promise<Record<Bucket, string[]>> {
  fs.mkdirSync(targetDir, { recursive: true });

  const buckets = Object.fromEntries(BUCKETS.map((b) => [b, [] as string[]])) as Record<Bucket, string[]>;
  // 어떤 종류의 차이가 몇 건 나오는지. 같은 원인이 반복되면 여기서 드러난다.
  const diffKinds = new Map<string, number>();
  let narrowed = false;

  const sources = walk(sourceDir)
    .filter((p) => ['.hwp', '.hwpx'].includes(path.extname(p).toLowerCase()))
    .sort();
  if (sources.length === 0) {
    console.log(`변환할 문서가 없습니다: ${sourceDir}`);
    return buckets;
  }

  for (const source of sources) {
    const name = path.basename(source);

    let converted;
    try {
      converted = await convertOne(source, targetDir);
    } catch (err) {
      if (err instanceof rhwp.RhwpError) {
        buckets['오류'].push(`${name}: ${err.toString()}`);
        continue;
      }
      throw err;
    }
    const { target, result } = converted;

    const verify = result.verify;
    if (verify === null) {
      // verify 를 요청했는데 보고가 없다. "검증 안 함"과 "통과"는 다르므로 통과로 세지 않는다.
      buckets['오류'].push(`${name}: verify 를 요청했는데 보고가 없음`);
      continue;
    }
    if (verify.reparseError) {
      // 저장은 됐는데 되읽지 못했다 = 산출물이 깨졌다. 차이보다 심각하다.
      buckets['재파싱실패'].push(`${name}: ${verify.reparseError}`);
      continue;
    }
    if (verify.identical) {
      buckets['통과'].push(name);
      continue;
    }

    buckets['차이'].push(`${name}: ${verify.diffCount}건`);

    // 무엇이 달라졌는지는 ir-diff 가 안다. 개수만 세면 원인을 못 찾는다.
    //
    // `categories` 는 목록이 아니라 **{ 범주: 건수 } 맵**이다. 목록으로 알고
    // 순회하면 `not iterable` 로 터진다 — 차이가 없는 폴더에서는 이 줄에 닿지도
    // 않으므로, 그런 착각은 정상 문서만 있는 동안 조용히 숨어 있다.
    const diff = await rhwp.irDiff(source, target);
    for (const [kind, count] of Object.entries(
      diff.getOr<Record<string, number>>('categories', {}),
    )) {
      diffKinds.set(kind, (diffKinds.get(kind) ?? 0) + count);
    }

    if (!narrowed) {
      narrowed = true;
      await narrowBySection(source, target);
    }
  }

  if (diffKinds.size > 0) {
    // 범주는 셀 주소까지 갈라져 수백 종이 나올 수 있다. 전부 찍으면 정작 사람이
    // 봐야 할 집계가 스크롤 밖으로 밀린다 — 많이 나온 순으로 앞부분만 남긴다.
    const ranked = [...diffKinds].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
    console.log(`\n차이 범주 ${ranked.length}종 (건수, 상위 15):`);
    for (const [kind, count] of ranked.slice(0, 15)) {
      console.log(`  ${kind}: ${count}`);
    }
    if (ranked.length > 15) {
      console.log(`  … 외 ${ranked.length - 15}종`);
    }
  }

  return buckets;
}

async function main(source: string, target: string): Promise<number> {
  const buckets = await audit(source, target);

  console.log('\n집계:');
  for (const bucket of BUCKETS) {
    console.log(`  ${bucket}: ${buckets[bucket].length}`);
  }

  for (const bucket of PROBLEM_BUCKETS) {
    for (const item of buckets[bucket].slice(0, 10)) {
      console.log(`  [${bucket}] ${item}`);
    }
    if (buckets[bucket].length > 10) {
      console.log(`  [${bucket}] … 외 ${buckets[bucket].length - 10}건`);
    }
  }

  // 문제 문서가 하나라도 있으면 판정 실패다. 도구는 정상 동작했고, 틀린 것은
  // "이 폴더는 무손실로 변환된다"는 단언이다 — 그래서 1 이 아니라 3.
  const problems = PROBLEM_BUCKETS.reduce((sum, b) => sum + buckets[b].length, 0);
  return problems > 0 ? 3 : 0;
}

const argv = process.argv.slice(2);
const [source, target] = argv;
if (argv.length !== 2 || source === undefined || target === undefined) {
  console.error('사용법: npx tsx examples/09-convert-audit.ts 입력폴더 출력폴더');
  process.exit(2);
}

process.exit(await main(source, target));
