# Changelog — @rhwp/node

이 패키지의 변경 사항을 기록합니다. 저장소 전체 변경은 루트
[`CHANGELOG.md`](../../CHANGELOG.md)를 봅니다.

## 미출시 — 0.1.0

첫 Node/TypeScript 바인딩 (#3776, M19). 파이썬 1호(#3762, M18)와 같은 계약을
같은 층 구조로 옮긴 것이며, **판정 로직을 새로 만들지 않는다.** 이 패키지는 rhwp CLI
의 `--json` 봉투와 `mcp-serve` 세션 프로토콜을 TypeScript 표면으로 재포장한다.

### 계약

- **exit 3/4 는 예외가 아니다.** `--verify` 불일치와 `--verify-pages` 페이지 수
  불일치는 *도구가 정상 동작한 결과*이므로 반환값의 판정 필드로 돌아온다.
  `throwOnVerdict: true` 를 명시했을 때만 `VerdictFailed` 로 던진다.
  exit 1 은 `RhwpRuntimeError`(고장), exit 2 는 `UsageError`(호출 조립 버그)다.
- **없는 봉투 필드에 접근하면 조용한 `undefined` 가 아니라 예외**가 난다. 있는 필드를
  함께 알려준다. 오타 한 글자가 "검증 통과"로 읽히는 사고를 막는 장치다.
- **`null` 과 실패는 다르고, `null` 과 `[]` 도 다르다.** `verify: null` 은 "검증하지
  않았다", `changedPages: null` 은 "모른다", `[]` 는 "없다"이다.
- **바이너리를 동봉하지 않는다.** 탐색 순서는 `RHWP_BIN` → 패키지 동봉(`dist/_bin/`)
  → `PATH`. 환경변수를 줬는데 못 쓰면 다음 후보로 넘어가지 않고 즉시
  `BinaryNotFoundError` 를 던진다 — 지정한 바이너리 대신 다른 것이 조용히 실행되는
  편이 더 나쁘다.
- **이름 변환은 기계적이다.** `camelCase ↔ snake_case` 를 규칙으로만 오간다
  (`pageCount ↔ page_count`). 수기 개명을 넣지 않는 이유는, 예외를 하나 두는 순간
  봉투의 새 필드가 자동으로 흘러오지 않기 때문이다.

### 추가된 API

- **예외** (`errors`) — `RhwpError` · `BinaryNotFoundError` · `UsageError` ·
  `RhwpRuntimeError` · `VerdictFailed` · `ProtocolError` · `SessionClosedError` ·
  `RhwpTimeoutError`, 종료 코드 상수 `EXIT_OK|RUNTIME|USAGE|VERIFY|VERIFY_PAGES`,
  `raiseForExit()` · `isKnownExitCode()`.
- **바이너리 탐색** (`binary`) — `findBinary()` · `clearBinaryCache()` ·
  `binaryName()` · `ENV_VAR`.
- **실행** (`process`) — `runJson()` · `runNdjson()` · `runRaw()` ·
  `iterNdjson()`(스트리밍, 중단 시 자식 정리). 종료 코드 검사는 **파싱 뒤**에 한다 —
  exit 3 에서도 판정 봉투가 나오기 때문이다.
- **봉투** (`envelope`) — `Envelope<T>` · `VerifyReport`. `children(key)` 는 배열 항목 중
  **객체가 아닌 것과 다시 배열인 것**을 걸러 낸다 — `Envelope` 생성자가 배열을 거부하므로,
  안 거르면 `[[1,2]]` 같은 값 하나가 조회와 무관한 자리에서 `TypeError` 를 낸다.
- **1층 무상태 명령** (`commands`) — `info` `exportText` `exportStructure`
  `exportTables` `fields` `search` `digest` `capabilities` `exportSvg` `exportPdf`
  `exportMarkdown` `exportHml` `exportDoclang` `thumbnail` `extractPages`
  `buildFromIngest` `exportHwpx` `convert` `irDiff` `renderDiff` `fillFields`
  `replaceText` `setCell` `batch` `exportIrSchema` `exportCapabilitiesSchema`.
  - `extractPages(path, from, to, opts)` 는 범위를 **숫자 두 인자**로 받는다.
    rhwp 의 `extract-pages` 가 `--from`/`--to` 만 받기 때문이다 — 범위 문자열
    어휘(`--pages`)는 `digest` 쪽 것이라 섞으면 exit 2 다. `from`·`to` 는 **1 기준**
    (rhwp 전체에서 유일한 1 기준 축). 다른 명령의 `page` 와 `digest` 의 `pages` 는
    전부 0 기준이다.
  - `exportHwpx(path, {out})` · `convert(path, {out})` 의 **산출 경로는 위치 인자**다.
    두 명령 모두 `-o` 를 모른다("알 수 없는 옵션: -o", exit 2). `exportHwpx` 는 생략하면
    `<입력 stem>.hwpx` 지만 `convert` 는 기본 경로가 없어 **`out` 이 필수**이며,
    빠뜨리면 프로세스를 띄우기 전에 `UsageError` 로 무엇이 없는지 이름으로 알린다.
  - `thumbnail(path, {base64|dataUri})` 의 두 플래그는 **파일 출력을 대체하고 서로
    배타적**이다. 켜면 `out` 을 줘도 파일이 생기지 않고 봉투의 `output` 이 `null` 이며,
    둘을 함께 켜면 나중 플래그가 이겨 `dataUri` 만 온다. 하나만 고른다.
  - `renderDiff(path, pathB?, {via, page, maxDisp, throwOnVerdict})` 는 렌더 기하로
    시각 회귀를 판정한다. `pathB` 를 주면 두 파일 비교(`mode: "pair"`), 없으면 자기
    라운드트립(`mode: "roundtrip"`). **회귀는 예외가 아니라 봉투의 `status`·`regression`
    필드**이고 `--json` 모드에서 exit 3 이다. 자세한 것은 아래 "본체 변화 반영".
  - 명령별 세부 옵션 — `exportText({page})` · `exportStructure({mode})` ·
    `digest({maxChars})` · `exportPdf({page, backend, profile, fontPath})` ·
    `irDiff({section, paragraph})` ·
    `batch({threads, mode, query, outDir, verify, verifyPages})` ·
    `exportCapabilitiesSchema({bare, out})`.
    `irDiff` 의 `paragraph` 는 CLI 의 `-p`/`--para` 로, **쪽이 아니라 문단**이다 —
    다른 명령의 `page` 와 이름이 비슷해 가장 헷갈리는 자리다.
  - `exportCapabilitiesSchema()` 는 `capabilities` 산출물 자체의 JSON Schema 다.
    `export-ir-schema`(문서 모델) 와 짝이 되는 **명령 표면**의 자기서술이다.
    이쪽은 `-o` 를 열어 뒀다 — 저장해도 stdout 이 봉투를 유지하기 때문이다
    (`output`·`bytes` 가 담긴다).

### 본체 변화 반영 — `render-diff --json`

`render-diff` 가 [#3719](https://github.com/edwardkim/rhwp/issues/3719) §6-2 로
`--json` 을 갖게 됐고, 바인딩이 `renderDiff` 로 감쌌다. 종료 코드 규약이 모드마다
다르므로 그대로 옮긴다.

| 모드 | 회귀 검출 시 | 왜 |
|---|---|---|
| `--json` | **exit 3**(판정) | CI 가 "렌더가 깨졌다"와 "파일을 못 읽었다"를 같은 신호로 받으면 안 된다 |
| 사람용 출력 | **exit 1**(종전 그대로) | 이미 1 을 실패로 읽는 CI 스크립트를 깨지 않기 위해서다. 새 의미론은 `--json` 소비자에게만 준다 |

봉투 필드는 18개(`mode`·`sourceA`·`sourceB`·`via`·`pageFilter`·`threshold`·
`pageCountA`·`pageCountB`·`pageCountMismatch`·`maxDisp`·`worstPage`·`overPages`·
`structPages`·`hardStructPages`·`status`·`regression`·`pages`·`schemaVersion`).
`status` 는 `PASS`·`WARN_TEXTRUN`·`OVER`·`STRUCT_MISMATCH`·`PAGE_MISMATCH`·`LOAD_FAIL`
중 하나이고, 앞의 둘을 뺀 나머지가 `regression: true` 다.
- **2층 세션** (`session`) — `openDocument()` · `Document` · `Session`.
  `[Symbol.asyncDispose]` 로 `await using` 을 지원한다.
- **3층 계획** (`plan`) — `Plan` 빌더와 `PlanResult`. **위반은 예외가 아니라 결과다.**
  `check()` 는 계획서를 보내기 전에 `capabilities` 로 `run --dry-run` 지원을 확인하고,
  지원하지 않으면 **실행으로 내려가지 않고 예외를 던진다** — 미지원 rhwp 는 계획서의
  `dryRun` 을 무시하고 편집·저장하므로, 호출자가 "검사만 했다"고 믿는 사이 문서가
  바뀌는 조용한 데이터 사고가 된다. 캐시는 `clearPlanCapabilityCache()` 로 비운다.
- **IR 스키마** (`schema`) — `irSchema()` · `IrSchema` · `TypeDef` · `FieldDef`.
- **브라우저 어댑터** (`@rhwp/node/browser`) — `createBrowserClient()`.
- **생성 타입** (`ir`, `envelopes`) — `tools/gen-types.ts` 산출물. 손으로 고치지 않는다.

### 패키징

- ESM + CJS 듀얼 배포, 타입 선언 동봉. `.` 와 `./browser` 두 진입점.
- **런타임 의존성 0.** Node 내장(`node:child_process` · `node:fs` · `node:path`)만
  쓴다. 바인딩이 무거우면 "재포장"이 아니라 새 표면이 된다.
- Node 18 이상.

### 노출하지 않은 것

- **`json` 을 선언하지 않은 명령은 표면에 없다** — `dump-*`·`hwp5-*` 진단 계열,
  `mcp-serve`. 노출 기준은 "손으로 고른 목록"이 아니라 **`capabilities` 의 `json`
  선언**이며, 그래야 본체가 늘 때 바인딩이 뒤처졌음을 계약 패리티 가드가 잡는다.
  필요하면 `runJson`·`runRaw` 로 직접 부른다.
- **`renderDiff` 의 `--batch` 축(폴더 일괄)은 감싸지 않는다.** 그쪽은 한 줄 봉투가
  아니라 NDJSON 스트림이라 반환 타입이 다르다. 한 함수가 둘 다 처리하면 호출자가 받은
  값이 봉투인지 배열인지 **타입으로 알 수 없게 된다.** 폴더 일괄이 필요하면
  `runNdjson(['render-diff', '--batch', 폴더, '--json'])` 으로 직접 부른다.
- **`--json` 봉투를 깨는 플래그는 열지 않는다.** `exportText`·`exportStructure` 의
  `-o` 는 `--json` 모드에서 **조용히 무시된다**(실측: 파일이 생기지 않는다) — 받아 주면
  "저장했다"는 거짓말이 된다. `exportTables` 의 `-o` 는 더 나쁘다: stdout 이 사람용
  문장("표 추출 완료: N개 → 경로")으로 바뀌어 봉투 계약 자체가 깨지고 `runJson` 이
  `ProtocolError` 를 던진다. **뒤엣것은 바인딩이 피할 문제가 아니라 본체 쪽 별도
  이슈감이다** — `--json` 을 준 호출에서 출력 형식이 바뀌는 것은 명령 하나의 사정이
  아니라 봉투 계약 전체의 예외이기 때문이다.

  기록해 둘 차이: `exportCapabilitiesSchema` 의 `-o` 는 **열어 뒀다.** 같은 `-o` 라도
  이쪽은 저장 후에도 stdout 이 봉투를 유지한다(`output`·`bytes`). 노출 여부를 가른 것은
  플래그 이름이 아니라 **봉투 계약을 지키는가**다.

### 도구 선택 (실측으로 정해진 것)

- **ESLint 를 devDependency 에서 뺐다.** `@typescript-eslint@8` 의 peer 범위는
  `typescript >=4.8.4 <6.1.0` 인데 이 저장소는 **TypeScript 7** 을 쓴다. 넣어 두면
  `npm install` 자체가 ERESOLVE 로 실패한다(CI 4개 잡이 여기서 죽었다).
  타입 안전은 `tsc --noEmit` 이 strict 전 옵션으로 담당하므로 더 강한 게이트가 이미 있다.
  typescript-eslint 가 TS 7 을 지원하면 되돌린다.
- **타입 선언을 tsup 이 아니라 tsc 가 만든다.** tsup 의 dts 파이프라인은 내부에
  TypeScript 5.7 을 물고 있어 TS 7 프로그램에서 터진다
  (`Cannot read properties of undefined (reading 'useCaseSensitiveFileNames')`).
  번들만 tsup 에 맡기고 선언은 `tsconfig.build.json`(emitDeclarationOnly)로 낸다.
- **`.d.cts` 는 `.d.ts` 를 복제해 만든다** (`tools/emit-cjs-types.mjs`).
  `exports` 의 `require` 갈래가 `.d.cts` 를 가리키는데 tsc 는 `.d.ts` 만 내기 때문이다.
  이 패키지 타입에 ESM 전용 문법이 없어 같은 내용이면 충분하다.

### 알려진 제약

- **rhwp 바이너리를 동봉하지 않는다.** 파이썬 1호와 마찬가지로 npm 패키지에는 실행
  파일이 들어 있지 않다. 사용자가 별도로 설치하고 `RHWP_BIN` 이나 `PATH` 로
  알려줘야 한다. 플랫폼별 바이너리 배포(선택적 optionalDependencies)는 이번 범위
  밖이다.
- **브라우저에서는 서브프로세스를 띄울 수 없다.** `@rhwp/node/browser` 는
  `@rhwp/editor` 의 WASM 을 같은 `RhwpClient` 인터페이스로 감싸는 어댑터일 뿐이며,
  CLI 전용 명령(예: 배치 파이프라인)은 이 경로에서 쓸 수 없다.
- **단위 테스트는 바이너리 없이 돌지만, 통합 테스트는 실물이 필요하다.**
  `RHWP_BIN` 이 없으면 통합 프로젝트는 대상이 비어 조용히 건너뛴다. 로컬에서 통과했다는
  사실이 통합까지 통과했다는 뜻은 아니다.
- **생성 타입은 빌드된 rhwp 를 필요로 한다.** `npm run gen:check` 는 바이너리 없이
  돌지 않으므로 CI 의 별도 잡에서만 검증한다.
