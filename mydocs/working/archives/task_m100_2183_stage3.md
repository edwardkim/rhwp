# Task M100 #2183 Stage 3 완료 보고 — PR GitHub Actions 실측 및 리뷰 준비

- 이슈: #2183
- 상위 추적: #2022
- PR: #2216
- PR URL: `https://github.com/edwardkim/rhwp/pull/2216`
- 브랜치: `postmelee:task2183-frontend-ci-gate`
- base: `edwardkim:devel`
- 검증 HEAD: `c81a4af4b2279c2641cb34a80daaecbb1ec41dde`
- 작성일: 2026-07-11
- 단계: GitHub Actions 실측 완료 / Ready for review

## 1. 등록 결과

승인된 PR 본문으로 draft PR #2216을 생성하고 `@edwardkim`에게 직접 리뷰를 요청했다. 전체 검증과
체크리스트 갱신을 마친 뒤 2026-07-12 작업지시자가 Ready for review로 전환했다.

- 제목: `[CI] 프론트 패키지 변경 시 build/test gate 추가 (#2183)`
- issue 연결: `Refs #2183`
- 리뷰 요청 코멘트: `https://github.com/edwardkim/rhwp/pull/2216#issuecomment-4946198647`
- 현재 상태: Ready for review / Open

issue 자동 close는 작업지시자 승인 전 수행하지 않기 위해 `Closes`가 아니라 `Refs`를 사용했다.

## 2. 첫 CI run

- CI run: `https://github.com/edwardkim/rhwp/actions/runs/29154371713`
- 시작: 2026-07-11 22:28 KST
- `Build & Test` 완료: 2026-07-11 22:39 KST
- 전체 결과: PASS

workflow 자체가 frontend 영향 경로이므로 preflight는 다음과 같이 판정했다.

```text
preflight=success
fast_pass=false
frontend_required=true
frontend_reason=frontend-path:.github/workflows/ci.yml
build_default=success
native_skia=success
frontend=success
```

기존 required surface인 `CI / Build & Test`는 별도 check 이름 변경 없이 2초에 PASS했다. release용
`WASM Build`는 PR에서 기존대로 skip됐다.

## 3. frontend worker 실측

`Frontend package gates`는 clean Ubuntu runner에서 2분 21초에 PASS했다.

| 단계 | 결과 | 소요 |
|------|------|------|
| checkout | PASS | 22초 |
| Rust 1.93.1 + wasm target | PASS | 10초 |
| wasm-pack installer | PASS | 1초 |
| frontend cargo cache restore | MISS | 1초 미만 |
| `wasm-pack build --target web --dev` | PASS | 69초 |
| Node.js setup | PASS, `22.23.1` | 3초 |
| 네 package dependency install | PASS | 12초 |
| binding/editor contract | PASS | 1초 미만 |
| `npm/editor test --if-present` | PASS | 1초 미만 |
| shared/extension SW tests | PASS | 1초 |
| Studio unit tests | PASS | 2초 |
| Studio build | PASS | 4초 |
| Chrome/Firefox build | PASS | 3초 |
| extension dist contract | PASS | 1초 |
| VS Code compile | PASS | 6초 |
| frontend cargo cache save | SKIP | PR restore-only 계약 |

Node.js 22와 fresh dev WASM 조합에서 local Stage 2와 같은 consumer gate가 모두 통과했다.

## 4. 전체 workflow 비교

| check/job | 결과 | 소요 |
|-----------|------|------|
| Frontend package gates | PASS | 2분 21초 |
| Native Skia tests | PASS | 4분 31초 |
| Canvas visual diff | PASS | 3분 49초 |
| CodeQL Rust analyze | PASS | 7분 32초 |
| Build default-feature tests | PASS | 10분 33초 |
| Build & Test aggregate | PASS | 2초 |

frontend worker는 default-feature worker보다 8분 이상 먼저 끝났다. 첫 cache miss run에서도 기존 critical
path 안에 들어가므로 이 PR의 실측에서는 전체 wall-clock을 늘리지 않았다.

## 5. cache 관찰

### frontend cargo cache

- key: `Linux-frontend-wasm-cargo-${Cargo.lock hash}`
- exact/prefix restore: MISS
- PR save step: 조건대로 SKIP

첫 run이므로 expected miss다. merge 후 trusted `devel` push에서 exact miss cache가 저장된 뒤 후속 frontend
PR의 restore hit를 확인할 수 있다.

