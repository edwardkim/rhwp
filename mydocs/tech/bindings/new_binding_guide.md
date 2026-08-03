---
kind: guide
status: active
canonical: mydocs/tech/bindings/parity_contract.md
last_verified: 2026-08-03
---

# 새 언어 바인딩 추가 절차 — 파이썬·Node 가 실제로 한 일에서 뽑은 12단계

C#·Swift(M20) 또는 그 뒤의 언어를 붙이려는 사람이 읽는 문서다. 각 단계마다 **기존 두
구현이 그 자리에서 무엇을 했는지 코드 경로로 인용**한다 — 새로 설계할 것은 거의 없고,
대부분은 이미 두 번 풀린 문제를 세 번째 언어로 옮기는 일이다.

- 지켜야 할 계약: [`parity_contract.md`](parity_contract.md) (권위)
- 두 구현의 현행 차이: [`python_node_comparison.md`](python_node_comparison.md)
- 설계 전제: [`bindings_foundation.md`](../bindings_foundation.md)

## 0. 시작 전 — 착수 조건과 세 가지 함정

### 0.1 착수 조건

`bindings_foundation.md` §4: **M20 은 "공공 SI 수요 실증 1건"(이슈 유입 또는 사용 보고)이
착수 조건이다.** 수요 없이 만든 바인딩은 유지비만 남긴다.

### 0.2 함정 ① — `bindings/csharp`·`bindings/swift` 는 비어 있지 않다

**실측**: 그 자리에 이미 다른 계약의 구현이 있다.

```
bindings/Native/src/lib.rs   376줄  cdylib, C ABI 함수 4개
bindings/csharp/RhwpNative.cs 63줄  P/Invoke 2개 (ExportText, ExportMarkdown)
bindings/swift/Sources/…     274줄  Rhwp.swift + RhwpDocumentTextView.swift
```

노출 표면은 `rhwp_export_text`·`rhwp_export_markdown`·`rhwp_read_text`·`rhwp_string_free`
넷뿐이고(`bindings/swift/Sources/CRhwpNative/rhwp_native_ffi.h`), 봉투는
`{"ok":true,"pageCount":N,"files":[…]}` 형태로 **`schemaVersion` 도 종료 코드도 없다**
(`bindings/Native/src/lib.rs:323-341`).

`bindings/README.md:5-9` 가 이 계열을 "Native ABI" 로 따로 묶어 두었다.
자세한 비교와 M20 이 답해야 할 질문은 [`parity_contract.md`](parity_contract.md) §8 에 있다.
**그 질문에 먼저 답하지 않고 같은 디렉터리에 서브프로세스 바인딩을 만들면 한 네임스페이스에
두 계약이 공존한다.**

### 0.3 함정 ② — 재구현하고 싶어지는 순간이 온다

바인딩은 재포장이다. **문서 파싱·좌표 계산·판정 로직을 바인딩 언어로 다시 쓰지 않는다.**
두 번 쓰면 두 답이 언젠가 갈라지고, 그때 어느 쪽이 맞는지 아무도 모른다
(`bindings/README.md:12-16`).

### 0.4 함정 ③ — 파이썬을 그대로 베끼면 알려진 버그까지 온다

[`python_node_comparison.md`](python_node_comparison.md) 의 표류 20건 중 12건이 파이썬 쪽이다.
특히 D-1(`convert`/`export_hwpx` 의 `-o`), D-4(`--dry-run` 게이트 없음), D-5(인용 이스케이프)는
**베끼면 그대로 옮겨 온다.** 두 구현이 다를 때는 그 문서의 판정을 따른다.

---

## 1단계 — 표면 선택: 서브프로세스인가 C ABI 인가

`bindings_foundation.md` §2 의 매트릭스가 답한다: **1차 권고는 CLI 서브프로세스 래퍼.**
C ABI 는 "수요 실증 후 승격".

이유는 유지비다. 서브프로세스는 #2707 종료 코드 사전과 봉투 계약을 **그대로 재사용**하므로
바인딩에 남는 것은 봉투 파서뿐이다. C ABI 는 ABI 안정성·메모리 규약·플랫폼별 배포 행렬을
새로 만들어야 하고, 그건 되돌릴 수 없는 배포 표면이 된다.

