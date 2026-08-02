/*
 * 메일머지 — 서식 1 + CSV N행 → 산출물 N개.
 *
 * 한 건 실패로 전체를 멈추지 않는다. 실패한 행만 기록하고 나머지를 계속한다.
 * 300명 중 7번째에서 멈추면 앞의 6장도 다시 만들어야 하기 때문이다.
 *
 *   npx tsx examples/08-mail-merge.ts 서식.hwp 데이터.csv 출력폴더
 */

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import * as rhwp from '../src/index.js';

/**
 * 최소 CSV 파서 — 따옴표와 그 안의 쉼표·줄바꿈만 처리한다.
 *
 * 예제에 의존성을 늘리지 않으려고 직접 썼다. 실무에서는 검증된 CSV 라이브러리를
 * 쓰는 편이 낫다. 여기서 중요한 것은 파싱이 아니라 **실패를 잃지 않는 루프**다.
 */
function parseCsv(text: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let cell = '';
  let quoted = false;

  // BOM 은 첫 열 이름에 눈에 안 보이는 문자로 달라붙어 매칭을 조용히 깨뜨린다.
  const src = text.replace(/^﻿/, '');

  for (let i = 0; i < src.length; i += 1) {
    const ch = src[i];
    if (quoted) {
      if (ch === '"') {
        if (src[i + 1] === '"') {
          cell += '"';
          i += 1;
        } else {
          quoted = false;
        }
      } else {
        cell += ch;
      }
    } else if (ch === '"') {
      quoted = true;
    } else if (ch === ',') {
      row.push(cell);
      cell = '';
    } else if (ch === '\n') {
      row.push(cell.replace(/\r$/, ''));
      rows.push(row);
      row = [];
      cell = '';
    } else {
      cell += ch;
    }
  }
  if (cell !== '' || row.length > 0) {
    row.push(cell.replace(/\r$/, ''));
    rows.push(row);
  }
  return rows.filter((r) => r.some((c) => c.trim() !== ''));
}

/** 헤더 행을 열 이름으로 삼아 레코드 목록을 만든다. */
function loadRows(csvPath: string): Record<string, string>[] {
  const rows = parseCsv(fs.readFileSync(csvPath, 'utf-8'));
  if (rows.length < 2) return [];
  const header = (rows[0] ?? []).map((h) => h.trim());
  return rows.slice(1).map((r) => {
    const record: Record<string, string> = {};
    header.forEach((name, i) => {
      record[name] = r[i] ?? '';
    });
    return record;
  });
}

/** 파일명에 쓸 수 없는 문자를 지운다. 사람 이름이 경로 구분자를 품고 있을 수 있다. */
function safeStem(value: string, fallback: string): string {
  const cleaned = value.replace(/[\\/:*?"<>|]/g, '').trim();
  return cleaned || fallback;
}

async function merge(
  form: string,
  rows: Record<string, string>[],
  outDir: string,
): Promise<{ made: string[]; failures: string[] }> {
  // 서식이 가진 누름틀 이름을 먼저 확인한다. CSV 열 이름과 겹치지 않으면
  // fill-fields 는 "0칸 채움"으로 조용히 성공한다 — 그게 제일 나쁜 결과다.
  const available = new Set(
    (await rhwp.fields(form)).children('fields').map((f) => f.get<string>('name')),
  );
  if (available.size === 0) {
    throw new rhwp.RhwpRuntimeError(`누름틀이 없는 서식입니다: ${form}`);
  }

  const made: string[] = [];
  const failures: string[] = [];

  for (const [i, row] of rows.entries()) {
    const line = i + 1;

    const data: Record<string, string> = {};
    for (const [key, value] of Object.entries(row)) {
      if (available.has(key)) data[key] = value;
    }
    if (Object.keys(data).length === 0) {
      failures.push(`${line}행: 서식과 겹치는 열이 없습니다 (열: ${Object.keys(row).join(', ')})`);
      continue;
    }

    const stem = safeStem(row['성명'] ?? row['이름'] ?? '', `row${String(line).padStart(4, '0')}`);
    const out = path.join(outDir, `${stem}_${String(line).padStart(4, '0')}.hwp`);

    try {
      const result = await rhwp.fillFields(form, data, { out, verify: true });

      // 판정은 예외로 오지 않는다. 검증 보고를 직접 읽어야 실패가 드러난다.
      const verify = result.verify;
      if (verify === null) {
        // verify: true 를 줬는데 보고가 없다 = 계약 위반. 성공으로 세면 안 된다.
        failures.push(`${line}행: verify 를 요청했는데 보고가 없습니다`);
        continue;
      }
      if (!verify.identical) {
        failures.push(`${line}행: 저장본 검증 실패 (차이 ${verify.diffCount}건)`);
        continue;
      }

      const notFound = result.getOr<string[]>('notFound', []);
      if (notFound.length > 0) {
        failures.push(`${line}행: 채우지 못한 칸 ${notFound.join(', ')}`);
        continue;
      }

      made.push(out);
    } catch (err) {
      // 이 행만 실패로 기록하고 다음 행으로 간다. 루프 전체를 죽이지 않는다.
      if (err instanceof rhwp.RhwpError) {
        failures.push(`${line}행: ${err.toString()}`);
      } else {
        throw err;
      }
    }
  }

  return { made, failures };
}

async function main(form: string, csvPath: string, outDir: string): Promise<number> {
  const rows = loadRows(csvPath);
  if (rows.length === 0) {
    console.log(`데이터가 없습니다: ${csvPath}`);
    return 1;
  }

  fs.mkdirSync(outDir, { recursive: true });
  console.log(`${rows.length}행 처리 중…`);

  const { made, failures } = await merge(form, rows, outDir);

  console.log(`\n성공 ${made.length} / 실패 ${failures.length}`);
  for (const f of failures.slice(0, 20)) {
    console.log(`  ${f}`);
  }
  if (failures.length > 20) {
    console.log(`  … 외 ${failures.length - 20}건`);
  }

  // 실패 행은 목록으로 남는다. 종료 코드 하나로 뭉개면 "어느 행"이 사라진다.
  return 0;
}

const argv = process.argv.slice(2);
const [form, csvPath, outDir] = argv;
if (argv.length !== 3 || form === undefined || csvPath === undefined || outDir === undefined) {
  console.error('사용법: npx tsx examples/08-mail-merge.ts 서식.hwp 데이터.csv 출력폴더');
  process.exit(2);
}

process.exit(await main(form, csvPath, outDir));
