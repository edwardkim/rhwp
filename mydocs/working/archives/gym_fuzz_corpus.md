---
kind: investigation
status: active
canonical: gym/docs/fuzz_corpus.md
last_verified: 2026-08-18
---

# gym 코퍼스 퍼징 발견 엔진 — 결정적 변형·예외 경로 보강

Issue: #5256
Branch: `feat/gym-fuzz-corpus-hardening`
Date: 2026-08-18

## 1. 결론

`gym/tools/fuzz_corpus.py` 의 결정적 손상 변형을 확대하고, 발견 엔진
자신의 예외 경로를 분류·보고하도록 닫았다. 무작위는 쓰지 않는다. 같은
입력은 같은 라벨·바이트를 내고, 원본과 동일한 무의미 변형은 버린다.

없는 바이너리·빈 코퍼스·읽기 실패는 DoS 로 위장하지 않는다. 엔진은
예외로 죽지 않는다. 치명 예외(`KeyboardInterrupt` · `SystemExit` ·
`MemoryError` · `GeneratorExit`)는 삼키지 않는다.

검증:

- `python -m unittest scripts.tests.test_gym_fuzz_corpus`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

건드리지 않은 것:

- `gym/tools/trajectory.py` · `discriminate.py`
- `gym/packs/automation` · `core-cli` · `casual-rides`
- 다른 열린 PR 의 파일
- 새 CLI 플래그, 새 pack, 새 과제
- 원 `classify` 계약 (`code==1` 깨끗한 실패는 패닉이 아님)
- 원 라벨 `truncN` / `flipN` / `biglenN`

## 2. 배경

