# API 레퍼런스 — `@rhwp/node`

전체 공개 API. 설계 근거는 [`node_binding_guide.md`](../../../mydocs/manual/node_binding_guide.md),
계약의 원천은 rhwp 본체의 `capabilities` 자기서술이다.

## 목차

- [모듈 수준](#모듈-수준)
- [공통 옵션](#공통-옵션)
- [1층 — 무상태 명령](#1층--무상태-명령)
- [2층 — 세션](#2층--세션)
- [3층 — 계획](#3층--계획)
- [IR 스키마](#ir-스키마)
- [봉투](#봉투)
- [생성 타입](#생성-타입)
- [브라우저 어댑터](#브라우저-어댑터)
- [예외](#예외)
- [저수준](#저수준)

---

## 모듈 수준

### 진입점

| 지정자 | 내용 |
|---|---|
| `@rhwp/node` | 1·2·3층 전부. Node 전용(자식 프로세스를 띄운다) |
| `@rhwp/node/browser` | `createBrowserClient` — 서브프로세스가 불가능한 환경 |

ESM·CJS 듀얼이다. `import` 와 `require` 가 각각 다른 산출물을 가리키며 타입 선언도 따로 나온다.

### `findBinary(opts?: { refresh?: boolean }): string`

rhwp 실행 파일 경로를 돌려준다. 탐색 순서는 `RHWP_BIN` → 패키지 동봉(`dist/_bin/`) → `PATH`.

```ts
import { findBinary } from '@rhwp/node';
findBinary();            // '/usr/local/bin/rhwp'
findBinary({ refresh: true });   // 캐시 무시하고 재탐색
```

**예외**: `BinaryNotFoundError` — 세 경로 모두 실패. 메시지에 시도한 위치를 전부 담는다.

환경변수를 **줬는데 못 쓰면** 조용히 다음 경로로 넘어가지 않고 즉시 실패한다.
탐색은 동기 함수다 — 경로 확인에 I/O 대기가 필요 없고, 모든 명령의 첫 줄에서 불린다.
`RHWP_BIN=~/bin/rhwp`처럼 선행 `~`를 쓰면 macOS·Linux에서 홈 디렉터리로 확장한다.
Windows에서는 `~\\bin\\rhwp.exe`도 같은 방식으로 쓸 수 있다.

### `clearBinaryCache(): void`

탐색 캐시를 비운다. 테스트에서 환경변수를 바꿔 가며 검사할 때 쓴다.

### `binaryName(): string`

플랫폼별 실행 파일 이름(`rhwp` / 윈도우는 `rhwp.exe`).

### `ENV_VAR`

바이너리 경로 환경변수 이름(`'RHWP_BIN'`).

---

## 공통 옵션

1층 함수와 저수준 실행기는 아래 옵션을 공통으로 받는다. 명령 고유 옵션과 같은 객체에 섞어 준다.

| 옵션 | 타입 | 기본 | 설명 |
|---|---|---|---|
| `timeoutMs` | `number \| null` | `300000` | 제한 시간. `null` 이면 무제한 |
| `cwd` | `string` | 현재 디렉터리 | 자식 프로세스 작업 디렉터리 |
| `stdin` | `string` | — | 자식 stdin 으로 보낼 문자열 |
| `throwOnVerdict` | `boolean` | `false` | 참이면 exit 3/4 도 `VerdictFailed` 로 던진다 |

`throwOnVerdict` 의 기본이 거짓인 것이 이 바인딩의 핵심 규약이다 — 판정은 값으로 다룬다
([예외](#예외) 참조).

---

## 1층 — 무상태 명령

호출 하나 = 프로세스 하나 = 문서 재파싱 하나. 같은 문서를 반복해서 만질 거라면
[2층](#2층--세션)이 빠르다.

모든 함수는 `Promise<Envelope<R>>` 를 돌려준다. `R` 은 `capabilities` 의 명령별
`recordFields` 에서 [생성된](#생성-타입) 레코드 타입이다(`src/envelopes.ts`).
예외는 `batch` 하나로, NDJSON 레코드 **배열**을 돌려준다.

### 조회

| 함수 | 대응 CLI | 돌려주는 것 |
|---|---|---|
| `info(path, opts?)` | `info` | 포맷·크기·구역/쪽/문단 수·글꼴·제목 |
| `exportText(path, opts?: { page?: number })` | `export-text` | 쪽별 평문. `page` 는 **0 기준** |
| `exportStructure(path, opts?: { mode?: 'auto' \| 'outline' \| 'clause' })` | `export-structure` | 제목 계층·조문 트리 |
| `exportTables(path, opts?)` | `export-tables` | 표 전량 + 셀 좌표(병합·중첩 보존) |
| `fields(path, opts?)` | `fields` | 누름틀 목록(이름·안내문·현재값·위치) |
| `capabilities(opts?: { mcp?: boolean })` | `capabilities` | 도구 자기서술. `mcp: true` 면 MCP 도구 정의 |

```ts
const meta = await info('보고서.hwp');
meta.get('pageCount');
meta.get('format');       // 'hwp5' | 'hwpx' | 'hwp3' | 'hml'

const tables = await exportTables('양식.hwpx');
tables.get('tableCount');   // setCell 에 쓸 좌표의 출처

// 긴 문서에서 필요한 쪽만 — 문맥 창을 아낀다
const page3 = await exportText('보고서.hwp', { page: 2 });
```

`exportStructure` 의 `mode` 는 계층 분류 방식이다. `outline` 은 개요 번호(1./가./1))
기준이라 보고서·기획서에, `clause` 는 조문(편·장·절·관·조·항·호·목) 기준이라 법령·규정·
정관에 맞는다. 기본 `auto` 는 문서를 보고 하나를 고르며, 고른 결과가 봉투의 `mode` 에
담긴다 — 자동 판정이 기대와 다를 때(규정 문서를 개요로 읽는 등) 명시로 되돌린다.

> **이 세 명령에 `-o` 는 없다 — 일부러 열지 않았다.**
> `exportText`·`exportStructure` 는 `--json` 모드에서 `-o` 를 **조용히 무시한다**(실측:
> 디렉터리조차 생기지 않는다). 바인딩이 받아 주면 "저장했다"는 거짓말이 된다.
> 파일이 필요하면 봉투의 `pages[].text` 를 직접 쓴다.
>
> `exportTables` 는 더 나쁘다. `-o` 를 준 순간 stdout 이 사람용 문장
> ("표 추출 완료: N개 → 경로")으로 바뀌어 **`--json` 봉투 계약 자체가 깨지고**,
> `runJson` 이 `ProtocolError` 로 터진다. 바인딩이 옵션을 닫은 것은 회피일 뿐이고,
> `--json` 을 준 호출에서 출력 형식이 바뀌는 것은 **본체 쪽 별도 이슈감**이다.
> 반대 사례가 `exportCapabilitiesSchema`([스키마](#스키마) 절) 다 — 저장해도 stdout 이
> 봉투를 유지하므로 그쪽은 `out` 을 열어 뒀다. 노출을 가른 것은 플래그 이름이 아니라
> **봉투 계약을 지키는가**다.

### `search(path, query, opts?: { caseSensitive?: boolean; limit?: number })`

주소가 붙은 검색 — 매치마다 (구역·문단·**쪽**·문자 오프셋)과 문맥.

```ts
const hits = await search('보고서.hwp', '예산', { limit: 50 });
hits.get('matchCount');
hits.get('truncated');    // limit 로 잘렸는가
```

`-` 로 시작하는 검색어도 그대로 넘길 수 있다 — 내부에서 `--` 구분자를 쓴다.
`--` 뒤는 전부 위치 인자이므로 `--json` 은 구분자 **앞**에 조립된다.

### `digest(path, opts?: { sections?: boolean; pages?: string; maxChars?: number })`

요약·RAG 용 청킹. `sections: true` 는 주소를 보존한 절 단위, `pages: '0..4'` 는 쪽 범위 창
(0 기준, 양끝 포함).

`maxChars` 는 발췌 최대 문자 수다. 기본은 2000 이고 `sections: true` 면 절마다 240 이다.

```ts
await digest('보고서.hwp', { sections: true, maxChars: 600 });
```

이 값이 없으면 긴 문서에서 발췌만으로 모델 문맥 창을 다 먹는다. 창이 좁은 모델에 넘길 때
줄이고, 한 번에 더 읽히고 싶을 때 늘린다.

### 산출

| 함수 | 대응 CLI | 고유 옵션 |
|---|---|---|
| `exportSvg(path, opts?)` | `export-svg` | `page`(0 기준) |
| `exportPdf(path, opts?)` | `export-pdf` | `page` · `backend` · `profile` · `fontPath` |
| `exportMarkdown(path, opts?)` | `export-markdown` | `page`(0 기준) |
| `exportHml(path, opts?)` | `export-hml` | — (`out` 은 CLI 가 요구한다) |
| `exportDoclang(path, opts?)` | `export-doclang` | `assetsDir` |
| `thumbnail(path, opts?)` | `thumbnail` | `base64` · `dataUri` |
| `extractPages(path, from, to, opts?: { out: string })` | `extract-pages` | — (범위는 인자) |
| `buildFromIngest(spec, opts?: { out?: string })` | `build-from-ingest` | `mediaDir` |

여기까지는 전부 `out` 을 `-o` 로 보낸다. **`exportHwpx`·`convert` 만 위치 인자다**
([변환·비교](#변환비교) 참조).

#### `exportPdf(path, opts?: { out?: string; page?: number; backend?: 'svg' \| 'direct'; profile?: 'screen' \| 'print' \| 'high-quality' \| 'fast-preview'; fontPath?: string \| string[] })`

```ts
await exportPdf('보고서.hwp', {
  out: '보고서.pdf',
  profile: 'print',
  fontPath: ['/opt/fonts/hancom', '/usr/share/fonts'],
});
```

| 옵션 | 설명 |
|---|---|
| `backend` | 기본 `svg`. `direct` 는 `native-skia` 기능을 켜서 빌드한 바이너리에서만 동작한다 — 없는 빌드에 주면 실행 오류(exit 1)다. 실제로 쓴 backend 는 봉투의 `backend` 로 확인한다 |
| `profile` | 화면용(`screen`)과 인쇄용(`print`)은 렌더 품질이 다르다. 미리보기라면 `fast-preview` 로 시간을 아낀다 |
| `fontPath` | 폰트 탐색 경로. 한컴 전용 폰트(HY견명조 등)가 없는 서버·CI 에서 글자가 깨지거나 대체 폰트로 밀릴 때 지정한다 |

`fontPath` 는 문자열 하나도 되고 배열도 된다. 배열이면 **같은 플래그를 반복해서** 붙인다 —
쉼표로 이어 붙이지 않는 이유는 경로에 쉼표가 들어갈 수 있기 때문이다.

#### `thumbnail(path, opts?: { out?: string; base64?: boolean; dataUri?: boolean })`

첫 쪽 미리보기 이미지(PrvImage).

```ts
await thumbnail('보고서.hwp', { out: '미리보기.png' });          // 파일
const b = await thumbnail('보고서.hwp', { base64: true });        // 봉투의 base64
const d = await thumbnail('보고서.hwp', { dataUri: true });       // data:image/png;base64,…
```

**`base64`·`dataUri` 는 파일 출력을 대체하고, 서로 배타적이다.** 둘 중 하나를 켜면 `out`
을 줘도 파일이 생기지 않고 봉투의 `output` 이 `null` 이 된다(실측). 파일과 문자열을 둘 다
원하면 파일로 뽑은 뒤 직접 읽는다.

함께 켜면 나중 플래그가 이겨 `dataUri` 만 온다 — 봉투에 `base64` 가 있을 거라 믿고 읽으면
`undefined` 를 만난다. **하나만 고른다.**

`exportDoclang` 의 `assetsDir` 은 그림 등 이진 자원을 파일로 분리한다. 생략하면 base64
data URI 로 XML 에 인라인되어, 그림이 많은 문서는 XML 이 수십 MB 로 부풀어 파서가 감당하지
못한다. 기록 결과는 봉투의 `assetsDir`·`assetCount` 로 확인한다.

`buildFromIngest` 의 `mediaDir` 은 명세가 참조하는 그림 파일 디렉터리다. 생략하거나 없는
경로를 주면 이미지가 placeholder 로 처리되고 **경고만 남는다(실패하지 않는다)** — 그림이
빠진 산출물을 정상으로 오해하기 쉬우니 그림이 있는 명세라면 반드시 지정한다.

#### `extractPages(path, from, to, opts?: { out: string })`

쪽 범위만 남겨 새 문서로 저장한다. 범위를 `"2-4"` 같은 **문자열 하나가 아니라 두 인자**로
받는다.

```ts
await extractPages('보고서.hwp', 2, 4, { out: '발췌.hwp' });
// → rhwp extract-pages 보고서.hwp --from 2 --to 4 -o 발췌.hwp --json
```

**왜 두 인자인가**: rhwp 의 `extract-pages` 는 `--from N --to M` 만 받는다
(`capabilities` 의 `flags: ["--from","--to","--json"]`). 범위 문자열 어휘(`--pages`)는
`digest` 쪽 것이라 여기에 섞으면 `알 수 없는 옵션: --pages` 로 **exit 2** 가 난다.
바인딩이 문자열을 받아 내부에서 쪼개는 설계도 가능하지만, 그러면 `"2-4"`·`"2..4"`·
`"2,4"` 중 무엇이 맞는지를 바인딩이 새로 정하게 된다 — 재포장이 아니라 새 표면이다.

`out` 은 타입상 선택이지만 **실질적으로 필수**다. rhwp 는 출력 경로가 없으면 사용법을
찍고 exit 2 로 끝낸다(원본을 덮어쓰지 않는 것이 이 명령의 안전 규약이다).

> **쪽 기준이 명령마다 다르다.** `extractPages` 의 `from`·`to` 만 **1 기준**(첫 쪽 = 1)이고,
> `exportText` 의 `page`·`Document.renderPage`·`search` 결과의 `page` 는 전부 **0 기준**이다.
> 그대로 옮겨 쓰면 오류 없이 한 쪽 밀린 문서가 나온다. rhwp 본체가 그렇게 굳어 있어
> 바인딩이 임의로 통일하지 않는다 — 재포장은 계약을 바꾸지 않는다.
> `search` 가 `page: 1`(사람 기준 2쪽)을 줬다면 잘라 낼 때는 `extractPages(p, 2, 2, …)` 다.

**결과 쪽수가 요청 범위와 다를 수 있다.** rhwp 는 쪽 단위로 자르되 **문단 단위로** 지우고,
지운 뒤 레이아웃이 다시 흐른다. 봉투의 `pagesBefore`·`pagesAfter` 로 실제 결과를 읽는다.

### 변환·비교

#### `exportHwpx(path, opts?: { out?: string; verify?: boolean; verifyPages?: boolean; throwOnVerdict?: boolean })`

HWP → HWPX 변환. `verify: true` 면 봉투에 `verify.identical` 이 담긴다.

```ts
const result = await exportHwpx('원본.hwp', { out: '변환본.hwpx', verify: true });
if (!result.verify?.identical) {
  console.log(`차이 ${result.verify?.diffCount}건`);   // 예외가 아니라 판정이다
}
// → rhwp export-hwpx 원본.hwp 변환본.hwpx --verify --json
```

> **산출 경로가 `-o` 가 아니라 위치 인자다.** 이 명령은 `-o` 를 모른다 —
> 실측하면 `알 수 없는 옵션: -o` 로 **exit 2** 다. 다른 산출 명령과 다르므로 옮겨 쓸 때
> 주의한다. 생략하면 `<입력 stem>.hwpx` 다.

#### `convert(path, opts: { out: string; verify?: boolean; verifyPages?: boolean; throwOnVerdict?: boolean })`

HWPX·배포용 → 편집 가능 HWP5 변환. 판정 옵션은 `exportHwpx` 와 같고, **산출 경로도 같이
위치 인자다.** 다만 이쪽은 **`out` 이 필수다** — 기본 경로가 없어서 빠뜨리면 CLI 가 사용법
오류로 끝난다.

```ts
await convert('배포본.hwpx', { out: '편집본.hwp', verify: true });
// → rhwp convert 배포본.hwpx 편집본.hwp --verify --json

await convert('배포본.hwpx');   // UsageError — 프로세스를 띄우기 전에 던진다
```

**예외**: `UsageError` — `out` 없이 불렀을 때. 프로세스를 띄워 exit 2 를 받아 오는 대신
바인딩이 같은 판정을 먼저 내리는 이유는, 그래야 **무엇이 빠졌는지 이름으로** 알려 줄 수
있기 때문이다. CLI 사용법 덤프는 "인자가 틀렸다"까지만 말한다.

#### `irDiff(a, b, opts?: { section?: number; paragraph?: number })`

두 문서의 IR 차이를 범주별로. 봉투에 `a`·`b`·`identical`·`diffCount`·`categories`.

```ts
await irDiff('원본.hwp', '변환본.hwpx');
await irDiff('원본.hwp', '변환본.hwpx', { section: 0, paragraph: 12 });
// → rhwp ir-diff 원본.hwp 변환본.hwpx -s 0 -p 12 --json
```

차이가 수백 건 나오는 문서에서 범위를 좁혀 원인을 이분법으로 찾을 때 쓴다.
둘 다 **0 기준**이다.

> **`paragraph` 는 쪽이 아니라 문단이다.** CLI 의 `-p`/`--para` 이고, 다른 명령의
> `page`(역시 `-p`)와 플래그 문자까지 같아 가장 헷갈리는 자리다. `section` 과 함께 주면
> 그 구역의 그 문단으로 좁힌다.

#### `renderDiff(path, pathB?, opts?: { via?: 'hwpx' \| 'hwp'; page?: number; maxDisp?: number; throwOnVerdict?: boolean })`

렌더 **기하** 차이로 시각 회귀를 판정한다. `irDiff` 가 내용(IR)을 본다면 이쪽은 배치를
본다 — `verify.identical` 이 참이어도 렌더 결과가 밀렸을 수 있다.

```ts
// 자기 라운드트립 — HWPX 로 저장했다 다시 읽으면 배치가 밀리는가 (mode: 'roundtrip')
const geom = await renderDiff('보고서.hwp');

// 경유 포맷을 바꿔 HWP5 저장이 무엇을 잃는지 본다
await renderDiff('보고서.hwp', undefined, { via: 'hwp' });

// 두 파일 직접 비교 (mode: 'pair')
await renderDiff('원본.hwp', '변환본.hwpx', { maxDisp: 2.0 });
```

라운드트립인데 옵션을 주고 싶으면 `pathB` 자리에 `undefined` 를 넘긴다. 두 번째 인자를
선택 위치 인자로 둔 것은 CLI 의 두 형태(`render-diff A` / `render-diff A B`)를 그대로
옮긴 결과다.

| 옵션 | 설명 |
|---|---|
| `via` | 라운드트립 경유 포맷. 기본 `hwpx`. **`pathB` 를 준 pair 비교에서는 무시된다**(봉투의 `via` 가 `null`) |
| `page` | 이 쪽만 판정(0 기준). 문서에 없는 쪽 번호면 사용법 오류(exit 2)다 — 조용한 빈 결과로 내면 "필터가 안 맞았다"와 "차이가 없다"가 같은 출력이 된다 |
| `maxDisp` | 변위 임계(px). 기본 1.0. 봉투에는 `threshold` 로 담긴다 |
| `throwOnVerdict` | 참이면 회귀 검출(exit 3)을 `VerdictFailed` 로 던진다. 기본은 거짓 |

##### 회귀는 예외가 아니라 판정 필드다

```ts
const geom = await renderDiff('보고서.hwp');
if (geom.get('regression')) {
  console.log(geom.get('status'));      // 'OVER' | 'STRUCT_MISMATCH' | …
  console.log(geom.get('maxDisp'), geom.get('worstPage'), geom.get('overPages'));
}
```

`status` 는 여섯 값 중 하나이고, 앞의 둘을 뺀 나머지가 `regression: true` 다.

| `status` | 뜻 | `regression` |
|---|---|---|
| `PASS` | 임계 안 | `false` |
| `WARN_TEXTRUN` | TextRun ±1 로 설명되는 구조 차이 | `false` |
| `OVER` | 변위가 `maxDisp` 초과 | `true` |
| `STRUCT_MISMATCH` | TextRun ±1 로 설명 안 되는 구조 불일치 | `true` |
| `PAGE_MISMATCH` | 쪽 수 불일치 (시각 회귀 강신호) | `true` |
| `LOAD_FAIL` | 비교 대상을 못 읽음 | `true` |

##### 종료 코드가 모드마다 다르다

| 모드 | 회귀 검출 시 | 왜 |
|---|---|---|
| `--json` (= 이 함수) | **exit 3** | 판정이지 고장이 아니다. 1 로 내면 CI 가 "렌더가 깨졌다"와 "파일을 못 읽었다"를 같은 신호로 받는다 |
| 사람용 출력 | **exit 1**(종전 그대로) | 이미 1 을 실패로 읽는 CI 스크립트를 깨지 않기 위해서다. 새 의미론은 `--json` 소비자에게만 준다 |

이 함수는 `--json` 경로이므로 exit 3 을 받고, 바인딩 규약대로 예외가 아니라 값으로 전한다.
CLI 를 직접 부르는 기존 스크립트를 옮길 때 **두 모드의 코드가 다르다**는 점만 기억하면 된다.

##### `--batch` 는 감싸지 않는다

CLI 의 폴더 일괄(`render-diff --batch <폴더>`)은 한 줄 봉투가 아니라 **NDJSON 스트림**이라
반환 타입이 다르다. 한 함수가 둘 다 처리하면 호출자가 받은 값이 봉투인지 배열인지 타입으로
알 수 없게 된다. 필요하면 저수준으로 부른다.

```ts
const rows = await runNdjson(['render-diff', '--batch', '샘플/', '-o', '결과/', '--json']);
```

이 명령은 `capabilities` 가 선언한 것을 바인딩이 **일부러 덜 감싼** 유일한 자리다.
근거는 "노출 기준은 자기서술"이라는 원칙의 예외가 아니라, **한 함수의 반환 타입은 하나**
라는 더 앞선 제약이다.

### 편집

세 함수 모두 `out`·`dryRun`·`verify` 를 받고, 판정을 예외로 원하면 `throwOnVerdict` 를 더한다.
`dryRun: true` 는 파일을 쓰지 않고 변경 예정만 보고한다.

#### `fillFields(path, data, opts?: { out?: string; dryRun?: boolean; verify?: boolean })`

누름틀 채우기(메일머지). `data` 는 `{ 필드이름: 값 }`.

```ts
const result = await fillFields('서식.hwp', { 성명: '홍길동' }, { out: '제출본.hwp', verify: true });
result.get('filledCount');
result.get('notFound');
result.get('ambiguous');    // 순번 없이 준 반복 필드 — 몇 개 중 몇 개를 채웠는지
result.changedPages;
```

같은 이름이 여러 번 나오는 서식은 `"이름[N]"`(0 기준, `fields` 목록 순서)으로 지목한다.
순번 없이 주면 첫 번째만 채우고 `ambiguous` 에 보고한다.

#### `replaceText(path, find, replace, opts?: { occurrence?: number; ignoreCase?: boolean; out?: string; dryRun?: boolean; verify?: boolean })`

문자열 치환. `occurrence` 를 주면 그 순번(0 기준) 하나만 바꾼다.
치환 0건이면 출력 파일을 만들지 않는다 — 0건은 오류가 아니라 계수 보고다.

#### `setCell(path, table, row, col, text, opts?: { out?: string; keepStyle?: boolean; dryRun?: boolean; verify?: boolean })`

표 셀 기록. 좌표는 `exportTables` 로 확인한다. 병합으로 덮인 칸은 앵커 좌표를 안내하며
실패한다. `keepStyle: true` 면 셀 스타일 상속을 유지한다(기본은 정규화).

### 대량

#### `batch(sub, paths, opts?)`

폴더/목록 일괄 처리. NDJSON 레코드 **배열**을 돌려준다(봉투 객체가 아니라 평범한 객체).
`sub` 는 `export-text` · `info` · `export-structure` · `export-tables` · `fields` ·
`search` · `convert`.

```ts
for (const r of await batch('export-text', ['a.hwp', 'b.hwp'])) {
  if ('error' in r) console.error(`실패: ${r.source} — ${r.error}`);
}
```

| 옵션 | 적용 축 | 설명 |
|---|---|---|
| `threads` | 전체 | 파일 **간** 병렬 스레드 수. 기본은 CPU 코어 수 |
| `mode` | `export-structure` | 단건 `exportStructure` 의 `mode` 와 같은 값 |
| `query` | `search` | 찾을 문자열. 이 축에서는 **필수**(없으면 exit 2) |
| `outDir` | `convert` | 산출물을 모을 폴더. 이 축에서는 **필수** |
| `verify` | `convert` | 재파싱 IR 비교. 차이가 있으면 집계 종료 코드 3 |
| `verifyPages` | `convert` | 재파싱 쪽수 비교. 불일치면 집계 종료 코드 4 |
| `extraArgs` | 전체 | 위 옵션이 못 담는 축의 탈출구. 이름 붙은 옵션 **뒤에** 놓이므로 같은 플래그를 다시 주면 이쪽이 이긴다 |

```ts
await batch('convert', paths, { outDir: '결과', verify: true, threads: 2 });
await batch('search', paths, { query: '예산' });
```

`threads` 는 **낮추는 쪽으로 쓰는 값이다.** 공유 CI 러너나 메모리가 빠듯한 컨테이너에서
코어 수만큼 문서를 동시에 펼치면 OOM 으로 끝난다.

`convert` 축의 산출 이름은 `<입력이름>.hwp` 로 고정이다. **대소문자만 다른 이름을 포함해**
이름이 겹치면 한 건도 쓰지 않고 exit 2 로 끝난다 — 절반만 쓰고 멈추면 어디까지 진행됐는지
알 수 없기 때문이다.

`search` 축은 단건 `search` 와 달리 `--` 구분자를 쓰지 않으므로, `-` 로 시작하는 검색어는
이 경로로 넘기지 않는다.

기본 제한 시간은 **무제한**이다(단건은 300초) — 대량 작업은 오래 걸린다.

`verify`/`verifyPages` 판정은 예외로 오지 않는다. batch 는 스트림을 끝까지 흘린 뒤 집계
종료 코드(3/4)로만 신호하므로 판정은 레코드를 읽어 판단한다 — 그래서 이 축에는
`throwOnVerdict` 가 **없다.**

부분 실패도 실패지만 **성공분은 남는다**. 스트림을 통째로 버리지 말 것.
스트리밍이 필요하면 [`iterNdjson`](#iterndjsontargs-opts-asynciterableiteratort) 을 쓴다.

### 스키마

#### `exportIrSchema(opts?: { bare?: boolean })`

공개 IR 의 JSON Schema(2020-12) — **문서 모델**의 자기서술. `bare: true` 면 봉투 없이
스키마 본문만이라 JSON Schema 도구에 그대로 먹일 수 있다. 구조화된 접근은
[`irSchema()`](#ir-스키마).

#### `exportCapabilitiesSchema(opts?: { bare?: boolean; out?: string })`

`capabilities` 산출물 **자체의** JSON Schema — **명령 표면**의 자기서술이다.
`capabilities` 가 "이 도구에 어떤 명령이 있나"라면, 이쪽은 "그 대답이 어떤 모양인가"다.

```ts
const schema = await exportCapabilitiesSchema();
schema.get('definitionCount');           // 정의 수
schema.get('capabilitiesSchemaVersion'); // 봉투 schemaVersion 과 별개인 전역 버전
schema.get('mcpSchema');                 // MCP 도구 선언 쪽 스키마

// 저장소에 커밋해 두고 diff 로 표면 변화를 감시할 때
await exportCapabilitiesSchema({ out: 'schema/capabilities.json' });
```

**여기서는 `out` 을 열어 뒀다.** `exportText`·`exportTables` 의 `-o` 와 달리 이쪽은 저장
후에도 stdout 이 봉투를 유지하기 때문이다(`output`·`bytes` 가 담긴다). 같은 `-o` 라도
노출 여부를 가르는 기준은 플래그 이름이 아니라 **봉투 계약을 지키는가**다.

봉투 필드는 `schemaVersion` · `capabilitiesSchemaVersion` · `dialect` ·
`definitionCount` · `schema` · `mcpSchema` 다.

**둘을 따로 두는 이유**: `export-ir-schema` 는 "문서가 어떻게 생겼나"를, 이쪽은
"명령 표면의 자기서술이 어떻게 생겼나"를 서술한다. 하나로 합치면 문서 모델이
바뀔 때 명령 표면 타입까지 재생성되고, 그 반대도 마찬가지다.

**[타입 생성기](#생성-타입)와의 관계**: 생성기가 읽어 코드를 만드는 것은
`capabilities` **데이터** 쪽이고, 이 명령이 주는 것은 그 데이터의 **형태 계약**이다.
생성기가 `commands[].recordFields` 같은 경로에 기대는 모양이 여기 문서화돼 있으므로,
Rust 쪽이 표면 서술을 바꾸면 이 스키마의 정의 수·필드가 먼저 움직인다. 그래서 CI 의
`node-binding` 워크플로가 `src/capabilities_schema.rs` 를 트리거 경로에 넣어 둔다 —
스키마가 바뀌면 커밋된 생성 타입이 낡았는지부터 검사한다.

> **노출 기준은 `capabilities` 의 `json` 선언이다.** `json` 을 선언하지 않는 명령
> (`dump-*`·`hwp5-*` 진단 계열, `mcp-serve`)은 바인딩 표면에 **없다.** 손으로 고른
> 목록이 아니라 자기서술에 기준을 둔 덕분에, 본체가 늘면 바인딩이 뒤처졌다는 사실이
> 계약 패리티 가드에서 드러난다.
>
> `render-diff` 가 그 가드가 **양방향으로** 작동한 사례다. 초기 바인딩은 `--json` 이
> 없던 시절의 `render-diff` 를 감싸 **모든 호출이 exit 2** 였고(통합 테스트 실물 실행이
> 잡아 제거), 그 뒤 `render-diff --json` 이
> [#3719](https://github.com/edwardkim/rhwp/issues/3719) §6-2 로 들어오자 이번에는
> 반대로 **있는 계약을 빠뜨린** 상태가 됐다. 지금은 `renderDiff` 가 있다
> ([변환·비교](#변환비교) 절). 없는 계약을 감싸는 것과 있는 계약을 빠뜨리는 것은 둘 다
> 결함이고, 둘 다 같은 테스트 하나에 걸린다.
>
> 다만 **명령이 노출되는 것과 그 명령의 모든 플래그가 열리는 것은 다르다.** 봉투 계약을
> 깨거나(`exportTables -o`) 아무 일도 하지 않거나(`exportText -o`) 반환 타입을 바꾸는
> (`render-diff --batch`) 플래그는 옵션으로 열지 않았다. 각 자리에 이유를 적어 두었다.

---

## 2층 — 세션

### `openDocument(path, opts?: { password?: string; session?: Session; profile?: string }): Promise<Document>`

문서를 열어 핸들을 돌려준다.

| 옵션 | 설명 |
|---|---|
| `password` | 보호 문서 암호. 서버는 응답·세션 상태에 보존하지 않는다 |
| `session` | 이미 만든 `Session` 에 얹는다(주면 문서를 닫아도 세션은 남는다) |
| `profile` | 새 세션의 역할 프로필(도구 노출 범위 제한) |

이름이 `open` 이 아니라 `openDocument` 인 이유: `open` 은 Node·DOM 양쪽에서 이미 쓰이는
어휘라 named import 로 들여오면 충돌하기 쉽다.

### `class Document`

| 멤버 | 대응 도구 |
|---|---|
| `docId` | 서버가 발급한 핸들 식별자 |
| `info()` | `hwp_doc_info` |
| `text(opts?: { page?: number })` | `hwp_doc_text` (0 기준) |
| `fields()` | `hwp_doc_fields` |
| `tables()` | `hwp_doc_tables` |
| `search(query, opts?: { caseSensitive?: boolean })` | `hwp_doc_search` |
| `renderPage(page, output)` | `hwp_doc_render_page` |
| `fillFields(data)` | `hwp_doc_fill_fields` |
| `replaceText(find, replace, opts?: { caseSensitive?: boolean })` | `hwp_doc_replace_text` |
| `setCell(table, row, col, text)` | `hwp_doc_set_cell` |
| `save(output, opts?: { verify?: boolean })` | `hwp_doc_save` |
| `close()` | `hwp_close` (멱등) |
| `[Symbol.asyncDispose]()` | `await using` 지원 |

**`renderPage` 의 `output` 은 필수다.** 어디에 그렸는지 모르는 렌더는 눈검증 루프를 닫지
못하고, 임시 경로를 바인딩이 임의로 정하면 호출자가 그 파일을 지울 책임을 떠안는다.

편집 도구(`fillFields`·`replaceText`·`setCell`)는 **디스크에 쓰지 않는다.** 핸들의 IR 에
누적하고, `save()` 가 유일한 기록 지점이다. 저장 후에도 핸들은 열려 있어 이어서 편집·재저장할 수 있다.

```ts
const doc = await openDocument('서식.hwp');
try {
  await doc.fillFields({ 성명: '홍길동' });
  const saved = await doc.save('제출본.hwp', { verify: true });
  for (const page of saved.changedPages ?? []) {
    await doc.renderPage(page, `확인_${page}.svg`);   // 바뀐 쪽만 — 상수 비용
  }
} finally {
  await doc.close();
}
```

**예외**: `SessionClosedError` — 닫힌 핸들 재사용.

### `class Session`

`mcp-serve` 자식 프로세스 하나를 감싼 stdio JSON-RPC 클라이언트. 여러 문서를 한 서버에서
열고 싶을 때만 직접 만든다.

| 멤버 | 설명 |
|---|---|
| `call(name, args)` | 도구 하나 호출 → `Envelope` |
| `close()` | 서버 정리. **멱등** |

생성 옵션은 `profile`, `cwd`, `timeoutMs`다. `timeoutMs`는 호출 하나당 제한 시간이며,
기본은 `DEFAULT_SESSION_TIMEOUT_MS`(300000ms), `null`이면 무제한이다. stdio MCP에는
진행 중인 도구 호출의 취소 계약이 없으므로 제한 시간을 넘으면 `RhwpTimeoutError`와 함께
자식 서버를 종료한다. 해당 세션은 재사용하지 말고 새 `Session`을 만들어 재시도한다.

프로토콜 취급 규약:

| 상황 | 처리 | 왜 |
|---|---|---|
| 응답 `id` 가 요청과 다름 | 대조해서 맞는 것만 채택 | 파이프가 어긋나면 남의 답을 내 결과로 읽는다 |
| `id` 없는 프레임(알림) | 건너뜀 | 알림은 응답이 아니다 |
| `isError: true` | `UsageError` (envelope 보존) | 서버가 실은 `nextCall` 교정 단서를 잃지 않는다 |
| 프로세스 조기 종료 | `ProtocolError` (stderr 첨부) | 침묵보다 사유를 준다 |

**서버가 남으면 다음 작업이 파일을 못 연다.** `close()` 를 `finally` 에 두거나
`await using` 을 쓴다.

---

## 3층 — 계획

### `class Plan(input, output)`

체이닝으로 step 을 쌓는 빌더. 빌더는 **문법만** 검사하고(셀 값에 줄바꿈 금지, 좌표는
0 이상 정수 등), 실행 가능성은 rhwp 의 선검증이 판정한다 — 판정자를 두 곳에 두면 어긋난다.

| 메서드 | 설명 |
|---|---|
| `fillFields(data)` | 누름틀 채우기 |
| `replaceText(find, replace, opts?: { occurrence?: number; caseSensitive?: boolean })` | 치환 |
| `setCell(table, row, col, text, opts?: { keepStyle?: boolean })` | 셀 기록 |
| `setCheckbox(occurrence)` | 빈 체크박스(□ → ☑) 표시 |
| `verify(enabled?: boolean)` | 저장 직후 자기검증 요구 |
| `requireAllFieldsFound(enabled?: boolean)` | 못 찾은 필드 0 단언 |
| `toJSON(opts?: { dryRun?: boolean })` | 계획서 JSON 구조 |
| `check()` | **디스크 무변경** 검사 → `Promise<PlanResult>` |
| `run()` | 실행 → `Promise<PlanResult>` |

`toJSON` 이라는 이름은 우연이 아니다 — `JSON.stringify(plan)` 이 그대로 계획서를 낸다.
계획서를 파일로 남기면 감사 추적·재현이 따라온다.

```ts
const plan = new Plan('서식.hwp', '제출본.hwp')
  .fillFields({ 성명: '홍길동' })
  .setCheckbox(1)
  .verify();

const preview = await plan.check();
if (preview.ok) await plan.run();
else console.error(preview.describeViolations());
```

#### `check()` 는 지원 여부를 먼저 확인한다

`check()` 는 계획서에 `dryRun: true` 를 실어 보내기 **전에** `capabilities` 로
rhwp 의 `run` 이 `--dry-run` 을 선언하는지 확인한다. 선언하지 않으면 실행하지 않고
`RhwpError` 를 던진다.

```ts
try {
  await plan.check();
} catch (e) {
  // "이 rhwp 는 계획 --dry-run 을 지원하지 않습니다 (#3759 이전 버전)."
}
```

**왜 조용히 실행으로 내려가지 않는가**: `--dry-run` 을 모르는 rhwp 는 계획서의
`dryRun` 필드를 그냥 무시하고 **편집을 수행하고 저장한다**. 호출자는 "검사만 했다"고
믿고 있으므로, 실패도 예외도 없이 문서가 바뀐다. 조용한 데이터 사고이고, 눈으로
확인하기 전까지 아무도 모른다. 그래서 확실히 검사인 경우에만 진행하고, 아니면 멈춘다.

> **`check()` 실패는 계획이 나쁘다는 뜻이 아니다.** rhwp 가 낡았다는 뜻이다.
> 판정 실패(`preview.ok === false`)와 혼동하지 말 것 — 그쪽은 예외가 아니라 결과다.

확인 결과는 프로세스 수명 동안 캐시된다 — 계획 하나 부를 때마다 `capabilities` 를 다시
띄울 이유가 없다. 테스트에서 rhwp 를 바꿔 가며 검사할 때는 `clearPlanCapabilityCache()`
로 비운다.

### `clearPlanCapabilityCache(): void`

`check()` 의 `--dry-run` 지원 여부 캐시를 비운다. 테스트 전용이다.

### `class PlanResult`

| 멤버 | 설명 |
|---|---|
| `ok` | 위반 없이 통과했는가 |
| `violations` | 선검증 위반 목록 |
| `isDryRun` | 검사 전용 실행이었는가 |
| `preview` | 검사 모드의 step 별 미리보기 |
| `steps` | 실행 모드의 step 별 결과 |
| `describeViolations()` | 위반을 사람이 읽을 여러 줄로 |

`PlanResult` 도 봉투이므로 `verify`·`changedPages` 를 그대로 갖는다.

**위반은 예외가 아니라 결과다** — 계획을 고쳐 다시 검사하는 것이 정상 흐름이다.

---

## IR 스키마

### `irSchema(): Promise<IrSchema>`

`export-ir-schema` 를 읽어 온다. 문서를 입력으로 받지 않는다 — 스키마는 **타입의
자기서술**이지 특정 문서의 속성이 아니다.

### `class IrSchema`

| 멤버 | 설명 |
|---|---|
| `version` | IR 스키마 버전(봉투 `schemaVersion` 과 별개인 전역 버전) |
| `dialect` | JSON Schema 방언 URI |
| `root` | 루트 타입(`Document`) |
| `names()` | 정의 이름 목록 |
| `danglingReferences()` | 끊어진 `$ref` 를 (참조한 곳, 없는 이름)으로 |
| `get(name)` | 이름으로 `TypeDef` |
| `has(name)` | 있는지 |
| `[Symbol.iterator]()` | `for (const t of schema)` |

### `class TypeDef`

| 멤버 | 설명 |
|---|---|
| `name`, `description` | 이름·설명 |
| `isObject`, `isUnion` | 종류 |
| `variants` | 유니온이면 변형 이름 목록 |
| `fields` | 필드 목록(필수가 앞) |
| `field(name)` | 이름으로 하나 |

### `class FieldDef`

| 멤버 | 설명 |
|---|---|
| `name`, `description`, `required` | 기본 |
| `jsonType` | JSON 타입 |
| `ref`, `itemRef` | 참조 대상(배열이면 `itemRef`) |
| `enumValues` | 열거형 허용 값 |
| `tsType` | TypeScript 타입 문자열(코드 생성기가 쓴다) |

---

## 봉투

### `class Envelope<T>`

봉투 하나를 감싸는 읽기 전용 래퍼.

| 멤버 | 설명 |
|---|---|
| `raw` | 원문 봉투 |
| `get(key)` | 키 하나. **없는 키는 예외** |
| `getPath('verify.identical')` | 점 경로 조회 |
| `has(key)` | 있는지 (예외 없이) |
| `keys()` | 키 목록 |
| `child(key)` | 하위 **객체**를 봉투로. 객체가 아니면 `null` |
| `children(key)` | 배열 필드를 봉투 배열로 |
| `schemaVersion` | 봉투 스키마 버전 |
| `verify` | `VerifyReport \| null` (`null` = 검증 안 함) |
| `changedPages` | `number[] \| null` (`null` = 확정 불가) |

```ts
const meta = await info('보고서.hwp');
meta.get('pageCount');
meta.getPath('verify.identical');
meta.get('pageConut');    // 예외 — 있는 필드를 함께 알려준다
```

#### `children(key)` 는 봉투가 될 수 없는 항목을 걸러 낸다

```ts
const tables = await exportTables('양식.hwpx');
for (const t of tables.children('tables')) t.get('rowCount');
```

배열 항목 중 **객체가 아닌 것**(숫자·문자열·`null`·그리고 **다시 배열인 것**)은 제외한다.
`Envelope` 생성자는 배열을 거부하므로, `[[1,2]]` 같은 값을 걸러 내지 않으면 감싸는
그 자리에서 `TypeError` 가 난다 — 조회하려던 필드와 아무 상관 없는 곳에서 터지므로
원인 추적이 가장 어려운 형태다. `get()` 이 오타에 예외를 던지는 것과 달리 여기서는
조용히 제외하는 것이 맞다: 배열 필드에 이질적 항목이 섞이는 것은 **호출자의 실수가 아니라
봉투의 모양**이고, 호출자가 할 수 있는 일이 없다.

`child(key)` 도 같은 규칙이다 — 값이 `null`·비객체·배열이면 던지지 않고 `null` 을 준다.

**없는 필드 접근은 조용한 `undefined` 가 아니라 예외다.** 조용한 `undefined` 는 오타를
"값 없음"으로 둔갑시키고, 그 코드는 실패하지 않고 잘못된 결과를 낸다.

### `class VerifyReport`

| 멤버 | 설명 |
|---|---|
| `identical` | 저장본이 메모리 IR 과 같은가(판정의 전부) |
| `diffCount` | 차이 개수. 재파싱 실패면 `null` |
| `reparseError` | 저장본을 못 읽었을 때의 사유 |

**`null`(검증 안 함)과 실패는 다르다.** `changedPages` 의 `null`(모름)과 `[]`(없음)도 다르다.

---

## 생성 타입

`src/ir.ts` 와 `src/envelopes.ts` 는 **생성기 산출물이다. 손으로 고치지 않는다.**

| 출처 | 무엇 | 산출 |
|---|---|---|
| `rhwp export-ir-schema` | 공개 IR 정의 41개 | `src/ir.ts` |
| `rhwp capabilities` | 명령별 `recordFields` | `src/envelopes.ts` |

```bash
npm run gen:types     # 재생성 (tools/gen-types.ts)
npm run gen:check     # 디스크와 다르면 exit 1 — CI 가 뒤처짐을 잡는다
```

두 출처를 쓰는 이유: IR 스키마는 **문서 모델**을 서술하고, `capabilities` 는 **명령별 봉투**를
서술한다. 어느 한쪽만으로는 봉투 필드에 정적 타입을 붙일 수 없다.

---

## 브라우저 어댑터

### `createBrowserClient(wasm): RhwpClient`

`@rhwp/node/browser` 에서 들여온다. `@rhwp/editor` WASM 을 **같은 봉투 타입**으로 감싼다.

```ts
import { createBrowserClient } from '@rhwp/node/browser';

const client = createBrowserClient(wasm);
```

Node 경로와 브라우저 경로가 같은 `RhwpClient` 인터페이스를 구현하므로 소비자 코드는 환경
독립적이다. 서브프로세스를 띄울 수 없는 곳에서 전혀 다른 API 를 쓰게 하면, 같은 업무 로직을
두 벌 유지해야 한다.

**한계**: WASM 모듈은 호출자가 로드해 넘긴다(번들 크기를 바인딩이 강제하지 않는다).
파일 시스템에 쓰는 명령은 브라우저에서 성립하지 않는다.

---

## 예외

```
RhwpError
├── BinaryNotFoundError    실행 파일 없음
├── UsageError             exit 2 — 호출 조립 버그 (.suggestion .nextCall)
├── RhwpRuntimeError       exit 1 — 읽기·파싱·렌더·쓰기 실패, 그리고 알 수 없는 종료 코드
├── VerdictFailed          exit 3/4 — throwOnVerdict: true 일 때만
├── ProtocolError          stdout 이 계약 위반
├── SessionClosedError     닫힌 핸들 재사용
├── EnvelopeKeyError       봉투에 없는 필드 조회
└── RhwpTimeoutError       제한 시간 초과
```

이름이 `TimeoutError` 가 아니라 `RhwpTimeoutError` 인 이유: 전역 `Error` 계열과 이름이
겹치면 `catch (e) { if (e instanceof TimeoutError) }` 가 어느 쪽을 잡는지 흐려진다.

모든 예외가 갖는 것:

| 멤버 | 설명 |
|---|---|
| `message` | 사람이 읽을 설명 |
| `argv` | 실행한 명령줄 |
| `command` | 재현 가능한 명령 문자열(버그 리포트용, 공백은 따옴표로 감싼다) |
| `exitCode` | 종료 코드. 프로세스를 못 띄웠으면 `undefined` |
| `stderr` | 도구 진단 원문 |
| `lastDiagnostic` | stderr 마지막 줄 — 가장 구체적인 진단 |
| `envelope` | 파싱된 봉투(판정 근거 보존) |

### 하위 클래스 고유 멤버

| 클래스 | 멤버 | 설명 |
|---|---|---|
| `UsageError` | `suggestion` | stderr 의 `힌트:` 줄에서 뽑은 did-you-mean 제안 |
| `UsageError` | `nextCall` | 서버가 실은 교정 호출 `{ name, arguments?, why? }` |
| `VerdictFailed` | `isPageCountMismatch` | exit 4 인가 |

`instanceof` 는 트랜스파일 이후에도 동작한다 — 프로토타입을 명시 복원해 두었다.

### 종료 코드

| 상수 | 값 | 의미 |
|---|---|---|
| `EXIT_OK` | 0 | 성공 |
| `EXIT_RUNTIME` | 1 | 읽기·파싱·렌더·쓰기 실패 |
| `EXIT_USAGE` | 2 | 인자 오류(호출 조립 버그) |
| `EXIT_VERIFY` | 3 | 검증 단언 실패 |
| `EXIT_VERIFY_PAGES` | 4 | 페이지 수 불일치 |

### `raiseForExit(code, opts?)`

종료 코드를 검사해 필요하면 던진다. `opts` 는 `{ argv, stderr, envelope, throwOnVerdict, cause }`.
사전에 없는 0 아닌 코드는 `RhwpRuntimeError` 다 — 모르는 코드를 성공으로 취급하면
실패한 작업이 성공으로 보고된다.

### `isKnownExitCode(n): n is KnownExitCode`

알려진 코드인지. 타입 좁히기용.

---

## 저수준

직접 쓸 일은 드물지만, 바인딩이 아직 감싸지 않은 명령을 부를 때 필요하다.

### `runJson<T>(args, opts?): Promise<T>`

`--json` 명령을 실행하고 봉투를 돌려준다. 종료 코드 검사는 **파싱 뒤**에 한다 —
exit 3 일 때도 봉투에 판정 근거가 있기 때문이다. 순서를 뒤집으면 판정 근거를 버린다.

```ts
const envelope = await runJson(['새명령', '문서.hwp', '--json']);
```

stdout 이 JSON 이 아니거나, 성공(exit 0)인데 stdout 이 비어 있으면 `ProtocolError` 다.

### `runNdjson<T>(args, opts?): Promise<T[]>`

batch 계열. **부분 실패를 예외로 올리지 않는다** — 성공 레코드를 잃지 않기 위해서다.

### `runRaw(args, opts?): Promise<CompletedRun>`

원문 결과. `CompletedRun` 은 `{ argv, exitCode, stdout, stderr }`.
이미 파싱한 실패 봉투를 `envelopeHint`로 주면 `RhwpError.envelope`에 보존한다.

### `iterNdjson<T>(args, opts?): AsyncIterableIterator<T>`

NDJSON 을 나오는 대로 흘린다. 중단(`break`·예외)하면 자식 프로세스를 정리한다.

```ts
for await (const record of iterNdjson(['batch', 'info', '--json'], { stdin: pathsText })) {
  handle(record);
}
```

### 이름 변환

| 함수 | 설명 |
|---|---|
| `toSnake(s)` | `pageCount` → `page_count` |
| `toCamel(s)` | `page_count` → `pageCount` |
| `snakeKeys(v)` | 객체/배열의 키를 재귀 변환 |
| `camelKeys(v)` | 반대 방향 |

기계 변환이다. **수기 개명은 금지** — 사람이 이름을 붙이기 시작하면 매핑 표를 유지해야 하고,
그 표는 반드시 뒤처진다.

```
pageCount ↔ page_count
sourceA   ↔ source_a
irSchemaVersion ↔ ir_schema_version
```

봉투 키는 camelCase 이므로 TypeScript 에서는 변환 없이 그대로 쓰는 것이 기본이다.
변환기는 snake_case 를 쓰는 외부 시스템(파이썬 바인딩과 같은 데이터 형태를 주고받을 때 등)과
붙일 때 필요하다.
