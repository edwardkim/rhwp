---
kind: working
status: active
canonical: mydocs/working/gym_release_gate.md
last_verified: 2026-09-02
---

# gym 릴리스 게이트 — 예외 경로와 판정 정직성 보강

> **2026-09-02 운영 경계 정정:** 이 문서는 #5259 당시 구현 계보를 보존하는 작업
> 기록이다. 여기서 `release gate`·`block`이라 부른 결과는 이제 Gym 수동 차등 조사에만
> 적용한다. 같은 현재 rhwp가 만든 라이브 기대값은 독립 제품 정답지가 아니므로 일반
> PR/devel/main CI, 태그, Release Binary, npm·extension 게시를 허용하거나 차단하지 않는다.
> 현행 운영 정본은 [`gym/docs/release_gate.md`](../../../gym/docs/release_gate.md)의 0절과
> [`mydocs/manual/publish_guide.md`](../../manual/publish_guide.md)의 Gym 운영 경계다.

Issue: #5259
Branch: `feat/gym-release-gate-hardening`
Date: 2026-08-18
Worktree: `C:\Users\swsz9\rhwp-gym-release-gate` (isolation, `rhwp-desk*` 아님)

## 1. 결론

`gym/tools/release_gate.py` 의 기존 사원(pass / review / block)은 그대로 두고,
도구·전제 실패를 `fail`(exit 1) 로 분리했다. 구 바이너리 부재는 계속 skipped /
pass 다. 신 바이너리 부재, 판별 감사 실패, probe-failed, 깨진 차등 보고는
stable 로 위장하지 않는다. surface-changed 는 review, regression 은 block.
둘이 섞이면 표면이 이긴다.

새 CLI 플래그는 없다. 새 pack 은 없다. `discriminate.py` · `trajectory.py` ·
`release_diff.py` · `fuzz_corpus.py` · automation / core-cli / casual-rides 는
이 가지에서 열지 않았다. 열린 PR 5210–5248 · 5266 의 파일도 열지 않았다.

검증:

- `python -m unittest scripts.tests.test_gym_release_gate_workflow`
- `python -m unittest scripts.tests.test_gym_release_gate_exceptions`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

## 2. 배경

