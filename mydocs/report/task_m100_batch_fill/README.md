---
kind: report
status: active
canonical: mydocs/report/task_m100_batch_fill/README.md
last_verified: 2026-08-02
---

# #3719 §6-6 처리 기록 — `batch fill` (서식 1 + 데이터 N행 → 산출 N개)

## 문제

`edit fill-fields` 는 서식 1 → 산출 1 이다. 100명분 통지서를 만들려면 에이전트가
같은 도구를 100번 부르고, 그 사이 "몇 번째까지 했는지"를 스스로 들고 있어야 한다.
중간에 한 건이 실패하면 어디서 끊겼는지도 호출자의 기억에만 남는다. 메일머지는
문서 도구의 기본 수요인데 rhwp 에는 그 축이 없었다.

한편 `batch` 축은 이미 있다. 하지만 기존 7축의 입력은 **파일 경로 목록(stdin)** 이다.
메일머지의 입력은 경로가 아니라 **행**이다 — 같은 문서를 N번 여는 대신, 한 서식에
N개의 값 묶음을 붙인다. 이 차이를 뭉개면(예: 행마다 서식을 복사한 임시 파일을 만들어
stdin 으로 흘리기) 임시 파일 관리라는 새 실패 축이 생긴다.

## 구현

### 1. 채움 로직은 새로 쓰지 않았다 — 코어를 갈랐다

`edit_fill_fields` 를 둘로 나눴다(`src/main.rs`).

- `fill_fields_core(file_path, data, out_path, dry_run, verify) -> Result<FillOutcome, String>`
  — 채움의 **단 하나의** 구현. 종전 `edit_fill_fields` 본문 그대로다.
- `edit_fill_fields(args)` — 인자 파싱 + 사람용/JSON 출력만 남은 껍데기.

