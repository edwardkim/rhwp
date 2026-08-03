---
kind: canonical
status: active
canonical: mydocs/tech/bindings/parity_contract.md
last_verified: 2026-08-03
---

# 바인딩 동등성 계약 — 언어가 늘어도 답이 하나이기 위한 규약

로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M18~M20 의 바인딩 축은
"같은 문서를 물으면 언어와 무관하게 같은 답이 나온다"를 전제로 서 있다. 그런데 그
전제는 **아무도 강제하지 않으면 반드시 깨진다.** 실제로 깨져 있고, 이 문서를 쓰면서
확인한 표류가 §7 에 20건 있다.

이 문서는 바인딩 하나하나의 사용법이 아니라 **바인딩들 사이의 계약**을 고정한다.
설계 전제(왜 서브프로세스인가, IR 스키마 버저닝)는
[`bindings_foundation.md`](../bindings_foundation.md) 가 권위이고, 이 문서는 그 위에서
"둘 이상이 됐을 때 무엇을 같게 유지하는가"만 다룬다.

- 언어별 사용법: [`python_binding_guide.md`](../../manual/python_binding_guide.md),
  [`node_binding_guide.md`](../../manual/node_binding_guide.md)
- 새 언어를 추가하는 절차: [`new_binding_guide.md`](new_binding_guide.md)
- 현행 두 바인딩의 실측 차이 목록: [`python_node_comparison.md`](python_node_comparison.md)

## 0. 이 문서가 쓰는 "실측"의 뜻

이 문서의 모든 수치·동작 주장은 아래 환경에서 직접 실행해 얻었다. 근거를 대지 못하는
항목은 본문에 **확인되지 않음**으로 명시한다.

| 항목 | 값 |
|---|---|
| 바이너리 | `target/release/rhwp.exe`, `rhwp v0.8.2` |
| 봉투 `schemaVersion` | `1.0` |
| `capabilities` 명령 수 | 61개 (그중 `json:true` 31개) |
| 표본 문서 | `samples/2010-01-06.hwp` (hwp5, 6쪽, 87문단) |
| 파이썬 바인딩 | `bindings/python/src` 를 `PYTHONPATH` 로 직접 임포트, `RHWP_BIN` 지정 |
| 실행일 | 2026-08-03 |

Node 바인딩은 **소스 정적 대조만** 했다. 이 PC 에는 `bindings/node/node_modules` 가 없어
`vitest`·`tsc` 를 돌리지 못했다 — Node 쪽의 "실행하면 이렇게 된다"는 주장은 코드 경로
인용으로만 뒷받침되며, 그 사실을 해당 자리마다 적었다.

---

## 1. 계약의 범위 — 무엇이 "같아야" 하는가

바인딩은 새 표면이 아니라 **CLI `--json` 봉투와 `mcp-serve` 세션 도구의 재포장**이다
(`bindings_foundation.md` §2). 따라서 동등성은 "API 가 똑같이 생겼다"가 아니라
**같은 입력이 같은 봉투를 만들어 낸다**로 정의한다.

| 계층 | 동등해야 하는가 | 근거 |
|---|---|---|
| 봉투 원문(`.raw`) | **바이트 단위로 동일** | 같은 CLI 를 같은 인자로 부른다면 다를 이유가 없다 |
| 노출하는 명령 집합 | **동일** | 한쪽에만 있으면 그 언어 사용자는 "이 도구엔 그 기능이 없다"고 결론 내린다 |
| 명령별 옵션 집합 | **동일** | 옵션이 없으면 그 기능이 없는 것과 같다 |
| 종료 코드 → 오류 표현 | **의미가 동일** (이름은 언어 관례) | §3 |
| 접근자 이름 | 언어 관례를 따른다 | §2 |
| 비동기/동기 | 언어 관례를 따른다 | Node 는 Promise, 파이썬은 동기 — 강제하지 않는다 |
| 자원 정리 문법 | 언어 관례를 따른다 | `with` / `await using` / `IDisposable` |

**"언어 관례를 따른다"가 허용되는 자리와 아닌 자리를 가르는 기준은 하나다: 그 차이가
호출자의 *결론*을 바꾸는가.** `doc.fill_fields` 냐 `doc.fillFields` 냐는 결론을 바꾸지
않는다. `render_diff` 가 있느냐 없느냐는 바꾼다.

---

## 2. 봉투 필드 이름 — 결정과 근거

### 2.1 결정

> **봉투의 원문 키는 어떤 바인딩도 바꾸지 않는다. 언어 관례(snake_case 등)는
> *별칭 조회 계층*으로만 제공하고, 직렬화 가능한 원문(`.raw`)은 CLI 가 낸 그대로
> 보존한다.**

즉 "이름을 바꾼다 vs 그대로 둔다"의 답은 **둘 다**이되 층이 다르다.

- **저장 층**: 원문 그대로. `env.raw["pageCount"]` 는 언제나 존재한다.
- **조회 층**: 원문 키·snake_case·camelCase 를 모두 받는다. 없는 키는 조용히
  `None`/`undefined` 가 아니라 예외.

### 2.2 이 결정이 "이미 그렇게 되어 있다"는 근거

먼저 기존 둘이 실제로 무엇을 하는지 확인했다. 둘 다 원문을 보존한다.

