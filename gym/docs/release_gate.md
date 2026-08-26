---
kind: guide
status: active
canonical: gym/docs/release_gate.md
last_verified: 2026-08-18
---

# gym 릴리스 게이트 규약

이 문서는 `gym/tools/release_gate.py` 의 **판정 사원**, **예외 경로 계약**,
**워크플로 전제**를 고정한다. 작업 기록은
[`mydocs/working/gym_release_gate.md`](../../mydocs/working/gym_release_gate.md)
를 본다. 시험 계약은 `scripts/tests/test_gym_release_gate_workflow.py` 와
`scripts/tests/test_gym_release_gate_exceptions.py` 가 기계로 고정한다.

릴리스 차등(`release_diff.py`)은 오라클이다. 게이트는 그 오라클을 읽어
파이프라인 종료 코드로 묶는다. 오라클이 표면을 모르는데 안정이라고 쓰면
게이트도 속는다. 그래서 `probe-failed` 는 삼원 밖에 두고 fail(1) 로 받는다.

## 1. 왜 이 기둥이 필요한가

회귀 도구가 도구로만 있으면 사람이 기억해서 돌려야 한다. 릴리스 파이프라인에
물리면 잊어도 돈다. 게이트가 묶는 것은 세 층이다.

1. **종점 무결성** — `discriminate.py`. 일 안 한 제출이 만점을 받으면 벤치가
   거짓이다. 워크플로가 게이트보다 먼저 돌리고, exit 1 이면 잡을 닫는다.
2. **경로 무결성** — `trajectory.py`. 마지막 스텝을 빼도 통과하면 연극이다.
   같은 자리에서 먼저 돈다.
3. **시간축 차등 + 원장** — `release_diff.py` 와 `leaderboard.py verify`.
   게이트 러너가 이 둘을 하나의 판정으로 접는다.

이 도구는 새 CLI 를 열지 않는다. 새 pack 을 만들지 않는다. 차등 오라클의
분류 삼원(stable / regression / surface-changed)을 네 값으로 늘리지 않는다.
게이트가 더하는 것은 **파이프라인 판정**과 **예외를 위장하지 않는 접기**다.

## 2. 사용

```bash
python gym/tools/release_gate.py --old <직전 태그 바이너리> --new target/debug/rhwp
python gym/tools/release_gate.py --new target/debug/rhwp
python gym/tools/release_gate.py --old old/rhwp --new new/rhwp --pack core-cli
python gym/tools/release_gate.py --new target/debug/rhwp --no-leaderboard -o gate.json
python gym/tools/release_gate.py --new target/debug/rhwp --github-summary
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--old` | (없음) | 직전 릴리스 rhwp. 없거나 파일이 없으면 차등 생략. |
| `--new` | `find_bin` | 현재 rhwp. 없으면 fail. 생략이 아니다. |
| `--agent` | `claude-fable-5` | 차등에 넘기는 제출물 루트. |
| `--pack` | 전체 | 반복 지정 가능. 차등에 그대로 전달. |
| `--no-leaderboard` | off | 원장 검증 생략. |
| `--github-summary` | off | `GITHUB_STEP_SUMMARY` 에 표 추가. |
| `-o` / `--out` | (없음) | 판정 JSON. UTF-8 · BOM 없음 · LF. |

`--discriminate-fail`, `--preflight`, `--probe-failed` 같은 플래그는 없다.
판별 실패는 워크플로가 게이트보다 먼저 막거나, 프로그래매틱 `preflight=`
인자로만 넘긴다. CLI 표면을 늘리지 않는다.

프로브 명령은 게이트가 부르지 않는다. 차등 도구가 `rhwp capabilities` 를
부른다. 게이트는 그 보고의 `classification` 만 읽는다.

## 3. 판정 사원

게이트가 내는 판정은 넷이다. 차등 삼원과 1:1 이 아니다.

| 판정 | exit | 언제 |
|---|---|---|
| `pass` | 0 | 차등 stable 또는 skipped(구 바이너리 없음) + 원장 무결 |
| `fail` | 1 | 도구/전제 실패. 신 바이너리 부재, 보고 손상, probe-failed, 판별 실패 |
| `review` | 2 | 차등 surface-changed. 사람 판정. **자동 차단 아님** |
| `block` | 3 | 차등 regression, 또는 리더보드 체인 파손 |

규칙:

1. **regression 만 차단한다.** 표면 변경은 리뷰 신호다.
2. **도구 실패는 분류가 아니다.** probe-failed 를 pass 로 부르면 거짓말이다.
3. **구 바이너리 부재는 실패가 아니다.** 직전 태그를 안 빌드한 것과 현재
   릴리스가 없는 것은 다르다.
