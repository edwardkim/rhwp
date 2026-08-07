/**
 * 명령 래퍼 — 각 함수가 CLI 인자를 정확히 조립하는지.
 *
 * 봉투를 읽는 것은 통합 테스트가 본다. 여기서는 **인자 조립**만 본다. 이 층을
 * 따로 고정하는 이유가 있다: 플래그 하나가 빠지거나 이름이 어긋나면 도구는
 * 실패하지 않고 **조용히 다른 일을 한다**. `--from` 을 `--pages` 로 잘못 쓰면
 * exit 2 라도 나지만, `--max-chars` 를 안 붙이면 그냥 기본값으로 잘려 돌아오고
 * 봉투만 봐서는 무엇이 빠졌는지 알 길이 없다.
 *
 * 그래서 여기서 고정하는 것은 "동작"이 아니라 **argv 의 정확한 모양**이다.
 * 실물 rhwp 가 그 플래그를 실제로 받는지(값인지 토글인지)는 실행으로 확인했고,
 * 이 파일은 그 확인 결과가 코드에서 흘러내리지 않게 붙잡는다.
 */

import { describe, expect, it, vi } from 'vitest';

import { UsageError } from '../src/errors.js';
import type { Argument, RunOptions } from '../src/process.js';

// `vi.mock` 은 import 위로 끌어올려지므로, 아래 정적 import 로 들어오는 명령
// 모듈은 이미 가짜 process 를 물고 온다. 실행 파일을 찾는 코드가 아예 로드되지
// 않으므로 이 파일은 rhwp 바이너리 없이 돈다 — 단위 잡의 계약이다.
import * as rhwp from '../src/commands.js';

/**
 * 실행 대신 인자만 가로챈다.
 *
 * `vi.mock` 의 공장은 import 위로 끌어올려지므로, 수집 상자는 `vi.hoisted` 로
 * 같이 끌어올려야 한다 — 그러지 않으면 공장이 아직 없는 변수를 참조한다.
 */
const seen = vi.hoisted(() => ({
  argv: [] as Argument[][],
  options: [] as RunOptions[],
}));

vi.mock('../src/process.js', () => ({
  DEFAULT_TIMEOUT_MS: 300_000,
  runJson: async (args: readonly Argument[], options: RunOptions = {}) => {
    seen.argv.push([...args]);
    seen.options.push({ ...options });
    return { schemaVersion: '1.0' };
  },
  runNdjson: async (args: readonly Argument[], options: RunOptions = {}) => {
    seen.argv.push([...args]);
    seen.options.push({ ...options });
    return [];
  },
  runRaw: async () => ({ argv: [], exitCode: 0, stdout: '', stderr: '' }),
  iterNdjson: async function* () {
    /* 이 파일은 스트리밍 축을 쓰지 않는다. */
  },
}));

/**
 * 래퍼를 한 번 부르고 그 argv 를 문자열 배열로 돌려준다. 숫자 인자는 CLI 에
 * 문자열로 나가므로 여기서 맞춰 준다.
 *
 * 완성된 프로미스가 아니라 **함수**를 받는 이유: 인자로 `rhwp.info(...)` 를 쓰면
 * 호출이 이 함수보다 먼저 일어나 "몇 번 실행됐나"를 셀 수 없다. 실행 횟수를
 * 세는 것이 이 파일의 절반이다 — 조립만 하고 안 부르거나 두 번 부르는 버그를
 * argv 비교만으로는 잡지 못한다.
 */
async function argvOf(run: () => Promise<unknown>): Promise<string[]> {
  const before = seen.argv.length;
  await run();
  expect(seen.argv.length, '래퍼는 프로세스를 정확히 한 번 부른다').toBe(before + 1);
  return (seen.argv.at(-1) as Argument[]).map(String);
}

/** 마지막 호출의 실행 옵션. */
function lastOptions(): RunOptions {
  return seen.options.at(-1) as RunOptions;
}

/** `--name` 바로 뒤의 값. 없으면 실패시킨다. */
function valueAfter(argv: string[], name: string): string {
  const index = argv.indexOf(name);
  expect(index, `${name} 이 argv 에 없습니다: ${argv.join(' ')}`).toBeGreaterThanOrEqual(0);
  return argv[index + 1] as string;
}

// ── 조회 ────────────────────────────────────────────────────────────────────