| 동작 | 파이썬 | Node |
|---|---|---|
| 원문 보관 | `models.py:38` — `object.__setattr__(self, "_raw", dict(raw))` | `envelope.ts:81` — 생성자가 `source` 를 그대로 보관 |
| snake→원문 색인 | `models.py:41-44` — `index.setdefault(to_snake(key), key)` | `envelope.ts:87-92` — `index.set(toSnake(key), key)` |
| `.raw` 반환 | `models.py:82-84` — 원문 dict 사본 | `envelope.ts:100-102` — `{ ...this.source }` |
| 3중 조회 | `models.py:47-61` — 원문 → snake 색인 → camel 역변환 | `envelope.ts:125-138` — 같은 순서 |

결정적 근거는 **일괄 변환 함수가 존재하지만 바인딩 내부에서 한 번도 호출되지 않는다**는
사실이다.

- 파이썬: `_naming.py:59` `snake_keys` / `:74` `camel_keys` — `bindings/python/src` 전체
  grep 결과 정의부와 재귀 호출 외 사용처 0.
- Node: `naming.ts:80` `snakeKeys` / `:85` `camelKeys` — `bindings/node/src` 전체 grep 결과
  정의부와 `index.ts` 재수출 외 사용처 0.

두 바인딩 모두 이 함수들을 **호출자에게 선택지로만 제공**한다. 개명은 기본 경로가 아니다.

### 2.3 왜 이 쪽인가 (버린 대안)

**버린 대안 ①: 봉투를 통째로 언어 관례로 변환해서 넘긴다.**
`snake_keys(envelope)` 를 파이프라인에 넣으면 `env["page_count"]` 만 유효해진다.
그러면 세 가지가 깨진다.

1. **로그와 버그 리포트가 갈라진다.** 사용자가 붙여 넣은 JSON 이 CLI 출력과 다르면,
   메인테이너는 그게 도구 버그인지 바인딩 변환 버그인지 먼저 가려야 한다.
2. **문서가 언어 수만큼 갈라진다.** `capabilities` 의 `recordFields` 는 camelCase 목록
   하나다(실측: `render-diff` 의 `recordFields` 에 `pageCountA`·`hardStructPages` 등).
   변환하면 언어마다 그 목록을 다시 써야 하고, 그 순간 수기 매핑이 부활한다.
3. **왕복이 깨진다.** 계획서(`run --plan-json`)는 camelCase 로 보내야 한다. 받을 때
   바꾸고 보낼 때 되돌리면 변환 규칙 두 벌이 항상 왕복 일치해야 하는데, 그건
   `to_snake ∘ to_camel = id` 를 요구한다. 성립하지 않는다 — `pageCountA` →
   `page_count_a` → `pageCountA` 는 돌아오지만, 원문이 이미 snake 인 키
   (§2.4 의 `node_count`)는 돌아오지 않는다.

**버린 대안 ②: 원문 키만 받고 별칭을 두지 않는다.**
파이썬에서 `env["pageCount"]` 만 되고 `env.page_count` 가 안 되면, 그 언어를 쓰는 이유의
절반이 사라진다. 별칭은 **읽기 편의**일 뿐 저장 형식이 아니므로 위 세 문제를 만들지 않는다.

### 2.4 이 결정이 드러낸 본체 쪽 결함 (실측)

원문 보존을 계약으로 못 박으면, 원문이 규약을 어길 때 그 사실이 그대로 드러난다.

```
rhwp export-structure samples/2010-01-06.hwp --json
→ 최상위 키: mode, nodeCount, schemaVersion, source, structure,
             untrustedContent, untrustedFields          (camelCase)
→ structure 하위 키: mode, node_count, roots            (snake_case!)
```

봉투 전체를 재귀 순회해 `_` 가 든 키를 뽑으면 정확히 하나 나온다: `structure.node_count`.
같은 값이 최상위에는 `nodeCount` 로 있다. `info --json` 은 0건이다.

