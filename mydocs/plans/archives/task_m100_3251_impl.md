# 빈 문서 스타일 기본값 구현 계획

**Goal:** 문서를 열기 전에도 비활성 스타일 콤보에 `바탕글`을 표시한다.

**Architecture:** 초기 HTML에 하나의 fallback option을 둔다. 문서 로드 후 기존 `initStyleDropdown()`이 실제 문서 스타일로 이를 교체한다.

**Tech Stack:** HTML, TypeScript source-contract tests, Node test runner

## Global Constraints

- 기존 비활성 상태와 스타일은 변경하지 않는다.
- 문서 로드 후 실제 스타일 목록이 우선한다.
- 관련 없는 작업 파일은 스테이징하지 않는다.

---

### Task 1: 초기 스타일 옵션

**Files:**
- Modify: `rhwp-studio/index.html`
- Test: `rhwp-studio/tests/style-toolbar-grouped-ribbon.test.ts`

**Interfaces:**
- Consumes: `Toolbar.initStyleDropdown()`의 `replaceChildren()` 계약
- Produces: 초기 `#style-name` option `value="0"`, label `바탕글`

- [ ] **Step 1: 실패 테스트 작성**

```ts
assert.match(
  html,
  /<select id="style-name"[^>]*>\s*<option value="0">바탕글<\/option>\s*<\/select>/,
);
```

- [ ] **Step 2: 실패 확인**

Run: `/opt/homebrew/bin/node --test tests/style-toolbar-grouped-ribbon.test.ts`

Expected: 초기 option이 없어 FAIL.

- [ ] **Step 3: 최소 구현**

```html
<select id="style-name" class="sb-combo" title="스타일">
  <option value="0">바탕글</option>
</select>
```

- [ ] **Step 4: 검증**

Run:

```bash
/opt/homebrew/bin/node --test tests/style-toolbar-grouped-ribbon.test.ts
PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm test
PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run build
```

Expected: 모든 테스트와 빌드 PASS.

- [ ] **Step 5: 실제 화면 확인**

문서 URL 없이 Studio를 열어 `#style-name`이 비활성 상태에서 `바탕글`을
표시하는지 실제 브라우저 브라우저로 확인한다.

- [ ] **Step 6: 커밋**

```bash
git add rhwp-studio/index.html rhwp-studio/tests/style-toolbar-grouped-ribbon.test.ts
git commit -m "fix: show default style without document"
```
