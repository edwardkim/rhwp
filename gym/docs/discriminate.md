---
kind: guide
status: active
canonical: gym/docs/discriminate.md
last_verified: 2026-08-18
---

# gym 판별력 감사 규약

이 문서는 `gym/tools/discriminate.py` 의 **음성 대조 종류**, **오답
sentinel**, **artifact 무편집 복사**, **garbage 산출**, **보고 봉투**,
**경로 안전**, **예외 접기**를 고정한다. 작업 기록은
[`mydocs/working/gym_discriminate.md`](../../mydocs/working/gym_discriminate.md)
를 본다. 시험 계약은 `scripts/tests/test_gym_discriminate.py` 가 기계로
고정한다.

릴리스 차등(`release_diff.py`)은 시간축이고, 교차형식 차등
(`differential.py`)은 형식축이다. 이 도구는 **약한 오라클 축**이다.
채점이 '파일이 있나'만 보면 일 안 한 제출이 만점을 받는다. 그 결함을
과제 등재 전에 색출한다.

## 1. 왜 이 기둥이 필요한가

2026 벤치마크의 최대 위기는 false-pass 다. OpenAI 감사에서 SWE-Bench
Verified 최난도 과제의 59.4%가 버그를 고치지 않아도 테스트가 통과했다.
운동장도 같은 구멍에 빠질 수 있다.

- answer 과제가 키 존재만 보면 아무 문자열이나 통과한다.
- artifact 과제가 `file_exists` 만 보면 입력 복사본이 통과한다.
- artifact 과제가 `differs_from_input` 만 보면 1KiB garbage 가 통과한다.

그래서 각 과제에 **일부러 틀린 제출**을 넣어 채점한다. 그 제출이
통과하면 과제는 판별력이 없다. 거부하면 진짜 일을 요구한다.

이 도구는 "정답이 무엇인가"를 다시 계산하지 않는다. 러너가 이미 하는
일이다. 여기서 묻는 것은 하나다: **일 안 한 제출을 거부하는가.**

## 2. 사용

```bash
python gym/tools/discriminate.py --bin target/debug/rhwp
python gym/tools/discriminate.py --bin target/debug/rhwp --json
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--bin` | (필수) | rhwp 바이너리. `runner.find_bin` 이 상대경로를 절대화한다. |
| `--json` | 꺼짐 | 사람 요약 대신 `gymDiscrimination` 봉투를 stdout 에 쓴다. |

새 플래그는 없다. `--pack` / `--task` / `--limit` / `--out` 은 없다.
전 pack 을 도는 것이 이 감사의 점이다. 한 과제만 봐서 약한 오라클이
없다고 말하면 거짓말이다.

종료 코드:

| 코드 | 상수 | 의미 |
|---|---|---|
| 0 | `EXIT_OK` | `falsePass` 가 비었다 |
| 1 | `EXIT_FALSE_PASS` | 약한 오라클이 한 건이라도 있다 |

도구 자리 오류(`loadErrors` · `scoreErrors`)는 집계에 남기되, 음성
대조가 실제로 통과하지 않는 한 `ok` 를 뒤집지 않는다. 채점이 죽은
것을 "통과"로 부르지 않고, "거부"로 위장 집계해 만점 처리하지도
않는다 — 그 행은 `discriminates=true` 이되 `scoreErrors` 에 이유가
남는다.

음성 제출물은 `gym/submissions/_negative_control/<control>/<pack>/<task>/`
아래에 다시 만든다. 매 실행마다 이 뿌리를 지우고 시작한다.

## 3. 음성 대조 종류 — 세 칸만

`CONTROL_KINDS` 는 정확히 세 값이다. 시험이 이 튜플을 고정한다.

| id | 적용 | 쓰는 파일 | 페이로드 | 거부 이유 |
|---|---|---|---|---|
| `wrong-answer` | answer (및 artifact 가 아닌 과제) | `answer.json` | `WRONG_SENTINEL` | `answer_eq` 가 진값과 대조 |
| `input-copy` | artifact | `submit.files` | `task.input` 바이트 | 무편집 복사는 일을 하지 않음 |
| `garbage` | artifact | `submit.files` | `GARBAGE_BYTES` | 입력과 다른 쓰레기만으로는 부족 |

규칙:

1. **artifact 과제는 copy 와 garbage 를 둘 다 돌린다.** 한쪽만 거부하고
   다른 쪽이 통과하면 그 과제는 약한 오라클이다.
2. **그 외 과제(`answer` · `pair` · kind 없음)는 wrong-answer 만 돈다.**
   pair 에 산출 복사를 얹지 않는다. 그 확장 축은 이 이슈의 범위가 아니다.
3. **미지 대조 id 는 카탈로그에 없다.** `validate_report` 가 거부한다.
4. **네 번째 대조를 몰래 넣지 않는다.** truncate · empty-file ·
   missing-file 은 종류가 아니다. 파일이 없으면 러너가 이미 실패한다.

`controls_for(task)`:

| `submit.kind` | 반환 |
|---|---|
| `artifact` | `("input-copy", "garbage")` |
| 그 외 | `("wrong-answer",)` |

순서는 결정적이다. copy 가 garbage 보다 앞선다. pack · 과제 파일 이름
도 사전순이다.

## 4. 오답 sentinel

상수:

```
WRONG_SENTINEL = "__NEGATIVE_CONTROL_definitely_wrong__"
```

성질:

- 숫자 진값(0, 1, 쪽수, 표 수)과 타입이 다르다.
- 흔한 문자열 진값(`""`, `"0"`, `"ok"`, `"pages"`, `"true"`)과 값이 다르다.
- `None` / `True` / `False` / `[]` / `{}` 와도 다르다.
- `answer_eq` 의 `norm` 이 숫자 문자열을 float 로 접어도 이 문자열은
  숫자가 아니므로 접히지 않는다.

`answer_keys(task)` 는 `checks[].answer` 가 비지 않은 문자열인 것만
모은다. 빈 문자열 · 숫자 · 비-dict 검사 · `checks` 부재는 무시한다.

키가 있으면 `answer.json` 을 UTF-8 · BOM 없음 · `ensure_ascii=False` 로
쓴다. 키 순서는 정렬한다. 키가 없으면 `answer.json` 을 만들지 않는다.

artifact 과제에도 답 키가 있으면 sentinel 을 같이 쓴다. copy/garbage
대조가 산출만 바꾸고 답을 진값으로 남겨 통과하는 구멍을 막기 위함이다.
`wrong-answer` 모드를 artifact 에 직접 주면 산출 파일은 쓰지 않고 답만
쓴다.

## 5. artifact 무편집 복사

`artifact_mode="input-copy"` 일 때 각 안전 상대경로에 `task.input` 을
`shutil.copyfile` 한다.

- 입력 경로는 저장소 루트 기준 상대경로이거나 절대경로다.
- 입력 파일이 없으면 그 산출은 **건너뛴다.** 빈 파일을 지어내지 않는다.
  예외로 전수 감사를 죽이지 않는다. 건너뛴 사실은 `buildErrors` 에 남는다.
- 원본 픽스처는 읽기만 한다. 덮어쓰지 않는다.
- 같은 입력을 여러 산출 자리에 복사할 수 있다. 각각의 해시가 입력과
  같으면 `differs_from_input` 이 거부해야 한다.

`differs_from_input` 만 있는 과제는 garbage 를 통과시킬 수 있다. 그래서
복사 대조만으로 판별력을 선언하지 않는다.

## 6. garbage 산출

```
GARBAGE_MARKER = b"RHWP_GYM_GARBAGE_NEGATIVE_CONTROL\x00"
GARBAGE_REPEAT = 64
GARBAGE_BYTES  = GARBAGE_MARKER * 64
GARBAGE_MIN_SIZE = 1024
```

성질:

- 길이는 1KiB 를 넘는다(`garbage_meets_minimum`).
- UTF-8 로 디코드되지 않는다(널 바이트).
- JSON 객체도 XML 도 PDF 도 아니다.
- 입력 샘플 바이트와 다르다. `differs_from_input` 은 통과할 수 있다.
- 입력 파일이 없어도 쓴다. garbage 는 원본이 필요 없다.

이 대조가 통과하면 과제는 형식·핵심값 검사가 없다. `file_exists` +
`differs_from_input` 만으로는 부족하다는 것이 이 칸의 점이다.

실측 근거: `SR05` 는 `file_exists`(minBytes 128) + `differs_from_input` +
`json_value_eq(dialect=…)` 를 같이 건다. 앞의 두 칸만 있으면 garbage 가
통과한다. 세 번째 칸이 있어서 거부된다.

