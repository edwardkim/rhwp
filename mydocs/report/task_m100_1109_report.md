# Task M100-1109 — GitHub Actions runner 디스크 부족 정정 (최종 보고서)

- 이슈: [#1109](https://github.com/edwardkim/rhwp/issues/1109) (CLOSED)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1109`
- 일시: 2026-05-24

## 1. 증상 + 본질

### 1.1 증상

```
##[error]No space left on device : '/home/runner/actions-runner/cached/2.334.0/_diag/pages/...log'
```

- 실패 step: `cargo test --verbose` (build-and-test job)
- 실패 run: 26359599960 (2026-05-24 11:13)
- 영향 PR: #1085 (Task #1042)

### 1.2 본질 분석

- 28 examples 모두 컴파일 + 모든 integration test binary 생성
- actions/cache@v4 의 target/ cache 누적
- ubuntu-latest runner ~14GB 한도 초과 (cache + 빌드 + apt + diag log)

자세한 본질 분석: [`mydocs/troubleshootings/github_actions_runner_disk_space.md`](../troubleshootings/github_actions_runner_disk_space.md)

## 2. 다층 안전망 해결

### 2.1 메인테이너 레포 설정 변경 (2026-05-24)

작업지시자 보고:
- Actions cache 한도: **10GB → 15GB**
- Cache 지속 일: **7일 → 3일**

본 조치 후 장애 CI 재시작 → 통과 확인.

### 2.2 workflow Free disk space step (본 task)

`.github/workflows/ci.yml` 의 `build-and-test` + `wasm-build` 두 job 의 checkout 직후
미사용 사전 설치 toolchain 제거 step 추가 (25 lines):

```yaml
- name: Free disk space (remove unused pre-installed toolchains)
  run: |
    sudo apt-get clean
    sudo rm -rf /usr/share/dotnet
    sudo rm -rf /usr/local/lib/android
    sudo rm -rf /opt/ghc
    sudo rm -rf /opt/hostedtoolcache/CodeQL
    sudo docker image prune --all --force || true
    df -h
```

### 2.3 회수 효과 (예상)

| Toolchain | 크기 (~) |
|-----------|---------|
| `/usr/share/dotnet` | 2GB |
| `/usr/local/lib/android` | 14GB |
| `/opt/ghc` | 5GB |
| `/opt/hostedtoolcache/CodeQL` | 3GB |
| **합계** | **~24GB** |

본 프로젝트 Rust 전용 → 위 모두 미사용.

## 3. 정정 영역

### 3.1 `.github/workflows/ci.yml`

- `build-and-test` job: checkout 직후 `Free disk space` step (12 lines)
- `wasm-build` job: checkout 직후 동일 step (13 lines)
- 총 25 insertions

### 3.2 보존된 산출물

- `mydocs/plans/task_m100_1109.md` (수행 계획서)
- `mydocs/plans/task_m100_1109_impl.md` (구현 계획서)
- `mydocs/working/task_m100_1109_stage1.md` (Stage 1 보고서)
- `mydocs/report/task_m100_1109_report.md` (본 최종 보고서)
- `mydocs/troubleshootings/github_actions_runner_disk_space.md` (트러블슈팅)

## 4. 검증

### 4.1 로컬 영역

- YAML syntax 확인: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` → OK

### 4.2 CI 실측

- 메인테이너 조치 후 본 실패 CI 재시작 → 통과 (workflow 정정 전)
- 본 task 의 workflow 정정 효과는 push 후 후속 CI run 에서 `df -h` 출력으로 확인 가능

## 5. 메모리 룰 정합

- ✅ `feedback_diagnosis_layer_attribution` — 다중 누적 영역 본질 정확 식별
- ✅ `feedback_check_open_prs_first` — open PR 확인 완료
- ✅ `feedback_search_troubleshootings_first` — 사전 검색 + 본 task 결과 트러블슈팅 등록
- ✅ `feedback_assign_issue_before_work` — assignee 본인 등록 완료
- ✅ `feedback_process_must_follow` — 수행 → 구현 → Stage → 보고서 절차 준수

## 6. 학습 + 후속

### 학습

- CI 디스크 부족의 다중 영역 (runner toolchain + cache + 빌드 산출물 + diag log)
- 28 examples 누적이 cargo test 빌드 시간/디스크 영향 — 새 example 추가 시 일회성/지속성 판단
- WASM job 의 영역 분리 확인 (workflow_dispatch || tag) — CI 실패 본질 식별 시 trigger 조건 확인 필수
- 다층 안전망 (메인테이너 외부 조치 + workflow 정정) — 단일 해결 의존 회피

### 후속

- `examples/` 분류/정리 별도 task 후보 (28 → 핵심 + oneshot 분리)
- Cargo workspace 의 `[workspace.dev-dependencies]` 영역으로 examples crate 분리도 고려