4. **신 바이너리 부재는 실패다.** skipped 로 위장하지 않는다.
5. **판별 실패는 회귀가 아니다.** 오라클이 약한 것이지 관측이 갈린 것이 아니다.

`ok` 는 `pass` 와만 참. `reviewRequired` 는 `review` 와만 참. `blocked` 는
`block` 과만 참. `failed` 는 `fail` 과만 참. `validate_verdict` 가 이 표를
검사한다.

## 4. 차등 삼원을 게이트가 읽는 법

차등 오라클의 `classify` 는 오직 세 값만 낸다. 게이트는 그 값을 판정으로
옮긴다.

| 차등 분류 | 표면 | 관측 분기 | 게이트 이유 | 판정 | exit |
|---|---|---|---|---|---|
| `stable` | 아니오 | 아니오 | `stable` | pass | 0 |
| `regression` | 아니오 | 예 | `regression` | block | 3 |
| `surface-changed` | 예 | 아니오 | `surface-changed` | review | 2 |
| `surface-changed` | 예 | 예 | `surface-changed` | review | 2 |

추가 상태 — 삼원이 아니다.

| 차등 상태 | 게이트 이유 | 판정 | exit | 왜 |
|---|---|---|---|---|
| `skipped` (구 없음) | `missing-old-bin` | pass | 0 | 부재≠실패 |
| `probe-failed` | `probe-failed` | fail | 1 | 표면을 모름 |
| 보고 파일 없음 | `diff-report-missing` | fail | 1 | 오라클이 말을 안 함 |
| 보고가 JSON 이 아님 | `diff-report-invalid` | fail | 1 | 오라클이 깨짐 |
| 차등 도구 예외 | `diff-tool-error` | fail | 1 | 도구가 죽음 |

오라클이 실수로 `classification=regression` 과 `surfaceChanged=true` 를 같이
내면 게이트는 **review** 다. 표면이 회귀보다 앞선다는 #4661 정직 조항을
게이트가 다시 적용한다. `surface_wins_over_regression` 이 그 자리이다.

## 5. 예외 경로 — 이슈가 지목한 네 자리

#5259 가 고정하라고 한 자리는 넷이다. 각각이 다른 종료 코드와 다른 이유다.
한 칸으로 접으면 운영자가 다음 행동을 고르지 못한다.

### 5.1 구 바이너리 부재 (`missing-old-bin`)

직전 태그 바이너리를 안 빌드했거나 `--old` 를 안 줬다.

- 차등 분류: `skipped`
- 판정: `pass` (원장이 무결하면)
- exit: 0
- 워크플로: `if [ -f ./rhwp-old-bin ]` 가 거짓이면 `--old` 없이 게이트를 부른다

부재를 실패로 위장하지 않는다. 첫 태그, 수동 실행에서 old_ref 를 비운 경우가
정상 경로다.

### 5.2 신 바이너리 부재 (`missing-new-bin`)

현재 릴리스 실행 파일이 없다. `find_bin` 이 돌려준 경로가 디스크에 없다.

- 차등: 돌리지 않는다
- 원장: 돌리지 않는다
- 판정: `fail`
- exit: 1

구 바이너리 부재와 대칭으로 생략하면, "지금 무엇을 릴리스하는가" 가 없는
상태를 안정으로 위장한다. 그래서 이 자리만 fail 이다.

빈 문자열 `--new ""` 도 같은 이유다. PATH 이름 `rhwp` 만 있고 파일이 없으면
역시 fail 이다. 실행 파일을 알려면 `--new` 에 실제 경로를 준다.

### 5.3 판별 감사 실패 (`discriminate-fail`)

`discriminate.py` 가 0 이 아닌 종료 코드를 냈다. 일 안 한 제출이 만점을 받은
과제(약한 오라클)가 하나라도 있다.

- 워크플로: 게이트 스텝에 도달하지 않는다. 기본 `set -e` 로 잡이 실패한다.
- 러너: `preflight={"tool":"discriminate.py","exit":1}` 을 받으면 `fail` / exit 1
- **regression 으로 부르지 않는다.** 관측이 갈린 것이 아니다.
- **surface-changed 로 부르지 않는다.** 명령 표면 문제가 아니다.
- **pass 로 부르지 않는다.** 벤치가 거짓인데 릴리스를 통과시키면 안 된다.

판별 실패가 회귀보다 앞선다. 오라클이 약하면 차등 숫자도 믿을 수 없다.
`decide_verdict` 우선순위 0 이 이 자리다.