describe('조회 명령 — argv 모양', () => {
  it('info', async () => {
    expect(await argvOf(() => rhwp.info('a.hwp'))).toEqual(['info', 'a.hwp', '--json']);
  });

  it('fields', async () => {
    expect(await argvOf(() => rhwp.fields('a.hwp'))).toEqual(['fields', 'a.hwp', '--json']);
  });

  it('export-text', async () => {
    expect(await argvOf(() => rhwp.exportText('a.hwp'))).toEqual(['export-text', 'a.hwp', '--json']);
  });

  it('export-text — page 는 -p 로 나간다', async () => {
    expect(await argvOf(() => rhwp.exportText('a.hwp', { page: 2 }))).toEqual([
      'export-text',
      'a.hwp',
      '-p',
      '2',
      '--json',
    ]);
  });

  it('export-text — 0쪽도 붙는다 (falsy 라고 빠지면 안 된다)', async () => {
    expect(await argvOf(() => rhwp.exportText('a.hwp', { page: 0 }))).toContain('-p');
  });

  it('export-structure', async () => {
    expect(await argvOf(() => rhwp.exportStructure('a.hwp'))).toEqual([
      'export-structure',
      'a.hwp',
      '--json',
    ]);
  });

  it('export-structure — mode 는 --mode 로 나간다', async () => {
    for (const mode of ['auto', 'outline', 'clause'] as const) {
      const argv = await argvOf(() => rhwp.exportStructure('a.hwp', { mode }));
      expect(valueAfter(argv, '--mode')).toBe(mode);
    }
  });

  it('export-tables', async () => {
    expect(await argvOf(() => rhwp.exportTables('a.hwpx'))).toEqual([
      'export-tables',
      'a.hwpx',
      '--json',
    ]);
  });

  it('digest', async () => {
    expect(await argvOf(() => rhwp.digest('a.hwp'))).toEqual(['digest', 'a.hwp', '--json']);
  });

  it('digest — sections·pages·maxChars 가 모두 실린다', async () => {
    const argv = await argvOf(() =>
      rhwp.digest('a.hwp', { sections: true, pages: '0..4', maxChars: 800 }),
    );
    expect(argv).toContain('--sections');
    expect(valueAfter(argv, '--pages')).toBe('0..4');
    expect(valueAfter(argv, '--max-chars')).toBe('800');
  });

  it('explain', async () => {
    expect(await argvOf(() => rhwp.explain('a.hwp'))).toEqual(['explain', 'a.hwp', '--json']);
  });

  it('capabilities — 문서를 받지 않고 --json 도 붙이지 않는다', async () => {
    expect(await argvOf(() => rhwp.capabilities())).toEqual(['capabilities']);
    expect(await argvOf(() => rhwp.capabilities({ mcp: true }))).toEqual(['capabilities', '--mcp']);
  });

  it('export-ir-schema', async () => {
    expect(await argvOf(() => rhwp.exportIrSchema())).toEqual(['export-ir-schema', '--json']);
    expect(await argvOf(() => rhwp.exportIrSchema({ bare: true }))).toEqual([
      'export-ir-schema',
      '--bare',
      '--json',
    ]);
  });

  it('export-capabilities-schema', async () => {
    expect(await argvOf(() => rhwp.exportCapabilitiesSchema())).toEqual([
      'export-capabilities-schema',
      '--json',
    ]);
  });

  it('export-capabilities-schema — out 은 -o 로 나간다 (봉투가 유지되는 축)', async () => {
    expect(await argvOf(() => rhwp.exportCapabilitiesSchema({ bare: true, out: 'caps.json' }))).toEqual([
      'export-capabilities-schema',
      '--bare',
      '-o',
      'caps.json',
      '--json',
    ]);
  });

  it('export-plan-schema — bare·out 을 JSON 봉투와 함께 보낸다', async () => {
    expect(await argvOf(() => rhwp.exportPlanSchema({ bare: true, out: 'plan.json' }))).toEqual([
      'export-plan-schema',
      '--bare',
      '-o',
      'plan.json',
      '--json',
    ]);
  });

  it('export-agent-manifest — bare 를 JSON 봉투와 함께 보낸다', async () => {
    expect(await argvOf(() => rhwp.exportAgentManifest({ bare: true }))).toEqual([
      'export-agent-manifest',
      '--bare',
      '--json',
    ]);
  });

  it('export-provenance-map — 문서 경로 없이 JSON 봉투를 요청한다', async () => {
    expect(await argvOf(() => rhwp.exportProvenanceMap())).toEqual(['export-provenance-map', '--json']);
  });
});

