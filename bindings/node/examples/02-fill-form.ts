/*
 * 서식 채우기 — 검증까지 한 번에, 그리고 제출용 PDF.
 *
 * 판정은 예외가 아니라 반환값이다. 저장본이 의도한 문서인지 봉투로 확인한다.
 *
 *   npx tsx examples/02-fill-form.ts 서식.hwp 제출본.hwp [폰트폴더]
 */

import path from 'node:path';
import process from 'node:process';

import * as rhwp from '../src/index.js';

async function main(source: string, target: string, fontDir: string | undefined): Promise<number> {
  // 채울 이름은 문서에 물어본다. 이름을 지어내면 "채운 척"만 하고 조용히 지나간다.
  const available = (await rhwp.fields(source))
    .children('fields')
    .map((f) => f.get<string>('name'));
  if (available.length === 0) {
    console.log('누름틀이 없는 문서입니다.');
    return 1;
  }
  console.log(`채울 수 있는 누름틀: ${[...available].sort().join(', ')}`);

  const data: Record<string, string> = {};
  [...available].sort().forEach((name, i) => {
    data[name] = `자동입력-${i + 1}`;
  });

  // verify: true — 저장 후 되읽어 IR 이 같은지 도구가 스스로 확인한다.
  const result = await rhwp.fillFields(source, data, { out: target, verify: true });

  console.log(`\n채운 칸: ${result.get<number>('filledCount')}`);
  const notFound = result.getOr<string[]>('notFound', []);
  if (notFound.length > 0) {
    console.log(`못 찾은 이름: ${notFound.join(', ')}`);
  }

  // 여기가 이 예제의 요점이다. verify 는 세 가지 상태를 가진다.
  //   null   — 검증을 아예 요청하지 않았다 (모름)
  //   통과   — 저장본이 의도한 문서와 같다
  //   실패   — 저장본이 달라졌다 (도구는 정상 동작했고, 틀린 것은 문서에 대한 단언이다)
  // null 을 "통과"로 뭉개면 검증하지 않은 산출물을 검증했다고 보고하게 된다.
  const verify = result.verify;
  if (verify === null) {
    console.log('검증을 요청하지 않았습니다.');
  } else if (verify.identical) {
    console.log(`검증 통과 — 저장본이 의도한 문서와 같습니다: ${target}`);
  } else {
    console.log(`검증 실패 — 차이 ${verify.diffCount}건`);
    // 판정 실패는 예외가 아니다. 종료 코드 3 으로만 알린다(도구와 같은 어휘).
    return 3;
  }

  // changedPages 도 마찬가지로 세 가지다.
  //   null — 어느 쪽이 바뀌었는지 도구가 알려주지 않았다 (전부 다시 봐야 한다)
  //   []   — 바뀐 쪽이 없다 (눈으로 볼 것이 없다)
  //   [n…] — 이 쪽만 보면 된다
  const changed = result.changedPages;
  if (changed === null) {
    console.log('바뀐 쪽을 알 수 없습니다 — 전체를 확인하세요.');
  } else if (changed.length === 0) {
    console.log('바뀐 쪽 없음.');
  } else {
    console.log(`눈으로 확인할 쪽: ${changed.join(', ')}`);
  }

  // 제출용 PDF. 서식 작업의 끝은 대개 사람이 볼 파일이다.
  //
  //   profile   — `print` 는 인쇄 품질로 조판한다. 미리보기라면 `fast-preview` 로
  //               시간을 아낀다. 화면용과 인쇄용의 렌더 결과가 실제로 다르다.
  //   backend   — 기본 `svg`. `direct` 는 `native-skia` 로 빌드한 바이너리에서만
  //               동작하므로, 없는 빌드에 주면 실행 오류(exit 1)로 끝난다. 기본을
  //               명시해 두고 **실제로 쓴 backend 는 봉투에서 확인**한다.
  //   fontPath  — 한컴 전용 글꼴이 없는 서버·CI 에서 글자가 대체 글꼴로 밀린다.
  //               경로는 여러 번 줄 수 있고(문자열 하나 또는 목록), 내부에서
  //               `--font-path` 를 반복해 붙인다 — 쉼표로 이어 붙이지 않는다
  //               (경로에 쉼표가 들어갈 수 있다).
  const pdf = path.join(
    path.dirname(target),
    `${path.basename(target, path.extname(target))}.pdf`,
  );
  const rendered = await rhwp.exportPdf(target, {
    out: pdf,
    profile: 'print',
    backend: 'svg',
    // 옵션 타입이 `| undefined` 를 명시적으로 받으므로 조건부 스프레드가 필요 없다.
    fontPath: fontDir,
  });
  console.log(
    `\nPDF: ${rendered.get<string>('output')} ` +
      `(backend=${rendered.getOr<string>('backend', '?')}, ` +
      `${rendered.getOr<number>('renderedCount', 0)}쪽)`,
  );

  return 0;
}

const argv = process.argv.slice(2);
const [source, target, fontDir] = argv;
if (argv.length < 2 || argv.length > 3 || source === undefined || target === undefined) {
  console.error('사용법: npx tsx examples/02-fill-form.ts 서식.hwp 제출본.hwp [폰트폴더]');
  process.exit(2);
}

process.exit(await main(source, target, fontDir));
