---
kind: reference
status: active
canonical: mydocs/tech/bindings/parity_contract.md
last_verified: 2026-08-03
---

# 파이썬(M18)·Node(M19) 바인딩 실측 대조

두 바인딩이 **실제로** 어디가 다른지, 그 차이가 **의도인지 표류인지** 판정한 표다.
계약 자체는 [`parity_contract.md`](parity_contract.md) 가 권위이고, 이 문서는 그 계약에
비춘 현행 스냅샷이다.

**표류를 숨기지 않는 것이 이 문서의 목적이다.** 아래 20건 중 12건은 파이썬 쪽이
뒤처진 것이고, 그중 2건은 **기능이 아예 동작하지 않는다**(D-1) 또는 **안전 장치가
없다**(D-4).

## 0. 실측 조건

| 항목 | 값 |
|---|---|
| 바이너리 | `target/release/rhwp.exe`, `rhwp v0.8.2`, 봉투 `schemaVersion 1.0` |
| 파이썬 | `bindings/python/src` 를 `PYTHONPATH` 로 임포트, `RHWP_BIN` 지정, **실행 검증함** |
| Node | **소스 정적 대조만** — 이 PC 에 `node_modules` 가 없어 `vitest`·`tsc` 미실행 |
| 표본 | `samples/2010-01-06.hwp` |
| 일자 | 2026-08-03 |

Node 쪽 "이렇게 동작한다"는 서술은 전부 코드 경로 인용이다. 실행으로 확인한 항목은
그 자리에 **실측**이라고 적었다.

## 1. 규모

| | 파이썬 | Node |
|---|---|---|
| 소스 | `src/rhwp` 11파일 3,148줄 | `src` 14파일 5,377줄 |
| 테스트 | 15파일 (`conftest.py` 포함) | 20파일 (`helpers/` 2 포함) |
| 예제 | 9개 (`01`~`09`) | 12개 (`01`~`12`, 브라우저·렌더diff·타입봉투 추가) |
| 패키지 문서 | `docs/` 5개 1,553줄 | `docs/` 5개 2,940줄 |
| 1층 명령 래퍼 | 28개 | 31개 |

Node 가 큰 이유의 대부분은 **생성 타입 두 벌**이다(`ir.ts` 875줄, `envelopes.ts` 658줄 —
`tools/gen-types.ts` 산출물). 이건 의도된 차이다(§3-1).

---

## 2. 같은 것 — 계약의 뼈대는 실제로 공유된다

대조하기 전에 무엇이 이미 같은지 확인해 둔다. 아래는 두 구현이 **같은 문장·같은 순서**로
되어 있는 자리다.

| 항목 | 파이썬 | Node |
|---|---|---|
| 바이너리 탐색 순서 `RHWP_BIN` → 동봉 → PATH | `_binary.py:113-137` | `binary.ts:132-162` |
| `RHWP_BIN` 이 잘못되면 조용히 넘기지 않고 즉시 예외 | `_binary.py:77-80` | `binary.ts:97-100` |
| 탐색 결과 프로세스 수명 캐시 + 무효화 함수 | `_binary.py:33,41` | `binary.ts:26,38` |
| 종료 코드 상수 5개 | `errors.py:39-48` | `errors.ts:19-27` |
| exit 3/4 = 값, opt-in 시 예외 | `errors.py:200-209` | `errors.ts:235-245` |
| 사전에 없는 코드는 **조용히 통과시키지 않는다** | `errors.py:211-216` | `errors.ts:247-252` |
| 봉투 파싱을 종료 코드 검사보다 먼저 | `_process.py:162-194` | `process.ts:195-226` |
| 성공인데 stdout 이 비면 `ProtocolError` | `_process.py:196-203` | `process.ts:228-235` |
| batch: exit 1 은 예외 아님, exit 2 는 예외 | `_process.py:246-248` | `process.ts:296-299` |
| 계획 선검증 위반(`invalid`)은 예외 아니라 값 | `plan.py:205-226` | `plan.ts:290-302` |
| 원문 봉투 보존 + snake/camel 3중 조회 | `models.py:38-61` | `envelope.ts:81-138` |
| 없는 필드는 조용한 `None`/`undefined` 아니라 예외 | `models.py:59-61` | `envelope.ts:135-137` |
| `verify` 미요청은 `None`/`null`, 실패와 구분 | `models.py:109-117` | `envelope.ts:182-187` |
| `changedPages`: `None`(모름) ≠ `[]`(없음) | `models.py:119-130` | `envelope.ts:195-199` |
| 기본 제한 시간 300초 | `_process.py:27` | `process.ts:31` (300,000ms) |
| 불리언을 인자 값 위치에 넣으면 `TypeError` | `_process.py:61-66` | `process.ts:62-67` |
| 셸을 태우지 않는다 | `subprocess.run(argv)` | `spawn(…, {shell:false})` |
| 스트리밍 중단 시 자식 정리 | `_process.py:291-295` | `process.ts:353-358` |

