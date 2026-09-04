---
kind: working
status: active
canonical: mydocs/working/gym_from_e2e.md
last_verified: 2026-08-18
---

# gym from_e2e — 예외 경로와 CLI 정직성 보강

Issue: #5236
PR: https://github.com/edwardkim/rhwp/pull/5241
Branch: `feat/gym-from-e2e-pure`
Date: 2026-08-18

## 1. 결론

`gym/tools/from_e2e.mjs` 의 계약 파서·CSV 한 칸 편집·과제 조립은 그대로 두고,
예외를 kind 카탈로그로 접으며 CLI 로 재현할 수 없는 스튜디오 계약을 게이트에서
막았다. 파서를 느슨하게 만들지 않았다. 식·배열·식별자·템플릿은 계속 거부한다.
e2e 파일을 실행하지 않는다.

이 가지는 새 PR 을 열지 않는다. 같은 브랜치에 이어서 밀어 #5241 을 키운다.

검증:

- `node --test gym/tools/from_e2e_contract.test.mjs gym/tools/from_e2e_exceptions.test.mjs`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (JS/문서만, 사용자 지시)

## 2. 배경

원 PR(#5241)은 `applyCsvEdit` · `buildTask` · `buildReference` 를 순수 함수로
분리하고, 파서 시험을 39건으로 늘렸다. 대비 `upstream/devel` 삽입은 약 444줄
이었다.

그 상태의 빈틈:

1. 모든 실패가 민 문자열 `Error` 다. 인자 오류와 CSV 불일치와 없는 바이너리가
   같은 종료 코드로 떨어진다.
2. `readFileSync` · `JSON.parse` · `execFileSync` 예외가 그대로 올라 스택만
   보인다. kind 가 없다.
3. `gymContract` 에 `undo` · `menu` · `locator` 를 넣어도 파서가 객체로 읽으면
   과제 JSON 이 나온다. CLI 로 재현할 수 없는 계약을 온램프로 위장한다.
4. `chart-to-csv` 가 `ok: false` 를 내도 `charts[0].csv` 만 있으면 편집 CSV 를
   쓴다. CLI 실패를 성공으로 위장한다.
5. 과제 JSON 이 깨지면 `assertTaskIdAvailable` 가 도구 전체를 죽인다.
6. 카탈로그가 코드에만 있어 문서·시험이 같은 표를 공유하지 않는다.

분류를 네 값으로 늘리거나 파서를 느슨하게 하면 기존 39건 계약이 깨진다.
그래서 기존 메시지는 유지하고, `FromE2eError.kind` 를 얹었다.

## 3. 한 일

### 3.1 도구

`gym/tools/from_e2e.mjs`

- `FromE2eError` — kind + extras. 기존 메시지를 유지한다.
- `ERROR_KINDS` / `EXIT_BY_KIND` — 문서·시험이 같은 표.
- `classifyNodeError` / `wrapNodeError` / `exceptionReport` — Node 예외를
  kind 로 접는다. ENOENT 는 context 가 cli/bin 이면 missing-bin.
- `isFatalException` — SystemExit · GeneratorExit · fatal 표지 · OOM 은
  감싸지 않고 다시 올린다.
- `assertCliReproducibleContract` — sample/chart/edit 만. 스튜디오 키·여분
  키·from===to·경로 탈출을 막는다.
- `assertSafeSamplePath` — `..` · 절대경로 · NUL · 비-hwp 확장자.
- `extractChartCsvFromEnvelope` / `parseChartToCsvStdout` — 봉투를 순수로
  읽는다. ok=false 는 cli-error.
- `invokeChartToCsv` — execFileSync 를 주입할 수 있다. 시험이 목을 쓴다.
- `materializeFromContract` — 이미 뽑아 둔 CSV 와 계약을 과제/기준으로
  조립한다. 바이너리를 부르지 않는다.
- `runAsCli` — 치명 예외가 아니면 `{ok, exit, report}`. 프로세스를 시험이
  죽이지 않는다.
- `assertTaskIdAvailable` — 깨진 JSON 은 json-error. id 없는 JSON 은
  소유자가 아니다.

### 3.2 시험

- 기존 `from_e2e_contract.test.mjs` 39건은 같은 메시지로 유지.
- `from_e2e_exceptions.test.mjs` — kind 카탈로그, 정직 게이트, 봉투,
  materialize, runAsCli, CSV 격자, 스튜디오 키 전수.

### 3.3 문서

- `gym/docs/from_e2e.md` — 정본 규약. 한국어.
- 이 파일 — 작업 기록.

## 4. 정직 게이트가 막는 것

스튜디오 e2e 는 데이터 계약과 UI 계약을 한 파일에 섞는다. 어댑터가 UI 를
과제로 옮기면 gym 채점기가 CLI 로 재현하지 못해 거짓 만점 또는 영구 실패가
된다.

막는 키(일부): undo, redo, menu, click, dblclick, dialog, locator,
playwright, wasm, getChartDataByIndex, screenshot, ole, noTrace.

막는 CLI: table-to-csv, csv-to-table, fill-fields, export-pdf, render,
inspect, mcp-serve.

허용 CLI: chart-to-csv, csv-to-chart.

허용 op: file_exists, differs_from_input, value_eq.

파서는 extra 키를 읽을 수 있다. 게이트가 막는다. 두 층을 섞지 않는다.
파서 시험은 extra 키를 허용한 채 validateContract 만 부른다. 생성 경로는
게이트를 반드시 지난다.

## 5. 예외 표 — 작업 메모

exit 묶음은 사람이 고치는 자리를 가리킨다.

- 2 인자: 사용법. 계약 파일이 없어도 먼저 떨어진다.
- 3 계약: e2e 를 고치거나 ID 를 바꾼다. 바이너리와 무관.
- 4 CSV: 샘플이 바뀌었거나 point/series 가 틀렸다. chart-to-csv 를 다시 본다.
- 5 CLI: 바이너리·봉투. 단위 시험은 목으로만 본다.
- 6 I/O: 파일 없음·권한.
- 1 unexpected: 카탈로그에 없는 실패. 성공으로 위장하지 않는다.

ok=false 를 envelope-error 가 아니라 cli-error 로 둔 이유: 봉투 모양은
맞는데 CLI 가 실패를 선언한 것이다. 모양 오류와 섞으면 고치는 자리가
달라진다.

## 6. 파서를 느슨하게 하지 않은 이유

boolean·null·배열을 허용하면 `undo: true` 같은 UI 계약이 파서를 통과한다.
게이트가 막더라도, 파서가 식을 받기 시작하면 `import.meta` · 함수 호출을
언제든 다시 열 유혹이 생긴다. 외부 PR e2e 는 실행하면 안 된다.

그래서 파서 층은 객체·문자열·숫자만 남긴다. 게이트 층은 그 객체가 CLI
차트 왕복인지를 본다.

Unicode `\uAC00` 은 허용한다. 한글 샘플 경로를 계약에 쓰기 위해서다.
`\u{AC00}` 코드포인트 문법은 허용하지 않는다. JSON/제한 리터럴 범위를
넘긴다.

## 7. 라이브 왕복과의 경계

이 작업은 바이너리를 부르지 않는다. ST01 라이브 admission 은 이미 pack
README 에 있다. 여기서 다시 `chart-to-csv` 를 실호출하면 CI 가 rhwp
빌드를 기다리게 되고, 순수 시험 기둥이 깨진다.

목을 쓰는 자리:

- `invokeChartToCsv({ execFileSync })`
- `runAsCli(..., { execFileSync, dryRun: true, cwd })`

dryRun 은 파일을 쓰지 않는다. 임시 디렉터리에 e2e 픽스처만 두고 조립
결과를 본다.

## 8. 하지 않은 것

- 새 pack, 새 라이브 과제, ST02 실파일을 만들지 않았다.
- 표/필드 어댑터를 이 파일에 넣지 않았다.
- 파서에 배열·boolean 을 넣지 않았다.
- Rust 를 포맷하거나 clippy 를 돌리지 않았다.
- 새 PR 을 열지 않았다.

## 9. 재현

```bash
node --test gym/tools/from_e2e_contract.test.mjs gym/tools/from_e2e_exceptions.test.mjs
python gym/tools/audit.py
git diff --shortstat upstream/devel
```

SIZE GATE: upstream/devel 대비 insertions >= 3000.

## 10. 남은 위험

1. 스튜디오 키 목록이 닫힌 집합이다. 새 UI 키(`drag`, `wheel`)가 생기면
   목록에 추가해야 한다. 여분 키 전면 거부가 그 구멍을 대부분 막는다.
2. 따옴표 CSV 를 구현하지 않았다. 차트 수치 CSV 가 따옴표를 내기 시작하면
   csv-shape 또는 칸 불일치로 실패한다. 그때 축을 넓힐지 다른 어댑터로
   보낼지 결정한다.
3. `charts[0]` 만 본다. 계약의 chart 인덱스는 csv-to-chart 인자로만 간다.
   다중 차트 샘플에서 chart-to-csv 가 요청한 차트만 내는지가 CLI 계약이다.
   어댑터는 그 첫 csv 를 믿는다.

## 11. 커밋 범위

- `gym/tools/from_e2e.mjs`
- `gym/tools/from_e2e_contract.test.mjs` (기존 39건 유지, import 불필요)
- `gym/tools/from_e2e_exceptions.test.mjs` (신규)
- `gym/docs/from_e2e.md` (신규)
- `mydocs/working/gym_from_e2e.md` (신규)

생성기 임시 파일은 커밋하지 않는다.

## 12. 함수별 메모

### 12.1 FromE2eError

기존 `throw new Error(msg)` 를 `throw new FromE2eError(kind, msg)` 로
바꿨다. `instanceof Error` 는 참이라 기존 `assert.throws(..., /정규식/)`
이 그대로 통과한다. extras 는 시험과 로그가 offset·owners 를 보게 한다.

### 12.2 consumeContractLiteral

본문은 거의 그대로다. `fail()` 만 FromE2eError 를 던진다. 공백·주석·문자열
·숫자·객체 루프를 건드리지 않았다. 39건이 그 계약을 지킨다.

### 12.3 readContract / locateGymContract

표지 정규식은 그대로 `export\s+const\s+gymContract\s*=\s*\{` 다. 파일
읽기만 `readTextFileOrThrow` 로 감싸 ENOENT 를 missing-file 로 접는다.

### 12.4 validateContract

객체가 아닌 계약에 대한 가드를 앞에 넣었다. 나머지 필드 검사는 메시지
그대로다. extra 키는 여기서 막지 않는다.

### 12.5 assertCliReproducibleContract

생성 경로의 정직 게이트. validate + 경로 + 스튜디오 키 + 허용 키 +
from!==to. materialize 와 mainWith 가 부른다. 파서 단위 시험은 부르지
않는다.

### 12.6 applyCsvEdit

형 검사와 직사각 검사, 열 부재를 앞에 넣었다. 불일치·행 부재 메시지는
그대로다. 열 부재는 예전엔 `undefined ≠ from` 으로 mismatch 가 났다.
지금은 `csv-missing-col` 이다. 칸이 있는데 값만 다른 경우와 칸이 없는
경우를 섞지 않는다.

### 12.7 봉투

`charts[0].csv` 만 믿는다. 계약의 chart 인덱스는 csv-to-chart 인자로
간다. 다중 차트에서 CLI 가 다른 차트를 첫 칸에 내면 그건 CLI 버그다.
어댑터가 인덱스를 다시 고르면 CLI 계약을 복제하는 셈이다.

### 12.8 runAsCli

CLI 진입점은 process.exit 한다. 내보낸 runAsCli 는 exit 코드를 돌려
시험이 프로세스를 유지한다. dryRun 은 쓰기를 생략한다. cwd 주입으로
임시 e2e 픽스처를 쓴다.

### 12.9 wrapNodeError 와 치명 예외

처음에 wrap 이 치명 예외를 감싸 버려 시험이 실패했다. wrap 이
isFatalException 이면 다시 던지도록 고쳤다. invokeChartToCsv 의
execFileSync 가 SystemExit 를 내도 과제 JSON 을 쓰지 않는다.

## 13. 시험 설계

기존 파일은 원 메시지 회귀다. 새 파일은 kind·게이트·봉투·CLI 목이다.
한 파일에 섞으면 39건 회귀와 신규 표가 뒤섞여 리뷰가 어려워진다.

스튜디오 키는 목록 전수 시험이다. 키를 추가하면 시험이 자동으로 는다.
빼면 시험이 줄어 리뷰에서 보인다.

CSV 격자는 3 계열 × 4 값이다. 한 칸만 바뀌고 나머지는 그대로인지를
모든 좌표에서 본다. ST01 한 칸 시험만으로는 오프셋 실수를 놓친다.

## 14. 크기와 범위

SIZE GATE 는 upstream/devel 대비 insertions >= 3000 이다. 원 PR 은
444 였다. 보강은 예외 카탈로그·정직 게이트·시험·한글 문서다. 라이브
pack 을 부풀리지 않았다. 과제 복제는 정직 게이트와 반대로 간다.

## 15. 리뷰어에게

볼 곳:

1. 파서 fail() 메시지가 기존과 같은가.
2. STUDIO_ONLY_KEYS / FORBIDDEN_CLI_COMMANDS 가 차트 왕복 밖으로
   새는가.
3. ok=false 를 성공으로 쓰는 분기가 있는가.
4. wrapNodeError 가 치명 예외를 삼키는가.
5. 새 pack 이나 ST02 실파일이 생겼는가. 생기면 이 작업의 범위를
   넘는다.

안 봐도 되는 곳: Rust, rustfmt, pack JSON, baselines.

## 16. 다음이 있다면

- 스튜디오 표 편집 e2e 가 CLI table-to-csv 로 왕복하면 **다른**
  어댑터. 이 파일에 명령을 추가하지 않는다.
- 다중 차트 봉투가 `charts[i]` 를 명시하면 extract 가 인덱스를
  받을 수 있다. 지금은 CLI 가 요청한 차트만 낸다고 가정한다.
- 키 목록을 닫힌 집합으로 둘지, 허용 키 외 전부 거부로 충분할지.
  지금은 둘 다 한다. 허용 키 외 거부가 더 강한 문이다.

## 17. 로컬에서 확인한 숫자

- 기존 계약 시험 39 passed
- 예외·정직 시험 포함 215 passed
- `python gym/tools/audit.py` — 18 pack 전부 통과, 위반 0
- 라이브 rhwp / cargo fmt --all 은 이 보강에서 실행하지 않음
