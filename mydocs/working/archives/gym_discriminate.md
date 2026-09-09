---
kind: working
status: active
canonical: mydocs/working/gym_discriminate.md
last_verified: 2026-08-18
---

# gym 판별력 감사 — 음성 대조 세 종류 고정

Issue: #5255
Branch: `feat/gym-discriminate-hardening`
Date: 2026-08-18

## 1. 결론

`gym/tools/discriminate.py` 의 음성 대조를 세 종류로 고정하고, 경로
안전·예외 접기·보고 정직 계약을 순수 함수로 분리했다. 새 CLI 플래그와
새 pack 은 없다. `trajectory.py` · `fuzz_corpus.py` · `automation` /
`core-cli` / `casual-rides` 는 열지 않았다.

검증:

- `python -m unittest scripts.tests.test_gym_discriminate`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

## 2. 배경

devel 의 감사기는 이미 세 가지를 돌렸다.

1. answer 키에 `__NEGATIVE_CONTROL_definitely_wrong__`
2. artifact 산출에 입력 복사
3. artifact 산출에 `GARBAGE_BYTES`

시험은 다섯 개였다. 복사와 garbage 가 실제로 다른 바이트인지, 입력이
없을 때 전수 감사가 죽는지, `../` 산출 경로가 제출 폴더 밖으로 쓰는지,
깨진 과제 JSON 이 도구를 죽이는지, 채점 예외를 통과로 접는지 — 이
자리들은 시험이 없었다.

이슈 #5255 의 DoD 는 `additions >= 3000`, 음성 대조 종류를 시험으로
고정, `audit.py` 통과다. 새 CLI/pack 금지, 열린 PR 파일 미수정.

## 3. 한 일

### 3.1 도구

`gym/tools/discriminate.py`

- `CONTROL_KINDS` / `CONTROL_CATALOG` — `wrong-answer` · `input-copy` ·
  `garbage` 만. 문서·시험이 같은 표를 본다.
- `controls_for` — artifact 는 copy+garbage, 그 외는 sentinel.
- `answer_keys` — 비-dict 검사·빈 키를 무시한다.
- `normalize_rel` / `unsafe_rel_reason` — 절대·드라이브·UNC·홈·부모
  경로는 쓰지 않는다.
- `build_negative` — 미지 모드는 기존처럼 `ValueError`. 입력 없음은
  건너뛴다. garbage 는 원본이 필요 없다. 반환 dict 에 쓴 파일과
  오류를 남긴다. 기존 호출 서명(`task, dir, artifact_mode=`)은 유지.
- `score_task_safe` — 채점 예외는 `pass=False` 로 접는다.
  `KeyboardInterrupt` · `SystemExit` · `MemoryError` · `GeneratorExit`
  는 다시 올린다.
- `aggregate_rows` / `validate_report` — `ok` 는 `falsePass` 가 비었을
  때만 참. `discriminating` 산술을 검사한다.
- `discriminate` — 깨진 JSON 은 `loadErrors` 로 건너뛴다. 기존 키
  (`kind` · `schemaVersion` · `ok` · `taskCount` · `controlCount` ·
  `discriminating` · `falsePass` · `falsePassControls`) 의미는 그대로다.
- CLI 는 `--bin` · `--json` 만. 종료 코드 0/1.

### 3.2 시험

`scripts/tests/test_gym_discriminate.py`

기존 다섯 시험을 유지한다.

- 무편집 복사 + sentinel
- 음성이 통과하면 false-pass
- 음성이 거부되면 ok
- garbage 만 통과하면 대조 라벨이 따로 난다
- garbage 바이트가 입력과 다르다

추가한 칸:

