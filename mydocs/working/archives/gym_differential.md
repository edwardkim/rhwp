---
kind: working
status: active
canonical: mydocs/working/gym_differential.md
last_verified: 2026-08-18
---

# gym 교차형식 차등 — 예외 경로와 관문 정직성 보강

Issue: #5224
PR: https://github.com/edwardkim/rhwp/pull/5228
Branch: `feat/gym-differential-hardening`
Date: 2026-08-18

## 1. 결론

`gym/tools/differential.py` 의 짝짓기(`pick_twin_paths`)와 본문 해시
(`same_body_hash` / `body_hash_from_env`)는 그대로 두고, CLI·walk·쓰기
예외를 관측/오류 목록으로 접도록 닫았다. 해시를 못 구하면 동일 문서로
치지 않는다. IR 을 못 구하면 `contradiction` 으로 위장하지 않는다.

이 가지는 새 PR 을 열지 않는다. 같은 브랜치에 이어서 밀어 #5228 을 키운다.

검증:

- `python -m unittest scripts.tests.test_gym_differential`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

## 2. 배경

원 PR(#5228)은 쌍둥이 짝짓기·본문 해시·관측 대조·오검출 관문·
`gymDifferential` 보고 봉투를 순수 함수로 분리했다. 대비 `upstream/devel`
삽입은 약 697줄이었다.

그 상태의 빈틈:

1. `run_cli` 가 `subprocess.run` 예외를 그대로 올린다. 없는 바이너리·권한·
   시간초과가 차등 도구 전체를 죽인다.
2. `compare_twins` 의 주입 `run` 이 예외를 내면 한 쌍이 아니라 전수 스윕이
   멈춘다.
3. `export-text` 가 예외로 죽으면 본문 해시를 못 구하는데, 그 상태를
   어떻게 부를지 계약이 코드 주석에만 있다. `None==None` 을 참으로 접으면
   바이너리 부재가 "전 쌍 침묵" 이 된다.
4. `ir-diff` 가 예외로 죽으면 `identical` 을 못 본다. 그 상태를
   `contradiction` 으로 부르면 거짓말이다.
5. `os.walk` / `os.path.isdir` 의 `OSError` 가 짝 탐색을 죽인다.
6. `write_report` 실패가 도구를 죽인다. 디스크가 가득 찼다고 모순 집계를
   못 남긴다.
7. 카탈로그가 코드에만 있어 문서·시험이 같은 표를 공유하지 않는다.

관문 함수 자체를 네 값이 아닌 다섯 값으로 늘리면 기존 게이트 계약
(`classify_pair` ↔ findings 포함 여부)이 깨진다. 그래서
`classify_pair` 는 네 칸을 유지하고, 해시/IR 부재는 호출자가 `False` 로
넣는다.

## 3. 한 일

### 3.1 도구

`gym/tools/differential.py`

- `exception_kind` / `exception_observation` / `exception_tool_error` —
  예외를 관측/도구 오류 kind 로 접는다. CLI·hash·ir 의 `FileNotFound` 는
  `missing-bin`.
- `FATAL_EXCEPTIONS` — `KeyboardInterrupt` · `SystemExit` · `MemoryError` ·
  `GeneratorExit` 는 삼키지 않는다.
- `run_cli_safe` / `observe_with_run` — CLI 예외를 관측으로 접는다.
- `body_hash` / `body_hash_with_run` — 예외면 `None`. `same_body_hash` 는
  그대로 `None==None` 이 거짓.
- `ir_identity_with_run` — 예외면 `(False, None)`. `make_finding` 은
  `irIdentical=False` → `review`.
- `compare_twins_detailed` — 한 쌍의 예외는 `pairErrors` 로 남기고 그
  쌍만 건너뛴다. 3튜플 `compare_twins` 계약은 유지.
- `find_twins_in` — `isdir`/`walk` OSError 는 빈 목록. 없는 쌍을 지어내지
  않는다.
- `pick_twin_paths` — 비문자 경로만 순위에서 뺀다. 같은 디렉터리 우선·
  얕고 사전순 규칙은 그대로다.
- `write_report_safe` / `find_bin_safe` / `find_twins_safe` — 한 쓰기가
  전체를 죽이지 않는다.
- `build_report` — `toolFailed` / `exit` 부가. `ok` 는 여전히
  `contradictions==0`.
- `validate_report` — ok/집계/irIdentical/other-doc 정직 계약.
- `main` — 도구 실패면 exit 1. 모순이면 3. 아니면 0.

### 3.2 시험

`scripts/tests/test_gym_differential.py`

- 기존 순수 시험(`TwinDiscovery` · `BodyHash` · `Observation` ·
  `Classification` · `CompareTwins` · `Report`) 유지.
- `ExceptionKindTests` — kind 카탈로그, 치명 예외 표지.
- `TruncateAndTimeoutHelperTests` — head/timeout/limit/sha256.
- `ObservationEqualityEdgeTests` — NaN, inf, 중첩, bool, 오류 페이로드.
- `PairingHonestyTests` — 같은 디렉터리 우선, 역순 입력, walk OSError.
- `HashHonestyTests` — `None==None` 거짓, 공백 접힘, 글자 불변.
- `ClassifyHonestyTests` — 해시 부재를 contradiction 으로 부르지 않음.
- `CompareTwinsExceptionTests` — 같은 오류 양쪽 = 갈림 아님.
- `IrFailureIsNotContradictionTests` — ir-diff 예외 → review.
- `MissingHashIsNotContradictionTests` — 해시 예외 → other-doc.
- `ReportHonestyTests` — `validate_report` 가 ok/ir/other-doc 거짓말을 잡음.
- `RunCliSafeTests` / `WriteReportSafeTests` / `FindBinAndMainTests`.
- `GeneratedClassifyTableTests` / `GeneratedPairingTableTests` /
  `GeneratedEqualityTableTests` / `GeneratedHashTableTests`.

### 3.3 문서

- `gym/docs/differential.md` — 관문 네 칸, 짝짓기, 해시, 관측 kind,
  JSON 봉투, 예외 자리, 종료 코드, 표본.
- `mydocs/working/gym_differential.md` — 이 기록.

pack JSON 은 건드리지 않았다.

## 4. 정직 조항 — 바꾸지 않은 것

다음 계약은 원 PR 과 같다. 시험이 다시 고정한다.

```
classify_pair(갈림 없음, *)            → None
classify_pair(갈림, body_same=False, *) → other-doc
classify_pair(갈림, True, ir=True)      → contradiction
classify_pair(갈림, True, ir=False)     → review
```

- `same_body_hash(None, None)` 는 거짓
- `same_body_hash(hash, None)` 는 거짓
- `body_hash_from_env` 의 공백 무시 SHA-256 은 그대로
- `pick_twin_paths` 는 같은 디렉터리 우선, 없으면 얕고 사전순
- 숫자 `6` 과 `6.0` 은 같고, `True` 는 `1` 로 접히지 않는다
- `ok` 는 `contradictions==0` 과만 참
- 리뷰만 있으면 `ok=true` (자동 차단 아님)
- `other-doc` 은 findings 에 넣지 않는다
- CLI 플래그 `--limit` `--bin` `-o` 는 그대로다 (`--cli-timeout` 만 추가)

`other-doc` 을 `SEVERITIES` 에 넣지 않았다. 넣으면 이름만 같은 다른 문서가
결함 후보로 집계된다.

## 5. 예외 카탈로그

| context | 예외 | kind |
|---|---|---|
| cli / hash / ir | FileNotFoundError | missing-bin |
| * | PermissionError | permission |
| * | TimeoutExpired / TimeoutError | timeout |
| * | UnicodeError | decode-error |
| * | JSONDecodeError | value-error |
| * | TypeError | type-error |
| * | ValueError / KeyError / IndexError | value-error |
| * | OSError | os-error |
| * | RuntimeError | cli-error |
| * | 그 외 | unexpected |

해시 자리의 예외는 관측으로 남기지 않고 `None` 해시로 접는다. 관측
카탈로그에 `hash-missing` 을 만들지 않은 이유: 기존
`same_body_hash` / `classify_pair` 가 `None` 만 보면 되기 때문이다. 새
라벨을 넣으면 관문을 두 번 쓰게 된다.

IR 자리의 예외도 새 라벨이 아니다. `ir_identity` 가 봉투 부재를 이미
`(False, None)` 으로 접는다. 예외를 그 계약에 맞춘다.

## 6. 보고 상태 표

| 상황 | contradictions | reviews | other-doc 집계 | ok | exit |
|---|---|---|---|---|---|
| 갈림 없음 | 0 | 0 | 0 | 참 | 0 |
| 해시 다름 | 0 | 0 | ≥1 | 참 | 0 |
| 본문 같음, IR 다름, 갈림 | 0 | ≥1 | 0 | 참 | 0 |
| 본문 같음, IR 같음, 갈림 | ≥1 | * | 0 | 거짓 | 3 |
| find-bin/walk 실패 | 0 | 0 | 0 | 참 | 1 |
| 모순 + 도구 실패 | ≥1 | * | * | 거짓 | 1 |

마지막 두 행을 첫째 행으로 접으면 게이트가 속는다. `ok` 는 모순 유무만
말하고, exit 1 이 도구 실패를 가린다.

## 7. 검증 명령

저장소 루트에서:

```bash
python -m unittest scripts.tests.test_gym_differential
python gym/tools/audit.py
git diff --shortstat upstream/devel
```

packs 를 고치지 않았으므로 audit 는 기존처럼 전 pack 통과여야 한다.
`cargo fmt --all` 은 이번 변경에 해당 없다.

## 8. 의도적으로 하지 않은 것

- pack · reference · profile 편집 없음.
- `release_diff.py` / `release_gate.py` 편집 없음. 형식축과 시간축을
  섞지 않는다.
- `classify_pair` 시그니처·네 칸 의미 변경 없음.
- `pick_twin_paths` 순위 규칙 변경 없음.
- `same_body_hash` 의 `None` 계약을 참으로 바꾸지 않음.
- 무작위·시간·호스트 경로를 관측에 넣지 않음.
- 새 PR 을 열지 않음. 같은 브랜치에 커밋한다.

## 9. 남은 빈틈

- 라이브 바이너리로 139쌍을 실제로 도는 전수 주행은 이 가지에서 돌리지
  않았다. 단위 시험은 주입 `run` 으로 관문만 고정한다.
- `run_cli` 기본 timeout 은 0(무제한)이다. `--cli-timeout` 을 주지 않으면
  기존처럼 한 문서가 행하면 스윕도 행한다. 그건 호출자 몫이다.
- `observation_from_result` 가 이미 예외 관측(kind+error)을 보면 그대로
  통과시킨다. 실제 CLI 봉투가 `kind` 와 `error` 를 동시에 가지는 일은
  info/explain/fields 계약에 없다.

## 10. 파일 목록

| 경로 | 역할 |
|---|---|
| `gym/tools/differential.py` | 교차형식 오라클. 예외 접기 + 관문 정직 |
| `scripts/tests/test_gym_differential.py` | 바이너리 없는 계약 시험 |
| `gym/docs/differential.md` | 규약 (정본 표) |
| `mydocs/working/gym_differential.md` | 작업 기록 (여기) |

## 11. 커밋 메시지 (초안)

```
test(gym): differential 예외 경로와 관문 정직 시험을 보강한다

CLI/walk/쓰기 예외를 관측·도구 오류로 접고,
본문 해시 부재와 IR 실패를 contradiction 으로 위장하지 않는다.
```

## 12. 관문 함수를 읽은 자리

`classify_pair` 본체는 다음 여섯 줄이 전부다. 이 가지가 여기를 다섯 값으로
늘리지 않았음을 기록으로 남긴다.

```
if not diverged:
    return None
if not body_same:
    return "other-doc"
return "contradiction" if ir_identical else "review"
```

해시를 모르는 입력은 `body_same=False` 로 넣는다. 넣는 순간 `None==None`
을 참으로 접으면 전 쌍이 침묵한다. IR 을 모르는 입력은
`ir_identical=False` 로 넣는다. 넣는 순간 못 본 내부 모순이 생긴다.
둘 다 도구의 거짓말이다.

`same_body_hash` 본체:

```
return left is not None and right is not None and left == right
```

이 한 줄을 `left == right` 로 줄이면 `None==None` 이 참이 된다. 시험이
그 축약을 막는다.

`pick_twin_paths` 본체는 같은 디렉터리 교집합이 있으면 그 중 사전순
첫째, 없으면 `path_rank` 최소. 비문자 필터만 앞에 붙였다. 유효 경로의
순위는 원 PR 과 같다.

## 13. 시험 목록 (이 가지에서 늘어난 것)

기존 반: TwinDiscovery / BodyHash / Observation / Classification /
CompareTwins / Report.

추가 반:

| 클래스 | 고정하는 거짓말 |
|---|---|
| ExceptionKindTests | 치명 예외를 관측으로 부르지 않음 |
| TruncateAndTimeoutHelperTests | 빈 문자열을 해시로 위장하지 않음 |
| ObservationEqualityEdgeTests | True==1, 바이트==문자열 오신고 금지 |
| PairingHonestyTests | walk 순서로 대표 경로를 바꾸지 않음 |
| HashHonestyTests | None==None 을 동일 문서로 부르지 않음 |
| ClassifyHonestyTests | 해시 부재를 contradiction 으로 부르지 않음 |
| CompareTwinsExceptionTests | 같은 오류 양쪽 = 갈림 아님 |
| IrFailureIsNotContradictionTests | ir-diff 예외를 내부 모순으로 부르지 않음 |
| MissingHashIsNotContradictionTests | 해시 예외를 내부 모순으로 부르지 않음 |
| ReportHonestyTests | other-doc 을 findings 에 넣지 않음 |
| WriteReportSafeTests | 쓰기 실패가 ok 를 뒤집지 않음 |
| RunCliSafeTests | CLI 예외가 도구를 죽이지 않음 |
| FindBinAndMainTests | main exit 0/1/3 |
| CatalogContractTests | SEVERITIES 에 other-doc 없음 |
| Generated*TableTests | 표의 대칭·역순 입력 |
| RenderSummaryHonestyTests | 도구 실패 요약이 침묵인 척하지 않음 |
| FindingHonestyScanTests | severity 가 irIdentical 과 묶임 |

## 14. 크기 게이트에 넣는 파일만

packs 와 Rust 는 이 가지의 대상이 아니다. `git add -A` 를 쓰지 않는다.
추가·수정 파일은 다음 네 개다.

1. `gym/tools/differential.py`
2. `scripts/tests/test_gym_differential.py`
3. `gym/docs/differential.md`
4. `mydocs/working/gym_differential.md`

`.github/workflows/ci.yml` 은 원 PR 이 이미 한 줄 넣었다. 이 가지에서
다시 건드리지 않는다.
