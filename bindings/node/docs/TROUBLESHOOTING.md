# 문제 해결 — 증상으로 찾는 원인과 처방

증상 문자열로 검색해서 쓰는 문서다. 각 항목은 **왜 그렇게 설계됐는지**까지 적는다 —
이유를 모르면 같은 문제를 다른 방식으로 다시 만든다.

---

## 설치·실행

### `BinaryNotFoundError: rhwp 실행 파일을 찾지 못했습니다`

바인딩은 rhwp 실행 파일을 세 곳에서 찾는다. 메시지에 시도한 위치가 전부 적혀 있다.

```
1. RHWP_BIN (미설정)
2. 패키지 동봉 (/.../dist/_bin/rhwp)
3. PATH (rhwp 없음)
```

**처방**: 셋 중 하나를 만족시킨다.

```bash
export RHWP_BIN=/path/to/rhwp          # 가장 확실
# 또는 PATH 에 두기
```

**왜 자동 설치가 없나**: 패키지에 플랫폼별 바이너리를 동봉하려면 OS × arch 별 릴리스
매트릭스가 필요하고, 그것은 이 마일스톤의 범위 밖이다. 지금은 rhwp 를 따로 설치하는
대신, **어느 바이너리가 실행되는지 항상 명확하다**.

**왜 "없다"만 말하지 않나**: 시도한 경로를 전부 보여 주지 않으면 사용자가 어디에 둬야
할지 모른다. 오류 메시지의 목적은 비난이 아니라 다음 행동이다.

### `BinaryNotFoundError: RHWP_BIN 가 가리키는 실행 파일을 쓸 수 없습니다`

환경변수를 **줬는데** 그 경로가 없거나, 파일이 아니거나, 실행 권한이 없다.

**왜 조용히 넘어가지 않나**: 사용자는 그 바이너리를 쓰고 있다고 믿는데 다른 게 실행되면
"왜 내 수정이 반영 안 되지"라는 진단 불가 상황이 된다. 탐색 **순서 자체가 계약**인 이유다.

```bash
ls -l "$RHWP_BIN"        # 존재·권한 확인
chmod +x "$RHWP_BIN"     # 유닉스
```

윈도우에서는 실행 비트가 없으므로 확장자(`.exe`/`.bat`/`.cmd`)로 판단한다.
`RHWP_BIN` 에 디렉터리를 줘도 된다 — 그 안에서 `rhwp`(윈도우는 `rhwp.exe`)를 찾는다.

### 환경변수를 바꿨는데 예전 경로가 나온다

탐색 결과는 프로세스 수명 동안 캐시된다 — 명령 하나마다 `PATH` 를 훑을 이유가 없기 때문이다.

```ts
import { clearBinaryCache, findBinary } from '@rhwp/node';
clearBinaryCache();
findBinary({ refresh: true });
```

### `Exec format error` / `spawn ENOENT`

플랫폼이 맞지 않는 바이너리이거나(리눅스 빌드를 macOS 에서), 경로 자체가 사라졌다.
`findBinary()` 로 무엇이 선택됐는지 먼저 본다.

---

## 종료 코드·예외

### `UsageError: 호출 인자가 올바르지 않습니다`

**exit 2 — 호출 조립이 틀렸다. 우리 쪽(또는 호출자) 버그다.** 재시도해도 같은 결과가
나오므로 인자를 고쳐야 한다.

도구가 교정 단서를 줬으면 꺼내 쓴다.

```ts
try {
  await exportSvg('보고서.hwp');
} catch (e) {
  if (e instanceof UsageError) {
    console.error(e.suggestion);   // "가장 가까운 명령은 'export-svg' 입니다"
    console.error(e.nextCall);     // 기계가 그대로 따라할 수 있는 교정 호출
    console.error(e.command);      // 재현 가능한 명령 문자열
  }
}
```

흔한 원인:

| 원인 | 확인 |
|---|---|
| 없는 누름틀 이름 | `await fields(path)` |
| 범위 밖 표·셀 좌표 | `await exportTables(path)` |
| 범위 밖 쪽 번호 | `(await info(path)).get('pageCount')` |
| 쪽 기준 혼동 | `extractPages` 만 1 기준, 나머지는 0 기준 |

