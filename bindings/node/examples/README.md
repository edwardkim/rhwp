# `@rhwp/node` 예제

돌아가는 코드로 쓴 계약 설명서다. 각 예제는 "이 기능을 어떻게 부르나"가 아니라
**"이 계약을 어기면 무엇이 조용히 틀리나"**를 보인다. 주석이 코드보다 긴 파일이 있는
것은 그래서다.

예제는 소비자와 같은 표면(`src/index.ts`)을 쓰되, 빌드 산출물이 아니라 소스를 직접
가리킨다(`import * as rhwp from '../src/index.js'`). `npm run build` 없이 바로 돌기
위해서이고, 덕분에 `npm run typecheck` 가 예제까지 함께 검사한다 — 예제가 낡으면
빌드가 깨진다.

## 실행 준비

```bash
# 1) rhwp 실행 파일을 찾을 수 있어야 한다.
#    탐색 순서: RHWP_BIN → 패키지 동봉(dist/_bin/) → PATH
export RHWP_BIN=/path/to/rhwp          # Windows PowerShell: $env:RHWP_BIN = "C:\path\rhwp.exe"

# 2) 예제 실행 (bindings/node 에서)
npx tsx examples/01-read-document.ts 문서.hwp
```

`RHWP_BIN` 을 첫 순위에 둔 것은 계약이다. 로컬에서 고친 rhwp 를 가리켰는데 패키지
동봉본이 가로채면 "왜 내 수정이 반영 안 되지"라는 진단 불가 상황이 생긴다.

## 종료 코드

예제는 rhwp 본체와 **같은 어휘**를 쓴다. 이 구분이 이 저장소 전체의 규약이다.

| 코드 | 뜻 | 예제에서 언제 |
| --- | --- | --- |
| `0` | 성공 | |
| `1` | 런타임 실패 | 문서에 누름틀이 없음, 스키마가 깨짐, 계획 검사 불가 |
| `2` | 사용법 오류 | 인자 부족, 이 rhwp 가 모르는 명령, 계획 선검증 위반 |
| `3` | **판정** 실패 | `verify` 불일치, 시각 회귀, 근거 없는 인용 |

`3` 이 핵심이다. **도구는 정상 동작했고 문서에 대한 단언이 틀린 것**이다. 그래서
판정 실패는 예외로 오지 않고 봉투의 필드(`verify`·`status`·`regression`)로 온다.
예외를 원하면 `throwOnVerdict: true` 를 명시한다 — 기본값이 아니다.

## 예제 목록

| # | 파일 | 실행 | 보이는 계약 |
| --- | --- | --- | --- |
| 01 | `01-read-document.ts` | `문서.hwp` | 봉투 조회 4종(`get`/`getOr`/`children`/`raw`)의 차이. `exportText({page})` 로 한 쪽만, `exportStructure` 의 `mode` 자동 판정 결과 읽기, `thumbnail({dataUri})` 가 파일 출력을 **대체**한다는 것 |
| 02 | `02-fill-form.ts` | `서식.hwp 제출본.hwp [폰트폴더]` | `verify` 의 `null`(검증 안 함)과 실패는 다르다. `changedPages` 의 `null`(모름)과 `[]`(없음)도 다르다. 제출용 `exportPdf({profile,backend,fontPath})` |
| 03 | `03-session-edit.ts` | `서식.hwp 결과.hwp` | 세션은 `try/finally` 로 반드시 닫는다(자식 프로세스가 남는다). 표 좌표는 **조회로 확인**하고 추측하지 않는다. 바뀐 쪽만 렌더 |
| 04 | `04-plan-runner.ts` | `서식.hwp 제출본.hwp` | 계획은 검사(디스크 무변경) → 원자 실행. `check()` 는 rhwp 가 `run --dry-run` 을 모르면 **예외를 던진다** — 조용히 실제 실행으로 내려가지 않는다 |
| 05 | `05-batch-pipeline.ts` | `폴더 [스레드수]` | 부분 실패는 예외가 아니라 레코드의 `error` 필드다. `batch<T>()` 에 타입을 주면 캐스팅이 필요 없다. `threads` 는 낮추는 쪽으로 쓰는 값 |
| 06 | `06-ir-schema.ts` | `[타입이름]` | 바인딩이 IR 모양을 하드코딩하지 않는 이유 — 생성 타입의 원천을 그대로 조회한다 **(M18 필요)** |
| 07 | `07-rag-index.ts` | `문서.hwp [검증할문구]` | 청크에 주소(쪽·제목)를 남겨야 인용을 검증할 수 있다. `digest({sections,maxChars})` 로 문맥 창을 관리한다 |
| 08 | `08-mail-merge.ts` | `서식.hwp 데이터.csv 출력폴더` | 한 행이 실패해도 루프를 죽이지 않는다. `verify` 를 요청했는데 보고가 없으면 그것도 실패다 |
| 09 | `09-convert-audit.ts` | `입력폴더 출력폴더` | 산출 경로는 **위치 인자**다(`exportHwpx`·`convert` 는 `-o` 를 모른다). `convert` 는 출력이 **필수**. `ir-diff` 의 `categories` 는 목록이 아니라 `{범주: 건수}` 맵이고, `{section}` 으로 범위를 좁힌다 |
| 10 | `10-browser-usage.ts` | `문서.hwp` | Node/브라우저 공용 코드. 공유 인터페이스는 **양쪽에서 같은 의미로 도는 것만** 약속한다 — 그래서 옵션이 없고, 없을 수 있는 필드는 `getOr` 로 읽는다 |
| 11 | `11-render-diff-gate.ts` | `문서.hwp [비교본.hwp]` | 시각 회귀는 예외가 아니라 `status`·`regression` 필드다. 회귀가 난 쪽만 잘라 최소 재현을 만든다 — `renderDiff` 의 쪽은 0 기준, `extractPages` 는 **1 기준** |
| 12 | `12-typed-envelopes.ts` | `문서.hwp` | 생성 타입이 주는 것과 주지 않는 것. `.raw` 는 타입을 보증하고 존재는 보증하지 않으며, `.get()` 은 그 반대다 **(4번 항목은 M18 필요)** |

