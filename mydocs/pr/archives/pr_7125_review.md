# PR #7125 검토

- 검토일: 2026-09-14
- PR: https://github.com/edwardkim/rhwp/pull/7125
- 작성자: dependabot[bot], base devel. reviewer edwardkim 추가(기존 jangster77 유지).
- 경로: maintainer_general + intake_and_review + local_validation + multi_pr_update_branch + rework_and_exceptions; 렌더 의존성에는 visual_fixture_evidence 참고.
- 원 head: `ed495048782604aa0971912e01f762c5911d18d9`.
- 검토 base: `037e4906a93e99896daa145a5ee5517824bfeaf4`.
- 각 PR의 current-base merge-tree 및 여섯 건 누적 cherry-pick 모두 충돌 없음.
- 여섯 건 검토 순서: #7121 → #7122 → #7125 → #7126 → #7127 → #7128. #7121 실패 분리 후 나머지 다섯 건을 같은 순서로 재구성했다. 각 원 commit을 -x로 보존했고 원 PR branch는 변경하지 않았다. 기능상 선행 PR 의존성은 없으나 Cargo.lock 공통 변경을 누적으로 확인했다.
- 다섯 건 검증 후보: `4ff101bee6f187b20bda044d3182d64a6f2e95a0`.

## 검토·검증

package.json·package-lock.json만 변경하며 변경된 21개 dependency entry는 모두 dev dependency다. rolldown 1.2.8(+16 platform bindings), @oxc-project/types 0.149, postcss 8.5.28, picomatch 4.0.7, nanoid 3.3.19가 함께 갱신된다.
Node 24.15.0에서 Vite 엔진 조건 `^20.19.0 || >=22.12.0`을 충족한다. Studio(이미 #7123에서 병합), Chrome, Firefox에 설치된 Vite가 모두 8.3.0임을 확인했다.
빌드 스크립트는 Studio cwd에서 npx vite를 실행하므로 Studio 패키지 버전도 함께 확인했다.
`npm ci --ignore-scripts --no-audit --no-fund`, `npm run build --prefix rhwp-chrome` 성공. 두 extension dist+page-budget 계약 7개 통과.
실제 packaged Chrome 152.0.7977.54에서 viewer/options/print/service worker/content script smoke 통과.
Puppeteer 기본 기대 버전 152.0.7977.75 미설치로 첫 시도는 브라우저 시작 전에 실패했고, 기존 설치 경로를 `PUPPETEER_EXECUTABLE_PATH`로 명시한 두 번째 실행에서 통과했다. 제품 실패와 구분한다.
로그: `output/dependabot-20260914/chrome-build.log`, `extension-contracts.log`, `chrome-smoke-r2.log`.
[공식 Vite release](https://github.com/vitejs/vite/releases/tag/v8.3.0).
조판 원칙: 엔진 source·문서별 조건·baseline 변경 없음. 이번 검증은 번들·확장 동작이며 전체 시각 fidelity 판정이 아니다.
검증 입력: commit의 `samples/hwp3-pagedef-1915.hwp`, SHA256 `b272fdd218b4e91355167e63438a1605ef6902d75970c4ff5b8bae67087122d0`. 추적 여부 및 HEAD와 파일 동일성을 확인했다.

## 원격 CI 및 검증 경계

- Vite 8.3은 config의 `__dirname`이 미래 major의 native config loader에서 지원되지 않는다는 사전 경고를 냈다. 현재 기본 loader에서는 build 성공이며 config 수정은 하지 않았다. CanvasKit fs/path externalization, 큰 chunk, 별도 복사 asset 및 outDir 경고도 build 로그에 남았다. 실제 패키지 검사와 브라우저 동작은 통과했다.

- [Lint (fmt, clippy, WASM check)](https://github.com/edwardkim/rhwp/actions/runs/34800361317/job/103841903390): SKIPPED.
- [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/34800361317/job/103841903471): SKIPPED.
- [Frontend package gates](https://github.com/edwardkim/rhwp/actions/runs/34800361317/job/103841902611): SUCCESS.
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34800361317/job/103843803169): SUCCESS.

- 원 head의 진행/실패 check 없음. source·test·fixture 보정 없이 current-base merge clean이므로 원격 전체 CI를 재사용하고 로컬 집중 검증만 추가한다. SKIPPED를 실행 성공으로 세지 않는다.
- frontend 테스트에서 사용한 pkg는 기존 로컬 WASM SHA256 `61465d69c436028b5f5c8db93297a92fb7283cb58450ce0525b5b342943c1b9a`를 복사한 것이다. Rust 의존성 갱신을 반영해 새로 빌드한 WASM이라는 주장은 하지 않는다.
- primary devel·다른 작업의 worktree·원격 source 변경 없음. 원격 push·GitHub review/comment·merge는 하지 않았다.

## 판정

**승인**. 이는 기술 검토 판정이며 병합 권한 승인은 별도다.

병합 직전에 원 head·CI·current-base 충돌을 재확인한다. 나머지 다섯 건은 한 번의 일괄 병합 승인 대상으로 보고하고 #7121은 분리한다.

## 병합 후 기록

- 작업지시자의 5건 일괄 병합 승인으로 원 head 그대로 병합했다.
- merge SHA: `17edeb2d820c13e505c1b79ac2437cd048bcb34f`; 시각: 2026-09-14T07:33:16Z (UTC).
- 각 병합 직전 최신 devel fetch·merge-tree·exact head·성공 CI를 재확인했다. GitHub의 일시적 mergeable UNKNOWN은 재조회하여 MERGEABLE로 바뀐 뒤에만 진행했다.
- 5건 완료 뒤 devel `c3a200464ad09e843b4b3bfc08ba2469910d5280`과 로컬 검증 후보 `4ff101bee6f187b20bda044d3182d64a6f2e95a0`의 제품 tree가 동일함을 git diff로 확인했다.
- 별도 관련 issue 없음. maintainer 직접 반영으로 이 archive review와 오늘할일만 한 운영 기록 commit에 보존한다. 원격 Dependabot branch는 이번 작업에서 만들지 않았으므로 삭제하지 않는다.
- 이 merge의 duration 갱신 34818395337: cancelled. 일괄 병합 중 #7125·#7127의 중간 갱신은 cancelled였고 마지막 devel의 [34818448033](https://github.com/edwardkim/rhwp/actions/runs/34818448033)은 success다. CI 검증 실패로 취급하거나 재실행하지 않았다.
- 5개 merge 모두 Close Issues workflow success이며 별도 관련 issue 없음. CI·CodeQL·Adapter·Proptest 등 검증 workflow를 병합 뒤 새로 시작하지 않았다.
- 검토 로그는 기본 작업공간 `output/dependabot-20260914/`에 복사해 보존했다. 전용 worktree·로컬 review branch 및 이번 fetch ref는 후속 종료 시 제거하며 공유 `target/pr-review`와 다른 작업은 유지한다.