### `RhwpRuntimeError: 문서 처리에 실패했습니다`

**exit 1 — 읽기·파싱·렌더·쓰기가 실패했다.** 인자를 고쳐도 안 풀리며 입력 자체를 봐야 한다.

```ts
if (e instanceof RhwpRuntimeError) {
  console.error(e.stderr);           // 도구가 남긴 진단 원문
  console.error(e.lastDiagnostic);   // 가장 구체적인 한 줄
}
```

흔한 원인: 파일 없음 · 손상된 문서 · 암호 필요 · 디스크 쓰기 권한 없음.

**왜 진단이 stdout 이 아니라 stderr 에 있나**: `--json` 모드의 stdout 은 **순수 JSON**이다.
진단을 섞으면 파이프로 받는 쪽이 JSON 파서를 못 쓴다.

### 검증이 실패했는데 예외가 안 난다

**의도된 동작이다.** `--verify` 불일치나 회귀 검출은 **도구가 정상 동작한 결과**다.
판정은 반환값으로 읽는다.

```ts
const result = await exportHwpx('a.hwp', { out: 'b.hwpx', verify: true });
if (!result.verify?.identical) console.log(`차이 ${result.verify?.diffCount}건`);
```

예외를 원하면 명시한다.

```ts
await exportHwpx('a.hwp', { verify: true, throwOnVerdict: true });   // VerdictFailed
```

시각 회귀도 같다. `renderDiff` 는 회귀를 **봉투의 `status`·`regression` 필드**로 준다.

```ts
const geom = await renderDiff('a.hwp', 'b.hwpx');
if (geom.get('regression')) console.log(geom.get('status'), geom.get('maxDisp'));
```

**왜 기본이 아닌가**: 예외로 올리면 호출자가 `try/catch` 로 "고장"처럼 다루게 되고,
정작 봉투에 담긴 판정 근거를 읽지 않는다. 그러면 "변환은 됐지만 표 한 개가 달라졌다"와
"파일을 못 읽었다"가 같은 코드 경로로 처리된다 — 둘의 대응은 전혀 다른데도.

**왜 TypeScript 관례를 따르지 않나**: 관례보다 rhwp 계약의 의미론을 보존하는 쪽이 옳다고
판단했다. 파이썬 바인딩도 같은 선택을 했고, 두 언어가 다르게 굴면 계약이 언어마다 갈린다.

### `RhwpRuntimeError: 알 수 없는 종료 코드입니다 (N)`

rhwp 가 사전에 없는 코드를 냈다 — 본체와 바인딩 버전이 어긋났을 가능성이 높다.

**왜 조용히 통과시키지 않나**: 모르는 코드를 성공으로 취급하면 실패한 작업이 성공으로
보고된다. 조용한 성공은 조용한 실패보다 나쁘다.

```ts
(await capabilities()).get('exitCodes');   // 이 rhwp 가 아는 코드
```

### `instanceof` 가 안 먹는다

번들러가 `@rhwp/node` 를 **두 번**(ESM 사본 + CJS 사본) 넣으면 클래스 실체가 둘이 되어
`instanceof` 가 어긋난다. `npm ls @rhwp/node` 로 중복을 확인하고, 앱 전체를 한쪽
모듈 시스템으로 통일한다.

바인딩 자체는 트랜스파일 이후에도 `instanceof` 가 동작하도록 프로토타입을 명시 복원한다
(내장 `Error` 상속이 ES5 로 내려갈 때 깨지는 알려진 함정).

---

## 봉투·필드

### `봉투에 'xxx' 필드가 없습니다`

오타이거나, 그 명령이 그 필드를 내지 않는다. 메시지에 **있는 필드가 함께** 나온다.

**왜 `undefined` 가 아닌가**: 없는 필드가 조용히 `undefined` 가 되면, 이름을 잘못 쓴
코드가 "값이 없네"로 흘러가 가장 찾기 어려운 버그가 된다. `?.` 와 `??` 가 흔한 언어일수록
그 오독은 더 깊이 숨는다.

```ts
Object.keys(result.raw);                        // 실제 필드 목록
(await capabilities()).get('commands');         // 선언된 recordFields
```

