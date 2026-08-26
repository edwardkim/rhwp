# PR 5707 CI status

Source: `gh pr view 5707 --repo edwardkim/rhwp --json statusCheckRollup,commits`
Fetched: 2026-08-19T23:22Z (UTC)

HEAD: `d3da49359235b33c64ba8b5ea4ced406fd9f347c`
(`test(agent): 스킬마다 계약 게이트 3회, 생성 시 필수 (#5706)`)

## Commits

| SHA | Headline |
|-----|----------|
| `528625b5fa784abc9e3d332716686c10ac3be40a` | feat(agent): 스킬 라우터와 rhwp 실렌더 검증 (#5706) |
| `7531bcf710b973ab73b8b31f930098a37f8d7ec4` | fix(agent): rhwp-skill-router 스킬에 실행 가능한 rhwp 명령을 넣는다 (#5706) |
| `d3da49359235b33c64ba8b5ea4ced406fd9f347c` | test(agent): 스킬마다 계약 게이트 3회, 생성 시 필수 (#5706) |

## statusCheckRollup (HEAD `d3da493`)

| Check | Conclusion / status |
|-------|---------------------|
| cancel-stale-runs | SKIPPED |
| cancel-stale-runs | SUCCESS |
| adapter inter-diff preflight | SUCCESS |
| CI preflight | SUCCESS |
| CodeQL preflight | SUCCESS |
| Proptest preflight | SUCCESS |
| adapter inter-diff | IN_PROGRESS (no conclusion) |
| Analyze (javascript-typescript) | IN_PROGRESS (no conclusion) |
| Analyze (python) | IN_PROGRESS (no conclusion) |
| Analyze (rust) | IN_PROGRESS (no conclusion) |
| prop roundtrip | IN_PROGRESS (no conclusion) |
| WASM Build | SKIPPED |
| build-test-archive / Build test archive | IN_PROGRESS (no conclusion) |
| Lint (fmt, clippy, WASM check) | IN_PROGRESS (no conclusion) |
| Native Skia tests | IN_PROGRESS (no conclusion) |
| Frontend unit gates | SKIPPED |
| Frontend package gates | IN_PROGRESS (no conclusion) |

HEAD has not reached the regular-shard jobs. `test-regular-shard-3` and `Build & Test` are absent from this rollup.

## Highlight: shard 3 and Build & Test still failing on old SHA

Both failures are on the **first** PR commit, not HEAD.

| Check | Conclusion | SHA | Job |
|-------|------------|-----|-----|
| **test-regular-shard-3 / Default-feature tests (shard 3/3)** | **FAILURE** | `528625b5fa784abc9e3d332716686c10ac3be40a` | https://github.com/edwardkim/rhwp/actions/runs/32309190729/job/96251123547 |
| **Build & Test** | **FAILURE** | `528625b5fa784abc9e3d332716686c10ac3be40a` | https://github.com/edwardkim/rhwp/actions/runs/32309190729/job/96251950171 |

Sibling shards on the same old SHA: shard 1 SUCCESS, shard 2 SUCCESS, slow shard SUCCESS.

Middle SHA `7531bcf` cancelled `build-test-archive` / Native Skia before shards started.

**Yes: shard 3 is still failing on an old SHA (`528625b`). HEAD has not re-run it yet.**
