/*
 * 시각 회귀 게이트 — 회귀는 예외가 아니라 데이터다.
 *
 * 저장·변환이 조판을 얼마나 밀었는지는 IR 비교로는 안 보인다. IR 이 같아도 렌더
 * 결과가 다를 수 있고, 사람이 보는 것은 렌더 쪽이다. `render-diff` 는 두 렌더의
 * 쪽별 bbox 변위를 재어 판정한다.
 *
 * **판정 실패는 도구의 고장이 아니다.** 도구는 정상 동작했고 "이 저장은 조판을
 * 바꾸지 않는다"는 단언이 틀린 것이다. 그래서 회귀는 예외가 아니라 봉투의
 * `status`·`regression` 필드로 오고, 이 게이트는 종료 코드 3 으로 답한다.
 *
 * 회귀를 찾으면 그 쪽만 잘라 **최소 재현 문서**를 만든다 — 300쪽짜리 버그 신고서와
 * 1쪽짜리 재현 파일은 고쳐지는 속도가 다르다.
 *
 *   npx tsx examples/11-render-diff-gate.ts 문서.hwp            (자기 라운드트립)
 *   npx tsx examples/11-render-diff-gate.ts 원본.hwp 비교본.hwp  (두 파일 직접 비교)
 */

import path from 'node:path';
import process from 'node:process';

import * as rhwp from '../src/index.js';

/** 변위 임계(px). 렌더 반올림 수준의 흔들림을 회귀로 세지 않기 위한 값이다. */
const THRESHOLD_PX = 1.0;

/**
 * 판정 요약을 한 줄로.
 *
 * `maxDisp`·`worstPage` 는 **`null` 일 수 있다** — 측정할 수 없었다는 뜻이지
 * 0 이나 "0쪽"이 아니다. NaN 을 0 으로 위장시키면 소비자는 "차이 없음"으로 읽는다.
 */
function summarize(diff: rhwp.Envelope): string {
  const maxDisp = diff.getOr<number | null>('maxDisp', null);
  const worst = diff.getOr<number | null>('worstPage', null);
  return (
    `status=${diff.get<string>('status')} ` +
    `maxDisp=${maxDisp === null ? '측정불가' : `${maxDisp.toFixed(2)}px`} ` +
    `(임계 ${diff.getOr<number>('threshold', THRESHOLD_PX)}px) ` +
    `worstPage=${worst === null ? '없음' : worst} ` +
    `over=${diff.getOr<number>('overPages', 0)} ` +
    `struct=${diff.getOr<number>('structPages', 0)}`
  );
}

/** 가장 많이 밀린 쪽의 상위 변위를 찍는다. 어디가 얼마나 밀렸는지가 곧 단서다. */
function describeWorstPage(diff: rhwp.Envelope, worstPage: number): void {
  const page = diff.children('pages').find((p) => p.getOr<number | null>('page', null) === worstPage);
  if (page === undefined) {
    // `page` 필터를 걸어 실행하면 다른 쪽은 봉투에 없다. 없는 것을 "차이 없음"으로
    // 읽지 않도록 사실만 적는다.
    console.log(`  ${worstPage}쪽 상세가 봉투에 없습니다 (page 필터를 걸었나요?)`);
    return;
  }

  console.log(
    `  ${worstPage}쪽: 노드 ${page.getOr<number>('nodeCountA', 0)}→${page.getOr<number>('nodeCountB', 0)}` +
      `, 구조불일치=${page.getOr<boolean>('structureMismatch', false)}`,
  );
  for (const delta of page.children('topDeltas').slice(0, 5)) {
    const disp = delta.getOr<number | null>('disp', null);
    // 변위는 위치(dx·dy)뿐 아니라 **크기**(dw·dh) 변화도 포함한다. dx·dy 만 찍으면
    // "0px 밀렸는데 Δ가 6px"인 줄이 나와 읽는 사람이 도구를 의심하게 된다.
    console.log(
      `    ${delta.getOr<string>('nodeType', '?')} ${delta.getOr<string>('path', '?')} ` +
        `Δ=${disp === null ? '?' : disp.toFixed(2)}px ` +
        `(dx=${delta.getOr<number>('dx', 0)}, dy=${delta.getOr<number>('dy', 0)}, ` +
        `dw=${delta.getOr<number>('dw', 0)}, dh=${delta.getOr<number>('dh', 0)})`,
    );
  }
}

/**
 * 회귀가 난 쪽만 잘라 최소 재현 문서를 만든다.
 *
 * 쪽 번호 기준이 **다르다**. `render-diff` 의 쪽은 0 기준인데
 * `extractPages(from, to)` 는 **1 기준**이다 — CLI 가 그렇다(`--from 0` 은
 * "쪽 범위가 잘못됐습니다 … (1 기준)" 으로 exit 1). 이 바인딩의 다른 쪽 인자는
 * 전부 0 기준이라 여기서만 +1 이 붙는다. 이 한 칸을 빼먹으면 **옆 쪽을 잘라 놓고
 * 재현이 안 된다고 결론**짓게 된다.
 *
 * 산출 경로는 생략할 수 없다 — 빠뜨리면 CLI 가 사용법 오류(exit 2)로 끝낸다.
 */