### `result.verify` 가 `null` 인데 실패로 읽힌다

`null` 은 **"검증 안 함"**이지 "검증 실패"가 아니다.

```ts
if (result.verify === null) console.log('verify: true 를 주지 않았습니다');
else if (result.verify.identical) console.log('통과');
else console.log('실패');
```

### `changedPages` 가 `null` 이다

**확정할 수 없다는 뜻**이다. `[]`(바뀐 쪽 없음)과 다르다.

변경 문단 중 하나라도 조판 커버리지 밖이면 rhwp 는 부분 목록 대신 `null` 을 낸다 —
빠뜨린 쪽이 있는 목록은 **거짓 통과**를 만들기 때문이다. 확인해야 할 쪽을 하나 빠뜨린
검증은 검증하지 않은 것보다 위험하다.

```ts
const pages = result.changedPages;
if (pages === null) console.log('전체 확인이 필요합니다');
else if (pages.length === 0) console.log('바뀐 쪽이 없습니다');
else render(pages);
```

### 봉투를 고칠 수 없다

읽기 전용이다. 도구가 내놓은 판정을 호출자가 고치면 그 뒤의 모든 판단이 근거를 잃는다.
값을 가공하고 싶으면 `result.raw` 를 복사해 쓴다.

---

## 산출·출력

### `exportText`·`exportStructure`·`exportTables` 에 `out` 옵션이 없다

**일부러 열지 않았다.** rhwp 쪽 동작이 `--json` 봉투 계약과 맞지 않기 때문이다.

| 명령 | `-o` 를 주면 | 그래서 |
|---|---|---|
| `export-text` · `export-structure` | `--json` 모드에서 **조용히 무시**된다(실측: 디렉터리조차 생기지 않는다) | 바인딩이 받아 주면 "저장했다"는 거짓말이 된다 |
| `export-tables` | stdout 이 사람용 문장("표 추출 완료: N개 → 경로")으로 **바뀐다** | `--json` 봉투 계약이 깨져 `runJson` 이 `ProtocolError` 로 터진다 |

**처방**: 봉투 내용을 직접 쓴다.

```ts
import { writeFile } from 'node:fs/promises';
const env = await exportText('보고서.hwp');
const text = (env.get('pages') as Array<{ text: string }>).map((p) => p.text).join('\n');
await writeFile('본문.txt', text, 'utf8');
```

`exportTables` 쪽은 바인딩이 옵션을 닫은 것이 **회피일 뿐**이다. `--json` 을 준 호출에서
출력 형식이 바뀌는 것은 명령 하나의 사정이 아니라 봉투 계약 전체의 예외이므로,
근본 수정은 **본체 쪽 별도 이슈**다. 저수준으로 `runJson(['export-tables', p, '-o', d,
'--json'])` 를 부르면 지금도 `ProtocolError` 를 만난다 — 바인딩 버그가 아니다.

반대 사례로 `exportCapabilitiesSchema` 는 `out` 을 **연다.** 같은 `-o` 라도 이쪽은 저장
후에도 stdout 이 봉투를 유지하기 때문이다(`output`·`bytes`). 노출을 가르는 기준은 플래그
이름이 아니라 **봉투 계약을 지키는가**다.

### `exportHwpx`·`convert` 를 CLI 로 옮겼더니 exit 2 가 난다

**두 명령의 산출 경로는 `-o` 가 아니라 위치 인자다.** `-o` 를 주면 `알 수 없는 옵션: -o`
로 exit 2 다.

```bash
rhwp export-hwpx 원본.hwp -o 변환본.hwpx --json   # exit 2
rhwp export-hwpx 원본.hwp 변환본.hwpx --json      # 이것이 맞다
rhwp convert 배포본.hwpx 편집본.hwp --json
```

바인딩은 이 차이를 흡수하므로 TypeScript 에서는 `out` 하나로 통일돼 있다.

### `UsageError: convert 는 산출 경로가 필요합니다`

`convert` 는 기본 산출 경로가 **없어서** `out` 이 필수다(`exportHwpx` 는 생략하면
`<입력 stem>.hwpx`). 바인딩이 프로세스를 띄우기 전에 던지는 이유는, 그래야 **무엇이
빠졌는지 이름으로** 알려 줄 수 있기 때문이다 — CLI 사용법 덤프는 "인자가 틀렸다"까지만
말한다.

