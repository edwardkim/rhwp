---
kind: guide
status: active
canonical: gym/docs/from_e2e.md
last_verified: 2026-08-18
---

# gym from_e2e — 스튜디오 e2e → CLI 과제 어댑터 규약

이 문서는 `gym/tools/from_e2e.mjs` 의 **정직 경계**, **예외 kind**, **계약
리터럴 파서**, **CSV 한 칸 편집**, **과제/기준 조립**을 고정한다. 작업 기록은
[`mydocs/working/gym_from_e2e.md`](../../mydocs/working/archives/gym_from_e2e.md) 를
본다. 시험 계약은 `gym/tools/from_e2e_contract.test.mjs` 와
`gym/tools/from_e2e_exceptions.test.mjs` 가 바이너리 없이 고정한다.

pack 안내(`gym/packs/studio-e2e/README.md`)는 ST01 한 과제의 온램프를 말한다.
이 문서는 어댑터 도구 자체다. 새 pack 을 만들지 않는다. 라이브 과제를 손으로
늘리지 않는다.

## 1. 왜 이 기둥이 필요한가

rhwp-studio 기여자는 편집을 브라우저 e2e 로 검증한다. gym 의 축은 CLI
능력이다. 두 세계가 단절되면 스튜디오에서 증명한 문서 계약이 운동장 집계에
안 오른다.

어댑터는 그 다리다. e2e 파일에 `export const gymContract = { ... }` 한 조각을
달면, CLI 로 채점 가능한 과제·기준·편집 CSV 를 기계 생성한다. 편집 CSV 는
사람이 쓰지 않는다. `chart-to-csv` 로 실제 차트를 뽑아 계약이 지정한 한 칸만
바꾼다. 형태 맞추기는 rhwp 자신에게 맡긴다 — gym 라이브 오라클과 같은 원리다.

온램프 이슈는 #4756, 순수 시험 강화는 #5236 이다.

## 2. 정직 조항 — CLI 로 재현 가능한 계약만

이 어댑터는 **문서 데이터 계약만** 파생한다. e2e 가 브라우저에서 검증하는
나머지 — 컨텍스트 메뉴, 더블클릭 다이얼로그, Ctrl+Z 스냅샷 undo, 무편집
무흔적, 비-차트 OLE 음성계약 — 은 CLI 로 표현할 수 없고 gym 의 축이 아니라서
파생하지 않는다.

거짓말하면 안 되는 문:

1. **e2e 파일을 실행하지 않는다.** `import` 도 `eval` 도 `Function` 도 쓰지
   않는다. 외부 PR 의 e2e 는 검토 환경에서 신뢰할 수 없다.
2. **허용 값은 객체·문자열·숫자뿐이다.** 식·배열·식별자·템플릿·boolean·null
   은 거부한다. 파서를 느슨하게 풀어 임의 코드를 계약으로 위장하지 않는다.
3. **허용 CLI 는 `chart-to-csv` 와 `csv-to-chart` 뿐이다.** 표 왕복
   (`table-to-csv`/`csv-to-table`), 필드 채움(`fill-fields`), 렌더·inspect
   는 다른 pack 의 축이다. 이 어댑터에 넣으면 차트 온램프가 아닌 것을
   차트 온램프로 위장한다.
4. **허용 검사 op 는 `file_exists` · `differs_from_input` · `value_eq`
   뿐이다.** 스크린샷 대조는 스튜디오 표면이다.
5. **`ok=false` 봉투를 성공으로 위장하지 않는다.** CLI 가 실패한 차트를
   편집 CSV 로 쓰면 샘플이 맞다고 거짓말하는 것이다.
6. **from 과 to 가 같으면 거부한다.** 무편집을 왕복으로 위장하지 않는다.
7. **치명 예외는 삼키지 않는다.** 사용자가 끊었는데 과제 JSON 을 쓰면
   거짓말이다.

`honestyPolicy()` 가 이 표를 기계로 돌려준다. 시험이 `executesE2e === false`,
`usesEval === false`, `usesFunction === false` 를 고정한다.

## 3. 사용