describe('표 교환·구조 데이터 명령 — argv 모양', () => {
  it('table-to-csv — 표·출력·BOM 플래그를 정확히 조립한다', async () => {
    expect(await argvOf(() => rhwp.tableToCsv('a.hwp', { table: 0, out: 'a.csv', bom: true }))).toEqual([
      'table-to-csv',
      'a.hwp',
      '--table',
      '0',
      '-o',
      'a.csv',
      '--bom',
      '--json',
    ]);
  });

  it('csv-to-table — CSV·표·편집 공통 플래그를 정확히 조립한다', async () => {
    expect(
      await argvOf(() =>
        rhwp.csvToTable('a.hwp', {
          csv: 'a.csv',
          table: 0,
          out: 'b.hwp',
          dryRun: true,
          verify: true,
        }),
      ),
    ).toEqual([
      'csv-to-table',
      'a.hwp',
      '--csv',
      'a.csv',
      '--table',
      '0',
      '-o',
      'b.hwp',
      '--dry-run',
      '--verify',
      '--json',
    ]);
  });

  it('extract-data — kind·limit는 값 플래그이고 0도 유지한다', async () => {
    expect(await argvOf(() => rhwp.extractData('a.hwp', { kind: 'amount', limit: 0 }))).toEqual([
      'extract-data',
      'a.hwp',
      '--kind',
      'amount',
      '--limit',
      '0',
      '--json',
    ]);
  });
});

describe('inspect — 하위 명령별 플래그 격리', () => {
  it('hidden-text는 threshold·offpage만 붙인다', async () => {
    expect(
      await argvOf(() =>
        rhwp.inspect('hidden-text', 'a.hwp', {
          thresholdPt: 0,
          includeOffpage: true,
        }),
      ),
    ).toEqual(['inspect', 'hidden-text', 'a.hwp', '--threshold-pt', '0', '--include-offpage', '--json']);
  });

  it('injection은 confidence·fields만 붙인다', async () => {
    expect(
      await argvOf(() =>
        rhwp.inspect('injection', 'a.hwp', {
          minConfidence: 'high',
          includeFields: true,
        }),
      ),
    ).toEqual([
      'inspect',
      'injection',
      'a.hwp',
      '--min-confidence',
      'high',
      '--include-fields',
      '--json',
    ]);
  });

  it('unicode는 kind를 해당 하위 명령에만 붙인다', async () => {
    expect(await argvOf(() => rhwp.inspect('unicode', 'a.hwp', { kind: 'bidi' }))).toEqual([
      'inspect',
      'unicode',
      'a.hwp',
      '--kind',
      'bidi',
      '--json',
    ]);
  });
});

describe('search — 검색어 보호', () => {
  it('기본 모양', async () => {
    expect(await argvOf(() => rhwp.search('a.hwp', '예산'))).toEqual([
      'search',
      'a.hwp',
      '--json',
      '--',
      '예산',
    ]);
  });

  it('`-` 로 시작하는 어휘도 값으로 읽히도록 -- 뒤 마지막에 둔다', async () => {
    const argv = await argvOf(() => rhwp.search('a.hwp', '-예산'));
    expect(argv.slice(-2)).toEqual(['--', '-예산']);
  });

  it('caseSensitive:false 일 때만 --ignore-case', async () => {
    expect(await argvOf(() => rhwp.search('a.hwp', 'x', { caseSensitive: false }))).toContain(
      '--ignore-case',
    );
    expect(await argvOf(() => rhwp.search('a.hwp', 'x', { caseSensitive: true }))).not.toContain(
      '--ignore-case',
    );
    expect(await argvOf(() => rhwp.search('a.hwp', 'x'))).not.toContain('--ignore-case');
  });

  it('limit 은 검색어보다 앞에 온다', async () => {
    const argv = await argvOf(() => rhwp.search('a.hwp', 'x', { limit: 5 }));
    expect(valueAfter(argv, '--limit')).toBe('5');
    expect(argv.indexOf('--limit')).toBeLessThan(argv.indexOf('--'));
  });
});

// ── 산출 ────────────────────────────────────────────────────────────────────