### `thumbnail` 에 `out` 을 줬는데 파일이 없다

`base64` 나 `dataUri` 를 함께 켰을 것이다. **두 플래그는 파일 출력을 대체한다** — 켜면
`out` 을 줘도 파일이 생기지 않고 봉투의 `output` 이 `null` 이다.

```ts
await thumbnail(p, { out: 'a.png' });        // 파일
await thumbnail(p, { base64: true });        // 봉투의 base64
```

둘을 **함께 켜도 안 된다.** 나중 플래그가 이겨 `dataUri` 만 오므로, `base64` 를 읽으려던
코드가 `undefined` 를 만난다. 하나만 고른다.

### `extractPages` 결과가 한 쪽씩 밀린다

**`extractPages` 의 `from`·`to` 만 1 기준이다.** 다른 명령의 `page` 인자와 `digest` 의
`pages` 창은 전부 0 기준이다. rhwp 본체가 그렇게 굳어 있어 바인딩이 임의로 통일하지
않는다 — 재포장은 계약을 바꾸지 않는다.

```ts
const page = hits.children('matches')[0]!.get('page') as number;   // 0 기준
await extractPages(p, page + 1, page + 1, { out: '발췌.hwp' });     // 이것이 맞다
```

`--from 0` 은 실물이 "쪽 범위가 잘못됐습니다 … (1 기준)" 으로 **exit 1** 을 낸다.
범위 문자열(`--pages "2-4"`)은 `digest` 쪽 어휘라 여기 섞으면 `알 수 없는 옵션` 으로
exit 2 다.

**결과 쪽수가 요청 범위와 다를 수 있다.** rhwp 는 쪽 단위로 자르되 **문단 단위로** 지우고,
지운 뒤 레이아웃이 다시 흐른다. 봉투의 `pagesBefore`·`pagesAfter` 로 실제 결과를 읽는다.

### `renderDiff` 로 폴더를 통째로 돌리고 싶다

`--batch` 축은 이 함수가 감싸지 않는다. **NDJSON 스트림이라 반환 타입이 다르기**
때문이다 — 한 함수가 봉투와 배열을 모두 돌려주면 호출자가 받은 값이 무엇인지 타입으로
알 수 없다.

```ts
const rows = await runNdjson(['render-diff', '--batch', '샘플/', '-o', '결과/', '--json']);
```

### `renderDiff` 의 종료 코드가 CLI 와 다르다

**모드마다 다르다.** `--json` 모드(= 이 함수)에서 회귀 검출은 **exit 3**(판정)이고,
사람용 출력 모드는 **exit 1**(종전 그대로)이다.

새 의미론을 `--json` 소비자에게만 준 이유는 **이미 1 을 실패로 읽는 CI 스크립트를 깨지
않기 위해서**다. 3 으로 가른 이유는 그 반대편에 있다 — CI 가 "렌더가 깨졌다"와 "파일을
못 읽었다"를 같은 신호로 받으면 안 된다.

### `renderDiff` 가 `-p` 에 exit 2 를 낸다

문서에 없는 쪽 번호를 줬다(0 기준). 조용한 빈 결과로 내면 "필터가 안 맞았다"와
"차이가 없다"가 같은 출력이 되는데, 정반대 결론인데도 소비자는 후자로 읽는다.
그래서 사용법 오류로 끊는다.

---

## 세션

### `SessionClosedError: 닫힌 문서 핸들입니다`

`close()` 뒤에 `doc` 을 다시 썼다.

```ts
const doc = await openDocument('a.hwp');
await doc.close();
await doc.info();          // ← 여기서 실패
```

세션을 유지하고 싶으면 명시적으로 관리한다. `Session` 을 넘겨 만든 문서는 닫아도 서버가 남는다.

```ts
const session = new Session();
try {
  const a = await openDocument('a.hwp', { session });
  await a.close();                                  // 문서만 닫힘
  const b = await openDocument('b.hwp', { session });
  await b.close();
} finally {
  await session.close();
}
```

