// [Task #2370 클러스터 D] 소스 가드용 구간 추출 헬퍼.
//
// 정적 소스 가드는 "함수 A 안에 호출 B 가 있는가"를 자주 묻는데, 지금까지는
// `src.slice(idx, idx + 700)` 같은 **고정 창**이나 주석 문자열을 종료 키로 써 왔다.
// 둘 다 무해한 리팩터(주석 편집·함수 순서 변경·본문 증가)에 오탐/누탐한다.
// 여기서는 괄호·중괄호를 실제로 세어 구간을 잡는다 — 문자열/주석 안의 괄호는
// 건너뛰므로 본문이 길어져도, 주석이 바뀌어도 결과가 같다.

/** `from` 에서 시작해 그 뒤 첫 `open` 부터 짝이 맞는 `close` 까지(포함) 잘라낸다. */
export function balancedFrom(src: string, from: string, open: '(' | '{'): string {
  const start = src.indexOf(from);
  if (start === -1) throw new Error(`소스에서 찾지 못함: ${from}`);
  const openIdx = src.indexOf(open, start);
  if (openIdx === -1) throw new Error(`${from} 뒤에 ${open} 가 없음`);
  return src.slice(start, matchingIndex(src, openIdx) + 1);
}

/** `openIdx` 의 괄호와 짝이 맞는 닫는 괄호의 인덱스. 문자열·주석 내부는 무시한다. */
export function matchingIndex(src: string, openIdx: number): number {
  const open = src[openIdx];
  const close = open === '(' ? ')' : open === '{' ? '}' : open === '[' ? ']' : '';
  if (!close) throw new Error(`여는 괄호가 아님: ${open}`);

  let depth = 0;
  for (let i = openIdx; i < src.length; i += 1) {
    const skipped = skipNonCode(src, i);
    if (skipped !== i) {
      i = skipped - 1;
      continue;
    }
    const ch = src[i];
    if (ch === open) depth += 1;
    else if (ch === close) {
      depth -= 1;
      if (depth === 0) return i;
    }
  }
  throw new Error(`짝이 맞는 ${close} 를 찾지 못함`);
}

/**
 * `i` 가 문자열/템플릿/주석의 시작이면 그 끝 **다음** 인덱스를, 아니면 `i` 를 반환한다.
 * (정규식 리터럴은 다루지 않는다 — 가드 대상 소스에 괄호를 담은 정규식이 나오면
 *  그때 확장할 것.)
 */
function skipNonCode(src: string, i: number): number {
  const ch = src[i];
  if (ch === '/' && src[i + 1] === '/') {
    const nl = src.indexOf('\n', i);
    return nl === -1 ? src.length : nl;
  }
  if (ch === '/' && src[i + 1] === '*') {
    const end = src.indexOf('*/', i + 2);
    return end === -1 ? src.length : end + 2;
  }
  if (ch === "'" || ch === '"' || ch === '`') {
    for (let j = i + 1; j < src.length; j += 1) {
      if (src[j] === '\\') { j += 1; continue; }
      if (src[j] === ch) return j + 1;
    }
    return src.length;
  }
  return i;
}

/** `callee(` 로 시작하는 모든 호출의 전체 텍스트(인자 포함). */
export function callsOf(src: string, callee: string): string[] {
  const out: string[] = [];
  const needle = `${callee}(`;
  for (let i = src.indexOf(needle); i !== -1; i = src.indexOf(needle, i + 1)) {
    // `foo.bar(` 에서 `bar(` 를 찾은 것처럼 이름 일부만 걸린 경우 제외.
    if (/[A-Za-z0-9_$]/.test(src[i - 1] ?? '')) continue;
    out.push(src.slice(i, matchingIndex(src, i + callee.length) + 1));
  }
  return out;
}

/**
 * `pattern` 의 모든 매치가 `ranges` 중 하나에 포함되는지. 포함되지 않은 매치를 돌려준다.
 *
 * 구간을 찾을 때 두 가지를 지킨다 — 이 헬퍼가 조용히 틀리면 가드가 초록인 채로
 * 회귀를 통과시키므로(이 파일이 고치려는 바로 그 실패 모드) 방어적으로 쓴다.
 *
 * 1. **길이를 자기 문자열에서 센다.** 먼저 `indexOf` 로 걸러 낸 뒤 인덱스만 모으면
 *    걸러진 원소 때문에 뒤 항목의 길이가 어긋난다(짝이 밀림).
 * 2. **같은 텍스트가 여러 번 나오면 각 출현을 모두 구간으로 잡는다.** `indexOf` 는
 *    첫 출현만 주므로, 텍스트가 같은 호출이 둘 있으면 두 번째의 실제 위치가
 *    "구간 밖"으로 오판된다.
 * 찾지 못한 `range` 는 호출부의 전제가 깨진 것이므로 조용히 넘기지 않고 던진다.
 */
export function matchesOutside(src: string, pattern: RegExp, ranges: string[]): string[] {
  const spans: Array<readonly [number, number]> = [];
  for (const r of ranges) {
    let at = src.indexOf(r);
    if (at === -1) throw new Error(`구간을 소스에서 찾지 못함(호출부 전제 위반): ${r.slice(0, 60)}…`);
    while (at !== -1) {
      spans.push([at, at + r.length] as const);
      at = src.indexOf(r, at + 1);
    }
  }
  const outside: string[] = [];
  for (const m of src.matchAll(pattern)) {
    const at = m.index ?? -1;
    if (!spans.some(([a, b]) => at >= a && at < b)) outside.push(m[0]);
  }
  return outside;
}