**C ABI 를 고른다면 이 문서의 나머지는 절반만 적용된다** — 종료 코드가 없으므로 §5 의
판정/실패 구분을 다른 방식으로 표현해야 하고, 그 설계는 아직 없다.

---

## 2단계 — 모듈 골격을 두 구현과 같은 이름으로 자른다

두 바인딩의 파일 구성이 거의 1:1 이다. 세 번째도 같게 자른다 — 그래야 "저기서는 어떻게
했지"를 파일 이름으로 찾을 수 있다.

| 역할 | Python | Node | 새 바인딩 |
|---|---|---|---|
| 바이너리 탐색 | `_binary.py` (137줄) | `binary.ts` (163줄) | `Binary` |
| 종료 코드 → 오류 | `errors.py` (216줄) | `errors.ts` (268줄) | `Errors` |
| 프로세스 실행 | `_process.py` (295줄) | `process.ts` (359줄) | `Process` |
| 이름 변환 | `_naming.py` (95줄) | `naming.ts` (146줄) | `Naming` |
| 봉투 래퍼 | `models.py` (178줄) | `envelope.ts` (254줄) | `Envelope` |
| 1층 명령 | `commands.py` (505줄) | `commands.ts` (835줄) + `document-analysis.ts` (212줄) | `Commands` |
| 2층 세션 | `session.py` (394줄) | `session.ts` (456줄) | `Session` |
| 3층 계획 | `plan.py` (235줄) | `plan.ts` (312줄) | `Plan` |
| 스키마 소비 | `schema.py` (282줄) | `schema.ts` (324줄) | `Schema` |
| 생성 타입 | `ir.py` (613줄) | `ir.ts` + `envelopes.ts` (1,533줄) | 정적 타입 언어면 필수 |

**3층 구조(무상태 / 세션 / 계획)는 CLI 의 층과 같다.** 새로 발명한 계층이 아니라
`rhwp <명령>` / `mcp-serve` / `run --plan-json` 세 표면을 그대로 비춘 것이다.

---

## 3단계 — 바이너리 탐색: 순서 자체가 계약이다

```
1. 환경변수 RHWP_BIN
2. 패키지 동봉
3. PATH
```

`bindings_foundation.md` §3 이 고정했다. **순서를 바꾸면 안 되는 이유**가 두 구현의 주석에
같은 문장으로 있다(`_binary.py:9-10`, `binary.ts:10-11`): 개발자가 로컬 빌드를 가리키고
싶을 때(1) 패키지 동봉본(2)이 가로채면 "왜 내 수정이 반영 안 되지"라는 진단 불가 상황이
생긴다.

**구현 체크리스트:**

- [ ] `RHWP_BIN` 이 **설정됐는데 못 쓰면 다음으로 넘어가지 않고 즉시 예외**.
      (`_binary.py:75-80`, `binary.ts:78-100`) — 조용히 다른 바이너리가 실행되면
      디버깅이 불가능해진다.
- [ ] `RHWP_BIN` 이 디렉터리면 그 안의 실행 파일을 본다. (`_binary.py:71-72`, `binary.ts:87-90`)
- [ ] 홈 디렉터리(`~`) 확장을 한다. **Node 는 안 한다**([D-16](python_node_comparison.md)) —
      파이썬 쪽이 맞다.
- [ ] 실행 가능한 **파일**인지 확인한다. Windows 는 실행 비트가 없으므로 확장자
      (`.exe`/`.bat`/`.cmd`)로 판단. (`_binary.py:55-57`, `binary.ts:63-66`)
- [ ] `stat` 자체가 실패하는 경우(경로 길이·권한)를 예외 없이 `false` 로 처리.
- [ ] 결과를 프로세스 수명 동안 캐시하고, **테스트용 무효화 함수**를 둔다.
      이름은 `clearBinaryCache` 계열로 — 파이썬의 `clear_cache` 는 무엇의 캐시인지
      말하지 않는다([D-15](python_node_comparison.md)).
