import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

/** 옵트인 스킨 이름 → CSS 파일. 스킨 추가 시 여기와 THEME_SKINS, theme-init.js 를 함께 갱신한다. */
const OPT_IN_SKINS: ReadonlyArray<{ skin: string; file: string }> = [
  { skin: 'flat', file: 'src/styles/theme-flat.css' },
  { skin: 'oldschool', file: 'src/styles/theme-oldschool.css' },
];

for (const { skin, file } of OPT_IN_SKINS) {
  test(`${skin} 스킨 CSS의 모든 규칙은 data-theme-skin="${skin}" 스코프 아래에 있다`, () => {
    const css = source(file);
    const selectors = css
      .replace(/\/\*[\s\S]*?\*\//g, '')
      .split('}')
      .map((block) => block.split('{')[0]?.trim())
      .filter((selector): selector is string => !!selector);

    assert.ok(selectors.length > 0, '규칙이 최소 1개는 있어야 한다');
    for (const selector of selectors) {
      assert.match(
        selector,
        new RegExp(`\\[data-theme-skin="${skin}"\\]`),
        `옵트인 스코프가 없는 셀렉터: ${selector}`,
      );
    }
  });

  test(`${skin} 스킨의 라이트 팔레트는 다크 모드를 덮지 않도록 가드된다`, () => {
    const css = source(file);
    assert.match(
      css,
      new RegExp(`:root\\[data-theme-skin="${skin}"\\]:not\\(\\[data-theme-effective="dark"\\]\\)`),
    );
    // 가드 없는 :root 단독 블록에는 형태 값(radius/shadow/font)만 허용한다.
    const unguardedRoot = css.match(
      new RegExp(`:root\\[data-theme-skin="${skin}"\\]\\s*\\{([\\s\\S]*?)\\}`),
    );
    if (unguardedRoot) {
      assert.doesNotMatch(unguardedRoot[1], /--ui-bg|--accent-primary|--ruler-bg/);
    }
  });

  test(`보기 메뉴와 FOUC 스크립트가 ${skin} 스킨을 안다`, () => {
    const html = source('index.html');
    assert.match(
      html,
      new RegExp(`data-cmd="view:skin-${skin}"[^>]*data-theme-skin-choice="${skin}"`),
    );
    const themeInit = source('public/theme-init.js');
    assert.match(themeInit, new RegExp(`'${skin}'`));
  });
}

test('보기 메뉴는 기본(클래식) 스킨 항목을 노출한다', () => {
  const html = source('index.html');
  assert.match(html, /data-cmd="view:skin-default"[^>]*data-theme-skin-choice="default"/);
});

test('첫 실행 스킨 안내는 skinChosen=false 일 때만 표시 대상이다', async () => {
  // DOM 의존이 없는 판정 함수만 로드해 검증한다.
  const { shouldShowSkinOnboarding } = await import('../src/core/user-settings.ts');
  assert.equal(shouldShowSkinOnboarding({ skinChosen: false }), true);
  assert.equal(shouldShowSkinOnboarding({ skinChosen: true }), false);
});

test('스킨 CSS 는 하드코딩 색 폴백 없이 토큰만 참조한다 (다크에서 밝은 값 노출 방지)', () => {
  for (const { skin, file } of OPT_IN_SKINS) {
    const css = source(file);
    // var(--token, #fff) 처럼 색 리터럴을 폴백으로 두면 토큰이 없는 모드에서 그대로 그려진다.
    const literalFallback = css.match(/var\(--[a-z0-9-]+,\s*(#[0-9a-fA-F]{3,8}|rgb|hsl|white|black)/);
    assert.equal(
      literalFallback,
      null,
      `${skin}: 색 리터럴 폴백 발견 — ${literalFallback?.[0]}`,
    );
  }
});

test('올드스쿨 스킨은 다크 모드용 베벨 토큰을 별도로 정의한다', () => {
  const css = source('src/styles/theme-oldschool.css');
  const darkBlock = css.match(
    /:root\[data-theme-skin="oldschool"\]\[data-theme-effective="dark"\]\s*\{([\s\S]*?)\}/,
  );
  assert.ok(darkBlock, '다크 전용 --os-* 블록이 있어야 한다');
  // 라이트 블록이 정의한 스킨 로컬 토큰은 다크에서도 모두 정의되어야 한다.
  for (const token of ['--os-chrome', '--os-bevel-light', '--os-bevel-dark', '--os-bevel-darker']) {
    assert.match(darkBlock[1], new RegExp(`${token}\\s*:`), `다크 블록에 ${token} 누락`);
  }
});

test('올드스쿨 스킨의 단일 행 서식 바는 양쪽 베벨을 포함해 36px을 유지한다', () => {
  const oldschool = source('src/styles/theme-oldschool.css');
  const styleBar = source('src/styles/style-bar.css');
  assert.match(oldschool, /--style-bar-full-padding-top:\s*2px/);
  assert.match(
    styleBar,
    /@media \(min-width: 992px\)[\s\S]*padding:\s*var\(--style-bar-full-padding-top, 3px\) 8px 3px 22px/,
  );
});
