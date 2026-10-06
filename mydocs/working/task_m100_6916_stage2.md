# #6916 Stage 2 — 선택적 Gym 안내 분리 구현·focused 검증

- Issue: [#6916](https://github.com/edwardkim/rhwp/issues/6916)
- 날짜: 2026-09-09
- 승인: A안과 Stage 2 구현 승인 후 수행
- source branch: `task_m100_6916`
- 검증 code head: `369e789ee` (본체 구현 `938be665b` + 테스트 보정)
- 상태: Stage 2 완료, Stage 3 독립성·전체 검증 승인 대기. remote push·PR 없음.

## 1. 구현 결과

`rhwp://docs/gym` URI를 유지하고 `include_str!`의 입력을 Gym README에서
제품 소유 [선택적 안내](../manual/gym_optional_tool.md)로 바꿨다.
name/title/MIME와 기존 resources handler는 그대로이며 description·본문·size만 바뀐다.
네트워크 다운로드, Gym 경로 fallback, feature flag, 새 의존 crate는 추가하지 않았다.

- 내장 문서 원본 크기: Gym README 22,509 bytes → 제품 안내 1,250 bytes.
  바이너리 크기·성능 차이를 계측한 값은 아니다.
- Oracle advisory의 sparse checkout과 mirror test를 함께 교체했다.
  trigger·job·권한·timeout·verdict·승격 정책은 변경하지 않았다.
- Gym 운영 매뉴얼, Gym 에이전트 지침, 문서 지도와 과거 sparse 우회 처방에 새 경계를 반영했다.
- 일반 CLI/API, parser/renderer, Gym task/reference/check/score와 package manifest는 변경하지 않았다.

## 2. 회귀 테스트와 수정 전/후 증거

신규 원본은 `tests/cases/issue_6916_gym_optional_resource.rs`다.
nextest 런타임의 `CARGO_BIN_EXE_rhwp`를 우선하고 컴파일타임 경로를 fallback으로 사용한다.
저장소 밖 임시 working directory에서 파일 기반 stdin/stdout/stderr로 MCP를 실행한다.
파이프 버퍼 교착을 피하고, 20초 timeout과 child/임시 경로 정리 guard를 둔다.

검사 내용: initialize, URI의 유일한 목록 노출, name/title/MIME, 읽기 봉투, 선택적 안내의 내용·링크,
UTF-8 본문 크기와 size 일치, 미지 URI의 `-32002` 유지.

| 비교 | 입력·결과 |
| --- | --- |
| 수정 전 | `85fb3be0d`의 제품 코드 + 최종 테스트 원본 blob `d2fcda9500bf425e09207e9026e784984c5da304`. 선택적 안내 대신 전체 Gym README를 반환한다는 assertion에서 1건 실패, exit 100 |
| 수정 후 | exact code head `369e789ee`, 동일 테스트 원본. 1건 통과, exit 0 |

최초 테스트 작성 때 프로젝트에 없는 `tempfile`을 사용해 컴파일 exit 101이 발생했다.
이것은 제품 회귀 재현으로 세지 않았다. 의존성을 추가하지 않고 `std::fs::create_dir`로
새 고유 경로를 확보하는 guard로 고쳤다. 이미 존재하는 경로는 재사용·삭제하지 않는다.
위 red/green은 이 보정 이후 **동일 테스트 원본**으로 얻은 결과다.

검증 worktree를 후보로 전환할 때 보정된 테스트가 미커밋 상태라 Git이 전환을 거절했다.
후보 원본과 동일함을 확인한 뒤 검증 공간에서만 baseline 원본으로 되돌리고 exact 후보로 전환했다.
주 checkout의 변경을 되돌리거나 stash·고아 브랜치를 만들지 않았다.

## 3. 검증 명령·결과

review 환경에서 `node scripts/rust-test-suite-manifest.mjs --prepare` 후 각 case를
`node scripts/run-rust-test.mjs <case> -- --cargo-profile release-test --target-dir <shared-review-target>`로
순차 실행했다. source checkout에는 파생 suite를 생성·등록하지 않았다.

| 검증 | 결과 |
| --- | --- |
| 신규 `issue_6916_gym_optional_resource` | 1 passed |
| `mcp_resources_contract` | 3 passed; 광고된 모든 URI의 실제 읽기 포함 |
| `mcp_server_contract` | 26 passed |
| `mcp_spec_ledger_contract` | 4 passed |
| review `cargo fmt --all -- --check` | exit 0 |
| review manifest `--check` | exit 0 |
| `python3 -m unittest scripts.tests.test_oracle_public_advisory_workflow scripts.tests.test_gym_benchmark_validation scripts.tests.test_agent_frame scripts.tests.test_agent_org` | 24 passed |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 44 passed |
| `python3 -m unittest discover -s scripts/tests -p 'test_gym_*.py'` | 3,171건 중 3,170 passed, 1 skipped, 오류·실패 0 |
| `python3 gym/tools/audit.py --json` | ok=true, issueCount=0, toolFailed=false |
| `python3 gym/tools/oracle_probe.py --json` 및 `--selftest --json` | 각각 ok=true, issueCount=0 |
| `go run github.com/rhysd/actionlint/cmd/actionlint@v1.7.7 -shellcheck= -pyflakes= .github/workflows/oracle-public-advisory.yml` | exit 0; YAML/action 식 검사, 외부 shellcheck/pyflakes 미실행 |
| 변경 Markdown 5개 링크 검사 | 내부 상대 링크 이상 없음 |
| 변경 장기 문서 4개 `check_document_metadata.validate_file` | 오류 0 |
| `git diff --check c72ad805cc60e4a5cf5689c18b44e7214cec68fe...HEAD` | exit 0 |

Gym skip은 `test_gym_release_diff.CommittedReportTests.test_self_diff_report_is_stable`의
“커밋된 self-diff 리포트 없음”이다. 해당 모듈과 leaderboard의 verbose 재확인으로 사유를 식별했다.
nextest의 나머지 skipped는 focused filter로 선택하지 않은 테스트이며 전체 회귀 성공을 뜻하지 않는다.

현재 nextest 0.9.137은 저장소 권장 0.9.140보다 낮아 버전 경고와
`ci-duration-observation.junit.report-skipped` 설정 무시 경고를 냈다.
이번 실행은 default profile이고 테스트 성공/실패를 실제 확인했다. 도구 버전·설정을 임의 변경하지 않았다.

전체 메타데이터 검사에서는 기존 문서 4개의 필드 누락 16건이 나왔다. 대상은
`benchmark_vs_alternatives.md`, issue-4964 README, issue-5511 README 및 CLI inventory 문서다.
네 파일은 기준 `c72ad805...`와 동일함을 Git diff로 확인했다. 이번 변경 문서와 구분하여 수정하지 않았다.

## 4. 증적과 검증 공간

- 로컬 로그: `output/6916/stage2/`의 `red.log`, `green.log`, `mcp_*_contract.log`,
  `boundary-contracts.log`, `ci-impact.log`, `gym-contracts.log`, `gym-skip-detail.log`,
  `gym-audit.json`, `gym-oracle.json`, `gym-oracle-selftest.json`, `actionlint.log`, prepare/check 로그.
- 검증 worktree: `/home/edward/mygithub/rhwp-6916-review`, detached `369e789ee`.
- 공유 캐시: `/home/edward/mygithub/rhwp-shared-review-target`. 기존 캐시를 삭제·이동하지 않았다.
- review tracked 파일은 candidate와 동일하고 파생물은 검증 전용이다.
  Stage 3에서 이어 사용할 검증 공간 1개를 유지하며, 타스크 종료 후 정리한다.

## 5. 남은 검증과 다음 승인

이번 테스트는 Gym 없는 **working directory에서 이미 빌드된 MCP가 동작함**을 확인한다.
Gym 없는 **source tree에서의 제품 빌드·포장**까지 증명한 것은 아니다.

Stage 3에서 남은 항목은 Gym 제외 격리 source의 Linux release build·설치본 실행·포장,
Docker WASM·npm 포함물 검사, 외부 바이너리 지정 Gym core-cli canary,
PR 전 Rust 전체 lint 묶음과 release-test 전체 회귀다.
기존 Studio pkg·개발 서버·Gym 제출물은 덮어쓰지 않는다.
원격 적용과 Oracle exact-head run의 결과 확인도 아직 남아 있다.