## 예제가 만드는 파일

읽기만 하는 예제는 `01`·`06`·`10`·`12` 다. 나머지는 디스크에 쓴다.

| 예제 | 산출물 |
| --- | --- |
| 02 | `제출본.hwp`, 같은 이름의 `.pdf` |
| 03 | `결과.hwp`, 바뀐 쪽마다 `결과.pN.svg` |
| 07 | `문서.hwp.index.json` |
| 08 | `출력폴더/<이름>_NNNN.hwp` |
| 09 | `출력폴더/<이름>.hwpx` 또는 `.hwp` |
| 11 | 회귀가 있을 때만 `<이름>.pN.repro.hwp` |

## M18 이 머지되기 전에는 동작하지 않는 항목

`export-ir-schema` 는 아직 이 저장소의 rhwp 에 **없다**(M18 / #3762 미머지).
명령을 모르는 바이너리는 사용법 오류로 답하므로, 이 명령을 쓰는 지점은 **exit 2** 로
끝난다.

| 영향 | 증상 | 예제의 대응 |
| --- | --- | --- |
| `rhwp.irSchema()` / `rhwp.exportIrSchema()` | `UsageError` (exit 2) | `06` 은 안내 후 2 로 종료 |
| `12` 의 4)번 스키마 대조 | 같음 | 1~3 번(생성 봉투 타입)은 정상 동작하고, 4 번만 안내 후 2 로 종료 |

명령이 있는지 미리 보고 싶으면 도구의 자기서술을 읽는다 — 명령 표면의 단일 출처다.

```ts
const commands = (await rhwp.capabilities()).raw['commands'];
```

`src/ir.ts`·`src/envelopes.ts` 자체는 **커밋된 생성물**이라 M18 없이도 그대로 쓸 수
있다. 없는 것은 "지금 이 바이너리에게 스키마를 다시 물어보는" 경로뿐이다.

## 규약

새 예제를 더할 때 지키는 것들 — 기존 예제가 전부 이 모양이다.

- 파일 첫머리 블록 주석에 **목적 + 실행법**.
- 인자가 모자라면 사용법을 **stderr** 로 내고 `exit 2`. stdout 은 기계가 읽는 자리다.
- 판정 실패는 `3`, 런타임 실패는 `1`. 종료 코드를 하나로 뭉개지 않는다.
- 주석은 한국어로, **무엇이 아니라 왜**를 적는다. 특히 "이걸 안 하면 무엇이 조용히
  틀리는가"를 적는다.
- `null`(모름·미요청)과 `[]`·`false`(없음·실패)를 절대 섞지 않는다.
- 좌표(표·행·열)와 쪽 번호는 **조회 결과를 근거로** 쓰고 추측하지 않는다.
- 세션은 `try/finally` 로 정리한다.

검사는 두 가지로 한다. 둘 다 통과해야 한다.

```bash
node --experimental-strip-types --check examples/11-render-diff-gate.ts
npm run typecheck        # tsconfig.json 의 include 에 examples/**/*.ts 가 들어 있다
```
