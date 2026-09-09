# Task M100 #2183 Stage 1 완료 보고 — 프론트 CI workflow 구현

- 이슈: #2183
- 상위 추적: #2022
- 브랜치: `task2183-frontend-ci-gate`
- 기준 커밋: `3cf6d949a7617f066850065e245f4902a98e3c9e`
- 작성일: 2026-07-11
- 수행 계획서: `mydocs/plans/task_m100_2183.md`
- 구현 계획서: `mydocs/plans/task_m100_2183_impl.md`

## 1. 완료 범위

`.github/workflows/ci.yml`에 다음을 구현했다.

1. `preflight`의 frontend 영향 판정 output과 별도 detector
2. fresh dev WASM과 package gate를 실행하는 `Frontend package gates` worker
3. frontend worker 결과를 기존 required surface인 `Build & Test`에 연결하는 aggregate 판정

package source, lockfile, release WASM job, Rust worker profile, branch protection 이름은 변경하지 않았다.

## 2. frontend 영향 판정

### 경로

- `rhwp-studio/**`
- `rhwp-chrome/**`
- `rhwp-firefox/**`
- `rhwp-safari/**`
- `rhwp-vscode/**`
- `rhwp-shared/**`
- `npm/editor/**`
- `scripts/frontend-*.mjs`
- `web/**`
- `Cargo.lock`
- `src/wasm_api.rs`
- `.github/workflows/ci.yml`

PR은 paginated Pull Request Files API, branch push는 `before...after` compare를 사용한다. rename은 현재
`filename`과 이동 전 `previous_filename`을 모두 판정한다. empty/truncated 목록, zero SHA, API 오류, tag,
manual, 미지원 이벤트는 모두 `frontend_required=true`로 처리한다.

review-only fast-pass는 기존 detector가 우선한다. preflight 성공, fast-pass 아님, frontend 영향 있음의 세 조건을
모두 만족할 때만 frontend worker가 실행된다.

## 3. frontend worker

worker는 Rust jobs와 병렬 실행되며 다음 순서를 사용한다.

1. Rust `1.93.1`과 `wasm32-unknown-unknown`, wasm-pack 설치
2. frontend 전용 restore-only cargo cache 복원
3. `wasm-pack build --target web --dev`로 clean runner의 `pkg/` 생성
4. Node.js 22와 네 package-lock의 npm download cache 설정
5. Studio·Chrome·Firefox·VS Code 각각 `npm ci`
6. binding/editor contract와 `npm/editor test --if-present`
7. shared·Chrome·Firefox service worker tests
8. Studio unit test/build
9. Chrome/Firefox build와 extension dist contract
10. VS Code production compile

PR cache는 restore-only이며 trusted `devel`/`main` push의 exact miss만 frontend WASM cache를 저장한다.
generated `pkg/`와 `dist/`는 artifact로 upload하거나 commit하지 않는다.

## 4. aggregate 계약

`Build & Test.needs`에 `frontend-package-gates`를 추가했다.

- frontend required: frontend result가 `success`여야 한다.
- frontend 불필요: frontend result가 `skipped`여야 한다.
- unknown frontend output: required로 간주해 silent skip을 실패시킨다.
- preflight 실패 또는 Rust worker 비-success: 기존과 같이 실패한다.
- review-only fast-pass: worker `skipped`를 허용하고 기존 candidate green을 재사용한다.

`frontend_reason`에는 PR이 제어할 수 있는 파일 경로가 포함될 수 있으므로 shell script 본문에 expression으로
직접 삽입하지 않고 step environment를 통해 전달한 뒤 quoted `printf`로 출력한다.

## 5. 정적 검증 결과

| 검증 | 결과 | 비고 |
|------|------|------|
| `actionlint .github/workflows/ci.yml` | PASS | YAML, job dependency, expression, shell 정적 검사 |
| `git diff --check` | PASS | whitespace 오류 없음 |
| frontend detector fixture | 13/13 PASS | PR frontend/Cargo.lock/Rust-only/rename/empty/3,000 boundary, push frontend/Rust-only/300 boundary/zero SHA/tag, manual, API 오류 |
| `Build & Test` aggregate fixture | 8/8 PASS | fast-pass, required success/failure, skip, preflight/Rust 실패, unknown output |

fixture는 workflow의 inline script와 aggregate shell을 파일에서 직접 추출해 실행했다. 별도 복제 구현을
테스트한 것이 아니다.

## 6. 원 이슈 대비 범위 확장

#2183 원문은 fresh WASM 비용을 advisory 또는 후속 판단으로 남겼다. 그러나 clean checkout에는 `pkg/`가 없고
Studio·Chrome·Firefox·VS Code가 이를 직접 소비하므로, 실제 package build를 수행하면서 fresh binding만
생략할 수 없다.

Stage 1은 stale tracked output 복사 대신 검증용 dev WASM build를 worker에 포함했다. release WASM artifact와
profile은 변경하지 않았다. runner 비용과 cache 효과는 draft PR 실측 후 maintainer 리뷰 대상으로 남긴다.

## 7. 미실행 항목과 다음 단계

다음 항목은 Stage 2 범위이므로 아직 실행하지 않았다.

- repository Docker service를 통한 local fresh WASM 생성
- 네 package의 `npm ci`
- Studio/extension/VS Code build와 전체 frontend contract test

작업지시자 승인 후 Stage 2 local consumer gate를 실행한다. push, draft PR 생성, GitHub 코멘트는 별도 초안을
먼저 제시하고 승인받기 전에는 수행하지 않는다.
