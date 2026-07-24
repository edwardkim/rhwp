# PR #3267 검토 기록 — #3266 Native Skia·테스트 archive 병렬화

## 메타

| 항목 | 값 |
|---|---|
| PR | [#3267](https://github.com/edwardkim/rhwp/pull/3267) |
| 작성자 | `jangster77` (repository collaborator) |
| base | `devel` |
| 관련 이슈 | [#3266](https://github.com/edwardkim/rhwp/issues/3266) |
| 범위 | GitHub Actions CI job DAG, test archive upload 설정, 계획·검증 기록 |
| 문서 작성 시점 참고 | Draft PR. mergeable·CI·head SHA는 merge 전에 최신 상태를 다시 확인한다. |

## 변경과 판단

### 성공 경로 단축

`Build test archive`의 선행 조건을 `Native Skia tests`에서 분리했다. 두 job은 이제 같은
`preflight → lint → frontend-package-gates` 성공 뒤 별도 GitHub-hosted runner에서 병렬로 시작한다.
각 job의 workspace·Cargo target·`rust-cache` shared key는 각각 분리되어 있으므로 파일 lock이나 PR cache
writer 충돌은 없다. PR run은 두 cache 모두 restore-only다.

기준 run [30081855067](https://github.com/edwardkim/rhwp/actions/runs/30081855067)에서는 Native Skia
7분 27초 뒤 archive 7분 56초가 시작해 전체 26분 48초가 걸렸다. 이 변경은 성공 경로에서 두 구간을
겹쳐 약 4–8분의 critical path 단축을 목표로 한다. 작업지시자 지시에 따라 실제 절감값은 이 PR이
merge된 뒤 **다음 PR**에서의 job 시작·완료 시각으로 판정한다.

### 실패 전파 보존

`test-shard`는 `native-skia-tests`와 `build-test-archive`를 모두 `needs`로 가지고, 두 결과가
`success`일 때만 실행한다. 따라서 Native Skia가 실패하면 archive가 이미 실행 중이거나 완료돼도
8개 default-feature shard는 시작하지 않는다. lint 실패, frontend 대상 gate 실패, archive 실패도
기존처럼 shard를 막는다. frontend 비대상 job의 정상 `skipped`와 fast-pass 경로는 두 병렬 worker에
동일하게 적용한다.

### artifact upload

`tests.tar.zst`는 cargo-nextest가 이미 Zstandard 압축한 파일이므로 `upload-artifact`에
`compression-level: 0`을 지정했다. ZIP wrapper의 재압축 CPU를 제거하지만 artifact 내용, 이름,
1일 retention, 다운로드 방식 및 shard 수는 바꾸지 않는다.

## 사전 검증

| 검증 | 결과 |
|---|---|
| `actionlint .github/workflows/ci.yml` | PASS |
| Ruby `YAML.load_file('.github/workflows/ci.yml')` | PASS (`yaml ok`) |
| DAG 계약 검사 | PASS — archive/native 공통 gate, shard dual-success, 8개 matrix, aggregate 의존성, `compression-level: 0` |
| `git diff --check` | PASS |

Rust·TypeScript 제품 코드와 실행할 테스트 명령은 바꾸지 않았다. 따라서 로컬 cargo 전체 회귀를 중복 실행하지
않고, 최신 PR CI에서 Native Skia, test archive, 8개 default-feature shard, `Build & Test`, CodeQL,
Render Diff의 gate 동작을 최종 확인한다. 시간 단축 측정은 merge 뒤 다음 PR에서 수행한다.

## 범위 외와 리스크

- artifact를 8개 shard에 전송하는 구조는 유지한다. #3265에서 느린 shard 번호가 매번 달랐으므로,
  4-shard 또는 timing-aware 분할은 이번 PR에 추정으로 섞지 않고 #3266의 후속 측정으로 결정한다.
- Native Skia 실패 시 archive runner 시간이 일부 낭비될 수 있다. 이는 성공 경로를 줄이는 대신이며,
  shard 실행 차단과 `Build & Test` 실패 보고는 유지된다.
- renderer·WASM 출력·샘플·golden은 바꾸지 않아 visual sweep 대상이 아니다.

## 최종 권고

최신 PR head의 GitHub Actions가 통과하고, Native Skia와 archive의 병렬 시작 및 shard dual-success
차단 계약을 확인한 뒤 작업지시자 승인으로 ready 전환·merge를 판단한다. #3266의 시간 단축 완료 판정은
다음 PR의 실측까지 보류한다.
