---
kind: guide
status: active
canonical: mydocs/manual/e2e_verification_guide.md
last_verified: 2026-07-17
---

# CDP를 사용한 E2E 테스트 가이드

> Chrome DevTools Protocol(CDP)을 통해 rhwp-studio 편집기의 E2E 테스트를 자동 실행하고,
> 작업지시자가 Chrome 브라우저에서 테스트 과정을 실시간으로 시각적 확인할 수 있다.

---

## 1. 사전 준비

### 1.1 WASM 빌드

저장소 루트에서 [개발 환경 가이드](dev_environment_guide.md)의 WASM 빌드 절차를 따른다.

```bash
wasm-pack build --target web --out-dir pkg
```

빌드 결과물은 `pkg/` 폴더에 생성된다.

### 1.2 WSL2 네트워크 설정 (mirrored 모드)

WSL2에서 Windows 호스트의 Chrome CDP에 접속하려면, mirrored 네트워크 모드를 사용한다.
mirrored 모드에서는 Windows와 WSL2가 동일한 네트워크 스택을 공유하므로 `localhost`로 직접 통신할 수 있다.

**Windows 측 설정** — `C:\Users\<사용자>\.wslconfig` 파일 생성 또는 편집:

```ini
[wsl2]
networkingMode=mirrored
memory=20GB
processors=8
swap=4GB
dnsTunneling=true
```

> `networkingMode=mirrored`는 WSL 2.0.0 이상 + Windows 11 22H2 이상에서 지원된다.
> `wsl --version`으로 WSL 버전을 확인할 수 있다.

설정 후 PowerShell에서 WSL 재시작:

```powershell
wsl --shutdown
```

### 1.4 Chrome 디버깅 모드 시작 (Windows 호스트)

Windows CMD에서 실행:

```cmd
start chrome --remote-debugging-port=19222 --remote-debugging-address=0.0.0.0 --user-data-dir="C:\temp\chrome-debug1"
```

```osx
arch -arm64 "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --remote-debugging-port=19222 --remote-debugging-address=0.0.0.0 --user-data-dir="~/tmp/chrome-debug1"
```

| 옵션 | 설명 |
|------|------|
| `--remote-debugging-port=19222` | CDP 포트 (Puppeteer가 연결) |
| `--remote-debugging-address=0.0.0.0` | WSL2에서 접근 가능하도록 모든 인터페이스 바인딩 |
| `--user-data-dir` | 별도 프로필 (기존 Chrome과 충돌 방지) |

Chrome이 시작되면 빈 탭이 열린다. 테스트 실행 시 새 탭이 자동으로 열리고, 테스트 완료 후 자동으로 닫힌다.

#### CDP 연결 확인 (WSL2에서)

```bash
curl -s http://localhost:19222/json/version
```

정상이면 Chrome 버전 정보가 JSON으로 출력된다.

### 1.5 Vite 개발 서버 시작 (WSL2)

```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700 &
```

브라우저에서 `http://localhost:7700`으로 접속 가능한지 확인한다.

---

## 2. 테스트 실행

### 2.1 기본 실행

```bash
cd rhwp-studio
CHROME_CDP=http://localhost:19222 node e2e/edit-pipeline.test.mjs --mode=host
```

| 환경변수/옵션 | 설명 |
|-------------|------|
| `CHROME_CDP` | Chrome CDP 주소 (mirrored 모드에서는 `http://localhost:19222`) |
| `--mode=host` | 호스트 Chrome에 CDP 연결 (기본값) |
| `--mode=headless` | WSL2 내부 headless Chrome 사용 (시각 확인 불가) |

### 2.2 전체 테스트 목록