**뼈대는 공유돼 있다.** 아래 표류는 뼈대가 아니라 표면과 세부에서 났다.

---

## 3. 의도된 차이 (표류 아님)

### 3-1. 타입 생성 유무

Node 는 `capabilities.recordFields` 와 `export-ir-schema` **두 출처**에서 타입을 생성한다
(`tools/gen-types.ts` → `src/envelopes.ts`·`src/ir.ts`). 파이썬은 동적 `Envelope` 하나로
끝낸다.

근거가 문서에 있다(`bindings/node/docs/DESIGN.md` D3): 파이썬의 `Envelope` 는 봉투에 있는
것을 전부 노출하므로 **구조적으로 뒤처질 수 없고**, 동적 언어에서는 그것으로 충분했다.
TypeScript 는 다르다 — 사용자가 그 언어를 고른 이유의 상당 부분이 "필드 이름을 컴파일러가
확인해 준다"인데, `Record<string, unknown>` 만 주면 그 값어치를 통째로 버린다.

**판정: 의도.** M20 의 C#·Swift 는 정적 타입 계열이므로 Node 쪽 패턴을 승계한다.

### 3-2. 동기 vs 비동기 · 브라우저 어댑터 · 자원 정리 문법

- 파이썬은 동기(`subprocess.run`), Node 는 전부 `Promise`. `parity_contract.md` §1 이 허용.
- Node 에만 `browser.ts`(249줄) — `createBrowserClient`/`createNodeClient` 로 같은
  인터페이스 아래 WASM 경로를 둔다. 파이썬에는 대응물이 있을 이유가 없다(M19 범위).
- 파이썬 `with`(`session.py:238-247`), Node `close()` + `Symbol.asyncDispose`
  (`session.ts:288`, `:409`). 폴리필을 넣지 않는 이유는 `DESIGN.md` D12.

**판정: 셋 다 의도.**

### 3-3. `inspect` 의 옵션 검증 방식

파이썬은 런타임에 잘못된 조합을 `ValueError` 로 거부한다(`commands.py:191-212`).
Node 는 오버로드 3개로 컴파일 시점에 가른다(`document-analysis.ts:167-184`).

**판정: 절반 의도, 절반 표류.** 정적 검사로 옮긴 것은 타당하나, JS(타입 없는) 호출자가
`inspect('injection', p, {thresholdPt: 9})` 를 넘기면 **조용히 무시된다** — 파이썬은
거부한다. 인자 순서 문제는 별건이다(D-11).

---

## 4. 표류 — 파이썬이 뒤처진 것

### D-1. `convert(out=)`·`export_hwpx(out=)` 가 항상 실패한다 — **치명**

파이썬은 산출 경로를 `-o` 플래그로 붙인다.

```
commands.py:344   _flag(args, "-o", out)        # export_hwpx
commands.py:369   _flag(args, "-o", out)        # convert
```

CLI 는 `-o` 를 모른다. 산출 경로가 **위치 인자**다.

```
$ rhwp convert samples/2010-01-06.hwp -o out.hwp --json
알 수 없는 옵션: -o
사용법: rhwp convert <입력.hwp|입력.hwpx> <출력.hwp> [--verify] [--verify-pages] [--json]
exit=2

$ rhwp export-hwpx samples/2010-01-06.hwp -o out.hwpx --json
알 수 없는 옵션: -o
exit=2
```

바인딩을 직접 실행해 확인했다(**실측**):

```
rhwp.export_hwpx(S, out=…)  → UsageError: 호출 인자가 올바르지 않습니다 (exit 2)
rhwp.convert(S,     out=…)  → UsageError: 호출 인자가 올바르지 않습니다 (exit 2)
```

