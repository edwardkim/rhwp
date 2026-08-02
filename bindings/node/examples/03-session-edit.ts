/*
 * 세션 편집 — 한 번 열어 여러 번 만지고, 바뀐 쪽만 렌더한다.
 *
 * 1층 무상태 명령은 호출할 때마다 문서를 다시 판다. 편집이 여러 번이면 세션이
 * 싸다. 대신 세션은 반드시 닫아야 한다 — 자식 프로세스가 남기 때문이다.
 *
 *   npx tsx examples/03-session-edit.ts 서식.hwp 결과.hwp
 */

import path from 'node:path';
import process from 'node:process';

import * as rhwp from '../src/index.js';

/** 렌더 산출물 경로. `결과.hwp` → `결과.p3.svg` 처럼 쪽 번호를 파일명에 남긴다. */
function renderTarget(target: string, page: number): string {
  const dir = path.dirname(target);
  const stem = path.basename(target, path.extname(target));
  return path.join(dir, `${stem}.p${page}.svg`);
}

async function main(source: string, target: string): Promise<number> {
  const doc = await rhwp.openDocument(source);

  // try/finally 로 감싼다. 중간에 던져도 자식 프로세스를 남기지 않기 위해서다.
  // 런타임이 명시적 자원 관리를 지원하면 `await using doc = await rhwp.openDocument(...)`
  // 로 대신할 수 있다(Document 가 [Symbol.asyncDispose] 를 구현한다).
  try {
    const meta = await doc.info();
    console.log(`열림: ${meta.get<number>('pageCount')}쪽`);

    // 1) 누름틀 — 이름은 문서에 물어본다.
    //    세션 도구도 1층과 같은 봉투를 돌려주므로 조회 규약(`.get`/`.children`)이 같다.
    const [firstField] = (await doc.fields()).children('fields');
    if (firstField === undefined) {
      console.log('누름틀이 없는 문서입니다.');
      return 1;
    }
    const first = firstField.get<string>('name');
    await doc.fillFields({ [first]: '세션에서 입력' });
    console.log(`'${first}' 채움`);

    // 2) 표 — 행/열을 **추측하지 않는다**. 조회한 칸이 알려준 좌표만 쓴다.
    //    좌표를 확인할 수 없으면 편집을 건너뛴다. 0,0 을 넣어 보는 것은 편집이 아니라 도박이다.
    const [table] = (await doc.tables()).children('tables');
    const anchor = table
      ?.children('cells')
      .find((c) => c.has('row') && c.has('col'));
    if (table === undefined || anchor === undefined) {
      console.log('칸 좌표를 확인할 수 없어 set-cell 은 건너뜁니다.');
    } else {
      const row = anchor.get<number>('row');
      const col = anchor.get<number>('col');
      await doc.setCell(table.get<number>('index'), row, col, '세션에서 수정');
      console.log(`표 ${table.get<number>('index')} (${row}, ${col}) 수정`);
    }

    // 3) 검색 — 편집 결과를 세션 안에서 바로 확인한다.
    const found = await doc.search('보고');
    console.log(`'보고' 검색: ${found.getOr<number>('matchCount', 0)}건`);

    // 4) 저장 + 검증. 판정은 예외가 아니라 반환값이다.
    const saved = await doc.save(target, { verify: true });
    const verify = saved.verify;
    const verdict = verify === null ? '요청 안 함' : verify.identical ? '통과' : '실패';
    console.log(`저장: ${target} (검증 ${verdict})`);
    if (verify !== null && !verify.identical) {
      console.log(`  차이 ${verify.diffCount}건`);
      return 3;
    }

    // 5) 눈검증. 바뀐 쪽만 렌더하는 것이 요점이다 — 300쪽 문서를 통째로 그리지 않는다.
    //    null 은 "바뀐 쪽 없음"이 아니라 "모름"이다. 둘을 섞으면 회귀를 놓친다.
    const changed = saved.changedPages;
    if (changed === null) {
      console.log('  바뀐 쪽을 알 수 없습니다 — 눈검증은 전체를 대상으로 하세요.');
    } else if (changed.length === 0) {
      console.log('  바뀐 쪽이 없어 렌더할 것이 없습니다.');
    } else {
      for (const page of changed) {
        const out = renderTarget(target, page);
        // renderPage 는 출력 경로가 필수다 — 어디에 떨어졌는지 모르는 산출물을 만들지 않는다.
        await doc.renderPage(page, out);
        console.log(`  눈검증용 렌더: ${out}`);
      }
    }

    return 0;
  } finally {
    await doc.close();
  }
}

const argv = process.argv.slice(2);
const [source, target] = argv;
if (argv.length !== 2 || source === undefined || target === undefined) {
  console.error('사용법: npx tsx examples/03-session-edit.ts 서식.hwp 결과.hwp');
  process.exit(2);
}

process.exit(await main(source, target));
