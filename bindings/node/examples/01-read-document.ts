/*
 * 문서 읽기 — 요약·구조·평문·표·누름틀·미리보기.
 *
 * 봉투에서 값을 꺼내는 네 가지 방법을 한자리에서 보인다. 어느 것을 고르느냐가
 * "없는 필드"를 만났을 때의 결과를 바꾼다 — 그게 이 예제의 요점이다.
 *
 *   npx tsx examples/01-read-document.ts 문서.hwp
 */

import process from 'node:process';

// 빌드 산출물(dist)이 아니라 소스를 가리킨다. `npm run build` 없이 tsx 로 바로 돌리기 위해서다.
import * as rhwp from '../src/index.js';

async function main(path: string): Promise<number> {
  // 1) 요약.
  //
  //    `.get()` 은 없는 필드에 **예외를 던진다**. 오타가 조용한 `undefined` 로
  //    둔갑하면 그게 가장 찾기 어려운 버그이므로, 모든 문서에서 나오는 필드는
  //    이쪽으로 읽는다.
  //
  //    `.getOr()` 은 없어도 되는 필드 전용이다. 필수 필드에 쓰면 `.get()` 의
  //    보호를 스스로 버리는 것이다 — 제목 없는 문서는 흔하지만 쪽수 없는 문서는 없다.
  const meta = await rhwp.info(path);
  console.log(
    `포맷: ${meta.get<string>('format')}  쪽수: ${meta.get<number>('pageCount')}  ` +
      `구역: ${meta.get<number>('sections')}`,
  );
  console.log(`제목: ${meta.getOr<string>('title', '(없음)')}`);

  const fonts = meta.getOr<string[]>('fonts', []);
  if (fonts.length > 0) {
    console.log(`글꼴 ${fonts.length}종: ${fonts.slice(0, 5).join(', ')}`);
  }

  // 2) 구조 — 제목 계층. `mode` 를 생략하면 도구가 문서를 보고 고른다(`auto`).
  //    무엇을 골랐는지는 봉투의 `mode` 에 담기므로 **결과로 확인할 수 있다**.
  //    규정 문서를 개요로 잘못 읽었다면 `{ mode: 'clause' }` 로 되돌린다.
  const structure = await rhwp.exportStructure(path);
  console.log(
    `\n구조: ${structure.getOr<string>('mode', '?')} 기준, ` +
      `노드 ${structure.getOr<number>('nodeCount', 0)}개`,
  );

  // 3) 평문. 쪽 단위로 쪼개져 나온다 — "몇 쪽에 있었나"를 나중에 답할 수 있게.
  //
  //    `{ page }` 로 한 쪽만 받는다(**0 기준**). 300쪽 문서의 첫 쪽만 보려고
  //    전문을 메모리에 올릴 이유가 없고, 모델에 넘길 때는 그 차이가 곧 문맥 창이다.
  const text = await rhwp.exportText(path, { page: 0 });
  // `.children()` 은 배열 필드를 봉투 배열로 준다 — 항목마다 같은 조회 규약이 산다.
  const [firstPage] = text.children('pages');
  if (firstPage !== undefined) {
    const preview = firstPage.getOr<string>('text', '').slice(0, 120).replace(/\n/g, ' ');
    console.log(`1쪽 미리보기: ${preview}…`);
  }

  // 4) 누름틀. 이름을 알아야 채울 수 있으므로 먼저 조회한다(02 번 예제로 이어진다).
  const formFields = (await rhwp.fields(path)).children('fields');
  if (formFields.length > 0) {
    console.log(`\n누름틀 ${formFields.length}개:`);
    for (const f of formFields.slice(0, 10)) {
      console.log(`  - ${f.get<string>('name')}: ${JSON.stringify(f.getOr<string>('value', ''))}`);
    }
  }

  // 5) 표. 표 번호·행·열은 **추측하지 않는다**. set-cell 을 하려면 이 조회가 근거다.
  const tables = (await rhwp.exportTables(path)).children('tables');
  if (tables.length > 0) {
    console.log(`\n표 ${tables.length}개:`);
    for (const t of tables.slice(0, 5)) {
      const rows = t.getOr<number | null>('rows', null);
      const cols = t.getOr<number | null>('cols', null);
      const size = rows !== null && cols !== null ? `${rows}×${cols}, ` : '';
      console.log(`  - 표 ${t.get<number>('index')}: ${size}${t.children('cells').length}칸`);
    }
  }

  // 6) 미리보기 이미지(PrvImage).
  //
  //    `dataUri: true` 는 파일 저장을 **대체한다** — `out` 을 함께 줘도 파일은
  //    생기지 않고 봉투의 `output` 이 `null` 이 된다. `base64` 와는 **배타적**이라
  //    둘을 함께 켜면 나중 플래그만 남는다. 하나만 고른다.
  //
  //    미리보기가 없는 문서는 판정 실패가 아니라 **런타임 실패(exit 1)** 다 —
  //    "없는 것을 달라"고 한 요청이므로 도구는 그걸 오류로 알린다. 조회 예제가
  //    그것 때문에 통째로 죽을 이유는 없으니 여기서만 받아 넘긴다.
  try {
    const thumb = await rhwp.thumbnail(path, { dataUri: true });
    const uri = thumb.get<string>('dataUri');
    console.log(
      `\n미리보기: ${thumb.get<string>('mime')} ` +
        `${thumb.get<number>('width')}×${thumb.get<number>('height')}, ` +
        `data URI ${uri.length}자 (파일 없음: output=${JSON.stringify(thumb.get('output'))})`,
    );
  } catch (error) {
    if (!(error instanceof rhwp.RhwpError)) throw error;
    console.log('\n미리보기: 내장 썸네일이 없는 문서입니다.');
  }

  return 0;
}

const argv = process.argv.slice(2);
const [docPath] = argv;
if (argv.length !== 1 || docPath === undefined) {
  // 사용법은 stdout 이 아니라 stderr 로 낸다. stdout 은 기계가 읽는 자리다.
  console.error('사용법: npx tsx examples/01-read-document.ts 문서.hwp');
  process.exit(2);
}

// 처리하지 못한 예외는 Node 가 exit 1 로 끝낸다 — 도구 종료 코드 사전의 "런타임 실패"와 같다.
process.exit(await main(docPath));