- [ ] 못 찾았을 때 **시도한 위치를 전부** 메시지에 담는다. "없다"만 알려주면 사용자가
      어디에 둬야 할지 모른다. (`_binary.py:133-137`, `binary.ts:158-162`)

---

## 4단계 — 프로세스 실행: 계약을 신뢰하되 검증한다

`--json` 모드의 계약(실측: `rhwp capabilities` 의 `jsonContract`):

```
stdout   : 데이터(JSON/NDJSON)만 — 진단·진행·요약은 stderr
failure  : 단건 명령 실패 시 stdout 0바이트; batch 는 error 레코드 + 최종 exit 1
schemaPolicy: 필드 추가 허용, 변경·삭제는 schemaVersion 범프
```

**구현 체크리스트:**

- [ ] 셸을 태우지 않는다. `spawn(…, {shell:false})` (`process.ts:87`) — 셸을 거치면
      윈도우 인용 규칙 때문에 한글 경로가 깨진다.
- [ ] stdout/stderr 를 **UTF-8 로 명시 디코딩**한다. 플랫폼 기본 인코딩을 따르면
      윈도우에서만 한글이 깨지고, 그 깨짐이 "바인딩 버그"로 오인된다.
      (`_process.py:106-107`, `process.ts:123-124`)
- [ ] 불리언을 인자 **값 위치**에 넣으면 즉시 `TypeError`. 불리언은 플래그로 표현해야
      하므로 값 위치에 오면 호출 조립 버그다. (`_process.py:61-66`, `process.ts:62-67`)
- [ ] **봉투 파싱을 종료 코드 검사보다 먼저** 한다. exit 3 일 때도 봉투가 나오고, 그
      봉투에 판정 근거가 들어 있다. 순서를 뒤집으면 가장 중요한 정보를 버린다.
      (`_process.py:156-194`, `process.ts:191-236`)
- [ ] stdout 이 JSON 이 아니거나 **성공인데 비어 있으면** `ProtocolError`.
      빈 결과를 "차이 없음"으로 오독하는 것이 이 축에서 가장 위험한 실패다.
- [ ] NDJSON: exit 1 은 예외로 올리지 않고 레코드의 `error` 필드를 호출자가 보게 한다.
      exit 2 만 예외. (`_process.py:246-248`, `process.ts:296-299`)
- [ ] 스트리밍 이터레이터를 제공하고, **소비자가 중간에 멈춰도 자식을 정리**한다.
      남으면 파일을 잡고 있어 다음 작업이 막힌다. (`_process.py:291-295`, `process.ts:353-358`)
- [ ] 제한 시간 기본 **300초**. `null`/`None` 이면 무제한. (`_process.py:27`, `process.ts:31`)

---

## 5단계 — 오류 매핑: 판정과 고장을 섞지 않는다

[`parity_contract.md`](parity_contract.md) §3 이 권위다. 요약:

```
exit 1  → RhwpRuntimeError   (예외)
exit 2  → UsageError         (예외)
exit 3  → 반환값             (opt-in 시 VerdictFailed)
exit 4  → 반환값             (opt-in 시 VerdictFailed)
그 외 0 아님 → RhwpRuntimeError ("알 수 없는 종료 코드" — 조용히 통과 금지)
```

**실측으로 확인한 도구 동작:**

```
$ rhwp info nope.hwp --json
오류: 파일을 읽을 수 없습니다 - nope.hwp: … (os error 2)      # stderr
exit=1, stdout 0바이트

$ rhwp bogus-cmd --json
오류: 알 수 없는 명령입니다 - bogus-cmd                        # stderr
exit=2, stdout 0바이트

$ rhwp export-hwpx samples/2010-01-06.hwp out.hwpx --verify --json
exit=3, stdout 에 봉투 (verify.identical=false)
```

**구현 체크리스트:**

- [ ] 예외 계열 이름을 §3.3 표 그대로 쓴다. **시간 초과는 `RhwpTimeoutError`** —
      언어 내장 이름을 가리지 않는다.