- 카탈로그가 세 종류뿐인지
- sentinel 이 흔한 진값과 다른지
- garbage 가 1KiB 를 넘고 UTF-8/JSON/XML 이 아닌지
- 경로 탈출이 거절되는지
- 입력 부재가 예외가 아닌지
- answer 과제는 대조 1건, artifact 는 2건
- 채점 예외는 false-pass 가 아닌지
- 보고 검증이 모순을 잡는지
- CLI 에 새 플래그가 없는지

### 3.3 문서

- `gym/docs/discriminate.md` — 규약 정본
- `mydocs/working/gym_discriminate.md` — 이 기록

## 4. 의도적으로 하지 않은 일

- pair 산출에 copy/garbage 를 얹지 않았다. 이슈가 고정한 종류는
  answer sentinel · artifact copy · garbage artifact 다.
- `--pack` / `--limit` 을 달지 않았다. 전수 감사가 점이다.
- `schemaVersion` 을 1.1 로 올리지 않았다. 기존 키 의미가 그대로다.
- `ok` 를 `toolFailed` 와 AND 하지 않았다. 도구가 죽은 것과 약한
  오라클은 다른 신호다. 전자는 `toolFailed` 키로 남긴다.
- `gym/README.md` 를 고치지 않았다. 이 가지의 허용 파일 밖이다.
- `audit.py` 는 실행만 했고 수정하지 않았다.

## 5. 위험과 남은 구멍

1. **라이브 스윕은 바이너리가 필요하다.** 단위 시험은 러너를 목킹한다.
   게이트 워크플로가 실제 `discriminate.py --bin` 을 돈다.
2. **pair 과제는 sentinel 만 돈다.** 답 키가 없으면 빈 폴더를 채점한다.
   파일 검사가 있으면 실패(판별력 있음)다. 파일 검사조차 없으면
   빈 제출이 통과할 수 있다. 그 확장은 후속 이슈다.
3. **입력 없는 copy 는 파일을 만들지 않는다.** 일부 연산자는 "파일
   없음"을 실패로 접으므로 판별력 있음으로 집계된다. 이것이
   false-pass 를 숨기지는 않는다.
4. **열린 pack 확장 PR 과 겹치지 않으려면 pack 파일을 만지면 안
   된다.** 이 가지는 도구·시험·문서만 만진다.

## 6. 재현

격리 worktree: `C:\Users\swsz9\rhwp-gym-discriminate`
기준: `upstream/devel`
브랜치: `feat/gym-discriminate-hardening`

```bash
python -m unittest scripts.tests.test_gym_discriminate
python gym/tools/audit.py
git diff --shortstat upstream/devel
```

## 7. 시험 목록 (클래스)

| 클래스 | 고정하는 것 |
|---|---|
| `DiscriminateTests` | devel 때부터 있던 다섯 계약 |
| `CatalogContractTests` | 대조 id 세 개, 보고 키, 종료 코드 |
| `SentinelContractTests` | sentinel 안정성, 흔한 진값과 불일치 |
| `GarbageContractTests` | 1KiB, 널, JSON/XML/PDF 가 아님 |
| `PathSafetyTests` | `..` / 절대 / 드라이브 / UNC / 홈 거절 |
| `SubmitShapeTests` | answer·artifact·pair → 대조 튜플 |
| `BuildNegativeAnswerTests` | 답만 쓰고 산출은 안 씀 |
| `BuildNegativeCopyTests` | 입력 바이트 복사, 원본 무훼손, 입력 부재 |
| `BuildNegativeGarbageTests` | garbage 바이트, 원본 불필요 |
| `ScoreClassifyTests` | pass 해석, 치명 예외 재발생 |
| `LabelAndAggregateTests` | 과제 라벨 중복 제거, 대조 라벨 유지 |
| `ValidateReportTests` | ok ↔ falsePass 정직 |
| `DiscriminateDiscoveryTests` | 깨진 JSON 건너뜀, 빈 gym ok |
| `DiscriminateMatrixTests` | 세 종류의 실행 횟수와 false-pass 분리 |
| `BuildThenScoreIntegrationTests` | 채점 직전에 파일이 실제로 그 바이트 |
| `HumanReportTests` | 사람 출력 문장 |
| `CliMainTests` | `--bin`/`--json` 만, 종료 0/1 |
| `ThreeControlKindTableTests` | #5255 DoD 표 |