describe('산출 명령 — argv 모양', () => {
  it('export-svg', async () => {
    expect(await argvOf(() => rhwp.exportSvg('a.hwp'))).toEqual(['export-svg', 'a.hwp', '--json']);
    expect(await argvOf(() => rhwp.exportSvg('a.hwp', { out: 'o', page: 2 }))).toEqual([
      'export-svg',
      'a.hwp',
      '-o',
      'o',
      '-p',
      '2',
      '--json',
    ]);
  });

  it('export-pdf', async () => {
    expect(await argvOf(() => rhwp.exportPdf('a.hwp'))).toEqual(['export-pdf', 'a.hwp', '--json']);
  });

  it('export-pdf — backend·profile·page 가 모두 실린다', async () => {
    const argv = await argvOf(() =>
      rhwp.exportPdf('a.hwp', {
        out: 'o.pdf',
        page: 1,
        backend: 'svg',
        profile: 'print',
      }),
    );
    expect(argv).toEqual([
      'export-pdf',
      'a.hwp',
      '-o',
      'o.pdf',
      '-p',
      '1',
      '--backend',
      'svg',
      '--profile',
      'print',
      '--json',
    ]);
  });

  it('export-pdf — fontPath 는 문자열 하나도, 목록도 받는다', async () => {
    expect(await argvOf(() => rhwp.exportPdf('a.hwp', { fontPath: 'ttfs' }))).toEqual([
      'export-pdf',
      'a.hwp',
      '--font-path',
      'ttfs',
      '--json',
    ]);
    // 여러 경로는 이어 붙이지 않고 플래그를 반복한다 — 경로에 쉼표가 들어갈 수 있다.
    expect(await argvOf(() => rhwp.exportPdf('a.hwp', { fontPath: ['ttfs', 'my fonts'] }))).toEqual([
      'export-pdf',
      'a.hwp',
      '--font-path',
      'ttfs',
      '--font-path',
      'my fonts',
      '--json',
    ]);
  });

  it('export-markdown', async () => {
    expect(await argvOf(() => rhwp.exportMarkdown('a.hwp'))).toEqual([
      'export-markdown',
      'a.hwp',
      '--json',
    ]);
    expect(await argvOf(() => rhwp.exportMarkdown('a.hwp', { out: 'md', page: 0 }))).toEqual([
      'export-markdown',
      'a.hwp',
      '-o',
      'md',
      '-p',
      '0',
      '--json',
    ]);
  });

  it('export-hml', async () => {
    expect(await argvOf(() => rhwp.exportHml('a.hml', { out: 'b.hml' }))).toEqual([
      'export-hml',
      'a.hml',
      '-o',
      'b.hml',
      '--json',
    ]);
  });

  it('export-doclang — assetsDir 는 --assets-dir 로 나간다', async () => {
    expect(await argvOf(() => rhwp.exportDoclang('a.hwp'))).toEqual([
      'export-doclang',
      'a.hwp',
      '--json',
    ]);
    expect(
      await argvOf(() => rhwp.exportDoclang('a.hwp', { out: 'a.xml', assetsDir: 'assets' })),
    ).toEqual(['export-doclang', 'a.hwp', '-o', 'a.xml', '--assets-dir', 'assets', '--json']);
  });

  it('thumbnail — base64·dataUri 는 값 없는 토글이다', async () => {
    expect(await argvOf(() => rhwp.thumbnail('a.hwp'))).toEqual(['thumbnail', 'a.hwp', '--json']);
    expect(await argvOf(() => rhwp.thumbnail('a.hwp', { out: 't.png', base64: true }))).toEqual([
      'thumbnail',
      'a.hwp',
      '-o',
      't.png',
      '--base64',
      '--json',
    ]);
    expect(await argvOf(() => rhwp.thumbnail('a.hwp', { dataUri: true }))).toEqual([
      'thumbnail',
      'a.hwp',
      '--data-uri',
      '--json',
    ]);
  });

  it('thumbnail — 둘을 함께 주면 --base64 가 먼저 나간다', async () => {
    // 두 플래그는 CLI 에서 배타적이고 **나중 것이 이긴다**. 순서를 뒤집으면
    // JSDoc 이 약속한 "dataUri 가 온다"가 조용히 뒤집힌다.
    expect(await argvOf(() => rhwp.thumbnail('a.hwp', { base64: true, dataUri: true }))).toEqual([
      'thumbnail',
      'a.hwp',
      '--base64',
      '--data-uri',
      '--json',
    ]);
  });

  it('extract-pages — --pages 가 아니라 --from/--to 다', async () => {
    const argv = await argvOf(() => rhwp.extractPages('a.hwp', 1, 3, { out: 'b.hwp' }));
    expect(argv).toEqual([
      'extract-pages',
      'a.hwp',
      '--from',
      '1',
      '--to',
      '3',
      '-o',
      'b.hwp',
      '--json',
    ]);
    // `--pages` 는 digest 쪽 어휘다. 섞이면 CLI 가 exit 2 로 끝난다.
    expect(argv).not.toContain('--pages');
  });

  it('build-from-ingest — mediaDir 는 --media-dir 로 나간다', async () => {
    expect(await argvOf(() => rhwp.buildFromIngest('spec.json'))).toEqual([
      'build-from-ingest',
      'spec.json',
      '--json',
    ]);
    expect(
      await argvOf(() =>
        rhwp.buildFromIngest('spec.json', {
          out: 'new.hwpx',
          mediaDir: 'media',
        }),
      ),
    ).toEqual(['build-from-ingest', 'spec.json', '--media-dir', 'media', '-o', 'new.hwpx', '--json']);
  });
});

// ── 변환·비교 ───────────────────────────────────────────────────────────────

