---
kind: working
status: active
canonical: mydocs/working/task_m100_6628_stage3.md
issue: 6628
last_verified: 2026-09-02
---

# #6628 Stage 3 — 약한 answer/artifact 오라클 정산

## 1. 결론

replay된 `84d89dc50` 직전과 현재 tree를 같은 현행 판별기로 대조했다. 패치 직전에는
19개 task의 적용 가능한 음성 대조 28건이 전부 잘못 통과했고, 현재는 같은 28건이
전부 거부된다. 해당 19개 기준풀이도 현재 `rhwp v0.8.6`으로 19/19 통과했다.

전수 discrimination은 1,035개 task와 1,511개 음성 대조를 실행해 false-pass 0으로
끝났다. 검사를 삭제하거나 허용 종료 코드를 넓혀 얻은 통과는 없다.

## 2. 19 task / 28 false-pass 원인 대사

| 유형 | task | task 수 | 대조 수 | 패치 전 결함 | 현재 보호 |
|---|---|---:|---:|---|---|
| 동일 해시만 검사 | `automation/AU48` | 1 | 2 | 네 파일을 같은 원본 복사/garbage로 채우면 `same_hash` 통과 | 캡슐·가림본·개봉·복원본의 JSON key와 kind까지 검사 |
| 파일 존재·크기만 검사 | `studio-e2e/ST24`, `ST27`, `ST28`, `ST29`, `ST37`, `ST38` | 6 | 12 | 입력 복사와 1 KiB garbage가 `file_exists` 통과 | `text_file_envelope_eq`로 라이브 CSV 전체 바이트 의미 대조 |
| live-only answer | `studio-e2e/ST39`, `ST40` | 2 | 2 | 현재 rhwp 값만 검사하고 제출 `answer.json`을 읽지 않음 | answer 숫자 형식과 라이브 값/길이 일치 검사 |
| live-only answer | `work-receipt/WR05`, `WR08`, `WR09`, `WR10`, `WR14`, `WR15`, `WR16`, `WR50` | 8 | 8 | 상수·라이브 봉투만 검사하고 sentinel answer를 무시 | 지목 answer를 같은 라이브 봉투 좌표와 대조 |
| 오류 봉투 허용 | `work-receipt/WR20`, `WR44` | 2 | 4 | 손상 artifact가 만든 exit 3 봉투의 일부 숫자도 허용 | exit 0만 허용하고 `valid=true` 또는 `reproduced=1`까지 검사 |
| 합계 | 19 task | **19** | **28** | **28/28 false-pass** | **28/28 거부** |

artifact task에는 `input-copy`와 `garbage` 두 대조가, answer task에는
`wrong-answer` 한 대조가 적용된다. 따라서 위 task 목록과 현행 대조 생성 규칙만으로도
`1×2 + 6×2 + 2×1 + 8×1 + 2×2 = 28`이 재계산된다.

## 3. 전체 텍스트 대조 연산자의 보호 계약

`text_file_envelope_eq`는 task JSON에 CSV 정답을 복제하지 않는다. 채점 시점
`chart-to-csv --json`의 지목 문자열과 제출 파일 전체를 비교한다.

- CRLF, BOM, 셀 값까지 문자열 전체가 같아야 통과한다.
- 라이브 봉투 텍스트는 UTF-8 기준 8 MiB를 넘으면 비교하지 않고 실패한다.
- 제출 파일의 크기가 다르면 내용을 읽지 않고 실패한다. 거대한 제출물을 메모리에
  올리는 경로가 아니다.
- 크기가 같을 때만 UTF-8로 읽으며 잘못된 인코딩은 실패한다.
- 결과 상세에는 원문 대신 SHA-256과 UTF-8 바이트 수만 남긴다.

기존 정상·변조·크기 불일치·비문자 봉투 시험에 다음 음성 계약을 추가했다.

- 상한보다 큰 제출 파일은 읽기 전에 거부
- 상한보다 큰 라이브 봉투 텍스트 거부와 원문 비노출
- 같은 크기의 잘못된 UTF-8 제출 거부와 바이트 비노출

관련 operator/score/discrimination/pack 계약 327건과 전수 Gym Python 3,149건이
성공했다. 전수 묶음의 기존 정책상 skip 1건은 Stage 2와 동일하다.

## 4. reference와 현재 CLI 좌표

현재 바이너리의 `capabilities --search <명령> --json`으로 다음 표면을 확인했다.

