# #4586 Stage 2 완료보고서 — CLI·gym 판정 계약 구현

- **Issue**: [#4586](https://github.com/edwardkim/rhwp/issues/4586)
- **브랜치**: `task/4586-gym-t12-hwpx`
- **RED 커밋**: `a4e8f8831`

## 1. CLI 출력 가드

`src/main.rs`의 `convert_hwp`가 위치 인자를 파싱한 직후 출력 확장자를 검사한다.

- `.hwp`를 대소문자 무시로 허용한다.
- `.hwpx`, 확장자 없음, 다른 확장자는 입력 파일을 읽기 전에 exit 2로 거부한다.
- 사용법 오류의 stdout은 비어 있고 출력 파일을 만들지 않는다.
- stderr는 `convert`가 `.hwp` 출력임을 밝히고 HWPX 변환에 `export-hwpx`를 안내한다.

입력 형식에 따라 출력 포맷을 추측하거나 명령을 자동 전환하지 않았다. `convert`와 `export-hwpx`의
책임 경계를 명시적으로 유지한다.

## 2. gym 판정 계약

`gym/score.py`에 검사별 `expect_exits` 배열을 추가했다.

1. `expect_exits`가 있으면 비어 있지 않은 정수 배열인지 검증한다.
2. 없으면 기존 `expect_exit`을 단일 원소 배열로 바꾼다.
3. 둘 다 없으면 기존 기본값 0을 사용한다.
4. 실제 종료 코드가 허용 배열에 있으면 exit 3이어도 JSON 봉투를 파싱하고 값을 비교한다.

기존 task의 `expect_exit` 계약은 수정하지 않고 하위 호환으로 유지했다.

T12는 다음 두 검사를 순서대로 수행한다.

1. `info conv.hwpx --json`의 `format == "hwpx"`
2. `ir-diff` exit 0/3의 `identical`과 `answer.json` 비교

힌트도 `export-hwpx`로 고쳤다. 따라서 HWP5 위장 제출은 첫 검사에서 실패하고, 실제 HWPX의 정직한
`identical:false`는 두 번째 검사에서 정상 판정된다.

## 3. CI 배선

`.github/workflows/ci.yml`의 lint job에 `Validate gym scorer contracts` 단계를 추가했다.
`scripts/tests/test_gym_score.py`가 저장소에만 존재하고 실행되지 않는 상태를 막는다.

## 4. GREEN 검증

```text
cargo test --test issue_4586_gym_t12_contract -- --nocapture
  3 passed / 0 failed

python3 -m unittest scripts/tests/test_gym_score.py
  4 passed / 0 failed

cargo test --test output_axis_json_contract convert_json_envelope_with_verify -- --nocapture
  1 passed / 0 failed

cargo test --test issue_1638_convert_verify_gate \
  convert_verify_and_verify_pages_pass_for_hwp_source -- --nocapture
  1 passed / 0 failed

python3 -m unittest scripts/tests/test_workflow_contract_wiring.py
  3 passed / 0 failed

cargo fmt --all -- --check
git diff --check
  통과
```

실제 `output/4586/green/export/conv.hwpx`를 T12 검사로 판정한 결과도 두 검사 모두 `ok:true`였다.
로컬 원문은 `output/4586/green/README.md`에 기록했다.

## 5. Stage 3 잔여

- T12 기준 답안·스코어카드 보정
- 기준 실행 version/commit/capabilities digest 기록
- 생성기 `convert` 표본의 `.hwpx` 오류 수정과 생성 교본 재생성
- 판단 트리·CLI 매뉴얼·gym README 정합

Stage 2에서는 코드·task 계약만 고쳤으며 생성 문서와 기준선은 다음 승인 단계로 분리했다.