describe('변환 — 산출 경로는 위치 인자다', () => {
  it('export-hwpx 는 -o 를 쓰지 않는다 (CLI 가 모르는 옵션이다)', async () => {
    const argv = await argvOf(() => rhwp.exportHwpx('a.hwp', { out: 'b.hwpx' }));
    expect(argv).toEqual(['export-hwpx', 'a.hwp', 'b.hwpx', '--json']);
    expect(argv).not.toContain('-o');
  });

  it('export-hwpx — out 없이도 된다 (CLI 가 <입력 stem>.hwpx 를 쓴다)', async () => {
    expect(await argvOf(() => rhwp.exportHwpx('a.hwp'))).toEqual(['export-hwpx', 'a.hwp', '--json']);
  });

  it('export-hwpx — verify 플래그는 산출 경로 뒤에 붙는다', async () => {
    expect(
      await argvOf(() =>
        rhwp.exportHwpx('a.hwp', {
          out: 'b.hwpx',
          verify: true,
          verifyPages: true,
        }),
      ),
    ).toEqual(['export-hwpx', 'a.hwp', 'b.hwpx', '--verify', '--verify-pages', '--json']);
  });

  it('convert 도 위치 인자를 쓴다', async () => {
    const argv = await argvOf(() => rhwp.convert('a.hwpx', { out: 'b.hwp' }));
    expect(argv).toEqual(['convert', 'a.hwpx', 'b.hwp', '--json']);
    expect(argv).not.toContain('-o');
  });

  it('convert — verifyPages 도 노출한다', async () => {
    expect(
      await argvOf(() =>
        rhwp.convert('a.hwpx', {
          out: 'b.hwp',
          verify: true,
          verifyPages: true,
        }),
      ),
    ).toEqual(['convert', 'a.hwpx', 'b.hwp', '--verify', '--verify-pages', '--json']);
  });

  it('convert — out 이 없으면 프로세스를 띄우기 전에 거절한다', async () => {
    const before = seen.argv.length;
    await expect(rhwp.convert('a.hwpx')).rejects.toBeInstanceOf(UsageError);
    expect(seen.argv.length, '실행이 일어나면 안 된다').toBe(before);
  });

  it('ir-diff — section 은 -s, paragraph 는 -p', async () => {
    expect(await argvOf(() => rhwp.irDiff('a.hwpx', 'b.hwp'))).toEqual([
      'ir-diff',
      'a.hwpx',
      'b.hwp',
      '--json',
    ]);
    expect(await argvOf(() => rhwp.irDiff('a.hwpx', 'b.hwp', { section: 0, paragraph: 4 }))).toEqual([
      'ir-diff',
      'a.hwpx',
      'b.hwp',
      '-s',
      '0',
      '-p',
      '4',
      '--json',
    ]);
  });
});

describe('render-diff — 라운드트립과 pair', () => {
  it('경로 하나면 자기 라운드트립', async () => {
    expect(await argvOf(() => rhwp.renderDiff('a.hwp'))).toEqual(['render-diff', 'a.hwp', '--json']);
  });

  it('경로 둘이면 두 파일 직접 비교 — 둘 다 위치 인자다', async () => {
    expect(await argvOf(() => rhwp.renderDiff('a.hwp', 'b.hwp'))).toEqual([
      'render-diff',
      'a.hwp',
      'b.hwp',
      '--json',
    ]);
  });

  it('via·page·maxDisp 가 각각 붙는다', async () => {
    expect(await argvOf(() => rhwp.renderDiff('a.hwp', undefined, { via: 'hwp' }))).toEqual([
      'render-diff',
      'a.hwp',
      '--via',
      'hwp',
      '--json',
    ]);
    expect(await argvOf(() => rhwp.renderDiff('a.hwp', undefined, { page: 0 }))).toEqual([
      'render-diff',
      'a.hwp',
      '-p',
      '0',
      '--json',
    ]);
    expect(await argvOf(() => rhwp.renderDiff('a.hwp', undefined, { maxDisp: 0.5 }))).toEqual([
      'render-diff',
      'a.hwp',
      '--max-disp',
      '0.5',
      '--json',
    ]);
  });

  it('셋을 함께 주면 위치 인자 뒤, --json 앞에 순서대로', async () => {
    expect(
      await argvOf(() =>
        rhwp.renderDiff('a.hwp', 'b.hwp', {
          via: 'hwpx',
          page: 2,
          maxDisp: 1.5,
        }),
      ),
    ).toEqual([
      'render-diff',
      'a.hwp',
      'b.hwp',
      '--via',
      'hwpx',
      '-p',
      '2',
      '--max-disp',
      '1.5',
      '--json',
    ]);
  });

  it('미지정 시 플래그가 붙지 않는다', async () => {
    const argv = await argvOf(() => rhwp.renderDiff('a.hwp', 'b.hwp'));
    for (const flag of ['--via', '-p', '--max-disp', '-o', '--batch']) {
      expect(argv, `${flag} 가 옵션 없이 붙었습니다`).not.toContain(flag);
    }
  });

  it('--batch 는 감싸지 않는다 — NDJSON 이라 반환 타입이 다르다', async () => {
    // 한 함수가 봉투와 스트림을 다 돌려주면 호출자가 받은 값의 타입을 모른다.
    expect(await argvOf(() => rhwp.renderDiff('a.hwp'))).not.toContain('--batch');
  });

  it('회귀 판정을 예외로 올리는 선택이 실행 계층까지 간다', async () => {
    await argvOf(() => rhwp.renderDiff('a.hwp', 'b.hwp', { throwOnVerdict: true }));
    expect(lastOptions().throwOnVerdict).toBe(true);
  });

  it('기본은 회귀를 던지지 않는다 — status·regression 으로 읽는다', async () => {
    await argvOf(() => rhwp.renderDiff('a.hwp', 'b.hwp'));
    expect(lastOptions().throwOnVerdict).toBeUndefined();
  });
});

