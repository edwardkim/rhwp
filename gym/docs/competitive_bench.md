---
kind: guide
status: active
canonical: gym/docs/competitive_bench.md
last_verified: 2026-08-18
---

# gym 경쟁 벤치 — 순수 집계·평결·명시 예외

이 문서는 `gym/tools/competitive_bench.py` 의 **정직 경계**, **보고 봉투**,
**능력 매트릭스**, **집계·충실도·평결**, **명시 예외 코드**를 고정한다. 작업
기록은 [`mydocs/working/gym_competitive_bench.md`](../../mydocs/working/gym_competitive_bench.md)
를 본다. 시험 계약은 `scripts/tests/test_gym_competitive_bench.py` 와
`scripts/tests/test_gym_competitive_bench_exceptions.py` ,
`scripts/tests/test_gym_competitive_bench_extra.py` 가 바이너리 없이 고정한다.

packs·checks·coverage·robustness·profiles·README 는 이 기둥의 범위가 아니다.
새 CLI 플래그를 만들지 않는다. 기존 `--rhwp` · `--pyhwp` · `--soffice` ·
`--samples` · `--limit` · `--timeout` · `--out-json` · `--out-md` ·
`--from-json` · `--json` 이름만 쓴다.

## 1. 왜 이 기둥이 필요한가

"표준 도구 = 에이전트가 기본으로 집는 도구"라는 명제는 주장이 아니라
측정이어야 한다. 같은 `samples/` 파일에 같은 과제(본문 추출·메타·구조·변환)를
rhwp 와 대안 도구에 돌려, 성공률·중앙값 시간·간이 충실도를 재고, 문서화된
사실로 능력 매트릭스를 채운다.

시험이 얇으면 리더보드 비교가 흔들린다. 못 돌린 도구에 숫자를 싣거나,
실패한 실행의 0ms 로 중앙값을 끌어내리거나, 겹치지 않는 파일로 속도를
비교하면 채택 논거가 거짓이 된다. 이 기둥이 그 구멍을 막는다.

온램프 이슈는 #5229, 순수 함수·시험 강화는 #5239 이다.

## 2. 정직 조항

거짓말하면 안 되는 문:

1. **못 돌린 도구는 `available:false` + `reason` 이다.** `summary` ·
   `fidelityVsRhwp` · `overlapMs` · `runs` 를 싣지 않는다.
2. **medianMs / medianChars 는 성공 실행만 본다.** 실패의 짧은 시간으로
   속도를 꾸미지 않는다. `0` 은 빈 문서·즉시 반환이라 유효하고, `None` 만
   뺀다. `bool` 은 숫자가 아니다 (`True==1` 접힘 금지).
3. **충실도와 동일-집합 속도는 두 도구가 모두 성공한 파일만.** 겹침이 없으면
   `None` / `n=0` 이다. 기준 문자수가 0 이면 비율을 만들지 않는다.
4. **능력 매트릭스는 문서화·검증 가능한 사실만.** 값은 `yes` | `partial` |
   `no`. rhwp 행은 전 능력 `yes` 여야 한다.
5. **평결은 payload 숫자에서만 다시 유도한다.** 손글 승패를 `--from-json`
   재렌더가 남기지 않는다.
6. **깨진 입력은 숫자를 지어내지 않고 명시 오류로 멈춘다.** 파일 없음 ·
   UTF-8 아님 · JSON 깨짐 · 빈 스코어카드 · 알 수 없는 에이전트 · 봉투 형태.

`invented_metrics` / `payload_honesty_issues` / `require_honest_payload` 가
이 표를 기계로 돌린다.

## 3. 사용

라이브 스윕(이 문서의 단위 시험이 아님 — rhwp 바이너리가 필요하다):

