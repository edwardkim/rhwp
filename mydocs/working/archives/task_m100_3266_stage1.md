# 단계 완료보고서 — #3266 Stage 1 CI archive·Native Skia 병렬화

## 결과

`Build test archive`를 `Native Skia tests`의 후속 job에서 분리해 같은
`preflight → lint → frontend-package-gates` 성공 뒤에 병렬로 시작하게 했다.
기본 테스트 shard는 archive와 Native Skia가 모두 성공해야 실행한다.

```text
preflight → lint → frontend-package-gates
                    ├─ Native Skia tests ─┐
                    └─ Build test archive ┴→ Default-feature shards (8) → Build & Test
```

Native Skia 실패 시 archive가 진행 중이거나 완료될 수 있는 것은 의도된 trade-off다. 그러나
shard는 `needs['native-skia-tests'].result == 'success'`와
`needs['build-test-archive'].result == 'success'`를 함께 요구하므로 시작하지 않는다.

## 변경 범위

| 파일 | 변경 |
| --- | --- |
| `.github/workflows/ci.yml` | archive의 공통 gate 의존성·조건, shard의 dual-success 의존성·조건, 이미 압축된 `tests.tar.zst`의 `compression-level: 0` |
| `mydocs/plans/task_m100_3266.md` | 원인, 안전 조건, 구현과 검증 계획 |
| 본 문서 | 구현 결과와 검증 근거 |

변경하지 않은 항목:

- `Build & Test` required check와 worker 결과 집계
- shard 8개 matrix, `hash:i/8` 분할, shard 합계 검증
- artifact 이름, 1일 retention, 다운로드·실행 명령
- fast-pass와 frontend 비대상(`skipped`) 경로
- Rust·TypeScript 제품 코드 및 테스트 자체

`compression-level: 0`은 이미 Zstandard로 압축된 파일을 upload-artifact ZIP wrapper가 다시 압축하지
않게 한다. artifact의 내용·전송 대상 수는 바꾸지 않으며, 다음 PR CI에서 upload 시간과 전체 경로에
미치는 효과를 측정한다.

## 검증

| 검증 | 결과 |
| --- | --- |
| `actionlint .github/workflows/ci.yml` | PASS |
| Ruby `YAML.load_file('.github/workflows/ci.yml')` | PASS (`yaml ok`) |
| `git diff --check` | PASS |
| workflow DAG 계약 Ruby 검사 | PASS — archive/native 공통 조건, shard dual-success, 8개 matrix, `compression-level: 0`, aggregate dependency 확인 |

제품 테스트 명령은 변경하지 않았으므로 이 단계에서 cargo 전체 회귀는 실행하지 않았다. 이 PR의
GitHub Actions에서는 Native Skia·archive 병렬 시작, 8개 shard, `Build & Test`, CodeQL, Render Diff의
gate 계약만 확인한다. 시간 단축의 전후 비교는 이 변경 merge 뒤 **다음 PR**에서 수행한다.

## 후속 측정 기준

- 기준: run [30081855067](https://github.com/edwardkim/rhwp/actions/runs/30081855067), 전체 26분 48초
- 최근 기준: [#3265 update-branch run 30094656772](https://github.com/edwardkim/rhwp/actions/runs/30094656772), Native Skia 4분 8초, archive 7분 56초, 최장 shard 3분 4초
- 다음 PR에서 기록할 값: Native·archive 시작/종료 시각, artifact upload/download, shard별 nextest, 전체 wall-clock