// ── 편집 ────────────────────────────────────────────────────────────────────

describe('편집 명령 — argv 모양', () => {
  it('fill-fields', async () => {
    expect(await argvOf(() => rhwp.fillFields('a.hwp', { 성명: '홍길동' }))).toEqual([
      'edit',
      'fill-fields',
      'a.hwp',
      '--data',
      '{"성명":"홍길동"}',
      '--json',
    ]);
  });

  it('fill-fields — --data 는 한글을 이스케이프하지 않은 JSON 이다', async () => {
    const argv = await argvOf(() => rhwp.fillFields('a.hwp', { 기관명: '국립국어원' }));
    const data = valueAfter(argv, '--data');
    // CLI 는 UTF-8 을 그대로 받는다. \uXXXX 로 escape 하면 필드 이름이 안 맞아
    // "문서에 없는 필드" 로 조용히 흘러간다.
    expect(data).toBe('{"기관명":"국립국어원"}');
    expect(data).not.toContain('\\u');
    expect(JSON.parse(data)).toEqual({ 기관명: '국립국어원' });
  });

  it('fill-fields — 편집 공통 플래그', async () => {
    expect(
      await argvOf(() =>
        rhwp.fillFields('a.hwp', { 이름: '값' }, { out: 'b.hwp', dryRun: true, verify: true }),
      ),
    ).toEqual([
      'edit',
      'fill-fields',
      'a.hwp',
      '--data',
      '{"이름":"값"}',
      '-o',
      'b.hwp',
      '--dry-run',
      '--verify',
      '--json',
    ]);
  });

  it('replace-text', async () => {
    expect(await argvOf(() => rhwp.replaceText('a.hwp', '2025년', '2026년'))).toEqual([
      'edit',
      'replace-text',
      'a.hwp',
      '--find',
      '2025년',
      '--replace',
      '2026년',
      '--json',
    ]);
  });

  it('replace-text — occurrence 는 값, ignoreCase 는 토글', async () => {
    const argv = await argvOf(() =>
      rhwp.replaceText('a.hwp', '□', '☑', { occurrence: 2, ignoreCase: true }),
    );
    expect(valueAfter(argv, '--occurrence')).toBe('2');
    expect(argv).toContain('--ignore-case');
  });

  it('replace-text — 빈 문자열 치환(삭제)도 인자로 나간다', async () => {
    const argv = await argvOf(() => rhwp.replaceText('a.hwp', '삭제', ''));
    expect(argv[argv.indexOf('--replace') + 1]).toBe('');
  });

  it('set-cell — 좌표 4종이 순서대로 실린다', async () => {
    const argv = await argvOf(() => rhwp.setCell('a.hwp', 1, 2, 3, '값'));
    expect(argv).toEqual([
      'edit',
      'set-cell',
      'a.hwp',
      '--table',
      '1',
      '--row',
      '2',
      '--col',
      '3',
      '--text',
      '값',
      '--json',
    ]);
    expect(valueAfter(argv, '--table')).toBe('1');
    expect(valueAfter(argv, '--row')).toBe('2');
    expect(valueAfter(argv, '--col')).toBe('3');
    expect(valueAfter(argv, '--text')).toBe('값');
  });

  it('set-cell — 0 좌표도 빠지지 않는다', async () => {
    const argv = await argvOf(() => rhwp.setCell('a.hwp', 0, 0, 0, ''));
    expect(valueAfter(argv, '--table')).toBe('0');
    expect(valueAfter(argv, '--row')).toBe('0');
    expect(valueAfter(argv, '--col')).toBe('0');
  });

  it('set-cell — keepStyle 토글', async () => {
    expect(await argvOf(() => rhwp.setCell('a.hwp', 0, 0, 0, 'x', { keepStyle: true }))).toContain(
      '--keep-style',
    );
  });
});

// ── 대량 ────────────────────────────────────────────────────────────────────

