# Task M100-1109 Stage 1 — ci.yml 정정 (Free disk space step 추가)

- 이슈: [#1109](https://github.com/edwardkim/rhwp/issues/1109)
- 단계: Stage 1
- 브랜치: `local/task1109`
- 일시: 2026-05-24

## 1. 메인테이너 처리 확인 + 다층 안전망 결정

작업지시자 보고 (2026-05-24): 메인테이너가 레포지터리 설정 변경:
- Actions cache 한도: **10GB → 15GB**
- Cache 지속 일: **7일 → 3일**
- 장애 CI 재시작 → 통과

→ 본 task 의 본질 해결 완료 (workflow 외부 영역). 본 Stage 1 의 workflow 정정은
**다층 안전망** 으로 진행 (작업지시자 결정 — Workflow 정정도 진행).

## 2. 정정 영역

### 2.1 `.github/workflows/ci.yml` (25 insertions)

`build-and-test` + `wasm-build` 두 job 모두 checkout 직후 `Free disk space` step 추가:

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

본 step 의 회수 효과 (예상):
| Toolchain | 크기 (~) |
|-----------|---------|
| `/usr/share/dotnet` | 2GB |
| `/usr/local/lib/android` | 14GB |
| `/opt/ghc` | 5GB |
| `/opt/hostedtoolcache/CodeQL` | 3GB |
| **합계** | **~24GB** |

본 프로젝트는 Rust 전용 — 위 모두 미사용.

### 2.2 위치 결정 근거

`checkout 직후, Install Rust toolchain 직전` 에 위치:
- checkout 의 `actions-runner/cached/` 영역 보호 (실패 발생 영역)
- Rust toolchain 설치 전에 디스크 정리 → 후속 step 들이 여유 공간에서 동작
- `df -h` 출력으로 CI log 에 회수 효과 가시화

## 3. 검증

### 3.1 YAML syntax 확인

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"
# (no output → syntax OK)
```

### 3.2 로컬 영역 검증 한계

CI workflow 의 실제 효과는 push 후 actions 실행 관찰 필요 — Stage 2 에서 검증.

## 4. 메모리 룰 정합

- `feedback_diagnosis_layer_attribution` — 실패 step (cargo test) + 누적 영역 (cache + toolchain) 본질 정확 식별
- `feedback_check_open_prs_first` — open PR 확인 완료
- `feedback_search_troubleshootings_first` — 사전 검색 완료

## 5. 작업지시자 승인 요청

Stage 1 정정 (ci.yml 25 line 추가) 승인 → Stage 2 (push + 실측 검증 + 최종 보고서 +
트러블슈팅) 진행 여부.