## 7. 경로 안전

`submit.files` 항목은 제출 폴더 안으로만 쓴다.

| 입력 | `normalize_rel` | `unsafe_rel_reason` |
|---|---|---|
| `out.svg` | `out.svg` | `None` |
| `a/b/c.svg` | `a/b/c.svg` | `None` |
| `./a/./b` | `a/b` | `None` |
| `a\b\c` | `a/b/c` | `None` |
| `../x` | `None` | `parent` |
| `a/../b` | `None` | `parent` |
| `/abs` | `None` | `absolute` |
| `C:/abs` | `None` | `drive` |
| `//unc/x` | `None` | `unc` |
| `~/x` | `None` | `home` |
| `""` / `None` / `3` | `None` | `empty` / `not-str` |

거절된 항목은 쓰지 않는다. 이웃한 안전 경로는 그대로 쓴다. 전수 감사를
죽이지 않는다.

`join_sub` 는 정규화된 상대경로만 받는다. 불안전하면 `ValueError`.

## 8. JSON 봉투

`kind=gymDiscrimination`, `schemaVersion=1.0`. 키 집합은 시험이
`REPORT_KEYS` 로 고정한다.

| 키 | 형 | 의미 |
|---|---|---|
| `kind` | str | 항상 `gymDiscrimination` |
| `schemaVersion` | str | 항상 `1.0` |
| `ok` | bool | `falsePass` 가 비었을 때만 참 |
| `taskCount` | int | 읽기에 성공한 과제 수 |
| `controlCount` | int | 돌린 대조 행 수 = `len(results)` |
| `discriminating` | int | `taskCount - len(falsePass)` |
| `falsePass` | list | `pack/task` 라벨. 과제당 한 번 |
| `falsePassControls` | list | `pack/task (control)` 라벨. 대조마다 |

부가 키:

| 키 | 언제 |
|---|---|
| `results` | 모든 대조 행 `{pack,task,control,discriminates}` |
| `loadErrors` | 깨진 JSON · id 없음. 그 파일은 과제 수에 넣지 않는다 |
| `scoreErrors` | 채점 예외. 그 행은 통과로 치지 않는다 |
| `buildErrors` | 입력 없음 · 불안전 경로 |
| `toolFailed` / `toolErrors` | packs 목록 실패 |
| `controlKinds` | 카탈로그 id 세 개 |

`validate_report` 계약:

- 필수 키가 있다.
- `ok` 와 `falsePass` 가 모순되지 않는다.
- `discriminating == taskCount - len(falsePass)`.
- `controlCount == len(results)` (results 가 있을 때).
- 대조 id 는 카탈로그에 있다.
- `falsePass` 는 `pack/task` 형식이다.
- `falsePassControls` 는 `pack/task (control)` 형식이다.

한 과제가 copy 는 거부하고 garbage 는 통과하면 `falsePass` 에는 과제
라벨이 한 줄, `falsePassControls` 에는 `(garbage)` 한 줄이다. 두 대조가
모두 통과해도 과제 라벨은 한 줄이다.

깨진 과제 파일은 `taskCount` 에 넣지 않는다. 없는 과제를 판별력 있다고
부르지 않기 위함이다.

## 9. 예외 경로 — 한 과제가 전수 감사를 죽이지 않는다

| 자리 | 잡는 것 | 접는 곳 |
|---|---|---|
| 과제 JSON | `OSError` · `ValueError` · 비-객체 | `loadErrors`, 건너뜀 |
| 산출 경로 | 불안전 상대경로 | `buildErrors`, 그 파일만 거부 |
| 입력 복사 | 원본 없음 | `buildErrors`, 파일 생략 |
| 채점 | `CATCHABLE_EXCEPTIONS` | `pass=False` + `scoreErrors` |
| packs 목록 | `OSError` | `toolFailed`, 빈 보고 |

삼키면 안 되는 예외: `KeyboardInterrupt` · `SystemExit` · `MemoryError` ·
`GeneratorExit`. 사용자가 끊었는데 약한 오라클 0건이라고 쓰면 거짓말이다.

