# Task #689 구현계획서 — 그리드 모드 `getPageAtY` X 무시 정정 + Task #685 누락 6곳 동반 정정

- **대응 수행계획서**: [`task_m100_689.md`](task_m100_689.md)
- **이슈**: [#689](https://github.com/edwardkim/rhwp/issues/689)
- **단계 수**: 3 (CLAUDE.md 의 3~6 범위)
- **scope 확장 승인일**: 2026-05-08 — 본 작업 안에서 #685 누락분 6곳 buggy pageLeft 도 동반 정정.
- **공통 검증 명령**: `cd rhwp-studio && npx tsc --noEmit && npx vite build`
- **e2e 명령**: `node e2e/<name>.test.mjs --mode=headless` (vite dev server 별도 가동 필수, `:7700`)

---

## 단계별 사전 결정

### 헬퍼 시그니처 (확정)

```ts
/**
 * 그리드 모드 X+Y 인지 페이지 인덱스 반환.
 * 단일 컬럼 모드: getPageAtY 와 동치 (X 무관).
 * 그리드 모드: row(Y) 결정 후 같은 row 안에서 X 가 속하는 페이지 반환.
 *              gap 영역 (페이지 사이 빈 공간) click 은 가장 가까운 페이지로 fallback.
 */
getPageAtPoint(docX: number, docY: number): number {
  const rowLastIdx = this.getPageAtY(docY);
  if (!this.gridMode) return rowLastIdx; // 단일 컬럼은 X 무관

  // 같은 row 의 페이지 범위 (rowLastIdx 부터 row 시작까지)
  const rowOffset = this.pageOffsets[rowLastIdx];
  let rowFirst = rowLastIdx;
  while (rowFirst > 0 && this.pageOffsets[rowFirst - 1] === rowOffset) rowFirst--;

  // X 가 페이지 안에 속하는지 검사
  for (let i = rowFirst; i <= rowLastIdx; i++) {
    const left = this.pageLefts[i] ?? 0;
    const right = left + (this.pageWidths[i] ?? 0);
    if (docX >= left && docX <= right) return i;
  }

  // X 가 어느 페이지에도 속하지 않음 (gap / margin 영역) — 가장 가까운 페이지
  let bestIdx = rowFirst;
  let bestDist = Infinity;
  for (let i = rowFirst; i <= rowLastIdx; i++) {
    const left = this.pageLefts[i] ?? 0;
    const right = left + (this.pageWidths[i] ?? 0);
    const dist = docX < left ? left - docX : (docX > right ? docX - right : 0);
    if (dist < bestDist) { bestDist = dist; bestIdx = i; }
  }
  return bestIdx;
}
```

### 정정 사이트 분류 (확정)

| 분류 | 파일 | 라인 | `getPageAtY` 치환 | `pageLeft` 동반 정정 |
|------|------|------|-------------------|---------------------|
| input-handler-mouse | input-handler-mouse.ts | 20, 126, 173, 354, 428, 470, 807, 886, 928, 1143, 1193, 1240 | ✅ 14곳 | (#685 에서 이미 정정) |
| 그림 객체 중심 | input-handler.ts | 612 | ✅ | ✅ |
| 표 객체 중심 | input-handler.ts | 875 | ✅ | ✅ |
| 마우스 이벤트 | input-handler.ts | 972 | ✅ | ✅ |
| 마우스 이벤트 | input-handler.ts | 1542 | ✅ | ✅ |
| 표 이동 드래그 | input-handler-table.ts | 400 | ✅ | ✅ |
| 그림 이동 드래그 | input-handler-picture.ts | 594 | ✅ | ✅ |
| **소계** | | | **20곳** | **6곳 추가 정정** |

**미수정 (X 의미 없음)**:
- canvas-view.ts L120, L209 (`vpCenter` viewport 중심)
- input-handler-keyboard.ts L798 (`vpCenter` 키보드 viewport 중심)

**조사 후 분류 (Stage 1)**:
- coordinate-system.ts L18 `documentToPage(dx, dy)` — public method, 호출자 추적 후 분류

### TS 단위 테스트 부재 처리

rhwp-studio 는 puppeteer e2e 만 존재. 헬퍼 검증은 Stage 3 의 e2e assert 강화로 자동화 (zoom=0.5 모든 col + zoom=0.25 모든 col strict assert).

---

## Stage 1 — 헬퍼 추가 + `coordinate-system` 분류 확정

**목표**: `getPageAtPoint` 헬퍼를 도입. coordinate-system.ts 의 분류를 결정.

### Step 1.1 — `coordinate-system.ts` 호출자 추적

```bash
grep -rn "documentToPage\|coordinateSystem" rhwp-studio/src/ 2>/dev/null
```

분류:
- 호출자가 e.client / contentX/Y 같은 마우스 좌표 → `getPageAtPoint(dx, dy)` 로 헬퍼 호출 변경
- 호출자가 viewport center 또는 Y-only → 그대로 두거나 별도 결정

**결정 결과를 Stage 1 보고서에 기록**.

### Step 1.2 — `virtual-scroll.ts` 에 헬퍼 추가

[`rhwp-studio/src/view/virtual-scroll.ts`](../../rhwp-studio/src/view/virtual-scroll.ts) 의 `getPageAtY(docY)` 메서드 직후에 위 시그니처 그대로 추가.

`getPageAtY` 자체는 **수정 금지** — viewport-center 호출자 (canvas-view, input-handler-keyboard, 그리고 새 헬퍼 자체) 가 그대로 사용.

### Step 1.3 — coordinate-system 처리 (조건부)

Step 1.1 결과 마우스 컨텍스트로 분류되면 `documentToPage` 가 X 도 받도록 시그니처 변경 또는 내부에서 `getPageAtPoint` 호출. 호출자에 영향 없게 처리 가능한 방식 선택.

### Step 1.4 — typecheck + build

```bash
cd rhwp-studio && npx tsc --noEmit && npx vite build
```

기대: 무에러.

### Step 1.5 — 단일 컬럼 동치성 sanity (e2e)

```bash
node e2e/body-outside-click-fallback.test.mjs --mode=headless
```

기대: exit 0, 회귀 0. (헬퍼는 단일 컬럼에서 `getPageAtY` 와 동치 동작.)

### Step 1.6 — Stage 1 보고서 + 커밋

`mydocs/working/task_m100_689_stage1.md`:
- 헬퍼 시그니처 + 동작 설명
- coordinate-system.ts 분류 결과 (마우스 / viewport-center / 미분류)
- typecheck/build/e2e 무회귀 확인

```bash
git add rhwp-studio/src/view/virtual-scroll.ts \
        [coordinate-system.ts 변경 시] \
        mydocs/plans/task_m100_689.md \
        mydocs/plans/task_m100_689_impl.md \
        mydocs/working/task_m100_689_stage1.md
git commit -m "Task #689 Stage 1: getPageAtPoint 헬퍼 도입 + 호출자 분류 확정"
```

→ **승인 게이트**.

---

## Stage 2 — `getPageAtY` 20곳 치환 + buggy `pageLeft` 6곳 동반 정정

**목표**: 본 이슈의 본질 정정. 그리드 모드 모든 col click 정합 달성.

### Step 2.1 — `input-handler-mouse.ts` 14곳 `getPageAtY` 치환

**Before (반복 패턴)**:
```ts
const pi = this.virtualScroll.getPageAtY(cy);
// 또는
const pageIdx = this.virtualScroll.getPageAtY(contentY);
```

**After**:
```ts
const pi = this.virtualScroll.getPageAtPoint(cx, cy);
// 또는
const pageIdx = this.virtualScroll.getPageAtPoint(contentX, contentY);
```

각 라인의 X 변수명 (`cx` / `contentX`) 보존. 14곳 모두 `replace_all` 안전 (변수명 일관 — `cy`/`contentY` ↔ `cx`/`contentX` 동일 함수 내 짝).

`replace_all` 로 두 그룹 처리:
- `const pi = this.virtualScroll.getPageAtY(cy);` → `const pi = this.virtualScroll.getPageAtPoint(cx, cy);`
- `const pageIdx = this.virtualScroll.getPageAtY(contentY);` → `const pageIdx = this.virtualScroll.getPageAtPoint(contentX, contentY);`

### Step 2.2 — `input-handler.ts` 4곳 (L612, L875, L972, L1542) 동반 정정

각 사이트에서 두 변경:

**Before (예 — L972)**:
```ts
const pageIdx = this.virtualScroll.getPageAtY(contentY);
const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
const pageLeft = (scrollContent.clientWidth - pageDisplayWidth) / 2;
```

**After**:
```ts
const pageIdx = this.virtualScroll.getPageAtPoint(contentX, contentY);
const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);
```

`pageDisplayWidth` 변수는 다른 용도 (hit test bbox) 사용 가능성 있으므로 라인별 검증 — 사용 없으면 제거, 있으면 보존.

L612, L875 는 `cX, cY` (객체 중심 좌표) 변수 사용 — 이름 보존하면서 `getPageAtPoint(cX, cY)` 로 치환.

### Step 2.3 — `input-handler-table.ts:400` 동반 정정

같은 패턴:
```ts
const pi = this.virtualScroll.getPageAtPoint(cx, cy);  // ← getPageAtY → getPageAtPoint
const po = this.virtualScroll.getPageOffset(pi);
const pw = this.virtualScroll.getPageWidth(pi);
const pl = this.virtualScroll.getPageLeftResolved(pi, sc.clientWidth);  // ← (sc.clientWidth - pw)/2 정정
```

`pw` 는 동일 함수 내 다른 용도 가능성 검증 후 보존/제거 결정.

### Step 2.4 — `input-handler-picture.ts:594` 동반 정정

L400 과 동일 패턴 적용.

### Step 2.5 — Stage 1 결정 시 `coordinate-system.ts` 처리

Stage 1 분류 결과에 따라 처리.

### Step 2.6 — grep sweep

```bash
echo "=== getPageAtY 마우스 컨텍스트 잔여 (0 건 기대) ==="
grep -rn "getPageAtY" rhwp-studio/src/ | grep -v "viewport\|vpCenter\|virtual-scroll.ts"
# canvas-view.ts (viewport center) + input-handler-keyboard.ts (vpCenter) + virtual-scroll.ts (정의) 만 매칭되어야 함

echo "=== buggy pageLeft 잔여 (0 건 기대) ==="
grep -rnE "clientWidth\s*-\s*\w+\)\s*/\s*2" rhwp-studio/src/

echo "=== getPageAtPoint 호출 수 (20 기대) ==="
grep -rc "getPageAtPoint" rhwp-studio/src/
```

### Step 2.7 — typecheck + build + 기존 e2e 무회귀

```bash
cd rhwp-studio
npx tsc --noEmit
npx vite build
node e2e/body-outside-click-fallback.test.mjs --mode=headless
```

### Step 2.8 — Stage 2 보고서 + 커밋

`mydocs/working/task_m100_689_stage2.md`:
- 사이트별 변경 표 (20곳 `getPageAtY` + 6곳 `pageLeft`)
- grep sweep 결과
- typecheck/build/e2e 무회귀

```bash
git commit -m "Task #689 Stage 2: getPageAtY 20곳 → getPageAtPoint 치환 + Task #685 누락 6곳 buggy pageLeft 동반 정정"
```

→ **승인 게이트**.

---

## Stage 3 — e2e strict assert 활성화 + 시각 검증

**목표**: Task #685 + #689 결합 효과 (그리드 모드 모든 col click 정합) 자동 회귀화.

### Step 3.1 — 기존 SKIP 분기 제거 → 모든 col strict assert

[`e2e/grid-mode-click-coord.test.mjs`](../../rhwp-studio/e2e/grid-mode-click-coord.test.mjs) `probeClickAtPage` 의 SKIP 분기 제거:

**Before**:
```ts
if (probe.isLastCol) {
  assert(afterCorrectClick.rectPageIdx === pageIdx, `[${label}] CORRECT click → cursor.rectPageIdx=...`);
} else {
  console.log(`  SKIP: [${label}] non-last col rectPageIdx strict assert ... — Issue #689 후속`);
}
```

**After**:
```ts
assert(
  afterCorrectClick.rectPageIdx === pageIdx,
  `[${label}] CORRECT click → cursor.rectPageIdx=${afterCorrectClick.rectPageIdx} (기대 ${pageIdx}, col=${probe.col}/columns=${probe.columns})`
);
```

`probe.isLastCol` 필드 자체는 보존 (디버그 로그 가치).

### Step 3.2 — 추가 probe (중간 col 케이스)

기존 `[5]` 블록에 추가:

```ts
// page 0 (col 0) — 이미 있음
// page 1 (col 1) — 이미 있음 (zoom=0.5 last col)
// page 2 — 이미 있음 (col 0 of row 1)
// 추가: zoom=0.25 의 다양한 col 케이스
await page.evaluate(() => window.__inputHandler.viewportManager.setZoom(0.25));
await page.evaluate(() => new Promise(r => setTimeout(r, 600)));
await probeClickAtPage(page, 'page 0 (zoom=0.25 col 0)', 0, 100, 200);
await probeClickAtPage(page, 'page 2 (zoom=0.25 col 2 mid)', 2, 100, 200);
await probeClickAtPage(page, 'page 4 (zoom=0.25 col 4 last)', 4, 100, 200);
```

(zoom=0.25 last col probe 는 기존에 [3b] 블록에 있음 — 중복 방지 위해 정리.)

### Step 3.3 — e2e 실행 + PASS=13+/FAIL=0/SKIP=0 확인

```bash
node e2e/grid-mode-click-coord.test.mjs --mode=headless
```

기대:
- exit 0
- PASS ≥ 13 (zoom=0.5 col 0/1, zoom=0.25 col 0/2/4, zoom=1.0 col 0 의 cursor.pos + rectPageIdx 모두 PASS)
- FAIL = 0
- SKIP = 0

### Step 3.4 — 시각 검증 (호스트 모드)

```bash
node e2e/grid-mode-click-coord.test.mjs --mode=host
```

또는 작업지시자가 직접 vite dev 환경에서:
- `samples/hwpctl_action_table_v11.hwp` 로드
- zoom=0.5 / 0.25 그리드 모드에서 모든 col 페이지 클릭 → 캐럿 정상 배치 시각 확인
- zoom=1.0 일반 클릭 무회귀

### Step 3.5 — 진단노트 갱신

[`mydocs/troubleshootings/grid_mode_click_coord.md`](../troubleshootings/grid_mode_click_coord.md) 끝부분에 추가:

```markdown
### 완전 정정 완료 (2026-05-08, Task #689)

후속 결함 (`getPageAtY` X 무시) 정정 완료:
- `virtualScroll.getPageAtPoint(docX, docY)` 헬퍼 도입
- 마우스 컨텍스트 20곳 `getPageAtY` → `getPageAtPoint` 치환
- Task #685 sweep 누락 6곳 buggy `pageLeft` 동반 정정 (`getPageLeftResolved` 적용)
- e2e (`grid-mode-click-coord.test.mjs`) strict assert 활성화 — 모든 col 정합

→ Task #685 + #689 결합으로 그리드 모드 click 한컴 호환 완성.
```

### Step 3.6 — Stage 3 보고서 + 커밋

`mydocs/working/task_m100_689_stage3.md`:
- e2e assert 활성화 결과 (PASS 카운트)
- 시각 검증 결과
- 진단노트 갱신 내용

```bash
git commit -m "Task #689 Stage 3: e2e strict assert 활성화 + 시각 검증 + 진단노트 완전 정정 기록"
```

→ **승인 게이트**.

---

## 최종 단계 — 결과보고서 + #685 운용 보고

### 산출물

1. `mydocs/report/task_m100_689_report.md` — 최종 결과보고서:
   - 본질 정정 결과 (`getPageAtPoint` + 누락 6곳 동반 정정)
   - 검증 결과 (모든 자동/시각 PASS)
   - Task #685 + #689 결합 완성 명시
   - 회귀 영역 점검

2. `mydocs/orders/20260508.md` 갱신 — Task #689 상태 [완료] 표시 + Task #685 누락 발견/동반 정정 메모.

3. **Issue #685 운용 후속 코멘트** (선택) — #685 의 close 코멘트에서 "#689 에서 누락분 동반 정정 완료" 후속 안내 등록 가능.

### 커밋

```bash
git commit -m "Task #689: 최종 결과보고서 + orders 갱신 (closes #689)"
```

→ **승인 게이트**: 작업지시자 확인 후 GitHub 이슈 close + `local/devel` 머저 진행.

---

## 회귀 위험 요약

| 영역 | 위험 | 완화 |
|------|------|------|
| 단일 컬럼 모드 (`!gridMode`) | `getPageAtPoint` 가 `getPageAtY` 와 비트 단위 동치 안 되면 회귀 | 헬퍼 첫 분기에서 `if (!this.gridMode) return rowLastIdx` 명시 — Stage 1 sanity e2e 로 검증 |
| gap 영역 click (페이지 사이) | 어느 페이지에도 속하지 않을 때 동작 | "가장 가까운 페이지" fallback 정책 — 기존 `getPageAtY` 결과 (row last) 와 다를 수 있으나 사용자 경험 향상 |
| `pageDisplayWidth` 변수 잔여 | hit test bbox 사용처 보호 | 변수 보존 (Task #685 와 동일 정책) |
| viewport-center 영역 | 헬퍼 통일 욕심 시 회귀 | canvas-view.ts, input-handler-keyboard.ts 미수정 명시 |
| coordinate-system.ts | 분류 미확정 영역 | Stage 1 에서 호출자 추적 후 결정, 분류 결과 보고서 기록 |
| e2e 임계값 | 부동소수점 오차 / 중간 col 케이스 | 기존 0.01 px 임계 유지, 필요 시 Stage 3 에서 미세 조정 |

---

## 단계 완료 기준 종합

- [ ] Stage 1: 헬퍼 추가, coordinate-system 분류, typecheck/build/sanity, 보고서 + 커밋
- [ ] Stage 2: 20곳 `getPageAtY` 치환 + 6곳 `pageLeft` 동반 정정, grep sweep 0건, e2e 무회귀, 보고서 + 커밋
- [ ] Stage 3: e2e PASS=13+/FAIL=0/SKIP=0, 시각 검증, 진단노트 갱신, 보고서 + 커밋
- [ ] 최종: 결과보고서 + orders 갱신 + 이슈 #689 close 절차 + 머저
