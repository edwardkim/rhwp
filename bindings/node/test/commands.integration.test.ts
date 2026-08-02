/**
 * 1층(무상태 명령) 통합 — 실물 rhwp 와 실물 문서로 왕복한다.
 *
 * 여기서 확인하는 것은 "TypeScript 가 JSON 을 읽을 수 있나"가 아니다. 그건 자명하다.
 * 확인하는 것은 **바인딩이 계약을 그대로 재포장했나**다 — 판정이 예외가 아니라 값으로
 * 오는가, `dry-run` 이 정말 디스크를 건드리지 않는가, `null` 과 `[]` 가 구분되는가.
 * 이 셋 중 하나라도 무너지면 호출자는 실패한 작업을 성공으로 읽는다.
 */

import { existsSync } from 'node:fs';

import { describe, expect, it } from 'vitest';

import {
  EXIT_USAGE,
  RhwpRuntimeError,
  UsageError,
  batch,
  exportTables,
  exportText,
  fields,
  fillFields,
  info,
  runJson,
  search,
  setCell,
} from '../src/index.js';
import {
  fieldSample,
  fieldSampleReady,
  firstBodyTableCell,
  firstFieldName,
  hasBinary,
  pickNeedle,
  tableSample,
  tableSampleReady,
  useTempDir,
} from './helpers/integration.js';

/** `export-text` 봉투의 쪽 하나. */
interface TextPage {
  readonly page: number;
  readonly text: string;
}

/** `search` 봉투의 매치 하나. */
interface SearchMatch {
  readonly page: number;
  readonly section: number;
  readonly paragraph: number;
  readonly charOffset: number;
}

describe.skipIf(!fieldSampleReady)('1층 조회 — 봉투를 그대로 전달한다', () => {
  it('info 는 스키마 버전·쪽수·포맷을 담고, 원문 키와 get() 이 같은 값을 가리킨다', async () => {
    const envelope = await info(fieldSample());

    expect(envelope.schemaVersion).toBe('1.0');
    expect(envelope.get<number>('pageCount')).toBeGreaterThanOrEqual(1);
    expect(['hwp5', 'hwpx', 'hwp3', 'hml']).toContain(envelope.get<string>('format'));

    // 원문 키(camelCase)·snake_case·`.raw` 세 경로가 같은 값이어야 한다. 하나라도
    // 어긋나면 "속성으로 읽었더니 값이 없다"는 진단 불가 상황이 생긴다.
    expect(envelope.get('page_count')).toBe(envelope.get('pageCount'));
    expect(envelope.raw['pageCount']).toBe(envelope.get('pageCount'));
  });

  it('exportText 의 pages 길이가 pageCount 와 일치한다', async () => {
    const envelope = await exportText(fieldSample());

    const pages = envelope.get<readonly TextPage[]>('pages');
    // 쪽 하나가 조용히 빠지면 "본문이 없는 문서"로 오독된다. 길이 대조가 그 유일한
    // 자동 방어선이다 — 내용은 사람이 봐야 알지만 개수는 기계가 안다.
    expect(pages).toHaveLength(envelope.get<number>('pageCount'));
  });

  it('fields 는 누름틀 목록과 개수를 함께 준다', async () => {
    const envelope = await fields(fieldSample());

    const list = envelope.get<readonly unknown[]>('fields');
    expect(Array.isArray(list)).toBe(true);
    expect(envelope.get<number>('fieldCount')).toBe(list.length);
  });

  it('search 는 매치마다 쪽 번호를 붙여 준다 — 주소 없는 검색은 인용할 수 없다', async () => {
    const text = await exportText(fieldSample());
    const pages = text.get<readonly TextPage[]>('pages');
    const needle = pages.length > 0 ? pickNeedle(pages[0]?.text ?? '') : undefined;
    // 검색할 어휘가 없는 샘플에서 억지로 실패시키면 게이트가 신호를 잃는다.
    if (needle === undefined) return;

    const result = await search(fieldSample(), needle);

    expect(result.get<number>('matchCount')).toBeGreaterThanOrEqual(1);
    const pageCount = text.get<number>('pageCount');
    for (const match of result.get<readonly SearchMatch[]>('matches')) {
      expect(Number.isInteger(match.page)).toBe(true);
      expect(match.page).toBeGreaterThanOrEqual(0);
      expect(match.page).toBeLessThan(pageCount);
      expect(Number.isInteger(match.section)).toBe(true);
      expect(Number.isInteger(match.paragraph)).toBe(true);
      expect(Number.isInteger(match.charOffset)).toBe(true);
    }
  });

  it('`-` 로 시작하는 검색어도 옵션이 아니라 값으로 읽힌다', async () => {
    // `--` 구분자를 빠뜨리면 CLI 가 검색어를 알 수 없는 옵션으로 보고 exit 2 를 낸다.
    // 사용자 입력이 그대로 검색어가 되는 이상 이건 이론적 위험이 아니다.
    const result = await search(fieldSample(), '-이런어휘는문서에없다9999');

    expect(result.get<string>('query')).toBe('-이런어휘는문서에없다9999');
    expect(result.get<number>('matchCount')).toBe(0);
  });
});

