# rhwp-studio Grouped Style Ribbon Implementation Plan

**Goal:** Replace the mobile style toolbar’s flat wrap with an accessible grouped ribbon whose controls remain visible, compact, and visually distinct.

**Architecture:** Keep every existing control ID and command listener. Add structural wrappers in `index.html`, use CSS-only desktop/tablet/mobile layout rules, remove the decorative strike arrow, and replace the alignment data-URI rectangles with theme-aware SVG masks.

**Tech Stack:** HTML, CSS flex/grid, SVG masks, Node.js built-in test runner, TypeScript, Vite

## Global Constraints

- Every existing formatting command remains visible.
- Existing control IDs, `<label for>` relationships, command wiring, active-state updates, and dropdown behavior remain unchanged.
- Strikethrough is a direct toggle and has no dropdown indicator.
- Character effects, text color, and highlight retain dropdown indicators.
- No JavaScript viewport measurement, `ResizeObserver`, new dependency, or theme palette is added.
- Unrelated dirty files remain untouched.

---

### Task 1: Add the grouped-ribbon regression contract

**Files:**
- Create: `rhwp-studio/tests/style-toolbar-grouped-ribbon.test.ts`

**Interfaces:**
- Consumes: `index.html`, `src/styles/style-bar.css`, and `src/styles/responsive.css`.
- Produces: source-level guards for structure, mobile density, command affordances, and alignment icon implementation.

- [ ] **Step 1: Write the failing test**

```ts
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
const styles = readFileSync(new URL('../src/styles/style-bar.css', import.meta.url), 'utf8');
const responsive = readFileSync(new URL('../src/styles/responsive.css', import.meta.url), 'utf8');

const buttonMarkup = (id: string): string => {
  const match = html.match(new RegExp(`<button[^>]*id="${id}"[\\s\\S]*?<\\/button>`));
  assert.ok(match, `missing #${id}`);
  return match[0];
};

test('style toolbar uses ordered field and command groups', () => {
  const fields = html.indexOf('class="sb-field-grid"');
  const characters = html.indexOf('class="sb-command-band sb-character-band"');
  const paragraphs = html.indexOf('class="sb-command-band sb-paragraph-band"');

  assert.ok(fields >= 0);
  assert.ok(fields < characters);
  assert.ok(characters < paragraphs);
  assert.match(html, /class="sb-command-group sb-character-group"/);
  assert.match(html, /class="sb-command-group sb-color-group"/);
  assert.match(html, /class="sb-command-group sb-align-group"/);

  const fieldGrid = html.slice(fields, characters);
  for (const id of ['style-name', 'font-lang', 'font-name', 'font-size', 'linespacing-select']) {
    assert.match(fieldGrid, new RegExp(`id="${id}"`));
  }
});

test('only real menus retain dropdown affordances', () => {
  const strike = buttonMarkup('btn-strike');
  assert.doesNotMatch(strike, /sb-has-arrow|sb-dd/);
  assert.match(strike, /sb-strike/);

  for (const id of ['btn-charfx', 'btn-text-color', 'btn-highlight']) {
    const button = buttonMarkup(id);
    assert.match(button, /sb-has-arrow/);
    assert.match(button, /sb-dd/);
  }
  assert.match(buttonMarkup('btn-charfx'), /sb-effect-icon/);
  assert.match(buttonMarkup('btn-text-color'), /sb-color-visual/);
  assert.match(buttonMarkup('btn-highlight'), /sb-highlight-visual/);
});