### npm cache

`actions/setup-node`는 네 lockfile hash의 npm cache를 찾지 못했다. post step에서는 같은 key를 다른 job이
생성 중이라는 이유로 reserve에 실패했지만 job conclusion에는 영향을 주지 않았다. `node_modules`를 cache하지
않고 `npm ci`를 실행한다는 계약은 유지됐다.

## 6. 도구 버전 관찰

기존 release/Render Diff와 같은 installer URL은 runner에 `wasm-pack 0.13.1`을 설치했고 0.15.0 업데이트
경고를 출력했다. local Docker는 Dockerfile에 고정된 `wasm-pack 0.15.0`을 사용했다.

두 버전 모두 같은 HEAD의 fresh binding과 consumer gate를 통과했다. #2183에서 release/Render Diff installer
정책까지 함께 바꾸면 범위가 확장되므로 즉시 수정하지 않고 maintainer 리뷰 관찰로 남긴다. pin 통일이 필요하면
별도 workflow 도구chain 정리 단위로 다룬다.

## 7. 확인된 계약

1. workflow 변경은 frontend 영향으로 판정된다.
2. clean runner에서 fresh dev `pkg/` 생성 후 모든 package gate가 통과한다.
3. PR frontend cargo cache는 restore-only로 동작한다.
4. frontend success가 기존 `Build & Test` aggregate에 실제 반영된다.
5. release `WASM Build` trigger와 artifact 정책은 바뀌지 않는다.
6. frontend worker는 첫 miss에서도 기존 Rust critical path보다 짧다.

## 8. 남은 확인과 다음 단계

- trusted `devel` push의 frontend cargo cache save는 merge 뒤에만 실측 가능하다.
- Rust-only PR의 frontend skip은 fixture로 검증했으며 실제 별도 PR을 만들지 않는다.
- Stage 3 보고서의 trailing docs commit에서 review-only fast-pass 실패를 실측했고 아래 보정 작업으로 전환했다.
- PR 본문 체크리스트와 CI 결과 코멘트는 작업지시자 승인 후 게시를 완료했다.
- 작업지시자 승인으로 Ready for review 전환을 완료했으며, maintainer 리뷰 승인 전 merge와 #2183 close는
  수행하지 않는다.

## 9. trailing docs fast-pass 실패와 보정

Stage 3 보고서 commit `b73a1255`를 push한 두 번째 CI run
(`https://github.com/edwardkim/rhwp/actions/runs/29154801461`)에서 review-only fast-pass가 성립하지 않고 전체
jobs가 재실행됐다. 전체 결과는 다시 PASS했고 frontend worker는 2분 10초였지만 preflight reason은 다음이었다.

```text
fast_pass=false
reason=missing-build-and-test:82a297ab53e0e67c15f2622d4a8819699fef8cb7
```

API를 직접 비교한 결과는 다음과 같다.

- candidate SHA의 Check Runs API: 0건
- 첫 CI run metadata `head_sha`: candidate SHA와 일치
- 첫 CI run jobs API: `Build & Test=completed/success` 존재
- Actions workflow-runs의 `head_sha=` server filter: fork run 누락
- source branch filter: 두 CI run 반환, 응답 `head_sha`로 exact match 가능

이는 새 frontend 판정 문제가 아니라 기존 review-only detector가 fork PR의 API 가시성 차이를 처리하지 못한
경우다. 구현 계획의 fast-pass 보존 완료 조건을 만족하기 위해 다음 최소 보정을 적용한다.

1. preflight에 read-only `actions: read` 추가
2. 기존 Check Runs 조회를 우선 유지
3. check-run 부재 시 현재 PR branch의 `ci.yml` runs 조회
4. 응답 `head_sha`를 candidate와 exact match
5. 해당 run의 `Build & Test` job이 `completed/success`인 경우에만 fast-pass
6. 누락·진행·실패·API 오류는 full CI

실제 inline script를 추출한 fixture 8건에서 check-run 성공, Actions job fallback 성공, 진행 중, job 실패,
job 누락, wrong SHA, API 오류, trailing docs 부재를 검증했다. 최종 GitHub 실측은 최신
`upstream/devel@413d8a67` rebase와 보정 commit push 후 수행한다.

## 10. 최신 upstream 재검증

최신 `upstream/devel@413d8a67`로 rebase하고 fork PR fallback 보정 commit `c81a4af4`를 push한 뒤 전체
workflow를 다시 실행했다.