describe.skipIf(!tableSampleReady)('1층 조회 — 표 좌표는 조회해서 쓴다', () => {
  const tempPath = useTempDir();

  it('exportTables 가 알려 준 좌표로 setCell 이 실제로 기록한다', async () => {
    const listed = await exportTables(tableSample());
    const tables = listed.get<readonly unknown[]>('tables');
    // 개수 필드와 목록 길이가 어긋나면 어느 쪽을 믿어야 할지 알 수 없다.
    expect(listed.get<number>('tableCount')).toBe(tables.length);

    const address = await firstBodyTableCell(tableSample());
    // 본문 최상위 표가 없는 문서라면 검증할 좌표 계약 자체가 없다.
    if (address === undefined) return;

    const out = tempPath('셀기록.hwpx');
    const result = await setCell(
      tableSample(),
      address.table,
      address.row,
      address.col,
      '통합테스트값',
      { out, verify: true },
    );

    // 좌표를 추측했다면(예: 표 0번) 머리말 표에 걸려 런타임 실패가 났을 것이다.
    // 조회한 좌표가 그대로 먹힌다는 것이 이 테스트의 전부다.
    expect(result.get<number>('table')).toBe(address.table);
    expect(result.get<string>('newText')).toBe('통합테스트값');
    expect(result.get<string>('oldText')).toBe(address.oldText);
    expect(existsSync(out)).toBe(true);
    // 판정 자체의 값은 엔진의 문제다. 바인딩이 봐야 할 것은 "요청했으면 보고가 온다".
    expect(result.verify, 'verify: true 로 요청했는데 보고가 없다').not.toBeNull();
    expect(typeof result.verify?.identical).toBe('boolean');
  });
});