```bash
node gym/tools/from_e2e.mjs \
  --e2e rhwp-studio/e2e/issue-4694-chart-data-edit.test.mjs \
  --pack studio-e2e --id ST01 --bin target/debug/rhwp
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--e2e` | (필수) | gymContract 가 있는 e2e 경로. 저장소 루트 상대. |
| `--id` | (필수) | 과제 ID. `T01` · `ST01` · `AU13` 형식. |
| `--pack` | `studio-e2e` | 산출 pack. 보통 그대로 둔다. |
| `--bin` | `target/debug/rhwp` | `chart-to-csv` 를 부를 바이너리. |

산출:

- `gym/packs/<pack>/assets/<ID>-edit.csv`
- `gym/packs/<pack>/tasks/<ID>.json`
- `gym/packs/<pack>/reference/<ID>.json`

검증(라이브, 이 문서의 단위 시험이 아님):

```bash
python gym/tools/build_baseline.py --agent baseline --pack studio-e2e --bin <bin>
python gym/score.py --agent baseline --pack studio-e2e --bin <bin>
```

순수 시험(바이너리 없음):

```bash
node --test gym/tools/from_e2e_contract.test.mjs gym/tools/from_e2e_exceptions.test.mjs
python gym/tools/audit.py
```

## 4. gymContract 모양

허용되는 계약은 이것이다. 키가 더 있으면 정직 게이트가 `studio-only` 로
거부한다.

```js
export const gymContract = {
  sample: 'chart/세로막대형/묶은세로막대형.hwp',
  chart: 1,
  edit: { series: 0, point: 0, from: '4.3', to: '91.7' },
};
```

| 필드 | 형 | 제약 |
|---|---|---|
| `sample` | 문자열 | 비어 있지 않다. `samples/` 아래 상대 경로. `.hwp`/`.hwpx`. `..`·절대경로·NUL 금지. |
| `chart` | 정수 | 1 이상. |
| `edit` | 객체 | 배열·null 금지. |
| `edit.series` | 정수 | 0 이상. CSV 에서 헤더 다음 열 오프셋. |
| `edit.point` | 정수 | 0 이상. CSV 에서 헤더 다음 행 오프셋. |
| `edit.from` | 문자열 또는 숫자 | 현재 칸과 `String(from)` 으로 대조. |
| `edit.to` | 문자열 또는 숫자 | from 과 달라야 한다. |

`validateContract` 는 필드 형만 본다. `assertCliReproducibleContract` 가
경로 탈출·스튜디오 키·여분 키·from===to 를 추가로 막는다. `main` 은 둘 다
부른다. 파서 단위 시험은 `validateContract` 만 써서 extra 키 파싱을 허용한다.

## 5. 파서가 허용하는 것 / 거부하는 것

`parseContractLiteral` / `consumeContractLiteral` 은 JSON 이 아니라 **제한된
객체 리터럴**이다. 허용:

- 중첩 객체
- 작은따옴표·큰따옴표 문자열
- 식별자 키, 따옴표 키
- 유한 숫자(정수·소수·과학적 표기·음수)
- 줄 주석 `//`, 블록 주석 `/* */`
- 후행 쉼표
- 단순 escape: `\"` `\'` `\\` `\b` `\f` `\n` `\r` `\t` `\v`
- Unicode `\uAC00` 네 자리 hex

거부(메시지는 기존 시험이 정규식으로 고정):

| 입력 | 메시지 |
|---|---|
| 식별자 값 `someVar` | 객체·문자열·숫자 이외의 식은 허용하지 않는다 |
| 배열 `[...]` | 위와 같음 |
| 템플릿 `` `...` `` | 위와 같음 |
| `true`/`false`/`null`/`+1` | 위와 같음 |
| 최상위 문자열·숫자 | 최상위 값은 객체여야 한다 |
| 중복 키 | 중복 키 '…'는 허용하지 않는다 |
| 미닫힘 문자열 | 문자열이 닫히지 않았다 |
| 미닫힘 블록 주석 | 블록 주석이 닫히지 않았다 |
| `\x41` | 허용되지 않은 문자열 escape |
| `\uZZZZ` · `\u12` | 유효하지 않은 Unicode escape |
| 객체 뒤 `+ 1` · `; ...` | 뒤에 추가 식이 있다 |

객체 뒤 주석만 있으면 허용한다. `readContract` 는 `export const gymContract
= {` 표지를 찾은 뒤 객체만 소비한다. 파일 나머지 `throw` · `runTest()` 는
실행되지 않는다. `let gymContract` · `export let` 은 표지가 아니므로
`missing-contract` 다.

## 6. 예외 kind 카탈로그