- [ ] 예외에 `argv`·`exitCode`·`stderr`·`envelope` 를 싣는다.
- [ ] 재현용 명령 문자열 getter(`command`)를 둔다. **역슬래시를 따옴표보다 먼저
      이스케이프한다** — 순서가 뒤바뀌면 이미 넣은 이스케이프를 다시 이스케이프한다.
      (`errors.ts:190-201`. 파이썬은 이 처리가 없다 — [D-5](python_node_comparison.md))
- [ ] `UsageError` 에서 stderr 의 `힌트:` 줄을 `suggestion` 으로 꺼낸다.
      (`errors.py:112-119`, `errors.ts:130-140`)
- [ ] `UsageError` 에서 봉투의 `nextCall`(기계가 따라할 교정 호출)을 꺼낸다.
      (`errors.ts:145-151`. 파이썬에는 없다 — [D-8](python_node_comparison.md))
- [ ] opt-in 스위치를 둔다: `raise_on_verdict` / `throwOnVerdict` / 언어 관례.
      **기본값은 반드시 거짓.**
- [ ] 계획 선검증 위반은 exit 2 지만 `invalid` 가 있으면 값으로 돌려준다
      (§3.5). `invalid` 가 없는 exit 2 는 그대로 올린다.

---

## 6단계 — 봉투 래퍼: 원문을 바꾸지 않는다

[`parity_contract.md`](parity_contract.md) §2 가 권위다. 결정: **원문 키는 보존, 언어
관례는 별칭 조회 계층으로만.**

**구현 체크리스트:**

- [ ] `.raw` 는 CLI 가 낸 그대로. 직렬화하면 CLI stdout 과 같아야 한다.
- [ ] 조회는 원문 키·snake_case·camelCase 세 형태를 모두 받는다.
      (`models.py:47-61`, `envelope.ts:125-138`)
- [ ] **없는 필드는 조용한 `null` 이 아니라 예외.** 메시지에 **있는 필드 목록**을 함께
      싣는다 — 오타가 "값 없음"으로 둔갑하면 그게 가장 찾기 어려운 버그다.
      예외는 **오류 계열 안에** 둔다(Node 는 일반 `Error` 를 던진다 — [D-17](python_node_comparison.md)).
- [ ] `verify` 는 전용 타입으로 감싸고, **미요청은 `null`**. `null`(검증 안 함)과
      실패를 섞으면 검증하지 않은 저장을 통과로 읽는다.
- [ ] `verifyPages` 도 감싼다. 실측 봉투에 `{"after":6,"before":6,"identical":true}` 로
      들어 있는데 두 바인딩 모두 빠뜨렸다([D-10](python_node_comparison.md)).
- [ ] `changedPages` 는 `null`(모른다) 과 `[]`(바뀐 쪽 없다) 를 구분한다.
- [ ] **`verify` 객체에 진리값 훅을 두지 않는다.** `if (result.verify)` 가 언어마다 다른
      뜻이 되는 것이 [D-9](python_node_comparison.md) 다. `identical` 을 읽게 강제한다.
- [ ] `schemaVersion` major 를 대조하고 불일치면 `ProtocolError`
      ([`parity_contract.md`](parity_contract.md) §4.4-1. **두 바인딩 모두 아직 안 한다**).

---

## 7단계 — 이름 변환: 규칙을 코드로, 표는 만들지 않는다

```
약어 경계  ([A-Z])([A-Z][a-z])   →  $1_$2      # 고정 길이 (ReDoS 회피)
단어 경계  ([a-z0-9])([A-Z])     →  $1_$2
그 뒤 소문자화
```

`HTMLPage` → `html_page`, `pageCountA` → `page_count_a`, `irSchemaVersion` →
`ir_schema_version` (파이썬 규칙으로 실측 확인). 역변환은 `_` 분리 후 첫 글자 대문자화.