describe.skipIf(!fieldSampleReady)('1층 편집 — 판정은 예외가 아니라 값이다', () => {
  const tempPath = useTempDir();

  it('fillFields + verify 는 판정을 envelope.verify 로 돌려준다', async () => {
    const name = await firstFieldName(fieldSample());
    // 누름틀이 없는 샘플이면 채울 대상이 없다 — 계약이 아니라 데이터의 문제다.
    if (name === undefined) return;

    const out = tempPath('채움.hwp');
    // 판정 실패(exit 3)여도 던지지 않는 것이 기본이다. 여기서 예외가 나면 호출자는
    // `try/catch` 로 "고장"처럼 다루게 되고, 정작 판정 근거를 읽지 않는다.
    const result = await fillFields(fieldSample(), { [name]: '통합테스트' }, { out, verify: true });

    expect(existsSync(out)).toBe(true);
    const verify = result.verify;
    expect(verify, 'verify: true 로 요청했는데 보고가 없다').not.toBeNull();
    expect(typeof verify?.identical).toBe('boolean');
    expect(result.get<number>('filledCount')).toBe(1);
  });

  it('verify 를 요청하지 않으면 판정은 null 이다 — "검증 안 함"과 "검증 실패"는 다르다', async () => {
    const name = await firstFieldName(fieldSample());
    if (name === undefined) return;

    const out = tempPath('무검증.hwp');
    const result = await fillFields(fieldSample(), { [name]: '값' }, { out });

    // 둘을 섞으면 검증하지 않은 저장을 통과로 읽는다 — 가장 조용한 사고다.
    expect(result.verify).toBeNull();
  });

  it('dryRun 은 산출 파일을 만들지 않는다', async () => {
    const name = await firstFieldName(fieldSample());
    if (name === undefined) return;

    const out = tempPath('만들어지면안됨.hwp');
    const result = await fillFields(fieldSample(), { [name]: '값' }, { out, dryRun: true });

    expect(result.get<boolean>('dryRun')).toBe(true);
    expect(existsSync(out), 'dry-run 이 파일을 만들었다').toBe(false);
  });

  it('changedPages 는 null(모름)·[](없음)·[n](있음) 세 상태를 구분한다', async () => {
    const name = await firstFieldName(fieldSample());
    if (name === undefined) return;

    // 1) dry-run: 저장을 하지 않았으니 어느 쪽이 바뀔지 **확정할 수 없다** → null.
    const dry = await fillFields(fieldSample(), { [name]: '값' }, {
      out: tempPath('dry.hwp'),
      dryRun: true,
    });
    expect(dry.changedPages).toBeNull();

    // 2) 없는 누름틀만 지목: 저장은 했지만 바뀐 쪽이 **없다** → [].
    const nothing = await fillFields(
      fieldSample(),
      { 절대로존재하지않는누름틀XYZ: '값' },
      { out: tempPath('무변경.hwp') },
    );
    expect(nothing.changedPages).toEqual([]);

    // 3) 실제로 채움: 눈검증 대상 쪽이 온다 → [n].
    const changed = await fillFields(fieldSample(), { [name]: '값' }, {
      out: tempPath('변경.hwp'),
    });
    expect(changed.changedPages).not.toBeNull();
    expect(changed.changedPages?.length).toBeGreaterThanOrEqual(1);
    for (const page of changed.changedPages ?? []) {
      expect(Number.isInteger(page)).toBe(true);
    }

    // 셋을 falsy 로 뭉뚱그리면 1)과 2)가 같아진다 — "확인할 게 없다"는 잘못된 결론.
    expect(dry.changedPages).not.toEqual(nothing.changedPages);
  });
});

describe.skipIf(!hasBinary)('1층 실패 경로 — 종료 코드를 예외로 옮긴다', () => {
  it('없는 파일은 RhwpRuntimeError — 인자를 고쳐도 풀리지 않는 실패다', async () => {
    await expect(info('존재하지-않는-문서-integration.hwp')).rejects.toBeInstanceOf(
      RhwpRuntimeError,
    );
  });

  it('알 수 없는 명령은 UsageError 이고 교정 제안을 함께 준다', async () => {
    let caught: unknown;
    try {
      // 호출 조립 버그(우리 쪽 잘못)이므로 재시도가 아니라 수정이 답이다.
      await runJson(['expot-text', 'a.hwp', '--json']);
    } catch (error) {
      caught = error;
    }

    expect(caught).toBeInstanceOf(UsageError);
    const usage = caught as UsageError;
    expect(usage.exitCode).toBe(EXIT_USAGE);
    // 도구가 stderr 에 남긴 did-you-mean 을 바인딩이 구조화해 전달해야 한다.
    // 원문을 뒤지게 만들면 그 단서는 사실상 없는 것과 같다.
    expect(usage.suggestion).toBeDefined();
    expect(usage.suggestion).toContain('export-text');
    // 재현 명령이 그대로 붙여넣기 가능해야 버그 리포트가 성립한다.
    expect(usage.command).toContain('expot-text');
  });
});

describe.skipIf(!fieldSampleReady)('1층 대량 — 부분 실패도 결과다', () => {
  const tempPath = useTempDir();

  it('batch 는 실패 항목을 error 레코드로 남기고 성공 레코드를 버리지 않는다', async () => {
    const missing = tempPath('없는문서.hwp');
    const records = await batch('info', [fieldSample(), missing]);

    expect(records).toHaveLength(2);

    const failed = records.filter((record) => record.error !== undefined);
    const succeeded = records.filter((record) => record.error === undefined);

    // 스트림을 통째로 버리면 성공분까지 잃는다. batch 의 존재 이유가 사라진다.
    expect(failed).toHaveLength(1);
    expect(succeeded).toHaveLength(1);
    expect(succeeded[0]?.['pageCount']).toBeGreaterThanOrEqual(1);
    expect(failed[0]?.source).toBe(missing);
    expect(failed[0]?.error).toBeTruthy();
  });
});
