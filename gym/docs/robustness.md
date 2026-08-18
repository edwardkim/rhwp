---
kind: guide
status: active
canonical: gym/docs/robustness.md
last_verified: 2026-08-18
---

# gym 손상-강건성 감사 규약

이 문서는 `gym/tools/robustness.py` 의 **결정적 손상 카탈로그**와 **예외 경로
계약**을 고정한다. 운영 한 줄 요약은
[`gym/tools/README_robustness.md`](../tools/README_robustness.md) 를, 작업 기록은
[`mydocs/working/gym_robustness.md`](../../mydocs/working/gym_robustness.md) 를 본다.
시험 계약은 `scripts/tests/test_gym_robustness.py` 가 기계로 고정한다.

`fuzz_corpus.py` 는 발견 엔진이다. 이 도구는 릴리스 **게이트**다. 무작위가 없고,
같은 입력은 같은 라벨·바이트를 낸다. 원본과 바이트가 같은 무의미 변형은 버린다.

## 1. 왜 이 기둥이 필요한가

gym 의 앞 두 기둥은 과제의 채점을 지킨다.

| 기둥 | 도구 | 질문 |
|---|---|---|
| 종점 무결성 | `discriminate.py` (#4808) | 일 안 한 제출이 만점을 받나? |
| 경로 무결성 | `trajectory.py` (#4810) | 마지막 스텝을 빼도 통과하나? |
| 도구 강건성 | `robustness.py` (#4814 / #5218) | 손상 입력에 rhwp 가 패닉·행 하나? |

에이전트가 아무리 유능해도, 도구가 손상 문서에 죽으면 과제를 끝내지 못한다.
2026 프론티어(AgentHijack 등)가 환경 손상을 재는 이유와 같다. 벤치마크가 자기
도구의 적대적 강건성을 CI 로 인증하지 않으면, 능력 점수는 도구 수명의 상한선을
숨긴다.

이 감사기는 그 상한선을 **숫자로** 드러낸다.

- 패닉(exit 101 · 시그널/음수 코드 · `panicked` · 스택 오버플로) → 실패.
- 행(`TimeoutExpired` / timeout 표식) → 실패.
- 깨끗한 비-0 실패 · 경고 후 부분복구 · 정상 파싱 → 우아함(정상).

감사기 자신도 예외 경로에서 죽지 않는다. 읽을 수 없는 샘플, 빈/극소/거대 입력,
바이트가 아닌 입력, 프로브 시간초과·OS 오류는 분류해 보고하고 중단하지 않는다.

## 2. 사용

```bash
python gym/tools/robustness.py --bin target/debug/rhwp
python gym/tools/robustness.py --bin target/debug/rhwp --limit 40
python gym/tools/robustness.py --bin target/debug/rhwp --json
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--bin` | (필수) | rhwp 바이너리. `gym.core.runner.find_bin` 으로 해석한다. |
| `--limit` | 16 | 정렬된 `.hwp` 를 결정적 stride 로 뽑는 표본 수. |
| `--timeout` | 20 | 프로브 초. 비양수는 CLI 가 거절한다. |
| `--json` | off | `gymRobustness` 봉투를 stdout 에 쓴다. |

프로브 명령은 항상 `rhwp info <mutant> --json` 이다. 편집·렌더 명령은 이 게이트의
범위가 아니다. 발견 엔진이 그 축을 담당한다.

종료 코드:

| exit | 의미 |
|---|---|
| 0 | 패닉 0 · 행 0. 읽기실패·프로브오류는 ok 를 뒤집지 않는다. |
| 1 | 패닉 또는 행이 하나라도 있다. |
| 2 | 바이너리를 찾지 못했다. |

## 3. JSON 봉투

`kind=gymRobustness`, `schemaVersion=1.0`. 키 집합은 시험이 `REPORT_KEYS` 로 고정한다.

| 키 | 형 | 의미 |
|---|---|---|
| `kind` | str | 항상 `gymRobustness` |
| `schemaVersion` | str | 항상 `1.0` |
| `ok` | bool | `panics` 와 `hangs` 가 모두 비어 있을 때만 true |
| `samplesTested` | int | 실제로 고른 표본 수 |
| `totalSamples` | int | 디렉터리의 `.hwp` 총수 |
| `mutantsChecked` | int | 프로브까지 도달한 변형 수 |
| `gracefullyDegraded` | int | 비-0 이면서 패닉·행이 아닌 프로브 수 |
| `panics` | list[str] | `이름:라벨 (code N): 머리` |
| `hangs` | list[str] | `이름:라벨` |
| `unreadables` | list[str] | 읽기/형태/변형 생성 실패 |
| `probeErrors` | list[str] | 쓰기 실패·OS 오류·프로브 예외 |
| `inputShapes` | object | `empty`/`tiny`/`normal`/`huge` 카운트 |

`ok` 는 패닉·행만 본다. 읽기실패와 프로브오류는 **감사기 생존** 신호이지 rhwp
강건성 실패가 아니다. 바이너리가 없어도 감사기가 죽으면 게이트가 거짓 음성을
낸다.

`validate_report()` 가 이 계약을 다시 검사한다. 키가 빠지거나 `ok` 가 패닉·행과
어긋나면 위반 목록을 돌려준다.

## 4. 입력 형태

위치 기반 변형은 입력 길이에 따라 다르게 동작한다. 감사기는 표본을 네 형태로
접어 `inputShapes` 에 남긴다.

| 형태 | 조건 | 변형 동작 |
|---|---|---|
| `empty` | `n==0` | `empty-to-nul` 한 건만. 위치 기반 변형 없음. |
| `tiny` | `1 <= n <= 64` | 위치 기반은 가능하나 일부는 원본과 같아 버려진다. OLE 꼬리는 잘린 매직을 심는다. |
| `normal` | `65 <= n < 1MiB` | 전체 카탈로그. 크기 증가 변형 포함. |
| `huge` | `n >= 1MiB` | 크기 증가 변형(`splice-*`, `pad-eof`, `widen-gap`, `even-length-pad`)을 생략한다. |

형태 분류는 `classify_input_shape()` 가 맡는다. 비-바이트는 `TypeError` 다.
감사기는 그 예외를 `unreadables` 로 접고 다음 표본으로 간다.

## 5. 결정성 규칙

1. 입력은 bytes-like 만 받는다. `str`/`int`/`None` 을 조용히 인코딩하지 않는다.
2. 같은 바이트열은 같은 `(라벨, 바이트)` 순서를 낸다. 난수·시각· entroy 없음.
3. `add()` 는 `mut == data` 이면 버린다. 무의미 변형이 프로브 예산을 먹지 않는다.
4. 라벨은 가족 안에서 유일하다. 같은 바이트가 나와도 라벨이 다르면 둘 다 남긴다
   (예: 1바이트 입력의 `flip@10%` 와 `flip@90%`).
5. 조건부 가족(ZIP/HWP3)은 매직이 있을 때만 켠다. 없을 때는 inject 쪽이 켜진다.
6. 거대 입력은 크기 증가 변형을 건너뛴다. 게이트는 헤더/절단만으로 충분하다.

표본 선택은 파일명을 정렬한 뒤 stride 로 `limit` 개를 뽑는다. `.txt` 등 비-hwp 는
제외한다. 디렉터리를 읽을 수 없으면 빈 목록을 돌려 감사기가 죽지 않게 한다.

## 6. 분류기

### 6.1 패닉

`classify_panic(code, err)` = `is_panic(code, err)`.

패닉으로 본다:

- 출력(소문자)에 다음 표식이 있을 때: `panicked`, `stack overflow`, `core dumped`,
  `fatal runtime error`, `sigsegv`, `sigabrt`, `sigill`, `sigbus`,
  `access violation`, `segmentation fault`, `illegal instruction`, `abort trap`.
- `code == 101` (Rust abort).
- `code < 0` (POSIX 시그널 종료).
- `code >= 0` 이고 상위 두 비트가 `0xC0000000` (Windows NTSTATUS 예외).

패닉이 아니다:

- `code is None` 이고 표식이 없을 때.
- `code` 가 int 로 변환되지 않을 때.
- `1`·`255` 같은 일반 CLI 오류 코드. 깨끗한 실패를 패닉으로 오판하지 않는다.

시그널 종료(`-9`, `-24`, `-30`)는 **패닉 쪽**이다. 일부 환경은 시간초과를
SIGKILL 로 돌려주지만, 이 게이트는 `TimeoutExpired` 만을 행의 권위로 둔다.
시그널을 행으로 접으면 실제 크래시가 행으로 위장된다.

### 6.2 행

`classify_timeout(timed_out)` 이 true 일 때만 행이다.

- `True`
- `subprocess.TimeoutExpired`
- `TimeoutError`
- `OSError` 이면서 `errno` 가 `ETIMEDOUT` 또는 `ETIME`
- 소문자 표식: `timeout`, `timed out`, `time-out`, `time expired`, `deadline exceeded`

`False`/`None`/그 외 예외/`ValueError("timeout in name only")` 는 행이 아니다.

### 6.3 프로브 접기

`classify_probe_outcome(code, panicked, timed_out, head)`:

| 결과 | 조건 | 보고 |
|---|---|---|
| `hang` | timeout 분류가 true | `hangs` |
| `panic` | 패닉 분류 또는 `panicked` 플래그 | `panics` |
| `error` | 머리가 `oserror `/`probe-error `/`unreadable ` 로 시작 | `probeErrors` |
| `graceful` | `code not in (0, None)` | `gracefullyDegraded += 1` |
| `ok` | `code == 0` | 카운트만 |
| `error` | 그 외(`code is None` 이고 표식 없음) | `probeErrors` |

우선순위는 행 > 패닉 > 오류머리 > 우아함 > 정상 이다. 행과 패닉이 동시에 오면
행이 이긴다. 프로브가 timeout 을 이미 선언했기 때문이다.

## 7. 예외 경로 — 감사기가 죽지 않는 계약

감사기 자신이 예외로 죽으면 게이트는 "도구가 강건하다"는 거짓 음성을 못 내고,
반대로 CI 가 붉어져 강건성 결함과 하네스 결함을 구분할 수 없다. 그래서 모든
I/O·형식·프로브 경로는 분류 가능한 문자열로 접힌다.

| 경로 | 함수 | 접는 곳 | 보고 키 |
|---|---|---|---|
| 비-바이트 입력 | `coerce_bytes` | `TypeError` | 호출 측. 감사는 `unreadables` |
| 형태 분류 실패 | `classify_input_shape` | `TypeError` | `unreadables` |
| 한도/초 변환 실패 | `normalize_limit` / `normalize_timeout` | 0 | 표본 없음 / 프로브 거절 |
| 디렉터리 읽기 실패 | `select_samples` | `[] , 0` | 빈 보고 |
| 표본 읽기 실패 | `read_sample` | `(None, 이유)` | `unreadables` |
| 변형 생성 `TypeError` | `deterministic_mutants` | 감사 루프 | `unreadables` |
| 변형 쓰기 실패 | `write_mutant` | 이유 문자열 | `probeErrors` |
| timeout <= 0 | `probe` | `probe-error invalid-timeout` | `probeErrors` |
| 빈 바이너리 경로 | `probe` | `probe-error missing-bin` | `probeErrors` |
| `TimeoutExpired` | `probe` | `timed_out=True` | `hangs` |
| `OSError` (없는 바이너리) | `probe` | `oserror …` | `probeErrors` |
| 그 외 프로브 예외 | `probe` | `probe-error …` | `probeErrors` |
| 임시 디렉터리 실패 | `audit` | 이유 문자열 | `probeErrors` |
| `select_samples` 폭주 | `audit` | 이유 문자열 | `unreadables` |
| 바이너리 탐색 실패 | `main` | stderr + exit 2 | (보고 없음) |

`read_sample` / `write_mutant` / `probe` / `audit` / `main` 은 맨 바깥에서
`Exception` 을 삼킨다. `# noqa: BLE001` 주석이 "감사기 생존이 우선"임을 남긴다.
삼킨 예외는 메시지로 남기므로 침묵 삼킴이 아니다.

시험은 이 표의 각 칸을 목킹으로 고정한다. 바이너리 없이 돌아간다.

## 8. 변형 카탈로그

카탈로그는 `MUTANT_CATALOG` 다. `mutant_catalog()` 가 사본을 돌려 시험·문서가
같은 표를 본다. 무작위 필드가 없다.

아래 표의 `when` 은 생성 조건이다. 조건이 맞아도 원본과 바이트가 같으면
`add()` 가 버린다.

### 8.1 empty

| id | when | 하는 일 |
|---|---|---|
| `empty-to-nul` | `n==0` | NUL 한 바이트. 빈 입력의 유일한 변형. |

### 8.2 truncate

| id | when | 하는 일 |
|---|---|---|
| `truncate@P%` | `n>0` | 앞 `max(1, n*P/100)` 바이트만 남긴다. P ∈ {10,25,50,75,95,99} |
| `chop-last` | `n>=2` | 마지막 1바이트를 자른다. 오프바이원 레코드 끝. |
| `cut-first` | `n>=1` | 선두 1바이트를 버린다. 매직이 한 칸 밀린 파일. |
| `odd-length-chop` | 짝수이고 `n>=2` | 짝수 길이를 홀수로 만들어 UTF-16 워드 정렬을 깨뜨린다. |
| `shrink-gap` | `n>=8` | 1/4 지점의 4바이트를 삭제해 뒤 레코드를 당긴다. |

잘린 OLE/ZIP 은 중앙 디렉터리·FAT 가 없는 복합문서를 재현한다. 첫 주행이 잡은
HWP3 line-spacing 패닉도 짧은 본문에서 터졌다.

### 8.3 flip

| id | when | 하는 일 |
|---|---|---|
| `flip@P%` | `n>0` | 위치 `min(n-1, n*P/100)` 한 바이트를 XOR 0xFF. P ∈ {0,10,25,50,75,90,99} |

가장자리(`0%`, `99%`)는 퍼센트 위치와 겹치지 않을 수 있다. 1바이트 입력에서는
모든 플립이 같은 바이트를 건드리지만 라벨은 남는다.

### 8.4 header

| id | when | 하는 일 |
|---|---|---|
| `zero-header` | `n>0` | 선두 최대 512바이트를 0 으로 지운다. |
| `header-smash` | `n>0` | 선두 최대 64바이트를 `DEADBEEF` 반복으로 덮는다. |
| `rotate-header` | `n>=2` | 선두 8바이트를 한 칸 왼쪽 순환. |
| `increment-header` | `n>0` | 선두 8바이트에 1 을 더한다(랩어라운드). |
| `nibble-swap-head` | `n>0` | 선두 32바이트의 니블을 맞바꾼다. |

`zero-header` 와 `header-smash` 를 나눈 이유: 매직이 없는 것과 매직이 다른
손상인 것을 구분해야 파서의 형식 판별 분기를 따로 두드릴 수 있다.

### 8.5 ole

| id | when | 하는 일 |
|---|---|---|
| `ole-trunc-tail` | `n>64` | 꼬리 64바이트를 자른다. CFB 디렉터리/FAT 가 잘린 복합문서. |
| `ole-trunc-tail` | `n<=64` | 꼬리 `k` 바이트를 잘린 OLE 매직으로 바꾼다. |
| `ole-magic-poison` | `n>0` | 선두에 OLE 매직 XOR 0xFF 를 덮는다. |
| `ole-sector-shift-poison` | `n>=32` | 오프셋 30 의 섹터 시프트를 `0xFFFF` 로. 거대 섹터 할당 유도. |
| `ole-mini-fat-poison` | `n>=72` | 오프셋 60 의 MiniFAT 시작 섹터를 `0xFFFFFFFF` 로. |

HWP5 는 CFB(OLE) 다. 섹터 시프트와 MiniFAT 는 할당 폭주·무한 루프의 고전 위치다.

### 8.6 run

| id | when | 하는 일 |
|---|---|---|
| `ff-run` | `n>0` | 1/3 지점에 최대 128바이트의 `0xFF` 런. |
| `aa-run` | `n>0` | 선두 1/4 에 `0xAA` 런. |
| `nul-mid` | `n>0` | 한가운데 최대 64바이트를 NUL. UTF-16 종료 위조. |
| `00-run` | `n>0` | 2/3 지점에 NUL 런. 레코드 조기 종료. |
| `55-run` | `n>0` | 선두 1/5 에 `0x55` 런. |

런 패턴을 나눈 이유: `0xFF` 는 부호 없는 포화, `0x00` 은 종료, `0xAA`/`0x55` 는
교차 비트다. 한 패턴만 보면 파서의 다른 정수 해석을 놓친다.

### 8.7 unicode

| id | when | 하는 일 |
|---|---|---|
| `utf16-nul-sprinkle` | `n>=2` | 20/40/60/80% 짝수 오프셋에 U+0000. |
| `utf16-bom-inject` | `n>=2` | 선두에 UTF-16LE BOM(`FF FE`). |
| `utf8-overlong` | `n>=2` | 1/5 지점에 overlong NUL(`C0 80`). |
| `ascii-ctrl-sprinkle` | `n>0` | 15/35/55/75% 에 SOH(0x01). |
| `path-sep-sprinkle` | `n>0` | 18/42/66/88% 에 `/` 와 `\\` 를 교차. |

HWP 본문은 UTF-16LE 가 기본이다. NUL 뿌림은 문자열 절단을, BOM 은 인코딩 오인을,
overlong 은 UTF-8 검증을, 경로 구분자는 스트림 이름 해석을 두드린다.

### 8.8 zip

| id | when | 하는 일 |
|---|---|---|
| `zip-local-header-flip` | `PK\\x03\\x04` 존재 | 그 4바이트만 XOR 0xFF. |
| `zip-magic-inject` | 로컬 헤더 없음, `n>=4` | 선두에 로컬 헤더 매직을 심는다. |
| `zip-cd-magic-flip` | `PK\\x01\\x02` 존재 | 중앙 디렉터리 매직만 플립. |
| `zip-eocd-flip` | `PK\\x05\\x06` 존재 | EOCD 매직만 플립. |

HWPX 는 ZIP 이다. 로컬 헤더·중앙 디렉터리·EOCD 를 따로 두드려야 아카이브
탐색의 세 입구를 모두 본다. inject 는 비-ZIP(HWP5) 을 ZIP 으로 오인하게 한다.

### 8.9 length

| id | when | 하는 일 |
|---|---|---|
| `length-bomb@P%` | `n>=4` | 위치에 `0x7FFFFFFF` (u32 LE). P ∈ {10,40,70} |
| `length-zero@30%` | `n>=4` | `0x00000000`. 빈 레코드 조기 종료. |
| `length-one@60%` | `n>=4` | `0x00000001`. 오프바이원 슬라이스. |
| `i32-min@20%` | `n>=4` | `0x80000000`. 음수 길이. |
| `u16-max@12` | `n>=14` | 오프셋 12 의 u16 을 `0xFFFF`. |

길이 필드는 할당 폭주와 정수 오버플로의 입구다. 양수 포화·0·1·음수 최소를
모두 심어야 파서의 범위 검사가 한 갈래만 통과하는 일을 막는다.

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
| `decrement-tail` | `n>0` | 꼬리 8바이트에서 1 을 뺌. |

스트라이프는 "드문드문" 오염이다. 한 필드만 건드리는 flip 과, 구간을 덮는 run
사이에 있다. 체크섬·정렬·플래그 비트를 흔든다.

### 8.12 splice

크기 증가 변형. `n >= 1MiB` 이면 전부 생략한다.

| id | when | 하는 일 |
|---|---|---|
| `splice-nul-mid` | 거대 아님 | 한가운데에 NUL 16바이트. |
| `crlf-inject` | 거대 아님 | 한가운데에 CRLF. |
| `pad-eof` | 거대 아님 | 끝에 SUB(0x1A). 구식 EOF. |
| `widen-gap` | 거대 아님 | 1/4 지점에 NUL 4바이트. |
| `even-length-pad` | 홀수이고 거대 아님 | NUL 하나를 붙여 짝수로. |

끼워 넣기는 뒤 레코드의 오프셋을 민다. 절단이 "없는 바이트"를 재현한다면,
splice 는 "있으면 안 되는 바이트"를 재현한다.

### 8.13 hwp3

| id | when | 하는 일 |
|---|---|---|
| `hwp3-sig-flip` | `HWP Document File` 존재 | 서명 첫 4바이트 XOR 0xFF. |
| `hwp3-sig-inject` | 서명 없고 `n` 이 서명보다 김 | 선두에 HWP3 서명을 심음. |

HWP3 파서는 HWP5/OLE 와 다른 입구다. 첫 강건성 주행이 잡은 DoS 2건이 이
입구였다. 서명을 뒤집거나 심어 형식 판별 분기를 따로 두드린다.

## 9. 정상 2KiB 입력에서 항상 나오는 라벨

시험이 `ALWAYS_LABELS` 와 `EXPANDED_ALWAYS_LABELS` 로 고정한다. 2KiB
(`bytes(range(256))*8`) 는 ZIP/HWP3 서명이 없고 짝수이며 거대하지 않다.

기본(기존 게이트와 호환):

- `truncate@25%` `truncate@50%` `truncate@75%` `truncate@95%`
- `flip@10%` `flip@50%` `flip@90%`
- `zero-header` `header-smash` `ole-trunc-tail` `ff-run` `utf16-nul-sprinkle`

확대:

- 절단/플립 가장자리, `chop-last`, `cut-first`, `odd-length-chop`, `shrink-gap`
- 런 가족 전부, OLE poison, ZIP inject, 길이 폭탄 전부
- permute/stripe 가족, unicode 주입, splice 가족
- `hwp3-sig-inject`

나오지 않아야 하는 것: `zip-local-header-flip`, `zip-cd-magic-flip`,
`zip-eocd-flip`, `hwp3-sig-flip`, `empty-to-nul`, `even-length-pad`.

## 10. 새 변형을 넣는 절차

1. `MUTANT_CATALOG` 에 `id`/`family`/`when`/`why` 를 한 줄로 적는다. 무작위
   파라미터가 있으면 넣지 않는다.
2. `deterministic_mutants()` 에 같은 라벨로 `add()` 한다. 거대 입력에서 크기가
   늘면 `huge` 가드를 건다.
3. `mutant_family()` 에 라벨 → 가족 접기를 추가한다.
4. 정상 2KiB 에서 항상 나오면 `EXPANDED_ALWAYS_LABELS` 에 넣는다. 조건부이면
   시험에 "있을 때/없을 때" 쌍을 넣는다.
5. `scripts/tests/test_gym_robustness.py` 에 바이트 계약(어느 오프셋이 어떤
   상수인가)을 한 건 이상 적는다.
6. 이 문서의 해당 가족 표를 같은 커밋에서 고친다.

packs·checks·coverage·다른 도구는 건드리지 않는다. 이 기둥은 감사기 한 파일과
그 시험·문서만 키운다.

## 11. 발견 엔진과의 분업

| | `robustness.py` | `fuzz_corpus.py` |
|---|---|---|
| 역할 | 릴리스 게이트 | 발견 엔진 |
| 범위 | 결정적 부분집합 | 전 코퍼스 × 다명령 |
| 난수 | 없음 | 없음(결정적)이되 조합이 넓다 |
| 산출 | 패닉·행 0 인가 | file:line 클러스터 |
| 실패 | 회귀 | 아직 안 고친 DoS 목록 |

발견이 고유 버그를 내면 고치고, 이 게이트가 그 회귀를 막는다. 게이트를
exhaustive 로 키우면 CI 예산이 발견 엔진과 겹친다. 그래서 거대 입력의 splice
를 생략하고 표본을 stride 로 묶는다.

## 12. 시험 지도

`python -m unittest scripts.tests.test_gym_robustness scripts.tests.test_gym_audit`

| 클래스 | 고정하는 것 |
|---|---|
| `RobustnessTests` | 기존 게이트: 결정성, 기본 라벨, 패닉/행, JSON 봉투 |
| `ExpandedMutantContractTests` | 확대 가족·바이트 계약·조건부 ZIP/HWP3·거대 생략 |
| `ExceptionPathTests` | coerce/normalize/probe/audit 의 모든 예외 접기 |
| `ShapeAndSelectEdgeTests` | stride, 형태 카운트, 빈 디렉터리, 상수 계약 |

바이너리 없이 돈다. `probe` 는 목킹한다. 실제 `subprocess.run` 을 쓰는 시험은
없는 바이너리의 `OSError` 와 timeout/예외 목킹뿐이다.

`test_gym_audit` 는 전 pack 정합을 지킨다. 이번 변경은 pack 을 건드리지 않으므로
그대로 초록이어야 한다.

## 13. 사람이 읽는 출력

성공:

```text
gym 손상-강건성 감사: 샘플 16/N × 손상 M건 — 패닉 0 · 행 0
(우아한 실패/부분복구 G · 읽기실패 U · 프로브오류 P)
```

실패:

```text
gym 손상-강건성 감사: 패닉 A · 행 B — rhwp 가 손상 입력에 죽는다:
  - sample.hwp:ff-run (code 101): thread 'main' panicked ...
  - sample.hwp:truncate@50%
```

JSON 모드는 같은 봉투를 들여쓴다. 기계는 `ok` 와 두 리스트만 보면 된다.

## 14. 하지 않는 것

- 난수 변형. 재현이 안 되면 게이트가 아니다.
- 편집·렌더 명령 프로브. 발견 엔진의 몫.
- pack/과제 변경. 채점 기둥과 섞지 않는다.
- 읽기실패를 `ok=false` 로 뒤집기. 그건 환경 결함이다.
- 시그널 종료를 행으로 접기. 크래시 위장이다.
- 거대 입력에 바이트를 끼워 사본을 키우기. CI 예산을 먹는다.

이 문서가 코드와 다르면 코드와 시험을 이긴다. 문서만 고치고 시험을 안 고치면
계약이 아니다.