**판정: 본체 쪽 결함이다.** 봉투 키는 camelCase 가 계약인데 한 자리가 어긋나 있다.
두 바인딩 모두 3중 조회 덕에 **우연히** 동작하지만(`env["node_count"]` 도
`env["nodeCount"]` 도 통한다), 별칭 계층이 없는 언어(C#·Swift 의 정적 매핑)가 붙는 순간
그 자리에서 필드가 사라진다. 별도 업스트림 이슈감이다.

### 2.5 언어별 별칭 규칙 (M20 이 따라야 할 표)

| 언어 | 원문 저장 | 별칭 | 예약어 회피 |
|---|---|---|---|
| Python | `Envelope.raw` (dict) | `env.page_count` / `env["pageCount"]` / `env["page_count"]` | `_naming.py:86` `reserved_safe` — `from` → `from_` |
| Node/TS | `Envelope.raw` (제네릭 객체) | `env.get('pageCount')` / `get('page_count')` | `naming.ts:131-141` `RESERVED` 집합 |
| C# (M20) | `Envelope.Raw` (`JsonElement` 또는 `IReadOnlyDictionary`) | `env.Get("pageCount")` + `PageCount` PascalCase 프로퍼티 | C# 예약어는 `@` 접두 |
| Swift (M20) | `Envelope.raw` | `env["pageCount"]` + `pageCount` 프로퍼티 | 백틱 |

**변환 규칙은 코드로 고정하고 손으로 매핑표를 만들지 않는다.** 두 바인딩의 규칙은
정규식 두 개다.

```
약어 경계  파이썬 _naming.py:18  ([A-Z]+)([A-Z][a-z])
           Node   naming.ts:22   ([A-Z])([A-Z][a-z])      ← 고정 길이
단어 경계  양쪽                  ([a-z0-9])([A-Z])
```

Node 가 앞 그룹을 고정 길이로 바꾼 이유가 주석에 있다(`naming.ts:16-21`): 가변 길이
`[A-Z]+` 는 `AAAA…Aa` 입력에서 역추적이 다항으로 늘어난다(ReDoS). 결과는 같다 —
`HTMLPage` 로 실측 확인했다(파이썬 규칙도 `html_page`). **새 바인딩은 고정 길이 쪽을
쓴다.** 파이썬은 아직 가변 길이다(§7 D-6).

---

## 3. 판정 vs 실패 — #3719 불변식의 언어별 표현

### 3.1 종료 코드 사전 (실측: `rhwp capabilities` 의 `exitCodes`)

| 코드 | 도구 설명(원문 요약) | 성격 |
|---|---|---|
| 0 | 성공 | — |
| 1 | 런타임 실패 (읽기·파싱·렌더·쓰기) | **고장** |
| 2 | 사용법 오류 (인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과) | **고장(호출자 버그)** |
| 3 | 검증 단언 실패 — `convert`/`export-hwpx --verify` IR 차이, `edit` 3종 `--verify` 저장본 불일치, `run` 계획 assertions 미충족, `render-diff --json` 시각 회귀 | **판정** |
| 4 | `--verify-pages` 페이지 수 불일치 | **판정** |

### 3.2 계약

> **exit 3/4 는 예외가 아니다. 봉투를 반환값으로 돌려주고, 판정은 봉투의 필드로 읽는다.
> 예외를 원하는 호출자는 명시적으로 요청한다(기본값 아님).
> exit 1/2 는 예외다. 알 수 없는 코드도 예외로 올린다.**

근거는 두 바인딩의 주석이 같은 문장으로 적고 있다(`errors.py:9-14`, `errors.ts:10-13`):
`--verify` 가 불일치를 보고한 것은 **도구가 정상 동작한 결과**다. 예외로 만들면 호출자가
`try/except` 로 "고장"처럼 다루고, 정작 봉투에 담긴 판정 근거(`diffCount`·`status`·`pages`)를
읽지 않는다.

**실측으로 확인한 실행 경로** (파이썬, `RHWP_BIN` 지정):

```
rhwp export-hwpx samples/2010-01-06.hwp <out> --verify --json   → exit 3
rhwp.export_hwpx(S, verify=True, verify_pages=True)
  → 예외 없음. r.verify.identical == False, bool(r.verify) == False
```

exit 3 이 값으로 돌아온다는 것을 실제 실행으로 확인했다.

### 3.3 언어별 매핑표 (M20 이 채워야 할 자리)

| 상황 | Python (`errors.py`) | Node (`errors.ts`) | C#/Swift 지침 |
|---|---|---|---|
| 기반 | `RhwpError` (:51) | `RhwpError` (:57) | 언어의 최상위 예외를 상속한 단일 기반 |
| 바이너리 없음 | `BinaryNotFoundError` (:96) | `BinaryNotFoundError` (:116) | 이름 동일 |
| exit 2 | `UsageError` (:104) | `UsageError` (:124) | 이름 동일 |
| exit 1 | `RhwpRuntimeError` (:122) | `RhwpRuntimeError` (:160) | 이름 동일 |
| exit 3/4 (opt-in) | `VerdictFailed` (:130) | `VerdictFailed` (:168) | 이름 동일 |
| stdout 계약 위반 | `ProtocolError` (:144) | `ProtocolError` (:181) | 이름 동일 |
| 닫힌 세션 재사용 | `SessionClosedError` (:152) | `SessionClosedError` (:184) | 이름 동일 |
| 제한 시간 초과 | `TimeoutError` (:156) | `RhwpTimeoutError` (:187) | **`RhwpTimeoutError` 로 통일** |
| opt-in 스위치 이름 | `raise_on_verdict` | `throwOnVerdict` | 언어의 "예외를 올린다" 동사를 쓴다 |

시간 초과만 이름이 갈렸다. 파이썬이 내장 `TimeoutError` 를 가리는 쪽을 택했고
(`errors.py:156` 에 `noqa: A001 - 내장 이름 가림은 의도적(패키지 일관성)`), Node 는
접두를 붙였다. **새 바인딩은 접두형을 쓴다** — 내장 이름을 가리면 `except TimeoutError`
한 줄이 어느 쪽을 잡는지 파일마다 임포트를 봐야 알 수 있고, 그건 오류 처리 코드가 가장
피해야 할 성질이다. 파이썬 쪽 정정은 §7 D-7.

### 3.4 예외에 반드시 실어야 하는 것

두 바인딩 모두 예외에 네 가지를 담는다. **이것이 계약이다.**

| 필드 | 왜 필요한가 |
|---|---|
| `argv` | 재현 가능한 명령. 버그 리포트에 그대로 붙일 수 있어야 한다 |
| `exit_code` / `exitCode` | 프로세스를 못 띄웠으면 `None`/`undefined` — "0 아님"과 구분된다 |
| `stderr` | 진단은 stdout 이 아니라 stderr 에 있다. did-you-mean 힌트 포함 |
| `envelope` | **exit 3 일 때도 봉투가 나온다.** 판정 근거를 버리지 않는다 |

파싱을 종료 코드 검사보다 **먼저** 하는 이유가 여기 있다(`_process.py:156-194`,
`process.ts:191-236`). 순서를 뒤집으면 exit 3 봉투를 통째로 버린다.

### 3.5 계획 계층의 예외 — 유일한 정당한 예외의 예외

`run --plan-json` 의 선검증 위반은 **exit 2** 로 나온다. 기본 규약대로면 `UsageError` 다.
그러나 계획 실행에서 위반은 정상적인 결과이므로, `invalid[]` 를 담은 봉투는 값으로
돌려준다.

- 파이썬 `plan.py:205-226` — `UsageError` 를 잡아 `exc.envelope` 에 `invalid` 가 있으면
  `PlanResult` 로 변환, 없으면 재발생.
- Node `plan.ts:290-302` — 동일 로직.

**새 바인딩은 이 예외를 그대로 승계한다.** 조건도 같다: `invalid` 키가 **없는** exit 2 는
진짜 호출 조립 버그이므로 그대로 올린다.

### 3.6 batch 의 부분 실패

`batch` 는 NDJSON 이고 **부분 실패도 실패**다(capabilities `jsonContract.failure`:
"batch 는 error 레코드 + 최종 exit 1"). 규약:

- exit 1 을 예외로 올리지 않는다. 성공 레코드를 버리면 안 되기 때문이다.
- exit 2 는 스트림이 성립하지 않은 것이므로 예외로 올린다.
- 실패 항목은 `error` 필드를 단 레코드로 스트림에 남는다.

근거: `_process.py:207-249`, `process.ts:277-301` — 두 구현이 동일하다.

---

## 4. 버전 정합 — 확인하나, 무시하나

### 4.1 현행 상태 (실측)

**어느 바인딩도 런타임에 버전을 확인하지 않는다.**

- 파이썬: `__init__.py:115` `SUPPORTED_SCHEMA_VERSION = "1.0"` — 상수 선언뿐.
  비교는 통합 테스트 `test_envelope_parity.py:82` 에서만.
- Node: `index.ts:65` 동일 상수 — 비교하는 코드 없음. 통합 테스트
  `commands.integration.test.ts:58` 은 상수가 아니라 리터럴 `'1.0'` 과 비교한다.
- 바인딩 패키지 버전(둘 다 `0.1.0`)과 rhwp 바이너리 버전(`0.8.2`)을 대조하는 코드도 없다.

### 4.2 결정

> **① 봉투 `schemaVersion` 의 major 불일치는 거부한다(예외).
> ② minor 차이와 필드 추가는 통과시킨다.
> ③ 바이너리 semver 버전은 확인하지 않는다 — 대신 `capabilities` 로 *기능의 유무*를 묻는다.**

### 4.3 근거

**① major 를 막는 이유.** 봉투 계약은 "필드 추가 허용, 변경·삭제는 schemaVersion 범프"다
(실측: `capabilities.jsonContract.schemaPolicy`). 즉 major 가 올랐다는 것은 **필드의 의미가
바뀌었거나 사라졌다**는 뜻이고, 그 봉투를 구 바인딩이 읽으면 조용히 틀린 값을 낸다.
`ProtocolError` 는 "stdout 이 계약을 어겼다"는 계열이므로 이 자리에 맞다.

**② minor 를 통과시키는 이유.** 추가-전용 진화에서 새 필드는 구 바인딩에게 그냥 안 보일
뿐 해롭지 않다. 여기서 막으면 바이너리를 올릴 때마다 모든 바인딩이 동시에 릴리스돼야 하고,
그 결합은 "언어가 늘어도 계약은 한 곳" 이라는 축의 전제를 깨뜨린다.

**③ semver 를 안 보는 이유가 이 절의 핵심이다.** `rhwp 0.8.2` 라는 문자열은 "이 기능이
있는가"에 답하지 못한다. 버전 범위 표(`>=0.8.0` 이면 `--dry-run` 지원 …)를 바인딩마다
들고 있으면 그 표가 곧 수기 매핑이고, 언어 수만큼 복제되며, 포크·패치 빌드에서 틀린다.

**대신 쓸 것이 이미 있다.** Node 가 선례를 만들었다 — `plan.ts:248-275`
`assertDryRunSupported`:

```
capabilities 를 읽어 commands 중 name == 'run' 의 flags 에
'--dry-run' 이 있는지 본다. 없으면 던진다:
  "이 rhwp 는 계획 --dry-run 을 지원하지 않습니다 (#3759 이전 버전).
   check() 를 실행으로 대체하지 않습니다 — 검사인 줄 알고 문서가 편집되면 안 됩니다."
```

**버전이 아니라 자기서술에 묻는다.** 이것이 계약이다. 결과는 프로세스 수명 동안 캐시하고
(`plan.ts:240`), 테스트를 위한 무효화 함수를 둔다(`clearPlanCapabilityCache`, `plan.ts:277`).

### 4.4 새 바인딩이 구현해야 하는 세 지점

1. `Envelope` 생성 시점 또는 `runJson` 반환 직전에 `schemaVersion` major 대조.
   불일치면 `ProtocolError` — 메시지에 **바인딩이 아는 버전과 도구가 낸 버전을 모두** 싣는다.
2. 기능 유무가 안전에 영향을 주는 자리(현재 알려진 것: 계획 `--dry-run`)마다
   `capabilities` 게이트. **"지원 안 하면 안전한 쪽으로 대체"는 금지** — `check()` 를
   `run()` 으로 대체하면 검사인 줄 알고 파일이 만들어진다.
3. 바인딩 패키지 버전은 rhwp 버전과 **독립**으로 매긴다(현행 둘 다 `0.1.0` 대 `0.8.2`).
   동기화하면 바인딩 버그 수정에도 rhwp 릴리스가 필요해진다.

### 4.5 미결

`irSchemaVersion`(실측 `1.0`, `export-ir-schema` 봉투의 필드)이 올랐을 때 생성 타입을 가진
바인딩(Node·M20)이 어떻게 반응해야 하는지는 **확인되지 않음**. 현재 두 바인딩 모두
`IrSchema.version` 으로 읽기만 하고(`schema.py:164-174`, `schema.ts:213-219`) 대조하지 않는다.
생성물이 저장소에 커밋되므로 CI 의 `--check` 재생성이 사실상의 가드지만, 그건 개발 시점
가드일 뿐 사용자 런타임 가드가 아니다.

---

## 5. 노출 범위 — 무엇을 바인딩에 넣지 않는가

### 5.1 실측: 도구가 선언한 표면

`rhwp capabilities` 기준 명령 61개, 그중 `json:true` 31개. 범주 분포는
`diagnostic 25 / export 18 / query 8 / internal 5 / edit 3 / serve 1 / batch 1`.

`json:true` 이면서 `category:diagnostic` 인 것은 정확히 셋이다:
**`dump-pages`, `ir-diff`, `render-diff`**.

### 5.2 현행 노출 (실측)

| 명령 | 범주 | Python | Node |
|---|---|---|---|
| `dump-pages` | diagnostic | 미노출 | 미노출 |
| `ir-diff` | diagnostic | `ir_diff` | `irDiff` |
| `render-diff` | diagnostic | **미노출** | `renderDiff` |

파이썬 미노출은 실행으로 확인했다: `hasattr(rhwp, "render_diff") == False`.

Node 의 패리티 테스트는 `diagnostic`·`internal`·`serve` 를 통째로 제외 목록에 넣어 두고
(`parity.integration.test.ts:50`), 정작 `ir-diff`·`render-diff` 는 감싼다. **제외 목록과
실제 노출이 어긋나 있고, 테스트는 그 어긋남을 잡지 못한다** — 그 테스트는
"선언 → 래퍼 존재" 한 방향만 보기 때문이다(§6.2).

### 5.3 결정

> **노출 여부는 `category` 가 아니라 "봉투가 안정 계약인가"로 가른다.**

| 기준 | 노출 | 예 |
|---|---|---|
| `capabilities` 에 `json:true` + `recordFields` 가 선언돼 있고, 그 필드가 계약 테스트로 지켜진다 | **한다** | `ir-diff`, `render-diff` |
| 봉투 모양이 엔진 내부 구조를 그대로 비추어 자주 바뀐다 | **안 한다** | `dump-pages` (조판 진단용 페이지 항목 덤프) |
| 픽스처 생성기 등 저장소 내부용 (`category:internal`) | **안 한다** | 5건 |
| 프로세스를 띄우는 축 (`category:serve`) | **안 한다** — 2층 `Session` 이 이미 담당 | `mcp-serve` |
| CLI 가 서브커맨드로 갈라지는 명령 | 서브커맨드마다 함수 | `edit` → `fill_fields`/`replace_text`/`set_cell` |
| 위층이 감싸는 명령 | 1층에 두지 않는다 | `run` → 3층 `Plan` |

`edit` 을 문자열 디스패치(`edit(path, "fill-fields", …)`)로 두지 않는 이유는
Node 주석이 정확하다(`parity.integration.test.ts:56`): 오타를 런타임까지 미룬다.

`run` 을 1층에 두지 않는 이유(같은 파일 :63-66): 호출자가 계획서 JSON 을 손으로 조립하게
되고, 빌더의 문법 검사와 `check()` 미리보기를 통째로 우회한다.

### 5.4 플래그는 명령과 다른 기준으로 가른다

Node 가 `DESIGN.md` D14 에서 세운 규칙을 이 문서가 계약으로 승격한다.

> **`capabilities` 의 `flags` 는 "이 명령이 이 플래그를 파싱한다"만 말한다. `--json`
> 모드에서 그 플래그가 무엇을 하는지는 말하지 않는다. 봉투 계약을 깨거나 아무 일도
> 하지 않는 플래그는 닫는다.**

이 PC 에서 재실측한 결과 (표는 전부 직접 실행):

| 플래그 | 실측 동작 | 판정 |
|---|---|---|
| `export-text -o` (`--json` 동반) | 무시 — 파일이 생기지 않음 | 닫는다 |
| `export-structure -o` | 동일 | 닫는다 |
| `export-tables -o` | stdout 이 `표 추출 완료: 12개 → …` 사람 문장으로 **바뀐다** | 닫는다. **본체 쪽 결함** |
| `export-capabilities-schema -o` | stdout 이 봉투를 유지 | 연다 |
| `convert -o` / `export-hwpx -o` | `알 수 없는 옵션: -o`, **exit 2** | 위치 인자로 넘긴다 |
| `render-diff --batch` | 봉투가 아니라 NDJSON 스트림 | 닫는다(반환 타입이 달라진다) |

`export-tables -o` 는 바인딩이 옵션을 닫은 것이 **회피일 뿐 수정이 아니다.** `--json` 을
준 호출에서 출력 형식이 바뀌는 것은 명령 하나의 사정이 아니라 봉투 계약 전체의 예외이므로,
별도 업스트림 이슈감으로 남긴다.

### 5.5 노출한 명령의 옵션은 전부 열어야 한다

§5.4 의 예외를 뺀 나머지에서, **한쪽 바인딩에만 있는 옵션은 표류다.** 현행 실측 차이는
§7 D-12 에 있다(파이썬에 `export_text(page=)`, `export_structure(mode=)`,
`digest(max_chars=)`, `ir_diff(section=, paragraph=)` 가 없다).

---

## 6. 동등성을 테스트로 강제하는 방법

계약을 문서로만 두면 지켜지지 않는다. §7 의 표류 20건이 그 증거다. 이 절은 **강제 장치의
설계**다. 세 층으로 나눈다.

### 6.1 A층 — 자기서술 대조 (바인딩마다, 바이너리 필요)

`capabilities` 가 단일 출처다. 수기 목록을 두지 않는다.

**A-1. 선언 → 래퍼 (이미 Node 에 있음)**
`parity.integration.test.ts:76-111`. 선언된 `json:true` 명령마다 대응 함수가 모듈에
있는지. 없으면 그 명령을 그 언어 사용자는 못 쓴다.

**A-2. 래퍼 → 선언 (양쪽 모두 없음 — 신규)**
반대 방향. 모듈이 내보내는 명령 래퍼마다 `capabilities` 에 대응 명령이 있는지.
없으면 도구에서 사라진 명령을 바인딩이 계속 광고하고 있는 것이다.
**§5.2 의 어긋남(제외 목록에 diagnostic 을 넣고 diagnostic 을 감쌈)은 이 방향의 테스트가
없어서 드러나지 않았다.** 이 테스트는 §5.3 의 노출 기준을 명시 목록으로 받아
"기준에 없는데 감쌌다"를 실패로 만든다.

**A-3. 옵션 → 플래그**
래퍼가 조립하는 플래그가 `capabilities` 의 `flags` 에 있는지. 이 테스트가 있었다면
파이썬의 `convert(out=)` → `-o` 버그(§7 D-1)가 머지 전에 잡혔다.
구현은 인자 조립을 **실행에서 분리**해야 한다 — 현재 두 바인딩 모두 조립과 실행이
한 함수에 있어 argv 만 뽑아 볼 수 없다. 이것이 이 층의 유일한 구조 변경 요구다.

**A-4. 종료 코드 사전 대조 (양쪽에 이미 있음)**
바인딩이 매핑하는 다섯 코드를 도구의 `exitCodes` 가 전부 설명하는지.
`parity.integration.test.ts:159-172`, `test_envelope_parity.py:107-121`.

### 6.2 B층 — 교차 실행 골든 (바인딩들 사이, 신규)

A층은 각 바인딩이 **혼자서** 도구와 어긋나지 않았는지만 본다. 두 바인딩이 서로 다른
방식으로 어긋나 있으면 A층은 둘 다 통과시킨다. 그래서 B층이 필요하다.

**설계:**

```
입력: 고정 표본 문서 N개 × 명령·옵션 조합 M개  (= 케이스 목록 JSON 한 벌)
절차:
  1. CLI 로 직접 실행 → stdout 원문을 기준(golden)으로 저장
  2. 각 바인딩으로 같은 케이스 실행 → envelope.raw 를 JSON 직렬화
  3. 정규화 후 바이트 비교
비교 대상: 봉투 원문만. 접근자 이름·반환 타입은 비교하지 않는다(§1)
```

**정규화가 필요한 필드 (실측으로 확인한 비결정 요소):**

| 필드 | 왜 다른가 | 처리 |
|---|---|---|
| `source` | 호출자가 넘긴 경로 문자열 그대로 (실측: `"samples/2010-01-06.hwp"`) | 파일명만 남긴다 |
| `output` | 절대 경로 (실측: `"C:/Users/…/out.hwpx"`) | 파일명만 남긴다 |
| `bytes`·`sizeBytes` | 산출물 크기 — 같은 입력이면 같아야 한다 | **정규화하지 않는다.** 다르면 그게 결함이다 |

**케이스 목록을 어디에 두는가**: 저장소 한 곳(`bindings/parity/cases.json` 등)에 두고
언어별 러너가 읽는다. 언어마다 목록을 복제하면 그 목록 자체가 갈라진다.

**실패 시 보고**: 어느 케이스의 어느 키가 다른지 + 양쪽 값. "다르다"만 알려주면
`export-structure` 의 `node_count` 같은 자리를 찾는 데 반나절이 든다.

**이 층이 잡았을 표류**: D-3(파이썬이 `--json` 없이 스키마 명령을 부른다 — 오늘은
출력이 같아 무해하지만, 달라지는 순간 이 테스트가 즉시 잡는다).

### 6.3 C층 — 오류·판정 매핑 대조 (신규)

봉투가 같아도 **오류 표현이 다르면 동등하지 않다.** 케이스 목록은 §3.3 표에서 나온다.

```
케이스: (상황, 기대 종료 코드, 기대 예외 계열, 예외인가 값인가)
  없는 파일          → exit 1 → RhwpRuntimeError      → 예외
  알 수 없는 명령    → exit 2 → UsageError            → 예외
  --verify 불일치    → exit 3 → (없음)                 → 값 (verify.identical == False)
  --verify + opt-in  → exit 3 → VerdictFailed          → 예외
  --verify-pages 불일치 → exit 4 → (없음)              → 값
  계획 선검증 위반   → exit 2 → (없음, invalid 있음)   → 값 (PlanResult.ok == False)
  계획 조립 버그     → exit 2 → UsageError             → 예외
  stdout 이 JSON 아님 → —     → ProtocolError          → 예외
  제한 시간 초과     → —      → RhwpTimeoutError       → 예외
```

각 바인딩이 이 표를 자기 언어로 구현하고, **표 자체는 저장소 한 곳**에 둔다.
파이썬은 이 표의 절반을 이미 갖고 있다(`test_envelope_parity.py:148-157`).

**"예외인가 값인가" 열이 이 층의 존재 이유다.** 클래스 이름이 같아도 한쪽이 던지고
한쪽이 돌려주면 같은 코드가 다른 결과를 낸다.

### 6.4 어디서 도는가

| 층 | 바이너리 | 두 바인딩 런타임 | CI 위치 |
|---|---|---|---|
| A | 필요 | 각각 따로 | 각 바인딩 잡 (이미 있음) |
| B | 필요 | **동시에 필요** | 새 잡 — Python·Node 를 함께 설치한 러너 |
| C | 필요 | 각각 따로 | 각 바인딩 잡 |

B층만 새 잡을 요구한다. 언어가 늘 때마다 그 러너에 한 줄 추가하는 구조여야 한다 —
"바인딩 × 바인딩" 행렬로 짜면 M20 에서 6쌍이 된다. **각자 golden 과 비교**하는 별 모양이
정답이다.

### 6.5 마킹 규약

바이너리가 없으면 A·B·C 층은 **건너뛴다**(파이썬 `pytestmark = pytest.mark.integration`,
Node `describe.skipIf(!hasBinary)`). 단위 테스트는 가짜 바이너리로 바이너리 없이 돈다 —
두 바인딩 모두 이미 이 구조다(`tests/conftest.py`, `test/helpers/fake-binary.ts`).

---

## 7. 확인된 표류 목록 (2026-08-03 기준)

세부 근거와 판정(의도인가 표류인가)은 [`python_node_comparison.md`](python_node_comparison.md)
에 있다. 여기에는 계약 위반 여부만 적는다.

| # | 내용 | 계약 위반 | 심각도 |
|---|---|---|---|
| D-1 | 파이썬 `convert(out=)`·`export_hwpx(out=)` 가 `-o` 를 붙여 **항상 exit 2** | §5.5 | **치명** — 기능이 죽어 있다 |
| D-2 | 파이썬에 `render_diff` 없음 | §1 명령 집합 | 높음 |
| D-3 | 파이썬 스키마 명령이 `--json` 없이 실행 | §6.2 | 낮음(현재 무해) |
| D-4 | 파이썬 `Plan.check()` 에 `--dry-run` 지원 게이트 없음 | §4.4-2 | **높음** — 검사가 실행이 될 수 있다 |
| D-5 | 파이썬 `_quote` 가 역슬래시를 이스케이프하지 않음 | §3.4 `argv` | 중간 |
| D-6 | 파이썬 약어 정규식이 ReDoS 가능 형태 | §2.5 | 중간 |
| D-7 | `TimeoutError` vs `RhwpTimeoutError` | §3.3 | 중간 |
| D-8 | 파이썬 `UsageError` 에 `next_call` 없음 | §3.4 | 낮음 |
| D-9 | `VerifyReport` 진리값 의미가 다름 | §1 결론을 바꾼다 | **높음** |
| D-10 | 양쪽 다 `verifyPages` 접근자 없음 | §1 (공통 결손) | 낮음 |
| D-11 | `inspect` 인자 순서가 반대 | §1 | 중간 |
| D-12 | 파이썬에 4개 명령의 옵션 누락 | §5.5 | 중간 |
| D-13 | 파이썬 `iter_ndjson` 이 공개 API 아님 | §1 | 낮음 |
| D-14 | `Session` 옵션 비대칭 (timeout / cwd) | §1 | 낮음 |
| D-15 | 바이너리 탐색 보조 API 이름·노출 비대칭 | §1 | 낮음 |
| D-16 | `RHWP_BIN` 의 `~` 확장이 파이썬만 됨 | §1 | 낮음 |
| D-17 | `Envelope` 접근자 집합 비대칭 | 언어 관례 범위 | 낮음 |
| D-18 | `raise_for_exit`/`isKnownExitCode` 노출 비대칭 | §1 | 낮음 |
| D-19 | 파이썬에 "선언 → 래퍼" 패리티 테스트 없음 | §6.1 A-1 | **높음** — D-2·D-12 의 원인 |
| D-20 | Node `runRaw` 가 예외에 봉투를 싣지 않음 | §3.4 | 낮음 |

**D-19 가 구조적 원인이다.** 파이썬에는 표면 완결성 테스트가 없어서 D-2·D-12 가
머지까지 살아남았다. §6.1 A-1 을 파이썬에 추가하는 것이 이 목록에서 가장 먼저 할 일이다.

---

## 8. `bindings/` 안의 세 번째 계약 — M20 이 먼저 정리해야 할 것

`bindings/csharp`·`bindings/swift`·`bindings/Native` 는 **비어 있지 않다.** 실측:

```
bindings/Native/src/lib.rs                        376줄  (cdylib, C ABI)
bindings/csharp/RhwpNative.cs                      63줄  (P/Invoke)
bindings/swift/Sources/Rhwp/Rhwp.swift            182줄
bindings/swift/Sources/Rhwp/RhwpDocumentTextView.swift  92줄
bindings/swift/Sources/CRhwpNative/rhwp_native_ffi.h    17줄
```

`bindings/README.md:5-9` 이 이들을 "Native ABI" 로, python·node 를 "CLI subprocess
bindings" 로 **명시적으로 갈라 놓았다.** 즉 M20 이 미착수인 것은 맞지만, 그 자리에는
이미 다른 계약의 구현물이 있다.

### 8.1 그 계약은 이 문서의 계약과 호환되지 않는다

| 항목 | CLI 서브프로세스 계열 (M18·M19) | C ABI 계열 (기존 csharp/swift) |
|---|---|---|
| 표면 | 명령 28개 + 세션 + 계획 | **함수 3개** — `rhwp_export_text`, `rhwp_export_markdown`, `rhwp_read_text` (`rhwp_native_ffi.h`) |
| 봉투 | `schemaVersion` 필수, camelCase, 명령별 `recordFields` | `{"ok":true,"pageCount":N,"files":[…]}` / `{"ok":false,"error":"…"}` (`Native/src/lib.rs:323-341`) |
| `schemaVersion` | 있음 | **없음** |
| 종료 코드 | 0/1/2/3/4 사전 | 없음 — `ok` 불리언 하나 |
| 판정(exit 3/4) | 값으로 표현 | **표현 수단 없음** |
| 오류 | 5계열 예외 + 봉투 보존 | `error` 문자열 하나 (Swift 는 `RhwpError.exportFailed(String)`, `Rhwp.swift:49-53`) |
| 진단 보존 | `argv`·`stderr`·`envelope` | 없음 |
| panic 처리 | 해당 없음 | `catch_unwind` → `{"ok":false,"error":"FFI 호출 중 panic이 발생했습니다."}` (`lib.rs:274-289`) |

`bindings_foundation.md` §2 의 표면 판단 매트릭스는 C ABI 를 **"수요 실증 후 승격"** 으로
두었다. 기존 구현은 그 판단보다 앞서 존재하는 코드다.

### 8.2 M20 이 답해야 할 질문 (이 문서는 답을 강제하지 않는다)

1. C#/Swift 의 CLI 서브프로세스 바인딩을 **같은 디렉터리에** 만들 것인가, 별 이름으로
   가를 것인가. 같은 디렉터리에 두면 `Rhwp` 네임스페이스에 두 계약이 공존한다.
2. 기존 C ABI 를 **유지**할 것인가(SwiftUI `RhwpDocumentTextView` 라는 실사용 형태가 있다),
   서브프로세스 계열로 흡수할 것인가.
3. 유지한다면 C ABI 쪽에 `schemaVersion` 과 종료 코드 상당물을 도입할 것인가.

**확인되지 않음**: 기존 C ABI 계열이 어느 이슈에서 왔는지, 현재 사용자가 있는지.
`bindings/README.md` 에도 `bindings_foundation.md` 에도 그 이력이 없다.

### 8.3 그때까지의 최소 방어

`bindings/README.md` 가 두 계열을 갈라 놓은 문장을 **강화한다**: C ABI 계열은 이 문서의
동등성 계약 대상이 **아니며**, §6 의 A·B·C 층 테스트도 적용하지 않는다. 한 디렉터리
안에 두 계약이 있다는 사실 자체를 문서가 먼저 말해야, 다음 사람이 `bindings/csharp` 를
보고 "M20 이 이미 됐네" 라고 결론 내리지 않는다.

---

## 9. 문서 자체의 표류 (실측)

`bindings/README.md:24-26` 은 파이썬을 "**submitted, not merged** (PR #3775)", Node 를
"**in progress**" 로 적고 있다. 그러나 이 워크트리(upstream/devel 기준)에는 두 디렉터리가
모두 소스와 테스트를 갖춘 상태로 존재한다(파이썬 `src/rhwp` 11파일 3,148줄, Node `src`
14파일 5,377줄 — `wc -l` 실측). **상태 문구가 뒤처졌다.**

같은 파일 :22-26 이 가리키는 `mydocs/manual/python_binding_guide.md` 는 실제로 존재한다
(193줄). 링크는 유효하고 상태 문구만 낡았다.

바인딩이 늘수록 이런 상태 문구가 늘어난다. **상태는 문서에 쓰지 말고 디렉터리 존재로
말하게 하는 편이 낫다** — 이 문서는 상태를 쓰지 않고 "실측 시점의 코드가 이렇다"만 쓴다.

---

## 10. 관련 문서

- [`bindings_foundation.md`](../bindings_foundation.md) — 표면 판단·IR 스키마 버저닝의 권위
- [`new_binding_guide.md`](new_binding_guide.md) — 새 언어를 붙이는 절차
- [`python_node_comparison.md`](python_node_comparison.md) — 표류 20건의 근거와 판정
- [`README.md`](README.md) — 이 디렉터리의 지도
- [`python_binding_guide.md`](../../manual/python_binding_guide.md) ·
  [`node_binding_guide.md`](../../manual/node_binding_guide.md) — 언어별 사용법
- [`envelope_provenance.md`](../envelope_provenance.md) — `untrustedContent` 표지 계약
- 로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M18~M20