**왜 재사용을 막지 않고 예외를 내나**: 닫힌 핸들에 조용히 `undefined` 를 돌려주면
"편집이 반영 안 된 것 같은데 오류는 없다"가 된다.

### 프로그램이 끝났는데 프로세스가 남는다

세션을 열고 닫지 않았다. **서버가 남으면 파일을 잡고 있어 다음 작업이 막힌다.**

```ts
const doc = await openDocument('a.hwp');
try { /* ... */ } finally { await doc.close(); }
```

`close()` 는 멱등이다 — 두 번 불러도 안전하다. 정리 경로에서는 새 예외를 만들지 않는다
(원인 예외를 가리면 진단이 어려워진다). stdin 을 닫아도 안 죽으면 강제 종료한다.

### `await using` 이 `Symbol.asyncDispose is not defined` 로 죽는다

`await using` 은 TypeScript 5.2+ 문법이지만, **런타임에 `Symbol.asyncDispose` 가 실제로
있어야** 한다(Node 20+). Node 18 에서는 폴리필을 넣거나 `try/finally` 를 쓴다.

```ts
// 폴리필 (앱 진입점에서 한 번)
(Symbol as { asyncDispose?: symbol }).asyncDispose ??= Symbol.for('Symbol.asyncDispose');
```

**왜 바인딩이 폴리필을 넣지 않나**: 전역을 건드리는 부작용은 라이브러리가 할 일이 아니다.
`sideEffects: false` 를 선언한 패키지가 전역을 오염시키면 번들러의 가정이 깨진다.

### `ProtocolError: 응답 없이 종료했습니다`

`mcp-serve` 가 죽었다. `e.stderr` 에 사유가 있다.

**왜 침묵하지 않나**: 서버가 죽었는데 `pending` 상태로 남기면 호출이 영원히 매달린다.
프로토콜 위반은 즉시 드러내는 편이 싸다.

### `UsageError: hwp_doc_xxx 호출이 거부됐습니다`

도구가 `isError` 를 세웠다. 서버가 교정 단서를 실어 보내면 `e.envelope` 에 있다.

```ts
if (e instanceof UsageError) console.error(e.nextCall);
// { name: 'hwp_open', arguments: { path: '...' }, why: '...' }
```

`nextCall` 은 **기계가 그대로 따라할 수 있는 교정 호출**이다. 그래서 예외를 던질 때도
봉투를 버리지 않는다.

---

## 계획

### `RhwpError: 이 rhwp 는 계획 --dry-run 을 지원하지 않습니다 (#3759 이전 버전)`

`Plan.check()` 가 던졌다. **계획이 나쁘다는 뜻이 아니라 rhwp 가 낡았다는 뜻이다.**

```ts
try {
  const preview = await plan.check();
} catch (e) {
  // rhwp 를 갱신한다. 검사 없이 run() 을 부를지는 사람이 정한다.
}
```

`check()` 는 계획서에 `dryRun: true` 를 실어 보내기 **전에** `capabilities` 로 rhwp 의
`run` 이 `--dry-run` 을 선언하는지 확인하고, 선언하지 않으면 **실행하지 않고 던진다.**

**왜 조용히 실행으로 내려가지 않나**: `--dry-run` 을 모르는 rhwp 는 계획서의 `dryRun`
필드를 **그냥 무시하고 편집을 수행하고 저장한다.** 실측으로 재현했다.

```
$ rhwp run --plan-json '{…,"dryRun":true,"steps":[{"action":"replace_text",…}]}' --json
{"changedPages":[0],"output":"…/plan_out.hwp",…,"verify":null}
exit=0

$ ls -l plan_out.hwp
-rw-r--r-- 1 … 473600 … plan_out.hwp        ← 편집본이 생겼다
```

exit 0, 경고 없음, 봉투에 `dryRun` 필드 없음 — 그런데 473,600바이트짜리 편집본이 있다.
호출자는 "검사만 했다"고 믿는다. **실패도 예외도 없이 문서가 바뀌므로 눈으로 열어보기
전까지 아무도 모른다.** 조용한 데이터 사고이고, "미지원이면 그냥 실행"이라는 편의의
대가로 바인딩이 지불해도 되는 것이 아니다.