- CI run: `https://github.com/edwardkim/rhwp/actions/runs/29158715334`
- 전체 결과: PASS
- `Build & Test`: PASS, 4초
- `Build default-feature tests`: PASS, 9분 40초
- `Native Skia tests`: PASS, 5분 36초
- `Frontend package gates`: PASS, 2분 10초
- `Canvas visual diff`: PASS, 3분 39초
- CodeQL Rust analyze: PASS, 8분 44초

최신 upstream에서도 frontend consumer gate와 기존 required surface가 모두 통과했다. 이 보고서 갱신만 담은
후속 commit을 review-only 후보로 사용해, candidate run의 `Build & Test=completed/success`를 우선
Check Runs 경로 또는 Actions Jobs fallback이 식별하고 `fast_pass=true`로 전환하는지 최종 실측한다.

## 11. review-only fast-pass 최종 실측

위 보고서 갱신만 담은 commit `36bfb223`의 CI run에서 review-only fast-pass가 정상 동작했다.

- CI run: `https://github.com/edwardkim/rhwp/actions/runs/29159065890`
- candidate SHA: `c81a4af4b2279c2641cb34a80daaecbb1ec41dde`
- preflight: PASS, 4초
- `fast_pass=true`
- reason: `build-and-test-green:success`
- `Build default-feature tests`: SKIP
- `Native Skia tests`: SKIP
- `Frontend package gates`: SKIP
- `Build & Test`: PASS, 3초
- Render Diff와 CodeQL: review-only fast-pass로 worker SKIP

이번 run에서는 candidate의 Check Runs API가 정상 노출되어 기존 우선 경로가 선택됐다. 따라서 Actions Jobs
fallback의 GitHub 실측을 인위적으로 만들지는 않았고, fork run의 check-run 부재를 재현한 fixture를 포함한
8건의 추출 script 검증으로 보완했다. fallback은 exact workflow, event, branch, `head_sha`, aggregate job의
`completed/success`를 모두 요구하며, 누락·진행·실패·API 오류에서는 full CI로 닫힌다.

결론적으로 최신 upstream의 전체 CI와 후속 review-only fast-pass가 모두 PASS했다. #2183의 구현·로컬 gate·
GitHub Actions 실측 범위에서 남은 실패는 없으며, trusted `devel` push의 frontend cargo cache save만 merge 후
관찰 항목으로 남는다.

## 12. maintainer 승인과 merge 전 보정

maintainer는 2026-07-12 PR review `#pullrequestreview-4679480288`에서 구현과 검증을 승인했다. merge 전 필수
요청은 최신 `devel` rebase와 `mydocs/orders/20260712.md` 충돌 해소 1건이며, 다음 두 항목은 비차단
제안으로 남겼다.

1. `Cargo.lock` 변경도 fresh dev WASM 산출에 영향을 줄 수 있으므로 frontend trigger에 추가 검토
2. Actions installer와 Docker의 wasm-pack 버전 pin 통일

첫 항목은 #2183 detector의 직접 누락이므로 root `Cargo.lock` exact-file trigger와 fixture를 이번 PR에
추가한다. 두 번째 항목은 frontend·release WASM·Render Diff 공통 toolchain 정책이므로 이 PR에서 installer를
변경하지 않고 별도 후속 이슈 #2233으로 분리했다.

최신 `upstream/devel@c7864c62`로 rebase하고 upstream의 PR #2216 승인 기록과 기존 #2183 실행 행을 모두
보존해 orders 충돌을 해소했다. 보정 후 workflow 정적 검증과 전체 GitHub Actions를 다시 통과한 뒤에만
merge 가능한 상태로 판단한다.

보정 commit `b3b0af01`의 전체 GitHub Actions run
(`https://github.com/edwardkim/rhwp/actions/runs/29205856559`)은 다음과 같이 PASS했다.

- `Frontend package gates`: 2분 21초
- `Native Skia tests`: 4분 22초
- `Canvas visual diff`: 4분 15초
- CodeQL Rust analyze: 7분 33초
- `Build default-feature tests`: 10분 26초
- `Build & Test`: 3초

PR은 최신 base `c7864c62`, head `b3b0af01` 기준 `MERGEABLE / CLEAN`이고 maintainer 승인도 유지됐다.
후속 문서 전용 상태 갱신의 review-only fast-pass까지 통과하면 merge 전 필수 조건은 모두 충족된다.