테스트 스크립트의 단일 권위 목록은 [`rhwp-studio/e2e/MANIFEST.md`](../../rhwp-studio/e2e/MANIFEST.md)다
(#2353 — 분류·상태·용도·샘플·배선 전수). 파일 추가/변경/폐기 시 MANIFEST 를 함께 갱신하고,
`python3 scripts/check_e2e_manifest.py` 로 대조를 확인한다 (양방향 대조 + 명명 정합 + 배선 실재).

### 2.3 headless 모드 (CI용)

시각적 확인 없이 자동 실행:

```bash
cd rhwp-studio
node e2e/edit-pipeline.test.mjs --mode=headless
```

headless 모드에서는 WSL2 내부의 Chromium을 사용하므로 Windows Chrome이 필요 없다.

---

## 3. 테스트 구조

### 3.1 공통 패턴 (`runTest`)

모든 테스트는 `helpers.mjs`의 `runTest()` 래퍼를 사용하여 일관된 구조를 따른다:

```javascript
import { runTest, createNewDocument, clickEditArea, assert, screenshot } from './helpers.mjs';

runTest('테스트 제목', async ({ page, browser }) => {
  // 새 빈 문서 생성
  await createNewDocument(page);
  await clickEditArea(page);

  // 테스트 로직...
  assert(condition, '검증 메시지');
  await screenshot(page, 'step-name');
});
```

`runTest()`가 자동으로 처리하는 항목:

| 항목 | 설명 |
|------|------|
| 브라우저 연결 | `launchBrowser()` → CDP 또는 headless |
| 페이지 생성 | `createPage()` → 윈도우 크기 설정 (host: 1280x750, headless: 1280x900) |
| 앱 로드 | `loadApp()` → Vite 서버 + WASM 초기화 대기 |
| 에러 처리 | try/catch → 에러 스크린샷 + `process.exitCode = 1` |
| 탭 정리 | 테스트가 연 탭만 닫기 (호스트 Chrome의 기존 탭 유지) |
| HTML 보고서 | `output/e2e/{테스트명}-report.html` 자동 생성 |

옵션:
- `{ skipLoadApp: true }` — 앱 로드 생략 (hwpctl-basic처럼 별도 HTML 페이지 사용 시)

### 3.2 문서 로드 패턴

**새 빈 문서 생성:**

```javascript
await createNewDocument(page);  // eventBus emit + 캔버스 대기
```

**HWP 파일 로드:**

```javascript
const { pageCount } = await loadHwpFile(page, 'biz_plan.hwp');
// samples/ 폴더에서 fetch → WASM loadDocument → 캔버스 대기
```

### 3.3 헬퍼 함수 (helpers.mjs)

#### 브라우저/페이지 생명주기

| 함수 | 설명 |
|------|------|
| `launchBrowser()` | Chrome CDP 연결 또는 headless 시작 |
| `createPage(browser, width?, height?)` | 테스트용 탭 생성 + 크기 설정 |
| `closePage(page)` | 탭 닫기 |
| `closeBrowser(browser)` | 테스트 탭 닫기 + CDP disconnect 또는 headless close |

#### 앱/문서 로드

| 함수 | 설명 |
|------|------|
| `loadApp(page)` | Vite 서버에서 앱 로드 + WASM 초기화 대기 |
| `createNewDocument(page)` | 새 빈 문서 생성 + 캔버스 대기 |
| `loadHwpFile(page, filename)` | HWP 파일 fetch + loadDocument + 캔버스 대기 |
| `waitForCanvas(page, timeout?)` | 편집 영역 캔버스 대기 |

#### 편집/입력

| 함수 | 설명 |
|------|------|
| `clickEditArea(page)` | 편집 영역 캔버스 클릭하여 포커스 |
| `typeText(page, text)` | 키보드로 텍스트 입력 (글자별 30ms 지연) |

#### 조회/검증

| 함수 | 설명 |
|------|------|
| `getPageCount(page)` | WASM API로 페이지 수 조회 |
| `getParagraphCount(page, secIdx?)` | WASM API로 문단 수 조회 |
| `getParaText(page, secIdx, paraIdx, maxLen?)` | WASM API로 문단 텍스트 조회 |
| `assert(condition, message)` | PASS/FAIL 출력 + 리포터 자동 기록 |
| `screenshot(page, name)` | 스크린샷 저장 + 리포터에 자동 연결 |

#### 테스트 러너

| 함수 | 설명 |
|------|------|
| `runTest(title, testFn, options?)` | 테스트 실행 래퍼 (생명주기 + 에러 처리 + 보고서) |
| `setTestCase(name)` | 보고서 내 테스트 케이스 그룹명 설정 |

### 3.4 WASM API 직접 호출

키보드 입력 외에 WASM API를 직접 호출하여 정밀한 편집 테스트를 수행할 수 있다:

```javascript
const result = await page.evaluate(() => {
  const w = window.__wasm;

  // 텍스트 삽입
  w.doc.insertText(0, 0, 0, 'Hello');

  // 문단 분할
  w.doc.splitParagraph(0, 0, 5);

  // 표 삽입
  const tr = JSON.parse(w.doc.createTable(0, 1, 0, 2, 2));

  // 셀 텍스트 삽입
  w.doc.insertTextInCell(0, tr.paraIdx, tr.controlIdx, 0, 0, 0, 'Cell');

  // 페이지 브레이크
  w.doc.insertPageBreak(0, 0, 5);

  // 문단 병합
  w.doc.mergeParagraph(0, 1);

  // 캔버스 재렌더링 트리거 (WASM API 직접 호출 후 필수)
  window.__eventBus?.emit('document-changed');

  return { pageCount: w.doc.pageCount() };
});
```

> **중요**: WASM API를 직접 호출한 후에는 반드시 `window.__eventBus?.emit('document-changed')`를
> 호출하여 캔버스를 갱신해야 화면에 반영된다. 키보드 입력(`typeText`)은 자동으로 처리된다.

---

## 4. HTML 테스트 보고서

모든 테스트 실행 시 `output/e2e/` 폴더에 HTML 보고서가 자동 생성된다.

### 4.1 보고서 파일

`runTest()`를 사용하는 테스트는 자동으로 보고서를 생성한다:

```
output/e2e/
  blogform-report.html
  copy-paste-report.html
  debug-pagination-report.html
  debug-table-pos-report.html
  debug-textbox-report.html
  edit-pipeline-report.html
  footnote-insert-report.html
  footnote-vpos-report.html
  hwpctl-basic-report.html
  line-spacing-report.html
  page-break-report.html
  responsive-report.html
  shape-inline-report.html
  shift-end-report.html
  text-flow-report.html
  typesetting-report.html
```

### 4.2 보고서 내용

- **요약 대시보드**: Total / Passed / Failed / Skipped 카운트
- **TC별 카드**: 각 테스트 케이스의 assertion 결과 + 스크린샷
- **스크린샷 인라인**: base64로 인코딩되어 별도 파일 없이 단일 HTML로 확인 가능

### 4.3 보고서 확인

```bash
# 테스트 실행 (보고서 자동 생성)
cd rhwp-studio
CHROME_CDP=http://localhost:19222 node e2e/copy-paste.test.mjs --mode=host

# 보고서 열기 (Windows — WSL2에서 실행)
explorer.exe "$(wslpath -w ../output/e2e/copy-paste-report.html)"
```

### 4.4 assert와 screenshot의 리포터 연동

`assert()`는 PASS/FAIL을 콘솔에 출력하는 동시에 내장 리포터에 자동 기록한다.
`screenshot()`은 스크린샷을 파일로 저장하고, 리포터의 마지막 assertion에 자동 연결한다.

```javascript
await screenshot(page, 'step-01');      // 스크린샷 저장
assert(count === 1, '페이지 수 = 1');   // PASS/FAIL + 리포터 기록 + 스크린샷 연결
```

### 4.5 자체 리포터 사용 (edit-pipeline, responsive)

`TestReporter` 클래스를 직접 사용하면 테스트 케이스 그룹화 등 세밀한 제어가 가능하다:

```javascript
import { TestReporter } from './report-generator.mjs';

const reporter = new TestReporter('나의 테스트');
reporter.pass('TC #1', '텍스트 삽입 성공');
reporter.fail('TC #2', '페이지 수 불일치');
reporter.skip('TC #3', 'API 미지원');
reporter.generate('../output/e2e/my-report.html');
```

---

## 5. 스크린샷

테스트 실행 시 각 단계의 스크린샷이 `rhwp-studio/e2e/screenshots/` 폴더에 저장된다.
스크린샷은 HTML 보고서에 base64로 인라인 포함된다.

```
rhwp-studio/e2e/screenshots/
  cp-01-typed.png
  cp-02-pasted.png
  cp-03-final.png
  edit-01-split.png
  edit-06-table-insert.png
  ...
  error.png              ← 에러 발생 시 자동 촬영
```

---

## 6. 새 테스트 추가 방법

### 6.1 새 빈 문서 테스트

```javascript
import {
  runTest, createNewDocument, clickEditArea, typeText,
  screenshot, assert, getPageCount,
} from './helpers.mjs';

runTest('나의 새 테스트', async ({ page }) => {
  await createNewDocument(page);
  await clickEditArea(page);

  await typeText(page, 'Hello World');
  await screenshot(page, 'my-01-input');

  const pages = await getPageCount(page);
  assert(pages === 1, `페이지 수 확인: ${pages}`);
});
```

### 6.2 HWP 파일 로드 테스트

```javascript
import { runTest, loadHwpFile, screenshot, assert } from './helpers.mjs';

runTest('나의 파일 테스트', async ({ page }) => {
  const { pageCount } = await loadHwpFile(page, 'my-sample.hwp');
  assert(pageCount >= 1, `문서 로드 성공 (${pageCount}페이지)`);
  await screenshot(page, 'my-01-loaded');

  // WASM API 직접 호출
  const text = await page.evaluate(() =>
    window.__wasm?.getTextRange(0, 0, 0, 50) ?? ''
  );
  assert(text.includes('기대하는 텍스트'), `첫 문단 텍스트 확인`);
});
```

### 6.3 검증 패턴

| 패턴 | 사용 함수 |
|------|----------|
| 문단 텍스트 확인 | `getParaText(page, sec, para)` 또는 `w.doc.getTextRange(sec, para, offset, count)` |
| 셀 텍스트 확인 | `w.doc.getTextInCell(sec, para, ctrl, cell, cellPara, offset, count)` |
| 문단 수 확인 | `getParagraphCount(page, sec)` 또는 `w.doc.getParagraphCount(sec)` |
| 페이지 수 확인 | `getPageCount(page)` 또는 `w.doc.pageCount()` |
| 줄 정보 확인 | `JSON.parse(w.doc.getLineInfo(sec, para, offset))` |
| SVG 렌더링 확인 | `w.doc.renderPageSvg(pageNum)` |

---

## 7. 트러블슈팅

### CDP 연결 실패

```
TypeError: Failed to fetch browser webSocket URL
```

- Chrome이 디버깅 모드로 실행 중인지 확인 (`start chrome --remote-debugging-port=19222 ...`)
- `CHROME_CDP=http://localhost:19222`로 설정되어 있는지 확인
- 포트 프록시 설정 확인: `netsh interface portproxy show v4tov4`
- WSL2에서 연결 테스트: `curl -s http://localhost:19222/json/version`

### 캔버스를 찾을 수 없음

```
Error: 편집 영역 캔버스를 찾을 수 없습니다
```

- Vite 개발 서버가 `0.0.0.0:7700`에서 실행 중인지 확인
- WASM 빌드(`pkg/`)가 최신인지 확인
- 새 문서 생성 또는 파일 로드 후 캔버스가 생성되었는지 확인

### WASM API 호출 후 화면 미갱신

- `window.__eventBus?.emit('document-changed')` 호출 확인
- `await page.evaluate(() => new Promise(r => setTimeout(r, 300)))` 안정화 대기 추가

### 테스트가 타이밍 문제로 실패

- `typeText` 대신 `page.keyboard.type(text, { delay: 5 })`로 빠르게 입력
- WASM API 직접 호출로 전환 (키보드 입력보다 안정적)
- 안정화 대기 시간 증가 (`setTimeout` 값 조정)

### 샘플 파일 누락

```
Error: 파일 로드 실패 (biz_plan.hwp): HTTP 404
```

- `rhwp-studio/public/samples/` 폴더에 해당 HWP 파일이 있는지 확인
- `samples/` 폴더에서 복사: `cp samples/biz_plan.hwp rhwp-studio/public/samples/`
