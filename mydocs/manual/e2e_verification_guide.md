# E2E 조판 자동 검증 가이드

## 개요

빈 문서에서 프로그래밍으로 문서를 생성하고, 렌더링 결과가 기대값과 일치하는지 **자동으로 검증**하는 체계입니다.

```
시나리오 정의 → 자동 실행 → 렌더 트리 측정 → 규칙 기반 검증 → 결과 보고
```

## 사전 조건

1. WASM 빌드 완료 (`docker compose --env-file .env.docker run --rm wasm`)
2. Vite dev server 실행 중 (`cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700`)
3. Chrome CDP 연결 가능 (호스트 또는 headless)

## 실행 방법

```bash
cd rhwp-studio

# 호스트 Chrome CDP
CHROME_CDP=http://localhost:19222 node e2e/tac-verify.test.mjs --mode=host

# headless Chrome
node e2e/tac-verify.test.mjs --mode=headless
```

## 파일 구조

```
rhwp-studio/e2e/
├── scenario-runner.mjs      # 시나리오 실행기 + 측정기 + 검증기
├── tac-verify.test.mjs       # 인라인 TAC 표 검증 시나리오 모음
├── tac-inline-create.test.mjs # 한컴 방식 입력 E2E (단계별 스크린샷)
├── helpers.mjs               # 공통 헬퍼 (moveCursorTo 등)
└── screenshots/              # 자동 생성 스크린샷
```

## 시나리오 작성

### 시나리오 정의

JSON 객체로 문서 작성 순서를 선언합니다. 한컴에서의 입력 순서를 그대로 반영합니다.

```javascript
const scenario = {
  name: 'TC-V01: 인라인 TAC 표 기본',
  steps: [
    { type: 'text',  value: 'TC #20',           label: 'title' },
    { type: 'enter',                             label: 'enter1' },
    { type: 'text',  value: '표 앞 텍스트   ',    label: 'before-text' },
    { type: 'inlineTable',
      rows: 2, cols: 2,
      colWidths: [6777, 6777],                   // HWPUNIT 단위 열 폭
      cells: ['1', '2', '3', '4'],               // 셀 텍스트 (좌→우, 위→아래)
      label: 'table' },
    { type: 'text',  value: '   표 뒤 텍스트',    label: 'after-text' },
    { type: 'enter',                             label: 'enter2' },
    { type: 'text',  value: '다음 줄',            label: 'last-line' },
  ],
};
```

### step 종류

| type | 설명 | 필수 속성 | 선택 속성 |
|------|------|----------|----------|
| `text` | 키보드로 텍스트 입력 | `value` | `label` |
| `enter` | Enter 키 (문단 분할) | — | `label` |
| `inlineTable` | 인라인 TAC 표 삽입 | `rows`, `cols`, `colWidths` | `cells`, `label`, `sec`, `para` |

### label

각 step에 `label`을 지정하면:
- 스크린샷 파일명에 포함됨 (`v01-03-table.png`)
- 스냅샷 키로 사용됨 (규칙 검증에서 참조)
- 생략 시 `step-0`, `step-1`, ... 자동 생성

### 기대값 정의

구조 검증과 레이아웃 규칙 검증으로 구성됩니다.

```javascript
const expectations = {
  // ── 구조 검증 ──
  pageCount: 1,
  paragraphs: [
    { index: 0, text: 'TC #20' },                      // 정확한 텍스트 일치
    { index: 1, textContains: ['표 앞', '표 뒤'] },     // 부분 문자열 포함
    { index: 2, textContains: ['다음 줄'] },
  ],

  // ── 레이아웃 규칙 검증 ──
  layout: [
    { rule: 'inline-order', paraIndex: 1 },
    { rule: 'table-baseline-align', paraIndex: 1, controlIndex: 0, tolerance: 10.0 },
    { rule: 'space-before-table', paraIndex: 1, controlIndex: 0, minGap: 5.0 },
    { rule: 'space-after-table', paraIndex: 1, controlIndex: 0, minGap: 5.0 },
    { rule: 'stable-after-enter', paraIndex: 1,
      compareSteps: ['table', 'enter2'], tolerance: 3.0 },
  ],
};
```