```bash
cargo build --bin rhwp
python gym/tools/competitive_bench.py \
    --rhwp target/debug/rhwp --pyhwp .venv/Scripts/hwp5txt \
    --limit 25 \
    --out-json mydocs/tech/benchmark_vs_alternatives.json \
    --out-md   mydocs/tech/benchmark_vs_alternatives.md
```

재렌더(벤치 재실행 없음, 결정론):

```bash
python gym/tools/competitive_bench.py \
    --from-json mydocs/tech/benchmark_vs_alternatives.json \
    --out-md mydocs/tech/benchmark_vs_alternatives.md
```

순수 시험(바이너리 없음):

```bash
python -m unittest scripts.tests.test_gym_competitive_bench \
    scripts.tests.test_gym_competitive_bench_exceptions \
    scripts.tests.test_gym_competitive_bench_extra -v
python gym/tools/audit.py
```

`cargo fmt --all` 은 이 기둥의 범위가 아니다. Python·문서만 고친다.

## 4. 보고 봉투

`kind=gymCompetitiveBench`, `schemaVersion=1.0`. `assemble_payload` 와
`--from-json` 재렌더 JSON 이 같은 계약을 찍는다. 옛 JSON 에 kind 가 없어도
`stamp_report_contract` 가 채운다. 다른 kind 는 거절한다.

| 키 | 형 | 의미 |
|---|---|---|
| `kind` | str | 항상 `gymCompetitiveBench` |
| `schemaVersion` | str | 항상 `1.0` |
| `generatedAt` | str 또는 null | 측정 시각. 재렌더는 유지하거나 비울 수 있다. |
| `toolOrder` | str[] | 표 열 순서. 기본 `rhwp`, `pyhwp`, `soffice`, `hwplib` |
| `env` | object | OS·Python·rhwp 버전/프로파일·코퍼스 수·도구 가용성 |
| `tasks` | object[] | 과제별 도구 결과 |
| `capabilityMatrix` | object | 컬럼·행. 값은 yes/partial/no |
| `verdict` | str[] | 숫자에서 다시 유도한 평결 문장 |
| `scorecard` | object | 선택. 붙일 때만 요약. 평결 숫자를 바꾸지 않는다. |

`payload_shape_issues` 는 최소 형태만 본다. `tasks` 누락·배열 아님·잘못된
kind/schemaVersion. 정직성 가드(`payload_honesty_issues`)는 빈 tasks, 비가용
칸의 숫자, 깨진 run, 매트릭스 구멍까지 본다.

## 5. 코퍼스 선택

`select_corpus_paths` 는 파일시스템을 보지 않는다. POSIX 정규화 후
`.hwp`/`.hwpx` 만 취하고 형식별 정렬한다. `limit>0` 이면 형식별 앞 limit 개,
`limit<=0` 이면 전부. 중복 경로는 첫 등장만. `.txt` · `.md` · `.doc` 는
무시한다. 대소문자가 다른 경로는 별개다.

`discover_corpus` 는 `samples_dir` 의 바로 아래 `*.hwp`/`*.hwpx` 만 본다.
중첩 폴더는 넣지 않는다. 경로는 REPO_ROOT 상대 POSIX 로 내서 커밋 JSON 에
머신 절대경로가 새지 않게 한다. 임시 폴더 시험이 그 선택을 고정한다.

## 6. 집계

`summarize_runs(runs)`:

| 키 | 규칙 |
|---|---|
| `attempted` | 정규화한 run 수 |
| `ok` / `fail` | `ok` 가 참인 것만 성공 |
| `successRate` | `ok/attempted` 를 소수 3자리. attempted=0 이면 None |
| `medianMs` | 성공 run 의 숫자 ms 중앙값. 실패·bool·None 제외 |
| `medianChars` | 성공 run 의 숫자 chars 중앙값. 0 은 남긴다 |
| `byExt` | 형식별 attempted/ok/fail. ext 가 비면 file 에서 추론 |

