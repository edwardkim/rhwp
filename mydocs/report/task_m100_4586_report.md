# task_m100_4586 최종 보고서 — gym T12 HWPX 형식·판정 계약 보정

- **Issue**: [#4586](https://github.com/edwardkim/rhwp/issues/4586)
- **계획서**: [수행계획](../plans/task_m100_4586.md) · [구현계획](../plans/task_m100_4586_impl.md)
- **단계 기록**: [stage1](../working/task_m100_4586_stage1.md) · [stage2](../working/task_m100_4586_stage2.md) · [stage3](../working/task_m100_4586_stage3.md)
- **브랜치**: `task/4586-gym-t12-hwpx`
- **기준**: `upstream/devel` `d30e5d4af`
- **작성일**: 2026-08-11 KST

## 1. 결과

gym T12가 HWPX 변환 과제인데도 `convert ... conv.hwpx`를 안내하고, HWP5 바이트를 `.hwpx` 이름으로
저장한 제출도 IR 동일성만 맞으면 통과시키던 계약을 바로잡았다.

- `convert`는 HWP5 전용으로 유지하고 `.hwp` 출력만 대소문자 무시로 허용한다.
- HWPX 변환은 `export-hwpx`만 사용한다.
- T12는 먼저 산출물의 실제 형식이 `hwpx`인지 검사한다.
- `ir-diff --json`의 정직한 차이 종료 코드 3도 JSON 봉투를 판정할 수 있게 `expect_exits`를 추가했다.
- 실제 HWPX 변환의 `identical:false`를 T12 정답으로 고정했다.
- 생성 교본, 판단 트리, CLI 매뉴얼, gym README와 CI 실행 계약을 같은 규칙으로 정합화했다.

## 2. 원인과 해결

### 2.1 형식 위장

기존 `convert`는 출력 확장자를 검증하지 않고 항상 HWP5 바이트를 썼다. T12 힌트가 출력명을
`conv.hwpx`로 지정했기 때문에 파일명만 HWPX인 HWP5 문서가 만들어졌다. 채점은 IR 동일성만 비교하여
이 잘못된 산출물을 통과시켰다.

`convert_hwp`가 입력 파일을 읽기 전에 출력 확장자를 검사하도록 했다. `.hwp` 이외에는 exit 2,
빈 stdout, 출력 미생성으로 종료하고 `export-hwpx`를 안내한다. 명령의 책임을 파일명에 따라 자동
전환하지 않았다.

### 2.2 차이를 나타내는 정상 JSON 봉투

`ir-diff --json`은 차이가 있으면 exit 3과 함께 판정 가능한 JSON 봉투를 반환한다. 기존 scorer는
단일 `expect_exit`만 받아 exit 3 봉투를 값 비교 전에 실패 처리했다.

검사별 `expect_exits` 정수 배열을 추가하고, 허용된 종료 코드이면 JSON 파싱과 `answer_eq`·`value_eq`를
계속하도록 했다. 기존 `expect_exit`과 기본값 0은 그대로 유지했다.

### 2.3 전체 회귀에서 드러난 기존 픽스처 충돌

첫 전체 회귀는 5,763건 중 `cli_exit_codes::unreadable_input_reports_runtime_failure` 한 건이 실패했다.
이 테스트는 입력 파일 없음 계약을 검증하면서 `convert` 출력에 `.hwpx`를 사용해 새 출력 확장자 계약이
먼저 발동했다. 제품 코드를 완화하지 않고 다음처럼 테스트 전제를 분리했다.

- `convert`: 실제 확장자가 `.hwp`인 고유 임시 경로
- `export-hwpx`: 실제 확장자가 `.hwpx`인 고유 임시 경로

단일 재검증 1/1 통과 후 전체 회귀를 다시 실행해 5,763/5,763 통과를 확인했다.

## 3. 주요 변경

| 영역 | 변경 |
| --- | --- |
| `src/main.rs` | `convert`의 `.hwp` 출력 사전 검증과 `export-hwpx` 안내 |
| `gym/score.py` | 복수 허용 종료 코드 `expect_exits`와 exit 3 봉투 판정 |
| `gym/tasks/T12.json` | 실제 `export-hwpx`, 형식 검사, IR 차이 판정 |
| `scripts/tests/test_gym_score.py` | scorer 하위 호환·복수 종료 코드 계약 |
| `.github/workflows/ci.yml` | gym scorer 계약 테스트 배선 |
| `tests/issue_4586_gym_t12_contract.rs` | CLI 형식·종료 코드·산출물 계약 |
| `tests/cli_exit_codes.rs` | 입력 오류 테스트의 출력 형식 전제 분리 |
| `gym/baselines/claude-fable-5/T12/` | 실제 HWPX 기준 판정과 재현 메타데이터 |
| `tools/gen_agent_codex.py` 및 생성 문서 | HWP5/HWPX 명령 경계와 결정적 plan 해시 정합 |
| 사용자 문서 | 판단 트리, CLI 명령, gym 판정 규칙 정합 |

## 4. 기준 실행

T12 focused 실행은 공개 표본 `samples/field-01.hwp`를 실제 HWPX로 변환해 수행했다.

```text
T12 변환 자기검증: pass, 2/2 checks
- HWPX 형식 확인: expected hwpx / actual hwpx
- 변환물 IR 대조: expected false / actual false
```

| 항목 | 값 |
| --- | --- |
| rhwp version | `rhwp v0.8.2` |
| source commit | `1ac75c0aad1056195c34cac2cc036d2943c5f99d` |
| capabilities SHA-256 | `62aea3df8bc40dd679247c044093e41fc54d1d80396c2b4b5b445ec843ffe27c` |
| source SHA-256 | `518cb939079e6e0640a5f813597f744e2528a17ca52ee418929f1c8f4b5380c0` |
| HWPX SHA-256 | `41c07ba1a1f00b356b0ca6ccbef986747ae177cfee10f243a6905d82ad698617` |

로컬 원문 증적은 gitignore 대상인 `output/4586/`에 보존한다.

## 5. 검증

| 게이트 | 결과 |
| --- | --- |
| `issue_4586_gym_t12_contract` | 3 passed / 0 failed |
| `scripts/tests/test_gym_score.py` | 5 passed / 0 failed |
| 기존 convert output axis | 1 passed / 0 failed |
| #1638 convert verify 계약 | 1 passed / 0 failed |
| workflow contract wiring | 3 passed / 0 failed |
| agent Codex contract | 2 passed / 0 failed |
| T12 실제 focused 판정 | 2/2 checks passed |
| 생성 교본 `--check` | 명령 83 · 실측 18 · 계약만 65 · 변경 0 |
| `cargo clippy --all-targets -- -D warnings` | 통과, 경고 0 |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |
| release-test 전체 회귀 | **5,763 passed / 0 failed / 36 skipped / 6 slow**, 221.286초 |

전체 회귀는 `cargo-nextest 0.9.137`에서 수행했다. 저장소 권장 버전 `0.9.140`보다 낮다는 경고가
있었지만 실행과 결과 집계는 정상 완료됐다. `actionlint`는 로컬 환경에 설치되어 있지 않아 별도 실행하지
못했고, workflow 배선 회귀 3건과 GitHub CI에서 최종 확인한다.

렌더러·레이아웃·WASM·rhwp-studio 코드는 변경하지 않았다. 따라서 시각 검증과 Docker WASM 빌드는
이번 변경의 적용 게이트가 아니다.

## 6. 단계 커밋

| 커밋 | 내용 |
| --- | --- |
| `d25375877` | 구현 계획 확정 |
| `a4e8f8831` | HWPX 형식 계약 RED 고정 |
| `1ac75c0aa` | CLI·gym HWPX 판정 계약 구현 |
| `c428b4a04` | T12 기준선·생성 교본·문서 정합 |
| `db772d92f` | 전체 회귀에서 발견한 CLI 입력 오류 픽스처 보정 |

## 7. 다음 단계

로컬 구현과 필수 검증은 끝났다. 원격 push와 PR 생성은 작업지시자의 별도 승인 후 수행한다.