## 검증 규칙 상세

### inline-order

표가 텍스트 사이에 인라인으로 배치되었는지 x좌표 순서를 확인합니다.

```javascript
{ rule: 'inline-order', paraIndex: 1 }
```

- 표와 같은 y 범위(±10px)의 TextRun을 수집
- 표 앞 텍스트(x+w ≤ 표 x)와 뒤 텍스트(x ≥ 표 x+w)가 존재하는지 확인
- TextRun이 분리되지 않은 경우(표 앞에 전체 텍스트 하나)도 통과

### table-baseline-align

표 하단이 텍스트 베이스라인 부근에 정렬되었는지 확인합니다.

```javascript
{ rule: 'table-baseline-align', paraIndex: 1, tolerance: 10.0 }
```

- `tolerance`: 허용 px 차이 (기본 5.0)
- 표 앞 텍스트의 y좌표와 표 하단(y+h)의 차이를 비교

### space-before-table / space-after-table

표 앞/뒤에 공백이 렌더링되었는지 확인합니다.

```javascript
{ rule: 'space-before-table', paraIndex: 1, minGap: 5.0 }
```

- `minGap`: 최소 간격 px (기본 3.0)
- 표 x와 앞 텍스트 끝 x 사이의 간격을 측정

### stable-after-enter

두 시점의 스냅샷을 비교하여 표 위치가 변하지 않았는지 확인합니다.

```javascript
{ rule: 'stable-after-enter', paraIndex: 1,
  compareSteps: ['table', 'enter2'], tolerance: 3.0 }
```

- `compareSteps`: 비교할 두 step의 label
- 두 스냅샷에서 같은 paraIndex의 표 bbox를 비교
- `tolerance`: 허용 dx/dy px (기본 2.0)

## 테스트 실행 코드

```javascript
import { runTest, createNewDocument, clickEditArea } from './helpers.mjs';
import { runScenario } from './scenario-runner.mjs';

runTest('테스트 이름', async ({ page }) => {
  await createNewDocument(page);
  await clickEditArea(page);
  await runScenario(page, scenario, expectations, 'screenshot-prefix');
});
```

### runScenario 반환값

```javascript
const { results, snapshots, finalState } = await runScenario(page, scenario, expectations);

// results: 검증 결과 배열
// [{ rule: 'pageCount', pass: true, message: '...' }, ...]

// snapshots: 단계별 렌더 트리
// { 'title': { tables: [...], textRuns: [...] }, 'table': {...}, 'final': {...} }

// finalState: 최종 문서 상태
// { pageCount: 1, paraCount: 3, paragraphs: [{ index: 0, text: '...' }, ...] }
```

## 시나리오 추가 예시

### 인라인 표 2개 연속

```javascript
const scenario = {
  name: 'TC-V04: 인라인 표 2개',
  steps: [
    { type: 'text', value: '앞 ' },
    { type: 'inlineTable', rows: 1, cols: 2, colWidths: [4000, 4000],
      cells: ['A', 'B'], label: 'table1' },
    { type: 'text', value: ' 중간 ' },
    { type: 'inlineTable', rows: 1, cols: 2, colWidths: [4000, 4000],
      cells: ['C', 'D'], label: 'table2' },
    { type: 'text', value: ' 뒤' },
  ],
};

const expectations = {
  pageCount: 1,
  paragraphs: [
    { index: 0, textContains: ['앞', '중간', '뒤'] },
  ],
  layout: [
    { rule: 'inline-order', paraIndex: 0 },
  ],
};
```

### 큰 표와 작은 표