모든 잡히는 실패는 `FromE2eError.kind` 를 가진다. `ERROR_KINDS` 와
`EXIT_BY_KIND` 가 문서·시험·코드의 같은 표다.

| kind | exit | 언제 |
|---|---|---|
| `missing-arg` | 2 | `--e2e` 또는 `--id` 없음. 값이 `--` 로 시작 |
| `parse-error` | 3 | 계약 리터럴 문법 |
| `missing-contract` | 3 | `export const gymContract` 표지 없음 |
| `validate-error` | 3 | 필드 형, from===to, 확장자 |
| `studio-only` | 3 | UI 키, 여분 키, 금지 CLI/op |
| `path-escape` | 3 | `..` · 절대경로 · NUL |
| `task-id-invalid` | 3 | ID 가 `[A-Z]{1,3}[0-9]{2}` 가 아님 |
| `task-id-conflict` | 3 | 다른 pack 이 같은 ID 를 가짐 |
| `type-error` | 3 | 원문·argv·CSV·edit 형 |
| `csv-mismatch` | 4 | 칸 값이 from 과 다름. 샘플이 바뀜 |
| `csv-missing-row` | 4 | point 행이 없음 |
| `csv-missing-col` | 4 | series 열이 없음 |
| `csv-shape` | 4 | 들쭉날쭉하거나 계열 열이 없음 |
| `missing-bin` | 5 | rhwp 실행 파일이 없음 |
| `cli-error` | 5 | 비정상 종료 또는 `ok=false` |
| `envelope-error` | 5 | chart-to-csv 출력이 봉투가 아님 |
| `timeout` | 5 | CLI 시간초과 |
| `missing-file` | 6 | e2e 파일이 없음 |
| `permission` | 6 | 읽기/실행 권한 |
| `os-error` | 6 | 그 외 OS/`ERR_*` |
| `decode-error` | 6 | 유니코드 |
| `json-error` | 6 | 과제 JSON 파싱 |
| `unexpected` | 1 | 카탈로그 밖 |

`classifyNodeError` 가 Node 예외를 이 표로 접는다.

| 조건 | context | kind |
|---|---|---|
| `FromE2eError` | 아무거나 | 그 kind 유지 |
| `ENOENT` | `cli`/`bin` | `missing-bin` |
| `ENOENT` | 그 외 | `missing-file` |
| `EACCES`/`EPERM` | 아무거나 | `permission` |
| `ETIMEDOUT` | 아무거나 | `timeout` |
| `status` 숫자 | 아무거나 | `cli-error` |
| `SyntaxError` | `json`/`envelope` | `json-error` |
| `SyntaxError` | 그 외 | `parse-error` |
| `TypeError`/`RangeError` | 아무거나 | `type-error` |
| `URIError` 또는 decode 메시지 | 아무거나 | `decode-error` |
| `ERR_*` 또는 `errno` | 아무거나 | `os-error` |
| 그 외 | 아무거나 | `unexpected` |

`wrapNodeError` 는 치명 예외를 감싸지 않고 다시 올린다.

## 7. 종료 코드

| exit | 묶음 | 의미 |
|---|---|---|
| 0 | 성공 | 과제·기준·CSV 를 썼다 |
| 1 | unexpected | 카탈로그 밖. 위장하지 않는다 |
| 2 | 인자 | 사용법을 고치면 된다 |
| 3 | 계약 | 파서·검증·정직 게이트·ID |
| 4 | CSV | 샘플과 계약이 어긋남 |
| 5 | CLI | 바이너리·봉투 |
| 6 | I/O | 파일·권한·디코드 |

`runAsCli` 는 치명 예외가 아니면 `{ ok, exit, report }` 를 낸다. CLI 진입점은
`process.exit(exit)` 한다. 시험은 `runAsCli` 를 직접 불러 프로세스를 죽이지
않는다.

## 8. CLI 봉투

`chart-to-csv --json` 은 순수 JSON 이다. 머리줄 strip 없이 `charts[0].csv` 를
쓴다. `extractChartCsvFromEnvelope` 규칙:

- 봉투가 객체가 아니면 `envelope-error`
- `ok === false` 이면 `cli-error`. CSV 가 있어도 쓰지 않는다
- `charts` 가 배열이 아니거나 비면 `envelope-error`
- `charts[0]` 이 객체가 아니거나 `csv` 가 빈 문자열이면 `envelope-error`

