# PR #3269 검토 기록 — #3268 Frontend package gates·Lint 병렬화

## 메타

| 항목 | 값 |
|---|---|
| PR | [#3269](https://github.com/edwardkim/rhwp/pull/3269) |
| 작성자 | `jangster77` (repository collaborator) |
| base | `devel` |
| 관련 이슈 | [#3268](https://github.com/edwardkim/rhwp/issues/3268) |
| 범위 | GitHub Actions CI job DAG 및 계획·검증 기록 |
| 문서 작성 시점 참고 | `d88d093` (Update branch merge), `CLEAN` / `MERGEABLE`; merge 직전에 최신 상태를 재확인한다. |

## 변경과 판단

`Frontend package gates`의 선행 조건을 `preflight`만 남겨, `Lint (fmt, clippy, WASM check)`와
동시에 시작하게 했다. Frontend는 Lint의 산출물·workspace·`target/`을 읽지 않는다. 두 job은 별도
`ubuntu-latest` hosted runner에서 실행되고 PR cache는 restore-only이므로, 이 병렬화로 Cargo file
lock이 생기지 않는다.

Native Skia tests와 Build test archive는 그대로 Lint와 Frontend 모두의 성공을 요구한다. 따라서
Lint 또는 Frontend 실패 때 이미 실행 중인 다른 gate의 runner 시간은 일부 소모될 수 있어도,
Native/archive 및 8개 default-feature shard는 시작하지 않는다. 사용자가 검토한 4-way 병렬화는
[#3064](https://github.com/edwardkim/rhwp/issues/3064)의 실패 전파 gate를 약화하므로 적용하지 않고,
현행 두 단계 병렬 구조를 유지한다.

Renderer·WASM 출력·샘플·golden을 변경하지 않았다. 시각 결과에 영향을 주는 변경이 아니므로 visual
sweep 대상은 아니다. Render Diff CI는 workflow 공통 gate로서 실행·통과를 확인한다.

## 사전 및 원격 검증

| 검증 | 결과 |
|---|---|
| `git diff --check` | PASS |
| `actionlint .github/workflows/ci.yml` | PASS |
| Ruby YAML·DAG 계약 검사 | PASS — Lint/Frontend은 preflight 뒤 독립, Native/archive의 dual-success gate 및 shard 의존성 보존 |
| 최신 CI [30100367780](https://github.com/edwardkim/rhwp/actions/runs/30100367780) | 모든 job SUCCESS — preflight, Lint, Frontend, Native Skia, archive, 8 shards, Build & Test |
| 최신 CodeQL [30100367764](https://github.com/edwardkim/rhwp/actions/runs/30100367764) | PASS — Rust, Python, JavaScript/TypeScript 분석 및 CodeQL gate |
| 최신 Render Diff [30100367949](https://github.com/edwardkim/rhwp/actions/runs/30100367949) | PASS — Canvas visual diff |

CI run 30100367780에서 preflight가 14:17:54 UTC에 끝난 뒤 Lint와 Frontend가 모두 14:17:57 UTC에
시작했다. Lint는 14:19:27 UTC, Frontend는 14:20:00 UTC에 성공했고, 그 뒤 Native Skia와 archive가
동시에 14:20:03 UTC에 시작했다. 이는 의도한 gate와 병렬 시작을 실제 run으로 확인한 결과다.

해당 CI run의 workflow-level 결론은 마지막에 `cancelled`로 표시됐지만, 위의 실제 CI job 전부와 PR의
required check rollup은 SUCCESS이며, PR은 문서 작성 시점에 `CLEAN` / `MERGEABLE`이다. 이 기록은
workflow-level 표기 대신 개별 required check와 merge 직전 재확인을 merge 판단 근거로 삼는다.

## 범위 외와 리스크

- fast-pass 대상은 넓히지 않았다. `.github/workflows/**` 변경은 계속 full CI 대상이다.
- CI 실패 시 실행 중인 작업을 선택적으로 중지하는 기능은 이번 범위에 포함하지 않았다. GitHub `needs`는
  후속 job 시작을 막지만 이미 시작한 독립 runner를 취소하지 않는다.
- shard별 시간 불균형의 재분할은 #3266의 후속 실측 범위이며, 이 PR은 분할 로직을 바꾸지 않는다.

## 최종 권고

최신 PR head의 required checks가 재확인되어 있고 작업지시자의 merge 승인이 있으므로, 문서 전용 후속
commit의 fast-pass check까지 확인한 뒤 squash merge를 권고한다. merge 후 #3268 자동 close, `devel`
동기화 및 작업 브랜치 정리를 수행한다.