describe('batch — 목록은 인자가 아니라 stdin 이다', () => {
  it('기본 모양', async () => {
    expect(await argvOf(() => rhwp.batch('info', ['a.hwp']))).toEqual(['batch', 'info', '--json']);
  });

  it('경로는 argv 가 아니라 stdin 으로 흐른다', async () => {
    await argvOf(() => rhwp.batch('export-text', ['a.hwp', 'b.hwp', 'c.hwp']));
    // 경로를 argv 에 실으면 목록이 길어질 때 명령줄 길이 한도에 걸린다.
    expect(seen.argv.at(-1)).not.toContain('a.hwp');
    expect(lastOptions().stdin).toBe('a.hwp\nb.hwp\nc.hwp\n');
  });

  it('빈 목록은 프로세스를 띄우기 전에 거절한다', async () => {
    const before = seen.argv.length;
    await expect(rhwp.batch('info', [])).rejects.toThrow(/최소 1개/);
    expect(seen.argv.length, '실행이 일어나면 안 된다').toBe(before);
  });

  it('제한 시간 기본값은 무제한이다 (대량은 오래 걸린다)', async () => {
    await argvOf(() => rhwp.batch('info', ['a.hwp']));
    expect(lastOptions().timeoutMs).toBeNull();
  });

  it('축별 플래그가 모두 실린다', async () => {
    const argv = await argvOf(() =>
      rhwp.batch('convert', ['a.hwpx'], {
        threads: 4,
        outDir: 'out',
        verify: true,
        verifyPages: true,
      }),
    );
    expect(argv).toEqual([
      'batch',
      'convert',
      '--threads',
      '4',
      '--out-dir',
      'out',
      '--verify',
      '--verify-pages',
      '--json',
    ]);
  });

  it('mode·query 도 실린다', async () => {
    expect(await argvOf(() => rhwp.batch('export-structure', ['a.hwp'], { mode: 'clause' }))).toEqual([
      'batch',
      'export-structure',
      '--mode',
      'clause',
      '--json',
    ]);
    expect(await argvOf(() => rhwp.batch('search', ['a.hwp'], { query: '예산' }))).toEqual([
      'batch',
      'search',
      '--query',
      '예산',
      '--json',
    ]);
  });

  it('extraArgs 는 이름 붙은 옵션 뒤, --json 앞에 온다', async () => {
    const argv = await argvOf(() =>
      rhwp.batch('search', ['a.hwp'], {
        query: 'x',
        extraArgs: ['--query', 'y'],
      }),
    );
    expect(argv).toEqual(['batch', 'search', '--query', 'x', '--query', 'y', '--json']);
    expect(argv.at(-1)).toBe('--json');
  });
});

// ── 옵션 미지정 시 플래그가 붙지 않는다 ─────────────────────────────────────

