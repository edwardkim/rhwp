# 이주 가이드 — 기존 JS/TS HWP 도구·COM 자동화에서 옮겨오기

기존 방식과 `@rhwp/node` 는 **접근 방식이 다르다.** API 이름을 하나씩 대응시키는 대신,
무엇이 왜 다른지부터 정리한다. 그래야 옮긴 코드가 제대로 돈다.

---

## 1. 근본 차이 셋

### ① COM 자동화가 아니다

한글 프로그램을 띄워 조작하는 방식(윈도우 COM/OLE 자동화)은 **한컴 오피스가 설치된
윈도우**에서만 돈다. `rhwp` 는 문서 포맷을 직접 파싱·직렬화하므로 리눅스·macOS·컨테이너에서
돌고, 한글 설치가 필요 없다.

| | COM 자동화 | rhwp |
|---|---|---|
| 플랫폼 | 윈도우 + 한글 설치 | 어디서나 (+ 브라우저는 WASM) |
| 서버 배포 | 라이선스·GUI 세션 필요 | 바이너리 하나 |
| 실패 양상 | GUI 대화상자로 멈춤 | 종료 코드 + 봉투 |
| 병렬 처리 | 인스턴스 충돌 | 프로세스만큼 |
| CI 에서 | 사실상 불가 | 컨테이너에서 그대로 |

### ② 읽기 전용이 아니다

평문만 뽑는 라이브러리와 달리 **편집·저장·렌더**까지 한다. 다만 그만큼 "저장본이 진짜
맞나"가 중요해지므로 `verify` 규약이 있다.

### ③ 판정이 데이터다

