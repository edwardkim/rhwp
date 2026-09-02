import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

import {
  isSameRenderDocument,
  type RenderDocumentIdentity,
} from '../src/view/render-document-identity.ts';

// [#3315] 문서 범위 파생 상태의 수명은 문서 경계가 정한다.
//
// `imageRetryCounts`·`prefetchedImageSignatures` 는 항목마다 문서 신원(digest·generation)을 들고
// 다닌다. 그래서 새 문서에서 **잘못 맞아떨어지지는 않는다** — 정확성은 이미 서 있다.
//
// 문제는 수명이다. 두 맵을 비우는 자리는 `dispose()` 뿐인데 `CanvasView.dispose` 에는 호출부가
// 없다(문서 닫기·뷰 교체 기능이 생길 때를 위한 자리다). 그래서 문서를 열 때마다 그 문서의 페이지
// 수만큼 항목이 쌓이고, 세션이 끝날 때까지 하나도 줄지 않는다.
//
// 거두는 자리는 편집 경계(`resetImageRetryState`)도, 페이지 풀 해제도 아니다 — 둘 다 페이지마다
// 재렌더를 한 번 더 돌게 만든다(#3672 가 없앤 그 비용). 문서 경계(`beginDocument`)다.

const studioRoot = fileURLToPath(new URL('..', import.meta.url));

const DOC_A: RenderDocumentIdentity = { digest: 'blake3:aaaa', generation: 1 };
const DOC_B: RenderDocumentIdentity = { digest: 'blake3:bbbb', generation: 2 };

const signature = (identity: RenderDocumentIdentity) => ({
  documentDigest: identity.digest,
  documentGeneration: identity.generation,
  imageKeys: '{"keys":["bin:0:1:src"]}',
  hadRawSvg: false,
});

/** 두 맵에 페이지 항목을 심는다 (private 필드 — 런타임에는 평범한 속성이다). */
const seedPageEntries = (renderer: any, identity: RenderDocumentIdentity, pages: number): void => {
  for (let pageIdx = 0; pageIdx < pages; pageIdx += 1) {
    renderer.imageRetryCounts.set(pageIdx, `${identity.digest}|${identity.generation}|k`);
    renderer.prefetchedImageSignatures.set(pageIdx, signature(identity));
  }
};

const entryCounts = (renderer: any): { retry: number; prefetch: number } => ({
  retry: renderer.imageRetryCounts.size,
  prefetch: renderer.prefetchedImageSignatures.size,
});

async function withPageRenderer(
  run: (make: (wasm: any) => any) => void | Promise<void>,
): Promise<void> {
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { PageRenderer } = await vite.ssrLoadModule('/src/view/page-renderer.ts');
    await run((wasm: any) => new PageRenderer(wasm));
  } finally {
    await vite.close();
  }
}

test('[#3315] 문서가 갈리면 옛 문서의 페이지 항목을 거둔다', async () => {
  await withPageRenderer((make) => {
    const wasm = { documentDigest: DOC_A.digest, documentGeneration: DOC_A.generation };
    const renderer = make(wasm);

    renderer.beginDocument();
    seedPageEntries(renderer, DOC_A, 40);
    assert.deepEqual(entryCounts(renderer), { retry: 40, prefetch: 40 });

    // 다음 문서를 연다 — 세대는 로드마다 오르므로 신원이 반드시 갈린다.
    wasm.documentDigest = DOC_B.digest;
    wasm.documentGeneration = DOC_B.generation;
    renderer.beginDocument();

    assert.deepEqual(
      entryCounts(renderer),
      { retry: 0, prefetch: 0 },
      '옛 문서의 항목은 새 문서에서 다시 읽히지 않는다 — 읽히지 않을 뿐 사라지지도 않으면 '
        + '세션 내내 쌓인다',
    );
  });
});

