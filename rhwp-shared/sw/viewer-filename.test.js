import assert from 'node:assert/strict';
import test from 'node:test';

// Exercise the real launchers: URL encoding must preserve literal percent sequences
// and Unicode, while local paths must never reach Studio's document name.
const names = [
  ['POSIX', '/Users/example/Downloads/신청서(SW) (개인)_1.hwp', '신청서(SW) (개인)_1.hwp'],
  ['Windows', String.raw`C:\Users\example\Downloads\신청서(1).hwpx`, '신청서(1).hwpx'],
  ['UNC', String.raw`\\server\share\문서.hml`, '문서.hml'],
  ['basename', '신청서_원본(1).hwp', '신청서_원본(1).hwp'],
  ['literal percent', '/Downloads/100%_%2F_%5C_%252F.hwp', '100%_%2F_%5C_%252F.hwp'],
  ['URL metacharacters', '/Downloads/문서 #1 & 2+.hwp', '문서 #1 & 2+.hwp'],
  ['decomposed Unicode', '/Downloads/졸업작품.hwp'.normalize('NFD'), '졸업작품.hwp'.normalize('NFD')],
  ['empty', '', null],
  ['missing', undefined, null],
  ['directory only', '/Downloads/', null],
];

for (const browserName of ['chrome', 'firefox']) {
  const apiName = browserName === 'chrome' ? 'chrome' : 'browser';
  const base = `${browserName === 'chrome' ? 'chrome' : 'moz'}-extension://test/viewer.html`;
  const launcher = await import(`../../rhwp-${browserName}/sw/viewer-launcher.js`);
  for (const reuse of [false, true]) {
    for (const [label, input, expected] of names) {
      test(`${browserName}: ${reuse ? 'reuse' : 'new tab'} preserves ${label} filename (#6961)`, async () => {
        const previous = globalThis[apiName];
        const created = [];
        const updated = [];
        globalThis[apiName] = {
          runtime: { getURL: () => base },
          tabs: {
            create: async options => { created.push(options); },
            query: async () => [{ id: 42, url: base }],
            update: async (id, options) => { updated.push({ id, ...options }); },
          },
        };
        try {
          const sourceUrl = 'https://example.com/download?id=1&token=a%2Fb';
          const options = Object.freeze({ url: sourceUrl, filename: input });
          if (reuse) await launcher.openViewerOrReuse(options);
          else launcher.openViewer(options);
          const calls = reuse ? updated : created;
          assert.equal(calls.length, 1);
          assert.equal((reuse ? created : updated).length, 0);
          if (reuse) assert.equal(calls[0].id, 42);
          const params = new URL(calls[0].url).searchParams;
          assert.equal(params.get('filename'), expected);
          assert.equal(params.get('url'), sourceUrl);
          assert.equal(options.filename, input, 'caller metadata must remain unchanged');
        } finally {
          if (previous === undefined) delete globalThis[apiName];
          else globalThis[apiName] = previous;
        }
      });
    }
  }
}