예외를 던지는 대신 봉투에 판정을 담는다. 자세한 것은 [§5](#5-예외-처리-바꾸기).

---

## 2. 비동기와 타입

가장 먼저 부딪히는 두 가지다.

### 전부 비동기다

동기 API 로 문자열을 뽑던 코드는 함수 시그니처부터 바뀐다.

```ts
// 기존 감각
const text = extractTextSync('문서.hwp');

// rhwp — 프로세스를 띄우므로 동기일 수 없다
const result = await exportText('문서.hwp');
```

동기 함수는 `findBinary()`·`toSnake()`·`Plan` 빌더 체이닝뿐이다.
`Plan` 은 **`check()`/`run()` 에서만** 비동기가 된다 — 빌더는 문법 검사만 하기 때문이다.

### 봉투 값은 `unknown` 에서 출발한다

```ts
const meta = await info('문서.hwp');
meta.get('pageCount');                       // unknown
Number(meta.get('pageCount'));               // 좁혀서 쓴다
```

생성된 봉투 타입을 쓰면 정적으로 잡힌다([API §생성 타입](API.md#생성-타입)).
`any` 로 열어 두지 않는 이유는, `any` 가 그 값에서 파생된 모든 검사를 무력화하기 때문이다.

---

## 3. 문서 읽기

### 평문 추출

```ts
// 기존: 문서 전체를 한 문자열로
const text = extractText('문서.hwp');

// rhwp: 쪽별로 나뉘어 온다 — 주소가 보존된다
const result = await exportText('문서.hwp');
const pages = result.get('pages') as Array<{ text: string }>;
const whole = pages.map((p) => p.text).join('\n');   // 통합하고 싶으면
const first = pages[0].text;                          // 쪽 단위가 필요하면
```

**왜 쪽별인가**: RAG·인용 검증에서 "몇 쪽"에 답하려면 주소가 필요하다. 통합은 호출자가
언제든 할 수 있지만, 잃어버린 주소는 복구할 수 없다.

### 문서 정보

```ts
const meta = await info('문서.hwp');
meta.get('pageCount');
meta.get('format');      // 'hwp5' | 'hwpx' | 'hwp3' | 'hml'
meta.get('sections');
meta.get('fonts');
```

### 표 읽기

```ts
const tables = (await exportTables('문서.hwp')).get('tables') as Table[];
for (const t of tables) for (const c of t.cells) console.log(t.index, c.row, c.col, c.text);
```

**병합 셀 주의**: 병합된 셀은 **좌상단(앵커) 좌표 하나로만** 나온다. 덮인 좌표는 목록에
없다. 격자를 만들 때 빈 칸을 기본값으로 채워야 한다
([요리책 §4](COOKBOOK.md#4-표를-데이터셋으로)).

---

## 4. 문서 편집

### 누름틀 채우기

```ts
// COM 방식은 대략 이런 흐름이었다
// hwp.PutFieldText('성명', '홍길동');
// hwp.Save();

// rhwp — 한 번에, 검증까지
const result = await fillFields('서식.hwp', { 성명: '홍길동' }, { out: '제출본.hwp', verify: true });
if (!result.verify?.identical) throw new Error('저장본 불일치');
```

동명 누름틀은 `[순번]`(0 기준)으로 지정한다: `{ '성명[0]': '갑', '성명[1]': '을' }`.

### 여러 편집을 한 번에

COM 은 문서를 열어 두고 하나씩 조작한 뒤 저장했다. `rhwp` 에는 두 가지 대응이 있다.

**세션(2층)** — COM 의 사용 감각에 가깝다.

```ts
const doc = await openDocument('서식.hwp');
try {
  await doc.fillFields({ 성명: '홍길동' });
  await doc.replaceText('2025년', '2026년');
  await doc.save('제출본.hwp', { verify: true });
} finally {
  await doc.close();
}
```

편집 도구는 디스크에 쓰지 않는다 — `save()` 가 유일한 기록 지점이다. COM 의 "열고 고치고
저장"과 같은 모양이되, **어디서 디스크가 바뀌는지가 코드에 명시된다.**

**계획(3층)** — 더 안전하다. 하나라도 불가능하면 **아무것도 저장하지 않는다.**

```ts
const plan = new Plan('서식.hwp', '제출본.hwp')
  .fillFields({ 성명: '홍길동' })
  .replaceText('2025년', '2026년')
  .verify();

if ((await plan.check()).ok) await plan.run();   // check() 는 디스크 무변경
```

COM 에서 "중간에 실패해 반쯤 편집된 문서가 남는" 문제를 겪었다면 3층이 답이다.

### 자원 정리 — COM 의 `Quit()` 에 해당하는 것

COM 코드에서 한글 인스턴스를 안 닫아 프로세스가 쌓이던 문제는 그대로 재현될 수 있다.
세션은 자식 프로세스를 띄우기 때문이다.

```ts
try { /* ... */ } finally { await doc.close(); }
```

`close()` 는 멱등이고, 런타임이 지원하면 `await using` 으로 줄일 수 있다.

---

## 5. 예외 처리 바꾸기

가장 흔한 이주 실수다.

```ts
// 기존 감각 — 모든 실패가 예외
try {
  await someLegacyLib.convert('a.hwp', 'b.hwpx');
} catch {
  handleFailure();
}
```

> 위 `convert` 는 **기존 도구**의 것이다. `@rhwp/node` 의 `convert` 는 시그니처가
> 다르다 — `convert(path, { out })` 이고 `out` 은 **필수**다(출력 기본값이 없다).

`rhwp` 는 **고장과 판정을 가른다.**

```ts
let result;
try {
  result = await exportHwpx('a.hwp', { out: 'b.hwpx', verify: true });
} catch (e) {
  if (e instanceof RhwpRuntimeError) { /* 못 읽었다·못 썼다 — 진짜 고장 */ }
  else if (e instanceof UsageError) { /* 인자가 틀렸다 — 우리 코드 버그 */ }
  throw e;
}

// 변환은 됐는데 내용이 달라졌다 → 예외가 아니라 판정
if (!result.verify?.identical) console.warn(`차이 ${result.verify?.diffCount}건`);
```

**왜**: 검증 실패를 예외로 만들면 호출자가 "고장"으로 다루고, 정작 봉투에 담긴 판정
근거를 읽지 않는다. 예외가 편하면 `throwOnVerdict: true` 로 명시한다.

| 기존 | rhwp |
|---|---|
| `catch (e)` 전부 | `e instanceof RhwpError` (모든 rhwp 예외의 기반) |
| 파일 없음·손상 | `RhwpRuntimeError` |
| 잘못된 인자 | `UsageError` (`.suggestion`·`.nextCall` 에 교정 힌트) |
| GUI 대화상자로 멈춤 | 해당 없음 — 종료 코드로 즉시 끝난다 |
| 변환 손실 | **예외 아님** — `result.verify.identical` |

---

## 6. 대량 처리

```ts
// 기존: 루프 — 실패 하나가 루프를 멈춘다
for (const path of paths) handle(extractText(path));

// rhwp: 배치 — 부분 실패를 잃지 않는다
for (const record of await batch('export-text', paths)) {
  if ('error' in record) { logFailure(record.source, record.error); continue; }
  handle(record.pages);
}
```

**부분 실패도 실패지만 성공분은 남는다.** 수백 건 중 하나가 손상됐다고 나머지를 버릴
이유가 없다.

`Promise.all(paths.map(info))` 로 바꾸고 싶은 유혹을 참자 — 호출 하나가 프로세스 하나다.

---

## 7. 없는 기능·다른 기능

### `rhwp` 에 없는 것

| 기존 기능 | 대안 |
|---|---|
| 한글 GUI 조작(인쇄 대화상자 등) | 없음 — 포맷 수준 작업만 |
| 매크로 실행 | 없음 |
| 실시간 편집 UI | `rhwp-studio`·`@rhwp/editor`(별도) |

### `rhwp` 에만 있는 것

| 기능 | 설명 |
|---|---|
| `verify` | 저장본이 의도한 문서인지 자기검증 |
| `changedPages` | 편집이 바꾼 쪽 지정 → 그 쪽만 렌더해 확인 |
| `Plan` | 원자적 다단 편집 (전부 아니면 전무) |
| `irDiff` | 두 문서의 구조 차이를 범주별로 |
| `digest` | 주소를 보존한 RAG 청킹 |
| `exportIrSchema` | 문서 모델의 기계 판독 스키마 |
| MCP 서버 | 에이전트가 바로 붙는 도구 표면 |
| 브라우저 경로 | 같은 인터페이스의 WASM 어댑터 |

---

## 8. 이주 점검표

- [ ] 동기 API 를 **`await`** 로 바꿨는가 (그리고 호출자 시그니처까지 전파했는가)
- [ ] 평문 추출이 **쪽별**로 온다는 것을 반영했는가
- [ ] 표의 **병합 셀**이 앵커 좌표로만 나온다는 것을 처리했는가
- [ ] `extractPages` 만 **1 기준**, 나머지 `page` 는 **0 기준**임을 확인했는가
- [ ] `catch` 를 **고장/판정**으로 갈랐는가 (`instanceof`, 메시지 문자열 아님)
- [ ] 저장에 `verify: true` 를 붙였는가
- [ ] `verify` 의 `null`(검증 안 함)과 실패를 구분했는가
- [ ] `changedPages` 의 `null`(모름)과 `[]`(없음)을 구분했는가
- [ ] 세션을 `try/finally` 로 감쌌는가 (안 그러면 프로세스가 남는다)
- [ ] 대량 처리에서 **부분 실패**를 버리지 않는가
- [ ] 동시 실행에 상한을 걸었는가 (`Promise.all` 로 전부 던지지 않았는가)
- [ ] 서버라면 `timeoutMs` 를 명시했는가
- [ ] 브라우저 번들에 Node 진입점을 넣지 않았는가 (`@rhwp/node/browser`)
- [ ] `tsconfig` 의 `moduleResolution` 이 `node16`/`bundler` 인가

---

## 9. 성능 감각

| 작업 | 대략 |
|---|---|
| 1층 호출 하나 | 프로세스 기동(수십 ms) + 문서 파싱 |
| 세션 열기 | 파싱 한 번, 이후 호출은 재파싱 없음 |
| 배치 | 프로세스 하나가 목록 전체를 처리 |

**같은 문서에 3회 이상 접근하면 세션이 유리하다.** 서로 다른 문서 수백 개는 배치가 유리하다.

```ts
// 느림 — 같은 문서를 세 번 파싱
await info(p); await fields(p); await exportTables(p);

// 빠름 — 한 번 파싱
const doc = await openDocument(p);
try { await doc.info(); await doc.fields(); await doc.tables(); } finally { await doc.close(); }
```

인프로세스(napi) 경로는 없다. 프로세스 기동 비용이 실제 병목으로 실증되기 전에는
ABI 매트릭스를 떠안지 않는다는 판단이다([`DESIGN.md`](DESIGN.md) D1).

---

## 10. 라이선스

`rhwp` 와 이 바인딩은 **MIT** 다. 상용 제품에 넣을 수 있고, 소스 공개 의무가 없다.

---

## 막히면

[문제 해결](TROUBLESHOOTING.md)에 증상별 처방이 있다. 그래도 안 되면 이슈를 열 때
아래를 붙여 달라.

```ts
console.log(process.version, process.platform, findBinary(), (await capabilities()).get('version'));
```