test('[#3315] 긴 문서를 본 뒤 짧은 문서를 열면 긴 문서의 페이지가 남지 않는다', async () => {
  await withPageRenderer((make) => {
    const wasm = { documentDigest: DOC_A.digest, documentGeneration: DOC_A.generation };
    const renderer = make(wasm);

    // 400쪽짜리 그림 문서를 끝까지 훑는다.
    renderer.beginDocument();
    seedPageEntries(renderer, DOC_A, 400);

    // 그리고 5쪽짜리 문서를 연다. 페이지 색인이 겹치는 0..4 는 덮이지만 5..399 는 덮을
    // 항목이 없다 — 거두지 않으면 그 395개가 세션 내내 남는다.
    wasm.documentDigest = DOC_B.digest;
    wasm.documentGeneration = DOC_B.generation;
    renderer.beginDocument();
    seedPageEntries(renderer, DOC_B, 5);

    assert.deepEqual(
      entryCounts(renderer),
      { retry: 5, prefetch: 5 },
      '항목 수는 지금 열린 문서의 페이지 수여야 한다 — 세션에서 본 가장 긴 문서의 페이지 수가 '
        + '아니다',
    );
  });
});

test('[#3315] 같은 문서를 다시 준비하면 항목을 그대로 둔다', async () => {
  await withPageRenderer((make) => {
    const wasm = { documentDigest: DOC_A.digest, documentGeneration: DOC_A.generation };
    const renderer = make(wasm);

    renderer.beginDocument();
    seedPageEntries(renderer, DOC_A, 12);
    renderer.beginDocument();

    assert.deepEqual(
      entryCounts(renderer),
      { retry: 12, prefetch: 12 },
      '신원이 같으면 거둘 이유가 없다 — 거두면 같은 페이지를 다시 데우고 다시 그린다',
    );
  });
});

test('[#3315] 편집 경계는 여전히 재시도 키를 비우지 않는다 (#3672 회귀 방지)', async () => {
  await withPageRenderer((make) => {
    const wasm = { documentDigest: DOC_A.digest, documentGeneration: DOC_A.generation };
    const renderer = make(wasm);

    renderer.beginDocument();
    seedPageEntries(renderer, DOC_A, 8);
    // `refreshPages` → `releaseAllRenderedPages` 가 편집마다 부르는 자리.
    renderer.resetImageRetryState();

    assert.deepEqual(
      entryCounts(renderer),
      { retry: 8, prefetch: 8 },
      '편집마다 비우면 페이지마다 재렌더가 한 번 더 돈다 — #3672 가 없앤 비용이다',
    );
  });
});

test('[#3315] 문서 신원을 모르면 범위를 비워 둔다', async () => {
  await withPageRenderer((make) => {
    const wasm: { documentDigest: string | null; documentGeneration: number } = {
      documentDigest: DOC_A.digest,
      documentGeneration: DOC_A.generation,
    };
    const renderer = make(wasm);

    renderer.beginDocument();
    seedPageEntries(renderer, DOC_A, 5);

    // 신원을 알 수 없는 문서(digest 계산 실패). 항목을 어느 문서 것이라 표시할 수 없다.
    wasm.documentDigest = null;
    wasm.documentGeneration = 3;
    renderer.beginDocument();

    assert.deepEqual(entryCounts(renderer), { retry: 0, prefetch: 0 });
    // 그리고 그 상태가 "같은 문서"로 굳지 않아야 한다 — 다음 준비에서도 거둔다.
    seedPageEntries(renderer, DOC_A, 5);
    renderer.beginDocument();
    assert.deepEqual(entryCounts(renderer), { retry: 0, prefetch: 0 });
  });
});

test('[#3315] 문서 신원 판정은 한 규칙이다', () => {
  assert.equal(isSameRenderDocument(DOC_A, { ...DOC_A }), true);
  assert.equal(isSameRenderDocument(DOC_A, DOC_B), false);
  // 세대만 달라도 다른 문서 인스턴스다 — 같은 파일을 다시 연 경우가 여기 걸린다.
  assert.equal(isSameRenderDocument(DOC_A, { digest: DOC_A.digest, generation: 2 }), false);
  // 신원을 모르면 어느 것과도 같지 않다 — 두 `null` 도 같지 않다.
  assert.equal(isSameRenderDocument({ digest: null, generation: 1 }, { digest: null, generation: 1 }), false);
  assert.equal(isSameRenderDocument(null, DOC_A), false);
});
