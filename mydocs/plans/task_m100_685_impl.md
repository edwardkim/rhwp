# Task #685 구현 계획서 — 그리드 모드 click 좌표 단일 컬럼 가정 일괄 정정

- **대응 수행계획서**: [`task_m100_685.md`](task_m100_685.md)
- **이슈**: [#685](https://github.com/edwardkim/rhwp/issues/685)
- **단계 수**: 3 (CLAUDE.md 의 3~6 범위)
- **공통 검증 명령**: `cd rhwp-studio && npx tsc --noEmit` (타입체크), `npx vite build` (빌드)
- **e2e 실행 환경**: 별도 터미널에서 `cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700` 후 `node e2e/<name>.test.mjs --mode=headless`

---

## 단계별 사전 결정

### 헬퍼 시그니처 (확정)

[`rhwp-studio/src/view/virtual-scroll.ts`](../../rhwp-studio/src/view/virtual-scroll.ts) 에 `getPageLeft` 직후 추가:

```ts
/**
 * 페이지의 X 좌표를 그리드/단일 컬럼 모드 통합으로 반환.
 * 그리드 모드: pageLefts[i] 그대로.
 * 단일 컬럼 모드(sentinel −1): (containerWidth - pageWidth) / 2 fallback.
 */
getPageLeftResolved(pageIdx: number, containerWidth: number): number {
  const pl = this.pageLefts[pageIdx] ?? -1;
  if (pl >= 0) return pl;
  const pw = this.pageWidths[pageIdx] ?? 0;
  return (containerWidth - pw) / 2;
}
```

### TS 단위 테스트 부재 처리

rhwp-studio 는 TS 단위 테스트 프레임워크 없음 (e2e puppeteer 만 존재). 헬퍼 검증은 다음 두 경로로 수행:

- **Stage 1 자체 검증**: 기존 [`input-handler.ts:2579-2581`](../../rhwp-studio/src/engine/input-handler.ts#L2579-L2581) 의 verbose sentinel 패턴을 새 헬퍼로 치환 → 동작 동치성으로 헬퍼의 단일 컬럼/그리드 동작 양쪽 검증.
- **Stage 3 e2e 검증**: `grid-mode-click-coord.test.mjs` 의 `dumpGridState()` 안에서 `vs.getPageLeftResolved(i, sc.clientWidth)` 호출값이 (그리드 모드) `vs.getPageLeft(i)` 와 일치하고, (단일 컬럼) `(clientWidth - pw)/2` 와 일치함을 assert.

---

## Stage 1 — `getPageLeftResolved` 헬퍼 추가 + 기존 verbose 사용처 1곳 정리

**목표**: 헬퍼 도입과 동치성 검증을 한 단계로 묶음. 동작 변경 없음 (refactor only).

**수정 파일**:
- `rhwp-studio/src/view/virtual-scroll.ts` (헬퍼 추가)
- `rhwp-studio/src/engine/input-handler.ts` (L2579-L2581 단순화)

### Step 1.1 — virtual-scroll.ts 에 헬퍼 추가

[`rhwp-studio/src/view/virtual-scroll.ts:155-157`](../../rhwp-studio/src/view/virtual-scroll.ts#L155-L157) 직후에 다음 메서드 추가:

```ts
  /**
   * 페이지의 X 좌표를 그리드/단일 컬럼 모드 통합으로 반환.
   * 그리드 모드: pageLefts[i] 그대로.
   * 단일 컬럼 모드(sentinel −1): (containerWidth - pageWidth) / 2 fallback.
   */
  getPageLeftResolved(pageIdx: number, containerWidth: number): number {
    const pl = this.pageLefts[pageIdx] ?? -1;
    if (pl >= 0) return pl;
    const pw = this.pageWidths[pageIdx] ?? 0;
    return (containerWidth - pw) / 2;
  }
```

기존 `getPageLeft(pageIdx)` 는 raw accessor 로 보존 (`canvas-view.ts`, `field-marker-renderer.ts`, `caret-renderer.ts` 호출자 무회귀).

### Step 1.2 — input-handler.ts:2579-2581 단순화

[`rhwp-studio/src/engine/input-handler.ts:2572-2589`](../../rhwp-studio/src/engine/input-handler.ts#L2572-L2589) `formBboxToOverlayRect` 메서드 안:

**Before**:
```ts
const scrollContent = this.container.querySelector('#scroll-content');
const contentWidth = scrollContent?.clientWidth ?? 0;
const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
const pageLeft = this.virtualScroll.getPageLeft(pageIdx) >= 0
  ? this.virtualScroll.getPageLeft(pageIdx)
  : (contentWidth - pageDisplayWidth) / 2;
```

**After**:
```ts
const scrollContent = this.container.querySelector('#scroll-content');
const contentWidth = scrollContent?.clientWidth ?? 0;
const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, contentWidth);
```

`pageDisplayWidth` 변수는 이 메서드 내에서 위 한 곳에만 쓰이므로 제거. `bbox.w * zoom` 등은 `bbox.w` 사용 (pageDisplayWidth 의존 없음 — Read 검증 완료).

### Step 1.3 — 타입체크 + 빌드

```bash
cd rhwp-studio
npx tsc --noEmit
npx vite build
```

기대: 둘 다 무에러. `getPageLeftResolved` 가 `VirtualScroll` 타입에 추가되었으므로 호출부 타입 일치.

### Step 1.4 — 동치성 sanity check

수동: 기존 `formBboxToOverlayRect` 가 호출되는 양식 개체 (samples/exam_form.hwp 등) 를 열어 양식 오버레이가 그리드 모드(zoom=0.5)/단일 컬럼(zoom=1.0) 모두 정상 위치에 표시되는지 시각 확인. 단, 동치 refactor 이므로 기능 회귀는 일어나지 않아야 함.

자동 회귀가 필요하면 [`rhwp-studio/e2e/body-outside-click-fallback.test.mjs`](../../rhwp-studio/e2e/body-outside-click-fallback.test.mjs) 를 headless 로 1회 실행하여 무회귀 확인 (vite dev 별도 터미널 가동 필요).

### Step 1.5 — Stage 1 완료 보고서 + 커밋

`mydocs/working/task_m100_685_stage1.md` 작성. 내용:
- 변경 요약 (2 파일, ~+10 LOC, −2 LOC)
- 타입체크/빌드 통과 확인
- 동치성 검증 결과
- 다음 단계 안내

커밋:
```bash
git add rhwp-studio/src/view/virtual-scroll.ts \
        rhwp-studio/src/engine/input-handler.ts \
        mydocs/working/task_m100_685_stage1.md \
        mydocs/plans/task_m100_685.md \
        mydocs/plans/task_m100_685_impl.md
git commit -m "Task #685 Stage 1: getPageLeftResolved 헬퍼 추가 + formBboxToOverlayRect 단순화"
```

→ **승인 게이트**: 작업지시자 확인 후 Stage 2 진행.

---

## Stage 2 — `input-handler-mouse.ts` 14곳 헬퍼 치환

**목표**: 본 이슈의 핵심 정정. 그리드 모드 click 좌표 어긋남 해소.

**수정 파일**: `rhwp-studio/src/engine/input-handler-mouse.ts` 만.

### 치환 규칙 (모든 14곳 동일 패턴)

**Before** (변형 패턴 3종):
```ts
// 패턴 A (대다수)
const pl = (sc.clientWidth - pw) / 2;

// 패턴 B
const pageLeft = (scrollContent.clientWidth - pageDisplayWidth) / 2;

// 패턴 C (변수명 sc/pw 또는 scrollContent/pageDisplayWidth 혼재)
const pageLeft = ((sc as HTMLElement).clientWidth - pageDisplayWidth) / 2;
```

**After**:
```ts
// 변수명은 해당 라인의 기존 이름 유지 (pl 또는 pageLeft) + 페이지 인덱스도 해당 라인의 기존 이름 (pi 또는 pageIdx 또는 picBbox.pageIndex)
const pl = this.virtualScroll.getPageLeftResolved(pi, sc.clientWidth);
// 또는
const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);
```

**중요 주의**:
- `pw` / `pageDisplayWidth` 변수는 같은 함수 내 다른 좌표 계산(예: hit test 의 `x + pw`, bbox `x + pageDisplayWidth` 등)에서도 쓰이므로 **변수 자체는 보존**, 단 `(... - pw)/2` 표현식만 헬퍼 호출로 교체.
- 페이지 인덱스 변수명은 라인별로 확인 후 그대로 사용 (`pi` / `pageIdx` / `picBbox.pageIndex`).

### Step 2.1 — 14곳 라인별 치환 (라인 번호 역순 권장: 변경 후 라인 번호 시프트 영향 최소화)

대상 라인 (보고서 기준 — 실측 시 ±10 라인 허용):

| 라인 | 함수 | 페이지 인덱스 변수 | 치환 후 라인 |
|------|------|-------------------|-------------|
| 1243 | `handleResizeHover` | `pageIdx` | `const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);` |
| 1196 | `onMouseMove (table hover)` | `pi` | `const pl = this.virtualScroll.getPageLeftResolved(pi, scrollContent.clientWidth);` |
| 1146 | `onMouseMove (picture hover)` | `pi` | `const pl = this.virtualScroll.getPageLeftResolved(pi, scrollContent.clientWidth);` |
| 931 | `onMouseMove (connector)` | `pi` | `const pl = this.virtualScroll.getPageLeftResolved(pi, sc.clientWidth);` |
| 889 | `onContextMenu` | `pageIdx` | `const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);` |
| 811 | `onDblClick` | `pageIdx` | `const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, (sc as HTMLElement).clientWidth);` |
| 475 | `onMouseDown (일반 click)` | `pageIdx` | `const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);` |
| 431 | `onMouseDown (table resize)` | `pageIdx` | `const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);` |
| 357 | `onMouseDown (single picture)` | `pi` | `const pl = this.virtualScroll.getPageLeftResolved(pi, sc.clientWidth);` |
| 296 | `onMouseDown (rotate)` | `picBbox.pageIndex` | `const pl = this.virtualScroll.getPageLeftResolved(picBbox.pageIndex, sc.clientWidth);` |
| 279 | `onMouseDown (line endpoint)` | `picBbox.pageIndex` | `const pl = this.virtualScroll.getPageLeftResolved(picBbox.pageIndex, sc.clientWidth);` |
| 176 | `onMouseDown (multi picture)` | `pi` | `const pl = this.virtualScroll.getPageLeftResolved(pi, sc.clientWidth);` |
| 129 | `onMouseDown (선택된 표)` | `pi` | `const pl = this.virtualScroll.getPageLeftResolved(pi, sc.clientWidth);` |
| 23 | `onConnectorMouseDown` | `pi` | `const pl = this.virtualScroll.getPageLeftResolved(pi, sc.clientWidth);` |

각 라인은 Edit 도구의 정확한 매칭(주변 1~2 줄 컨텍스트 포함)으로 단건 치환 — `replace_all` 금지. 함수마다 변수명/scrollContent 별칭이 다르므로 일괄 sed 위험.

### Step 2.2 — `pageDisplayWidth` 잔여 사용처 점검

각 함수에서 `pageDisplayWidth` 변수가 click 좌표 외에도 hit test bbox 등에 쓰이면 보존. 안 쓰이면 unused-var 경고 회피를 위해 함께 제거.

```bash
grep -n "pageDisplayWidth\|const pw =" rhwp-studio/src/engine/input-handler-mouse.ts | head -40
```

→ Stage 2 작업 시 확인. 보통 같은 함수 안에서 `bbox.x + pageDisplayWidth` 같은 hit test 영역 결정에 사용되므로 보존 권장.

### Step 2.3 — 정정 누락 sweep

```bash
grep -nE "\(.*clientWidth\s*-\s*\w*[Pp]age\w*[Ww]idth\w*\s*\)\s*/\s*2" rhwp-studio/src/engine/input-handler-mouse.ts
grep -nE "\(sc\.clientWidth\s*-\s*pw\)\s*/\s*2" rhwp-studio/src/engine/input-handler-mouse.ts
```

기대 출력: 둘 다 0 건. 1건 이상이면 Stage 2 미완 — 해당 라인 추가 치환.

### Step 2.4 — 타입체크 + 빌드

```bash
cd rhwp-studio
npx tsc --noEmit
npx vite build
```

기대: 둘 다 무에러.

### Step 2.5 — 기존 e2e 무회귀 점검

별도 터미널에서 vite dev server 가동 후:

```bash
cd rhwp-studio
node e2e/body-outside-click-fallback.test.mjs --mode=headless
```

기대: 모든 PASS, FAIL 0.

가능하면 [`text-flow.test.mjs`](../../rhwp-studio/e2e/text-flow.test.mjs) 도 추가 회귀 확인.

### Step 2.6 — Stage 2 완료 보고서 + 커밋

`mydocs/working/task_m100_685_stage2.md` 작성. 내용:
- 14곳 라인별 치환 결과 표 (Before / After 일치 확인)
- 잔여 `(clientWidth - pw)/2` 패턴 grep 결과 0건 확인
- 타입체크/빌드 통과
- e2e 무회귀 결과
- Stage 3 (assert 강화) 안내

커밋:
```bash
git add rhwp-studio/src/engine/input-handler-mouse.ts \
        mydocs/working/task_m100_685_stage2.md
git commit -m "Task #685 Stage 2: input-handler-mouse 14곳 헬퍼 치환 — 그리드 모드 click 좌표 정정"
```

→ **승인 게이트**: 작업지시자 확인 후 Stage 3 진행.

---

## Stage 3 — e2e assert 강화 + 시각 검증

**목표**: 본 정정이 실제 그리드 모드 click 동작을 정상화했음을 자동 회귀로 증명. 단일 컬럼 모드 무회귀 보장.

**수정 파일**: `rhwp-studio/e2e/grid-mode-click-coord.test.mjs` 만.

### Step 3.1 — `dumpGridState` 에 헬퍼 동치성 assert 추가

기존 `dumpGridState` (라인 22-53) 의 `page.evaluate` 안에서 각 row 계산 시 `helperResolved` 필드 추가:

```ts
const correct = vs.getPageLeft(i);
const pw = vs.getPageWidth(i);
const buggy = (clientWidth - pw) / 2;
const helperResolved = vs.getPageLeftResolved(i, clientWidth);  // ← 추가
const col = i % Math.max(columns, 1);
const delta = correct >= 0 ? (buggy - correct) : 0;
const helperDelta = correct >= 0 ? (helperResolved - correct) : (helperResolved - buggy);
rows.push({ i, col, pw, correct, buggy, helperResolved, helperDelta, delta });
```

함수 종료 직전 (return state 직전) 에 도입한 `assert` 사용:

```ts
import { runTest, loadHwpFile, screenshot, assert } from './helpers.mjs';
// ...
// 헬퍼 동치성 — 모든 페이지에 대해 helperDelta 가 0 (sub-pixel 오차 흡수 위해 |x| < 0.01)
const maxHelperDelta = Math.max(...state.rows.map(r => Math.abs(r.helperDelta)));
assert(maxHelperDelta < 0.01,
  `[${label}] getPageLeftResolved == 기대값 (max|delta|=${maxHelperDelta.toFixed(4)}px)`);
```

기대: 모든 모드에서 PASS (그리드: helperResolved == pageLefts[i], 단일 컬럼: helperResolved == buggy fallback 공식).

### Step 3.2 — `probeClickAtPage` 에 click→cursor 정합 assert 추가

기존 `probeClickAtPage` (라인 55-139) 마지막 console.log 뒤에:

```ts
// 본 fix 이후, CORRECT click → 의도한 페이지에 cursor 배치
assert(
  afterCorrectClick.rectPageIdx === pageIdx,
  `[${label}] CORRECT click → cursor.rectPageIdx=${afterCorrectClick.rectPageIdx} (기대 ${pageIdx})`
);
// 추가: cursor.pos 가 null 이 아니어야 함 (페이지 영역 내 정상 hit)
assert(
  afterCorrectClick.pos !== null,
  `[${label}] CORRECT click → cursor.pos !== null`
);
```

기대: zoom=0.5 page 0(col 0), page 1(col 1), page 2 모두 PASS.

### Step 3.3 — zoom=1.0 (단일 컬럼) baseline 검증

기존 `[4] zoom=1.0` 블록 (라인 173-179) 직후에:

```ts
// 단일 컬럼 baseline — 모든 페이지 helperResolved == buggy 공식 (sentinel fallback 동치)
const z10MaxDelta = Math.max(...stateZ10.rows.slice(0, 8).map(r => Math.abs((r.helperResolved - r.buggy))));
assert(z10MaxDelta < 0.01, `zoom=1.0 단일 컬럼: helperResolved == fallback 공식 (max|delta|=${z10MaxDelta.toFixed(4)}px)`);
```

기대: 단일 컬럼 모드에서 헬퍼와 fallback 공식 비트 단위 일치.

### Step 3.4 — zoom=1.0 click 정합도 추가 검증

`[4] zoom=1.0` 블록 뒤에 (또는 새 블록):

```ts
// zoom=1.0 click baseline — 단일 컬럼에서 page 0 click 시 cursor 정합
console.log('\n[4b] zoom=1.0 click baseline');
await probeClickAtPage(page, 'page 0 (single col)', 0, 100, 200);
```

`probeClickAtPage` 의 assert 가 자동 실행되어 단일 컬럼 모드 click 무회귀 확인.

### Step 3.5 — 보고서 그룹화

`runTest` 가 자동으로 `e2e-output/grid-mode-click-coord-report.html` 생성. PASS/FAIL 카운트가 보고서에 집계되도록 `assert` 호출만 추가하면 충분.

### Step 3.6 — e2e 실행 + 모든 PASS 확인

```bash
# 별도 터미널
cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700

# 본 터미널
cd rhwp-studio && node e2e/grid-mode-click-coord.test.mjs --mode=headless
```

기대 결과:
- 모든 `assert` PASS (FAIL 0)
- `process.exitCode == 0` 종료
- 콘솔에 zoom=0.5/0.25 grid 모드 + zoom=1.0 baseline 모두 PASS 로그

### Step 3.7 — 호스트 Chrome CDP 모드 시각 검증

```bash
# 호스트 Chrome (또는 동등 환경) 에 원격 디버깅 활성 후
cd rhwp-studio && node e2e/grid-mode-click-coord.test.mjs --mode=host
```

수동 시각 확인 (1회):
- zoom=0.5 시 page 1 (col 1) 클릭 → 캐럿이 page 1 의 의도한 위치에 정확히 배치
- zoom=0.25 시 양 끝 컬럼 (col 0, col 4) 클릭 → 캐럿이 클릭한 페이지 안에 배치
- zoom=1.0 일반 클릭 무회귀

### Step 3.8 — 진단 노트 끝부분에 정정 완료 기록 추가

`mydocs/troubleshootings/grid_mode_click_coord.md` 끝부분 (라인 157 뒤) 에 추가:

```markdown
## 정정 완료 (2026-05-08, Task #685)

본 결함은 Task #685 에서 정정됨:
- `virtualScroll.getPageLeftResolved(pageIdx, containerWidth)` 헬퍼 도입
- `input-handler-mouse.ts` 14곳 + `input-handler.ts` 1곳 헬퍼 일괄 치환 (총 15곳)
- e2e (`grid-mode-click-coord.test.mjs`) 회귀 assert 추가 — zoom=0.5/0.25 모든 col CORRECT click → 의도한 페이지에 캐럿 배치 확인

회귀 영역 확인됨: 단일 컬럼 (zoom > 0.5) 무회귀, 그리드 모드 양 끝 컬럼 정합.
```

### Step 3.9 — Stage 3 완료 보고서 + 커밋

`mydocs/working/task_m100_685_stage3.md` 작성. 내용:
- e2e assert 추가 위치 / 검증 항목 표
- headless 실행 결과 (모든 PASS 카운트)
- host 모드 시각 확인 결과 (스크린샷 경로)
- 진단 노트 갱신 내용

커밋:
```bash
git add rhwp-studio/e2e/grid-mode-click-coord.test.mjs \
        mydocs/troubleshootings/grid_mode_click_coord.md \
        mydocs/working/task_m100_685_stage3.md
git commit -m "Task #685 Stage 3: 그리드 모드 click 좌표 회귀 assert 강화 + 시각 검증"
```

→ **승인 게이트**: 작업지시자 확인 후 최종 보고서 단계.

---

## 최종 단계 — 결과 보고서 + 이슈 close

**목표**: 타스크 종결.

### 산출물

1. `mydocs/report/task_m100_685_report.md` — 최종 결과보고서:
   - 본질 진단 요약 (이슈 #685 정량 측정 인용)
   - 정정 결과 (15곳 치환 + 헬퍼 1개 + e2e 강화)
   - 검증 결과 (typecheck/build/e2e/시각 모두 PASS)
   - 회귀 영역 확인
   - 후속 조사 제안 (키보드/IME/Touch 동일 결함 가능성)

2. `mydocs/orders/2026-05-07.md` (또는 작업 시작일 파일) 갱신 — 타스크 #685 상태 [완료] 표시.

### 커밋

```bash
git add mydocs/report/task_m100_685_report.md \
        mydocs/orders/2026-05-07.md
git commit -m "Task #685: 최종 결과보고서 + orders 갱신 (closes #685)"
```

→ **승인 게이트**: 작업지시자 확인 후 GitHub 이슈 close 진행 (`gh issue close 685` 또는 `closes #685` 커밋이 merge 되면 자동 close).

### 머저 절차 (작업지시자 권한)

```bash
# 작업지시자가 직접 수행
git checkout local/devel
git merge local/task685 --no-ff -m "Merge local/devel: Task #685 — 그리드 모드 click 좌표 일괄 정정"
# (원격 push 는 devel branch 통해서 별도 시점)
```

본 구현 절차 안에서는 머저를 수행하지 않음 (작업지시자 결정).

---

## 회귀 위험 요약 (모든 Stage 공통)

| 영역 | 위험 | 완화 |
|------|------|------|
| 단일 컬럼 click 좌표 | 헬퍼 fallback 공식이 기존과 비트 단위 일치하지 않으면 회귀 | Stage 1 sanity check (formBboxToOverlayRect 동치) + Stage 3 zoom=1.0 baseline assert |
| `pageDisplayWidth` 변수 잔여 사용 | 변수 제거 시 hit test 깨짐 | Step 2.2 cross-check, 변수 보존 우선 |
| 변수명 mismatch | `pi`/`pageIdx`/`picBbox.pageIndex` 혼용 | 일괄 치환 금지, 라인별 단건 Edit |
| e2e 임계값 | 부동소수점 오차로 false fail | 0.01 px 임계로 sub-pixel 흡수 |
| canvas-view/field-marker/caret renderer 4곳 | 헬퍼 미사용 → 만약 그리드 모드 동작이 깨지는 일이 있으면 본 작업과 무관 회귀 | 본 작업 미수정 — 별도 후속 |

---

## 단계 완료 기준 종합

- [ ] Stage 1 통과: 헬퍼 추가 + verbose 패턴 정리, 타입체크/빌드 PASS, 동치성 시각 sanity OK, 보고서 + 커밋
- [ ] Stage 2 통과: 14곳 치환, grep sweep 0건, 타입체크/빌드 PASS, 기존 e2e 무회귀, 보고서 + 커밋
- [ ] Stage 3 통과: e2e assert 강화 후 모든 PASS, host 모드 시각 확인, 진단 노트 갱신, 보고서 + 커밋
- [ ] 최종: 결과보고서 + orders 갱신 + 이슈 #685 close 절차 진행