게이트 CLI 에 `--discriminate` 를 달지 않는다. 워크플로가 이미 먼저 돌린다.
두 번 돌리면 CI 가 두 배가 되고, 플래그를 늘리면 "새 CLI 없음" 계약을 깬다.

### 5.4 표면 변경 대 회귀 (`surface-changed` vs `regression`)

같은 "관측이 갈렸다" 라도 표면이 바뀌었으면 회귀가 아니다.

| 표면 digest | 관측 분기 | 부르는 이름 | 사람 다음 행동 |
|---|---|---|---|
| 같음 | 없음 | stable | 없음. 릴리스해도 된다 |
| 같음 | 있음 | regression | 막는다. 동작이 바뀌었다 |
| 다름 | 없음 | surface-changed | 리뷰. 명령이 늘거나 빠졌다 |
| 다름 | 있음 | surface-changed | 리뷰. 관측 변화는 표면 탓일 수 있다 |

한컴 정답지가 없다. 게이트는 어느 쪽이 옳은지 말하지 않는다. 표면이 바뀐
릴리스를 자동으로 막으면, 의도된 명령 추가가 전부 차단된다. 그래서 review
이지 block 이 아니다.

리더보드 체인 파손은 이 표 밖에 있다. 원장 무결은 사람 리뷰로 넘기지 않는다.
surface-changed + 원장 파손 = block.

## 6. 우선순위

`decide_verdict` 가 이유 목록에서 하나를 고른다. 앞이 이긴다.

| 순위 | 이유 | 판정 |
|---|---|---|
| 0 | `discriminate-fail` · `trajectory-fail` · `audit-fail` | fail |
| 1 | `missing-new-bin` · `probe-failed` · `diff-report-*` · `diff-tool-error` · `leaderboard-error` | fail |
| 2 | `regression` · `leaderboard-broken` | block |
| 3 | `surface-changed` | review |
| 4 | `stable` · `skipped` · `missing-old-bin` | pass |

읽기:

- 판별이 실패했는데 차등이 회귀여도 fail 이다. 오라클을 먼저 고친다.
- 신 바이너리가 없는데 원장이 파손돼도 fail 이다. 비교 대상이 없다.
- 표면이 바뀌었는데 원장이 파손되면 block 이다. 원장은 리뷰 대상이 아니다.
- 구 바이너리만 없으면 pass 다. 원장이 파손되면 그때는 block.

## 7. 이유 카탈로그

`REASONS` 튜플과 문서·시험이 같은 표를 본다.

| 이유 | 판정 | 한 줄 |
|---|---|---|
| `stable` | pass | 표면과 관측이 같다 |
| `skipped` | pass | 차등을 돌리지 않았다 (구 없음과 동의) |
| `missing-old-bin` | pass | 구 바이너리 없음. 차등 생략 |
| `missing-new-bin` | fail | 신 바이너리 없음. 현재 릴리스 부재 |
| `discriminate-fail` | fail | 약한 오라클. 차등 분류 아님 |
| `trajectory-fail` | fail | 연극 과제. 차등 분류 아님 |
| `audit-fail` | fail | 그 외 전제 감사 실패 |
| `probe-failed` | fail | 표면을 못 잼. 삼원으로 위장 금지 |
| `diff-report-missing` | fail | 차등 JSON 이 없다 |
| `diff-report-invalid` | fail | 차등 JSON 이 깨졌다 |
| `diff-tool-error` | fail | 차등 도구가 예외로 죽음 |
| `surface-changed` | review | 명령 표면이 바뀜. 차단 아님 |
| `regression` | block | 표면 같고 관측이 갈림 |
| `leaderboard-broken` | block | 원장 체인 파손 |
| `leaderboard-error` | fail | 원장 도구가 예외로 죽음 |
| `write-error` | fail | 판정 파일을 못 씀 (종료 코드는 기존 판정 유지 가능) |
| `unexpected` | fail | 카탈로그 밖 |

`write-error` 는 `main` 이 판정을 이미 계산한 뒤에 난다. 디스크가 가득 찼다고
회귀를 안정으로 바꾸지 않는다. 콘솔 종료 코드는 계산된 판정을 따른다.
`-o` 파일이 비거나 부분만 쓰여 있으면 오류 기록을 봉투에 남긴다.

## 8. 예외를 접는 자리

감사기(이 게이트) 자신은 한 서브프로세스의 예외로 멈추지 않는다. 치명
예외만 다시 올린다.

