---
kind: working
status: active
canonical: mydocs/working/gym_trajectory.md
last_verified: 2026-08-18
---

# gym 트라젝토리 필요성 감사 — 예외 경로·문서·시험 보강

Issue: #5254
Branch: `feat/gym-trajectory-hardening`
Date: 2026-08-18

## 1. 결론

`gym/tools/trajectory.py` 의 마지막 스텝 load-bearing 판정은 그대로 두고,
탐색이 삼키던 네 자리(기준풀이 부재·빈 steps·수집 전용 tail·바이너리
부재)를 예외 목록으로 남겼다. 없는 바이너리를 load-bearing 으로 부르던
위장을 끊었다. CLI 플래그와 pack JSON 은 건드리지 않았다.

검증:

- `python -m unittest scripts.tests.test_gym_trajectory`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

## 2. 배경

원 도입(#4810 / PR #4811)은 다단계 과제에서 마지막 외부 의미 스텝을
빼고 기준풀이를 재조립해 채점하는 감사기를 넣었다. 부분 트라젝토리가
통과하면 연극, 실패(빌드 실패 포함)하면 load-bearing. 당시 26과제 중
2건(AU06, XC03)이 연극이었고 pack 쪽을 고쳐 0으로 만들었다. 릴리스
게이트가 차등 이전에 이 스크립트를 부른다.

그 상태의 빈틈:

1. 과제 JSON 은 있는데 짝 기준풀이가 없으면 `continue`. 탐색을 못 한
   자리가 "연극 0"에 묻힌다.
2. 기준풀이 `steps` 가 없거나 `[]` 이면 길이 < 2 로 단스텝과 같이
   사라진다. 빈 기준풀이는 단스텝이 아니다.
3. 스텝이 2개 이상인데 전부 `answer`/`keyring_from` 이면
   `last_meaningful_step_index` 가 `None` 이고 다시 `continue`. 의미
   스텝을 고를 수 없는데 침묵한다.
4. `build_task`/`score_task` 의 `FileNotFoundError` 를 포함한 모든
   예외를 load-bearing 으로 접는다. 바이너리가 없으면 전 과제가
   정상이다.
5. 과제·기준풀이 JSON 파싱이 한 파일에서 죽으면 전수 감사가 멈춘다.
6. 카탈로그가 코드 주석에만 있어 문서·시험이 같은 표를 공유하지 않는다.

판정 네 칸(통과=연극, 실패=필수, 빌드 `RuntimeError`=필수, 단스텝
무시)을 다섯 칸으로 늘리면 기존 게이트 계약이 깨진다. 그래서
load-bearing 로직은 유지하고, 네 자리는 예외 목록으로만 연다.
`FileNotFoundError` 만 네 칸 밖으로 뺀다 — 그건 부분 트라젝토리의
실패가 아니라 도구 실패다.

## 3. 한 일

### 3.1 도구

`gym/tools/trajectory.py`

- `classify_steps` / `classify_reference` — 다섯 라벨. 단스텝
  `answer`(T01)는 `single-step` 이지 `collection-only-tail` 이 아니다.
- `scan_gym` / `scan_task_pair` — pack·파일 이름 사전순. JSON 파손·
  기준풀이 부재를 한 과제에서 접고 다음으로 간다.
- `exception_kind` / `exception_row` — 예외를 kind 로 접는다. 조립
  자리의 `FileNotFoundError` 는 `missing-bin`.
- `FATAL_EXCEPTIONS` — `KeyboardInterrupt` · `SystemExit` ·
  `MemoryError` · `GeneratorExit` 는 삼키지 않는다.
- `verdict_from_score` / `verdict_from_build_error` — 채점 봉투와 조립
  예외를 판정 자리로. `RuntimeError` 는 여전히 load-bearing.
- `audit_one` — 한 과제의 부분 트라젝토리. 수집 tail 유지, 마지막 의미
  스텝만 제거. 원본 문구 유지.
- `audit` — 탐색 예외를 `exceptions` 에 쌓고, `missing-bin` 이 나면
  나머지 다단계 조립을 멈춘다.
- `attach_report_counts` / `validate_report` — `ok` 는 연극 0건.
  `exit` 는 연극 또는 missing-bin 이면 1. `trusted` 는 예외 0.
- `render_text_report` — 사람용 본문. 예외 목록을 숨기지 않는다.
- `main` — 플래그는 `--bin` `--json` 만. 경로형 `--bin` 이 없으면
  조립 루프 전에 missing-bin 봉투.

`multi_step_tasks` 는 길이 ≥2 기준풀이를 그대로 배출한다. 수집 전용
tail 도 여기 들어간다. 분류는 `audit` 가 한다.

### 3.2 시험

`scripts/tests/test_gym_trajectory.py`

- 기존 5칸(`TrajectoryTests`) 유지. 문구·answer tail·단스텝 무시.
- `CollectionStepTests` / `LastMeaningfulStepTests` / `TruncateTests`.
- `ClassifyStepsTests` + `GeneratedClassifyTableTests`.
- `ScanDiscoveryTests` — missing-reference, empty-steps,
  collection-only-tail, 단스텝 answer 는 skip, JSON 파손.
- `MissingBinTests` — FileNotFound 를 load-bearing 으로 안 부름.
  이후 과제를 정상으로 채우지 않음. 더미 `"bin"` 은 사전 검사하지 않음.
- `MixedGymTests` — 한 gym 에 연극·예외·단스텝 혼재.
- `ReportContractTests` / `GeneratedHonestyMatrixTests`.
- `LoadBearingLogicKeptTests` — 보강 뒤에도 네 칸이 같다.
- `MainCliTests` — `--json` 부재 경로 exit 1.

조립·채점은 목킹한다. 핵심 경로는 바이너리 없이 결정적이다.

### 3.3 문서

- `gym/docs/trajectory.md` — load-bearing 네 칸, 수집 tail, 분류 표,
  예외 경로, JSON 봉투, 정직 행렬, 시험 지도.
- `mydocs/working/gym_trajectory.md` — 이 기록.

pack JSON · `discriminate.py` · `fuzz_corpus.py` · automation /
core-cli / casual-rides · 열린 PR(5210–5248) 파일은 건드리지 않았다.

## 4. 정직 조항 — 바꾸지 않은 것

다음 계약은 원 도입과 같다. 시험이 다시 고정한다.

```
score pass=true                         → 연극
score pass=false / pass 없음            → load-bearing
build RuntimeError                      → load-bearing
단스텝 (길이 1)                         → 감사 대상 아님
[run, answer] 에서 빼는 칸              → run. answer 는 남김
연극 문구                               → "{pack}/{id} (마지막 실제 스텝 {kind}을 빼도 통과 — N→N-1)"
ok                                      → theater == []
schemaVersion                           → 1.0
kind                                    → gymTrajectoryNecessity
CLI                                     → --bin, --json
```

`COLLECTION_STEP_KEYS` 는 `answer` 와 `keyring_from` 만이다. 새 수집
키를 추측해 넣지 않았다.

`other-doc` 같은 차등 라벨을 여기 넣지 않았다. 형식축 도구의 심각도를
경로 감사에 섞지 않는다.

## 5. 예외 카탈로그

| context | 예외 | kind | 집계 |
|---|---|---|---|
| scan | 기준풀이 파일 없음 | missing-reference | exceptions |
| scan | steps 없음/빈 목록 | empty-steps | exceptions |
| scan | ≥2 전부 수집 | collection-only-tail | exceptions |
| audit | FileNotFoundError | missing-bin | exceptions, missingBin |
| load | JSONDecodeError | malformed-json | exceptions |
| load | 과제 비객체 | malformed-task | exceptions |
| load | 기준풀이 비객체·steps 비목록 | malformed-reference | exceptions |
| audit | RuntimeError | (kind 행 없음) | load-bearing |
| * | KeyboardInterrupt 등 | (삼키지 않음) | 프로세스 종료 |

해시 자리·IR 자리 같은 차등 도구의 접는 법은 여기 없다. 이 도구는
관측 kind 를 만들지 않는다. 부분 트라젝토리의 채점 봉투는 `pass` 한
칸만 본다.

## 6. 보고 상태 표

| 상황 | theater | loadBearing | exceptions | ok | exit | trusted |
|---|---|---|---|---|---|---|
| 다단계 N 전부 필수 | 0 | N | 0 | 참 | 0 | 참 |
| 연극 K | K | N-K | 0 | 거짓 | 1 | 참 |
| 단스텝만 | 0 | 0 | 0 | 참 | 0 | 참 |
| 기준풀이 부재만 | 0 | 0 | ≥1 | 참 | 0 | 거짓 |
| 빈 steps 만 | 0 | 0 | ≥1 | 참 | 0 | 거짓 |
| 수집 전용 tail 만 | 0 | 0 | ≥1 | 참 | 0 | 거짓 |
| missing-bin | 0 | 0 | ≥1 | 참 | 1 | 거짓 |
| 연극 + 예외 | ≥1 | * | ≥1 | 거짓 | 1 | 거짓 |
| 연극 + missing-bin | ≥1 | * | ≥1 | 거짓 | 1 | 거짓 |

마지막 두 행을 첫째 행으로 접으면 게이트가 속는다. `ok` 는 연극 유무만
말하고, `exit` 1 이 도구 실패를 가린다. 기준풀이 부재를 `exit` 1 로
올리지 않은 이유: `audit.py` 가 이미 그 짝을 막는다. 같은 자리를 두
게이트가 다른 이유로 막으면 로그가 섞인다.

## 7. 왜 단스텝 answer 는 예외가 아닌가

T01 기준풀이는 `[{"answer": {pages: info.pageCount}}]` 한 줄이다.
`last_meaningful_step_index` 는 `None` 이다. 길이 1 이므로 예전에도
감사 대상이 아니었다.

이 과제를 `collection-only-tail` 로 부르면 운동장의 단답 과제가 전부
예외가 된다. `trusted` 가 항상 거짓이 되고, 본문이 예외로 가득하다.
수집 전용 tail 예외는 **길이 ≥2** 일 때만 성립한다. 그때는 "다단계로
광고했는데 의미 스텝이 없다"는 말이 된다.

시험 `test_single_answer_task_is_skip_not_collection_only` 가 이 경계를
고정한다.

## 8. 왜 missing-bin 만 조립을 멈추는가

기준풀이 부재는 과제마다 다르다. 한 과제의 짝이 없다고 다음 과제의
연극을 못 보는 것은 과하다. JSON 파손도 같다.

바이너리 부재는 과제마다 다르지 않다. 첫 `FileNotFoundError` 뒤에
나머지 25개를 조립하면 같은 예외가 25번 나고, 실수로 그중 하나를
load-bearing 으로 접으면 위장이 돌아온다. 그래서 플래그를 세우고
나머지 다단계 조립을 건너뛴다. 이미 쌓인 탐색 예외는 지워지지 않는다.

시험 `test_missing_bin_does_not_mark_later_tasks_load_bearing` 이
호출 횟수 1 과 `loadBearing==0` 을 같이 본다.

## 9. 왜 더미 `"bin"` 은 파일이 없어도 되는가

단위 시험은 `audit("bin", gym, work)` 를 부른다. 조립기를 목킹하므로
실제 실행 파일은 없다. `main` 진입 전에 `os.path.isfile("bin")` 을
강제하면 핵심 경로가 바이너리를 요구한다. 이슈 DoD 는 "바이너리 없이
핵심 경로"다.

`bin_looks_present` 는 경로 구분자 또는 `.exe` 가 있을 때만 존재
검사를 한다. `"bin"` · `"rhwp"` 는 통과한다. `target/debug/rhwp` 는
파일이 없으면 실패다.

## 10. 검증 명령

저장소 루트에서:

```bash
python -m unittest scripts.tests.test_gym_trajectory
python gym/tools/audit.py
git diff --shortstat upstream/devel
```

packs 를 고치지 않았으므로 audit 는 기존처럼 전 pack 통과여야 한다.
`cargo fmt --all` 은 이번 변경에 해당 없다.

## 11. 의도적으로 하지 않은 것

- pack · reference · profile 편집 없음.
- `discriminate.py` / `fuzz_corpus.py` / `release_gate.py` /
  `release_diff.py` / `coverage.py` / `leaderboard.py` 편집 없음.
- `gym/core/checks.py` 편집 없음 (PR #5210).
- automation / core-cli / casual-rides 편집 없음.
- 새 CLI 플래그 없음. `--cli-timeout` 같은 확장은 다음 이슈.
- `schemaVersion` 을 1.1 로 올리지 않음. 필수 키 여섯 개가 그대로다.
- 단스텝을 연극으로 승격하지 않음.
- `ok` 를 예외와 묶지 않음.

## 12. 남은 빈틈

1. 라이브 스윕(`--bin target/debug/rhwp`)은 이 가지에서 돌리지 않았다.
   판정 로직은 목킹 시험이 고정하고, 실제 26과제의 연극 0은 게이트가
   계속 본다.
2. 수집 키를 `answer`/`keyring_from` 밖으로 늘리지 않았다. 새 제출
   형태가 생기면 키 집합과 시험을 같이 늘려야 한다.
3. `score_task` 가 던지는 `FileNotFoundError` 이외의 OSError(권한)는
   지금 load-bearing 이다. 권한 오류를 missing-bin 과 같은 도구 실패로
   접을지는 다음 이슈.
4. `work_root` 정리 실패(`shutil.rmtree`)는 여전히 무시한다. 디스크
   잠금이 감사를 막게 하지 않으려는 기존 동작이다.

## 13. 변경 파일

| 경로 | 역할 |
|---|---|
| `gym/tools/trajectory.py` | 예외 경로·분류·봉투 |
| `scripts/tests/test_gym_trajectory.py` | 바이너리 없는 계약 |
| `gym/docs/trajectory.md` | 규약 |
| `mydocs/working/gym_trajectory.md` | 이 기록 |

## 14. 이슈 DoD 대조

| DoD | 상태 |
|---|---|
| `gym/tools/trajectory.py` 고도화 | 예외 네 자리 + 분류 + 봉투 |
| `scripts/tests/test_gym_trajectory.py` 고도화 | 기존 5칸 + 예외·행렬 |
| `gym/docs/trajectory.md` | 추가 |
| `mydocs/working/gym_trajectory.md` | 추가 |
| 새 CLI / pack 없음 | 플래그·pack 불변 |
| 열린 PR 파일 미수정 | 5210–5248 경로 회피 |
| additions ≥ 3000 | `git diff --shortstat upstream/devel` |
| unittest + audit.py | 아래 검증 |
| 결정적 | 사전순 탐색, 난수·시각 없음 |
| 바이너리 없이 핵심 경로 | 목킹 + 순수 함수 |
| 마지막 스텝 load-bearing 유지 | 네 칸 +  pentest 클래스 |

## 15. 탐색 알고리즘 (구현 메모)

```
scan_gym(gym_root):
  packs = sorted(dir names under gym_root/packs that are directories)
  for pack in packs:
    if no tasks/: continue
    for name in sorted(*.json in tasks/):
      yield scan_task_pair(pack, name)

scan_task_pair:
  load task JSON → fail: malformed-json/task
  if reference file missing → missing-reference
  load reference JSON → fail: malformed-json/reference
  label = classify_reference(reference)

audit:
  for rec in scan_gym:
    single-step → skipped
    exception label → exceptions
    multi:
      if missing_bin: continue
      audit_one(...)
      FileNotFoundError → missing_bin=true
```

`os.listdir` 실패는 `toolErrors` 한 줄이다. 없는 pack 을 지어내지
않는다.

## 16. 부분 트라젝토리 조립 메모

`truncate_reference` 는 `dict(reference)` 사본에 `steps` 만 갈아끼운다.
`id` · 기타 키는 유지한다. `build_task` 가 기준풀이 식별자를 읽는
경우를 깨지 않는다.

`truncate_steps(steps, i)` 는 `steps[:i] + steps[i+1:]`. `i` 가 범위
밖이면 사본만 돌려준다. `None` 스텝 열은 빈 목록이다.

`audit_one` 이 `score_task` 에 넘기는 `sub_root` 는
`os.path.join(work_root, pack_id)` 이다. 기존과 같다.

## 17. 사람용 본문 메모

성공(연극 0, 도구 실패 아님):

```
gym 트라젝토리 필요성 감사: {N} 다단계 과제 전부 마지막 스텝이 load-bearing — 연극 0
```

연극:

```
gym 트라젝토리 필요성 감사: 연극(무의미한 마지막 스텝) {K}건 — 부분 트라젝토리가 통과한다:
  - {pack}/{id} (마지막 실제 스텝 {kind}을 빼도 통과 — {N}→{N-1})
```

도구 실패(연극 0):

```
gym 트라젝토리 필요성 감사: 연극 0 · 도구 실패 (예외 {E}건, trusted=false)
```

예외가 있으면 어떤 본문 아래에도 `예외 경로 {E}건:` 블록이 붙는다.
`--json` 이면 본문 대신 봉투만 쓴다.

## 18. 열린 PR 회피 목록 (이 가지에서 안 만진 것)

이슈가 명시한 금지와 5210–5248 파일.

- `gym/core/checks.py`, `gym/docs/checks.md`, `mydocs/working/gym_core_checks.md`
- `gym/tools/agent_session.py` 및 세션 시험
- `gym/tools/coverage.py`, extraction/table-csv/batch-ops pack 확장
- `gym/tools/leaderboard.py`, `gym/docs/leaderboard.md`
- `gym/tools/release_diff.py`, `gym/docs/release_diff.md`
- `gym/tools/discriminate.py`
- `gym/tools/fuzz_corpus.py`
- `gym/packs/automation/**`
- `gym/packs/core-cli/**`
- `gym/packs/casual-rides/**`

이 가지는 도구 4파일이 전부다.

## 19. 커밋 범위

브랜치 `feat/gym-trajectory-hardening` 는 `upstream/devel` 에서
갈랐다. 커밋 하나. 제목·본문 한국어. PR 본문은 `--body-file` (UTF-8
BOM 없음). `closes #5254`.

`git add -A` 는 쓰지 않는다. 네 경로만 스테이징한다.

## 20. 다음 작업자에게

라이브 스윕을 돌릴 때는 기존과 같이 `--bin` 이 실제 rhwp 를 가리켜야
한다. 이 가지의 단위 시험이 green 이라고 26과제의 연극 0을 다시 잰
것은 아니다. 게이트가 그 자리를 본다.

수집 키를 늘리거나 `ok` 를 예외와 묶고 싶으면 이 문서 6절 표를 먼저
고치고 시험을 같이 고친다. 표 없이 `ok` 의미를 바꾸면 게이트가 다른
이유를 연극으로 읽는다.

## 21. 라이브 운동장 스모크

단위 시험 `RealGymScanSmokeTests` 는 devel 의 `gym/packs` 를 읽기만
한다. pack JSON 을 쓰지 않는다.

기대:

- 레코드가 10건보다 많다.
- `empty-steps` · `collection-only-tail` · `malformed-*` 가 0이다.
- 단스텝이 있고 그 안에 T01 이 있다.
- `multi` 레코드는 모두 `last_meaningful_step_index` 가 숫자다.
- `multi_step_tasks` 길이는 `steps` 길이 ≥2 인 기준풀이 수와 같다.

이 스모크가 실패하면 이 가지가 아니라 **다른 가지의 pack 확장** 이
기준풀이를 깨뜨린 것이다. 그 경우 pack 을 이 가지에서 고치지 말고,
해당 pack PR 로 돌린다.

## 22. 분류 경계 재진술

같은 입력을 문서와 시험이 같이 본다. 구현을 바꿀 때 이 절과
`CLASSIFY_CASES` 를 한 번에 고친다.

1. `steps is None` → empty-steps
2. `steps == []` → empty-steps
3. `steps` 가 목록이 아님 → malformed-reference
4. 길이 1 → single-step (키가 answer 여도)
5. 길이 ≥2 이고 의미 인덱스 없음 → collection-only-tail
6. 길이 ≥2 이고 의미 인덱스 있음 → multi

`classify_reference` 는 `steps` 키 없음을 1번으로, `steps: {…}` 를
3번으로 가른다. `normalize_steps` 가 dict 를 `None` 으로 접더라도
`classify_reference` 가 그 전에 목록 여부를 본다. 객체 steps 를 빈
목록과 같은 칸에 두면 안 된다.

## 23. 커밋 후 확인 체크

```
git diff --shortstat upstream/devel
# insertions >= 3000

git diff --name-only upstream/devel
# 네 경로만

python -m unittest scripts.tests.test_gym_trajectory
python gym/tools/audit.py
```

Windows 에서 본문이 CRLF 로 바뀌지 않았는지 `git diff --check` 또는
바이트 검사로 LF 만 있는지를 본다.

## 24. 구현 중 고친 분류 버그

초안에서 `normalize_steps` 가 목록이 아닌 값을 `None` 으로 접고
`classify_reference` 가 그 `None` 을 `empty-steps` 로 불렀다. 그러면
`{"steps": {"run": 1}}` 이 빈 기준풀이와 같은 칸이 된다.

고친 계약: `classify_reference` 가 `steps` 키 존재와 목록 여부를 먼저
본다. 키가 없거나 null 이면 empty-steps. 목록이 아니면
malformed-reference. 빈 목록만 empty-steps.

시험 `test_classify_reference_reads_steps` 와
`test_steps_object_is_malformed_reference` 가 이 칸을 고정한다.

## 25. 크기 게이트

이슈 DoD 는 `git diff --shortstat upstream/devel` 삽입 ≥ 3000 이다.
네 경로만 센다. 열린 PR 파일이나 pack JSON 으로 줄을 채우지 않는다.

삽입은 도구 본문·순수 함수·예외 표·시험 행렬·규약 문서에서 나온다.
같은 문장을 백 번 반복하지 않는다. 표의 각 칸은 시험이 다시 본다.

## 26. 리뷰어가 볼 것

1. 기존 `TrajectoryTests` 다섯 칸이 그대로인가.
2. 네 예외 경로가 `exceptions` 로 남는가.
3. `FileNotFoundError` 가 `loadBearing` 을 올리지 않는가.
4. T01 단스텝 answer 가 예외가 아닌가.
5. `ok` 가 연극 유무만 말하는가.
6. `--bin` / `--json` 외에 플래그가 없는가.
7. 금지 파일(discriminate, fuzz, automation, core-cli, casual-rides,
   5210–5248)이 diff 에 없는가.