**처방**: rhwp 를 갱신한다. 갱신할 수 없고 위험을 감수하겠다면 `run()` 을 **명시적으로**
부른다 — 그 선택은 바인딩이 아니라 사람이 한다.

> 이 예외는 계획 위반(`preview.ok === false`)과 **전혀 다른 사건**이다. 그쪽은 예외가
> 아니라 결과다.

확인 결과는 프로세스 수명 동안 캐시된다. 테스트에서 rhwp 를 바꿔 가며 검사하려면
`clearPlanCapabilityCache()` 로 비운다.

### `check()` 가 통과했는데 `run()` 이 실패한다

원칙적으로 없어야 한다 — 검사와 실행이 **같은 판정자**를 쓰기 때문이다. 발생하면
버그이므로 이슈를 열어 달라. 단, 검사와 실행 사이에 **문서가 바뀌면** 결과가 달라진다.

### 위반이 났는데 예외가 아니다

**의도된 동작이다.** 위반은 결과이고, 계획을 고쳐 다시 검사하는 것이 정상 흐름이다.
예외로 만들면 `try/catch` 안에서 계획을 고치는 어색한 코드가 된다.

```ts
const result = await plan.check();
if (!result.ok) console.error(result.describeViolations());
```

### 빌더가 값을 거부한다 (셀 값의 줄바꿈 등)

빌더는 **문법만** 검사한다: 셀 값에 줄바꿈·탭 금지, 좌표는 0 이상 정수, step 0개 금지.

**왜 여기서만 막나**: 실행 가능성(그 필드가 존재하는가, 그 좌표가 병합으로 덮였는가)은
rhwp 의 선검증이 판정한다. 판정자를 두 곳에 두면 반드시 어긋나고, 그때 어느 쪽이 맞는지
아무도 모른다.

---

## ESM · CJS

### `ERR_REQUIRE_ESM` / `Cannot use import statement outside a module`

앱과 패키지의 모듈 시스템이 어긋났다. 이 패키지는 듀얼이므로 **양쪽 다 된다** — 문제는
보통 앱 설정이다.

| 증상 | 처방 |
|---|---|
| CJS 앱인데 ESM 산출물이 걸림 | `require('@rhwp/node')` 그대로 쓴다. 번들러라면 `conditions` 확인 |
| ESM 앱인데 `__dirname` 오류 | 앱 코드 문제. 바인딩은 두 경우를 모두 처리한다 |
| 타입만 `any` 로 보임 | `moduleResolution` 이 `node16`/`bundler` 인지 확인 |

### `tsconfig` 를 고쳐도 타입이 안 잡힌다

`exports` 맵 기반 패키지는 **`moduleResolution: "node"`(구식)에서 서브 경로를 못 읽는다.**
`@rhwp/node/browser` 가 특히 그렇다.

```jsonc
{ "compilerOptions": { "module": "node16", "moduleResolution": "node16" } }
// 번들러를 쓴다면 "bundler"
```

**왜 듀얼로 내나**: Node 생태계가 아직 둘로 갈려 있다. 한쪽만 내면 다른 쪽 사용자는
바인딩을 쓰기 위해 앱 구조를 바꿔야 한다 — 재포장 도구가 요구할 일이 아니다.

### 브라우저 번들에 `node:child_process` 가 딸려 온다

`@rhwp/node` 를 브라우저 번들에 넣었다. 브라우저에서는 `@rhwp/node/browser` 만 들여온다.

---

## 타입

### `get()` 의 결과가 `unknown` 이라 못 쓰겠다

`get()` 은 키 이름을 문자열로 받으므로 정적으로 타입을 좁힐 수 없다. 생성된 봉투 타입을
쓰거나, 사용 지점에서 좁힌다.

```ts
const matches = hits.get('matches') as Array<{ page: number; context: string }>;
```

**왜 `any` 로 열어 두지 않나**: `any` 는 그 값에서 파생된 모든 타입 검사를 무력화한다.
`unknown` 은 한 번 좁히도록 강제하고, 그 지점이 곧 "여기서 계약을 가정했다"는 표시가 된다.

### 생성 타입이 스키마와 어긋난다

```bash
npm run gen:check     # 검사 — 다르면 exit 1
npm run gen:types     # 재생성
```

