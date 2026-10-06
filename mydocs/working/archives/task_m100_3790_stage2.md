# task_m100_3790 Stage 2 1차 결과 — shadow 판정 실측

- **Issue**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **Stage 1 PR**: [#3792](https://github.com/edwardkim/rhwp/pull/3792)
- **Stage 2.5 PR**: [#3823](https://github.com/edwardkim/rhwp/pull/3823) (draft)
- **브랜치**: `codex/issue-3790-shadow-observation`
- **기준**: `upstream/devel` `91f5131815dc`
- **최신 동기화 기준**: `upstream/devel` `2971a1d9a6ca` (#3892 포함)
- **live 관찰 시작**: 2026-08-02 17:28:53 UTC, #3792 merge 직후
- **기록 시각**: 2026-08-03 KST
- **상태**: 1차 실측 및 Stage 2.5 구현 완료, draft PR 원격 검증 중, Stage 3 활성화 보류

## 1. 측정 방법과 해석 경계

live 표본은 #3792 merge 뒤 생성된 `pull_request` CI run을 대상으로 했다. `CI preflight` log의
`SHADOW_*` 환경값, 같은 PR의 실제 파일 목록, legacy CI worker 결과를 대조했다. Job Summary 본문은
별도 API가 없어도 완료된 preflight log의 summary step 환경값에서 판정 결과를 회수할 수 있었다.

historical replay는 측정 시점의 최근 종료 PR 60개를 `gh pr list --state closed --limit 60`으로 가져와
merge된 classifier version 1에 다시 입력했다. PR `headRefOid`와 Actions `headSha`가 같은 최신 run을
연결하고, `gh run view --json jobs`의 `startedAt`부터 `completedAt`까지를 runner 사용 시간으로 합산했다.

수집 명령의 핵심 표면은 다음과 같다.

```bash
gh pr list --repo edwardkim/rhwp --state closed --limit 60 \
  --json number,title,headRefOid,changedFiles,files,url
gh run list --repo edwardkim/rhwp --workflow ci.yml --event pull_request --limit 400 \
  --json databaseId,headSha,status,conclusion,createdAt,updatedAt,url
gh run view RUN_ID --repo edwardkim/rhwp --json jobs
```

GraphQL `files`가 100개만 반환한 PR은 부분 목록으로 분류하지 않고 수집기에서 `full`로 닫았다. 아래의
`graphql-file-list-truncated`는 이 replay 수집기의 안전 경계이며, 실제 workflow의 REST pagination 및
PR 3,000개 경계 reason과는 구분한다.

runner-minute는 실제 실행된 job 시간을 더한 값이다. 병렬 job 합계이므로 wall time 절감과 같지 않으며,
아직 구현하지 않은 frontend `unit` gate 자체의 시간도 추정하지 않았다.

## 2. merge 이후 live shadow

관찰 시점에 merge 이후 생성된 live CI는 고유 PR 4건이다. 새 push로 취소된 #3819의 직전 run 1건과
#3740의 이전 run 2건은 중복 표본에서 제외하고 각 PR의 최신 완료 run을 사용했다.

| PR | CI run | 실제 파일 축 | shadow 결과 | legacy 결과 |
| --- | --- | --- | --- | --- |
| [#3749](https://github.com/edwardkim/rhwp/pull/3749) | [30759050941](https://github.com/edwardkim/rhwp/actions/runs/30759050941) | Rust 비렌더 2개 + 문서 | `rust=true`, `frontend=none`, `render=false`, `native_skia=false`, `CodeQL=rust`, `classified:rust` | full CI 성공 |
| [#3771](https://github.com/edwardkim/rhwp/pull/3771) | [30759557019](https://github.com/edwardkim/rhwp/actions/runs/30759557019) | Studio unit + Rust + `src/wasm_api/tests.rs` | `full`, `fail-closed:wasm-contract` | full CI 성공 |
| [#3819](https://github.com/edwardkim/rhwp/pull/3819) | [30759875509](https://github.com/edwardkim/rhwp/actions/runs/30759875509) | Rust renderer + Python fixture tool | `full`, `fail-closed:unclassified-path` | full CI 성공 후 merge |
| [#3740](https://github.com/edwardkim/rhwp/pull/3740) | [30760733806](https://github.com/edwardkim/rhwp/actions/runs/30760733806) | 364개 변경, Rust renderer + fixture/census tool + rename | `full`, `fail-closed:rename` | Rust fmt 실패 차단 |

#3749 preflight는 shadow input 15개를 수집했고 checkout·수집·분류·summary step이 모두 성공했다. 실제
legacy CI도 lint, archive, 8개 shard, Native Skia, aggregate가 모두 성공했다. classifier가 생략 후보로
본 Native Skia는 5분 21초, Rust 외 CodeQL 두 job은 3분 38초를 사용했지만 둘 다 성공했다.

#3771은 frontend 파일이 있어도 WASM 경계를 함께 바꾸므로 mode를 부분 승격하지 않고 모든 축을 `full`로
닫았다. 이는 mixed 변경이 가장 보수적인 축으로 승격되어야 한다는 계약과 일치하며, 실제 legacy
worker도 전부 성공했다.

#3819는 renderer가 있어 Rust render 축 자체는 명확하지만 `tools/make_*.py`를 현재 classifier가 별도
lane으로 분류하지 않는다. 일부 파일만 보고 좁히지 않고 최종 결과를 full로 승격했고, legacy worker도
모두 성공한 뒤 PR이 merge됐다.

#3740의 최신 run은 364개 파일 목록에 rename이 포함되어 `fail-closed:rename`으로 full 판정됐다. 이전
2개 run은 새 push로 취소됐고, 최신 run에서는 `cargo fmt --all -- --check`가
`tests/issue_3738_rowbreak_table_footnote_fragment.rs`의 실제 formatting 차이를 차단했다. lint 실패 뒤
archive·shard·Native Skia가 실행되지 않은 것은 기존 dependency 계약이며 classifier false negative가
아니다.

live 고유 PR 4건은 모두 완료됐다. 다만 `classified` non-full 완료 표본은 Rust 비렌더 #3749 한 건뿐이다.
저장소의 기존 CI 측정 기준상 1~4건은 소수 관측값이므로 P50/P90이나 활성화 판단에 사용하지 않는다.

## 3. 최근 종료 PR 60건 historical replay

### 3.1 판정 분포

| 판정 | 건수 | 비율 |
| --- | ---: | ---: |
| `full` | 38 | 63.3% |
| Rust 비렌더 | 16 | 26.7% |
| Rust 렌더 | 4 | 6.7% |
| frontend `unit` | 1 | 1.7% |
| review-only | 1 | 1.7% |
| frontend `package` | 0 | 0% |
| frontend render | 0 | 0% |

`full` 38건의 reason은 다음과 같다.

| reason | 건수 | 근거 |
| --- | ---: | --- |
| `main-render-boundary` | 29 | `src/main.rs`; #3789 완료 전 보수적 full |
| `workflow-contract` | 4 | `.github/workflows/**` |
| `graphql-file-list-truncated` | 2 | replay 수집 결과 100/161, 100/169개로 잘림 |
| `wasm-contract` | 1 | `src/wasm_api.rs` 계열 |
| `classifier-contract` | 1 | #3792 자체 |
| `unclassified-path` | 1 | #3795의 `tools/agent_preflight.py` |

실제 workflow·WASM·classifier·미분류·mixed 경로가 모두 full로 닫혔다. 최근 종료 PR 60건 replay에는
rename과 PR 3,000개 API 경계의 자연 표본이 없었지만, rename은 이후 live #3740에서 full fallback이
확인됐다. PR 3,000개 경계는 Stage 1 단위·workflow 계약 테스트로만 확인된 상태다.

`src/main.rs`가 60건 중 29건을 full로 만든 것은 [#3789](https://github.com/edwardkim/rhwp/issues/3789)의
모듈 경계 분리가 후속 확장성에 실제 가치가 있음을 보여준다. 다만 #3789가 완료되기 전에는 이 29건을
부분 판정으로 바꾸지 않는다.

### 3.2 false negative 대조

- Rust 비렌더 16건 중 15건은 Native Skia가 실제 실행되어 모두 성공했고, 1건(#3778)은 기존
  review-only commit 재사용 fast-pass로 worker 전체가 이미 skipped 됐다.
- frontend `unit` 표본 #3785에서는 classifier가 생략 후보로 본 Rust lint/archive/8 shard/Native Skia와
  Canvas가 모두 실제 실행되어 성공했다.
- 비-full 코드 PR 21건의 CodeQL을 대조했다. Rust-only 20건 중 19건은 JavaScript/Python/Rust matrix가
  실제 실행되어 모두 성공했고 1건은 기존 fast-pass였다. frontend-only #3785의 세 언어도 모두 성공했다.
- #3722의 CI failure는 classifier가 유지하는 default-feature shard 4에서 발생했다. 같은 run의
  Native Skia는 성공했다. 즉 `rust_required=true`가 실제 Rust 회귀 차단을 유지하면서
  `native_skia_required=false`만 좁히는 판정과 충돌하지 않는다.
- classifier가 생략 후보로 판정한 실제 worker에서 code failure는 관찰되지 않았다.

따라서 이 표본에서 관찰된 false negative는 0건이다. 다만 successful legacy worker는 해당 worker가
불필요했다는 완전한 증명이 아니므로, 이는 경로 계약과 실패 결과를 함께 본 활성화 전 근거로만 쓴다.

## 4. 실제 runner-minute 절감 후보

| 축·표본 | n | 실제 생략 후보 시간 | 해석 |
| --- | ---: | ---: | --- |
| Rust 비렌더의 Native Skia | 15 | 합계 83분 29초, P50 5분 33초, P90 6분 18초 | Stage 4 runner 비용 절감; 대부분 critical path 밖 |
| #3785 frontend-only의 Rust worker | 1 | 33분 41초 | lint + archive + Native Skia + 8 shard 합계 |
| Rust-only의 JavaScript/Python CodeQL | 19 | 합계 66분 17초, P50 3분 28초, P90 3분 41초 | Stage 5 runner 비용 절감 |
| #3785의 Rust/Python CodeQL | 1 | 11분 24초 | JavaScript/TypeScript만 유지 |
| #3785의 Canvas visual diff | 1 | 5분 24초 | Stage 3 render 비영향 skip |
| **측정된 생략 후보 합계** | — | **200분 15초** | 최근 60 PR replay 기준, 단계 전체 활성화 상한 |

#3785의 main CI는 16분 4초였고 frontend package gate는 시작 후 2분 14초에 끝났다. Rust worker를 생략하면
aggregate 전제상 main CI critical path가 대략 2분 30초 수준까지 줄 수 있지만, 이는 queue가 같고 새
`unit` gate가 현재 package gate보다 느리지 않다는 조건부 추정이다. 반대로 live #3749에서는 Native
Skia가 archive/shard보다 먼저 끝났으므로 5분 21초를 생략해도 CI wall time은 거의 줄지 않고
runner-minute만 줄어든다.

200분 15초에는 frontend `unit`과 `package` 자체를 나눈 추가 절감은 포함하지 않았다. 반대로 full 38건은
절감 0으로 계산했으며, #3789 완료 뒤 재분류될 가능성도 포함하지 않았다.

## 5. 판정과 다음 관찰 게이트

classifier version 1을 보정해야 할 false negative는 발견하지 못했다. 그러나 Stage 3 활성화는 아직
진행하지 않는다.

1. merge 이후 live 고유 PR 표본이 4건뿐이고 완료 non-full 표본은 Rust 비렌더 1건뿐이다.
2. live non-full frontend `unit`, `package`와 `render_required=true`가 모두 0건이다.
3. historical replay 60건에도 `package`와 frontend render가 없어, Stage 1 fixture 외 운영 표본이 없다.
4. rename full fallback은 live #3740에서 확인했지만 PR 3,000개 경계는 합성 계약 테스트만 있다.

Stage 2의 기존 run은 PR merge ref에서 checkout한 classifier를 사용해 authority가
`pr-merge-advisory`였다. 다음 활성화 판정은 Stage 2.5의 `pr-base-trusted-shadow` authority로 완료된
live `classified` code run이 최소 5건 쌓인 뒤 갱신한다. 그 안에 frontend
`unit|package`와 `render_required=true`를 각각 한 건 이상 확보하지 못하면 해당 축은 계속 advisory로
남긴다. mixed full은 #3771의 WASM, #3819의 미분류 tooling, #3740의 rename에서 관찰했으며, 추가
표본에서도 shadow step 실패·생략 후보 worker failure가 없어야 한다.

관찰 게이트가 충족되기 전에는 worker `if`, Render Diff trigger, CodeQL matrix를 변경하지 않는다.
다음 실측 갱신에서도 false negative가 0이고 frontend 두 mode와 render 축이 모두 관찰되면 Stage 3 PR을
시작한다.

## 6. Stage 2.5 — trusted-base shadow

Stage 3 전에 classifier 구현의 신뢰 경계를 먼저 고정한다.

- `pull_request`에서는 `github.event.pull_request.base.sha`의
  `scripts/ci-impact-classifier.cjs`만 sparse checkout한다.
- push·manual 실행은 기존처럼 해당 실행의 `github.sha`를 사용한다.
- checkout credential은 저장하지 않고, classifier 실행 step에는 토큰을 전달하지 않는다.
- PR 판정 authority는 `pr-base-trusted-shadow`로 기록해 기존 `pr-merge-advisory` 표본과 구분한다.
- checkout 실패 또는 base에 classifier 파일이 없는 sparse checkout 성공은 authority도
  `unavailable-advisory`로 낮추며, checkout·수집·분류 실패는 기존 기본값인 모든 영향축 `full`로 남는다.
- 기존 `frontend_required` 및 Rust·frontend·Native Skia worker `if`는 shadow output을 참조하지 않는다.

이 단계는 신뢰된 classifier 버전을 운영 표본에 적용하기 위한 관찰 단계다. 실제 skip 활성화는 아니며,
Stage 2.5 merge 이후 새 authority로 누적된 표본만 Stage 3 활성화 게이트에 사용한다.

여기서 authority는 base SHA에서 읽은 classifier 파일의 provenance만 뜻한다. workflow YAML, 인라인 수집
script, classifier 호출부와 미래 worker `if`는 PR merge ref의 제어를 받으므로 Stage 2.5만으로 trusted
execution이 완성되지는 않는다. Stage 3 전에 trusted controller/policy check를 두거나 신뢰되지 않은 PR을
full로 유지하는 Stage 2.6 경계를 별도로 확정한다.

## 6.1 #3892 이후 topology 반영

#3892가 legacy 단일 archive·8 shard를 세 builder와 `slow/1/2/3` 네 archive·네 worker로 바꿨다. 이 변경은
full CI wall time을 줄였지만 frontend-only PR에서 Rust worker를 실행하는 영향축 문제는 남긴다. 따라서
Stage 2의 8-shard runner-minute는 historical 수치로 보존하고, Stage 4 조건화는 세 builder·네 worker와
aggregate 진리표를 함께 다룬다. Stage 6 artifact retry도 논리 label별 test archive, archive expected
count와 worker run count를 기준으로 다시 설계한다. cache 비교는 #3810의 정리 직후 4.73GB와 임의 시점
총량을 직접 비교하지 않고 Stage 4 이후 다음 sweep 직후 snapshot을 사용한다.

## 7. Stage 2.5 로컬 검증

2026-08-03 KST에 다음 집중 검증을 수행했다.

```bash
node --test scripts/tests/ci-impact-classifier.test.cjs
python3 -m unittest \
  scripts/tests/test_ci_impact_workflow.py \
  scripts/tests/test_render_diff_workflow.py
actionlint .github/workflows/ci.yml
git diff --check
```

- classifier 단위 테스트: 20개 통과
- workflow 계약 테스트: 7개 통과
- `actionlint`: 진단 없음
- whitespace 검사: 통과

Rust·Studio 소스와 worker 실행 계약은 바꾸지 않았으므로 장시간 Rust 전체 테스트는 로컬 범위에서
제외한다. 원격 PR에서는 기존 full CI가 그대로 실행되어 shadow 변경이 worker 결과를 바꾸지 않는지
확인한다.

2026-08-04 KST에는 리뷰 F1 보정과 최신 `upstream/devel` 동기화 뒤 같은 집중 검증을 다시 실행했다.
classifier 파일 존재 검사를 포함한 workflow 계약 테스트 7건, classifier 단위 테스트 20건이 통과했고,
`actionlint`는 `.github/workflows/ci.yml`, `.github/workflows/build-nextest-archives.yml`,
`.github/workflows/run-nextest-archives.yml` 모두 진단이 없었다. `git diff --check`도 통과했다. 최신
GitHub Actions는 이 보정 commit이 push된 head에서 다시 확인한다.

2026-08-03 KST에 [draft PR #3823](https://github.com/edwardkim/rhwp/pull/3823)을 `devel` 대상으로
생성했다. review request는 보내지 않았으며, 원격 CI 통과와 trusted authority summary 확인 전에는
ready for review로 전환하지 않는다.
