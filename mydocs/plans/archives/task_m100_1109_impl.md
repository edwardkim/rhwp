# Task M100-1109 — GitHub Actions runner 디스크 부족 정정 (구현 계획서)

- 이슈: [#1109](https://github.com/edwardkim/rhwp/issues/1109)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1109`
- 일시: 2026-05-24
- 수행 계획서: [`task_m100_1109.md`](task_m100_1109.md)

## 1. 정정 영역 (사전 분석 완료)

### 1.1 `.github/workflows/ci.yml`

현재 `build-and-test` job 의 step 흐름:

```
1. uses: actions/checkout@v5
2. Install Rust toolchain
3. Cache cargo registry & build  ← 누적 caches
4. Format check
5. Build
6. Canvas layer parity tests
7. Check WASM target
8. Install native Skia runtime packages  ← apt-get install
9. Native Skia tests
10. Run tests  ← 본 실패 영역
11. Clippy
```

→ **checkout 직후 (step 2 직전) `Free disk space` step 추가**:

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

`wasm-build` job 도 동일 패턴 (선제적):
- 현재 trigger 는 `workflow_dispatch || tag` 만 → 본 실패 영역 아니지만 안전 + 동일성 위해 적용

## 2. 단계 구성 (2 단계)

### Stage 1 — ci.yml 정정

**파일**: `.github/workflows/ci.yml`

**변경**:
- `build-and-test` job 의 checkout 직후 `Free disk space` step 추가
- `wasm-build` job 의 checkout 직후 동일 step 추가
- `df -h` 출력으로 회수 효과 가시화 (CI log 확인용)

**검증** (로컬에서 가능한 범위):
- YAML syntax 확인 (`yamllint .github/workflows/ci.yml`)
- ci.yml 구조 정합 (들여쓰기, step 순서)

### Stage 2 — push + CI 실측 + 최종 보고서

**검증 방법**:
- `local/task1109` 브랜치 직접 push 가능 — main/devel PR 전 CI 실행 관찰
- 또는 다른 PR 의 새 commit 으로 trigger
- CI log 의 `df -h` 출력 → before/after 디스크 사용량 확인

**최종 보고서**: `mydocs/report/task_m100_1109_report.md`
- ci.yml 변경 내용 + 회수 효과 (df -h 결과)
- 본 실패 영역 (cargo test) 정상 통과 확인

**트러블슈팅**: `mydocs/troubleshootings/github_actions_runner_disk_space.md`
- 재발 시 사전 검색 자료
- 본 task 의 본질 분석 + 해결 패턴

## 3. 위험 분석

| 위험 | 영향 | 완화 |
|------|------|------|
| 제거된 toolchain (dotnet/android/ghc) 가 다른 step 에서 필요 | CI 실패 | 본 프로젝트 Rust 전용 — 미사용 확인 |
| docker prune 가 cache 영향 | cache miss → 빌드 시간 증가 | docker image prune 만 — actions/cache 의 cargo cache 와 무관 |
| YAML 들여쓰기 오류 | CI workflow 전체 invalid | 로컬 syntax 확인 + small step 추가만 |
| `df -h` 출력으로 log 크기 증가 | 약간 — 무시 가능 | 단일 step log line ~10 |

## 4. 적용 후 효과 측정

- `df -h` before: 14GB 한도, 사용량 ~12GB (예상)
- `df -h` after: 사용량 ~3GB 회수 (~9GB 가용 추가)
- `Run tests` step 정상 통과
- 디스크 부족 오류 미발생

## 5. 산출물

- `.github/workflows/ci.yml` (Free disk space step 추가)
- `mydocs/working/task_m100_1109_stage1.md` (Stage 1 보고서)
- `mydocs/report/task_m100_1109_report.md` (최종)
- `mydocs/troubleshootings/github_actions_runner_disk_space.md` (트러블슈팅)

## 6. 작업지시자 승인 요청

1. 본 구현 계획 (2 단계) 승인 여부
2. Stage 1 의 step 추가 위치 (checkout 직후) 권장 수용 여부
3. Stage 2 의 push 검증 방법 (local/task1109 직접 push) 권장 수용 여부