Node 는 위치 인자로 넘기고, **그 사실을 주석에 실측으로 적어 뒀다**
(`commands.ts:534-536`: "이 명령은 `-o` 를 모른다(\"알 수 없는 옵션: -o\", exit 2)").
게다가 `convert` 는 산출 경로가 필수라는 것까지 반영해, 프로세스를 띄우기 전에 같은
판정을 내린다(`commands.ts:559-566`).

**판정: 표류.** 파이썬 쪽 변환 API 두 개가 산출 경로를 지정하는 순간 죽어 있다.
`parity_contract.md` §6.1 A-3(옵션 → 플래그 대조)이 있었으면 머지 전에 잡혔다.

### D-2. `render_diff` 없음

`hasattr(rhwp, "render_diff") == False` (**실측**). Node 는 `commands.ts:657` 에 있다.

`render-diff` 는 `--json` 모드에서 시각 회귀를 **exit 3** 으로 낸다(실측:
`capabilities.exitCodes["3"]` 에 "render-diff --json 시각 회귀 검출(사람 모드는 종전대로 1)").
즉 이 축의 판정 규약이 적용되는 명령인데, 파이썬 사용자는 그 판정에 닿을 방법이 없다.

`capabilities` 대조를 실행해 파이썬 미노출 `json` 명령을 뽑으면 정확히 셋이다(**실측**):
`export-capabilities-schema`, `export-ir-schema`, `render-diff`.
앞의 둘은 `schema.py` 에 다른 이름의 대체물이 있고(D-3), `render-diff` 는 대체물이 없다.

**판정: 표류.**

### D-3. 스키마 명령을 `--json` 없이 부른다

```
schema.py:239   run_json(["export-ir-schema"], …)
schema.py:249   run_json(["export-capabilities-schema"], …)
schema.py:277   run_json(["export-ir-schema"], …)
schema.py:282   run_json(["export-capabilities-schema"], …)
```

파이썬의 다른 27개 래퍼는 전부 `args.append("--json")` 을 한다. 이 넷만 빠져 있다.

오늘은 무해하다(**실측**): `export-ir-schema` 와 `export-ir-schema --json` 의 stdout 이
**44,119바이트로 동일**하고, 최상위 키도 같다(`definitionCount`, `dialect`,
`irSchemaVersion`, `schema`, `schemaVersion`).

하지만 그건 우연이다. `--json` 이 붙지 않은 호출은 "사람 모드"이고, `export-tables -o`
처럼 사람 모드에서 출력이 바뀐 전례가 있다(§5-1). Node 는 `--json` 을 붙인다
(`commands.ts:277`, `:299`).

부수 차이: Node 는 `--bare`(스키마 본문만)와 `-o`(파일 저장, 봉투 유지)를 옵션으로 연다.
파이썬은 둘 다 없다.

**판정: 표류(잠복).** `parity_contract.md` §6.2 B층 골든 비교가 있으면 출력이 갈라지는
순간 즉시 잡힌다.

### D-4. `Plan.check()` 에 `--dry-run` 지원 게이트가 없다 — **안전**

Node:

```
plan.ts:223-227   async check(...) { await assertDryRunSupported(options); … }
plan.ts:248-275   capabilities 의 run 명령 flags 에 '--dry-run' 이 없으면 던진다:
   "check() 를 실행으로 대체하지 않습니다 — 검사인 줄 알고 문서가 편집되면 안 됩니다."
```

파이썬:

```
plan.py:188-194   def check(...): return _execute(self.to_dict(dry_run=True), …)
```

게이트가 없다. 계획서에 `dryRun: True` 필드를 실어 보낼 뿐이다(`plan.py:186-187`).
#3759 이전 바이너리가 그 필드를 무시하면 **검사인 줄 알았던 호출이 실제 편집을 수행한다.**

**판정: 표류.** Node 의 설계 기록이 이 위험을 명시적으로 다뤘고(`DESIGN.md` D13),
파이썬은 그 이전에 만들어져 반영되지 않았다.
`parity_contract.md` §4.4-2 가 이걸 계약으로 못 박는다.

### D-5. `_quote` 가 역슬래시를 이스케이프하지 않는다