채점 예외는 false-pass 가 아니다. 통과를 관측하지 못했기 때문이다. 그
행의 `discriminates` 는 참(거부한 것으로 집계)이고 이유는
`scoreErrors` 에 남는다. 반대 — 예외를 통과로 접으면 — 감사기가 스스로
약한 오라클이 된다.

## 10. 사람 출력

`--json` 이 아니면 두 갈래다.

성공:

```
gym 판별력 감사: N 과제 전부 음성 대조를 거부 — 약한 오라클 0
```

실패:

```
gym 판별력 감사: 약한 오라클(false-pass) K건 — 일 안 한 제출이 통과한다:
  - pack/task
대조별:
  - pack/task (control)
```

기존 한 줄 형식(`N 과제 전부…` / `약한 오라클(false-pass) K건`)은
유지한다. 시험과 릴리스 게이트가 그 문장을 본다.

## 11. 하지 않는 일

- 새 rhwp CLI 를 만들지 않는다.
- 새 gym pack 을 만들지 않는다.
- `trajectory.py` · `fuzz_corpus.py` · `automation` / `core-cli` /
  `casual-rides` pack 을 고치지 않는다.
- 열린 PR 이 만지는 파일을 고치지 않는다.
- 라이브 채점 기대를 골든 파일로 박제하지 않는다. 기대값은 러너가
  계산하고, 이 도구는 음성 제출만 만든다.

## 12. 다른 기둥과의 자리

| 도구 | 축 | 거짓말을 막는 관문 |
|---|---|---|
| `discriminate.py` | 약한 오라클 | 음성 대조가 반드시 실패 |
| `audit.py` | 정합 | 과제↔기준 짝, ID 고유 |
| `differential.py` | 형식 | 본문 해시가 다를 때 결함으로 부르지 않음 |
| `release_diff.py` | 시간 | 표면 변경을 regression 으로 부르지 않음 |

릴리스 게이트(`gym-release-gate.yml`)는 차등 **이전**에 이 감사를 돈다.
벤치마크 자체가 성립하는지 먼저 본다. 표면 변경과 달리 약한 오라클은
리뷰 신호가 아니라 결함이다.

## 13. 시험이 고정하는 표

`scripts/tests/test_gym_discriminate.py` 의 핵심 칸:

1. `wrong-answer` 는 `WRONG_SENTINEL` 을 쓰고 입력 바이트를 쓰지 않는다.
2. `input-copy` 는 저장소 샘플 바이트와 같고 `GARBAGE_BYTES` 와 다르다.
3. `garbage` 는 `GARBAGE_BYTES` 와 같고 입력 바이트와 다르다.
4. artifact 과제의 `controlCount` 는 2 다.
5. garbage 만 통과하면 `falsePassControls` 는 `pack/task (garbage)` 한 줄이다.
6. 세 종류 이외의 대조 id 는 보고 검증이 거부한다.

이 여섯 칸이 깨지면 도구를 고쳐야 한다. 시험을 느슨하게 만들지 않는다.

## 14. 연산자가 어느 대조를 거부해야 하는가

채점 연산자는 이 도구가 만들지 않는다. 다만 음성 대조가 통과하면
**어느 연산자가 비었는지**가 바로 드러난다. 아래 표는 과제 작성자가
약한 오라클을 피할 때 보는 대응이다.

| 대조가 통과함 | 비어 있는 검사 | 넣어야 할 연산자 |
|---|---|---|
| `wrong-answer` | 답을 진값과 대조하지 않음 | `answer_eq` / `len_answer_eq` |
| `input-copy` | 입력과 같은 바이트를 허용 | `differs_from_input` |
| `garbage` | 입력과 다른 쓰레기면 통과 | `json_value_eq` · `xml_root_eq` · `csv_cell_eq` · 형식 검사 |
| copy 는 거부, garbage 는 통과 | 형식·핵심값이 없음 | 위에 더해 지목 검사 |
| 두 artifact 대조가 모두 통과 | 파일 존재만 봄 | `file_exists` 만으로는 부족 |

`file_exists` 의 `minBytes` 는 garbage(1KiB 초과)를 막지 못한다.
`differs_from_input` 은 복사를 막지만 garbage 는 통과시킨다. 둘을
같이 걸어도 garbage 칸이 남는다. 그래서 산출 과제는 **형식 또는
핵심값**을 한 칸 더 둬야 한다.