배치가 새 편집 경로를 갖지 않는 것이 요점이다. 순번 지목(`이름[N]`, #3476)·모호성
보고(`ambiguous`)·혼동 이름 경고(`confusable`, #3707)·형식 보존(#3383)·저장 직후
자기검증(#3702)·`changedPages`(#3712)가 두 곳으로 갈라지면, 단건으로 검증한 서식이
배치에서 다르게 채워지고 그 차이는 **산출물 N개가 나온 뒤에야** 드러난다.

실패 표현만 바꿨다. 종전에는 `eprintln!` + `return EXIT_RUNTIME` 이었는데, 배치는
뒤 행이 남아 있어 프로세스를 끊을 수 없다. 그래서 코어는 `Err(사람이 읽는 사유)` 를
돌려주고, 단건은 stderr + exit 1 로, 배치는 그 행의 `error` 레코드로 바꾼다.

### 2. 스트리밍 기계도 새로 쓰지 않았다 — 공유했다

`run_batch` 안에 인라인돼 있던 "작업 간 병렬 + 한계 재정렬 버퍼" 를
`batch_stream_records(n, threads, make, out) -> BatchStreamTally` 로 뽑았다.
작업 단위가 무엇인지는 `make: Fn(usize) -> Value + Sync` 가 정하고,
순서 보존·역압(back-pressure)·broken pipe 중단·종료 코드 집계는 두 축이 공유한다.

- `run_batch`: `make = |idx| batch_record(mode, &paths[idx])` (경로)
- `run_batch_fill`: `make = |idx| batch_fill_record(form, idx, &rows[idx], …)` (행)

종료 코드 집계는 `BatchStreamTally::exit_code()` 한 곳으로 모았다 —
`error` 있으면 1 > `verifyPages` 불일치 4 > `verify` 차이 3 > 0.

### 3. `fill` 축은 파싱부터 갈랐다

`run_batch` 최상단에서 `fill` 이면 바로 `run_batch_fill(&args[1..])` 로 넘긴다.
stdin 경로 목록 읽기를 **절대 타지 않게** 하는 것이 목적이다. MCP 서버가 자식을
띄울 때 stdin 을 null 로 닫으므로(`mcp_serve::run_cli_tool`) 읽으려 들면 그 자리에서
빈 목록이 되는데, 그 경로 자체를 없앴다.

### 4. 산출 이름 — 미리, 결정적으로, 덮어쓰지 않게

`batch_fill_output_paths` 가 **한 바이트도 쓰기 전에** 전 행의 경로를 정한다
(`batch convert` 의 사전 충돌 점검과 같은 규약).

- `--name-field` 값 → `sanitize_output_stem` → 비면 1 기준 순번(`0001`, 최소 4자리)
- 금지 문자(`< > : " / \ | ? *`)·제어 문자는 `_` 로 치환 → 구분자가 사라지므로
  `../../탈출` 같은 값도 `--out-dir` 밖으로 나갈 수 없다
- Windows 가 조용히 잘라내는 이름 끝 공백·점 제거, 예약 장치 이름(CON·NUL·COM1…)은
  앞에 `_`
- 80자 상한(경로 260자 한도 여유)
- 중복은 소문자 키로 판정해 `_2`·`_3` 을 붙인다 — 대소문자만 다른 이름도 한 파일이
  되는 파일시스템에서 **Linux CI 는 통과하고 사용자만 데이터를 잃는** 사고를 막는다

미리 계산하는 두 번째 이유는 병렬 실행에서도 이름이 행 순서만으로 정해지기 때문이다.
`--threads 1` 과 `--threads 8` 의 결과가 같아야 한다.

### 5. 데이터 읽기 — JSONL / CSV

- `.jsonl`·`.ndjson`: 한 줄 한 객체. 빈 줄은 건너뛴다.
- `.csv`: 첫 줄 헤더 = 누름틀 이름. `parse_csv_records` 는 RFC 4180 그 자체만 구현한다
  (따옴표 안의 쉼표·줄바꿈·`""`, CRLF/LF). 전용 crate 를 새로 들이지 않았다.
- UTF-8 BOM 제거 — 엑셀 저장본을 남겨 두면 첫 헤더가 `BOM+이름` 이 되어 그 열이
  통째로 `notFound` 가 된다(오류 없이 한 칸 빈 문서가 나온다).

**행의 결함은 파일 전체의 실패가 아니다.** 깨진 JSONL 줄·헤더와 칸 수가 다른 CSV 행은
`FillRow::Broken` 으로 들고 가서 `error` 레코드가 된다. 반면 확장자·헤더 중복·빈 헤더·
닫히지 않은 따옴표·0행은 **한 행도 처리하기 전에** 끝낼 입력 오류라 exit 2 다.

### 6. 표면 동기화 (드리프트 가드)

- `capabilities`: `batch.subcommands` += `fill`,
  `batch.flags` += `--form`·`--name-field`·`--dry-run`,
  `commands[batch].flags` 동일 갱신(축 선언과 항목 선언이 어긋나면 안 된다),
  `batch.input`·`batch.output`·`batch.mcp.available` 에 fill 의 다른 입력 축 명시
- `capabilities --mcp`: `hwp_batch_fill` 신규 —
  `required: ["form","data","outDir"]`, 선택 3종은 `optionalArgs.when` 으로 배선
  (`nameField`→`--name-field`, `verify`→`--verify`, `dryRun`→`--dry-run`).
  `MCP_STDIN_TOOLS` 에는 **넣지 않았다** — stdin 을 읽지 않는다.
- `--help`: `batch fill` 전용 블록. "이 축만 stdin 을 읽지 않는다"를 못 박았다.

## 명령 계약

```
rhwp batch fill --form <서식.hwp|서식.hwpx> --data <행.jsonl|행.csv> --out-dir <폴더> --json
                [--name-field <필드>] [--verify] [--dry-run] [--threads <N>]
```

레코드(행마다 한 줄)는 단건 `edit fill-fields --json` 봉투 + `row` 다.

| 상황 | 레코드 | 종료 코드 |
| --- | --- | --- |
| 전 행 성공 | `{schemaVersion, source, row, dryRun, output, outputFormat, filledCount, filled, notFound, ambiguous, confusable, changedPages, verify}` | 0 |
| 서식에 없는 이름 섞임 | 위와 같되 `notFound:[…]` (**행 실패가 아니다**) | 0 |
| 행 파싱 실패·채움 실패 | `{schemaVersion, source, row, error, exitClass:"runtime"}` | 1 |
| 인자·데이터 형식 오류 | 없음(stdout 0바이트) | 2 |
| `--verify` 불일치 | `verify:{identical:false, diffCount:N}` (산출물은 남는다) | 3 |

`--dry-run` 은 `dryRun:true` 와 함께 **만들 예정** 경로를 `output` 에 담는다.
디스크에는 파일도 폴더도 만들지 않는다 — 선검증은 실행과 같은 명령줄에서 `--dry-run`
하나만 빼면 되는 것이라야 뜻이 있어서 `--out-dir` 는 이때도 요구한다.

## 실측 (evidence.txt 원문)

1. JSONL 3행 → 산출 3개, `--name-field 작성자` 값이 겹치자 `홍길동.hwp` /
   `홍길동_2.hwp` 로 갈렸다(덮어쓰기 0). exit 0, 19ms/32스레드.
2. 엑셀 CSV(BOM+CRLF+따옴표): `주식회사 가,나` · `줄1\n줄2` · `인용 "값"` 이 그대로
   들어갔고 `notFound` 는 비었다. `홍/길:동` → `홍_길_동.hwp`, 이름 필드가 빈 행은
   `0002.hwp`. `--verify` 는 두 행 다 `identical:true`.
3. 깨진 행 2개를 섞은 4행: 레코드 **4건 전부**(`row` 0·1·2·3), 산출 2개, exit 1.
   순번은 `0001`·`0004` 로 행 번호를 따라간다(실패 행이 순번을 당기지 않는다).
4. `--dry-run`: 레코드 3건 `dryRun:true`·`changedPages:null`, out-dir 자체가 생기지
   않음, exit 0.
5. stdin 에 쓰레기 2줄을 흘려도 결과 동일(3행 3산출) — 데이터는 파일에서만 온다.
6. 인자 오류 8종 전부 exit 2 / stdout 0바이트 / out-dir 미생성.
7. 축 스코프 6종 전부 exit 2 (`batch info --form|--name-field|--dry-run`,
   `batch fill --mode|--query|--verify-pages`).
8. 자기서술: `batch.subcommands` 에 `fill`, `batch.flags` 10종, `commands[batch].flags`
   동일, `hwp_batch_fill` 의 `required`/`props`/`args`/`optionalArgs` 전부 배선,
   `invocation.stdinTools` 에는 없음.

## 검증

- 신규 `tests/batch_fill_contract.rs` **25건 green**
  (기본 축 2 · CSV 2 · 이름 충돌·정규화 5 · 실패 전파 4 · 선검증/입력축/verify 3 ·
   인자 오류 1(8케이스) · 축 스코프 1 · 자기서술 4 · 부분 성공 2 · 재파싱 1).
- 무회귀: `batch_axes_contract` 17 · `cli_json_contract` 26 · `mcp_server_contract` 22 ·
  `edit_fill_fields_contract` 7 · `changed_pages_contract` 5 · `run_plan_contract` 8.
- `cargo clippy -- -D warnings` 0.
- fmt: 변경한 `.rs` 2개 `rustfmt --check` clean
  (이 PC 는 `cargo fmt --all` 이 os error 206 으로 불가 → 파일 단위 `newline_style=Auto`).

드리프트 가드는 기존 것 + 신규 자기서술 테스트가 모두 통과한다:
`capabilities_mcp_tool_definitions_contract`(required 배열) ·
`every_declared_input_property_is_wired_to_the_cli`(선언=배선) ·
`capabilities_covers_every_help_command` / `help_covers_every_capabilities_command` ·
`capabilities_declared_flags_are_real_cli_flags`(축↔항목 일치) ·
신규 `declared_batch_flags_are_accepted_by_some_axis`(선언한 플래그를 CLI 가 실제로 수용).

## 남은 것

- **매뉴얼**: `mydocs/manual/cli_commands.md` 의 `batch` 절에 fill 축을 추가해야 한다.
  이번 작업은 같은 저장소를 동시에 만지는 다른 작업과 충돌하지 않도록
  `src/main.rs` · 신규 테스트 · 본 보고서로 범위를 한정했다.
- **암호 문서**: `batch` 축 전체가 `--password` 계열을 거부한다(기존 규약). 암호 서식의
  메일머지는 batch 의 credential 전달 계약이 정의된 뒤의 일이다.
- **입력 형식 혼합**: 산출 형식은 서식 형식을 따른다(HWPX 서식 → HWPX 산출). fill 축에는
  형식을 바꾸는 스위치를 두지 않았다 — 형식 변환은 `convert`/`export-hwpx` 의 책임이다.
- **`--out-dir` 의 기존 파일**: 중복 해소는 이번 실행 안에서만 한다. 같은 폴더에 두 번
  실행하면 앞 실행의 산출물을 덮어쓴다(`batch convert` 와 같은 동작). 실행 간 보존이
  필요하면 별도 이슈로 다룬다.