`src/ir.ts`·`src/envelopes.ts` 는 생성기 산출물이다. **손으로 고치면 다음 재생성에서
사라진다.** IR 이 바뀌었는데 다시 만들지 않은 PR 을 CI 가 잡는다.

### 새 rhwp 명령을 바인딩이 모른다

계약 패리티 가드가 CI 에서 잡지만, 로컬에서는 저수준 API 로 우회할 수 있다.

```ts
const envelope = await runJson(['새명령', '문서.hwp', '--json']);
```

그다음 `src/commands.ts` 에 래퍼를 추가하는 PR 을 열어 달라.

---

## 인코딩

### 한글이 깨진다

바인딩은 stdout·stderr 를 **UTF-8** 로 읽는다. 실물 rhwp(Rust)는 콘솔 코드페이지와
무관하게 항상 UTF-8 을 주고받으므로 정상 경로에서는 문제가 없다.

깨진다면 **rhwp 가 아닌 것**이 실행되고 있을 가능성이 높다.

```ts
console.log(findBinary());   // 무엇이 실행되는지 확인
```

### 윈도우에서만 테스트가 깨진다

거의 항상 **가짜 픽스처**가 원인이다. 테스트용 가짜 바이너리는 플랫폼 기본 인코딩을
따르므로, 윈도우(cp949)에서만 한글이 깨져 "바인딩 버그"로 오인된다.

픽스처가 stdout·stdin 을 **UTF-8 로 명시**해야 한다.

```ts
process.stdout.write(Buffer.from(JSON.stringify(envelope), 'utf8'));
```

파이썬 픽스처라면 이렇게 한다.

```python
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", newline="\n")
sys.stdin = io.TextIOWrapper(sys.stdin.buffer, encoding="utf-8")
```

**왜 이 함정이 반복되나**: 실물과 가짜의 인코딩 규약이 다른데, 그 차이가 한글이 섞인
경로에서만 드러나기 때문이다. 파이썬 바인딩(M18)에서 같은 원인으로 테스트가 두 번
실패했고, 원인은 바인딩이 아니라 픽스처였다.

---

## 성능

### 같은 문서에 여러 번 호출하니 느리다

1층은 호출마다 프로세스를 띄우고 문서를 다시 파싱한다. **2층 세션**을 쓴다.

```ts
const doc = await openDocument('큰문서.hwp');   // 한 번만 파싱
try {
  await doc.info(); await doc.fields(); await doc.tables();
} finally { await doc.close(); }
```

같은 문서에 3회 이상 접근하면 세션이 유리하다.

### `RhwpTimeoutError: 제한 시간을 초과했습니다`

대형 문서 렌더·변환이 오래 걸린다(기본 300초).

```ts
await exportPdf('큰문서.hwp', { out: 'a.pdf', timeoutMs: 1_800_000 });
await exportPdf('큰문서.hwp', { out: 'a.pdf', timeoutMs: null });   // 무제한
```

**왜 기본이 무제한이 아닌가**: 서버 안에서 도는 코드가 무제한 대기하면 그것이 곧 서비스
거부 경로다. 무제한은 사람이 지켜보는 배치 작업에서 **명시적으로** 고르는 값이다.

### 대량 처리에서 메모리가 늘어난다

`batch` 는 전 레코드를 배열로 모은다. 스트리밍이 필요하면 `iterNdjson` 을 쓴다.

```ts
for await (const record of iterNdjson(['batch', 'info', '--json'], { stdin: pathsText })) {
  handle(record);
}
```

### `Promise.all` 로 던졌더니 시스템이 멈춘다

호출 하나 = 프로세스 하나다. 수백 개를 동시에 띄우면 메모리와 파일 핸들이 고갈된다.
`batch`(프로세스 하나가 목록 전체 처리)를 쓰거나 동시성 상한을 건다.

---

## 그래도 안 되면

이슈를 열 때 아래를 함께 붙여 달라 — 재현이 절반이다.

```ts
import { findBinary, capabilities } from '@rhwp/node';

console.log('노드:', process.version, process.platform, process.arch);
console.log('바이너리:', findBinary());
console.log('rhwp:', (await capabilities()).get('version'));
```

예외가 났다면 `e.command` 가 그대로 재현 명령이다.