| 자리 | 잡는 것 | 접는 곳 |
|---|---|---|
| `find_bin_safe` | OSError 등 | 바이너리 기록 status=error |
| `path_exists_safe` | OSError | 없는 것으로 본다 |
| `run_tool_safe` | timeout · missing-bin · OSError | 도구 오류 기록 |
| `load_json_safe` | FileNotFound · JSONDecode · OSError | 보고 손상 이유 |
| `remove_safe` | OSError | 무시. 판정을 바꾸지 않음 |
| `write_verdict_safe` | OSError | `writeError` |
| `write_github_summary` | OSError | 반환만. 판정 유지 |
| `gate` 전제 | `preflight` 입력 | `fold_preflight` |

삼키지 않는 것: `KeyboardInterrupt` · `SystemExit` · `MemoryError` ·
`GeneratorExit`. 사용자가 끊었는데 안정 보고를 내면 거짓말이다.

`exception_kind` 는 context 를 본다. 같은 `FileNotFoundError` 라도
바이너리 자리면 `missing-bin`, 차등 보고 자리면 `diff-report-missing` 이다.

## 9. JSON 봉투

`kind=gymReleaseGate`, `schemaVersion=1.0`. 기존 필드 의미는 그대로다.
#5259 가 이유를 가리키려고 칸을 더했다. 키 집합은 시험이 `VERDICT_KEYS` 로
고정한다.

| 키 | 형 | 의미 |
|---|---|---|
| `kind` | str | 항상 `gymReleaseGate` |
| `schemaVersion` | str | 항상 `1.0` |
| `diff` | obj | 차등 요약 또는 skipped/unavailable |
| `leaderboard` | obj | `{ok, exit}` 또는 `{ok: null, reason}` |
| `verdict` | str | pass / review / block / fail |
| `exit` | int | 0 / 1 / 2 / 3 |
| `reason` | str | 이긴 이유 코드 |
| `reasons` | list | 모인 이유. 우선순위 입력 |
| `ok` | bool | pass 와만 참 |
| `reviewRequired` | bool | review 와만 참 |
| `blocked` | bool | block 과만 참 |
| `failed` | bool | fail 과만 참 |
| `old` / `new` | obj | `{role, given, resolved, status, reason}` |
| `preflight` | obj | `{ok, audits, reasons, failed}` |
| `errors` | list | 접힌 예외 기록 |

`diff` 가 성공했을 때 들어 있는 칸:

| 키 | 의미 |
|---|---|
| `classification` | 삼원 또는 skipped / probe-failed / unavailable |
| `divergences` | 관측 분기 수 |
| `surfaceChanged` | 표면이 달랐나 |
| `tasksCompared` | 비교한 과제 수 |
| `reasonCode` | 게이트가 읽은 이유 |
| `toolExit` | 차등 도구의 종료 코드 |

`validate_verdict` 가 정직 계약을 검사한다.

- 사원일 때 `exit` · `ok` · `reviewRequired` · `blocked` · `failed` 가 표와
  같아야 한다.
- `pass` 인데 `reason` 이 `regression` / `surface-changed` /
  `discriminate-fail` / `missing-new-bin` / `probe-failed` 이면 거짓말이다.
- `review` 인데 `reason` 이 `surface-changed` 가 아니면 거짓말이다.
- `block` 인데 `reason` 이 `regression` / `leaderboard-broken` 이 아니면
  거짓말이다.
- `fail` 인데 `reason` 이 `regression` 또는 `surface-changed` 이면 거짓말이다.
- `probe-failed` 분류를 pass / review / block 으로 부르면 거짓말이다.

## 10. 워크플로 계약

독립 워크플로 `.github/workflows/gym-release-gate.yml`. 릴리스 본체
(`release-binary.yml`)는 건드리지 않는다.

순서:

1. 현재 커밋에서 `cargo build --bin rhwp`
2. **Discrimination audit** — `python3 gym/tools/discriminate.py --bin target/debug/rhwp`
3. (old_ref 가 있을 때만) 구 태그 worktree 빌드 → `./rhwp-old-bin`
4. **Trajectory necessity audit** — `python3 gym/tools/trajectory.py --bin target/debug/rhwp`
5. **Run release gate** — old 파일이 있으면 `--old`, 항상 `--new`
6. 판정 JSON 을 아티팩트로 업로드 (`if: always()`)

고정하는 것:

- `workflow_dispatch` + `push.tags: v*`
- `permissions.contents: read` 만. write 권한 없음
- 판별·트라젝토리가 게이트보다 먼저
- 판별 스텝에 `--old` 없음 (현재 벤치만 본다)
- 판별 스텝에 `continue-on-error` / `|| true` 없음
- 게이트 스텝도 종료 코드를 삼키지 않음
- old 파일이 없으면 `--old` 없이 게이트를 부름 (부재≠실패)
- 업로드는 실패해도 돈다

판별이 exit 1 을 내면 5번에 도달하지 않는다. 그게 `discriminate-fail` 의
운영 경로다. 러너의 `preflight=` 는 같은 이유를 시험이 바이너리 없이
재현하려고 둔 입구이지, 새 CLI 가 아니다.

## 11. GitHub step summary

`--github-summary` 이고 `GITHUB_STEP_SUMMARY` 가 있으면 표를 덧붙인다.
환경 변수가 없으면 아무 것도 하지 않는다 (실패가 아니다).

표 칸: 릴리스 차등, 리더보드 체인, 신/구 바이너리 상태, 전제 감사, 이유.

이유별 주석:

- `surface-changed` — 차단이 아니라 리뷰 신호
- `discriminate-fail` — 회귀가 아니다. 약한 오라클
- `missing-new-bin` — 차등 생략이 아니다
- `probe-failed` — 표면을 모르면 분류하지 않는다
- `regression` — 어느 쪽이 한컴과 맞는지는 말하지 않는다. 차단만 한다

쓰기 실패는 판정을 바꾸지 않는다.

## 12. 오검출 관문 요약

게이트가 거짓말하지 않도록 지키는 문:

1. **표면이 회귀보다 앞선다.** 오라클이 실수해도 게이트가 다시 적용한다.
2. **도구 실패는 삼원이 아니다.** probe-failed / 보고 손상 / 도구 예외는
   fail(1) 이다.
3. **구 부재 ≠ 신 부재.** 한쪽만 skipped 다.
4. **판별 실패 ≠ 회귀.** 오라클 결함과 동작 변화를 같은 exit 로 묶지 않는다.
5. **원장 파손은 리뷰가 아니다.** surface-changed 여도 체인이 깨지면 block.
6. **치명 예외는 삼키지 않는다.**
7. **한 쓰기가 판정을 뒤집지 않는다.**
8. **CLI 표면을 늘리지 않는다.** 전제는 워크플로와 `preflight=` 뿐이다.

## 13. 시험이 고정하는 것

```bash
python -m unittest scripts.tests.test_gym_release_gate_workflow
python -m unittest scripts.tests.test_gym_release_gate_exceptions
```

바이너리 없이 돈다. `run_tool` · `os.path.exists` · `runner.find_bin` 을
목으로 갈아끼운다.

고정하는 축:

- 워크플로 존재, 본체 비침습, 수동+태그, 읽기 권한, 요약 플래그
- 판별·트라젝토리가 게이트보다 앞섬
- 판별 실패가 잡을 닫음 (`continue-on-error` 없음)
- 러너 사원: stable→0, surface-changed→2, regression→3
- 구 부재 → skipped / pass. 신 부재 → fail / 1
- 판별 실패 → fail / 1. block/review 로 위장 금지
- 표면 × 분기 × 전제 × 원장 생성 표. 위장 조합 없음
- `probe-failed` / 깨진 JSON / 도구 OSError → fail
- 원장 검증이 `--bin <new>` 를 그대로 씀
- `validate_verdict` 정직 계약
- `main` 의 exit 0/1
- CLI 에 새 플래그 없음
- 치명 예외 비삼킴
- 문서 두 파일이 이유 코드를 포함

## 14. 이 도구가 하지 않는 것

- 새 CLI 플래그를 추가하지 않는다.
- 새 gym pack 을 만들지 않는다.
- `discriminate.py` · `trajectory.py` · `release_diff.py` · `fuzz_corpus.py`
  를 고치지 않는다.
- 자동화 pack · core-cli · casual-rides 과제 JSON 을 고치지 않는다.
- 한컴 문서가 맞는지 틀리는지 말하지 않는다.
- 어느 바이너리가 "더 옳은지" 고르지 않는다.
- 표면이 바뀐 릴리스를 자동으로 막지 않는다.
- 판별 실패를 회귀로 부르지 않는다.
- 신 바이너리 부재를 차등 생략으로 부르지 않는다.
- 치명 예외를 삼켜 성공인 척하지 않는다.
- 릴리스 본체 워크플로에 침습하지 않는다.

## 15. 관련 기둥