**실측** — 입력 `C:\my dir\` (끝에 역슬래시 1개):

```
파이썬 errors.py:160-165  →  "C:\my dir\"      ← 끝 역슬래시가 닫는 따옴표를 먹는다
Node   errors.ts:197-201  →  "C:\\my dir\\"
```

Node 주석이 이유를 정확히 적어 뒀다: 붙여넣으면 명령이 깨지고, 최악의 경우 다음 인자와
뭉쳐 **다른 명령**이 된다. 파이썬의 조건식은 트리거 문자 집합에 역슬래시가 아예 없다
(`if arg and not any(ch.isspace() or ch in "\"'" for ch in arg)`) — Node 는 `[\s"'\\]`.

영향 범위는 `RhwpError.command` 뿐이다(버그 리포트용 재현 문자열). 실행에는 쓰이지 않는다.

**판정: 표류.** Node 가 나중에 고친 자리를 파이썬이 그대로 갖고 있다.

### D-6. 약어 정규식이 ReDoS 가능 형태

```
_naming.py:18   ([A-Z]+)([A-Z][a-z])      ← 가변 길이
naming.ts:22    ([A-Z])([A-Z][a-z])       ← 고정 길이
```

Node 주석(`naming.ts:16-21`)이 이유와 등가성을 함께 적었다: 앞 그룹이 가변이면
`AAAA…Aa` 에서 역추적이 다항으로 늘어난다. 결과는 같다 — 경계는 "대문자 하나 뒤에
대문자+소문자"라는 국소 조건이기 때문이다.

**실측 등가 확인**: `to_snake("HTMLPage")` → `html_page` (파이썬), 고정 길이 규칙도 동일.

노출도는 낮다(봉투 키는 도구가 만든다). 그러나 `Envelope` 는 **문서에서 온 키**도 색인에
넣는다(`models.py:41-44` 는 최상위 키 전부를 돈다) — 누름틀 이름 같은 사용자 데이터가
키가 되는 경로가 있으므로 완전히 무관하지는 않다.

**판정: 표류.**

### D-7. `TimeoutError` vs `RhwpTimeoutError`

파이썬은 내장 `TimeoutError` 를 가린다(`errors.py:156`, `noqa: A001 - 내장 이름 가림은
의도적(패키지 일관성)`). Node 는 `RhwpTimeoutError` (`errors.ts:187`).

한쪽이 의도라고 주석에 적었으므로 "실수"는 아니다. 그러나 두 바인딩 사이에서 **같은
개념의 이름이 다르다**는 사실은 남는다. `parity_contract.md` §3.3 이 접두형으로 통일하기로
결정했다 — 내장을 가리면 `except TimeoutError` 한 줄이 어느 쪽을 잡는지 임포트를 봐야
알 수 있고, 그건 오류 처리 코드가 가장 피해야 할 성질이다.

**판정: 표류(계약이 한쪽을 골라야 하는 자리).**

### D-8. `UsageError.next_call` 없음

Node `errors.ts:145-151` 는 봉투의 `nextCall`(서버가 실어 보낸 교정 호출 — 기계가 그대로
따라할 수 있는 형태)을 getter 로 꺼낸다. 파이썬에는 없다.

파이썬은 `suggestion`(stderr 의 `힌트:` 줄)만 있고, Node 는 둘 다 있다.

**판정: 표류.** 에이전트가 교정 루프를 도는 축(#3828 계열)에서 파이썬 쪽만 한 단계
떨어진다.

### D-11. `inspect` 인자 순서가 반대

```
파이썬 commands.py:175   inspect(path, subcommand, *, …)
Node   document-analysis.ts:185   inspect(target, path, options)
```

**같은 이름의 함수가 첫 인자로 다른 것을 받는다.** 두 바인딩을 함께 쓰는 코드베이스
(예: 파이썬 파이프라인 + Node CLI 도구)에서 이건 즉시 사고다. 게다가 둘 다 문자열이라
타입이 걸러 주지도 않는다.

Node 쪽이 CLI 순서(`rhwp inspect <검사> <파일>`)와 일치한다.

**판정: 표류.** 어느 쪽으로 통일할지는 `parity_contract.md` 가 정하지 않았다 —
CLI 순서를 따르는 Node 쪽이 자연스럽지만, 파이썬은 이미 배포 형태를 갖췄으므로
호환 유지 기간이 필요하다. **확인되지 않음: 파이썬 패키지가 PyPI 에 실제로 배포됐는지.**

### D-12. 명령 옵션 누락 4건

| 명령 | Node 옵션 | 파이썬 |
|---|---|---|
| `export-text` | `page` (`-p`) — `commands.ts:114-120` | **없음** (`commands.py:73`) |
| `export-structure` | `mode`(`auto`/`outline`/`clause`) — `commands.ts:141-159` | **없음** (`commands.py:78`) |
| `digest` | `maxChars` (`--max-chars`) — `commands.ts:229-232` | **없음** (`commands.py:134`) |
| `ir-diff` | `section`(`-s`), `paragraph`(`-p`) — `commands.ts:577-592` | **없음** (`commands.py:375`) |

셋 다 CLI 에서 동작하는 것을 확인했다(**실측**):

```
rhwp export-text  … -p 1 --json          → {"omittedCount":0,"pageCount":1,"pages":[…]}
rhwp export-structure … --mode clause --json → {"mode":"clause", …}
```

`export-structure --mode` 는 자동 판정이 기대와 다를 때(규정 문서를 개요로 읽었을 때)
되돌리는 유일한 수단이다. `digest --max-chars` 는 문맥 창이 좁은 모델에 넘길 때 필수다.
**옵션이 없으면 그 기능이 없는 것과 같다.**

**판정: 표류.** 원인은 D-19.

### D-13. `iter_ndjson` 이 공개 API 가 아니다

`hasattr(rhwp, "iter_ndjson") == False` (**실측**). 함수는 `_process.py:252` 에 있지만
`_process.__all__`(:23)에도 `__init__.py` 의 임포트 목록(:53)에도 없다.

패키지 문서는 이걸 쓰라고 안내한다 — `bindings/python/docs/TROUBLESHOOTING.md:311`:
`from rhwp._process import iter_ndjson`. **밑줄 모듈에서 임포트하라고 문서가 안내하는
상태**다. Node 는 `index.ts:133` 에서 `iterNdjson` 을 정식 수출한다.

**판정: 표류.**

### D-14. `Session` 옵션 비대칭

```
파이썬 session.py:52   Session(*, profile=None, timeout=300.0)      ← cwd 없음
Node   session.ts:36   SessionOptions { profile?, cwd? }            ← timeout 없음
```

세션은 장수명 프로세스라 **양쪽 다 필요하다.** 파이썬은 작업 디렉터리를 못 바꾸고,
Node 는 응답이 영원히 안 와도 안 끊긴다.

**판정: 양방향 표류(각자 하나씩 없다).**

### D-15. 바이너리 탐색 보조 API 비대칭

| | 파이썬 | Node |
|---|---|---|
| 캐시 비우기 | `clear_cache` (패키지 루트에 노출) | `clearBinaryCache` |
| 실행 파일 이름 | `binary_name()` — **루트 미노출** | `binaryName()` — 노출 |
| 동봉 위치 | `BUNDLED_DIR` 상수 — **루트 미노출** | `bundledDir()` 함수 — 노출 |
| `ENV_VAR` | 노출 | 노출 |

파이썬의 `clear_cache` 라는 이름은 패키지 루트에서 **무엇의** 캐시인지 말하지 않는다
(계획 캐시·스키마 캐시가 늘면 충돌한다). 또 `_binary.py:24` 의 `__all__` 에는 `ENV_VAR` 가
없는데 `__init__.py:51` 이 그것을 임포트한다 — 동작은 하지만 `__all__` 이 실제 표면과
어긋나 있다.

**판정: 표류(경미).**

### D-16. `RHWP_BIN` 의 `~` 확장

```
파이썬 _binary.py:70   Path(raw).expanduser()
Node   binary.ts:86    resolve(raw)                ← ~ 확장 없음
```

`RHWP_BIN=~/bin/rhwp` 가 파이썬에서는 되고 Node 에서는 안 된다. Node 는 `~` 로 시작하는
디렉터리를 실제로 찾다가 `BinaryNotFoundError` 를 낸다.

부수: 파이썬은 `shutil.which`(`_binary.py:91`)를 쓰므로 Windows 에서 `PATHEXT` 와
현재 디렉터리 규칙을 따르고, Node 는 `PATH` 항목을 직접 순회한다(`binary.ts:110-118`).
**확인되지 않음**: 이 차이가 실제로 다른 바이너리를 고르는 경로가 있는지 실측하지 않았다.

**판정: 표류(경미).**

### D-19. 파이썬에 "선언 → 래퍼" 패리티 테스트가 없다 — **구조적 원인**

Node `test/parity.integration.test.ts:76-111`:

> 선언된 `json` 명령마다 바인딩에 대응 함수가 있다.
> 수기 목록을 두지 않는다 — 모듈이 실제로 내보내는 이름이 곧 표면이다.
> `export-tables` → `exportTables` 변환도 **바인딩 자신의 `toCamel`** 을 쓴다.

파이썬 `tests/test_envelope_parity.py` 는 7개 테스트를 갖지만 **이 테스트가 없다.**
있는 것은 "선언한 필드가 봉투에 나오는가", "종료 코드 사전이 상수를 덮는가",
"MCP 도구가 실존 명령을 가리키는가" 등 — 전부 **도구 쪽 정합성**이고, **바인딩의 표면
완결성**을 보는 것은 하나도 없다.

**그래서 D-2 와 D-12 가 살아남았다.** 이 목록에서 가장 먼저 고칠 항목이다.

---

## 5. 표류 — Node 가 뒤처지거나 양쪽이 같이 빠진 것

### D-9. `VerifyReport` 의 진리값 의미가 다르다 — **높음**

```
파이썬 models.py:160-162   def __bool__(self): return self.identical
Node   envelope.ts:34-62   VerifyReport 에 진리값 훅 없음 (객체는 항상 truthy)
```

같아 보이는 코드가 반대 결론을 낸다.

```python
if result.verify:          # 파이썬: "검증을 통과했나"
```
```ts
if (saved.verify) {        // Node: "검증을 요청했나" — 실패해도 참
```

Node 문서는 이 함정을 알고 있어서 예제를 `if (!saved.verify?.identical)` 로 쓴다
(`index.ts:32`). 즉 **회피는 하지만 방지는 안 한다.**

파이썬 쪽 설계 의도 자체는 명시돼 있다(`models.py:161`: "`if result.verify:` 가
'통과했나'로 읽히도록"). 그러나 그 의도는 `verify` 가 `None` 일 때(미요청) 와 `identical`
이 거짓일 때가 **둘 다 거짓**이 된다는 뜻이기도 하다 — 두 바인딩이 공들여 구분한
"검증 안 함 ≠ 검증 실패"를 파이썬의 진리값이 다시 뭉갠다.

**판정: 양쪽 다 문제.** `parity_contract.md` §1 의 "결론을 바꾸는 차이" 에 해당한다.
가장 안전한 통일은 **어느 쪽도 진리값 훅을 두지 않고 `identical` 을 읽게 강제**하는 것이다.

### D-10. `verifyPages` 접근자가 양쪽에 없다

**실측** 봉투:

```json
"verify":      {"diffCount":0,"identical":true},
"verifyPages": {"after":6,"before":6,"identical":true}
```

두 바인딩 모두 `verify` 는 전용 클래스로 감싸지만(`models.py:110`, `envelope.ts:182`)
`verifyPages` 는 감싸지 않는다. 파이썬에서는 3중 조회 덕에
`r["verify_pages"]` 가 동작하고(**실측**: `{'after': 6, 'before': 6, 'identical': True}`),
Node 에서는 `env.child('verifyPages')` 로 닿는다.

exit 4 는 exit 3 과 **다른 판정**인데(페이지 수 불일치), 그 근거를 읽는 길이 일반 조회뿐이다.

**판정: 공통 결손.** 심각도는 낮으나 M20 이 정적 매핑을 만들 때 빠뜨리기 쉬운 자리다.

### D-17. `Envelope` 접근자 집합 비대칭

| | 파이썬 | Node |
|---|---|---|
| 매핑 프로토콜 | `Mapping` 구현 (`for k in env`, `len`, `.keys()`) | 없음 — `keys()` 만 |
| 중첩 자동 래핑 | 접근 시점에 자동 (`models.py:165-173`) | 없음 — `child()`/`children()` 명시 |
| 기본값 조회 | `get_path(dotted, default)` | `getPath(dotted)` (기본값 인자 없음), `getOr(key, fallback)` |
| 직렬화 | 없음 | `toJSON()` |
| 없는 키 예외 타입 | `KeyError` / `AttributeError` | 일반 `Error` — **`RhwpError` 계열 아님** |

마지막 줄이 눈에 띈다. Node 의 `Envelope.get` 은 `throw new Error(...)`
(`envelope.ts:135`)로 오류 분류 체계 밖의 예외를 던진다. `catch (e) { if (e instanceof
RhwpError) … }` 로 거르는 코드가 이걸 놓친다.

**판정: 앞의 세 줄은 언어 관례(허용), 마지막 줄은 표류.**

### D-18. `raise_for_exit` / `isKnownExitCode` 노출 비대칭

Node 는 `raiseForExit` 과 `isKnownExitCode` 를 `index.ts:94-95` 에서 수출한다.
파이썬은 `errors.py:35` 의 `__all__` 에 `raise_for_exit` 을 넣었지만 `__init__.py` 가
임포트하지 않아 **`rhwp.raise_for_exit` 이 존재하지 않는다**(`hasattr` 실측: `False`).
`isKnownExitCode` 대응물은 아예 없다.

저수준 `run_raw(check=False)` 를 쓰는 사용자는 종료 코드 처리를 손으로 다시 써야 한다.

**판정: 표류(경미).**

### D-20. Node `runRaw` 가 예외에 봉투를 싣지 않는다

파이썬 `_process.py:78,130` 은 `envelope_hint` 를 받아 `raise_for_exit` 에 넘긴다.
Node `process.ts:172-178` 의 `runRaw` 는 `envelope` 를 넘기지 않는다(`runJson` 은 넘긴다,
`process.ts:224`).

`runRaw` 를 직접 쓰면서 봉투를 미리 갖고 있는 경로에서 판정 근거가 예외에서 빠진다.
실사용 빈도는 낮다.

**판정: 표류(경미).**

---

## 6. 요약 — 어느 쪽이 앞서 있나

| 영역 | 앞선 쪽 | 왜 |
|---|---|---|
| 명령 표면 완결성 | **Node** | 패리티 테스트가 강제 (D-2·D-12·D-19) |
| CLI 실제 동작 반영 | **Node** | `-o` 3종을 실측으로 가름 (D-1, D-3) |
| 안전 게이트 | **Node** | `--dry-run` capabilities 확인 (D-4) |
| 문자열 처리 견고성 | **Node** | 인용 이스케이프·ReDoS (D-5, D-6) |
| 오류 진단 정보 | **Node** | `nextCall` (D-8) |
| 판정 읽기의 안전성 | **Node** | 진리값 함정을 만들지 않음 (D-9) |
| 저수준 API 접근성 | **Node** | `iterNdjson`·`raiseForExit` 수출 (D-13, D-18) |
| 오류 계열의 일관성 | **파이썬** | 봉투 조회 실패도 표준 예외 계열 (D-17) |
| 봉투 조회 편의 | **파이썬** | `Mapping` + 자동 래핑 + `get_path(default)` |
| 환경변수 관용도 | **파이썬** | `~` 확장 (D-16) |

**Node 가 앞선 자리의 거의 전부가 "파이썬을 만든 뒤에 알게 된 것"이다.** M19 가 M18 을
읽고 만들어졌으니 자연스러운 결과이고, 문제는 그 학습이 **역류하지 않았다**는 것이다.
바인딩이 셋·넷이 되면 이 역류 비용이 언어 수만큼 곱해진다. 그래서
[`parity_contract.md`](parity_contract.md) §6 이 필요하다.

## 7. 관련 문서

- [`parity_contract.md`](parity_contract.md) — 이 표가 비추는 계약
- [`new_binding_guide.md`](new_binding_guide.md) — 새 언어가 두 구현에서 뽑아야 할 단계
- [`README.md`](README.md) — 이 디렉터리의 지도
- [`bindings_foundation.md`](../bindings_foundation.md) — 설계 전제
- [`python_binding_guide.md`](../../manual/python_binding_guide.md) ·
  [`node_binding_guide.md`](../../manual/node_binding_guide.md)
- `bindings/python/docs/DESIGN.md` · `bindings/node/docs/DESIGN.md` — 버린 대안 기록
- 로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M18~M20