`parseChartToCsvStdout` 은 BOM 과 앞뒤 공백을 벗긴 뒤 JSON.parse 한다. HTML
· 부분 JSON · 숫자 · 문자열 JSON 은 봉투가 아니다.

단위 시험은 `execFileSync` 를 목으로 갈아끼운다. 실제 rhwp 를 부르지 않는다.

## 9. CSV 한 칸 편집

`applyCsvEdit(baseCsv, edit)`:

1. CRLF 를 LF 로 정규화하고 마지막 개행을 벗긴 뒤 줄로 가른다.
2. 각 줄을 쉼표로 가른다. 따옴표 CSV 를 구현하지 않는다. 차트 수치 CSV 는
   따옴표가 없다. 따옴표가 필요하면 이 어댑터의 축이 아니다.
3. 모든 행의 열 수가 같고, 헤더 폭이 2 이상이어야 한다. 아니면 `csv-shape`.
4. 데이터 행은 `rows[1 + point]`, 열은 `1 + series`(첫 열은 범주 라벨).
5. 행이 없으면 `csv-missing-row`. 열이 없으면 `csv-missing-col`.
6. 칸 값이 `String(from)` 과 다르면 `csv-mismatch`. 샘플이 바뀐 것이다.
7. 그 칸만 `String(to)` 로 바꾸고, 결과는 항상 LF 로 끝난다.

ST01 은 계열 0 · 값 0 · `4.3` → `91.7` 이다. e2e 의 SENTINEL 과 같다.

## 10. 과제와 기준

`buildTask` 가 만드는 검사 세 개:

1. `file_exists` `out.hwp` `minBytes: 1` — 산출물이 있다.
2. `differs_from_input` — 무편집 복사를 거부한다.
3. `value_eq` `changedCount == 0` — 같은 CSV 를 `csv-to-chart --dry-run` 으로
   재적용하면 이미 목표값이다.

`buildReference` 는 한 스텝이다.

```
csv-to-chart {input} --csv <asset> --chart N -o {sub:out.hwp} --json
```

`assertTaskChecksAreCliReproducible` / `assertReferenceIsCliReproducible` 이
이 모양에서 벗어나면 `studio-only` 로 막는다. 조립 함수가 나중에 렌더
명령을 넣어도 게이트가 잡아낸다.

## 11. 과제 ID

형식은 `TASK_ID_PATTERN` = `^[A-Z]{1,3}[0-9]{2}$` 이다. `T01` · `ST01` ·
`AU13` · `SE01` 이 통과한다. 소문자·세 자리 숫자·네 글자 접두는
`task-id-invalid`.

`assertTaskIdAvailable` 는 다른 pack 의 `tasks/*.json` 을 읽어 `id` 가
같으면 `task-id-conflict` 다. 같은 pack 의 기존 ID 는 허용한다(재생성).
`gym/packs` 가 없으면 통과한다. JSON 이 깨지면 `json-error` 로 접고 도구를
죽이지 않는 척 성공하지 않는다. `id` 필드가 없는 JSON 은 소유자가 아니다.

SE01 은 security pack 이 가진다. studio-e2e 에서 SE01 을 만들려 하면
거부한다. 기존 시험이 그 메시지를 고정한다.

## 12. 샘플 경로

`assertSafeSamplePath`:

