---
kind: working
status: active
canonical: mydocs/working/gym_robustness.md
last_verified: 2026-08-18
---

# gym 손상-강건성 감사 — 결정적 변형 확대 작업 기록

Issue: #5218
PR: https://github.com/edwardkim/rhwp/pull/5221
Branch: `feat/gym-robust-mutants`
Date: 2026-08-18

## 1. 결론

`gym/tools/robustness.py` 의 결정적 손상 변형을 확대하고, 감사기 자신의 예외
경로를 분류·보고하도록 닫았다. 무작위는 쓰지 않는다. 같은 입력은 같은
라벨·바이트를 내고, 원본과 동일한 무의미 변형은 버린다.

이 가지는 새 PR 을 열지 않는다. 같은 브랜치에 이어서 밀어 #5221 을 키운다.

검증:

- `python -m unittest scripts.tests.test_gym_robustness scripts.tests.test_gym_audit`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

## 2. 배경

원 PR(#5221)은 `header-smash` / `ole-trunc-tail` / `ff-run` / `utf16-nul-sprinkle`
/ `zip-local-header-flip` 다섯 가족과 `classify_panic` / `classify_timeout` /
JSON 봉투(`kind=gymRobustness`, `schemaVersion=1.0`)를 넣었다. 대비
`upstream/devel` 삽입은 약 168줄이었다.

그 상태의 빈틈:

1. 정상 입력의 기본 변형이 12건 전후. ZIP 이 아니면 13번째가 없다. 헤더·본문·
   길이·유니코드·아카이브 입구를 한 갈래만 두드린다.
2. `probe` 가 `TimeoutExpired` 만 잡고, 없는 바이너리·권한·그 외 예외는 감사기를
   죽일 수 있다.
3. `select_samples` 가 없는 디렉터리에서 `os.listdir` 예외를 그대로 올린다.
4. 빈/극소/거대 입력을 형태로 남기지 않아 표본 편향을 보고에서 못 본다.
5. 카탈로그가 코드에만 있어 문서·시험이 같은 표를 공유하지 않는다.

중단된 구조 초안 `tmp-gym-rescue/robustness.py` 는 예외 접기와 일부 확대 변형
(가장자리 플립, `aa-run`, `nul-mid`, `zip-magic-inject`, `ole-magic-poison`,
`length-bomb`, permute/stripe, `chop-last`, `splice-nul-mid`)을 가지고 있었다.
초안의 예외 경로는 현행의 **진부분집합이 아니라 상위집합**이다 — `TypeError`
강제, `OSError` 프로브 접기, `unreadables`/`probeErrors`/`inputShapes` 봉투.
그래서 초안을 버리고 다시 쓰지 않고, 그 상위집합을 흡수한 뒤 가족을 더 늘렸다.

## 3. 한 일

### 3.1 감사기

`gym/tools/robustness.py`

- `coerce_bytes` — bytes/bytearray/memoryview 만. 그 외 `TypeError`.
- `classify_input_shape` — empty/tiny/normal/huge.
- `normalize_limit` / `normalize_timeout` — 변환 불능은 0.
- `MUTANT_CATALOG` + `mutant_catalog` / `catalog_ids` / `catalog_families` /
  `mutant_family`.
- 확대 결정적 변형(아래 4절).
- `read_sample` / `write_mutant` — 실패를 문자열로.
- `probe` — invalid-timeout, missing-bin, `TimeoutExpired`, `OSError`, 그 외
  예외를 머리 문자열로.
- `classify_panic` 표식 확대. `classify_timeout` 이 `TimeoutError` ·
  `ETIMEDOUT` · 표식 문자열을 받는다.
- `classify_probe_outcome` — hang/panic/error/graceful/ok.
- `empty_report` / `validate_report` / `format_human_report`.
- `audit` 가 읽기실패·쓰기실패·프로브예외를 삼키고 보고에 남긴다.
- 봉투에 `unreadables`, `probeErrors`, `inputShapes` 를 추가.

### 3.2 시험

`scripts/tests/test_gym_robustness.py`

- 기존 `RobustnessTests` 유지. 봉투 키 집합을 새 `REPORT_KEYS` 에 맞춤.
- `ExpandedMutantContractTests` — 가족 매핑, 헤더/OLE/길이/런/유니코드/
  permute/stripe/splice 바이트 계약, ZIP/HWP3 조건부, 거대 입력 splice 생략.
- `ExceptionPathTests` — coerce, normalize, 없는 디렉터리, 읽기/쓰기 실패,
  probe timeout/OSError/RuntimeError, audit 의 unreadable/write/probe 접기,
  스키마 검증.
- `ShapeAndSelectEdgeTests` — stride 안정, 형태 카운트, 빈 샘플 1건 프로브,
  상수 계약.

### 3.3 문서

- `gym/docs/robustness.md` — 규약. 카탈로그·분류·예외 표의 정본.
- `gym/tools/README_robustness.md` — 운영 한 페이지.
- 이 파일 — 작업 기록.

packs·checks·coverage·다른 도구는 손대지 않았다.

## 4. 변형 확대 목록

기존(원 PR 유지):

- `truncate@25,50,75,95%`
- `flip@10,50,90%`
- `zero-header`, `header-smash`, `ole-trunc-tail`, `ff-run`, `utf16-nul-sprinkle`
- `zip-local-header-flip` (ZIP 로컬 헤더가 있을 때만)
- `empty-to-nul`

초안에서 흡수:

- `flip@0%`, `flip@99%`
- `aa-run`, `nul-mid`
- `zip-magic-inject`
- `ole-magic-poison`
- `length-bomb@10,40,70%`
- `reverse-prefix`, `swap-ends`
- `high-bit-stripe`
- `chop-last`
- `splice-nul-mid` (거대 생략)

이번에 추가:

| 라벨 | 가족 | 의도 |
|---|---|---|
| `truncate@10%`, `truncate@99%` | truncate | 더 짧은/거의 전체 절단 |
| `cut-first` | truncate | 매직이 한 칸 밀림 |
| `odd-length-chop` | truncate | UTF-16 워드 정렬 파괴 |
| `shrink-gap` | truncate | 중간 4바이트 삭제 |
| `flip@25%`, `flip@75%` | flip | 사분면 플립 |
| `rotate-header` | header | 매직 순환 |
| `increment-header` | header | 매직 +1 |
| `nibble-swap-head` | header | 니블 교환 |
| `ole-sector-shift-poison` | ole | CFB 섹터 시프트 0xFFFF |
| `ole-mini-fat-poison` | ole | MiniFAT 시작 0xFFFFFFFF |
| `00-run`, `55-run` | run | 종료/교차 비트 런 |
| `utf16-bom-inject` | unicode | LE BOM |
| `utf8-overlong` | unicode | C0 80 |
| `ascii-ctrl-sprinkle` | unicode | SOH |
| `path-sep-sprinkle` | unicode | `/` `\\` |
| `zip-cd-magic-flip` | zip | 중앙 디렉터리 |
| `zip-eocd-flip` | zip | EOCD |
| `length-zero@30%` | length | 길이 0 |
| `length-one@60%` | length | 길이 1 |
| `i32-min@20%` | length | 음수 최소 |
| `u16-max@12` | length | 오프셋 12 u16 포화 |
| `slide-window-left/right` | permute | 32바이트 창 이동 |
| `repeat-mid-block` | permute | 중간 블록 복제 |
| `low-bit-stripe` | stripe | LSB XOR 0x01 |
| `xor-stride7` | stripe | 7바이트마다 0xA5 |
| `interleave-zero-head` | stripe | 홀수 인덱스 0 |
| `duplicate-prefix` | stripe | 이중 매직 |
| `tail-over-head` | stripe | 꼬리로 헤더 덮음 |
| `invert-tail-64` | stripe | 꼬리 비트 반전 |
| `complement-mid-32` | stripe | 중간 비트 반전 |
| `bit-rotate-head` | stripe | 1비트 회전 |
| `decrement-tail` | stripe | 꼬리 -1 |
| `crlf-inject` | splice | CRLF 끼움 |
| `pad-eof` | splice | 0x1A EOF |
| `widen-gap` | splice | 중간 NUL 4 |
| `even-length-pad` | splice | 홀수→짝수 |
| `hwp3-sig-flip` / `hwp3-sig-inject` | hwp3 | HWP3 입구 |

거대 입력(`n >= 1MiB`)에서 크기가 는 변형은 만들지 않는다.

## 5. 예외 경로 표 (시험과 같은 칸)

| 입력 | 기대 | 시험 |
|---|---|---|
| `deterministic_mutants("ab")` | `TypeError` | `test_coerce_bytes_accepts_bytes_like_only` |
| `classify_input_shape(None)` | `TypeError` | 위와 같음 |
| `normalize_limit("nope")` | 0 | `test_normalize_limit_and_timeout` |
| `normalize_timeout(0)` | 0 | 위와 같음 |
| 없는 디렉터리 `select_samples` | `([], 0)` | `test_select_samples_oserror_and_bad_limit` |
| 없는 파일 `read_sample` | `(None, 이유)` | `test_read_sample_missing_and_success` |
| `write_mutant(..., "str")` | `TypeError: …` | `test_write_mutant_typeerror_and_oserror` |
| `probe(..., timeout=0)` | `probe-error invalid-timeout` | `test_probe_invalid_timeout_and_missing_bin` |
| `probe("", …)` | `probe-error missing-bin` | 위와 같음 |
| 없는 바이너리 `probe` | `oserror …`, 패닉 아님 | `test_probe_oserror_is_not_panic` |
| `TimeoutExpired` | hang | `test_probe_timeout_expired_is_hang` |
| `RuntimeError` in `run` | `probe-error RuntimeError` | `test_probe_unexpected_exception_is_error` |
| 읽기 거부 샘플 | `unreadables`, ok 유지 | `test_audit_records_unreadable_samples` |
| 쓰기 실패 | `probeErrors`, checked=0 | `test_audit_records_write_errors` |
| 프로브 예외 머리 | `probeErrors`, ok 유지 | `test_audit_records_probe_error_heads` |
| `probe` 가 raise | 삼켜서 `probeErrors` | `test_audit_probe_raising_is_caught` |
| 변형 생성 TypeError | `unreadables` | `test_audit_mutant_typeerror_is_unreadable` |
| 빈 보고 | `validate_report` 빈 목록 | `test_empty_report_validates` |
| 키 누락/`ok` 불일치 | 위반 목록 | `test_validate_report_detects_schema_breaks` |

## 6. 호환

기존 시험이 기대한 것:

- `ALWAYS_LABELS` 12개가 2KiB 입력에 있다.
- ZIP 이 아니면 `zip-local-header-flip` 이 없다. ZIP 이면 로컬 헤더 4바이트가
  XOR 0xFF.
- 빈 입력은 `[("empty-to-nul", b"\0")]`.
- 극소 입력도 결정적이고 라벨이 유일하다.
- `is_panic` / `classify_timeout` 의 기존 판정.
- `select_samples` 의 stride 결정성.
- 패닉·행 플래그, 우아한 실패의 `ok=true`.

바뀐 것:

- JSON 키 집합이 9개에서 12개로 늘었다. `unreadables`/`probeErrors`/
  `inputShapes`. 기존 `test_json_report_shape` 의 `set(r) == set(REPORT_KEYS)`
  를 새 키에 맞췄다.
- 정상 2KiB 의 변형 수가 12+ 에서 40+ 로 늘었다. `assertGreaterEqual(..., 12)`
  는 40 으로 올렸다. 하한만 올려 호환을 유지한다.
- 사람용 성공 메시지에 읽기실패·프로브오류 카운트를 붙였다.

`ok` 의미는 그대로다. 패닉·행만 뒤집는다.

## 7. 의도적으로 안 한 일

- `gym/README.md` 본문 수정. 이미 기둥을 설명하고, 카탈로그 정본은
  `gym/docs/robustness.md` 로 분리했다.
- pack/과제/checks/coverage 변경. 원 PR 범위와 같다.
- `fuzz_corpus.py` 연동. 분업을 문서에만 적었다.
- 실제 rhwp 바이너리로 전 코퍼스 주행. 이 작업의 게이트는 unittest 다.
- `cargo fmt --all`. 사용자 지시. Rust 를 건드리지 않았다.
- 새 PR. 같은 가지에 커밋·푸시만.

## 8. 결정 기록

### 8.1 초안을 흡수한 이유

초안의 `classify_timeout` 은 `TimeoutExpired` 뿐 아니라 `TimeoutError` 와
`ETIMEDOUT` 을 행으로 본다. 현행은 `TimeoutExpired` 만. 초안이 상위집합이다.
`probe` 의 `OSError` 접기도 같다. 하위집합을 유지하면 없는 바이너리에서
감사기가 죽는다. 그래서 초안을 버리고 현행을 키우는 쪽이 아니라, 초안의
예외 접기를 그대로 옮긴 뒤 가족을 더했다.

초안에 없던 것(섹터 시프트, MiniFAT, HWP3 서명, ZIP CD/EOCD, 길이 0/1/i32min,
splice 확대, 스키마 검증)은 이 커밋에서 보탰다.

### 8.2 읽기실패가 ok 를 안 뒤집는 이유

샘플 디렉터리 권한·부분 손상은 환경이다. 그걸 `ok=false` 로 접으면 CI 가
"rhwp 가 죽는다"고 거짓말한다. 리스트로 남기고 사람은 `unreadables` 길이를
보면 된다.

### 8.3 시그널을 행으로 안 보는 이유

일부 러너는 timeout 을 SIGKILL(-9) 로 돌린다. 그걸 행으로 접으면 실제
세그폴트도 행이 된다. 게이트의 행 권위는 `TimeoutExpired` 한 갈래다.
시그널은 패닉 쪽(음수 코드)으로 남긴다. `_posix_signal_timeout` 은 이 결정을
문서화하는 헬퍼일 뿐 `classify_timeout` 이 부르지 않는다.

### 8.4 거대 입력에서 splice 를 생략하는 이유

1MiB 사본에 16바이트를 끼우는 일은 게이트 판별력을 거의 안 올리고 메모리와
시간을 먹는다. 헤더 오염과 절단이 같은 파서 입구를 더 값싸게 두드린다.

### 8.5 라벨 유일, 바이트 중복 허용

1바이트 입력에서 `flip@10%` 와 `flip@90%` 는 같은 바이트다. 라벨을 합치면
가장자리 계약이 극소 입력에서 사라진다. 시험은 라벨 유일만 강제하고, 바이트
중복은 허용한다. `add()` 가 버리는 것은 원본과 같은 경우뿐이다.

## 9. 재현

```text
# 이 작업나무에서
python -m unittest scripts.tests.test_gym_robustness scripts.tests.test_gym_audit

# 변형 수만 보고 싶을 때
python -c "import importlib.util; from pathlib import Path; p=Path('gym/tools/robustness.py'); s=importlib.util.spec_from_file_location('r', p); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); print(len(m.deterministic_mutants(bytes(range(256))*8)))"
```

바이너리가 있으면:

```text
python gym/tools/robustness.py --bin target/debug/rhwp --json
```

이 작업은 바이너리 주행을 게이트에 넣지 않았다.

## 10. 파일 목록

| 경로 | 역할 |
|---|---|
| `gym/tools/robustness.py` | 감사기. 카탈로그·분류·예외 접기 |
| `scripts/tests/test_gym_robustness.py` | 계약 시험 |
| `gym/docs/robustness.md` | 규약 정본 |
| `gym/tools/README_robustness.md` | 운영 메모 |
| `mydocs/working/gym_robustness.md` | 이 기록 |

## 11. 후속

- 릴리스 게이트 워크플로가 이 도구를 이미 부른다면, 변형 수 증가로 시간이
  늘었는지 한 번 재면 좋다. 표본 기본값 16 은 그대로다.
- 발견 엔진(`fuzz_corpus.py`)이 새 가족을 재사용할지는 별 이슈. 지금은
  게이트와 발견의 분업을 유지한다.
- 실제 코퍼스에서 `hwp3-sig-flip` / `zip-eocd-flip` 이 몇 건이나 켜지는지는
  바이너리 주행 때 `inputShapes` 와 라벨 히스토그램을 보면 된다. 감사 보고에
  라벨 히스토그램은 아직 없다. 필요하면 `schemaVersion` 을 올리지 않고
  선택 키로 넣는 편이 봉투 호환에 안전하다.

## 12. 커밋 메시지 초안

```text
feat(#5218): gym 손상-강건성 감사에 결정적 변형과 예외 경로를 보강한다

같은 입력은 같은 라벨·바이트를 내고, 원본과 동일한 무의미 변형은 버린다.
OLE/ZIP/HWP3·길이·유니코드·스트라이프·스플라이스 가족을 늘리고,
읽기/쓰기/프로브 예외는 보고로 접어 감사기가 죽지 않게 한다.
카탈로그 규약과 운영 메모, 작업 기록을 같은 표로 고정한다.
```

이 초안을 실제 커밋에 쓴다. 새 PR 없음. `git add -A` 없음.