async function cutRepro(source: string, page0: number): Promise<string> {
  const dir = path.dirname(source);
  const stem = path.basename(source, path.extname(source));
  // 잘라낸 문서는 HWP5 로 직렬화된다 — 입력이 .hwpx 여도 확장자는 .hwp 다.
  const out = path.join(dir, `${stem}.p${page0}.repro.hwp`);

  const cut = await rhwp.extractPages(source, page0 + 1, page0 + 1, { out });
  console.log(
    `  최소 재현: ${cut.get<string>('output')} ` +
      `(${cut.getOr<number>('pagesBefore', 0)}쪽 → ${cut.getOr<number>('pagesAfter', 0)}쪽, ` +
      `문단 ${cut.getOr<number>('paragraphsKept', 0)}개 유지)`,
  );
  return out;
}

async function main(a: string, b: string | undefined): Promise<number> {
  // throwOnVerdict 를 쓰지 않는다.
  //
  // 켜면 회귀가 예외로 올라오고, 그 순간 `status`·`maxDisp`·`pages[].topDeltas`
  // 가 담긴 봉투를 게이트가 쓰지 못한다 — catch 블록에서 예외를 다시 뜯어 봉투를
  // 꺼내는 코드는 값으로 받는 코드보다 길고 틀리기 쉽다. 무엇보다 회귀는 **찾으려고
  // 부른 것**이지 사고가 아니다. 예외는 못 부른 경우(파일 없음·바이너리 없음)를 위해
  // 남겨 둔다. 여기서 예외가 나면 그건 정말로 실행에 실패한 것이다.
  const diff = await rhwp.renderDiff(a, b, {
    // 라운드트립일 때만 의미가 있다. 두 파일 비교(pair)에서는 무시되고 봉투의
    // `via` 가 null 로 온다 — "hwpx 로 재검증했다"고 잘못 보고하지 않도록.
    via: 'hwpx',
    maxDisp: THRESHOLD_PX,
  });

  const mode = diff.get<string>('mode');
  const via = diff.get<string | null>('via');
  console.log(`비교 방식: ${mode}${via === null ? '' : ` (경유 ${via})`}`);
  console.log(summarize(diff));

  if (diff.getOr<boolean>('pageCountMismatch', false)) {
    console.log(
      `쪽 수가 다릅니다: ${diff.getOr<number>('pageCountA', 0)} → ${diff.getOr<number>('pageCountB', 0)}`,
    );
  }

  // `regression` 이 판정의 전부다. `status` 는 사유를 말한다.
  //   PASS            — 임계 안
  //   WARN_TEXTRUN    — TextRun ±1 로 설명되는 조성 노이즈. 하드 실패가 아니다.
  //   OVER            — 변위가 임계를 넘었다
  //   STRUCT_MISMATCH — 노드 구성이 다르다
  //   PAGE_MISMATCH   — 쪽 수가 다르다
  // status 만 보고 `!== 'PASS'` 로 게이트를 짜면 WARN_TEXTRUN 에서 거짓 경보가 난다.
  if (!diff.get<boolean>('regression')) {
    if (diff.get<string>('status') === 'WARN_TEXTRUN') {
      console.log('통과 (TextRun ±1 조성 노이즈 — 하드 실패로 세지 않습니다)');
    } else {
      console.log('통과 — 조판이 유지됐습니다.');
    }
    return 0;
  }

  console.log('\n시각 회귀:');
  const worst = diff.getOr<number | null>('worstPage', null);
  if (worst === null) {
    // 변위를 잴 기준 자체가 없다(대개 쪽 수 불일치). 자를 쪽을 고를 근거가 없으므로
    // 재현 파일을 만들지 않는다 — 아무 쪽이나 잘라 두면 재현 실패를 오진한다.
    console.log('  변위를 측정하지 못해 재현 쪽을 고를 수 없습니다.');
    return 3;
  }

  describeWorstPage(diff, worst);

  // 두 파일 비교라면 양쪽 다 잘라야 재현이 성립한다 — 한쪽만 있으면 비교할 짝이 없다.
  const reproA = await cutRepro(a, worst);
  if (b !== undefined) {
    await cutRepro(b, worst);
    console.log('  두 재현 파일을 같은 명령으로 다시 비교하세요.');
    return 3;
  }

  // 라운드트립이면 재현 파일 한 장으로 곧장 다시 잰다. 재현되면 신고서가 1쪽으로
  // 줄고, 재현되지 않으면 원인이 그 쪽이 아니라 문서 전역(구역 설정·쪽 나눔)에 있다.
  //
  // `pathB` 자리에 `undefined` 를 명시해야 옵션을 줄 수 있다.
  const recheck = await rhwp.renderDiff(reproA, undefined, {
    via: 'hwpx',
    page: 0,
    maxDisp: THRESHOLD_PX,
  });
  console.log(
    `  재현 확인: ${recheck.get<boolean>('regression') ? '재현됨' : '재현되지 않음'} — ${summarize(recheck)}`,
  );

  // 판정 실패는 3. 도구는 정상이었고 문서에 대한 단언이 틀렸다.
  return 3;
}

const argv = process.argv.slice(2);
const [a, b] = argv;
if (argv.length < 1 || argv.length > 2 || a === undefined) {
  console.error('사용법: npx tsx examples/11-render-diff-gate.ts 문서.hwp [비교본.hwp]');
  process.exit(2);
}

process.exit(await main(a, b));
