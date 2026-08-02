/*
 * RAG 색인 — 주소를 잃지 않는 청킹.
 *
 * 평문을 뽑아 외부에서 자르면 "몇 쪽에 있었나"에 답할 수 없다. rhwp 는 조판
 * 엔진을 갖고 있으므로 청크마다 주소(쪽·제목)를 붙일 수 있고, 그래야 나중에
 * 인용을 **검증**할 수 있다. 검증할 수 없는 인용은 인용이 아니라 주장이다.
 *
 *   npx tsx examples/07-rag-index.ts 문서.hwp [검증할문구]
 */

import fs from 'node:fs';
import process from 'node:process';

import * as rhwp from '../src/index.js';

/** 절마다 남길 최대 문자 수. 문맥 창이 좁은 모델에 넘길 것을 전제로 잡는다. */
const MAX_CHARS_PER_SECTION = 600;

/** 색인 한 조각. text 만 남기면 답은 만들 수 있어도 근거는 못 댄다. */
interface Chunk {
  text: string;
  source: string;
  page: number | null;
  heading: string | null;
  /** 발췌에 실제로 담긴 길이. */
  charCount: number;
  /**
   * 절 **원문**의 길이. 발췌보다 크면 이 청크는 절의 앞부분만 갖고 있다.
   *
   * 두 값을 하나로 합치면 "이 청크가 절 전체"라고 잘못 믿게 되고, 그 믿음 위에서
   * 만든 답은 뒷부분을 못 본 채 단정한다.
   */
  sectionChars: number;
}

/** 절 단위로 자르되 주소를 보존한다. */
async function buildIndex(path: string): Promise<Chunk[]> {
  // sections: true — 고정 길이로 자르지 않고 문서가 스스로 나눈 경계를 쓴다.
  // 절 경계는 의미 경계에 가깝고, 무엇보다 쪽 번호를 함께 알려준다.
  //
  // maxChars — 발췌 길이 상한. `sections: true` 의 기본은 절마다 240자라 짧고,
  // 이 값이 없으면 긴 문서에서 발췌만으로 모델의 문맥 창을 다 먹는다. 색인의
  // 청크 크기는 검색 품질과 비용을 동시에 결정하므로 **명시**하는 편이 낫다.
  const digest = await rhwp.digest(path, {
    sections: true,
    maxChars: MAX_CHARS_PER_SECTION,
  });

  // 절 봉투의 필드 이름은 `excerpt`·`title` 이다(`text`·`heading` 이 아니다).
  // 이름을 짐작해서 읽으면 `.getOr()` 의 기본값만 돌아오고, 색인은 **0개인 채로
  // 성공한다** — 조용한 실패의 교과서적인 모양이다. 그래서 반드시 있어야 하는
  // 필드는 `.get()` 으로 읽어 이름이 틀리면 즉시 예외가 되게 한다.
  const chunks: Chunk[] = [];
  for (const s of digest.children('sections')) {
    const text = s.get<string>('excerpt').trim();
    if (!text) continue;
    // 제목 없는 절은 빈 문자열로 온다 — `''` 와 "제목을 모른다"는 다르므로 여기서 가른다.
    const title = s.getOr<string>('title', '').trim();
    chunks.push({
      text,
      source: path,
      // undefined 를 그대로 두지 않고 null 로 명시한다 — JSON 으로 나가면
      // undefined 필드는 사라지고, "쪽을 모른다"와 "쪽 필드가 없다"가 구분되지 않는다.
      page: s.getOr<number | null>('page', null),
      heading: title === '' ? null : title,
      charCount: text.length,
      sectionChars: s.getOr<number>('charCount', text.length),
    });
  }
  return chunks;
}

/** 인용문이 실제로 나오는 쪽 번호. 비어 있으면 그 인용은 근거가 없다. */
async function verifyCitation(path: string, quote: string): Promise<number[]> {
  const found = await rhwp.search(path, quote);
  const pages = new Set<number>();
  for (const m of found.children('matches')) {
    const page = m.getOr<number | null>('page', null);
    if (page !== null) pages.add(page);
  }
  return [...pages].sort((a, b) => a - b);
}

async function main(path: string, quote: string | undefined): Promise<number> {
  const chunks = await buildIndex(path);
  if (chunks.length === 0) {
    console.log('색인할 내용이 없습니다 (빈 문서이거나 절 구조가 없음)');
    return 1;
  }

  const total = chunks.reduce((sum, c) => sum + c.charCount, 0);
  const origin = chunks.reduce((sum, c) => sum + c.sectionChars, 0);
  console.log(
    `청크 ${chunks.length}개, 발췌 ${total.toLocaleString('ko-KR')}자 ` +
      `(절 원문 ${origin.toLocaleString('ko-KR')}자)`,
  );
  if (origin > total) {
    // 색인이 문서의 일부만 담고 있다는 사실은 숨기면 안 된다. 이 상태로 "문서에
    // 없는 내용"이라고 답하면 그건 근거 없는 부정이다. 봉투의 `truncated` 도 같은
    // 사실을 도구 쪽에서 알려 준다.
    console.log(`  (절마다 앞 ${MAX_CHARS_PER_SECTION}자만 담았습니다 — 원문의 일부입니다)`);
  }

  const addressed = chunks.filter((c) => c.page !== null).length;
  console.log(`주소 있는 청크: ${addressed}/${chunks.length}`);
  if (addressed < chunks.length) {
    // 주소 없는 청크를 섞어 두면 나중에 "이 문장 몇 쪽?"에 답할 수 없다. 숨기지 않는다.
    console.log('  (일부 청크는 쪽을 확정할 수 없습니다)');
  }

  console.log('\n앞 3개:');
  for (const c of chunks.slice(0, 3)) {
    const head = c.heading ?? '(제목 없음)';
    const preview = c.text.slice(0, 60).replace(/\n/g, ' ');
    console.log(`  [${c.page ?? '?'}쪽] ${head}: ${preview}…`);
  }

  if (quote !== undefined) {
    const pages = await verifyCitation(path, quote);
    if (pages.length > 0) {
      console.log(`\n인용 '${quote}' → ${pages.join(', ')}쪽에서 확인됨`);
    } else {
      console.log(`\n인용 '${quote}' 를 문서에서 찾지 못했습니다 — 근거 없는 인용입니다`);
      // 문서는 멀쩡하고 도구도 정상이다. 틀린 것은 인용에 대한 단언이므로 exit 3.
      return 3;
    }
  }

  const out = `${path}.index.json`;
  fs.writeFileSync(out, `${JSON.stringify(chunks, null, 2)}\n`, 'utf-8');
  console.log(`\n색인 저장: ${out}`);
  return 0;
}

const argv = process.argv.slice(2);
const [docPath, quote] = argv;
if (argv.length < 1 || argv.length > 2 || docPath === undefined) {
  console.error('사용법: npx tsx examples/07-rag-index.ts 문서.hwp [검증할문구]');
  process.exit(2);
}

process.exit(await main(docPath, quote));
