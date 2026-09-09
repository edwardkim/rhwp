---
kind: working
status: active
canonical: mydocs/working/gym_release_diff.md
last_verified: 2026-08-18
---

# gym 릴리스 차등 — 예외 경로와 분류 정직성 보강

Issue: #5234
PR: https://github.com/edwardkim/rhwp/pull/5248
Branch: `feat/gym-release-diff-hardening`
Date: 2026-08-18

## 1. 결론

`gym/tools/release_diff.py` 의 분류 삼원(stable / regression / surface-changed)은
그대로 두고, 바이너리·pack·쓰기 예외를 관측/오류 목록으로 접도록 닫았다.
표면을 재지 못하면 그 삼원 중 아무것으로도 위장하지 않는다. `probe-failed` 는
분류가 아니라 도구 실패 상태다.

이 가지는 새 PR 을 열지 않는다. 같은 브랜치에 이어서 밀어 #5248 을 키운다.

검증:

- `python -m unittest scripts.tests.test_gym_release_diff`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

## 2. 배경

원 PR(#5248)은 `classify` · `surface_changed` · `exit_for` · `build_report` 를
순수 함수로 분리하고, 관측 kind(`value`/`exit`/`nojson`/`digfail`/`no-cmd`/
`resolve-error`)와 JSON 봉투 필드(`classificationReason` · `exit` · `ok` ·
`reviewRequired` · `observationsSkipped`)를 넣었다. 대비 `upstream/devel`
삽입은 약 439줄이었다.

그 상태의 빈틈:

1. `capabilities_digest` 가 `subprocess.run` 예외를 그대로 올린다. 없는
   바이너리·권한·시간초과가 차등 도구 전체를 죽인다.
2. `observe` 가 `resolve_args` 의 일부 예외만 잡고, `run_cli` 의
   `TimeoutExpired` · `FileNotFoundError` · `PermissionError` 는 잡지 않는다.
3. `observation_from_result` 가 `dig` 의 `ValueError`(비정수 인덱스)를 잡지
   않아 깨진 경로 하나가 전 비교를 멈춘다.
4. `load_pack` / `discover_packs` 실패가 한 pack 에서 전체를 멈춘다.
5. digest 를 못 얻었을 때 분류를 어떻게 할지 계약이 없다. `None != "abc"` 를
   `surface_changed` 에 넣으면 표면 변경으로 오신고하고, `None != None` 은
   안정으로 오신고한다.
6. 카탈로그가 코드에만 있어 문서·시험이 같은 표를 공유하지 않는다.

분류 함수 자체를 네 값으로 늘리면 기존 게이트 계약
(`CLASSIFICATIONS` ↔ `EXIT_BY_CLASS`)이 깨진다. 그래서 `classify` 는 삼원을
유지하고, 표면을 모를 때는 호출하지 않는다.

## 3. 한 일

### 3.1 도구

`gym/tools/release_diff.py`

- `exception_kind` / `exception_observation` / `exception_probe` — 예외를
  관측/프로브 kind 로 접는다. `resolve` context 의 `FileNotFoundError` 는
  기존 계약대로 `resolve-error` 다. digest/cli 의 같은 예외는 `missing-bin`.
- `FATAL_EXCEPTIONS` — `KeyboardInterrupt` · `SystemExit` · `MemoryError` ·
  `GeneratorExit` 는 삼키지 않는다.
- `probe_capabilities` — capabilities 를 예외 없이 접는다. 실패 시
  `digest=None`.
- `can_classify_surface` — 두 digest 가 문자열일 때만 참.
- `classify_or_probe_failed` — 분류 가능하면 `classify`, 아니면
  `probe-failed`. `classify` 본체는 그대로다.
- `status_exit` / `reason_for` — 삼원은 `EXIT_BY_CLASS`, 도구 실패는 exit 1.
- `observation_from_result` — `ValueError` · `AttributeError` · 그 외 잡히는
  예외를 `digfail` 로 접는다. head 는 `truncate_head`.
- `observe` — `run_cli` 예외를 관측으로 접는다. 비-dict 검사는 `type-error`.
- `diff_task` — 한 검사의 예외는 그 행만 건너뛴다. 거짓 분기를 만들지 않는다.
- `load_pack_safe` / `discover_packs_safe` / `find_bin_safe` /
  `write_report_safe` / `compare_packs` — 한 pack·한 쓰기가 전체를 죽이지
  않는다.
- `build_report` — digest 가 문자열이 아니면 `build_probe_failed_report`.
  기존 문자열 digest 경로는 그대로 삼원을 낸다.
- `validate_report` — ok/review/surface/exit/divergences 정직 계약.
- `main` — 프로브 실패면 exit 1. 성공이면 0/2/3. pack 오류는 부가 필드.

### 3.2 시험

`scripts/tests/test_gym_release_diff.py`

- 기존 순수 시험(`ObservationTests` · `ClassificationTests` · `ReportTests`)
  유지.
- `ExceptionKindTests` — context 분기, 치명 예외 표지.
- `ProbeCapabilitiesTests` — 성공 digest 일치, missing/permission/timeout/
  OSError, KeyboardInterrupt 비삼킴.
- `ClassifyHonestyTests` — `classify` 가 `probe-failed` 를 내지 않음.
  확장 진릿값 표. 표면이 모든 분기 모양을 이김.
- `ObservationEqualityEdgeTests` — NaN, inf, 중첩, bool, 오류 페이로드.
- `ObserveExceptionTests` — run_cli/resolve/dig 예외 관측.
- `DiffTaskExceptionTests` — 한쪽만 CLI 실패면 분기, 양쪽 같으면 분기 아님.
- `ReportHonestyTests` — digest 부재를 삼원으로 부르지 않음.
  `validate_report` 가 ok/review/surface 거짓말을 잡음.
- `MainEntryExceptionTests` — main 의 exit 0/1/2/3.
- `GeneratedEqualityTableTests` / `GeneratedClassifyTableTests` — 표 계약.
- `ProbeFailedDisguiseTests` — 요약이 "표면 같음" 이라고 거짓말하지 않음.

### 3.3 문서

- `gym/docs/release_diff.md` — 분류 삼원, probe-failed, 관측 kind, 동일성,
  파일 연산 배제, JSON 봉투, 예외 자리, 종료 코드, 오검출 관문.
- `mydocs/working/gym_release_diff.md` — 이 기록.

pack JSON 은 건드리지 않았다.

## 4. 정직 조항 — 바꾸지 않은 것

다음 계약은 원 PR 과 같다. 시험이 다시 고정한다.

```
classify(False, 거짓값) → stable
classify(False, 참값)   → regression
classify(True,  *)      → surface-changed
```

- `CLASSIFICATIONS = ("stable", "regression", "surface-changed")`
- `EXIT_BY_CLASS = {stable: 0, regression: 3, surface-changed: 2}`
- `exit_for("skipped")` 는 `KeyError`
- `ok` 는 `stable` 과만 참
- `reviewRequired` 는 `surface-changed` 와만 참
- 숫자 `6` 과 `6.0` 은 같고, `True` 는 `1` 로 접히지 않는다
- 파일 연산자 네 개는 관측에서 뺀다
- CLI 플래그 `--old` `--new` `--agent` `--pack` `-o` 는 그대로다
  (`--digest-timeout` 만 추가)

`probe-failed` 를 `CLASSIFICATIONS` 나 `EXIT_BY_CLASS` 에 넣지 않았다.
넣으면 `test_exit_codes_match_gate_contract` 가 깨지고, 게이트가 도구 실패를
리뷰/차단으로 오독할 수 있다.

## 5. 예외 카탈로그

| context | 예외 | kind |
|---|---|---|
| resolve | FileNotFoundError | resolve-error |
| digest / cli | FileNotFoundError | missing-bin |
| * | PermissionError | permission |
| * | TimeoutExpired / TimeoutError | timeout |
| * | UnicodeError | decode-error |
| * | JSONDecodeError | value-error |
| * | KeyError / IndexError / AttributeError | digfail |
| * | TypeError | type-error |
| * | ValueError | value-error |
| * | OSError | os-error |
| * | RuntimeError | cli-error |
| * | 그 외 | unexpected |

`observe` 의 resolve 경로는 `exception_observation(..., context="resolve")` 를
쓴다. 기존 시험 `test_missing_hash_placeholder_is_an_observation` 은
`FileNotFoundError` → `{kind: resolve-error, error: FileNotFoundError}` 를
그대로 통과해야 한다.

## 6. 보고 상태 표

| 상황 | classification | exit | ok | review | surfaceChanged |
|---|---|---|---|---|---|
| digest 같음, 분기 0 | stable | 0 | 참 | 거짓 | 거짓 |
| digest 같음, 분기 ≥1 | regression | 3 | 거짓 | 거짓 | 거짓 |
| digest 다름, 분기 0 | surface-changed | 2 | 거짓 | 참 | 참 |
| digest 다름, 분기 ≥1 | surface-changed | 2 | 거짓 | 참 | 참 |
| digest 한쪽/양쪽 없음 | probe-failed | 1 | 거짓 | 거짓 | 거짓 |

마지막 행을 둘째 행이나 셋째 행으로 접으면 게이트가 속는다.

## 7. 검증 명령

저장소 루트(`rhwp-scaffold-final`)에서:

```bash
python -m unittest scripts.tests.test_gym_release_diff
python gym/tools/audit.py
git diff --shortstat upstream/devel
```

packs 를 고치지 않았으므로 audit 는 기존처럼 전 pack 통과여야 한다.
`cargo fmt --all` 은 이번 변경에 해당 없다.

## 8. 의도적으로 하지 않은 것

- pack · reference · profile 편집 없음.
- `release_gate.py` 편집 없음. 게이트는 차등 JSON 을 읽기만 한다. 프로브
  실패 시 classification 이 `probe-failed` 가 되므로, 게이트가 그 값을
  모르면 자기 쪽에서 다루면 된다. 차등 오라클이 안정으로 위장하는 편이
  더 위험하다.
- `classify` 시그니처·삼원 의미 변경 없음.
- 무작위·시간·호스트 경로를 관측에 넣지 않음.
- 새 PR 을 열지 않음. 같은 브랜치에 커밋한다.

## 9. 남은 빈틈

- 라이브 바이너리 두 개를 실제로 돌리는 자기-대조는 이 가지에서 돌리지
  않았다. 커밋된 `gym/release-diff.json` 이 있으면 단위 시험이 읽고, 없으면
  skip 한다.
- `run_cli` 자체에 timeout 인자를 넣지는 않았다. 관측 쪽 timeout 은
  `run_cli` 가 올린 `TimeoutExpired` 를 접는 계약이다. 러너가 시간제한을
  안 걸면 이 도구의 observe 도 행할 수 있다. 그건 runner 의 몫이다.
- capabilities 가 비정상 종료해도 stdout 해시는 기존처럼 계산한다. 빈
  출력의 digest 는 오류가 아니라 고정값이다. 양쪽이 같이 빈 출력이면
  표면은 같다.

## 10. 파일 목록

| 경로 | 역할 |
|---|---|
| `gym/tools/release_diff.py` | 차등 오라클. 예외 접기 + 분류 정직 |
| `scripts/tests/test_gym_release_diff.py` | 바이너리 없는 계약 시험 |
| `gym/docs/release_diff.md` | 규약 (이 문서의 정본 표) |
| `mydocs/working/gym_release_diff.md` | 작업 기록 (여기) |

## 11. 커밋 메시지 (초안)

```
test(gym): release_diff 예외 경로와 분류 정직 시험을 보강한다

capabilities/CLI/pack 예외를 관측·프로브 오류로 접고,
표면을 모를 때는 stable/regression/surface-changed 로 위장하지 않는다.
```

## 12. 분류 함수를 읽은 자리

`classify` 본체는 다음 열 줄이 전부다. 이 가지가 여기를 네 값으로 늘리지
않았음을 기록으로 남긴다.

```
if surface:
    return "surface-changed"
if divergences:
    return "regression"
return "stable"
```

표면을 모르는 입력은 이 함수에 넣지 않는다. 넣는 순간 `None != "abc"` 가
표면 변경이 되고, `None != None` 이 안정이 된다. 둘 다 도구의 거짓말이다.

## 13. 시험 목록 (이 가지에서 늘어난 것)

기존 반: Observation / Equality / DiffTask / Classification / Report /
CommittedReport.

추가 반:

| 클래스 | 고정하는 거짓말 |
|---|---|
| ExceptionKindTests | resolve 의 FileNotFound 를 missing-bin 으로 부르지 않음 |
| TruncateAndDigestHelperTests | 빈 문자열을 digest 로 위장하지 않음 |
| ProbeCapabilitiesTests | 예외를 digest 문자열로 접지 않음 |
| ClassifyHonestyTests | classify 가 네 번째 값을 내지 않음 |
| ObservationEqualityEdgeTests | True==1, 바이트==문자열 오신고 금지 |
| ObserveExceptionTests | CLI 예외가 도구를 죽이지 않음 |
| DiffTaskExceptionTests | 같은 오류 양쪽 = 분기 아님 |
| ReportHonestyTests | digest 부재 → 삼원 금지 |
| WriteAndSummaryExceptionTests | 프로브 실패 요약이 "표면 같음" 이라 하지 않음 |
| PackLoadSafeTests | 한 pack 실패가 분류를 뒤집지 않음 |
| MainEntryExceptionTests | main exit 0/1/2/3 |
| CatalogContractTests | 삼원 튜플·이유 문구가 승자를 가리지 않음 |
| GeneratedEqualityTableTests | 표의 대칭 |
| GeneratedClassifyTableTests | 표에 probe-failed 행 없음 |
| ProbeFailedDisguiseTests | None digest 를 surface-changed/stable/regression 으로 부르지 않음 |
| HonestyInvariantScanTests | ok/review/surface 상호 일관 |

## 14. 크기 게이트에 넣는 파일만

packs 와 Rust 는 이 가지의 대상이 아니다. `git add -A` 를 쓰지 않는다.
추가·수정 파일은 다음 네 개다.

1. `gym/tools/release_diff.py`
2. `scripts/tests/test_gym_release_diff.py`
3. `gym/docs/release_diff.md`
4. `mydocs/working/gym_release_diff.md`