- 빈 문자열 → `validate-error`
- NUL → `path-escape`
- 절대경로(POSIX 또는 `C:\`) → `path-escape`
- 경로 조각 `..` → `path-escape`
- 확장자가 `.hwp`/`.hwpx`(대소문자 무시)가 아니면 `validate-error`

한글 경로(`세로막대형/묶은세로막대형.hwp`)는 허용한다. ST01 이 그 경로다.

## 13. 스튜디오 전용 키

`STUDIO_ONLY_KEYS` 에 있는 키가 계약 어디든 있으면 `studio-only` 다.
목록을 느슨하게 풀어 브라우저 e2e 전체를 과제로 위장하지 않는다.

ui, menu, click, dblclick, doubleClick, undo, redo, hotkey, shortcut,
dialog, pageObject, selector, locator, snapshot, trace, playwright,
canvas, pointer, hover, contextMenu, keyboard, mouse, viewport,
screenshot, wasm, getChartDataByIndex, setChartDataByIndex, window,
document, browser, e2eOnly, studioOnly, ole, noTrace.

허용 키 밖의 임의 키(`meta`, `hint`)도 `studio-only` 다. 파서는 읽을 수
있어도 `assertCliReproducibleContract` 가 막는다.

금지 CLI: table-to-csv, csv-to-table, fill-fields, export-pdf,
export-png, export-svg, render, inspect, thumbnail, mcp-serve.

## 14. 예외 경로 — 도구가 죽는 자리 / 접는 자리

접는 자리(`runAsCli` 가 report 로 접는다):

| 자리 | 잡는 것 | kind |
|---|---|---|
| argv | 필수 인자 없음 | `missing-arg` |
| readContract | ENOENT | `missing-file` |
| locateGymContract | 표지 없음 | `missing-contract` |
| parse | 문법 | `parse-error` |
| validate / 정직 게이트 | 필드·키·경로 | `validate-error` / `studio-only` / `path-escape` |
| 과제 ID 스캔 | JSON 파싱 | `json-error` |
| 과제 ID 스캔 | 충돌 | `task-id-conflict` |
| chart-to-csv | ENOENT | `missing-bin` |
| chart-to-csv | status | `cli-error` |
| 봉투 | 모양 / ok=false | `envelope-error` / `cli-error` |
| applyCsvEdit | 칸·모양 | `csv-*` |
| 쓰기 | OSError | `os-error` / `permission` |

죽이는 자리(삼키면 거짓말):

- `SystemExit`
- `GeneratorExit`
- `fatal === true` 로 표지된 예외
- `ERR_WORKER_OUT_OF_MEMORY`

`KeyboardInterrupt` 에 해당하는 Node 신호는 프로세스 기본 동작에 맡긴다.
어댑터가 잡아 성공 보고를 만들지 않는다.

## 15. 순수 함수와 바이너리 경계

바이너리 없이 고정하는 함수:

- `parseContractLiteral` / `locateGymContract` / `readContract`(임시 파일)
- `validateContract` / `assertCliReproducibleContract` / `assertSafeSamplePath`
- `applyCsvEdit` / `splitCsvRows` / `joinCsvRows` / `assertCsvRectangular`
- `buildTask` / `buildReference` / `dumpJson`
- `extractChartCsvFromEnvelope` / `parseChartToCsvStdout`
- `materializeFromContract`
- `classifyNodeError` / `exceptionReport` / `exitCodeForKind`
- `parseFromE2eArgv` / `assertTaskIdFormat`
- `runAsCli`(목 `execFileSync`, `dryRun: true`)

바이너리가 필요한 자리:

- 실제 `chart-to-csv` 호출
- `build_baseline` / `score` 왕복

단위 시험은 후자를 부르지 않는다. 라이브 왕복은 pack README 의 admission
이다.

## 16. ST01 표본

출처 e2e: `rhwp-studio/e2e/issue-4694-chart-data-edit.test.mjs`

같은 코어: e2e 의 `getChartDataByIndex`/`setChartDataByIndex` 와 CLI
`chart-to-csv`/`csv-to-chart` 는 같은 native 를 구동한다. 그래서 데이터
계약이 CLI 로 충실히 왕복한다. UI 계약은 왕복하지 않는다.

과제 입력: `samples/chart/세로막대형/묶은세로막대형.hwp`
편집: 차트 1, 계열 0, 값 0, `4.3` → `91.7`

`assets/ST01-edit.csv` 첫 데이터 칸만 91.7 이다. 계열명·라벨·다른 값은
`chart-to-csv` 원본 그대로다.

## 17. 이 도구가 하지 않는 것

- 새 pack 을 만들지 않는다.
- 라이브 과제를 손으로 늘리지 않는다.
- e2e 전체를 공짜로 과제로 바꾸지 않는다.
- 표·필드·렌더 계약을 차트 어댑터에 섞지 않는다.
- 파서를 느슨하게 풀어 식을 허용하지 않는다.
- `ok=false` 를 성공으로 쓰지 않는다.
- 치명 예외를 삼켜 과제를 쓰지 않는다.
- Rust 를 포맷하지 않는다. 이 기둥이 만지는 것은 JS·문서뿐이다.

## 18. 관련 기둥

| 기둥 | 도구 | 질문 |
|---|---|---|
| 스튜디오 온램프 | `from_e2e.mjs` | e2e 데이터 계약을 CLI 과제로 파생하나? |
| pack 정합 | `audit.py` | 과제↔기준 짝, ID 고유인가? |
| 라이브 오라클 | `score.py` | 같은 CSV 재적용이 changedCount 0 인가? |
| 종점 무결성 | `discriminate.py` | 일 안 한 제출이 만점을 받나? |

어댑터는 과제를 낳는다. 감사기는 짝을 본다. 채점기는 라이브로 다시 계산한다.
어댑터가 스튜디오 UI 를 과제로 넣으면 채점기가 CLI 로 재현할 수 없어 거짓
만점이 된다. 그래서 정직 게이트가 먼저 막는다.

## 19. 시험이 고정하는 것

`node --test gym/tools/from_e2e_contract.test.mjs gym/tools/from_e2e_exceptions.test.mjs`

고정하는 축:

- 기존 파서 메시지(허용 리터럴, 식 거부, Unicode, 주석, 따옴표 키).
- SE01 충돌 · ST01 허용.
- applyCsvEdit 성공/불일치/행 부재. ST01 한 칸.
- buildTask/buildReference 명령 배열.
- 예외 kind 카탈로그와 exit.
- 치명 예외는 다시 올라온다.
- 스튜디오 키 전수 거부.
- 봉투 ok=false 는 cli-error.
- materialize 가 ST01 형태를 조립한다.
- runAsCli 인자 오류는 exit 2.

Rust 시험·`cargo fmt --all` 은 이 기둥의 범위가 아니다.

## 20. 실패 시나리오 표본

아래는 시험이 고정하는 최소 표본이다. kind 와 exit 가 표와 어긋나면
어댑터가 거짓말하는 것이다.

### 20.1 missing-arg

```
$ node gym/tools/from_e2e.mjs
from_e2e missing-arg [main] 필수: --e2e <경로> --id <과제ID> ...
```

exit 2. 파일을 읽기 전에 떨어진다.

### 20.2 missing-contract

e2e 에 `export const other = {}` 만 있으면 `missing-contract`, exit 3.
`let gymContract` · `export let gymContract` 도 같다. 표지는
`export const gymContract = {` 뿐이다.

### 20.3 parse-error

```js
export const gymContract = {
  sample: globalThis.process.exit(1),
};
```

`객체·문자열·숫자 이외의 식은 허용하지 않는다`. 파일을 실행하지 않았으므로
프로세스는 죽지 않는다. kind 는 `parse-error`, exit 3.

### 20.4 studio-only

```js
export const gymContract = {
  sample: 'chart/sample.hwp',
  chart: 1,
  edit: { series: 0, point: 0, from: '4.3', to: '91.7' },
  undo: 1,
};
```

파서는 `undo: 1` 을 숫자로 읽는다. 게이트가 `CLI 로 재현할 수 없는 스튜디오
키: undo` 로 거부한다. `undo: true` 는 파서가 먼저 식을 거부한다. 둘 다
과제를 쓰지 않는다.

### 20.5 path-escape

`sample: '../secret.hwp'` 는 `path-escape`. `samples/` 밖으로 나가 임의의
HWP 를 차트 샘플로 위장하지 않는다.

### 20.6 csv-mismatch

헤더+한 행 `,s1\nrow,4.3\n` 에 `from: '9'` 를 주면

`계약 불일치: (계열 0, 값 0) 현재 '4.3' ≠ from '9' — 샘플이 바뀌었다`

kind `csv-mismatch`, exit 4. 샘플을 고치거나 계약을 고친다. 칸을 억지로
덮어쓰지 않는다.

### 20.7 csv-missing-row / csv-missing-col

`point: 5` 인데 데이터 행이 하나면 `point 5 데이터 행이 없다`.
`series: 5` 인데 열이 하나면 `series 5 열이 없다`.
들쭉날쭉한 행은 `csv-shape`.

### 20.8 cli-error · envelope-error

```json
{"ok":false,"charts":[{"csv":",s1\nr,1\n"}]}
```

CSV 가 있어도 `ok=false` 이므로 `cli-error`. 성공으로 위장하지 않는다.

```json
{"ok":true}
```

`charts` 가 없으므로 `envelope-error`. HTML 이나 부분 JSON 도 같다.

### 20.9 missing-bin · permission

없는 바이너리는 `ENOENT` + context cli → `missing-bin`, exit 5.
권한 거부는 `EACCES` → `permission`, exit 6. 같은 ENOENT 라도 e2e 파일을
못 읽으면 `missing-file` 이다. context 가 없으면 거짓말이다.

### 20.10 task-id-conflict

studio-e2e 에서 `--id SE01` 은 security/SE01.json 과 충돌한다. 기존 시험
메시지는 `과제 ID 'SE01' 가 다른 pack에 이미 있다: security/SE01.json`.
같은 pack 의 ST01 재생성은 허용한다.

### 20.11 json-error

다른 pack 의 tasks/XX01.json 이 `{` 만 있으면 스캔이 `json-error` 로 접힌다.
깨진 파일을 무시하고 충돌 없음으로 통과하면 ID 고유성 감사가 뚫린다.

## 21. 생성 경로 의사코드

```
parseFromE2eArgv
  → assertTaskIdFormat
  → readContract          # 무실행 정적 파서
  → assertCliReproducibleContract
  → assertTaskIdAvailable
  → invokeChartToCsv      # 여기만 바이너리
  → materializeFromContract
       validate + 정직 게이트 + applyCsvEdit
       + buildTask + buildReference
       + 검사/기준 CLI 재현 게이트
  → write assets/tasks/reference
```

어느 한 칸에서 FromE2eError 가 나면 `runAsCli` 가 report 로 접는다.
치명 예외는 이 사다리를 건너뛰고 프로세스를 죽인다.

`dryRun: true` 는 마지막 쓰기만 생략한다. 시험이 임시 디렉터리에서 조립
결과만 본다.

## 22. 허용/거부 리터럴 빠른 표

| 원문 | 파서 | 게이트 |
|---|---|---|
| `{ sample: 'a.hwp', chart: 1, edit: { series: 0, point: 0, from: '1', to: '2' } }` | 허용 | 허용 |
| `{ sample: "\\uAC00/x.hwp", ... }` | 허용(가/x.hwp) | 허용 |
| `{ ..., // 주석 }` | 허용 | 필드에 따름 |
| `{ n: 1e3 }` | 허용 | extra 키 → studio-only |
| `{ hint: 'x', sample, chart, edit }` | 허용 | studio-only |
| `{ undo: 1, sample, chart, edit }` | 허용 | studio-only |
| `{ undo: true, ... }` | 거부(식) | 도달 안 함 |
| `{ sample: someVar }` | 거부(식) | 도달 안 함 |
| `{ sample: ['a'] }` | 거부(배열) | 도달 안 함 |
| `{ sample: \`a\` }` | 거부(템플릿) | 도달 안 함 |
| `{ sample: 'a.hwp' } + 1` | 거부(후행 식) | 도달 안 함 |
| `{ sample: '../x.hwp', chart, edit }` | 허용 | path-escape |
| `{ ..., from: '1', to: '1' }` | 허용 | validate-error |

## 23. ST01 검사 명령 고정

`buildTask` 의 세 번째 검사 cmd 는 이 순서다. 시험이 배열 동등을 고정한다.

```
csv-to-chart
{file:out.hwp}
--csv
gym/packs/studio-e2e/assets/ST01-edit.csv
--chart
1
--dry-run
--json
```

기준풀이 run:

```
csv-to-chart
{input}
--csv
gym/packs/studio-e2e/assets/ST01-edit.csv
--chart
1
-o
{sub:out.hwp}
--json
```

`--dry-run` 은 검사에만 있다. 기준풀이는 실제 `out.hwp` 를 쓴다. 둘 다
같은 CSV 를 쓴다. CSV 를 사람이 고치면 라이브 오라클이 깨지므로 어댑터가
`chart-to-csv` 산출에서만 만든다.

## 24. 자주 하는 실수

1. **e2e 를 import 해서 계약을 읽기.** top-level `runTest` 가 브라우저를
   기동한다. 무실행 파서만 쓴다.
2. **boolean 을 계약에 넣기.** 파서가 식을 거부한다. 숫자 `1` 도 게이트가
   스튜디오 키면 막는다. UI 플래그는 e2e 에만 둔다.
3. **표 과제를 이 어댑터로 만들기.** `table-to-csv` 는 금지 CLI 다.
   table-csv pack 의 온램프를 따로 둔다.
4. **from===to 로 "이미 맞는 샘플" 과제.** differs_from_input 이 영원히
   실패하거나, 무편집 복사가 통과한다. 게이트가 막는다.
5. **ok=false CSV 를 자산으로 쓰기.** 샘플이 차트가 아닌데 과제가 생긴다.
6. **깨진 과제 JSON 을 건너뛰기.** ID 충돌을 놓친다. json-error 로 멈춘다.

## 25. 변경 시 같이 고칠 것

카탈로그를 만지면 세 곳이 동시에 바뀌어야 한다.

- `gym/tools/from_e2e.mjs` 의 `ERROR_KINDS` / `EXIT_BY_KIND` / `STUDIO_ONLY_KEYS`
- `gym/docs/from_e2e.md` 이 표
- `gym/tools/from_e2e_exceptions.test.mjs` 의 카탈로그 동등 시험

하나만 고치면 문서가 거짓이거나 시험이 거짓이다.

파서 거부 메시지를 바꾸면 `from_e2e_contract.test.mjs` 의 기존 39건이
깨진다. 메시지를 바꿀 이유가 있으면 그 시험과 이 문서 5절을 같이 고친다.

## 26. 버전 메모

- 2026-08-14: ST01 온램프. studio-e2e pack.
- 2026-08-18: 순수 파서·조립 시험 39건 (#5241 원 커밋).
- 2026-08-18: 예외 kind · 정직 게이트 · 문서 (#5241 보강).

## 27. 기여자 체크리스트

새 e2e 에서 과제를 파생할 때:

1. 데이터 계약만 `gymContract` 에 적는다. UI 단언은 e2e 본문에 남긴다.
2. `sample` 은 `samples/` 아래 상대 경로, `.hwp`/`.hwpx` 다.
3. `from` 과 `to` 가 다르다. SENTINEL 값이 샘플에 실제로 있는지
   `chart-to-csv` 로 확인한다.
4. `--id` 가 다른 pack 과 겹치지 않는다. `audit.py` 가 나중에 다시 본다.
5. 생성 후 `build_baseline` + `score` 가 3/3 인지 확인한다. 이 문서의
   단위 시험은 그 왕복을 대신하지 않는다.

어댑터를 고칠 때:

1. 기존 파서 메시지 39건을 깨지 않는다.
2. kind 를 추가하면 `ERROR_KINDS` · `EXIT_BY_KIND` · 문서 6절 ·
   exceptions 시험 카탈로그를 같이 고친다.
3. 허용 CLI 를 늘리지 않는다. 새 왕복은 새 도구다.
4. `eval` / `new Function` / `import(e2e)` 를 넣지 않는다.
5. 치명 예외를 catch 에서 삼키지 않는다.

## 28. 봉투 JSON 표본

성공(시험이 목이 돌려주는 최소):

```json
{
  "ok": true,
  "charts": [
    { "csv": ",계열 1,계열 2,계열 3\n항목 1,4.3,2.4,2\n" }
  ]
}
```

실패(성공으로 쓰면 안 됨):

```json
{
  "ok": false,
  "error": "chart index out of range",
  "charts": []
}
```

`ok` 가 없고 `charts[0].csv` 만 있으면 추출은 허용한다. CLI 가 옛 봉투를
낼 수 있다. 명시적 `ok: false` 만 실패로 본다. `ok` 를 생략한 채 빈
charts 는 envelope-error 다.

## 29. ID 형식 보기

| ID | 통과 | 이유 |
|---|---|---|
| T01 | 예 | 한 글자 + 두 숫자. core-cli |
| ST01 | 예 | studio-e2e |
| SE01 | 예 | 형식은 맞다. 다른 pack 충돌은 별 검사 |
| AU13 | 예 | 세 글자 |
| st01 | 아니오 | 소문자 |
| ST1 | 아니오 | 숫자 한 자리 |
| ST001 | 아니오 | 숫자 세 자리 |
| TOOL01 | 아니오 | 접두 네 글자 |
| ST | 아니오 | 숫자 없음 |

충돌 검사는 형식 통과 뒤에 한다. 형식이 틀린 ID 는 스캔까지 가지 않는다.

## 30. 한 줄 요약

from_e2e 는 스튜디오 e2e 의 **차트 데이터 계약**만 CLI 과제로 옮긴다.
실행하지 않고, 식을 받지 않고, UI 를 위장하지 않고, CLI 실패를 성공으로
쓰지 않는다. 예외는 kind 로 접고, 치명 예외는 죽인다. 그것이 정직하다.
