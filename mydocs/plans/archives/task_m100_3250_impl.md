# 문서 복구 대화상자 화면 내 배치 구현 계획

**Goal:** 복구 후보가 많아도 문서 복구 대화상자를 viewport 안에 유지하고 후보 목록만 세로 스크롤하게 한다.

**Architecture:** 기존 `ModalDialog` 구조는 유지하고 복구 대화상자에만 class를 추가한다. CSS flex 축소와 `overflow-y: auto`로 후보 목록에 남은 높이를 할당하며 JavaScript 높이 계산은 추가하지 않는다.

**Tech Stack:** TypeScript DOM API, CSS flexbox/overflow, Node.js built-in test runner, Vite

## Global Constraints

- 제목, 안내 문구, `복구`·`삭제`·`나중에` 동작은 항상 보여야 한다.
- 후보 목록만 세로로 스크롤한다.
- 후보 정렬, 기본 선택, 복구 및 draft 삭제 정책은 변경하지 않는다.
- 공통 `ModalDialog`의 다른 소비자에게 새 scroll 정책을 적용하지 않는다.
- 기존 모바일 bottom-sheet 표현과 프로젝트 색상·간격을 유지한다.
- JavaScript 높이 측정이나 resize listener를 추가하지 않는다.

---

### Task 1: 복구 대화상자 viewport containment

**Files:**
- Create: `rhwp-studio/tests/recovery-modal-layout.test.ts`
- Modify: `rhwp-studio/src/recovery/recovery-ui.ts:14-122`
- Modify: `rhwp-studio/src/styles/dialogs.css:17-60`
- Modify: `rhwp-studio/src/styles/responsive.css:362-388`

**Interfaces:**
- Consumes: `ModalDialog.show()`가 생성하는 `.dialog-wrap`, `.dialog-title`, `.dialog-body`, `.dialog-footer`
- Produces: `.recovery-dialog`, `.recovery-dialog-body`, `.recovery-draft-list`, `.recovery-draft-copy`, `.recovery-draft-title` layout hooks

- [ ] **Step 1: 실패하는 layout 계약 테스트 작성**

```typescript
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const recoveryUi = readFileSync(new URL('../src/recovery/recovery-ui.ts', import.meta.url), 'utf8');
const dialogs = readFileSync(new URL('../src/styles/dialogs.css', import.meta.url), 'utf8');
const responsive = readFileSync(new URL('../src/styles/responsive.css', import.meta.url), 'utf8');

test('recovery dialog scopes viewport containment and scrolling to the draft list', () => {
  assert.match(recoveryUi, /this\.dialog\.classList\.add\('recovery-dialog'\)/);
  assert.match(recoveryUi, /body\.classList\.add\('recovery-dialog-body'\)/);
  assert.match(recoveryUi, /list\.classList\.add\('recovery-draft-list'\)/);
  assert.match(recoveryUi, /text\.classList\.add\('recovery-draft-copy'\)/);
  assert.match(recoveryUi, /title\.classList\.add\('recovery-draft-title'\)/);

  assert.match(
    dialogs,
    /\.dialog-wrap\.recovery-dialog\s*\{[^}]*display:\s*flex;[^}]*flex-direction:\s*column;[^}]*max-height:\s*calc\(100dvh - 32px\);[^}]*overflow:\s*hidden;/s,
  );
  assert.match(
    dialogs,
    /\.recovery-dialog\s*>\s*\.recovery-dialog-body\s*\{[^}]*display:\s*flex;[^}]*flex-direction:\s*column;[^}]*min-height:\s*0;[^}]*overflow:\s*hidden;/s,
  );
  assert.match(
    dialogs,
    /\.recovery-draft-list\s*\{[^}]*flex:\s*1 1 auto;[^}]*min-height:\s*0;[^}]*overflow-y:\s*auto;/s,
  );
  assert.match(dialogs, /\.recovery-draft-copy\s*\{[^}]*min-width:\s*0;/s);
  assert.match(dialogs, /\.recovery-draft-title\s*\{[^}]*overflow-wrap:\s*anywhere;/s);
  assert.match(
    responsive,
    /\.dialog-wrap\.recovery-dialog\s*\{[^}]*max-height:\s*90dvh;[^}]*overflow:\s*hidden;/s,
  );
});
```

