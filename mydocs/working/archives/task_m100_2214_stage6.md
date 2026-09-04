# Task M100 #2214 Stage 6 완료보고서 — E2E 발견성과 재발 방지 문서 보강

## 0. 판정 요약

- **Stage 판정**: 완료
- **production 변경**: 없음
- **E2E 발견성**: `npm run e2e:issue-2214` 추가, Render Diff workflow syntax gate 연결
- **문서화**: troubleshooting 신규 1개, tech/manual 기존 문서 각 1개 보강
- **Studio 검증**: 214/214, build, renderer contract 통과
- **브라우저 검증**: npm script로 HWP/HWPX 각 1회와 raw 8건 GREEN
- **산출물 정책**: JSON/PNG/timeline, WASM package, build cache는 ignored local evidence로 유지
- **제품 동일성**: Stage 5 production source와 WASM/JS/d.ts hash 동일

## 1. PR 자산 감사 결과

`/private/tmp`의 #2214 관련 자산을 tracked test, 재생성 가능한 산출물과 구버전 진단 worktree로
분류했다.

| 자산 | 규모·상태 | 판정 |
|------|-----------|------|
| `tests/issue_2214_page_local_repaint.rs` | non-ignored 3 tests | PR 유지, Rust 영구 GREEN |
| `tests/issue_2214_cache_matrix_probe.rs` | ignored 30-case, 약 84초 | PR 유지, #2193용 diagnostic |
| Studio unit/E2E source | npm test + 수동 browser gate | PR 유지 |
| `output/poc/task2214/` | 약 12MB, JSON 21·PNG 233 | PR 제외, 재생성 가능한 실행 증거 |
| `pkg/`, `target/`, `node_modules/`, `dist/` | build/runtime cache | PR 제외 |
| detached latest worktree | Stage 2 이전 진단 source/output | 최종 branch가 상위 집합, PR 제외 |
| final E2E review 복사본 | summary + PNG 약 2.6MB | 실행별 복사본, PR 제외 |

픽셀 SHA는 같은 실행에서 44번째 경계 뒤 화면이 시간 경과로 후퇴하지 않는지 판정하는 값이다.
Chrome, OS font와 DPR이 다른 머신 사이의 golden image가 아니므로 JSON/PNG를 tracked oracle로
추가하지 않았다. 구조 기대값은 Rust/Studio assertion에, 화면 시간축은 재생성 가능한 E2E에 둔다.

## 2. E2E 실행 진입점과 CI syntax gate

`rhwp-studio/package.json`에 다음 script를 추가했다.

```text
npm run e2e:issue-2214 -- --runs=1
```

기본 command는 headless 모드를 사용하고, `--runs`·`--formats` 같은 뒤쪽 인자를 기존 runner로
전달한다. Vite와 Chrome 경로는 기존 E2E 환경변수 계약을 그대로 사용한다.

`.github/workflows/render-diff.yml`의 기존 script syntax 단계에 다음을 추가했다.

```text
node --check e2e/issue-2214-page-local-repaint.test.mjs
```

전체 focused E2E는 로컬에서 약 95초이고 Vite, Chrome build, font/DPR 통제가 필요하다. Linux
runner의 결과를 별도로 검증하지 않은 채 필수 CI 실행으로 올리지 않고, 이번 Stage에서는 값싼
syntax 부식 방지만 추가했다.

## 3. 재사용 문서

### 3.1 troubleshooting

`mydocs/troubleshootings/deferred_cell_edit_cache_coherence.md`는 이슈별 결과 수치가 아니라 다음
재발 진단 절차를 고정한다.

- model/LINE_SEG → cell layout cache → page tree → cursor → Canvas 층별 판정
- cold/warm, batch/sequential, direct/path 통제
- cache-only와 full-pagination oracle 분리
- scoped cache invalidation 불변식
- stable 동기 pre-cursor 0회와 flow boundary 동기 1회 flush 시도
- raw/redo/lifecycle과 one-shot effect
- global clear, post-cursor flush, timing hard gate와 raw golden 커밋 금지
- mixed-style ordered geometry의 미검증 한계와 후속 판정 기준

### 3.2 편집 action 기술 계약

`mydocs/tech/edit_action_undo_redo_architecture.md`에 현재 구현된 `TextMutationEffects`를 추가했다.