| 기둥 | 도구 | 질문 |
|---|---|---|
| 종점 무결성 | `discriminate.py` | 일 안 한 제출이 만점을 받나? |
| 경로 무결성 | `trajectory.py` | 마지막 스텝을 빼도 통과하나? |
| 도구 강건성 | `robustness.py` | 손상 입력에 rhwp 가 패닉·행 하나? |
| 릴리스 차등 | `release_diff.py` | 두 바이너리가 같은 관측을 내나? |
| 릴리스 게이트 | `release_gate.py` | 차등 + 원장을 파이프라인 판정으로 묶나? |

차등은 오라클이다. 게이트는 오라클을 읽는다. 오라클이 표면을 모르는데
안정이라고 쓰면 게이트도 속는다. 그래서 `probe-failed` 를 삼원 밖에 둔다.
판별은 오라클의 오라클이다. 그게 실패하면 차등 숫자도 내려놓는다.

## 16. 봉투 표본

아래는 시험이 조립하는 최소 표본이다. 필드의 참/거짓이 판정과 어긋나면
`validate_verdict` 가 거부한다.

### 16.1 pass — stable

```json
{
  "kind": "gymReleaseGate",
  "schemaVersion": "1.0",
  "diff": {
    "classification": "stable",
    "divergences": 0,
    "surfaceChanged": false,
    "tasksCompared": 91,
    "reasonCode": "stable"
  },
  "leaderboard": {"ok": true, "exit": 0},
  "verdict": "pass",
  "exit": 0,
  "reason": "stable",
  "ok": true,
  "reviewRequired": false,
  "blocked": false,
  "failed": false
}
```

자기-대조(같은 바이너리를 `--old` 와 `--new` 에 넣음)는 이 모양이어야 한다.

### 16.2 pass — 구 바이너리 없음

```json
{
  "diff": {
    "classification": "skipped",
    "reason": "구 바이너리 없음 — 차등 생략(직전 태그 미빌드)",
    "reasonCode": "missing-old-bin"
  },
  "leaderboard": {"ok": null, "reason": "커밋된 리더보드 없음 — 검증 생략"},
  "verdict": "pass",
  "exit": 0,
  "reason": "missing-old-bin"
}
```

첫 태그, 수동 실행에서 old_ref 를 비운 정상 경로다.

### 16.3 fail — 신 바이너리 없음

```json
{
  "diff": {
    "classification": "unavailable",
    "reasonCode": "missing-new-bin"
  },
  "new": {"status": "missing", "reason": "not-found"},
  "verdict": "fail",
  "exit": 1,
  "reason": "missing-new-bin",
  "ok": false,
  "failed": true
}
```

skipped 가 아니다. 현재 릴리스가 없다.

### 16.4 fail — 판별 감사 실패

```json
{
  "preflight": {
    "ok": false,
    "failed": true,
    "reasons": ["discriminate-fail"],
    "audits": [
      {"tool": "discriminate", "exit": 1, "ok": false, "reason": "discriminate-fail"}
    ]
  },
  "verdict": "fail",
  "exit": 1,
  "reason": "discriminate-fail",
  "failed": true
}
```

워크플로 경로에서는 이 봉투가 안 나올 수 있다 — 잡이 게이트 전에 죽는다.
러너가 같은 이유를 재현할 수 있어야 시험이 위장을 잡는다.

### 16.5 review — surface-changed

```json
{
  "diff": {
    "classification": "surface-changed",
    "divergences": 70,
    "surfaceChanged": true,
    "reasonCode": "surface-changed"
  },
  "verdict": "review",
  "exit": 2,
  "reason": "surface-changed",
  "reviewRequired": true,
  "blocked": false
}
```

관측이 70 갈려도 차단이 아니다. 명령 표면이 바뀐 릴리스다.

### 16.6 block — regression

```json
{
  "diff": {
    "classification": "regression",
    "divergences": 4,
    "surfaceChanged": false,
    "reasonCode": "regression"
  },
  "verdict": "block",
  "exit": 3,
  "reason": "regression",
  "blocked": true,
  "reviewRequired": false
}
```

표면이 같은데 쪽수가 6→7 이면 순수 동작 변화다. 어느 쪽이 한컴과 맞는지는
이 게이트가 말하지 않는다.

### 16.7 fail — probe-failed

```json
{
  "diff": {
    "classification": "probe-failed",
    "reasonCode": "probe-failed"
  },
  "verdict": "fail",
  "exit": 1,
  "reason": "probe-failed",
  "failed": true,
  "ok": false,
  "reviewRequired": false,
  "blocked": false
}
```

표면을 모르면 분류하지 않는다. pass 도 review 도 block 도 아니다.

### 16.8 block — 원장 파손

