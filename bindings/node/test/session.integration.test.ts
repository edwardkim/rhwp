/**
 * 2층(세션) 통합 — `mcp-serve` 자식 하나에 문서를 얹고 왕복한다.
 *
 * 1층은 호출마다 프로세스를 띄우고 문서를 다시 파싱한다. 30쪽짜리 문서를 열 번
 * 만지면 열 번 파싱한다. 2층이 존재하는 이유가 그 비용이고, 그 대가로 **상태**가
 * 생긴다. 상태가 생기면 두 가지가 새로 위험해진다:
 *
 * - 닫힌 핸들을 다시 쓰면 조용히 다른 문서를 만질 수 있다 → `SessionClosedError`
 * - 예외로 빠져나가면 서버가 남는다 → 다음 작업이 파일을 못 연다
 *
 * 이 파일은 그 두 가지를 본다. 편집이 `save()` 전까지 디스크를 건드리지 않는다는
 * 계약도 함께 확인한다 — 중간 실패가 반쪽 문서를 남기지 않는 근거가 그것이다.
 */

import { readFileSync, existsSync, statSync } from 'node:fs';

import { describe, expect, it } from 'vitest';

import { Session, SessionClosedError, openDocument } from '../src/index.js';
import {
  fieldSample,
  fieldSampleReady,
  firstFieldName,
  tableSample,
  tableSampleReady,
  useTempDir,
} from './helpers/integration.js';

describe.skipIf(!fieldSampleReady)('2층 세션 — 열기·조회·편집·저장·닫기', () => {
  const tempPath = useTempDir();

  it('한 핸들로 조회하고 편집하고 저장하고 닫는다', async () => {
    const out = tempPath('세션산출.hwp');
    const doc = await openDocument(fieldSample());
    try {
      expect(doc.docId).toBeTruthy();

      const meta = await doc.info();
      expect(meta.get<number>('pageCount')).toBeGreaterThanOrEqual(1);

      const text = await doc.text({ page: 0 });
      expect(text.get<readonly unknown[]>('pages')).toHaveLength(1);

      const name = await firstFieldName(fieldSample());
      if (name !== undefined) {
        await doc.fillFields({ [name]: '세션값' });
        // 편집은 핸들의 IR 에만 쌓인다. 여기서 파일이 생긴다면 "중간 실패가 반쪽
        // 문서를 남기지 않는다"는 계약이 이미 깨진 것이다.
        expect(existsSync(out), 'save() 전에 산출물이 생겼다').toBe(false);
      }

      const saved = await doc.save(out, { verify: true });
      expect(existsSync(out)).toBe(true);
      expect(saved.verify, 'verify: true 로 저장했는데 보고가 없다').not.toBeNull();
      expect(typeof saved.verify?.identical).toBe('boolean');

      // 저장 후에도 핸들은 열려 있다 — 이어서 편집·재저장할 수 있어야 한다.
      const again = await doc.info();
      expect(again.get<number>('pageCount')).toBe(meta.get<number>('pageCount'));
    } finally {
      await doc.close();
    }
  });

  it('renderPage 가 지정한 경로에 SVG 를 만든다 — 눈검증 루프가 세션 안에서 닫힌다', async () => {
    const target = tempPath('쪽0.svg');
    const doc = await openDocument(fieldSample());
    try {
      // `output` 은 선택 인자가 아니다. 어디에 그렸는지 모르는 렌더는 눈검증을 닫지
      // 못하고, 바인딩이 임시 경로를 임의로 정하면 그 파일을 지울 책임이 호출자에게
      // 떠넘겨진다.
      expect(doc.renderPage.length).toBe(2);

      const rendered = await doc.renderPage(0, target);

      expect(rendered.get<string>('output')).toBeTruthy();
      expect(existsSync(target), 'SVG 파일이 나오지 않았다').toBe(true);
      expect(statSync(target).size).toBeGreaterThan(0);
      // 봉투만 그럴듯하고 파일은 빈 껍데기인 경우를 걸러 낸다.
      expect(readFileSync(target, 'utf8').slice(0, 400)).toContain('<svg');
    } finally {
      await doc.close();
    }
  });

  it('닫힌 핸들을 다시 쓰면 SessionClosedError — 조용히 다른 문서를 만지지 않는다', async () => {
    const doc = await openDocument(fieldSample());
    await doc.close();

    await expect(doc.info()).rejects.toBeInstanceOf(SessionClosedError);
    // 닫기는 멱등이어야 한다. finally 와 명시적 close 가 겹칠 때 두 번째가 터지면
    // 원래 예외가 정리 예외에 가려져 사라진다 — 여기서 던지면 이 테스트가 실패한다.
    await doc.close();
  });
});