원 도구(#4828 / PR #4829)는 전 코퍼스 × 다명령 × 결정적 변형을 병렬로
두들겨 패닉을 `file:line` 으로 묶는 발견 엔진이다. `robustness.py` 가
릴리스 게이트라면 이 도구는 그 앞단이다. 대비 `upstream/devel` 의 구현은
약 210줄, 시험은 5건이었다.

그 상태의 빈틈:

1. 정상 입력의 기본 변형이 13건(trunc 5 + flip 5 + biglen 3). 헤더·OLE·
   ZIP·유니코드·permute/stripe/splice 가 없다. 게이트(#5218)가 이미 그
   가족을 두드리는데 발견 엔진이 더 얇으면, 게이트가 놓친 **명령 축**
   (`export-render-tree` 등)에서 같은 손상을 다시 볼 수 없다.
2. `probe` 가 `TimeoutExpired` 만 잡고, 없는 바이너리·권한·그 외 예외는
   엔진을 죽일 수 있다. 발견 엔진이 죽으면 CI 가 붉어져 rhwp DoS 와
   하네스 결함을 구분할 수 없다.
3. `select_samples` 가 없는 디렉터리에서 `os.listdir` 예외를 그대로
   올린다. 빈 코퍼스와 없는 디렉터리를 구분하는 깃발이 없다.
4. 읽을 수 없는 표본이 전 주행을 멈춘다. 한 파일이 잠겨 있다고 나머지
   수백 개를 안 두드린다.
5. `as_completed` 의 완료 순서가 클러스터 `example` 을 흔든다. 위치
   묶음 자체는 집합이라 안전하지만, 보고 diff 가 워커에 흔들린다.
6. 카탈로그가 코드에만 있어 문서·시험이 같은 표를 공유하지 않는다.
7. JSON 봉투에 예외 자리가 없다. `ok=true` 가 "DoS 0" 인지 "못 돌렸다"
   인지 구분할 수 없다.

이슈 #5256 의 DoD 는 이 빈틈을 닫는 것이다. additions >= 3000.
unittest + audit.py. 무작위 금지. 새 CLI/pack 없음.

게이트 보강(#5218 / `feat/gym-robust-mutants`)의 예외 접기와 확대 변형을
참고하되, 발견 엔진의 정체성은 유지했다.

- 원 라벨 `trunc25` 를 `truncate@25%` 로 바꾸지 않는다. 이미 이 태그로
  열린 재현체가 있다.
- 명령은 여러 개다. 게이트는 `info` 하나다.
- 산출은 위치별 클러스터다. 게이트는 평평한 패닉/행 목록이다.
- 표본 확장자는 `.hwp`/`.hwpx`/`.hml` 이다. 게이트는 `.hwp` 만.

## 3. 한 일

### 3.1 도구

`gym/tools/fuzz_corpus.py`

- `coerce_bytes` — bytes/bytearray/memoryview 만. 그 외 `TypeError`.
- `classify_input_shape` — empty/tiny/normal/huge.
- `normalize_limit` / `normalize_timeout` / `normalize_workers`.
- `parse_commands` — 빈 토큰·중복 제거. 전부 비면 기본 명령.
- `is_sample_name` — `.hwp`/`.hwpx`/`.hml`, 대소문자 무시.
- `MUTANT_CATALOG` + `mutant_catalog` / `catalog_ids` / `catalog_families`
  / `mutant_family`.
- 확대 결정적 변형(아래 4절). 원 라벨은 앞에 남긴다.
- `read_sample` / `write_mutant` — 실패를 문자열로.
- `find_bin_safe` — `find_bin` + exists. 없으면 `missing-bin`.
- `probe` — invalid-timeout, missing-bin, 빈 명령, `TimeoutExpired`,
  `FileNotFoundError`, `PermissionError`, 그 외 예외를 `(error, kind)` 로.
- `classify` 기존 계약 유지. `classify_timeout` / `classify_probe_outcome`
  / `is_panic_code` 를 옆에 둔다.
- `exception_kind` — context 가 probe/read/select/find-bin 이면
  FileNotFound 의 kind 가 갈린다 (missing-bin / unreadable / empty-corpus).
- `FATAL_EXCEPTIONS` — 삼키지 않는다.
- `empty_report` / `validate_report` / `format_human_report` / `resolve_exit`.
- `fuzz` 가 읽기실패·쓰기실패·프로브예외를 삼키고 보고에 남긴다.
- 봉투에 `unreadables`, `probeErrors`, `toolErrors`, `emptyCorpus`,
  `missingBin`, `toolFailed`, `inputShapes`, `exit` 를 추가.
- 클러스터 정렬을 `(-count, location|command)` 로 고정.
- `main` 은 바이너리 부재·임시 디렉터리 실패·fuzz 폭주를 exit 2 로.
  JSON 은 stdout, 사람용 실패는 stderr.

종료 코드: 0=깨끗, 1=DoS 발견, 2=도구 실패.

`ok` = 패닉·행 부재 **그리고** `toolFailed` 가 거짓. 빈 코퍼스는 ok.
없는 바이너리는 not ok.

### 3.2 시험

`scripts/tests/test_gym_fuzz_corpus.py`

- 기존 `FuzzCorpusTests` 5건 유지. 봉투에 키가 늘어도 그 5건은 같은
  주장을 한다.
- `ExpandedMutantContractTests` — 가족 매핑, 헤더/OLE/길이/런/유니코드/
  permute/stripe/splice 바이트 계약, ZIP/HWP3 조건부, 거대 입력 splice
  생략, 원 라벨 생존.
- `ClassifyAndSelectTests` — None/비문자 err, timeout 표식, hwpx/hml
  포함, 없는 디렉터리, 명령 정규화, stride 안정.
- `ExceptionPathTests` — 없는 바이너리(exit 2 JSON/사람), 빈 코퍼스,
  읽기 실패, 쓰기 실패, 프로브 예외, TypeError 변형, 없는 디렉터리,
  `validate_report`, `format_human_report`, shape 집계, 워커 1=워커 4
  클러스터.
- `MainCliTests` — `main(["--bin", 없는경로, "--json"])` 이 2 를 내고
  봉투를 쓴다. 있는 바이너리면 `fuzz` 를 부른다.
- `GeneratedCatalogTableTests` / `HonestyTests` — 카탈로그 why 비지
  않음, 예외 kind 에 panic/hang 없음, 프로브 오류는 행이 아님.

subprocess 는 전부 목킹한다. 바이너리 없이 돈다.

### 3.3 문서

- `gym/docs/fuzz_corpus.md` — 분업, 사용, 봉투, 형태, 결정성, 분류,
  예외 세 자리, 카탈로그 13가족, 클러스터링, 게이트 분업, 시험, 하지
  않는 것.
- `mydocs/working/gym_fuzz_corpus.md` — 이 기록.

pack JSON 은 건드리지 않았다.

## 4. 확대 변형

원 13라벨은 그대로 산다.

| 원 라벨 | 하는 일 |
|---|---|
| `trunc5/25/50/75/95` | 앞 N% 절단 |
| `flip10/30/50/70/90` | 그 위치 1바이트 XOR 0xFF |
| `biglen10/40/70` | 그 위치에 `0x7FFFFFFF` |

뒤에 덧붙인 가족:

- truncate 확대: `trunc1/10/99`, `chop-last`, `cut-first`,
  `odd-length-chop`, `shrink-gap`
- flip 확대: `flip0/25/75/99`
- length 확대: `length-zero30`, `length-one60`, `i32-min20`, `u16-max12`
- header: `zero-header`, `header-smash`, `rotate-header`,
  `increment-header`, `nibble-swap-head`
- ole: `ole-trunc-tail`, `ole-magic-poison`, `ole-sector-shift-poison`,
  `ole-mini-fat-poison`
- run: `ff-run`, `aa-run`, `nul-mid`, `00-run`, `55-run`
- unicode: `utf16-nul-sprinkle`, `utf16-bom-inject`, `utf8-overlong`,
  `ascii-ctrl-sprinkle`, `path-sep-sprinkle`
- zip: 로컬/CD/EOCD flip 또는 magic inject
- permute / stripe / splice / hwp3: 게이트와 같은 축, 라벨만 발견 엔진
  표기

2KiB 정상 입력에서 `LEGACY_ALWAYS_LABELS` + `EXPANDED_ALWAYS_LABELS` 가
모두 나온다. ZIP 매직이 없으면 `zip-local-header-flip` 은 없고
`zip-magic-inject` 가 있다. HWP3 서명이 없으면 inject, 있으면 flip.
거대(1MiB+)는 splice 성장을 생략한다. 빈 입력은 `empty-to-nul` 한 건.

무작위 필드가 없다. `os.urandom` / `random` / `time` 을 변형 생성에
쓰지 않는다.

## 5. 예외 카탈로그

| context | 예외 | kind |
|---|---|---|
| probe / find-bin | FileNotFoundError | missing-bin |
| read | FileNotFoundError / PermissionError / OSError | unreadable |
| select | FileNotFoundError / OSError | empty-corpus |
| * | PermissionError (probe/write) | permission |
| * | TimeoutExpired / TimeoutError | timeout |
| * | UnicodeError | decode-error |
| * | TypeError | type-error |
| * | ValueError / KeyError / IndexError | value-error |
| * | OSError | os-error |
| * | 그 외 | unexpected |

`probe` 의 빈 경로·비양수 timeout·빈 명령은 예외가 나기 전에
`missing-bin` / `invalid-timeout` / `value-error` 로 접는다.

`fuzz("", …)` 는 프로브 전에 `missingBin` 이다. `fuzz(있는것처럼 보이는
가짜, 빈 디렉터리)` 는 `emptyCorpus` 다. `fuzz(있는것처럼 보이는 가짜,
표본은 있는데 probe 가 FileNotFound)` 는 전량이 missing-bin 이면
`toolFailed` 다.

이 세 자리가 이슈가 명한 예외 경로다. 시험이 각각 고정한다.

## 6. 정직 조항 — 바꾸지 않은 것

```
classify(101, "panicked at src/x.rs:42:9") → ("panic", "src/x.rs:42")
classify(134, "stack overflow")            → ("panic", "stack-overflow")
classify(101, "")                          → ("panic", "code101")
classify(-1073741819, "")                  → ("panic", "code-1073741819")
classify(1, "오류: 유효하지 않은 파일")      → (None, None)
classify(0, "정상")                         → (None, None)
```

- 같은 위치의 다른 명령 패닉은 한 클러스터
- 행은 명령 버킷
- `ok` 는 패닉·행이 없고 도구가 실패하지 않았을 때만 true
- 빈 코퍼스는 ok (발견할 것이 없음)
- 없는 바이너리는 not ok (발견을 못 함)
- 프로브 오류는 hangClusters 에 넣지 않는다
- `other-doc` 같은 새 심각도를 만들지 않았다. 발견 엔진의 심각도는
  패닉/행/없음 세 칸이다. 도구 실패는 네 번째 칸이 아니라 **도구 상태**다.

`probe-failed` 를 분류 삼원에 넣지 않은 이유와 같다. 분류는 rhwp 의
동작이고, 도구 실패는 하네스의 동작이다. 둘을 섞으면 "고유 버그 목록"이
"오늘 디스크가 가득 찼음" 과 같은 칸에 앉는다.

## 7. 크기와 범위

이 가지는 Python 도구·시험·문서만 만진다. `src/` 와 `tests/cases/` 와
`gym/packs/` 는 그대로다. 그래서 `cargo fmt --all` 을 돌리지 않는다.
unit-test-tiers / rust-test-suite-manifest 도 다시 생성하지 않는다.

삽입이 두꺼운 이유:

- 변형 카탈로그를 게이트와 같은 축으로 맞추되 원 라벨을 유지하려면
  생성 함수와 시험 바이트 계약이 길어진다.
- 예외 세 자리(없는 바이너리·빈 코퍼스·읽기 실패)를 거짓말 없이 접으려면
  봉투 키와 validate 와 main 경로가 길어진다.
- 문서가 카탈로그와 같은 표를 보지 않으면 다음 확장이 또 코드에만
  남는다. #5256 이 docs 를 DoD 에 넣은 이유다.

얇게 예외 세 줄만 잡으면 재현이 다시 깨진다. 이슈가 말한 그대로다:
"코퍼스 퍼즈가 결정성과 예외 처리 없이 커지면 재현이 안 된다."

## 8. 로컬 검증 실측

작업 트리: `C:\Users\swsz9\rhwp-gym-fuzz-corpus`
브랜치: `feat/gym-fuzz-corpus-hardening` ← `upstream/devel`

```
python -m unittest scripts.tests.test_gym_fuzz_corpus
python gym/tools/audit.py
```

pack 을 안 바꿨으므로 audit 는 기존과 같아야 한다. fuzz_corpus 시험은
바이너리 없이 목킹만 탄다.

## 9. 다음에 하지 말 것

- 원 라벨을 게이트 표기로 리네임하지 말 것.
- `classify` 에 `core dumped` 를 패닉 표식으로 승격하려면 별도 이슈.
  이번 가지는 원 계약을 유지한다. 깨끗한 실패와 겹칠 위험이 있다.
- 새 `--samples-dir` 플래그를 넣지 말 것. 라이브러리 `fuzz()` 가 이미
  받는다. CLI 표면을 늘리면 이슈 범위를 넘는다.
- trajectory / discriminate / automation / core-cli / casual-rides 를
  이 가지에서 고치지 말 것.