```json
{
  "diff": {"classification": "skipped", "reasonCode": "missing-old-bin"},
  "leaderboard": {"ok": false, "exit": 3},
  "verdict": "block",
  "exit": 3,
  "reason": "leaderboard-broken"
}
```

차등을 안 돌려도 원장이 깨지면 막는다.

## 17. 운영자가 읽는 법

종료 코드를 보고 다음 행동을 고른다.

| exit | 하는 일 |
|---|---|
| 0 | 릴리스를 진행해도 된다. 구 바이너리가 없었으면 다음 태그부터 차등이 돈다. |
| 1 | 도구/전제를 고친다. 약한 오라클이면 과제를 고친다. 바이너리 경로를 확인한다. 차등 보고가 깨졌으면 차등 도구를 본다. **동작을 되돌리는 자리가 아니다.** |
| 2 | 변경 로그를 읽고 의도된 표면 변경인지 사람이 판정한다. 의도면 진행, 아니면 명령을 되돌린다. |
| 3 | 관측이 갈렸거나 원장이 깨졌다. 릴리스를 막는다. 차등 JSON 의 `diffs` 또는 원장 verify 출력을 본다. |

exit 1 과 exit 3 을 같은 "실패" 로 접으면, 약한 오라클과 쪽수 회귀를 같은
티켓으로 보낸다. 그래서 사원이 넷이다.

## 18. 로컬에서 재현하는 법

바이너리 없이 계약을 확인한다.

```bash
python -m unittest scripts.tests.test_gym_release_gate_workflow -q
python -m unittest scripts.tests.test_gym_release_gate_exceptions -q
python gym/tools/audit.py
```

라이브 경로(실제 rhwp 두 개)는 기존과 같다.

```bash
python gym/tools/release_gate.py --old <구> --new target/debug/rhwp -o gate.json
```

`gate.json` 의 `verdict` / `exit` / `reason` 이 이 문서의 표와 같아야 한다.
`validate_verdict` 를 파이썬에서 부르면 위장 조합을 기계가 거부한다.

라이브 스윕은 이 가지의 완료 조건이 아니다. 단위 시험이 예외 경로를 고정하고,
audit.py 가 pack 정합을 본다. 새 pack 이 없으므로 audit 는 기존과 같아야 한다.

## 19. 변경을 열 때

이 파일을 고치려면 같은 커밋에서 시험과 이유 카탈로그를 같이 고친다.

- 새 이유 코드를 넣으면 `REASONS` · `VERDICT_BY_REASON` · `REASON_TEXT` ·
  이 문서 7절 · 시험 표를 같이 늘린다.
- 새 CLI 플래그를 넣지 않는다. 필요하면 이슈를 새로 연다.
- 판정 사원에 다섯 번째 값을 넣지 않는다. 새 상황은 이유 코드로 접는다.
- `discriminate.py` 를 이 가지에서 고치지 않는다. 종료 코드 계약만 읽는다.

## 20. 용어

| 말 | 뜻 |
|---|---|
| 삼원 | 차등 오라클의 stable / regression / surface-changed |
| 사원 | 게이트의 pass / review / block / fail |
| 전제 감사 | 게이트보다 먼저 도는 판별·트라젝토리 |
| 약한 오라클 | 일 안 한 제출이 만점을 받는 과제 |
| 연극 | 마지막 스텝을 빼도 통과하는 다단계 과제 |
| 부재≠실패 | 구 바이너리가 없는 것을 오류로 부르지 않는 결 |
| 정직 조항 | 무엇이 바뀌었나를 가리키고 어느 쪽이 옳은지는 말하지 않는다 |
| 위장 | 한 이유를 다른 판정으로 부르는 것. 시험이 거부한다 |

## 21. 결정 트리 (운영자가 따라가는 순서)

게이트가 실제로 걷는 순서와 같다. 위에서 막히면 아래를 보지 않는다.

```
preflight.discriminate exit != 0 ?  → fail / discriminate-fail
preflight.trajectory  exit != 0 ?  → fail / trajectory-fail
new bin missing / empty / find_bin 실패 ? → fail / missing-new-bin
old bin omitted or missing ?
    yes → diff = skipped / missing-old-bin
    no  → run release_diff.py
          report missing ? → fail / diff-report-missing
          report not JSON ? → fail / diff-report-invalid
          classification == probe-failed ? → fail / probe-failed
          surfaceChanged or classification == surface-changed ?
              → reason = surface-changed
          classification == regression ? → reason = regression
          classification == stable ? → reason = stable
          else → fail / unexpected
ledger exists and verify_board ?
    yes → leaderboard.py --bin <new> verify
          도구 예외 ? → fail / leaderboard-error
          exit != 0 ? → reason += leaderboard-broken
decide_verdict(reasons) → pass | fail | review | block
```

