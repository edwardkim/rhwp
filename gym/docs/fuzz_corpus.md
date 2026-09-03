---
kind: guide
status: active
canonical: gym/docs/fuzz_corpus.md
last_verified: 2026-09-02
---

# gym 코퍼스 퍼징 발견 엔진 규약

이 문서는 `gym/tools/fuzz_corpus.py` 의 **결정적 손상 카탈로그**와 **예외
경로 계약**을 고정한다. 구현은 [`gym/tools/fuzz_corpus.py`](../tools/fuzz_corpus.py),
계약 시험은 [`scripts/tests/test_gym_fuzz_corpus.py`](../../scripts/tests/test_gym_fuzz_corpus.py)
다. 작업 기록은 [`mydocs/working/gym_fuzz_corpus.md`](../../mydocs/working/gym_fuzz_corpus.md)
를 본다.

`robustness.py` 는 릴리스 **게이트**다. 이 도구는 그 앞단의 **발견 엔진**이다.
무작위가 없고, 같은 입력은 같은 라벨·바이트를 낸다. 원본과 바이트가 같은
무의미 변형은 버린다. 없는 바이너리·빈 코퍼스·읽기 실패는 DoS 로 위장하지
않는다.

## 1. 왜 이 기둥이 필요한가

gym 의 세 기둥은 과제의 채점과 도구의 수명을 지킨다.