`SR03`(answer, 손실 계수) — `answer_eq(loss)` 가 있으면 sentinel 이
거부된다. 이 칸이 없으면 `wrong-answer` 가 만점이다.

`SR05`(artifact, IR 스키마) — `file_exists` + `differs_from_input` +
`json_value_eq(dialect=…)` 세 칸이 있어야 copy 와 garbage 를 둘 다
거부한다. 앞의 두 칸만 있으면 garbage 가 통과한다.

## 15. 음성 제출 디렉터리

한 번의 전수 감사는 아래 나무를 지우고 다시 만든다.

```
gym/submissions/_negative_control/
├── wrong-answer/<pack>/<task>/answer.json
├── input-copy/<pack>/<task>/<submit.files…>
└── garbage/<pack>/<task>/<submit.files…>
```

- 뿌리는 `NEGATIVE_DIRNAME` (`_negative_control`) 이다.
- 대조 id 가 첫 폴더다. 채점 목킹이 경로에 `garbage` / `input-copy`
  를 보고 갈라지는 시험이 이 배치를 전제로 한다.
- 과제 폴더 이름은 `task.id` 다. 파일 이름과 id 가 달라도 id 를 쓴다.
- 매 실행 `shutil.rmtree` 후 시작한다. 이전 실행의 쓰레기 파일이
  다음 채점을 통과시키지 못하게 한다.

라이브 채점은 `runner.score_task(task, <control>/<pack>, bin)` 이다.
러너는 `<control>/<pack>/<task.id>` 를 제출 폴더로 본다.

## 16. 보고 봉투 보기

약한 오라클이 없는 최소 봉투:

```json
{
  "kind": "gymDiscrimination",
  "schemaVersion": "1.0",
  "ok": true,
  "taskCount": 2,
  "controlCount": 3,
  "discriminating": 2,
  "falsePass": [],
  "falsePassControls": []
}
```

answer 1 + artifact 1 이면 `controlCount` 는 3 이다. `ok` 는
`falsePass` 가 비었을 때만 참이다. `discriminating` 은
`taskCount - len(falsePass)` 이지 `controlCount` 가 아니다.

garbage 만 통과한 artifact 한 건:

```json
{
  "ok": false,
  "taskCount": 1,
  "controlCount": 2,
  "discriminating": 0,
  "falsePass": ["serialization/SR05"],
  "falsePassControls": ["serialization/SR05 (garbage)"]
}
```

과제 라벨은 한 줄이다. 대조 라벨은 통과한 대조만 남긴다. copy 는
거부했으므로 `falsePassControls` 에 `(input-copy)` 가 없다.

## 17. 결정적 순서

같은 gym 나무를 두 번 돌리면 같은 `results` 순서가 나와야 한다.

1. pack id `sorted(os.listdir)`
2. 과제 파일 이름 `*.json` 사전순
3. 그 과제의 `controls_for` 튜플 순서

`--limit` 이 없으므로 접두를 자를 자리가 없다. 순서가 흔들리면
`falsePassControls` 스냅샷 시험이 깨진다. 파일 시스템 나열 순서를
그대로 쓰지 말고 항상 정렬한다.

## 18. FAQ

**Q. 왜 garbage 를 1KiB 넘게 쓰나?**
`file_exists` 의 `minBytes` 기본은 1, 어떤 과제는 128 이다. 몇 바이트
짜리 마커는 그 칸에 걸린다. 1KiB 를 넘기면 "파일이 있다"는 이유로
통과하지 못하고, 형식 검사가 있는지만이 남는다.

**Q. 입력 파일이 없으면 copy 를 실패로 부르나?**
부르지 않는다. 쓸 바이트가 없다. 산출 파일을 만들지 않고
`buildErrors` 에 남긴다. 채점은 빈 제출을 보게 되고, 파일 검사가
있으면 거부(판별력 있음)다.

**Q. 채점이 예외를 내면 약한 오라클인가?**
아니다. 통과를 관측하지 못했다. `scoreErrors` 에 남기고
`discriminates=true` 로 집계한다. 예외를 통과로 접으면 감사기 자신이
약한 오라클이 된다.

**Q. pair 과제는 왜 copy/garbage 가 없나?**
이슈 #5255 가 고정한 종류는 answer sentinel · artifact copy ·
garbage artifact 다. pair 는 `wrong-answer` 만 돈다. 산출 쌍에
복사를 얹는 확장은 후속이다.