| 명령 | 이 task군에서 쓰는 좌표 | capabilities 결과 |
|---|---|---|
| `chart-to-csv` | `chartCount`, `charts`, `charts[0].csv` | 명령·`--json`·record field 존재 |
| `replay` | `mode`, `input`, `reproduced`, `expectedOutputSha256` | 명령·`--json`·record field·exit 3 불일치 계약 존재 |
| `lineage` | `depth`, `valid` | 명령·`--json`·record field 존재 |
| `audit` | `total`, `reproduced` | 명령·`--json`·record field 존재 |

`ST40` reference는 배열 자체가 아니라 제출 answer 계약과 같은 길이를 만들도록
`len: true`를 사용한다. `WR14`~`WR16`은 거짓 해시 기각을 시험하므로 reference의
answer 수집 단계만 의도된 exit 3을 허용한다. task 채점은 그 오류 봉투의 지목 값과
제출 answer까지 함께 대조하므로 허용 범위를 넓힌 false-pass가 아니다.

실행 바이너리는 `rhwp v0.8.6`, SHA-256
`14d0e8ef71f762a062ecf436c88e7d9b0ea719ce9a22178c8e51c09567ddddec`였다.
이 Stage의 최종 source head는 Python·task·문서 변경뿐이므로 Rust 바이너리를 다시
빌드하지 않았다.

## 5. 표적 패치 전후와 positive 대조

19개 task만 임시 tree에 materialize해 동일 바이너리와 현행
`discriminate.py`로 측정했다. 임시 제출물은 실행 종료와 함께 제거했다.

```text
before 84d89dc50:
  task 19 · control 28 · false-pass task 19 · false-pass control 28 · error 0

current:
  task 19 · control 28 · false-pass task 0 · false-pass control 0 · error 0

current targeted positive:
  task 19 · built 19 · failed 0 · skipped 0
```

positive 대조는 같은 19개 task/reference만 `build_baseline.py`의 실제
`process_one_task` 경로로 실행했다. 전수 positive baseline은 계획대로 Stage 4에서
한 번만 실행한다.

## 6. 전수 discrimination 결과

```text
python3 gym/tools/discriminate.py --bin target/debug/rhwp --json

kind             gymDiscrimination
schemaVersion    1.0
ok               true
taskCount        1,035
controlCount     1,511
discriminating   1,035
falsePass        0
falsePassControls 0
loadErrors       0
buildErrors      0
toolErrors       0
toolFailed       false
elapsed          446.53s
```

결과 행은 1,511개이며 거부되지 않은 행은 0이다. `validate_report` 위반도 0이고,
`discriminating == taskCount - len(falsePass)` 산술도 일치한다.

### `scoreErrors` 116건의 해석

JSON에는 `scoreErrors` 116건이 함께 남았다. 이를 숨기지 않고 전건 분류했다.

- 고유 task: artifact 과제 58개
- control: `input-copy` 58건 + `garbage` 58건
- 사유: 전부 `answer.json 파싱 실패`
- CLI·경로·도구 오류: 0

이 task들은 `answer.json`을 artifact 파일 목록에도 포함한다. 음성 대조가 모든 선언
artifact에 HWP 입력 바이트 또는 garbage를 쓰므로 `answer.json`이 잘못된 UTF-8/JSON이
되고, 채점기가 제출 형식 단계에서 거부한다. 현행 discrimination 정본은 채점 예외를
false-pass로 뒤집지 않고 `scoreErrors`에 보존한다. 따라서 116건은 정상 제출 실패나
오라클 중단이 아니라 의도된 불량 제출의 조기 거부다. `ok=true`만 읽어 이 사실을
누락하지 않도록 최종 보고에도 수치와 유형을 유지한다.

## 7. 구조·회귀 검증

```text
python3 -m unittest discover -s scripts/tests -p 'test_gym_*.py'
Ran 3149 tests — OK (skipped=1)

python3 gym/tools/audit.py --json
21 pack · task/reference 1,035/1,035 · issue 0

git diff --check
통과
```

## 8. Stage 3 판정과 다음 단계

Stage 3 종료 게이트를 충족했다.

- 알려진 19 task/28 false-pass의 패치 직전 재현: 충족
- 같은 19 task/28 control의 현재 거부와 positive 19/19: 충족
- 새 연산자의 정상·변조·과대 입력·인코딩 실패 보호: 충족
- reference/CLI 명령·결과 좌표 일치: 충족
- 전수 1,035 task/1,511 control false-pass 0: 충족
- 검사 삭제·exit 허용 확대로 얻은 통과 없음: 충족

다음 Stage 4는 전수 positive baseline과 trajectory를 실행한다. discrimination은 이번
Stage에서 이미 현행 head로 전수 완료했으므로 같은 source head라면 중복 실행하지 않고
이 원문 결과를 Stage 4의 음성 축 증적으로 재사용한다. source가 바뀌면 다시 실행한다.