```javascript
const scenario = {
  name: 'TC-V05: 다양한 크기 표',
  steps: [
    { type: 'text', value: '작은표: ' },
    { type: 'inlineTable', rows: 1, cols: 1, colWidths: [3000],
      cells: ['S'], label: 'small' },
    { type: 'text', value: ' 큰표: ' },
    { type: 'inlineTable', rows: 3, cols: 3, colWidths: [5000, 5000, 5000],
      cells: ['1','2','3','4','5','6','7','8','9'], label: 'large' },
    { type: 'text', value: ' 끝' },
  ],
};
```

### 한컴 원본 파일 비교 (골든 테스트)

```javascript
import { loadHwpFile } from './helpers.mjs';
import { captureLayout } from './scenario-runner.mjs';

// 한컴 원본 로드 → 렌더 트리 추출 → 기대값으로 사용
const { pageCount } = await loadHwpFile(page, 'tac-case-001.hwp');
const goldenLayout = await captureLayout(page, 0);

// 빈 문서에서 동일 구조 생성 → 렌더 트리 비교
await createNewDocument(page);
await runScenario(page, scenario, expectations);
const generatedLayout = await captureLayout(page, 0);

// 좌표 비교
const goldenTable = goldenLayout.tables[0];
const genTable = generatedLayout.tables[0];
const dx = Math.abs(goldenTable.x - genTable.x);
const dy = Math.abs(goldenTable.y - genTable.y);
assert(dx < 5 && dy < 5, `한컴 대비 차이: dx=${dx} dy=${dy}`);
```

## WASM API 참조

시나리오 실행기가 내부적으로 사용하는 API:

| API | 용도 |
|-----|------|
| `createTableEx(json)` | 인라인 TAC 표 생성 (`treatAsChar: true`) |
| `insertTextLogical(sec, para, logicalOffset, text)` | 논리적 오프셋으로 텍스트 삽입 |
| `getLogicalLength(sec, para)` | 논리적 문단 길이 (텍스트 + 컨트롤) |
| `logicalToTextOffset(sec, para, logicalOffset)` | 논리적 → 텍스트 오프셋 변환 |
| `navigateNextEditable(sec, para, charOffset, delta, contextJson)` | 커서 이동 (컨트롤 건너뛰기) |
| `getPageRenderTree(pageNum)` | 렌더 트리 JSON (좌표 검증용) |
| `insertTextInCell(sec, para, ctrl, cell, cellPara, offset, text)` | 셀 내 텍스트 삽입 |

## 출력

### 콘솔 출력

```
  === 시나리오: TC-V01: 인라인 TAC 표 기본 ===
  실행 완료: 1페이지, 3문단
  ✓ [pageCount] 페이지 수: 기대=1 실제=1
  ✓ [paragraph-contains] pi=1 '배치 시작' 포함: true
  ✓ [inline-order] 인라인 순서: 앞=3 뒤=11 같은줄=18
  ✓ [table-baseline-align] 세로 정렬: 표하단=187.8 textY=195.8 차이=8.0px (허용=10)
  결과: 7 통과, 0 실패
```

### 스크린샷

각 step마다 `e2e/screenshots/{prefix}-{번호}-{label}.png`에 저장됩니다.

### HTML 보고서

`output/e2e/{테스트명}-report.html`에 인라인 스크린샷 포함 보고서가 생성됩니다.

---

## SVG 회귀 검증 (Rust 유닛 테스트 기반)