`fidelity_vs_ref` 는 겹친 성공 파일의 `got/base` 중앙값이다. 기준 0 은 쌍을
만들지 않는다. `overlap_median_ms` 는 같은 파일집합의 (tool, ref) 중앙값이다.

## 7. 봉투 파서

info / structure / export-text 는 한 겹의 `data`/`result`/`payload` 만 벗긴다.
본문 표지(`pages`·`text`·`format`·`pageCount`·`nodeCount`·`structure`·
`sections`·`paraCount`)가 안쪽 객체에 있을 때만 래퍼로 인정한다.

| 함수 | 빈·깨진·배열 JSON | 인정하는 본문 |
|---|---|---|
| `parse_json_object` | None | 최상위 객체 |
| `parse_rhwp_info` | None | format/pageCount/sections/paraCount 중 하나. bool pageCount 는 None |
| `parse_rhwp_structure` | None | nodeCount/structure/mode 중 하나 |
| `parse_rhwp_text_chars` | None | pages[].text 합 또는 최상위 text. 비문자열 page 는 건너뜀. 빈 pages 는 0 |
| `parse_rhwp_info_fields` | None | 있는 스칼라만. extra 키는 버림 |
| `parse_rhwp_structure_nodes` | None | nodeCount 또는 nodes/sections/children 길이 |

바이트 stdout 은 UTF-8 `errors=replace` 로 읽는다. 이건 **측정 경로**다.
파일에서 읽는 경로(`utf8_decode`)는 깨진 바이트를 바꿔 넣지 않고
`encoding` 오류다. 두 길을 섞지 않는다.

## 8. 칸 렌더와 평결

`_fmt_cell`:

- 비가용 → `n/a: <이유>` (이유 없으면 `실행 불가`)
- attempted==0 또는 summary 가 객체 아님 → `n/a: 시도 없음`
- 그 외 → `{ms} · {rate%}({ok}/{att}) · 충실도 {fid}`. 없는 숫자는 `-`

파이프·개행은 `escape_md_cell` 이 `\|` 와 공백으로 바꿔 표를 지키다.

`verdict_lines` 가지:

| 조건 | 문장 |
|---|---|
| rhwp 가용 | `rhwp 는 export-text 에서 ok/att … 중앙값` |
| pyhwp 비가용 | `실행하지 않았다(이유)` 후 반환 |
| pyhwp 가용 | HWP5 ok/att, HWPX ok/att (구조적 한계) |
| overlap tool_faster | pyhwp 가 더 빨랐다 — 그대로 적는다 |
| overlap ref_faster | rhwp 가 더 빨랐다 |
| overlap tie | 중앙값이 같았다 |
| overlap n=0 | 겹친 파일이 없어 비교를 만들지 않았다 |
| soffice 비가용 | 이 머신에서 실행하지 않았다 |
| info/structure/convert 에 rhwp 만 가용 | 폭: … 과제는 rhwp 만 |
| 매트릭스 exclusive yes | 능력: 라벨… 는 rhwp 만 yes |

`refresh_verdict` 가 손글 승패를 덮어쓴다. `--from-json` 재렌더는 이 함수를
거친다.

## 9. 능력 매트릭스

컬럼 순서: 크로스플랫폼, 단일 자립 바이너리, 에이전트-네이티브 CLI, MCP,
메모리 안전, 검증 가능 작업, 편집, 렌더.

`validate_capability_matrix` 가 잡는 것:

- matrix/columns/rows 가 객체·비지 않은 배열이 아님
- column 에 key 없음, column 중복
- row 가 객체 아님, tool 없음, tool 중복
- 컬럼 키 누락, 값이 yes|partial|no 밖
- rhwp 행이 yes 가 아님

`exclusive_yes(matrix, "rhwp")` 는 라이브 매트릭스에서 mcp · verifiable ·
singleBinary · memSafe · agentCli 를 낸다. edit · render · crossPlatform 은
대안도 yes/partial 이라 독점 목록에 없다.

