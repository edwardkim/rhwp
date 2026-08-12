# #4586 구현계획서 — gym T12 HWPX 형식·판정 계약 보정

- **Issue**: [#4586](https://github.com/edwardkim/rhwp/issues/4586)
- **수행계획서**: `mydocs/plans/task_m100_4586.md`
- **브랜치**: `task/4586-gym-t12-hwpx`
- **기준 커밋**: `d30e5d4af`

## 1. CLI 출력 형식 가드

### `src/main.rs`

`convert_hwp`가 위치 인자 파싱을 끝낸 직후, 입력 파일을 읽기 전에 출력 경로의 확장자를 검사한다.

- 허용: `.hwp`, 대소문자 무시
- 거부: `.hwpx`, 확장자 없음, 그 밖의 모든 확장자
- 결과: `EXIT_USAGE`(2), stdout 0바이트, 출력 파일 미생성
- stderr: `convert`가 HWP5 출력 명령임을 밝히고 HWPX 변환은 `rhwp export-hwpx`를 사용하라고 안내

입력과 출력의 형식 책임을 파일명으로 추정해 변환 분기를 바꾸지 않는다. `convert`는 계속 HWP5만,
`export-hwpx`는 계속 HWPX만 담당한다. `batch convert`는 기존처럼 `.hwp` 이름을 생성하므로 동작을
바꾸지 않는다.

### `tests/issue_4586_gym_t12_contract.rs`(신설)

- 공개 fixture `samples/field-01.hwp`로 `.hwpx` 출력 거부를 검증한다.
- 종료 코드 2, stdout 0바이트, 안내 문자열, 산출물 없음까지 한 계약으로 고정한다.
- `.HWP` 출력은 성공해 대소문자 호환을 보존한다.
- 실행 파일은 nextest archive 호환 `rhwp_bin()` 패턴을 사용한다.

## 2. gym 채점 계약

### `gym/score.py`

검사별 허용 종료 코드를 다음 우선순위로 정규화한다.

1. `expect_exits`가 있으면 정수 배열을 허용 집합으로 사용
2. 없으면 기존 `expect_exit` 단일 값을 사용
3. 둘 다 없으면 기존 기본값 `0`

실제 종료 코드가 허용 집합에 포함되면 exit 3이어도 JSON 봉투 파싱과 `answer_eq`/`value_eq` 비교를
계속한다. 허용되지 않은 코드는 기존처럼 오류 세부를 남긴다. 기존 task JSON은 수정 없이 같은 판정을
유지한다.

### `gym/tasks/T12.json`

- 힌트: `rhwp export-hwpx <입력> conv.hwpx --verify --json`
- 검사 1: `rhwp info conv.hwpx --json`의 `format`이 `hwpx`
- 검사 2: `ir-diff`의 `identical`과 answer를 비교하며 `expect_exits:[0,3]`

형식 검사를 먼저 두어 HWP5 위장 제출이 IR 동일성 비교와 무관하게 실패하게 한다.

### `scripts/tests/test_gym_score.py`(신설), `.github/workflows/ci.yml`

표준 `unittest`와 `unittest.mock`으로 다음을 검증한다.

- exit 3 + `identical:false` + answer false는 통과
- exit 3이 허용 목록에 없으면 실패
- 기존 `expect_exit:0` 검사는 이전과 동일하게 통과
- 복수 허용 코드 오류 메시지가 실제·허용 값을 구분해 남음

CI의 workflow contract 단계에 이 테스트를 명시적으로 배선해 테스트 파일만 존재하고 실행되지 않는
상태를 막는다.

## 3. 기준선과 문서

### gym 기준선

- `gym/baselines/claude-fable-5/T12/answer.json`: 실제 HWPX 비교 결과인 `false`
- `gym/baselines/claude-fable-5/scorecard.json`과 `report.md`: 수정된 T12 계약 결과로 보정
- 최종 보고서: 기준 실행에 사용한 rhwp version, 코드 commit, `capabilities` 원문 SHA-256과 재현 명령 기록

기준선의 바이너리 산출물은 `gym/.gitignore` 정책대로 커밋하지 않는다. 재현 가능한 명령과 판정 봉투를
`output/4586/`에 보존하고 공개 보고서에는 비식별 요약과 해시만 싣는다.

### 자기서술·사용자 문서

- `tools/gen_agent_codex.py`: `convert` 실측 표본 출력을 `conv.hwp`로 수정
- `mydocs/manual/agent_codex/40_변환과_렌더.md`: 생성기로 재생성, 수기 수정 금지
- `mydocs/manual/agent_codex/01_판단트리.md`: HWP→HWPX `export-hwpx`, HWPX/배포용→HWP5 `convert`로 분리
- `mydocs/manual/cli_commands.md`: 잘못된 출력 확장자 거부·exit 2·산출물 없음·대체 명령 명시
- `gym/README.md`: 복수 판정 종료 코드가 있는 검사 계약을 짧게 설명

## 4. 검증 명령

Stage별 focused 검증은 같은 checkout/target에서 순차 실행한다.

```bash
cargo test --test issue_4586_gym_t12_contract -- --nocapture
python3 -m unittest scripts/tests/test_gym_score.py
cargo test --test output_axis_json_contract convert_json_envelope_with_verify -- --nocapture
python3 tools/gen_agent_codex.py --check
cargo test --test agent_codex_contract
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
git diff --check
```

Rust parser/model/CLI 변경이므로 PR 직전 전체 release-test 회귀는
`mydocs/manual/pr_review/local_validation.md` §4.3 대상이다. 다만 긴 전체 회귀와 PR CI는 focused 결과를
공유한 뒤 작업지시자의 별도 승인을 받아 실행한다.

## 5. 증적

`output/4586/`에 다음을 생성한다.

- `red/`: 현행 HWP5 위장 통과와 실제 HWPX 실패 봉투
- `green/`: `.hwpx` 출력 거부, 실제 HWPX 형식 확인, exit 3의 false 정답 통과 봉투
- `metadata/`: `rhwp --version`, source commit, `rhwp capabilities` SHA-256

렌더링·레이아웃·WASM 변경이 없으므로 시각 증적과 WASM 빌드는 적용하지 않는다. 최종 보고서에 이 생략
근거를 명시한다.