test('mobile ribbon is compact without hiding command glyphs', () => {
  assert.match(responsive, /#style-bar\s*\{[^}]*flex-direction:\s*column;/s);
  assert.match(
    responsive,
    /\.sb-field-grid\s*\{[^}]*grid-template-columns:\s*68px 54px minmax\(96px,\s*1fr\) 72px 72px;/s,
  );
  assert.match(responsive, /#style-bar \.sb-btn\s*\{[^}]*width:\s*29px;[^}]*height:\s*29px;/s);
  assert.match(responsive, /#style-bar \.sb-has-arrow\s*\{[^}]*width:\s*38px;/s);
  assert.doesNotMatch(responsive, /\.sb-ga\s*\{\s*display:\s*none;/);
});

test('alignment icons use the shared theme-aware mask contract', () => {
  assert.match(styles, /\.sb-align\s*\{[^}]*background-color:\s*currentColor;[^}]*mask/s);
  for (const name of ['left', 'center', 'right', 'justify', 'distribute', 'split']) {
    assert.match(styles, new RegExp(`\\.sb-al-${name}\\s*\\{[^}]*--sb-align-icon:`));
  }
  assert.doesNotMatch(styles, /\.sb-al-(?:left|center|right|justify|distribute|split)\s*\{[^}]*background-image:/s);
});
```

- [ ] **Step 2: Run the test and confirm the regression**

Run: `/opt/homebrew/bin/node --test tests/style-toolbar-grouped-ribbon.test.ts`

Expected: FAIL because the group wrappers, direct strike markup, compact mobile rules, and mask icons do not exist.

### Task 2: Implement the grouped style ribbon

**Files:**
- Modify: `rhwp-studio/index.html`
- Modify: `rhwp-studio/src/styles/style-bar.css`
- Modify: `rhwp-studio/src/styles/responsive.css`

**Interfaces:**
- Consumes: existing control IDs and `Toolbar` query selectors.
- Produces: `.sb-field-grid`, `.sb-command-band`, `.sb-command-group`, `.sb-character-group`, `.sb-color-group`, and `.sb-align-group`.

- [ ] **Step 1: Group existing controls without changing IDs**

In `#style-bar`, place the five existing `.sb-field` elements in:

```html
<div class="sb-field-grid">
  <!-- existing style, language, font, size, and line-spacing fields -->
</div>
```

Place existing character and color controls in:

```html
<div class="sb-command-band sb-character-band">
  <div class="sb-command-group sb-character-group">
    <!-- bold, italic, underline, strike, character effects -->
  </div>
  <div class="sb-command-group sb-color-group">
    <!-- text color and highlight -->
  </div>
</div>
```

Place the six existing alignment buttons in:

```html
<div class="sb-command-band sb-paragraph-band">
  <div class="sb-command-group sb-align-group">
    <!-- left, center, right, justify, distribute, split -->
  </div>
</div>
```

Change the command-specific markup to:

```html
<button id="btn-strike" class="sb-btn" title="취소선">
  <span class="sb-ga sb-strike">가</span>
</button>
```

Add `sb-effect-icon` to `#charfx-icon`. Wrap the existing text-color glyph and `#color-bar` in `.sb-color-visual`; wrap the highlight glyph and `#highlight-bar` in `.sb-highlight-visual`. Keep the existing `.sb-dd` spans on those three real menus.

- [ ] **Step 2: Add base group and icon styling**

Add these layout contracts to `src/styles/style-bar.css`:

```css
.sb-field-grid,
.sb-command-band,
.sb-command-group {
  display: flex;
  align-items: center;
}

.sb-field-grid { gap: 2px; flex-shrink: 0; }
.sb-command-band { gap: 5px; flex-shrink: 0; }
.sb-command-group { gap: 1px; flex-shrink: 0; }
.sb-command-group + .sb-command-group {
  padding-left: 5px;
  border-left: 1px solid var(--ui-border);
}
```

Strengthen `.sb-strike` with a positioned `::after` center line. Add an `sb-effect-icon::after` sparkle. Make `.sb-color-visual` and `.sb-highlight-visual` vertical visual stacks inside horizontal menu buttons.

Replace `.sb-align` background images with:

```css
.sb-align {
  width: 18px;
  height: 18px;
  background-color: currentColor;
  -webkit-mask: var(--sb-align-icon) center / 18px 18px no-repeat;
  mask: var(--sb-align-icon) center / 18px 18px no-repeat;
}
```

Define `--sb-align-icon` on all six `.sb-al-*` classes using the approved 18×18 path-only SVG masks.

- [ ] **Step 3: Add the mobile grouped-ribbon layout**

In the `max-width: 767px` block, use:

```css
#style-bar {
  flex-direction: column;
  align-items: stretch;
  min-height: 40px;
  padding: 5px 6px 6px;
  gap: 5px;
}

.sb-field-grid {
  display: grid;
  grid-template-columns: 68px 54px minmax(96px, 1fr) 72px 72px;
  gap: 4px;
  width: 100%;
}

.sb-field-grid .sb-field {
  min-width: 0;
  flex-direction: column;
  align-items: stretch;
  gap: 2px;
}

.sb-field-grid .sb-field-label {
  margin-left: 1px;
  font-size: 9px;
  line-height: 1;
}

.sb-character-band,
.sb-paragraph-band {
  width: 100%;
}

#style-bar .sb-btn {
  width: 29px;
  min-width: 29px;
  height: 29px;
  min-height: 29px;
  padding: 0;
}

#style-bar .sb-has-arrow {
  width: 38px;
  min-width: 38px;
}
```

Make all five field controls fill their grid column at 27px height. Remove the mobile `.sb-ga { display: none; }` rule and remove `.sb-btn` from the later coarse-pointer minimum-height rule so it cannot override ribbon density.

- [ ] **Step 4: Run focused tests**

Run:

```bash
/opt/homebrew/bin/node --test \
  tests/style-toolbar-grouped-ribbon.test.ts \
  tests/responsive-toolbar-layout.test.ts \
  tests/accessibility-shell.test.ts
```

Expected: all grouped-ribbon, wrapping, and accessibility tests pass.

- [ ] **Step 5: Run production and full-suite verification**

Run: `PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin /opt/homebrew/bin/npm run build`

Expected: TypeScript and Vite production build pass.

Run: `PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin /opt/homebrew/bin/npm test`

Expected: full Node suite passes.

- [ ] **Step 6: Verify the running editor**

At 412px, verify with 실제 브라우저:

- `styleBar.scrollWidth <= styleBar.clientWidth`
- field grid, character band, and paragraph band have ordered, stable rows
- no visible `.sb-btn` has zero-sized icon content
- strike has no arrow; character effects, text color, and highlight do
- strike toggles without opening a menu

Repeat containment checks at 883px and 1024px, then restore the user’s 412×914 viewport.

- [ ] **Step 7: Commit implementation only after verification**

Stage only:

- `rhwp-studio/index.html`
- `rhwp-studio/src/styles/style-bar.css`
- `rhwp-studio/src/styles/responsive.css`
- `rhwp-studio/tests/style-toolbar-grouped-ribbon.test.ts`

Leave all unrelated worktree changes untouched.
