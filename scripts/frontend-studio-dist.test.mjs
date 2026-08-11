import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const STUDIO_ASSETS = path.join(ROOT, 'rhwp-studio/dist/assets');
const HOTPATCH_RUNTIME = path.join(ROOT, 'rhwp-studio/src/core/subsecond-runtime.ts');

/**
 * 프로덕션 번들에 한 번도 나오면 안 되는 핫패치 표지.
 *
 * [#4580] 최소화(minify)를 견디는 것만 고른다. 클래스·함수 이름은 rolldown 이 재작명하므로
 * `SubsecondRevisionWatcher` 가 없다는 사실은 모듈이 빠졌다는 증거가 되지 못한다 — 실제로
 * 수정 전 번들에도 그 이름은 0회였다. 남는 것은 두 종류뿐이다.
 *
 * - 문자열 리터럴: 데브서버 소켓 경로.
 * - 외부 객체(wasm export·`HwpDocument` 핸들)의 속성 이름: 재작명하면 호출이 깨지므로 남는다.
 *
 * 아래 첫 테스트가 각 표지를 개발 전용 모듈에서 다시 찾아본다. 모듈에서 사라진 표지를 계속
 * 감시하며 "0건" 을 자축하는 일이 없게 하기 위해서다.
 */
const HOTPATCH_MARKERS = [
  '_dioxus',
  'subsecondProbe',
  'applySubsecondDevtoolsMessage',
  'getRenderCodeRevision',
  'rebuildDerivedState',
];

/**
 * 벤더 이름 자체. 프로덕션 번들 어디에도 나오면 안 된다.
 *
 * [#4580] 위 표지 목록이 하나씩 놓칠 수 있는 것을 이쪽이 통째로 막는다 — 개발 전용 런타임이
 * 빠졌더라도 프로덕션 코드의 필드·메서드 이름이 벤더를 부르고 있으면 여기서 걸린다(최소화는
 * 클래스 멤버 이름을 건드리지 않는다). 실제로 모듈만 뺐을 때 `subsecondRevisionWatcher`
 * `startSubsecondHotpatchSupport` 같은 이름 10건이 남아 있었다.
 */
const VENDOR_NAMES = ['subsecond', 'Subsecond', 'dioxus', 'Dioxus'];

/** 번들을 실제로 읽었다는 증거. 이게 없으면 "부재" 단언이 빈 문자열에서 공짜로 통과한다. */
const BUNDLE_SENTINEL = 'document-view-changed';

function readStudioBundle() {
  const files = readdirSync(STUDIO_ASSETS).filter((file) => file.endsWith('.js'));
  assert.ok(
    files.length > 0,
    `${STUDIO_ASSETS} 에 자바스크립트 번들이 없다 — 먼저 \`npm --prefix rhwp-studio run build\``,
  );
  return files.map((file) => readFileSync(path.join(STUDIO_ASSETS, file), 'utf8')).join('\n');
}

function countOccurrences(haystack, needle) {
  let count = 0;
  let from = 0;
  for (;;) {
    const at = haystack.indexOf(needle, from);
    if (at < 0) return count;
    count += 1;
    from = at + needle.length;
  }
}

test('hot-patch markers still name something in the development-only runtime', () => {
  const runtime = readFileSync(HOTPATCH_RUNTIME, 'utf8');
  const stale = HOTPATCH_MARKERS.filter((marker) => !runtime.includes(marker));
  assert.deepEqual(
    stale,
    [],
    '이 표지들은 개발 전용 런타임에 더는 없다 — 번들 감시가 헛돌고 있으므로 목록을 고쳐라',
  );
});

test('studio production bundle carries no hot-patch development runtime', () => {
  const bundle = readStudioBundle();
  assert.ok(
    bundle.includes(BUNDLE_SENTINEL),
    `번들에서 ${BUNDLE_SENTINEL} 를 못 찾았다 — 엉뚱한 파일을 읽고 있다`,
  );

  const leaked = HOTPATCH_MARKERS
    .map((marker) => [marker, countOccurrences(bundle, marker)])
    .filter(([, count]) => count > 0);
  assert.deepEqual(
    leaked,
    [],
    '개발 전용 핫패치 런타임이 프로덕션 번들에 실렸다 (표지, 등장 횟수)',
  );
});

test('studio production bundle never names the hot-patch vendor', () => {
  const bundle = readStudioBundle();
  assert.ok(bundle.includes(BUNDLE_SENTINEL), `번들에서 ${BUNDLE_SENTINEL} 를 못 찾았다`);

  const named = VENDOR_NAMES
    .map((name) => [name, countOccurrences(bundle, name)])
    .filter(([, count]) => count > 0);
  assert.deepEqual(
    named,
    [],
    '프로덕션 번들이 핫패치 벤더의 이름을 부른다 — 도메인 어휘로 바꿔라 (이름, 등장 횟수)',
  );
});