**Q. 새 플래그를 달면 안 되나?**
전수 감사가 점이다. `--pack` 으로 한 pack 만 보면 다른 pack 의
약한 오라클을 못 본다. CLI 표면은 `--bin` 과 `--json` 만 유지한다.

## 19. 함수 표면 (시험이 import 하는 것)

도구를 통째로 갈아엎지 않도록, 아래 이름은 시험을 깨지 않고는
지우지 않는다.

| 이름 | 순수? | 역할 |
|---|---|---|
| `WRONG_SENTINEL` / `GARBAGE_BYTES` | 상수 | 페이로드 |
| `CONTROL_KINDS` / `CONTROL_CATALOG` | 상수 | 세 종류 |
| `answer_keys` | 예 | 답 키 집합 |
| `controls_for` | 예 | 과제 → 대조 튜플 |
| `normalize_rel` / `unsafe_rel_reason` | 예 | 경로 안전 |
| `build_negative` | 아니오 | 제출 폴더 구성 |
| `score_discriminates` | 예 | 채점 봉투 → 판별력 |
| `aggregate_rows` / `validate_report` | 예 | 보고 정직 |
| `discriminate` | 아니오 | 전수 루프 |
| `parse_args` / `main` | 아니오 | `--bin` · `--json` |

`discriminate(..., score_fn=)` 는 시험 주입용이다. CLI 플래그가 아니다.
주입이 없으면 `runner.score_task` 를 쓴다.

## 20. 잘못된 수정 예시

이 도구를 고칠 때 하지 말아야 할 것.

1. **garbage 를 빈 파일로 바꾸기.** `file_exists` 기본 `minBytes=1` 에
   걸려 형식 검사 없이도 거부된다. 그러면 garbage 칸이 사라진다.
2. **copy 만 돌리고 garbage 를 옵션으로 빼기.** `differs_from_input`
   만 있는 과제가 만점을 받는다.
3. **`ok` 를 `toolFailed` 와 AND 하기.** 디스크 오류와 약한 오라클을
   같은 종료 코드로 묶으면 게이트가 원인을 못 가린다.
4. **깨진 JSON 을 `taskCount` 에 넣기.** 없는 과제를 판별력 있다고
   부르게 된다.
5. **`submit.files` 의 `..` 를 그대로 `join` 하기.** 음성 대조가
   저장소 밖의 파일을 덮어쓴다.
6. **채점 예외를 `pass=True` 로 접기.** 감사기 자신이 약한 오라클이
   된다.

## 21. 게이트와의 연결

`.github/workflows/gym-release-gate.yml` 는 구/신 바이너리 차등 앞에
`python gym/tools/discriminate.py --bin <new>` 를 돈다. 이 가지가
워크플로 파일을 고치지 않는 이유: 열린 PR 과 겹치고, CLI 표면이
그대로라 고칠 것이 없다.

게이트가 보는 것:

- 종료 코드 0/1
- 사람 출력의 `약한 오라클` 문장 (json 이 아님)

`--json` 은 로컬·시험용이다. 게이트는 기본(사람) 출력을 쓴다.

## 22. 상수 표

시험이 값을 고정하는 상수다. 바꾸면 기존 다섯 시험 또는 카탈로그
시험이 깨진다.

| 상수 | 값 |
|---|---|
| `WRONG_SENTINEL` | `__NEGATIVE_CONTROL_definitely_wrong__` |
| `GARBAGE_MARKER` | `b"RHWP_GYM_GARBAGE_NEGATIVE_CONTROL\x00"` |
| `GARBAGE_REPEAT` | `64` |
| `GARBAGE_MIN_SIZE` | `1024` |
| `REPORT_KIND` | `gymDiscrimination` |
| `SCHEMA_VERSION` | `1.0` |
| `NEGATIVE_DIRNAME` | `_negative_control` |
| `EXIT_OK` / `EXIT_FALSE_PASS` | `0` / `1` |

`GARBAGE_BYTES` 는 `MARKER * REPEAT` 이다. 길이를 손으로 하드코딩하지
말고 `garbage_size()` 와 `garbage_meets_minimum()` 을 본다.

`expected_control_count(task)` 는 `len(controls_for(task))` 다.
artifact 는 2, 그 외는 1. 보고의 `controlCount` 는 전 과제 합이다.