원 구현(#4662)은 차등 분류 세 값을 종료 코드 0/2/3 으로 묶고, 구 바이너리가
없으면 차등을 생략했다. 워크플로가 판별·트라젝토리를 게이트보다 먼저 돌린다.
대비 `upstream/devel` 의 러너는 약 150줄, 워크플로 시험은 약 147줄이었다.

그 상태의 빈틈:

1. **신 바이너리 부재를 검사하지 않는다.** `find_bin` 이 `rhwp` 라는 이름만
   돌려줘도 차등·원장을 그대로 부른다. 구 부재와 같은 skipped/pass 로
   떨어질 수 있다. 현재 릴리스가 없는 상태를 안정으로 위장한다.
2. **`probe-failed` 를 pass 로 읽는다.** 차등이 표면을 못 재면 분류가
   `probe-failed` 다. 러너는 `regression` / `surface-changed` 가 아니면
   전부 pass 였다. 오라클이 말을 안 했는데 안정이다.
3. **차등 보고가 없으면 게이트가 죽는다.** `json.load` 가 예외를 올린다.
   깨진 JSON, 안 쓰인 `-o`, 권한 오류가 파이프라인을 트레이스백으로 끝낸다.
   판정 봉투가 없다.
4. **판별 실패를 러너가 모른다.** 워크플로가 먼저 막지만, 프로그래매틱
   호출·재현 시험에서 그 이유를 차등 분류와 구분할 함수가 없다. 운영자가
   exit 1 과 exit 3 을 같은 "실패" 로 접기 쉽다.
5. **오라클이 `regression` + `surfaceChanged=true` 를 내도 게이트는 block
   이다.** 표면이 앞선다는 정직 조항을 다시 적용하지 않는다.
6. **카탈로그가 코드에만 있다.** 문서·시험이 같은 표를 공유하지 않는다.

분류 함수를 네 값으로 늘리면 기존 게이트 계약
(`stable→0`, `surface-changed→2`, `regression→3`)이 깨진다. 그래서 차등
삼원은 유지하고, 도구/전제 실패만 사원의 네 번째 값 `fail` 로 받는다.

## 3. 한 일

### 3.1 도구

`gym/tools/release_gate.py`

- `VERDICTS` / `EXIT_BY_VERDICT` / `REASONS` / `VERDICT_BY_REASON` /
  `REASON_TEXT` — 시험·문서가 같은 표를 본다.
- `FATAL_EXCEPTIONS` — KeyboardInterrupt · SystemExit · MemoryError ·
  GeneratorExit 는 삼키지 않는다.
- `exception_kind` / `exception_record` — context 가 `diff-report` 이면
  FileNotFound 는 `diff-report-missing`.
- `reason_for_audit` / `fold_preflight` — 판별·트라젝토리 종료 코드를
  이유 코드로 접는다. 0 이면 이유 없음.
- `map_diff_classification` / `surface_wins_over_regression` — 삼원 +
  skipped/probe-failed. 표면 플래그가 있으면 regression 을 review 로 되돌린다.
- `decide_verdict` — 우선순위 0(전제) > 1(도구 실패) > 2(block) > 3(review)
  > 4(pass).
- `find_bin_safe` / `path_exists_safe` / `resolve_bin_record` — 바이너리
  존재를 예외 없이 기록한다.
- `run_tool_safe` / `load_json_safe` / `remove_safe` — 서브프로세스와
  보고 읽기가 게이트를 죽이지 않는다.
- `extract_diff_fields` — 분류 키가 없으면 invalid. dict 가 아니면 invalid.
- `run_release_diff` / `run_leaderboard` — 오류를 이유 코드로 접는다.
  원장 `--bin` 은 호출자가 준 `new_bin` 그대로다.
- `gate(..., preflight=None)` — 기존 위치 인자 다섯 개는 그대로. 여섯 번째는
  선택. CLI 플래그가 아니다.
- 신 바이너리가 없으면 차등·원장을 부르지 않고 `fail`.
- 전제가 실패하면 차등 숫자를 믿지 않고 `fail`. 원장만 선택적으로 본다.
- `validate_verdict` — ok/review/blocked/failed/exit/reason 정직 계약.
- `render_summary_lines` / `write_github_summary` — 이유별 주석.
- `write_verdict_safe` — UTF-8 · BOM 없음 · LF. 쓰기 실패는 판정을 유지.
- `parse_args` / `main` — 기존 플래그만. `main(argv)` 를 시험이 직접 부른다.

### 3.2 시험

`scripts/tests/test_gym_release_gate_workflow.py`

- 기존 워크플로 계약 9 + 러너 판정 6 을 유지.
- `test_missing_old_binary_skips_diff_not_fail` 는 신 경로를 있다고 보고
  구 부재만 분리한다. 예전처럼 `exists=False` 전역 목은 신 부재와 겹친다.
- 판별이 `--old` 를 받지 않는지, `continue-on-error` 가 없는지, 업로드가
  `always()` 인지, write 권한이 없는지를 고정.
- 문서 두 파일이 이유 코드를 포함하는지 고정.

`scripts/tests/test_gym_release_gate_exceptions.py` (신규)

- 카탈로그·CLI 비확장·치명 예외 표지.
- `reason_for_audit` — discriminate 실패는 regression 이 아님.
- `fold_preflight` — None/한 줄/목록/쓰레기 입력.
- `map_diff_classification` / `surface_wins_over_regression`.
- `decide_verdict` 행렬 (구/신 부재, 판별, 표면, 회귀, 원장).
- `validate_verdict` 위장 조합.
- `gate` 라이브 목: 신 부재, 구 부재, 판별 실패가 회귀를 이김, 표면+분기,
  오라클 실수(regression+surface), probe-failed, 깨진 JSON, 도구 OSError,
  KeyboardInterrupt 비삼킴, 원장 예외 vs 원장 파손.
- step summary 문구, 쓰기 형식, `main` exit 0/1.
- 표면 × 분기 × 전제 × 원장 생성 표.

### 3.3 문서

- `gym/docs/release_gate.md` — 사원, 네 예외 경로, 우선순위, 이유 카탈로그,
  JSON 봉투, 워크플로, 오검출 관문, 표본 8종.
- `mydocs/working/gym_release_gate.md` — 이 기록.

pack JSON 은 건드리지 않았다.

## 4. 정직 조항 — 바꾸지 않은 것

다음 계약은 #4662 와 같다. 시험이 다시 고정한다.

```
stable           → pass   / 0
surface-changed  → review / 2
regression       → block  / 3
missing-old      → skip diff, pass (원장 무결 시)
broken board     → block  / 3
leaderboard --bin 은 게이트의 new_bin
CLI: --old --new --agent --pack --no-leaderboard --github-summary -o
```

더한 것(기존과 모순되지 않음):

```
missing-new      → fail / 1
discriminate-fail→ fail / 1
probe-failed     → fail / 1
bad diff report  → fail / 1
surface flag + regression 분류 → review (표면이 이김)
```

`fail` 을 차등 `CLASSIFICATIONS` 에 넣지 않았다. 넣으면 오라클 계약을
게이트가 침습한다. 오라클은 삼원, 게이트는 사원이다.

## 5. 예외 카탈로그

| context | 예외 | kind |
|---|---|---|
| bin | FileNotFoundError | missing-bin |
| diff-report | FileNotFoundError | diff-report-missing |
| * | PermissionError | permission |
| * | TimeoutExpired / TimeoutError | timeout |
| * | UnicodeError | decode-error |
| * | JSONDecodeError | invalid-json |
| * | KeyError / IndexError / AttributeError | key-error / index-error / type-error |
| * | TypeError | type-error |
| * | ValueError | value-error |
| * | OSError | os-error |
| * | RuntimeError | runtime-error |
| * | 그 외 | unexpected |

`reason_for_audit`:

| tool | exit | reason |
|---|---|---|
| discriminate.py | 0 | (없음) |
| discriminate.py | ≠0 | discriminate-fail |
| trajectory.py | 0 | (없음) |
| trajectory.py | ≠0 | trajectory-fail |
| 그 외 | ≠0 | audit-fail |
| 아무거나 | 비정수 | audit-fail |

## 6. 우선순위 표 (운영)

| 들어온 이유들 | 이긴 이유 | 판정 |
|---|---|---|
| stable | stable | pass |
| missing-old-bin | missing-old-bin | pass |
| missing-new-bin | missing-new-bin | fail |
| discriminate-fail | discriminate-fail | fail |
| discriminate-fail + regression | discriminate-fail | fail |
| discriminate-fail + surface-changed | discriminate-fail | fail |
| probe-failed | probe-failed | fail |
| surface-changed | surface-changed | review |
| surface-changed + 원장 파손 | leaderboard-broken | block |
| regression | regression | block |
| regression + 원장 파손 | regression (또는 broken, 둘 다 block) | block |
| missing-old + 원장 파손 | leaderboard-broken | block |
| 원장 도구 예외 | leaderboard-error | fail |

이 표가 이슈 본문의 "Exception paths: missing old/new bin, discriminate fail,
surface-changed vs regression" 을 닫는다.

## 7. 열지 않은 파일

사용자 지시와 열린 PR 충돌 방지.

- `gym/tools/trajectory.py`
- `gym/tools/discriminate.py`
- `gym/tools/fuzz_corpus.py`
- `gym/tools/release_diff.py` (PR #5248)
- `gym/packs/automation/**` (PR #5266)
- `gym/packs/core-cli/**`
- `gym/packs/casual-rides/**`
- `gym/core/checks.py` (PR #5210)
- `gym/tools/coverage.py` / `gym/docs/coverage.md` (PR #5212)
- `gym/docs/release_diff.md` (PR #5248)
- `gym/docs/checks.md` · `agent_session.md` · `robustness.md` ·
  `pack_health.md` · `differential.md` · `competitive_bench.md` ·
  `from_e2e.md` · `leaderboard.md` (각 열린 PR)
- `.github/workflows/gym-release-gate.yml` — 계약은 시험이 읽고, 파일은
  그대로 둔다. 새 플래그·새 스텝을 넣지 않는다.

연 파일:

- `gym/tools/release_gate.py`
- `scripts/tests/test_gym_release_gate_workflow.py`
- `scripts/tests/test_gym_release_gate_exceptions.py` (신규)
- `gym/docs/release_gate.md` (신규)
- `mydocs/working/gym_release_gate.md` (신규)

## 8. 기존 시험을 고친 이유

`test_missing_old_binary_skips_diff_not_fail` 는 `os.path.exists` 를 전부
거짓으로 목했다. 그 목은 신 바이너리도 없다고 말한다. 신 부재를 fail 로
닫으면 이 시험이 구 부재를 더 이상 대변하지 못한다.

고친 목:

- `ledger.ndjson` 만 없다 (원장을 안 돌린다)
- 그 외 경로는 있다 (신 바이너리 존재)
- `old_bin=None` 이라 구는 omitted → skipped / pass

신 부재는 `test_missing_new_fails` 가 따로 고정한다. 한 시험이 두 예외를
동시에 목하면 어느 쪽이 이겼는지 알 수 없다.

## 9. CLI 를 늘리지 않은 이유

이슈 본문: "새 CLI/pack 없음."

판별 실패를 CLI 로 받으려면 `--preflight` 또는 `--discriminate-report` 가
필요하다. 워크플로는 이미 게이트 전에 판별을 돌린다. 플래그를 달면

- 같은 감사를 두 번 돌리거나
- 워크플로 YAML 을 고쳐 열린 계약 시험을 흔들거나
- "새 CLI 없음" 을 깨게 된다.

그래서 `gate(..., preflight=)` 는 파이썬 입구만 연다. `parse_args` 는
기존 플래그만 안다. 시험이 `--discriminate-fail` / `--preflight` 를 주면
`SystemExit` 이어야 한다.

## 10. 스키마 1.0 을 유지한 이유

칸을 더했지만 `schemaVersion` 은 `1.0` 이다. 기존 소비자가 `kind` 와
`verdict` / `exit` / `diff.classification` 만 읽으면 그대로 동작한다.
`reason` · `old` · `new` · `preflight` 는 부가다. 메이저 범프는 기존 칸의
의미를 바꿀 때 한다. 이번에는 의미를 좁혀 위장을 제거했을 뿐이다.

`fail` 판정은 예전 러너에 없었다. 예전에는 그 자리들이 pass 이거나
트레이스백이었다. 소비자가 0/2/3 만 알고 있으면 1 은 "도구 실패" 로
읽는 것이 Git 관례와 같다.

## 11. 검증 실측

작업 트리에서:

```
python -m unittest scripts.tests.test_gym_release_gate_workflow \
                   scripts.tests.test_gym_release_gate_exceptions -q
python gym/tools/audit.py
```

기대한 것:

- 워크플로 + 예외 경로 시험 전부 통과
- audit.py 전 pack 통과 (이 가지가 pack 을 안 건드렸으므로 devel 과 동일)

`cargo fmt --all` · `cargo test` · `cargo clippy` 는 호출하지 않는다.
Rust 파일이 없다.

## 12. 크기 게이트

이슈 DoD: `additions >= 3000`. 채우는 내용은 계약 표·예외 카탈로그·
봉투 표본·생성 시험이다. 빈 줄이나 반복 문장으로 숫자를 만들지 않았다.

삽입이 모이면 `git diff --shortstat upstream/devel` 의 insertions 가
3000 이상이어야 PR 을 연다.

## 13. PR

- base: `devel`
- head: `feat/gym-release-gate-hardening`
- 제목·본문: 한국어
- `closes #5259`
- `--body-file` (UTF-8 BOM 없음)
- `git add -A` 사용 안 함. 연 파일만 스테이징.

## 14. 남은 빈틈 (이 가지의 밖)

- 워크플로가 판별 실패 이유를 게이트 봉투에 남기지 않는다. 잡이 먼저
  죽기 때문이다. 아티팩트에 판별 JSON 을 올리려면 YAML 을 고쳐야 하고,
  그건 별 이슈다.
- 라이브 바이너리 두 개로 게이트를 돌리는 스윕은 단위 시험이 대체한다.
  CI 의 기존 워크플로가 그 경로다.
- `trajectory-fail` 은 이유 코드와 우선순위만 있다. 이슈가 지목한 자리는
  판별이다. 트라젝토리 도구 자체는 열지 않았다.
- 차등 오라클의 `probe-failed` 보고 형식은 #5248 가지가 고정한다. 게이트는
  `classification` 문자열만 읽는다.

## 15. 한 줄

게이트는 차등을 재판정하지 않는다. 차등이 말한 것을 파이프라인 종료 코드로
옮기고, 말할 수 없는 자리(부재·전제 실패·보고 손상)를 안정으로 부르지
않는다.

## 16. 함수 목록 (러너)

순수 또는 예외를 접는 함수만 적는다. `gate` / `main` 은 조합이다.

| 함수 | 순수 | 역할 |
|---|---|---|
| `is_fatal_exception` | 예 | 삼키면 안 되는 예외 |
| `truncate_head` | 예 | 오류 머리 |
| `exception_kind` | 예 | 예외 → kind |
| `exception_record` | 예 | 오류 한 줄 |
| `normalize_tool_name` | 예 | 스크립트 경로 → 짧은 이름 |
| `reason_for_audit` | 예 | 전제 종료 코드 → 이유 |
| `fold_preflight` | 예 | 전제 입력 정규화 |
| `map_diff_classification` | 예 | 차등 분류 → 이유 |
| `surface_wins_over_regression` | 예 | 표면 플래그가 회귀를 이김 |
| `decide_verdict` | 예 | 이유 목록 → 사원 |
| `extract_diff_fields` | 예 | 보고에서 칸만 추출 |
| `validate_verdict` | 예 | 정직 계약 |
| `render_summary_lines` | 예 | step summary 문자열 |
| `find_bin_safe` | 아니오 | find_bin 예외 접기 |
| `path_exists_safe` | 아니오 | exists 예외 접기 |
| `resolve_bin_record` | 아니오 | 바이너리 존재 기록 |
| `run_tool` / `run_tool_safe` | 아니오 | 서브프로세스 |
| `load_json_safe` | 아니오 | 보고 읽기 |
| `run_release_diff` | 아니오 | 차등 + 보고 |
| `run_leaderboard` | 아니오 | 원장 verify |
| `write_verdict_safe` | 아니오 | 판정 쓰기 |
| `gate` | 아니오 | 조합 |
| `main` | 아니오 | CLI |

순수 함수는 바이너리 없이 시험한다. 조합 함수는 `run_tool` 과 `exists` 를
목으로 갈아끼운다.

## 17. 기존 시험이 깨지지 않는 이유

| 기존 시험 | 의존 | 유지 방법 |
|---|---|---|
| `test_stable_passes` | `_gate_with_diff("stable")` | exists=True, 보고 분류 그대로 pass |
| `test_regression_blocks` | 같은 헬퍼 | reason=regression, block/3 |
| `test_surface_changed_is_review_not_block` | 같은 헬퍼 | review/2. 분기 70 이어도 block 아님 |
| `test_broken_leaderboard_blocks` | exists=True, run_tool=(3,…) | 원장 파손 → block/3 |
| `test_leaderboard_uses_the_selected_new_binary` | `--bin` 인자 | 해석 경로가 아니라 호출자 `new_bin` |
| `test_missing_old_binary_skips_diff_not_fail` | 예전 exists=False | 신 존재 목으로 분리 |
| 워크플로 9종 | YAML 문자열 | YAML 을 이 가지에서 안 고침 |

새 시험은 기존 이름을 바꾸지 않고 옆에서 더한다. `test_*workflow*.py`
패턴은 ci.yml 배선(#4080)이 강제하므로 파일명을 유지한다.
