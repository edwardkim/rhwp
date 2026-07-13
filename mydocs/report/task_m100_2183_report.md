# 최종 결과보고서 — Task M100 #2183: 프론트 package CI gate

- 이슈: #2183 / 상위 추적: #2022 / 작성일: 2026-07-13
- 구현 PR: #2216 / merge commit: `4f9aaaff12a34ad1cb1c5f782311f2f0280ac9a9`
- 계획: `plans/archives/task_m100_2183.md` + `plans/archives/task_m100_2183_impl.md`
- 단계 보고: `working/task_m100_2183_stage{1..3}.md`

## 요약

프론트 경로를 변경한 PR이 Rust CI만 통과하고 실제 Studio·확장·VS Code·`@rhwp/editor`를 검증하지 않던
false-green을 차단했다. frontend 영향 경로에서는 clean runner가 fresh dev WASM을 생성한 뒤 package별
install/test/build/compile 계약을 실행하고, 결과를 기존 required surface인 `CI / Build & Test`에 집계한다.

## 구현 결과

- frontend 영향 detector
  - Studio, Chrome, Firefox, Safari, VS Code, shared, `npm/editor`, legacy `web`
  - `scripts/frontend-*.mjs`, root `Cargo.lock`, `src/wasm_api.rs`, workflow 자체
  - rename의 현재·이전 경로 판정
  - empty/truncated/API 오류/미지원 이벤트는 frontend 실행으로 fail-open
- clean frontend worker
  - `wasm-pack build --target web --dev`로 ignored `pkg/`를 fresh 생성
  - Node.js 22와 Studio·Chrome·Firefox·VS Code package별 `npm ci`
  - binding/editor/shared SW/unit/build/dist/compile gate 실행
- aggregate·cache
  - frontend 결과를 기존 `Build & Test`에 연결하고 required/skip 상태를 강제 검증
  - PR은 frontend cargo cache restore-only
  - trusted `devel`/`main` exact miss만 cache save
- review-only fast-pass
  - 기존 Check Runs 조회를 우선 유지
  - fork PR check-run 부재 시 exact CI workflow·event·branch·head SHA·aggregate success를 Actions Jobs로 검증
  - 누락·진행·실패·API 오류는 full CI

## 검증 총괄

| 게이트 | 결과 |
|--------|------|
| `actionlint` / `git diff --check` | PASS / PASS |
| frontend detector fixture | 13/13 PASS (`Cargo.lock` 포함) |
| aggregate fixture | 8/8 PASS |
| review-only detector fixture | 8/8 PASS |
| local fresh WASM + binding/editor | PASS / 2 PASS |
| local shared·extension SW | 88 PASS |
| local Studio unit/build | 186 PASS / PASS |
| local Chrome·Firefox build + dist | PASS / 3 PASS |
| local VS Code compile | PASS |

최신 upstream rebase와 `Cargo.lock` 보정 후 전체 PR run
(`https://github.com/edwardkim/rhwp/actions/runs/29205856559`)도 모두 통과했다.

| GitHub Actions job | 결과 | 소요 |
|--------------------|------|------|
| Frontend package gates | PASS | 2분 21초 |
| Native Skia tests | PASS | 4분 22초 |
| Canvas visual diff | PASS | 4분 15초 |
| CodeQL Rust analyze | PASS | 7분 33초 |
| Build default-feature tests | PASS | 10분 26초 |
| Build & Test | PASS | 3초 |

최종 문서 commit의 review-only fast-pass run
(`https://github.com/edwardkim/rhwp/actions/runs/29206507981`)은 worker를 모두 skip하고 `Build & Test`를
3초에 통과했다.

## 리뷰·merge

- maintainer review `#pullrequestreview-4679480288`: APPROVE
- `Cargo.lock` trigger 제안: root exact-file 판정과 fixture로 #2216에 반영
- wasm-pack version pin 제안: 공통 toolchain 후속 이슈 #2233으로 분리
- `mydocs/orders/20260712.md` 충돌: upstream 승인 기록과 #2183 실행 행을 모두 보존해 해소
- PR #2216: 2026-07-13 merge, merge commit `4f9aaaff`

## trusted devel 실측

merge push run `https://github.com/edwardkim/rhwp/actions/runs/29206579472`은 전체 PASS했다.

- Frontend package gates: 2분 18초
- Native Skia tests: 4분 2초
- Build default-feature tests: 13분 2초
- Build & Test: 3초
- frontend cargo cache: exact miss
- trusted save key: `Linux-frontend-wasm-cargo-a46d0f017f4a35987ee21724119f2997aab983cdbf38f04e9b69da610d6e4499`
- cache upload: 226,230,556 bytes, save 성공
- npm download cache도 trusted push에서 save 성공

따라서 PR restore-only와 trusted exact-miss save 계약을 모두 실측했다.

## 잔여·후속

- #2233: Actions/Docker wasm-pack version pin과 dev/release/Render Diff toolchain 정합화
- #2234: trusted run에서 확인한 `actions/setup-node@v4`의 Node.js 20 deprecation 경고 해소와
  action major·npm cache 동작 검증
- #2187: #2183 선행 조건 완료. 최신 `devel` rebase와 새 `Frontend package gates` 실행 후 최종 리뷰
- #2125: #2187 처리 후 Phase A 착수

## 결론

#2183의 구현·검증·maintainer 승인·merge·trusted cache 실측이 완료됐다. 최종 보고 문서 반영 후 실행 이슈를
close하고, umbrella #2022의 다음 작업을 #2187 최종 리뷰로 전환할 수 있다.