**수기 매핑표를 만들지 않는 것이 이 단계의 요점이다**(`_naming.py:3-6`, `naming.ts:4-6`):
사람이 이름을 다시 붙이기 시작하면 봉투에 필드가 하나 늘 때마다 바인딩이 뒤처지고,
어느 쪽이 맞는지 알 수 없게 된다.

**주의**: 일괄 변환 함수(`snake_keys`/`snakeKeys`)를 만들되 **내부에서 호출하지 않는다.**
두 바인딩 모두 그렇게 되어 있다(grep 결과 사용처 0). 호출자가 원할 때만 쓰는 도구다.

**예약어 회피**를 언어별로 둔다: 파이썬 `keyword.iskeyword` (`_naming.py:86-90`),
Node 는 34개 집합 (`naming.ts:131-141`), C# 은 `@` 접두, Swift 는 백틱.

---

## 8단계 — 타입 생성: 정적 타입 언어면 필수

동적 언어(파이썬)는 `Envelope` 하나로 충분했다. **정적 타입 언어는 다르다** —
사용자가 그 언어를 고른 이유의 상당 부분이 "필드 이름을 컴파일러가 확인해 준다"인데
`Dictionary<string, object>` 만 주면 그 값어치를 통째로 버린다
(`bindings/node/docs/DESIGN.md` D3).

### 두 출처

| 출처 | 서술하는 것 | 산출 |
|---|---|---|
| `rhwp export-ir-schema` | 문서 모델(IR) | IR 타입 |
| `rhwp capabilities` | 명령별 봉투(`recordFields`) | 봉투 타입 |

**어느 한쪽으로 다른 쪽을 흉내 내면 그 순간 수기 매핑이 부활한다**
(`bindings/node/tools/gen-types.ts:12-20`).

### 실측 — 출처가 실제로 무엇을 주는가

```
$ rhwp export-ir-schema --json
{ definitionCount: 41, dialect: "https://json-schema.org/draft/2020-12/schema",
  irSchemaVersion: "1.0", schema: {...}, schemaVersion: "1.0" }

$ rhwp capabilities        ← --json 을 받지 않는다! (실측: "알 수 없는 옵션: --json")
{ commands: [61개], exitCodes: {...}, jsonContract: {...}, formats, batch,
  tool: "rhwp", version: "0.8.2", schemaVersion: "1.0" }

commands[].recordFields 예 (render-diff):
  schemaVersion, mode, sourceA, sourceB, via, pageFilter, threshold,
  pageCountA, pageCountB, pageCountMismatch, maxDisp, worstPage,
  overPages, structPages, hardStructPages, status, regression, pages
```

`capabilities` 가 `--json` 을 거부한다는 점을 놓치지 말 것. 두 바인딩 모두 이 명령만
`--json` 을 붙이지 않는다(`commands.py:169-172`, `commands.ts:255-260`).

### 생성기 계약

- [ ] `--check` 모드: 생성 결과가 디스크와 다르면 **exit 1**. CI 가 "스키마는 바뀌었는데
      타입을 다시 안 만든 PR"을 잡는다. (`gen_models.py:14-18`, `gen-types.ts:29-32`)
- [ ] 생성물은 저장소에 커밋한다. 대가는 문서 세 곳에 "손으로 고치면 다음 재생성에서
      사라진다"를 적는 것.
- [ ] 끊어진 `$ref` 를 먼저 검사한다 — 생성기가 절반쯤 만들다 죽는 것을 막는다.
      (`schema.py:208-222` `dangling_references`)

---

## 9단계 — 1층: 무상태 명령 래퍼

### 어떤 명령을 감싸는가

[`parity_contract.md`](parity_contract.md) §5.3 이 기준이다. 실측 기준 **28개**:

```
batch  build-from-ingest  capabilities  convert  csv-to-table  digest
export-capabilities-schema  export-doclang  export-hml  export-hwpx
export-ir-schema  export-markdown  export-pdf  export-provenance-map
export-structure  export-svg  export-tables  export-text  extract-data
extract-pages  fields  info  inspect  ir-diff  render-diff  search
table-to-csv  thumbnail
```