이 트리를 코드가 아니라 글로 다시 쓴 이유는, YAML 과 러너와 시험이 같은
질문을 다른 언어로 반복하지 않게 하기 위함이다. 새 예외를 넣을 때는 이
트리의 한 갈래에 이름을 붙이고 `REASONS` 에 한 줄을 더한다.

## 22. 위장 조합 거부 표

`validate_verdict` 가 거부하는 조합이다.  greener 가 되려면 판정·이유·exit
가 한 줄에 있어야 한다.

| verdict | reason | exit | 허용 |
|---|---|---|---|
| pass | stable | 0 | 예 |
| pass | missing-old-bin | 0 | 예 |
| pass | skipped | 0 | 예 |
| pass | regression | 0 | 아니오 |
| pass | surface-changed | 0 | 아니오 |
| pass | discriminate-fail | 0 | 아니오 |
| pass | missing-new-bin | 0 | 아니오 |
| pass | probe-failed | 0 | 아니오 |
| review | surface-changed | 2 | 예 |
| review | regression | 2 | 아니오 |
| review | stable | 2 | 아니오 |
| block | regression | 3 | 예 |
| block | leaderboard-broken | 3 | 예 |
| block | surface-changed | 3 | 아니오 |
| block | discriminate-fail | 3 | 아니오 |
| fail | missing-new-bin | 1 | 예 |
| fail | discriminate-fail | 1 | 예 |
| fail | probe-failed | 1 | 예 |
| fail | diff-report-missing | 1 | 예 |
| fail | diff-report-invalid | 1 | 예 |
| fail | regression | 1 | 아니오 |
| fail | surface-changed | 1 | 아니오 |
| fail | stable | 1 | 아니오 (도구 실패가 아님) |

`fail` + `stable` 은 표에서 거절로 적었다. 구현은 `reason` 이 `stable` 이면
`decide_verdict` 가 pass 를 낸다. 사람이 봉투를 손으로 조립해 `fail` 과
`stable` 을 붙이면 `validate_verdict` 가 `failed 는 fail 과만` 쪽보다
이유 정합에서 먼저 걸린다. 시험이 그 손을 막는다.

## 23. 워크플로 발췌와 러너 대응

워크플로 한 줄이 러너의 어느 갈래인지.

| YAML | 러너 |
|---|---|
| `if [ -f ./rhwp-old-bin ]` 참 | `gate(old, new, ...)` → 차등 실행 |
| `if [ -f ./rhwp-old-bin ]` 거짓 | `gate(None, new, ...)` → missing-old-bin |
| `python3 gym/tools/discriminate.py --bin target/debug/rhwp` 가 1 | 잡 종료. `preflight` 와 같은 이유 |
| `python3 gym/tools/trajectory.py --bin target/debug/rhwp` 가 1 | 잡 종료. trajectory-fail |
| `--github-summary` | `write_github_summary` |
| `-o gate-verdict.json` | `write_verdict_safe` |
| `if: always()` 업로드 | 판정이 fail/block 이어도 봉투를 남김 |
| `contents: read` | 게이트는 쓰기를 요구하지 않음 |

워크플로를 이 가지에서 고치지 않은 이유: 이미 판별·트라젝토리·old 생략이
맞게 배선돼 있다. YAML 을 고치면 열린 계약 시험과 운영 습관을 흔든다.
러너가 예외를 정직하게 접으면 YAML 은 그대로여도 DoD 가 닫힌다.

## 24. 자기 점검 질문

PR 리뷰어가 이 가지만 보고 물을 수 있는 질문.

1. 신 바이너리가 없는데 exit 0 인 시험이 있는가? 있으면 위장이다.
2. 판별 exit 1 을 block(3) 으로 읽는 분기가 있는가? 있으면 위장이다.
3. surface-changed + divergences>0 가 block 인가? 이면 정직 조항을 깼다.
4. `--preflight` 플래그가 argparse 에 생겼는가? 있으면 이슈 계약을 깼다.
5. `discriminate.py` 의 한 줄이라도 이 diff 에 있는가? 있으면 금지 파일을
   열었다.
6. `git diff --shortstat upstream/devel` 의 insertions 가 3000 미만인가?
   이면 DoD 미달이다. 빈 줄로 채우지 말고 표·표본·시험을 더한다.

이 여섯이 모두 아니오/해당없음 이면 이 문서는 구현과 같다.