| 신호 | 계약 |
|------|------|
| `deferredPagination` | pending을 등록하되 자체로 동기 flush를 요구하지 않음 |
| `cellFlowChanged` | deferred flow 경계에서 cursor 전에 동기 flush 1회 시도 |
| `paginationCompleted` | immediate mutation이 기존 pending/timer를 완료·제거 |

history merge 전 현재 effect 캡처, one-shot 소비, redo 재계산, undo immediate, IME/iOS OR 누적과
문서/handler 수명주기 reset을 normative contract로 기록했다.

### 3.3 리뷰 체크리스트

`mydocs/manual/edit_command_review_checklist.md`에 다음 검토 질문을 추가했다.

- page-local repaint 범위와 layout/page/cursor cache scope가 함께 정합한가
- stable 입력은 동기 pre-cursor flush 0회이고, flow 경계는 논리 mutation당 동기 flush를 한
  번 시도하는가
- pending 등록과 boundary flush 시도가 exact cursor 조회보다 앞서며 실패 시 pending을 보존하는가
- normal/redo/IME/iOS에서 effect를 한 번만 소비·누적하는가
- immediate delete/fallback과 문서·composition lifecycle이 이전 pending을 정리하는가

## 4. 검증 결과

| 검증 | 결과 |
|------|------|
| `node --check e2e/issue-2214-page-local-repaint.test.mjs` | 통과 |
| `npm pkg get scripts.e2e:issue-2214` | headless script 확인 |
| `npm test` | 214 passed / 0 failed |
| `npm run build` | TypeScript + Vite build 통과 |
| `npm run e2e:renderer-contract` | 통과 |
| workflow YAML parse | 통과 |
| `npm run e2e:issue-2214 -- --runs=1` | HWP/HWPX 2/2 GREEN, raw 8/8 GREEN |
| `git diff --check` | 통과 |

브라우저 smoke 관찰값은 다음과 같다.

| 형식 | stable operation p95 | boundary operation | boundary flush | 경계 flush 수 |
|------|---------------------:|-------------------:|---------------:|--------------:|
| HWP | 29.0ms | 998.9ms | 952.5ms | 1 |
| HWPX | 28.9ms | 953.6ms | 907.4ms | 1 |

각 형식의 IME/iOS stable은 flush 0, boundary는 1이었다. 43→44 crop은 10,074 pixel이 바뀌었고
44번째 뒤 네 지연 checkpoint는 changed pixel 0이었다. 시간은 관찰값이며 hard gate는 횟수·순서,
exact state와 동일 실행 내 픽셀 안정성이다.

## 5. Stage 5 동일성

Stage 5 commit `f0596ded`와 production Rust/Studio source, native tests, Studio tests, #2214 E2E를
`git diff --exit-code`로 대조했고 차이가 없었다. package hash도 동일하다.

| 산출물 | SHA-256 |
|--------|---------|
| `pkg/rhwp_bg.wasm` | `3e0f2432830acc6a829a24d73807582bdcafbc6610e7a4ec262437f05e8df8d9` |
| `pkg/rhwp.js` | `c0cf1254f922af0863f0c89bea0c47dd20dd07c1a212b4f5119b4fa6f3ae5a6f` |
| `pkg/rhwp.d.ts` | `72387ccea782846a00d56f2afa2e16e60957fd9fdf7efcdccd58f8998959b370` |

따라서 Rust 재빌드나 Stage 5 광역 Rust gate 재실행 조건은 발생하지 않았다. Stage 6은 npm/CI
진입점과 문서만 변경했다.

## 6. 변경 파일

- `.github/workflows/render-diff.yml`
- `rhwp-studio/package.json`
- `mydocs/troubleshootings/deferred_cell_edit_cache_coherence.md`
- `mydocs/tech/edit_action_undo_redo_architecture.md`
- `mydocs/manual/edit_command_review_checklist.md`
- `mydocs/plans/task_m100_2214_impl.md`
- `mydocs/working/task_m100_2214_stage6.md`
- `mydocs/report/task_m100_2214_report.md`
- `mydocs/orders/20260713.md`

새 fixture, raw JSON/PNG, package와 production source는 포함하지 않는다. push, PR 생성, 이슈 close도
별도 승인 전 수행하지 않는다.