## 8. 기존 다섯 시험과의 호환

devel 시험이 가정하는 것:

- `WRONG_SENTINEL` 문자열 그대로
- `GARBAGE_BYTES` 상수로 비교
- `build_negative(task, dir)` 기본 모드가 입력 복사
- `build_negative(..., artifact_mode="garbage")`
- `discriminate(bin, gym, neg)` 서명
- `runner.score_task` 를 갈아끼우면 채점이 바뀜
- artifact 한 과제의 `controlCount == 2`
- garbage 만 통과하면 `falsePassControls == ["p1/T (garbage)"]`
- `falsePass` 라벨은 `pack/task`

이 가지는 서명을 늘리지 않고 반환 키만 추가했다. 기존 다섯 시험은
그대로 통과해야 한다.

## 9. 크기 게이트

이슈 DoD: `additions >= 3000` vs `upstream/devel`. 허용 파일은 네 개다.

- `gym/tools/discriminate.py`
- `scripts/tests/test_gym_discriminate.py`
- `gym/docs/discriminate.md`
- `mydocs/working/gym_discriminate.md`

`git add -A` 금지. 위 경로만 스테이징한다. 다른 열린 PR 파일
(`trajectory.py`, `fuzz_corpus.py`, `gym/packs/automation`,
`gym/packs/core-cli`, `gym/packs/casual-rides`) 은 워킹트리에
나타나도 커밋에 넣지 않는다.

## 10. 설계에서 버린 대안

1. **네 번째 대조 `empty-file`.** 빈 파일은 `file_exists` 기본값에
   걸린다. 형식 검사가 없어도 거부되므로 판별력 신호가 약하다.
2. **네 번째 대조 `missing-file`.** 파일을 안 쓰는 것은 이미 입력
   부재 copy 경로가 한다. 종류를 늘리면 카탈로그 시험이 깨진다.
3. **artifact 에 `wrong-answer` 를 세 번째 대조로 추가.** 기존 시험
   `controlCount == 2` 가 깨진다. 답 키는 copy/garbage 제출에도
   sentinel 로 같이 쓰므로 세 번째 루프가 필요 없다.
4. **보고 `schemaVersion=1.1`.** 필수 키 의미가 그대로라 올릴 이유가
   없다. 부가 키는 `OPTIONAL_REPORT_KEYS` 로 남긴다.
5. **`--out` 파일 쓰기.** 게이트는 stdout 의 사람 출력을 본다.
   쓰기 경로를 늘리면 CLI 표면이 바뀐다.

## 11. 로컬 명령 기록

격리 worktree 에서 실행한 것:

```
python -m unittest scripts.tests.test_gym_discriminate
python gym/tools/audit.py
```

`cargo fmt --all` 과 `cargo fmt --all -- --check` 는 돌리지 않았다.
Rust 파일이 없다. 사용자 지시가 명시적으로 금지했다.

단위 시험은 바이너리 없이 150건 전후가 통과해야 한다. `audit.py` 는
devel 의 18 pack 정합을 그대로 통과해야 한다. pack JSON 을 이 가지가
만지지 않으므로 audit 실패는 회귀가 아니라 작업 트리 오염이다.

PR 본문은 한글, base 는 `devel`, `closes #5255`, `--body-file` 로
보낸다. 제목은 도구와 세 대조를 드러낸다.

닫는 문장: 음성 대조가 통과하면 과제가 틀린 것이 아니라 오라클이
약한 것이다. 그 신호를 시험이 고정한다.

이상.

(이 기록은 작업 메모다. 규약 정본은 `gym/docs/discriminate.md`.)