- [ ] **Step 2: 테스트가 현행 코드에서 올바르게 실패하는지 확인**

Run:

```bash
cd rhwp-studio
node --test tests/recovery-modal-layout.test.ts
```

Expected: FAIL because `recovery-dialog` layout hooks and scoped CSS rules do not exist.

- [ ] **Step 3: 복구 UI에 전용 layout hook 추가**

`recovery-ui.ts`의 기존 DOM 생성에 다음 class만 추가한다.

```typescript
body.classList.add('recovery-dialog-body');
list.classList.add('recovery-draft-list');
text.classList.add('recovery-draft-copy');
title.classList.add('recovery-draft-title');
```

후보의 grid 두 번째 열은 긴 파일명이 컨테이너를 밀지 않게 바꾼다.

```typescript
label.style.gridTemplateColumns = 'auto minmax(0, 1fr)';
```

`showAsync()`의 `super.show()` 직후 대화상자에 범위 class를 붙인다.

```typescript
this.dialog.classList.add('recovery-dialog');
```

- [ ] **Step 4: viewport와 목록 scroll CSS 구현**

`dialogs.css`에 복구 전용 규칙을 추가한다.

```css
.dialog-wrap.recovery-dialog {
  display: flex;
  flex-direction: column;
  max-height: calc(100dvh - 32px);
  overflow: hidden;
}

.recovery-dialog > .dialog-title,
.recovery-dialog > .dialog-footer {
  flex: 0 0 auto;
}

.recovery-dialog > .recovery-dialog-body {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.recovery-draft-list {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  -webkit-overflow-scrolling: touch;
}

.recovery-draft-copy {
  min-width: 0;
}

.recovery-draft-title {
  overflow-wrap: anywhere;
}
```

`responsive.css`의 모바일 대화상자 구간에 복구 전용 override를 추가한다.

```css
.dialog-wrap.recovery-dialog {
  max-height: 90dvh;
  overflow: hidden;
}
```

- [ ] **Step 5: 집중 테스트가 통과하는지 확인**

Run:

```bash
cd rhwp-studio
node --test tests/recovery-modal-layout.test.ts tests/recovery-ui.test.ts
```

Expected: 6 tests pass, 0 fail.

- [ ] **Step 6: TypeScript와 production build 검증**

Run:

```bash
cd rhwp-studio
npx tsc --noEmit
npm run build
```

Expected: both commands exit 0.

- [ ] **Step 7: 현재 실제 브라우저 viewport에서 runtime 계약 확인**

복구 대화상자가 열린 `882 × 758` viewport에서 다음 값을 확인한다.

```text
dialog.top >= 0
dialog.bottom <= innerHeight
footer.bottom <= innerHeight
draftList.scrollHeight > draftList.clientHeight
draftList overflow-y == auto
dialog.scrollWidth <= dialog.clientWidth
```

- [ ] **Step 8: 변경 범위와 whitespace 확인**

Run:

```bash
git diff --check
git diff -- rhwp-studio/src/recovery/recovery-ui.ts rhwp-studio/src/styles/dialogs.css rhwp-studio/src/styles/responsive.css rhwp-studio/tests/recovery-modal-layout.test.ts
```

Expected: whitespace errors 없음; recovery modal containment 관련 변경만 표시.

- [ ] **Step 9: 구현 단계 커밋**

```bash
git add rhwp-studio/src/recovery/recovery-ui.ts \
  rhwp-studio/src/styles/dialogs.css \
  rhwp-studio/src/styles/responsive.css \
  rhwp-studio/tests/recovery-modal-layout.test.ts
git commit -m "fix(studio): keep recovery dialog within viewport"
```
