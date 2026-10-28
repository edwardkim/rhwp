---
kind: working
status: active
canonical: mydocs/working/task_m100_6628_stage2.md
issue: 6628
last_verified: 2026-09-02
---

# #6628 Stage 2 — 전수 Python 계약 정상화

## 1. 결론

Stage 0에서 고정한 전수 기준선의 5 failure·4 error를 원인별로 정산했다.
`test_gym_*.py` 전수 결과는 3,146건 성공, 정책상 skip 1건이다. 깨진 시험을
삭제·skip하거나 기대값을 느슨하게 하지 않았다.

pack-health 196건은 메인테이너가 승인한 disposition에 따라 192개 검사기
오검출과 4개 실제 과제 지시 경계 결함으로 분리했다. 전역 `--exclude`와 예외
원장은 0개다.

## 2. 실패군별 원인과 처리

| 초기 실패군 | 초기 결과 | 원인 | 처리 |
|---|---:|---|---|
| coverage CLI | error 3 | 환경의 `python` 이름이 실행 파일이 아님 | 현재 시험 인터프리터 `sys.executable` 사용 |
| profile | failure 2, error 1 | 현행 구조화 예외와 과거 `FileNotFoundError` 기대 불일치, 구현 문자열 snapshot | `ScoreRunnerError.kind=missing-profile`과 실제 pack 선택 순서 검증 |
| schema hint table | failure 1 | 새 파일 연산자 16개의 권장 필드 누락 | 실제 연산자 접근 필드로 `CHECK_FIELD_HINTS` 완성 |
| tutorial front matter | failure 1 | 과거 날짜 `2026-08-18` 정문 고정 | kind/status/canonical과 ISO 날짜 형식 검증 |
| pack-health | failure 1, issue 196 | 문맥 없는 힌트 탐지와 4개 본문 요구사항 누락 | 아래 disposition 적용 |

profile 시험은 더 이상 `runner.py`의 대입문 한 줄을 검색하지 않는다. 가짜 profile과
pack scorer를 주입해 `profile_id`가 호출자 `pack_ids`보다 우선하며 선언 순서대로
채점되는지를 실행한다. 구현을 동등하게 리팩터링해도 동작이 같으면 통과한다.

## 3. pack-health 196건 disposition

| 진단 | 수 | 판정 | 처리 |
|---|---:|---|---|
| `hint_spoiler` | 57 | 오검출 | “정답 숫자를 박제하지 마라”라는 금지문을 “답은 N”과 구분 |
| `hint_answer_dump` | 133 | 오검출 | 경로 인덱스 102건과 명령 입력·본문 반복 JSON 31건을 answer 출력과 구분 |
| `check_missing_value` | 1 | 오검출 | `value_in`은 단수 `value`가 아니라 복수 `values`를 요구 |
| `hint_embeds_check_value` | 1 | 오검출 | WR16 해시는 판정 명령에 넣은 입력 인수이며 숨은 출력이 아님 |
| `hint_embeds_check_value` | 4 | 실제 지시 경계 결함 | 필수 요구사항을 힌트에서 본문으로 이동 |

실제 과제 수정은 다음 네 파일의 `instructions`뿐이다.

- `automation/AU14`: `(0,1)`에 넣을 `옆칸확장`을 본문에 명시
- `core-cli/T50`: `(0,0)`에 넣을 `다중표온램프`를 본문에 명시
- `text-editing/TE46`: 산출 형식 `hwp5`를 본문에 명시
- `text-editing/TE54`: 산출 형식 `hwp5`를 본문에 명시

네 task의 `submit`, `checks`, 짝 reference는 바꾸지 않았다. 과제를 쉽게 만들거나
채점 강도를 낮춘 변경이 아니라, 힌트를 열어야만 알 수 있던 필수 작업 입력을 지시
본문으로 올린 변경이다.

## 4. 힌트 탐지 보호 계약

검사기는 다음 문맥을 구별한다.

- `fields[0]` 같은 식별자 뒤 대괄호는 JSON answer 배열이 아니다.
- 독립된 `[4]`는 여전히 구체 answer 배열로 거부한다.
- `--data '{...}'`는 편집 명령 입력이며 채점 결과 봉투가 아니다.
- 본문에서 이미 요구한 key/value를 힌트 명령이 반복해도 유출이 아니다.
- check의 `cmd`에 명시적으로 들어간 기대 인수는 숨은 출력 정답이 아니다.
- “답은 4”와 “정답 숫자를 박제하지 마라”를 구분한다.

각 허용 문맥에는 양성 시험을, 실제 유출에는 음성 시험을 함께 유지한다. 현재 tree를
맞추기 위한 task ID 하드코딩은 없다.

## 5. CI 경계

Gym 관련 PR의 `Gym benchmark contracts`에 다음 모듈을 명시적으로 추가했다.

- `test_gym_coverage`
- `test_gym_profiles`
- `test_gym_schema`
- `test_gym_pack_health`

추가 후 빠른 contract job과 같은 명령은 2,123건을 통과했고 정책상 skip 1건이다.
Gym-only PR에서만 실행되며 제품 PR, devel/main push, tag와 릴리스 게시의 게이트로
확장하지 않는다.

## 6. 검증 결과

```text
python3 -m unittest discover -s scripts/tests -p 'test_gym_*.py'
Ran 3146 tests — OK (skipped=1)

python3 -m unittest scripts.tests.test_gym_pack_health
Ran 142 tests — OK

python3 gym/tools/pack_health.py --json
21 pack · 1,035 task · issue 0 · error 0 · warning 0

python3 gym/tools/audit.py --json
21 pack · task/reference 1,035/1,035 · issue 0

python3 gym/tools/authority_ledger.py --json
task/reference/entry 1,035/1,035/1,035 · issue 0
```

남은 skip은 `test_gym_release_diff.CommittedReportTests.test_self_diff_report_is_stable`의
“커밋된 self-diff 리포트 없음” 정책 분기다. 실패·오류가 아니며 이번 수정으로 새로
생기지 않았다.

권위 분포는 Stage 1과 동일하다: `self-live 987`, `contract-constant 28`,
`independent-fixture 20`, `external-oracle 0`. task 지시문만 고쳤으므로 판정 권위와
baseline source는 변하지 않았다.

## 7. Stage 2 판정과 다음 단계

Stage 2 종료 게이트를 충족했다.

- 전수 Gym Python failure/error 0: 충족
- 환경별 Python 이름 비의존: 충족
- profile 동작 계약과 구조화 오류 일치: 충족
- 등록 연산자 권장 필드 완전성: 충족
- pack-health issue 0, 미승인 예외 0: 충족
- check/reference·Rust 제품 코드 비변경: 충족

다음 Stage 3은 replay된 약한 answer/artifact 오라클 변경을 task별로 대사하고,
정상·변조·과대 입력·인코딩 실패 음성 시험이 채점 강도를 실제로 보존하는지
확인한다.