여기에 `edit` → `fill_fields`/`replace_text`/`set_cell` 3개, `run` → 3층 `Plan`.
제외: `dump-pages`(진단), `category:internal` 5건, `mcp-serve`(2층이 담당).

### 플래그는 명령과 다른 기준으로 가른다

**`capabilities` 의 `flags` 는 "이 명령이 이 플래그를 파싱한다"만 말한다. `--json` 모드에서
무엇을 하는지는 말하지 않는다.** 이 PC 에서 재실측한 결과:

| 플래그 | 실측 | 처리 |
|---|---|---|
| `export-text -o`, `export-structure -o` | 무시(파일 안 생김) | 열지 않는다 |
| `export-tables -o` | stdout 이 `표 추출 완료: 12개 → …` 사람 문장으로 바뀐다 | 열지 않는다 |
| `export-capabilities-schema -o` | 봉투 유지(`output`·`bytes`) | 연다 |
| `convert -o`, `export-hwpx -o` | `알 수 없는 옵션: -o`, **exit 2** | **위치 인자로** |
| `render-diff --batch` | NDJSON 스트림 | 열지 않는다(반환 타입이 달라진다) |

**`convert`/`export-hwpx` 는 반드시 위치 인자다.** 파이썬이 `-o` 를 쓰고 있어 두 API 가
죽어 있다([D-1](python_node_comparison.md)). Node 쪽 구현을 따른다
(`commands.ts:538-552`, `:559-575`). `convert` 는 산출 경로가 **필수**이므로 프로세스를
띄우기 전에 `UsageError` 를 낸다.

### 그 밖의 체크리스트

- [ ] 모든 래퍼가 `--json` 을 붙인다. **예외는 `capabilities` 하나.**
      (파이썬은 스키마 명령 4곳에서도 빠뜨렸다 — [D-3](python_node_comparison.md))
- [ ] 검색어처럼 `-` 로 시작할 수 있는 값은 `--` 구분자 뒤에 넘긴다.
      (`commands.py:126-129`, `commands.ts:206-217`)
- [ ] 반복 가능한 값 플래그(`--font-path`)는 쉼표로 잇지 말고 **플래그를 반복**한다 —
      경로에 쉼표가 들어갈 수 있다. (`commands.ts:77-89`)
- [ ] 하위 명령이 있는 `inspect` 는 **CLI 순서(`검사`, `파일`)** 로 인자를 받는다.
      파이썬은 반대다([D-11](python_node_comparison.md)).
- [ ] `batch` 는 파일 목록을 stdin 으로 흘려 넣고 NDJSON 레코드를 돌려준다.
      빈 목록은 실행 전에 거부. (`commands.py:484-505`)

---

## 10단계 — 2층 세션, 3층 계획

### 2층 — `mcp-serve` stdio 클라이언트

- [ ] JSON-RPC 프레임을 줄 단위로 주고받는다. 응답 `id` 대조 필수.
- [ ] `hwp_open` → 핸들(`docId`) → `hwp_doc_*` 도구 → `hwp_close`.
- [ ] 언어의 자원 정리 문법을 쓰되 **멱등 `close()` 가 계약**이다
      (`session.py:210-236`, `session.ts:256-286`). 문법 설탕(`with`, `await using`,
      `IDisposable`)은 그 위에 얹는다.
- [ ] 이미 닫힌 핸들 재사용은 `SessionClosedError`.
- [ ] **`timeout` 과 `cwd` 를 둘 다 받는다.** 파이썬은 `cwd` 가, Node 는 `timeout` 이
      없다([D-14](python_node_comparison.md)).
- [ ] 정리 경로에서 새 예외를 만들지 않는다 — 원인 예외를 가리면 진단이 어려워진다.
      (`session.ts:396-407`)

### 3층 — 계획 빌더

- [ ] 체이닝 빌더 → 계획서 JSON(`planVersion`, `input`, `output`, `steps`, `assertions`,
      `dryRun`) → `run --plan-json <문자열> --json`. (`plan.py:172-187`, `plan.ts:196-221`)