describe.skipIf(!fieldSampleReady)('2층 세션 — 정리 보장', () => {
  it('예외로 빠져나가도 세션이 정리되고, 정리된 세션은 되살아나지 않는다', async () => {
    const session = new Session();
    let escaped: unknown;

    try {
      const doc = await openDocument(fieldSample(), { session });
      expect(doc.docId).toBeTruthy();
      throw new Error('의도적 예외');
    } catch (error) {
      escaped = error;
    } finally {
      await session.close();
    }

    expect((escaped as Error).message).toBe('의도적 예외');
    // 멱등 확인 — 두 번째 close 가 던지면 여기서 실패한다.
    await session.close();

    // 서버가 실제로 내려갔는지를 이렇게 본다: 닫힌 세션으로 다시 호출하면 거부해야
    // 한다. 여기서 조용히 새 서버를 띄우면 "정리했다"는 보고가 거짓이 되고, 원래
    // 자식은 어디에도 잡히지 않은 채 남는다.
    //
    // (OS 의 rhwp 프로세스 수를 세지 않는 이유: 통합 테스트 파일들이 병렬로 돌아
    // 전역 개수는 다른 파일의 세션에 오염된다. 개수보다 계약이 정확한 신호다.)
    await expect(session.call('hwp_open', { path: fieldSample() })).rejects.toBeInstanceOf(
      SessionClosedError,
    );
  });
});

describe.skipIf(!fieldSampleReady)('2층 세션 — 한 서버에 문서 여럿', () => {
  const tempPath = useTempDir();

  it('공유 Session 위의 두 문서가 서로 다른 핸들로 독립 동작한다', async () => {
    // 서로 다른 문서를 쓰면 "핸들이 섞였는지"를 쪽수로 바로 알 수 있다.
    const secondPath = tableSampleReady ? tableSample() : fieldSample();

    const session = new Session();
    try {
      const first = await openDocument(fieldSample(), { session });
      const second = await openDocument(secondPath, { session });

      expect(first.docId).not.toBe(second.docId);

      const firstMeta = await first.info();
      const secondMeta = await second.info();
      expect(firstMeta.get<string>('source')).toContain('field-01');
      expect(secondMeta.get<number>('pageCount')).toBeGreaterThanOrEqual(1);

      // 한쪽을 닫아도 다른 쪽은 살아 있어야 한다 — 그렇지 않으면 세션 공유가
      // 이득이 아니라 새로운 결합이 된다.
      await first.close();
      await expect(first.info()).rejects.toBeInstanceOf(SessionClosedError);

      const stillAlive = await second.info();
      expect(stillAlive.get<number>('pageCount')).toBe(secondMeta.get<number>('pageCount'));

      // 저장 형식은 확장자가 아니라 입력 형식을 따른다(#3383). 이름만 맞춰 둔다.
      const out = tempPath(tableSampleReady ? '둘째저장.hwpx' : '둘째저장.hwp');
      const saved = await second.save(out);
      expect(saved.get<string>('output')).toBeTruthy();
      expect(existsSync(out)).toBe(true);

      await second.close();
    } finally {
      await session.close();
    }
  });
});