| 기둥 | 도구 | 질문 |
|---|---|---|
| 종점 무결성 | `discriminate.py` (#4808) | 일 안 한 제출이 만점을 받나? |
| 경로 무결성 | `trajectory.py` (#4810) | 마지막 스텝을 빼도 통과하나? |
| 도구 강건성 | `robustness.py` (#4814 / #5218) | 손상 입력에 rhwp 가 패닉·행 하나? |
| 발견 | `fuzz_corpus.py` (#4828 / #5256) | 전 코퍼스 × 다명령에서 고유 DoS 가 몇 곳인가? |

게이트는 바운드된 부분집합으로 "패닉·행 0"을 강제한다. 발견 엔진은 전
코퍼스를 여러 명령·여러 손상으로 exhaustive 하게 두들겨, 패닉을 **소스
위치(file:line)별로 묶어** "고쳐야 할 고유 버그 목록"을 낸다. 게이트가
회귀를 막고, 발견이 새 버그를 찾는다. 둘을 한 도구로 합치면 게이트가
느려지거나 발견이 부분집합에 갇힌다.

아무도 손으로 수백 문서를 수천 가지로 퍼징하지 않는다. 에이전트가 이걸
돌려 rhwp 를 계속 경화한다. 이 캠페인의 실제 DoS(렌더러·파서 오버플로·
무한루프·스택 오버플로)를 전부 이 엔진이 잡았다.

발견 엔진 자신도 예외 경로에서 죽지 않는다. 없는 바이너리, 빈 코퍼스,
읽을 수 없는 표본, 쓰기 실패, 프로브 시간초과·OS 오류는 분류해 보고하고
중단하지 않는다. 도구가 죽으면 CI 가 붉어져 하네스 결함과 rhwp DoS 를
구분할 수 없다.

## 2. 사용

```bash
python gym/tools/fuzz_corpus.py --bin target/debug/rhwp
python gym/tools/fuzz_corpus.py --bin <bin> --commands info,export-text
python gym/tools/fuzz_corpus.py --bin <bin> --limit 40 --workers 8 --json
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--bin` | (필수) | rhwp 바이너리. `gym.core.runner.find_bin` 으로 해석한다. |
| `--commands` | `info,export-text,export-structure,export-render-tree` | 쉼표구분 명령. 빈 토큰·중복은 버린다. |
| `--limit` | 0 | 표본 수. 0 은 전수. 음수·변환 불능은 0. |
| `--workers` | 8 | 병렬 워커. 비양수는 1. |
| `--timeout` | 10 | 프로브 초. 비양수는 프로브가 `invalid-timeout` 으로 거절. |
| `--json` | off | `gymFuzzCorpus` 봉투를 stdout 에 쓴다. |

새 플래그는 없다. `--samples-dir` 를 넣지 않는다. 코퍼스는 항상
`<repo>/samples` 다. 라이브러리 `fuzz()` 는 시험이 임시 디렉터리를 넘길
수 있게 `samples_dir` 인자를 받는다.

종료 코드:

| exit | 의미 |
|---|---|
| 0 | 패닉 클러스터 0 · 행 클러스터 0 · 도구 실패 없음. |
| 1 | 고유 패닉 또는 행이 하나라도 있다. |
| 2 | 바이너리를 찾지 못했거나 도구가 코퍼스를 쓰지 못했다. |

`ok` 는 패닉·행 부재 **그리고** `toolFailed` 가 거짓일 때만 true 다.
빈 코퍼스는 DoS 가 아니다 — `emptyCorpus=true`, `ok=true`, exit 0.
없는 바이너리는 DoS 가 아니다 — `missingBin=true`, `toolFailed=true`,
`ok=false`, exit 2. "DoS 0" 이라고 쓰면 거짓말이다.

읽기실패와 프로브오류는 **엔진 생존** 신호이지 rhwp DoS 가 아니다.
`ok` 를 뒤집지 않는다. 다만 모든 프로브가 `missing-bin` 이면 바이너리가
없는 것과 같으니 `toolFailed` 로 접는다.

## 3. JSON 봉투 (`gymFuzzCorpus` 1.0)

`kind=gymFuzzCorpus`, `schemaVersion=1.0`. 키 집합은 시험이 `REPORT_KEYS`
로 고정한다.

| 키 | 형 | 의미 |
|---|---|---|
| `kind` | str | 항상 `gymFuzzCorpus` |
| `schemaVersion` | str | 항상 `1.0` |
| `ok` | bool | 패닉·행이 없고 `toolFailed` 가 거짓일 때만 true |
| `samplesTested` | int | 실제로 고른 표본 수 |
| `totalSamples` | int | 디렉터리의 `.hwp`/`.hwpx`/`.hml` 총수 |
| `commands` | list[str] | 정규화된 명령 목록 |
| `mutantsPerSample` | int | 4KiB 정상 입력 기준 변형 수 힌트 |
| `runsChecked` | int | 프로브까지 도달한 실행 수 (변형 × 명령) |
| `distinctPanicSites` | int | `panicClusters` 길이. 고유 버그 수. |
| `panicClusters` | list[obj] | `{location, count, example}` 내림차순·위치 타이브레이크 |
| `hangClusters` | list[obj] | `{command, count, samples, example}` 내림차순·명령 타이브레이크 |
| `unreadables` | list[str] | 읽기/형태/변형 생성 실패 |
| `probeErrors` | list[str] | 쓰기 실패·OS 오류·프로브 예외 |
| `toolErrors` | list[str] | 바이너리 부재·코퍼스 나열 실패·스키마 위반 |
| `emptyCorpus` | bool | 표본 총수가 0 이고 코퍼스를 읽었다 |
| `missingBin` | bool | 바이너리 경로가 없거나 모든 프로브가 missing-bin |
| `toolFailed` | bool | 도구가 발견을 수행하지 못했다 |
| `inputShapes` | object | `empty`/`tiny`/`normal`/`huge` 카운트 |
| `exit` | int | 0 / 1 / 2. `resolve_exit()` 와 같아야 한다. |

`validate_report()` 가 이 계약을 다시 검사한다. 키가 빠지거나 `ok` 가
패닉·행·toolFailed 와 어긋나거나 `distinctPanicSites` 가 클러스터 수와
다르거나 `exit` 가 `resolve_exit` 와 다르면 위반 목록을 돌려준다.

패닉 클러스터의 `location` 은 `panicked at file:line` 캡처이거나
`stack-overflow` 이거나 `code{N}` 이다. 행 클러스터의 `command` 는
시간초과가 난 rhwp 명령이다. `samples` 는 그 명령에서 행이 난 원본
파일명의 정렬 유일 목록이다. `example` 은 `이름:라벨:명령` 태그 중
클러스터에 처음 들어온 것이다.

사람용 `format_human_report` 는 같은 사실을 빠뜨리지 않는다.

- 도구 실패: `코퍼스 퍼징: 도구 실패 — …`
- 빈 코퍼스: `코퍼스 퍼징: 빈 코퍼스 — 표본 0`
- 깨끗함: `코퍼스 퍼징: 샘플 A/B × 명령 C × N 실행 — DoS 0`
- DoS: `고유 패닉 X곳 · 행 클러스터 Y개` 와 `PANIC`/`HANG` 줄

`--json` 을 안 줘도 리뷰어가 클러스터를 본다.

## 4. 입력 형태

위치 기반 변형은 입력 길이에 따라 다르게 동작한다. 엔진은 표본을 네
형태로 접어 `inputShapes` 에 남긴다.

| 형태 | 조건 | 변형 동작 |
|---|---|---|
| `empty` | `n==0` | `empty-to-nul` 한 건만. 위치 기반 변형 없음. |
| `tiny` | `1 <= n <= 64` | 위치 기반은 가능하나 일부는 원본과 같아 버려진다. OLE 꼬리는 잘린 매직을 심는다. |
| `normal` | `65 <= n < 1MiB` | 전체 카탈로그. 크기 증가 변형 포함. |
| `huge` | `n >= 1MiB` | 크기 증가 변형(`splice-*`, `pad-eof`, `widen-gap`, `even-length-pad`)을 생략한다. |

형태 분류는 `classify_input_shape()` 가 맡는다. 비-바이트는 `TypeError`
다. 엔진은 그 예외를 `unreadables` 로 접고 다음 표본으로 간다.

게이트(`robustness.py`)는 `.hwp` 만 고른다. 발견 엔진은 `.hwp` · `.hwpx`
· `.hml` 을 고른다. HWPX 는 ZIP 이고 HML 은 텍스트다. 형식 축을 빼면
ZIP 조건부 가족과 HWP3 서명을 놓친다. 대소문자는 무시한다(`X.HWP` 도
표본이다). `.txt` 는 제외한다.

## 5. 결정성 규칙

1. 입력은 bytes-like 만 받는다. `str`/`int`/`None` 을 조용히 인코딩하지 않는다.
2. 같은 바이트열은 같은 `(라벨, 바이트)` 순서를 낸다. 난수·시각·entropy 없음.
3. `add()` 는 `mut == data` 이면 버린다. 무의미 변형이 프로브 예산을 먹지 않는다.
4. 라벨은 가족 안에서 유일하다. 같은 바이트가 나와도 라벨이 다르면 둘 다 남긴다.
5. 조건부 가족(ZIP/HWP3)은 매직이 있을 때만 켠다. 없을 때는 inject 쪽이 켠다.
6. 거대 입력은 크기 증가 변형을 건너뛴다. 발견은 헤더/절단/길이로도 충분하다.
7. 원 라벨(`truncN` / `flipN` / `biglenN`)은 유지한다. 이미 이 태그로 잡은
   재현체를 깨지 않기 위해서다. 확대 가족은 뒤에 덧붙인다.

표본 선택은 파일명을 정렬한 뒤 stride 로 `limit` 개를 뽑는다. `limit<=0`
이면 전수다. 디렉터리를 읽을 수 없으면 빈 목록을 돌려 엔진이 죽지 않게
한다. stride 반올림 중복은 순서를 유지한 채 제거한다. 같은 디렉터리·같은
limit 은 같은 목록을 낸다.

명령 목록은 `parse_commands` 가 정규화한다. 빈 토큰과 중복을 버리고
순서는 유지한다. 전부 비면 기본 명령으로 돌아간다. 기본을 비우면
"명령 0 × 실행 0 — DoS 0" 이 되어 거짓 음성이 난다.

워커 수는 클러스터 내용을 바꾸지 않는다. `workers=1` 과 `workers=4` 는
같은 `distinctPanicSites` 와 같은 location 을 낸다. `as_completed` 의
완료 순서는 비결정이지만, 클러스터 키는 집합이고 정렬 키는
`(-count, location|command)` 다. 예제의 태그는 "처음 들어온 것"이라
워커가 많으면 예제 문자열만 갈릴 수 있다. 위치와 건수는 갈리지 않는다.

## 6. 분류기

### 6.1 패닉 (`classify`)

기존 계약은 그대로다. 시험 `test_classify_distinguishes_panic_from_clean`
이 고정한다.

| 입력 | 결과 |
|---|---|
| `panicked at src/x.rs:42:9` | `("panic", "src/x.rs:42")` |
| `stack overflow` | `("panic", "stack-overflow")` |
| `code==101` (빈 출력) | `("panic", "code101")` |
| 음수 코드 (Windows AV) | `("panic", "code-1073741819")` |
| `code>=132` | `("panic", "code{N}")` |
| `code==1` 깨끗한 CLI 오류 | `(None, None)` |
| `code==0` 정상 | `(None, None)` |

`panicked at file:line` 정규식이 먼저다. 위치가 있으면 그 위치가
클러스터 키다. 위치가 없고 `stack overflow` 가 있으면 그 버킷. 그 다음이
어보트 코드다. 깨끗한 비-0 실패를 패닉으로 오판하지 않는다.

`err` 가 `None` 이거나 비문자면 빈 문자열로 접는다. `code` 가 int 가
아니면 `>=132` 분기를 타지 않는다. `101` 과 문자열 비교는 하지 않는다.

`is_panic_code` 는 `classify` 의 패닉 여부만 본다. hang 은 이 함수가
내지 않는다.

### 6.2 행

`classify_timeout(timed_out)` 이 true 일 때만 행이다.

- `True`
- `subprocess.TimeoutExpired`
- `TimeoutError`
- `OSError` 이면서 `errno` 가 `ETIMEDOUT` 또는 `ETIME`
- 소문자 표식: `timeout`, `timed out`, `time-out`, `time expired`, `deadline exceeded`

`False`/`None`/그 외 예외/`ValueError("timeout in name only")` 는 행이
아니다. 시그널 종료(`-9`)는 **패닉 쪽**이다. 일부 환경은 시간초과를
SIGKILL 로 돌려주지만, 이 엔진은 `TimeoutExpired` 만을 행의 권위로 둔다.
시그널을 행으로 접으면 실제 크래시가 행으로 위장된다.

`probe` 가 `TimeoutExpired` 를 받으면 `("hang", cmd)` 를 낸다. 버킷은
명령이다. 같은 명령의 행을 한 클러스터로 묶는다.

### 6.3 프로브 접기

`classify_probe_outcome(kind, bucket)`:

| 결과 | 조건 | 보고 |
|---|---|---|
| `hang` | `kind=="hang"` 또는 timeout 분류 | `hangClusters` |
| `panic` | `kind=="panic"` | `panicClusters` |
| `error` | `kind=="error"` 또는 bucket 이 예외 kind | `probeErrors` |
| `clean` | `kind in (None, "clean", "ok")` | 카운트만 |
| `error` | 그 외 | `probeErrors` |

우선순위는 행 > 패닉 > 오류 > 깨끗함 이다. 프로브가 timeout 을 이미
선언했기 때문이다. `permission` 같은 오류를 행 클러스터에 넣지 않는다.
넣으면 "고쳐야 할 DoS" 목록이 권한 오류로 더러워진다.

## 7. 예외 경로 — 엔진이 죽지 않는 계약

발견 엔진 자신이 예외로 죽으면 게이트 앞단이 "DoS 0" 이라는 거짓 음성을
못 내고, 반대로 CI 가 붉어져 rhwp 결함과 하네스 결함을 구분할 수 없다.
그래서 모든 I/O·형식·프로브 경로는 분류 가능한 문자열로 접힌다.

이슈 #5256 이 명시한 세 자리는 필수다.

| 경로 | 함수 | 접는 곳 | 보고 키 | exit |
|---|---|---|---|---|
| 없는 바이너리 | `find_bin_safe` / `probe` | `missing-bin` | `missingBin` · `toolFailed` · `toolErrors` | 2 |
| 빈 코퍼스 | `select_samples` / `fuzz` | 표본 0 | `emptyCorpus` | 0 |
| 읽기 실패 | `read_sample` | `(None, 이유)` | `unreadables` | (ok 유지) |

나머지 자리도 같은 규칙이다.

| 경로 | 함수 | 접는 곳 | 보고 키 |
|---|---|---|---|
| 비-바이트 입력 | `coerce_bytes` | `TypeError` | 호출 측. 엔진은 `unreadables` |
| 형태 분류 실패 | `classify_input_shape` | `TypeError` | `unreadables` |
| 한도/초/워커 변환 실패 | `normalize_*` | 0 또는 1 | 전수 / 프로브 거절 / 워커 1 |
| 디렉터리 읽기 실패 | `list_sample_names` | `(None, 이유)` | `toolErrors` 또는 `emptyCorpus` |
| 변형 생성 `TypeError` | `deterministic_mutants` | 엔진 루프 | `unreadables` |
| 변형 쓰기 실패 | `write_mutant` | 이유 문자열 | `probeErrors` |
| timeout <= 0 | `probe` | `invalid-timeout` | `probeErrors` |
| 빈 바이너리 경로 | `probe` / `fuzz` | `missing-bin` | `missingBin` |
| 빈 명령 | `probe` | `value-error` | `probeErrors` |
| `TimeoutExpired` | `probe` | `("hang", cmd)` | `hangClusters` |
| `FileNotFoundError` | `probe` | `missing-bin` | `probeErrors` (전량이면 toolFailed) |
| `PermissionError` | `probe` | `permission` | `probeErrors` |
| 그 외 프로브 예외 | `probe` | `exception_kind` | `probeErrors` |
| 임시 디렉터리 실패 | `main` | 이유 문자열 | `toolErrors` · exit 2 |
| `fuzz` 폭주 | `main` | 이유 문자열 | `toolErrors` · exit 2 |
| 바이너리 탐색 실패 | `main` | stderr + JSON | exit 2 |

`read_sample` / `write_mutant` / `probe` / `fuzz` / `main` 은 맨 바깥에서
`Exception` 을 삼킨다. `# noqa: BLE001` 주석이 "엔진 생존이 우선"임을
남긴다. 삼킨 예외는 메시지로 남기므로 침묵 삼킴이 아니다.

치명 예외(`KeyboardInterrupt` · `SystemExit` · `MemoryError` ·
`GeneratorExit`)는 삼키지 않는다. 사용자가 끊었는데 DoS 0 이라고 쓰면
거짓말이다. `is_fatal_exception` 이 그 네 가지를 가린다.
`exception_kind` 는 치명 여부를 바꾸지 않는다 — 호출자가 raise 한다.

시험은 이 표의 각 칸을 목킹으로 고정한다. 바이너리 없이 돌아간다.

### 7.1 없는 바이너리

`runner.find_bin` 은 없는 경로도 문자열로 돌려준다. `find_bin_safe` 가
`os.path.exists` 로 한 번 더 본다. 없거나 빈 경로면
`(None, "missing-bin: …")` 이다. `main` 은 이 자리에서 `fuzz` 를 부르지
않고 exit 2 를 낸다. JSON 이면 봉투를 stdout 에, 사람이면 요약을
stderr 에 쓴다.

`fuzz("", …)` 는 라이브러리 경로다. 빈 경로는 즉시 `missingBin` 이다.
존재하는 것처럼 보이는 가짜 경로로 `probe` 를 부르면 `FileNotFoundError`
가 `missing-bin` 으로 접힌다. 모든 프로브가 그 kind 이고 패닉·행이
없으면 `toolFailed` 다. "프로브는 돌았으니 DoS 0" 이라고 쓰면 거짓말이다.

### 7.2 빈 코퍼스

`samples` 에 `.hwp`/`.hwpx`/`.hml` 이 없으면 `emptyCorpus=true` 다.
`.txt` 만 있는 디렉터리도 빈 코퍼스다. 표본 0 은 발견할 DoS 가 없다는
뜻이지 도구 실패가 아니다. `ok=true`, exit 0. 사람용 문구는
`빈 코퍼스 — 표본 0` 이다. `DoS 0` 이라고 쓰면 전수를 돈 것처럼 보인다.

디렉터리 자체가 없으면 `list_sample_names` 가 `empty-corpus` 또는
`os-error` 로 접는다. `fuzz` 는 예외를 올리지 않는다.

### 7.3 읽기 실패

한 파일이 `PermissionError` / `FileNotFoundError` / 그 외 `OSError` 로
안 읽히면 `unreadables` 에 `이름: unreadable: …` 를 남기고 다음 표본으로
간다. 나머지 표본은 그대로 프로브한다. `ok` 는 패닉·행만 본다. 한 파일이
잠겨 있다고 전 코퍼스 발견을 멈추지 않는다.

`deterministic_mutants` 가 `TypeError` 를 내면 그것도 `unreadables` 다.
변형을 못 만든 표본은 프로브하지 않는다.

## 8. 변형 카탈로그

카탈로그는 `MUTANT_CATALOG` 다. `mutant_catalog()` 가 사본을 돌려
시험·문서가 같은 표를 본다. 무작위 필드가 없다.

아래 표의 `when` 은 생성 조건이다. 조건이 맞아도 원본과 바이트가 같으면
`add()` 가 버린다. 원 라벨은 게이트의 `truncate@P%` 와 달리 `truncP` 다.
이미 이 태그로 열린 이슈·재현체를 깨지 않기 위해서다.

### 8.1 empty

| id | when | 하는 일 |
|---|---|---|
| `empty-to-nul` | `n==0` | NUL 한 바이트. 빈 입력의 유일한 변형. |

### 8.2 truncate

| id | when | 하는 일 |
|---|---|---|
| `truncP` | `n>0` | 앞 `max(1, n*P/100)` 바이트만 남긴다. P ∈ {1,5,10,25,50,75,95,99} |
| `chop-last` | `n>=2` | 마지막 1바이트를 자른다. 오프바이원 레코드 끝. |
| `cut-first` | `n>=1` | 선두 1바이트를 버린다. 매직이 한 칸 밀린 파일. |
| `odd-length-chop` | 짝수이고 `n>=2` | 짝수 길이를 홀수로 만들어 UTF-16 워드 정렬을 깨뜨린다. |
| `shrink-gap` | `n>=8` | 1/4 지점의 4바이트를 삭제해 뒤 레코드를 당긴다. |

잘린 OLE/ZIP 은 중앙 디렉터리·FAT 가 없는 복합문서를 재현한다. 첫 주행이
잡은 HWP3 line-spacing 패닉도 짧은 본문에서 터졌다. `trunc5/25/50/75/95`
는 원 라벨이다. `trunc1/10/99` 는 확대분이다.

### 8.3 flip

| id | when | 하는 일 |
|---|---|---|
| `flipP` | `n>0` | 위치 `min(n-1, n*P/100)` 한 바이트를 XOR 0xFF. P ∈ {0,10,25,30,50,70,75,90,99} |

원 라벨은 `flip10/30/50/70/90` 이다. 가장자리(`0`, `99`)와 사분면
(`25`, `75`)을 보탠다. 1바이트 입력에서는 모든 플립이 같은 바이트를
건드리지만 라벨은 남는다.

### 8.4 length

| id | when | 하는 일 |
|---|---|---|
| `biglenP` | `n>=4` | 위치에 `0x7FFFFFFF` (u32 LE). P ∈ {10,40,70}. 원 라벨. |
| `length-zero30` | `n>=4` | `0x00000000`. 빈 레코드 조기 종료. |
| `length-one60` | `n>=4` | `0x00000001`. 오프바이원 슬라이스. |
| `i32-min20` | `n>=4` | `0x80000000`. 음수 길이. |
| `u16-max12` | `n>=14` | 오프셋 12 의 u16 을 `0xFFFF`. |

길이 필드는 할당 폭주와 정수 오버플로의 입구다. 양수 포화·0·1·음수
최소를 모두 심어야 파서의 범위 검사가 한 갈래만 통과하는 일을 막는다.

### 8.5 header

| id | when | 하는 일 |
|---|---|---|
| `zero-header` | `n>0` | 선두 최대 512바이트를 0 으로 지운다. |
| `header-smash` | `n>0` | 선두 최대 64바이트를 `DEADBEEF` 반복으로 덮는다. |
| `rotate-header` | `n>=2` | 선두 8바이트를 한 칸 왼쪽 순환. |
| `increment-header` | `n>0` | 선두 8바이트에 1 을 더한다(랩어라운드). |
| `nibble-swap-head` | `n>0` | 선두 32바이트의 니블을 맞바꾼다. |

`zero-header` 와 `header-smash` 를 나눈 이유: 매직이 없는 것과 매직이
다른 손상인 것을 구분해야 파서의 형식 판별 분기를 따로 두드릴 수 있다.

### 8.6 ole

| id | when | 하는 일 |
|---|---|---|
| `ole-trunc-tail` | `n>64` | 꼬리 64바이트를 자른다. CFB 디렉터리/FAT 가 잘린 복합문서. |
| `ole-trunc-tail` | `n<=64` | 꼬리 `k` 바이트를 잘린 OLE 매직으로 바꾼다. |
| `ole-magic-poison` | `n>0` | 선두에 OLE 매직 XOR 0xFF 를 덮는다. |
| `ole-sector-shift-poison` | `n>=32` | 오프셋 30 의 섹터 시프트를 `0xFFFF` 로. |
| `ole-mini-fat-poison` | `n>=72` | 오프셋 60 의 MiniFAT 시작 섹터를 `0xFFFFFFFF` 로. |

HWP5 는 CFB(OLE) 다. 섹터 시프트와 MiniFAT 는 할당 폭주·무한 루프의
고전 위치다. 게이트와 같은 오프셋을 쓰면 게이트가 놓친 명령 축(export-*)
에서 같은 손상을 다시 볼 수 있다.

### 8.7 run

| id | when | 하는 일 |
|---|---|---|
| `ff-run` | `n>0` | 1/3 지점에 최대 128바이트의 `0xFF` 런. |
| `aa-run` | `n>0` | 선두 1/4 에 `0xAA` 런. |
| `nul-mid` | `n>0` | 한가운데 최대 64바이트를 NUL. UTF-16 종료 위조. |
| `00-run` | `n>0` | 2/3 지점에 NUL 런. 레코드 조기 종료. |
| `55-run` | `n>0` | 선두 1/5 에 `0x55` 런. |

런 패턴을 나눈 이유: `0xFF` 는 부호 없는 포화, `0x00` 은 종료,
`0xAA`/`0x55` 는 교차 비트다. 한 패턴만 보면 파서의 다른 정수 해석을
놓친다.

### 8.8 unicode

| id | when | 하는 일 |
|---|---|---|
| `utf16-nul-sprinkle` | `n>=2` | 20/40/60/80% 짝수 오프셋에 U+0000. |
| `utf16-bom-inject` | `n>=2` | 선두에 UTF-16LE BOM(`FF FE`). |
| `utf8-overlong` | `n>=2` | 1/5 지점에 overlong NUL(`C0 80`). |
| `ascii-ctrl-sprinkle` | `n>0` | 15/35/55/75% 에 SOH(0x01). |
| `path-sep-sprinkle` | `n>0` | 18/42/66/88% 에 `/` 와 `\\` 를 교차. |

HWP 본문은 UTF-16LE 가 기본이다. NUL 뿌림은 문자열 절단을, BOM 은
인코딩 오인을, overlong 은 UTF-8 검증을, 경로 구분자는 스트림 이름
해석을 두드린다.

### 8.9 zip

| id | when | 하는 일 |
|---|---|---|
| `zip-local-header-flip` | `PK\\x03\\x04` 존재 | 그 4바이트만 XOR 0xFF. |
| `zip-magic-inject` | 로컬 헤더 없음, `n>=4` | 선두에 로컬 헤더 매직을 심는다. |
| `zip-cd-magic-flip` | `PK\\x01\\x02` 존재 | 중앙 디렉터리 매직만 플립. |
| `zip-eocd-flip` | `PK\\x05\\x06` 존재 | EOCD 매직만 플립. |

HWPX 는 ZIP 이다. 로컬 헤더·중앙 디렉터리·EOCD 를 따로 두드려야
아카이브 탐색의 세 입구를 모두 본다. inject 는 비-ZIP(HWP5) 을 ZIP 으로
오인하게 한다.

### 8.10 permute

| id | when | 하는 일 |
|---|---|---|
| `reverse-prefix` | `n>=2` | 선두 16바이트를 뒤집는다. |
| `swap-ends` | `n>=16` | 선두 8과 꼬리 8을 맞바꾼다. |
| `slide-window-left` | `n>=8` | 선두 32바이트를 한 바이트 왼쪽. |
| `slide-window-right` | `n>=8` | 선두 32바이트를 한 바이트 오른쪽. |
| `repeat-mid-block` | `n>=32` | 한가운데 32바이트를 바로 앞에 복사. |

매직 순서와 필드 정렬을 깨뜨린다. 바이트 값 오염(flip/run)과 다른 축이다.

### 8.11 stripe

| id | when | 하는 일 |
|---|---|---|
| `high-bit-stripe` | `n>0` | 16바이트마다 상위 비트. |
| `low-bit-stripe` | `n>0` | 16바이트마다 최하위 비트 반전. |
| `xor-stride7` | `n>0` | 7바이트마다 `0xA5` XOR. |
| `interleave-zero-head` | `n>0` | 선두 32의 홀수 인덱스를 0. |
| `duplicate-prefix` | `n>=16` | 선두 8을 바로 다음에 복제. |
| `tail-over-head` | `n>=32` | 꼬리 16을 선두에 덮음. |
| `invert-tail-64` | `n>0` | 꼬리 최대 64바이트 비트 반전. |
| `complement-mid-32` | `n>0` | 한가운데 최대 32바이트 비트 반전. |
| `bit-rotate-head` | `n>0` | 선두 16바이트를 1비트 왼쪽 회전. |
| `decrement-tail` | `n>0` | 꼬리 8바이트에서 1 을 뺀다. |

주기적 오염은 레코드 정렬·체크섬·부호 있는 길이를 흔든다. 한 자리만
뒤집으면 그 필드만 본다.

### 8.12 splice

| id | when | 하는 일 |
|---|---|---|
| `splice-nul-mid` | `n>0` 이고 거대 아님 | 한가운데에 NUL 16바이트. |
| `crlf-inject` | `n>0` 이고 거대 아님 | 한가운데에 CRLF. |
| `pad-eof` | `n>0` 이고 거대 아님 | 끝에 SUB(0x1A). |
| `widen-gap` | `n>0` 이고 거대 아님 | 1/4 지점에 NUL 4바이트. |
| `even-length-pad` | 홀수이고 거대 아님 | 끝에 NUL 하나. UTF-16 워드 맞춤. |

거대 입력에 바이트를 끼우면 사본이 커진다. 발견 엔진의 예산은 헤더·절단·
길이에 쓰는 편이 낫다. 게이트와 같은 생략 규칙이다.

### 8.13 hwp3

| id | when | 하는 일 |
|---|---|---|
| `hwp3-sig-flip` | `HWP Document File` 존재 | 서명 첫 4바이트 XOR 0xFF. |
| `hwp3-sig-inject` | 서명 없음, `n` 이 서명보다 김 | 선두에 HWP3 서명을 심는다. |

HWP3 파서는 HWP5 와 다른 입구다. 첫 주행의 line-spacing i32 오버플로는
이 형식에서 났다. 서명이 있는 파일만 flip 하고, 없는 파일에는 inject
한다. 둘을 동시에 켜면 같은 바이트를 두 번 두드린다.

## 9. 가족 매핑

`mutant_family(label)` 이 라벨을 가족 id 로 접는다. 카탈로그 확장 시
여기만 고친다. 빈 문자열·`None`·미지 라벨은 `other` 다. 시험이 전 라벨
표를 고정한다.

| 가족 | 라벨 접두/목록 |
|---|---|
| empty | `empty-to-nul` |
| truncate | `trunc*`, `chop-last`, `cut-first`, `odd-length-chop`, `shrink-gap` |
| flip | `flip*` |
| length | `biglen*`, `length-zero*`, `length-one*`, `i32-min*`, `u16-max*` |
| header | `zero-header`, `header-smash`, `rotate-header`, `increment-header`, `nibble-swap-head` |
| ole | `ole-trunc-tail`, `ole-magic-poison`, `ole-sector-shift-poison`, `ole-mini-fat-poison` |
| run | `ff-run`, `aa-run`, `nul-mid`, `00-run`, `55-run` |
| unicode | `utf16-*`, `utf8-overlong`, `ascii-ctrl-sprinkle`, `path-sep-sprinkle` |
| zip | `zip-*` |
| permute | `reverse-prefix`, `swap-ends`, `slide-window-*`, `repeat-mid-block` |
| stripe | `*-stripe`, `xor-stride7`, `interleave-zero-head`, `duplicate-prefix`, `tail-over-head`, `invert-tail-64`, `complement-mid-32`, `bit-rotate-head`, `decrement-tail` |
| splice | `splice-nul-mid`, `crlf-inject`, `pad-eof`, `widen-gap`, `even-length-pad` |
| hwp3 | `hwp3-sig-flip`, `hwp3-sig-inject` |

## 10. 클러스터링

발견 엔진의 산출은 "죽은 횟수"가 아니라 **고유 버그 목록**이다.

같은 `src/parser/foo.rs:120` 에서 스무 표본이 죽으면 버그 하나다. 명령이
`info` 든 `export-text` 든 위치가 같으면 한 클러스터다. 시험
`test_fuzz_clusters_panics_by_location` 이 이 계약을 고정한다.

행은 위치가 없다. timeout 은 명령을 버킷으로 쓴다. `info` 에서만 멈추고
`export-text` 는 끝나는 경우가 있다. 명령을 섞으면 "어느 경로가 루프인가"
가 사라진다.

클러스터 정렬:

- 패닉: `(-count, location)`
- 행: `(-count, command)`

건수가 같으면 위치가 사전순이다. 리포트 diff 가 워커 순서에 흔들리지
않게 하기 위해서다.

`distinctPanicSites` 는 `len(panicClusters)` 다. 이 숫자가 "고쳐야 할
고유 버그 수"다. `runsChecked` 는 예산이고, `distinctPanicSites` 가
일이다.

## 11. 게이트와의 분업

| | `robustness.py` | `fuzz_corpus.py` |
|---|---|---|
| 역할 | 결정적 회귀 표본 | 발견 엔진 |
| 표본 | `.hwp` 부분집합 | `.hwp`/`.hwpx`/`.hml` 전수 가능 |
| 명령 | `info --json` 하나 | 기본 4명령, 지정 가능 |
| 산출 | 패닉/행 목록 | 위치별·명령별 클러스터 |
| ok | 패닉·행 0 | 패닉·행 0 그리고 도구 실패 아님 |
| 변형 라벨 | `truncate@25%` | `trunc25` (원 재현체 호환) |
| 예외 | unreadables / probeErrors | 같은 접기 + missingBin / emptyCorpus |

회귀 표본이 실패한 변형을 발견 엔진이 다시 두드리는 것은 낭비처럼 보이지만,
표본은 `info` 만 본다. `export-render-tree` 에서만 죽는 버그는 표본을
통과한다. 발견 엔진이 그 축을 담당한다.

반대로 발견 엔진이 잡은 버그는 고친 뒤 결정적 부분집합에 넣어 회귀를
막는다. 발견 → 수정 → 회귀 표본. 이 고리가 이 도구의 존재 이유다.

## 12. 시험이 고정하는 것

`scripts/tests/test_gym_fuzz_corpus.py` 는 바이너리 없이 돈다. subprocess
는 목킹한다.

- 원 계약 5건: 결정적 변형, classify, select_samples, 위치 클러스터, 깨끗한
  주행.
- 확대 변형: 가족 매핑, 헤더/OLE/길이/런/유니코드/permute/stripe/splice
  바이트 계약, ZIP/HWP3 조건부, 거대 입력 splice 생략, 원 라벨 생존.
- 예외 경로: 없는 바이너리(exit 2), 빈 코퍼스(ok), 읽기 실패(unreadables),
  쓰기 실패, 프로브 예외, TypeError 변형, 없는 디렉터리.
- 봉투: `REPORT_KEYS`, `validate_report`, `ok`/`exit`/`emptyCorpus` 정직.
- 정직: 프로브 오류는 행이 아니다. 도구 실패는 DoS 0 이 아니다. 치명
  예외는 kind 로 위장하지 않는다.

`python -m unittest scripts.tests.test_gym_fuzz_corpus` 가 전부 통과해야
한다. `python gym/tools/audit.py` 는 pack 정합을 본다. 이 PR 은 pack 을
건드리지 않으므로 감사는 기존과 같아야 한다.

## 13. 하지 않는 것

- 새 CLI 플래그, 새 pack, 새 gym 과제.
- `random` / `secrets` / 시각 기반 시드. 재현이 안 되면 발견이 아니다.
- `trajectory.py` · `discriminate.py` · automation / core-cli / casual-rides
  pack. 다른 열린 PR 의 파일.
- `cargo fmt --all`. Python·문서만 바꾼다.
- 원 `classify` 계약 변경. `code==1` 깨끗한 실패를 패닉으로 승격하지 않는다.
- 원 라벨 삭제. `trunc25` 를 `truncate@25%` 로 바꾸지 않는다.

이 문서는 규약이다. 구현이 이 표와 다르면 구현이 틀린 것이다.