반환 행은 사본이다. 호출자가 고쳐도 다음 `capability_matrix()` 는 그대로다.

## 10. 예외 코드 카탈로그

모든 명시 실패는 `BenchError.code` 를 가진다. `ERROR_CODES` 와
`error_catalog()` 가 문서·시험·코드의 같은 표다. 기본 종료 코드는 2 다.
`BenchError` 가 아닌 예외는 `error_exit_code` 가 1 을 준다.

| code | 클래스 | 언제 |
|---|---|---|
| `missing-file` | `MissingFileError` | 경로 없음·빈 경로·디렉터리·읽기 OSError |
| `bad-json` | `BadJsonError` | 빈 문자열·잘린 JSON·트레일링 쉼표·최상위 비객체 |
| `encoding` | `EncodingError` | None·비바이트·UTF-8 디코드 실패. BOM 은 벗기고 통과 |
| `empty-scorecard` | `EmptyScorecardError` | packs/total 없음, 빈 packs, packsScored 0/null, 전부 unavailable, taskCount 0 |
| `unknown-agent` | `UnknownAgentError` | 빈 이름, 예약어, 경로 문자, 공백, 정규식 밖, 알려진 집합 밖 |
| `payload-shape` | `PayloadShapeError` | kind/tasks 형태, 비가용 칸의 숫자, 깨진 run |

`ERROR_KIND` 는 `gymCompetitiveBenchError` 다. `to_dict()` 는 `ok:false` 와
code/message/exitCode 를 내고, path 와 details 는 있을 때만 붙인다. 경로는
POSIX 로 정규화한다.

stderr 한 줄은 `오류[<code>]: <message> path=...` 이다. 코드가 앞에 있어
기계가 grep 할 수 있다.

### 10.1 에이전트 식별자

문법: `^[A-Za-z][A-Za-z0-9._-]{0,63}$`. 예약어
(`rhwp`·`pyhwp`·`soffice`·`hwplib`·`hancom`·`all`·`none`·`baseline` 등)는
식별자가 아니다. 도구 이름과 에이전트 이름을 표에서 섞지 않기 위해서다.

`discover_known_agents` 는 베이스라인·리더보드 스코어카드 폴더를 본다. 폴더
이름 `claude-fable-5-0000` 은 epoch 접미를 떼어 `claude-fable-5` 도 인정한다.
깨진 scorecard.json 은 그 파일만 건너뛰고 폴더 이름은 남긴다.

### 10.2 스코어카드

`kind=gymScorecard`, schemaVersion `1.0` 또는 `2.0`. 옛 카드에 kind 가 없어도
형태 오류로 치지 않는다. 측정이 하나도 없으면 `empty-scorecard` 다.

`attach_scorecard` 는 요약만 붙인다. `verdict` 숫자를 다시 쓰지 않는다.
채점 점수로 벤치 평결을 덮어쓰면 측정과 주장이 섞인다.

### 10.3 서브프로세스 실패 근사

`classify_cli_failure` 는 stderr 를 코드로 접는다. 숫자를 지어내지 않는다.

| 표식 | 코드 |
|---|---|
| timeout | `timeout` |
| not found / cannot find / 없는 파일 | `missing_input` |
| permission / 액세스가 거부 | `permission` |
| utf-8 / codec / decode | `encoding` |
| json | `bad-json` |
| 그 외 | `runtime` |

이 분류는 집계에 가짜 ms 를 넣지 않는다. run 의 `ok=false` 만 남긴다.

## 11. 문자열 로더와 경로 로더

두 함수를 갈라 둔다. 이름을 하나로 덮어쓰면 `--from-json` 시험이 깨진다.

| 함수 | 입력 | 실패 |
|---|---|---|
| `load_report_payload(raw)` | JSON 문자열 | `(None, issues)` |
| `load_report_from_path(path)` | 파일 경로 | `MissingFileError` / `BadJsonError` / `EncodingError` / `PayloadShapeError` |

