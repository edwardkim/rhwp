/**
 * `.d.ts` → `.d.cts` 복제.
 *
 * `exports` 맵의 `require` 갈래는 `.d.cts` 를 가리킨다(그래야 CJS 소비자가 ESM
 * 타입을 물고 "CJS 위장" 오류를 내지 않는다). 그런데 `tsc` 는 `.d.ts` 만 낸다 —
 * tsup 의 dts 파이프라인이 TypeScript 7 을 못 다뤄 선언 생성을 tsc 로 옮겼기
 * 때문이다(package.json 의 tsup.dts=false 주석 참조).
 *
 * 이 패키지의 타입에는 ESM 전용 문법이 없으므로 두 갈래가 같은 내용이면 충분하다.
 * 내용이 갈라져야 할 일이 생기면 그때 진짜 이중 생성으로 바꾼다.
 */
import { readdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

// `URL.pathname` 은 윈도우에서 `/C:/…` 를 준다 — 그대로 쓰면 `C:\C:\…` 가 된다.
// `fileURLToPath` 가 플랫폼별 변환을 담당한다.
const dist = fileURLToPath(new URL('../dist/', import.meta.url));
const entries = await readdir(dist);
let made = 0;

for (const name of entries) {
  if (!name.endsWith('.d.ts')) continue;
  const source = await readFile(join(dist, name), 'utf8');
  // 상대 import 확장자는 `.js` 그대로 둔다 — 타입 해석에는 영향이 없고,
  // 바꾸면 `.d.cts` 안에서 존재하지 않는 파일을 가리키게 된다.
  await writeFile(join(dist, name.replace(/\.d\.ts$/, '.d.cts')), source, 'utf8');
  made += 1;
}

console.log(`.d.cts ${made}개 생성`);
