---
kind: implementation_plan
status: proposed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-01
---

# PR #6564 메인터너 보정 수행계획

## 목표

PR #6564가 해결하려는 #4117의 본래 목적, 즉 셀 선택 클릭 없이 표 경계 hover와
resize를 시작하면서도 대형·분할 표의 마우스 이동 비용을 제한하는 계약을 최신
`devel` 위에서 완성한다.

원 contributor code commit은 보존한다. 보정은 최신 `devel` 병합, 기존 충돌 해결,
검증에서 확인된 캐시 불변식 결함과 그 회귀 테스트에 한정한다. contributor의 commit을
rebase·amend·force-push하지 않는다.

## 고정 기준선과 현재 사실

- 원 PR: [#6564](https://github.com/edwardkim/rhwp/pull/6564)
- 관련 이슈: [#4117](https://github.com/edwardkim/rhwp/issues/4117)
- contributor head: `0987770ef0174f0a4e4a0dbb39281707affd9e18`
- current `devel`: `0d1540931d59a8712c27f339fcbb71e1c00fd4b1`
- 분기 차이: `devel` 17 commits / PR 2 commits
- 충돌: `rhwp-studio/package.json` 한 파일
- 원 head의 focused 20건과 당시 GitHub Full CI는 성공했지만, 아래 두 반례를 덮지
  못하므로 current-base 보정 head를 새 검증 대상으로 삼는다.

## 재현된 결함과 보호 불변식

### R1 — 실패 메모가 페이지별로 유지되지 않는다

현재 단일 `tableBboxFetchFailure` 레코드는 page 0 실패 뒤 page 1 실패가 덮어쓴다.
page 0으로 돌아가면 다시 엔진을 호출해 “문서 변경 전 (표, 페이지)당 1회” 계약을
어긴다. 최소 재현 호출열은 `[0, 1, 0]`이다.

보호 불변식: 문서 snapshot이 바뀌기 전에는 서로 다른 모든 `(표, 페이지)` 실패를
각각 기억하고 같은 key를 반복 호출하지 않는다.

### R2 — 직접 성공 경로가 과거 실패를 해제하지 않는다

hover 조회 실패 뒤 셀 선택 mousedown의 직접 `getTableCellBboxes`가 성공해도 기존
실패 표식은 남는다. 표 밖 이동 또는 resize cleanup으로 bbox cache가 비워진 뒤 같은
페이지에 돌아오면 성공 가능한 조회가 실패 표식에 막힌다.

보호 불변식: 어떤 경로에서든 유효 bbox를 얻으면 그 표와 결과에 포함된 페이지의
실패 memory를 함께 제거한다.

### R3 — 분할 표의 페이지 membership을 이동마다 선형 검색한다

현재 cache hint와 hover page가 다르면 `cachedCellBboxes.some(...)`으로 전체 bbox를
매번 훑는다. 1,000셀 분할 표의 다른 페이지에서 mousemove마다 O(cells) scan이 남아
“표 진입 시 계산하고 이동은 cache만 읽는다”는 성능 의도에 맞지 않는다.

보호 불변식: bbox 성공 시 page membership `Set`을 한 번 만들고, hover hit는 O(1)
`Set.has(pageIdx)`로 판정한다.

## 수정 설계

1. `table-bbox-cache.ts`
   - 단일 실패 레코드를 `(sec, ppi, ci, pageIdx)` key의 `Set<string>`으로 바꾼다.
   - 성공 bbox를 cache하는 공용 helper를 추가한다.
   - helper가 `cachedTableRef`, `cachedCellBboxes`, page membership `Set`, 대응 실패
     key 제거를 원자적으로 수행한다.
   - hover cache hit에서 bbox 배열 `.some()`을 제거하고 membership `Set.has()`를 쓴다.
2. `input-handler.ts`
   - 실패 memory를 `Set<string>`으로 소유한다.
   - document snapshot 변경 시 `.clear()`하여 재시도를 허용한다.
3. `input-handler-mouse.ts`
   - 셀 선택 mousedown 성공 경로의 직접 cache 대입을 공용 helper로 교체한다.
   - 빈 결과는 성공으로 취급해 실패 memory를 지우지 않는다.
4. 회귀 테스트
   - page 0·1 실패 뒤 page 0 재진입이 재조회되지 않는 대조를 추가한다.
   - hover 실패 뒤 직접 성공·cache clear·재진입이 다시 조회되는 대조를 추가한다.
   - 분할 표 cache hit가 bbox 배열 `.some()`이 아니라 page membership을 쓰는 계약을
     고정한다.
5. latest `devel` 충돌
   - `package.json`의 #4117 E2E script와 devel의 #6557 두 E2E script를 모두 보존한다.
   - merge가 자동 처리한 `MANIFEST.md`와 table resize test도 PR 고유 diff로 다시
     대조한다.

## 단계와 검증

### Stage 1 — current-base 통합과 failing regression

- review branch에 current `upstream/devel`을 정상 merge한다.
- `package.json` 한 충돌만 의미 기반으로 해결한다.
- R1·R2·R3 회귀를 먼저 추가해 원 구현에서 실패하는 것을 확인한다.

### Stage 2 — cache 불변식 보정

- 위 공용 helper·failure Set·page membership Set을 구현한다.
- focused cache·mouse·pageHint test를 통과시킨다.
- source PR의 기존 50회 이동 1회 질의, 실패 재시도 금지, 분할 표 동작을 유지한다.

### Stage 3 — current-base 전체 로컬 게이트

Rust source가 포함된 candidate이므로 다음 필수 묶음을 순차 실행한다.

```text
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
```

- Studio: `npm ci`, TypeScript, `npm test`, E2E manifest 검사
- WASM: 표준 Docker `wasm` service로 current candidate를 fresh build
- browser: #4117 headless E2E를 실행해 클릭 전 hover cursor·marker·drag, +40px 폭
  변화, 엔진 호출 budget을 재확인한다.
- E2E screenshot을 직접 열어 표 경계·marker·drag 후 geometry를 판정한다.

### Stage 4 — review와 원격 게이트

- `mydocs/pr/pr_6564_review.md`에 원 head와 보정 head, conflict, 검증, 시각 판정을
  분리 기록한다.
- 판정은 원 head만의 승인으로 쓰지 않고 `메인터너 보정 후 수용 가능`로 기록한다.
- 작업지시자 승인 뒤에만 contributor source branch로 current candidate를 push한다.
- 새 head의 GitHub Full CI 성공과 `MERGEABLE/CLEAN`을 확인한 뒤 merge 승인을 다시
  요청한다.

## 제출·정리 경계

- 새 Rust integration source, generated suite·manifest, golden/baseline은 추가하지 않는다.
- source branch push, GitHub comment, merge는 각각 승인 전 수행하지 않는다.
- #6564 완료 뒤 review archive·오늘할일·issue/PR comment와 전용 worktree/branch 정리를
  post-merge 순서로 수행한다.