**위치**: `tests/svg_snapshot.rs` + `tests/golden_svg/*/page-N.svg`
**도입**: PR [#181](https://github.com/edwardkim/rhwp/pull/181) (by @seunghan91) · 이슈 [#173](https://github.com/edwardkim/rhwp/issues/173)

E2E 시나리오가 "문서 조작 → 렌더 좌표 검증" 이라면, SVG snapshot 은 **"샘플 HWPX 를 렌더했을 때 바이트 단위로 이전과 같은가"** 를 묻는다. 두 하네스는 상보적이며 역할이 다르다.

### 하네스 구조

```
tests/
├── svg_snapshot.rs             # 하네스 코드
└── golden_svg/
    ├── form-002/
    │   └── page-0.svg          # 기대값 (golden)
    └── table-text/
        └── page-0.svg
```

실행:

```bash
cargo test --test svg_snapshot
```

실패 메시지:

```
SVG snapshot mismatch for form-002/page-0.
  expected: tests/golden_svg/form-002/page-0.svg
  actual:   tests/golden_svg/form-002/page-0.actual.svg
Inspect the diff; if intentional, rerun with UPDATE_GOLDEN=1.
```

### Golden 재생성

렌더 변경이 **의도된** 경우 (예: 렌더러 버그 수정, 새 기능 추가):

```bash
UPDATE_GOLDEN=1 cargo test --test svg_snapshot
```

그 후 반드시 **두 번째 실행으로 결정성(determinism) 확인**:

```bash
cargo test --test svg_snapshot
# render_is_deterministic_within_process 포함 전체 통과해야 함
```

결정성 테스트 `render_is_deterministic_within_process` 는 같은 프로세스 내에서 같은 페이지를 두 번 렌더해 바이트 동일성을 확인한다. 실패하면 렌더러 내부에 **비결정적 로직** (예: HashMap iteration 순서) 이 있다는 뜻.

### 경고 — 렌더 영향 PR 머지 후 필수 체크

렌더러 · 레이아웃 · 레이아웃 관련 유틸 수정이 포함된 PR 을 머지한 직후에는 반드시 다음을 수행:

1. `cargo test --test svg_snapshot` 로 로컬 확인
2. 실패 시 diff 검토 → 의도된 변경이면 `UPDATE_GOLDEN=1` 재생성
3. 재생성한 golden 을 **같은 커밋 또는 바로 다음 fixup 커밋** 으로 push

이를 놓치면 CI `Build & Test` 가 "svg_snapshot → FAILED" 로 실패한다. 실제 이력:

- **2026-04-20**: PR [#221](https://github.com/edwardkim/rhwp/pull/221) (OLE/Chart/EMF) + [#213](https://github.com/edwardkim/rhwp/pull/213) 머지 후 CI 실패 → 커밋 `4694cd3` 로 golden 재생성
- **2026-04-23**: PR [#251](https://github.com/edwardkim/rhwp/pull/251) (PUA + border margin + table border) 머지 후 CI 실패 → 커밋 `99a2e1f` 로 golden 재생성

2회 연속 재현되어 "렌더 영향 PR 머지 → golden 재생성 체크" 는 릴리즈 체크리스트 수준으로 격상 필요.

---

## 향후 작업 — 한컴 PDF 기준 Visual Diff 하네스

이슈 [#253](https://github.com/edwardkim/rhwp/issues/253) 로 구상 진행 중.

### 현재 vs 목표

| 항목 | 현재 (`svg_snapshot.rs`) | 목표 (Visual Diff) |
|------|------------------------|---------------------|
| 기준점 | rhwp 자신의 이전 출력 | **한컴 PDF** (고정 기준) |
| 감지 능력 | 회귀 (이전과 달라졌는가) | 호환성 (한컴과 얼마나 다른가) |
| 실패 해석 | "의도된 변경일 수 있음" | "한컴과 벌어지고 있음" |

### 계기

PR [#251](https://github.com/edwardkim/rhwp/pull/251) 의 @seanshin 님이 로컬에서 "한컴 PDF ↔ rhwp SVG 페이지별 픽셀 비교" 파이프라인을 돌려 3건의 렌더 불일치를 발견했다. 해당 파이프라인은 저장소에 포함되지 않았으므로 공식 인프라로 도입이 필요.

### 선행 조건

- 한컴 2020 PDF 시드 수작업 준비 (맥 환경 전용)
- 한컴 PDF 를 저장소에 포함시킬 경우의 라이선스 범위 확인
- PDF → 이미지 변환 (Poppler / MuPDF / resvg 등)
- 픽셀 비교 알고리즘 + tolerance 정책

상세 진행은 이슈 #253 참조.