- [ ] step 이 하나도 없으면 실행 전에 거부.
- [ ] `check()` 는 `dryRun: true` 를 실어 보낸다.
- [ ] **`check()` 전에 `capabilities` 로 `run` 의 `--dry-run` 지원을 확인한다.**
      없으면 던진다 — `run()` 으로 대체하면 검사인 줄 알고 문서가 편집된다.
      (`plan.ts:248-275`. 파이썬에는 없다 — [D-4](python_node_comparison.md))
      확인 결과는 캐시하고 테스트용 무효화 함수를 둔다.
- [ ] 선검증 위반(`invalid`)은 예외가 아니라 결과. `ok`·`violations`·`preview`·`steps`
      접근자를 둔다. (`plan.py:36-88`, `plan.ts:41-83`)
- [ ] 위반을 사람이 읽을 여러 줄로 만드는 `describe_violations` 를 둔다 — 로그와 오류
      메시지에 그대로 쓰인다.

---

## 11단계 — 테스트

[`parity_contract.md`](parity_contract.md) §6 이 층 구조를 정한다. 새 바인딩이 **처음부터**
갖춰야 하는 것:

### 11.1 바이너리 없이 도는 단위 테스트

기여자에게 Rust 툴체인을 요구하면 기여가 줄고, 줄어든 기여는 곧 뒤처짐이다
(`bindings/node/docs/DESIGN.md` D10). 탐색·변환·예외 매핑·계획 직렬화는 순수 로직이다.

**가짜 바이너리 픽스처의 함정 두 가지** (Node 가 문서화해 뒀다,
`test/helpers/fake-binary.ts:8-25`):

1. **인코딩.** 실물 rhwp(Rust)는 콘솔 코드페이지와 무관하게 항상 UTF-8 바이트를 낸다.
   픽스처가 플랫폼 기본 인코딩을 따르면 윈도우에서만 한글이 깨지고, 그 깨짐이 "바인딩
   버그"로 오인된다. **파이썬판이 이 함정에 두 번 걸렸다.**
2. **Windows 의 `.cmd` 래퍼.** `shell:false` 로는 `.bat`/`.cmd` 를 실행할 수 없다.
   Node 는 인터프리터 실행 파일 자체를 rhwp 로 삼고 스크립트를 첫 인자로 앞세운다.
   **새 언어도 같은 문제를 만난다.**
3. 시나리오 이름을 자유 문자열로 두지 않는다 — 오타가 `default` 분기로 떨어져 exit 1 이
   되면 "런타임 실패 경로가 잘 도네"라며 **테스트가 거짓 통과**한다.

### 11.2 통합 테스트 (바이너리 필요, 없으면 건너뜀)

파이썬 `pytestmark = pytest.mark.integration` + `binary_path` 픽스처가 없으면
`pytest.skip` (`tests/conftest.py:44-50`). Node `describe.skipIf(!hasBinary)`.

### 11.3 A층 — 자기서술 대조 (필수)

- **A-1 선언 → 래퍼**: `capabilities` 의 `json:true` 명령마다 대응 함수가 있는가.
  **수기 목록을 두지 않고 모듈이 실제로 내보내는 이름을 본다.**
  이름 변환도 바인딩 자신의 `to_camel` 을 써야 규칙이 두 벌이 되지 않는다.
  (`parity.integration.test.ts:76-111`. **파이썬에는 없고, 그래서 D-2·D-12 가 살아남았다.**)
- **A-2 래퍼 → 선언**: 반대 방향. 노출 기준 목록에 없는데 감쌌으면 실패.
- **A-3 옵션 → 플래그**: 조립한 플래그가 `capabilities.flags` 에 있는가.
  **이 테스트가 있었으면 D-1 이 머지 전에 잡혔다.** 구현하려면 인자 조립을 실행에서
  분리해야 한다 — 새 바인딩은 처음부터 그렇게 짠다.
- **A-4 종료 코드 사전**: 매핑하는 다섯 코드를 도구가 전부 설명하는가.

### 11.4 B층 — 교차 실행 골든 (신규)