describe('옵션을 안 주면 플래그도 안 붙는다', () => {
  /**
   * 선택 플래그가 기본값으로라도 붙기 시작하면, 도구의 기본 동작이 바인딩의
   * 기본 동작으로 굳어 버린다. 그 뒤 CLI 가 기본값을 바꿔도 바인딩만 옛 값을
   * 고집하게 되고, 아무도 그 사실을 모른다.
   */
  const NEVER_WITHOUT_OPTIONS = [
    '-o',
    '-p',
    '-s',
    '--mode',
    '--max-chars',
    '--backend',
    '--profile',
    '--font-path',
    '--assets-dir',
    '--base64',
    '--data-uri',
    '--media-dir',
    '--threads',
    '--query',
    '--out-dir',
    '--verify',
    '--verify-pages',
    '--dry-run',
    '--ignore-case',
    '--keep-style',
    '--occurrence',
    '--bare',
    '--mcp',
    '--sections',
    '--pages',
    '--limit',
    '--bom',
    '--threshold-pt',
    '--include-offpage',
    '--min-confidence',
    '--include-fields',
    '--kind',
  ];

  const minimal: [string, () => Promise<unknown>][] = [
    ['info', () => rhwp.info('a.hwp')],
    ['exportText', () => rhwp.exportText('a.hwp')],
    ['exportStructure', () => rhwp.exportStructure('a.hwp')],
    ['exportTables', () => rhwp.exportTables('a.hwp')],
    ['fields', () => rhwp.fields('a.hwp')],
    ['search', () => rhwp.search('a.hwp', 'x')],
    ['digest', () => rhwp.digest('a.hwp')],
    ['explain', () => rhwp.explain('a.hwp')],
    ['capabilities', () => rhwp.capabilities()],
    ['exportIrSchema', () => rhwp.exportIrSchema()],
    ['exportPlanSchema', () => rhwp.exportPlanSchema()],
    ['exportAgentManifest', () => rhwp.exportAgentManifest()],
    ['exportCapabilitiesSchema', () => rhwp.exportCapabilitiesSchema()],
    ['exportProvenanceMap', () => rhwp.exportProvenanceMap()],
    ['tableToCsv', () => rhwp.tableToCsv('a.hwp')],
    ['csvToTable', () => rhwp.csvToTable('a.hwp', { csv: 'a.csv', table: 0 })],
    ['extractData', () => rhwp.extractData('a.hwp')],
    ['inspect(hidden-text)', () => rhwp.inspect('hidden-text', 'a.hwp')],
    ['inspect(injection)', () => rhwp.inspect('injection', 'a.hwp')],
    ['inspect(unicode)', () => rhwp.inspect('unicode', 'a.hwp')],
    ['exportSvg', () => rhwp.exportSvg('a.hwp')],
    ['exportPdf', () => rhwp.exportPdf('a.hwp')],
    ['exportMarkdown', () => rhwp.exportMarkdown('a.hwp')],
    ['exportHml', () => rhwp.exportHml('a.hml')],
    ['exportDoclang', () => rhwp.exportDoclang('a.hwp')],
    ['thumbnail', () => rhwp.thumbnail('a.hwp')],
    ['buildFromIngest', () => rhwp.buildFromIngest('spec.json')],
    ['exportHwpx', () => rhwp.exportHwpx('a.hwp')],
    ['irDiff', () => rhwp.irDiff('a.hwpx', 'b.hwp')],
    ['renderDiff', () => rhwp.renderDiff('a.hwp')],
    ['renderDiff(pair)', () => rhwp.renderDiff('a.hwp', 'b.hwp')],
    ['fillFields', () => rhwp.fillFields('a.hwp', { 이름: '값' })],
    ['replaceText', () => rhwp.replaceText('a.hwp', 'x', 'y')],
    ['setCell', () => rhwp.setCell('a.hwp', 0, 0, 0, 'v')],
    ['batch', () => rhwp.batch('info', ['a.hwp'])],
  ];

  it.each(minimal)('%s', async (_name, run) => {
    const argv = await argvOf(run);
    for (const flag of NEVER_WITHOUT_OPTIONS) {
      expect(argv, `${flag} 가 옵션 없이 붙었습니다: ${argv.join(' ')}`).not.toContain(flag);
    }
  });

  it('convert 도 산출 경로 말고는 아무 플래그도 안 붙인다', async () => {
    const argv = await argvOf(() => rhwp.convert('a.hwpx', { out: 'b.hwp' }));
    expect(argv.filter((a) => a.startsWith('--'))).toEqual(['--json']);
  });
});

// ── 실행 계층으로의 전달 ────────────────────────────────────────────────────

describe('실행 옵션 전달', () => {
  it('timeoutMs 가 실행 계층까지 간다', async () => {
    await argvOf(() => rhwp.info('a.hwp', { timeoutMs: 1234 }));
    expect(lastOptions().timeoutMs).toBe(1234);
  });

  it('timeoutMs: null(무제한)도 그대로 간다', async () => {
    await argvOf(() => rhwp.exportPdf('a.hwp', { timeoutMs: null }));
    expect(lastOptions().timeoutMs).toBeNull();
  });

  it('cwd 가 실행 계층까지 간다', async () => {
    await argvOf(() => rhwp.exportText('a.hwp', { cwd: '/work' }));
    expect(lastOptions().cwd).toBe('/work');
  });

  it('throwOnVerdict 가 판정 명령에서 실행 계층까지 간다', async () => {
    await argvOf(() =>
      rhwp.exportHwpx('a.hwp', {
        out: 'b.hwpx',
        verify: true,
        throwOnVerdict: true,
      }),
    );
    expect(lastOptions().throwOnVerdict).toBe(true);

    await argvOf(() => rhwp.convert('a.hwpx', { out: 'b.hwp', throwOnVerdict: true }));
    expect(lastOptions().throwOnVerdict).toBe(true);

    await argvOf(() => rhwp.fillFields('a.hwp', { 이름: '값' }, { verify: true, throwOnVerdict: true }));
    expect(lastOptions().throwOnVerdict).toBe(true);
  });

  it('기본은 판정을 던지지 않는다 — 봉투로 읽는 것이 이 바인딩의 규약이다', async () => {
    await argvOf(() => rhwp.exportHwpx('a.hwp', { out: 'b.hwpx', verify: true }));
    expect(lastOptions().throwOnVerdict).toBeUndefined();
  });

  it('산출 명령 공장이 만든 래퍼도 실행 옵션을 흘린다', async () => {
    await argvOf(() => rhwp.thumbnail('a.hwp', { base64: true, timeoutMs: 50, cwd: '/w' }));
    expect(lastOptions().timeoutMs).toBe(50);
    expect(lastOptions().cwd).toBe('/w');
  });
});