CLI `--from-json` 은 기존처럼 파일을 읽어 **문자열 로더**를 부른다. 플래그
이름을 바꾸지 않았다. 경로 로더는 스코어카드·정직 가드 시험이 예외 코드를
고정하려고 둔 순수 입구다.

## 12. 순수 함수와 바이너리 경계

바이너리 없이 고정하는 함수:

- `median` / `summarize_runs` / `normalize_run`
- `fidelity_pairs` / `fidelity_stats` / `fidelity_vs_ref`
- `overlap_ok_pairs` / `overlap_timing` / `overlap_median_ms`
- `parse_json_object` / `parse_rhwp_info` / `parse_rhwp_structure` / `parse_rhwp_text_chars`
- `parse_rhwp_info_fields` / `parse_rhwp_structure_nodes`
- `validate_capability_matrix` / `exclusive_yes` / `capability_label`
- `_fmt_cell` / `escape_md_cell` / `speed_cmp`
- `export_text_verdict` / `soffice_verdict` / `width_verdict` / `verdict_lines`
- `select_corpus_paths` / `resolve_tool` / `rhwp_profile_from_path`
- `assemble_env` / `assemble_payload` / `stamp_report_contract` / `refresh_verdict`
- `unavailable_result` / `available_result` / `invented_metrics`
- `BenchError` 가족 / `utf8_decode` / `parse_json_text` / `load_json_object`
- `agent_id_issues` / `normalize_agent_id` / `require_known_agent`
- `scorecard_kind_issues` / `scorecard_emptiness_issues` / `load_scorecard`
- `payload_honesty_issues` / `require_honest_payload`
- `classify_cli_failure` / `error_catalog`
- `render_report` / `dump_payload_json` / `write_text_lf`
- `--from-json` 재렌더 (`main(["--from-json", ...])`)

바이너리가 필요한 자리:

- `bench_rhwp_*` / `bench_pyhwp_text` / `bench_soffice_text`
- 라이브 `build_payload`

단위 시험은 후자를 부르지 않는다. 라이브 스윕은 기존과 같이 rhwp 가 필요하다.

## 13. 이 도구가 하지 않는 것

- 새 CLI 플래그를 만들지 않는다.
- packs·checks·coverage·robustness 를 건드리지 않는다.
- 못 돌린 도구에 0% 나 0ms 를 싣지 않는다.
- pyhwp 가 더 빠른 사실을 숨기지 않는다.
- HWPX 0/N 을 "지원함"으로 바꾸지 않는다.
- 손글 승패를 재렌더에 남기지 않는다.
- 깨진 JSON 을 빈 요약으로 바꾸지 않는다.
- 예약된 도구 이름을 에이전트 식별자로 받지 않는다.
- Rust 를 포맷하지 않는다. 이 기둥이 만지는 것은 Python·문서뿐이다.

## 14. 관련 기둥

| 기둥 | 도구 | 질문 |
|---|---|---|
| 경쟁 벤치 | `competitive_bench.py` | 같은 과제에서 대안 대비 숫자와 능력이 정직한가? |
| pack 정합 | `audit.py` | 과제↔기준 짝, ID 고유인가? |
| 손상 강건성 | `robustness.py` | 손상 입력에 rhwp 가 패닉·행 하나? |
| 종점 무결성 | `discriminate.py` | 일 안 한 제출이 만점을 받나? |

벤치는 도구 비교다. 감사기는 pack 짝을 본다. 두 축을 한 숫자에 섞지 않는다.

## 15. 시험이 고정하는 것

```bash
python -m unittest scripts.tests.test_gym_competitive_bench \
    scripts.tests.test_gym_competitive_bench_exceptions \
    scripts.tests.test_gym_competitive_bench_extra -v
```

고정하는 축:

- 중앙값·byExt·bool 제외·빈 실행.
- 충실도 겹침/0 기준/bool chars.
- 동일-집합 속도 n 과 None 쌍.
- info/structure/text 래퍼·빈 pages·비객체 page.
- 매트릭스 구멍·rhwp 전 yes·exclusive yes.
- n/a 칸에 숫자 없음. attempted=0 은 시도 없음.
- 평결: 더 빠름, 동률, 겹침 없음, HWPX 0/N, soffice 미가용, 폭.
- `--from-json` 재스탬프 kind=gymCompetitiveBench.
- 예외 코드 카탈로그와 to_dict.
- UTF-8 BOM 통과, 깨진 바이트는 encoding.
- 빈/없는/디렉터리 경로는 missing-file.
- 예약 에이전트·경로 문자·공백.
- 빈 스코어카드와 agent 불일치.
- 비가용 칸에 숫자를 실으면 payload-shape.

Rust 시험·`cargo fmt --all` 은 이 기둥의 범위가 아니다.

## 16. 실패 시나리오 표본

아래는 시험이 고정하는 최소 표본이다. code 가 표와 어긋나면 하네스가
거짓말하는 것이다.

### 16.1 missing-file

경로 `""` · `None` · 없는 파일 · 디렉터리. `read_bytes` /
`load_report_from_path` 가 `MissingFileError` 를 낸다.

### 16.2 bad-json

`""` · `"   "` · `"{"` · `'{"a": 1,}'` · 최상위 배열. `parse_json_text` /
`require_json_object` 가 `BadJsonError` 를 낸다. 문자열 로더
`load_report_payload("{")` 는 예외 대신 `(None, issues)` 다. CLI 재렌더가
exit 2 와 `깨졌다` 를 stderr 에 남기는 경로와 같다.

### 16.3 encoding

`utf8_decode(None)` · `utf8_decode(12)` · `b"\xff\xfe"`. BOM 이 붙은
`{"ok": true}` 는 통과한다. latin-1 `café` 파일은 encoding 이다.

### 16.4 empty-scorecard

kind 만 있는 객체, packs=[], packsScored=0, 전부 unavailable, 모든
taskCount=0. `load_scorecard` 가 `EmptyScorecardError` 를 낸다.

### 16.5 unknown-agent

`""` · `rhwp` · `a/b` · `bad agent` · `1agent` · 알려진 목록 밖의 이름.
스코어카드 agent 와 expected 가 달라도 이 코드다.

### 16.6 payload-shape

`kind=other`, 빈 tasks 를 `require_honest_payload` 에 넣을 때, 비가용 칸에
`summary`/`runs` 를 실을 때.

## 17. 재현

```bash
python -m unittest scripts.tests.test_gym_competitive_bench \
    scripts.tests.test_gym_competitive_bench_exceptions \
    scripts.tests.test_gym_competitive_bench_extra -v
python gym/tools/audit.py
git diff --shortstat upstream/devel
```

SIZE GATE: upstream/devel 대비 insertions >= 3000.

## 18. 남은 위험

1. 라이브 스윕은 여전히 rhwp 바이너리와 선택적 pyhwp/soffice 가 필요하다.
   단위 시험이 그 경로를 대체하지 않는다.
2. 충실도는 문자수 비율이다. 표 셀을 `<표>` 로 바꾸는 손실은 잡지만, 글자
   순서가 다른 동량은 1.0× 로 남을 수 있다. 구조 대조는 `ir-diff` 축이다.
3. 스코어카드 schema 2.0 의 추가 키(`packsUnavailable`)는 요약에 안 올린다.
   벤치 평결에 채점 점수를 섞지 않으려는 경계다.
4. `discover_corpus` 는 한 단계 glob 이다. `samples/chart/` 아래 파일은
   라이브 기본 선택에 안 들어간다. 재귀로 바꾸면 코퍼스 크기가 달라져
   과거 JSON 과 비교가 깨진다.