같은 케이스를 CLI·기존 바인딩·새 바인딩으로 돌려 **봉투 원문을 바이트 비교**한다.
정규화 대상은 `source`·`output`(경로)뿐. `bytes`·`sizeBytes` 는 정규화하지 않는다 —
다르면 그게 결함이다. 케이스 목록은 **저장소 한 곳**에 둔다.

### 11.5 C층 — 오류·판정 매핑 대조 (신규)

[`parity_contract.md`](parity_contract.md) §6.3 의 9개 케이스. "예외인가 값인가" 열이
핵심이다 — 클래스 이름이 같아도 한쪽이 던지고 한쪽이 돌려주면 다른 결과가 된다.

---

## 12단계 — 패키징

| 항목 | Python | Node | 새 바인딩 지침 |
|---|---|---|---|
| 이름 | `rhwp` (PyPI) | `@rhwp/node` (npm) | 생태계 관례 |
| 버전 | `0.1.0` | `0.1.0` | rhwp 버전(`0.8.2`)과 **독립** |
| 라이선스 | MIT | MIT | MIT |
| **런타임 의존성** | **0** (`dependencies = []`) | **0** (`"dependencies": {}`) | **0 을 유지한다** |
| 개발 의존성 | 상한 고정 (`mypy>=1.0,<1.11`) | `devDependencies` | 상한을 건다 |
| 하한 | Python `>=3.8` | Node `>=18` | 실제로 검증한 최저만 |
| 바이너리 동봉 | 미구현 | 미구현 | §3 의 2번 경로 |

**런타임 의존성 0 이 규약이다**(`pyproject.toml:29-30`: "바인딩이 무거우면 '재포장'이
아니라 새 표면이 된다"). 서브프로세스 실행·JSON 파싱은 어느 언어든 표준 라이브러리에 있다.

**개발 의존성에 상한을 거는 이유**(`pyproject.toml:33-34`): 상한이 없으면 린터·타입체커의
정책 변경이 제품 CI 를 바꾼다.

---

## 13. 완료 판정 — 이걸 다 채우면 끝이다

- [ ] `bindings_foundation.md` §4 의 착수 조건이 충족됐다.
- [ ] [`parity_contract.md`](parity_contract.md) §5.3 기준의 명령 28개 + `edit` 3 + `Plan` 을 노출한다.
- [ ] §9 의 플래그 표를 그대로 따른다(`convert`/`export-hwpx` 는 위치 인자).
- [ ] §3.3 예외 계열 8종을 같은 이름으로 갖는다(시간 초과는 접두형).
- [ ] exit 3/4 가 기본 경로에서 예외가 아니다 — 실행으로 확인했다.
- [ ] `schemaVersion` major 대조가 있다.
- [ ] 계획 `check()` 에 `capabilities` 게이트가 있다.
- [ ] A층 4개 테스트가 CI 에서 돈다.
- [ ] B층 골든 러너에 새 언어가 등록됐다.
- [ ] C층 9케이스가 돈다.
- [ ] 런타임 의존성 0.
- [ ] `bindings/README.md` 에 항목을 추가하고, 이 문서의 §2 표에 열을 추가했다.
- [ ] [`python_node_comparison.md`](python_node_comparison.md) 에 3자 비교로 확장했거나,
      새 비교 문서를 만들고 상호 링크했다.

## 14. 관련 문서

- [`parity_contract.md`](parity_contract.md) — 지켜야 할 계약 (권위)
- [`python_node_comparison.md`](python_node_comparison.md) — 베끼면 안 되는 자리 목록
- [`README.md`](README.md) — 이 디렉터리의 지도
- [`bindings_foundation.md`](../bindings_foundation.md) — 표면 판단·착수 조건
- [`python_binding_guide.md`](../../manual/python_binding_guide.md) ·
  [`node_binding_guide.md`](../../manual/node_binding_guide.md) — 언어별 사용법
- [`agent_surface_playbook.md`](../../manual/agent_surface_playbook.md) — 표면 추가 절차
- `bindings/python/docs/DESIGN.md` · `bindings/node/docs/DESIGN.md` — 버린 대안 기록
- 로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M18~M20
