# Issue #6988 구현·검증 보고서

- Issue: #6988 — Chrome 최초 다운로드 상태 저장과 완료 이벤트 경합
- 기준 SHA: `a3de5826c3b404bba8d7f3383d55c947f5a35aef` (`upstream/devel`)
- 검증한 구현 SHA: `cf76f3120a89745fd39fe1f31b48ddeba10e28e4`
- 브랜치: `codex/issue-6988-download-event-race`
- 환경: macOS arm64, Node 24.15.0, Google Chrome 153.0.8010.48, Rust 1.93.1
- 상태: 구현·로컬 검증 완료. 원격 push·PR 생성·CI·merge·이슈 close 미실행.

## 변경과 근거

최초 `storage.session.set`이 끝나기 전에 filename/complete 이벤트가 미추적 상태를 읽고 종료하면,
뒤늦은 onCreated는 미확정 metadata를 보류해 자동 열기 기회를 잃었다. 후보 처리 내부에 있던 잠금을
Chrome adapter의 ID별 이벤트 큐로 이동했다. onCreated/onChanged를 동기적으로 큐에 등록하고,
최초 상태 조회·저장부터 후보 처리·terminal 기록까지 도착 순서로 실행한다.

다른 ID는 독립적으로 진행한다. 실패는 기록 후 다음 이벤트로 이어지며, 마지막 이벤트가 끝나면
메모리 큐 항목을 제거한다. 신선도는 생성 이벤트 수신 시각으로 판단한다. session을 상태 정본으로
유지하고 공통 분류, 파일명 결정 단계, 로컬 파일 억제, Firefox/Safari는 변경하지 않았다.

## 검증 결과

| 검증 | 명령·방법 | 결과 |
| --- | --- | --- |
| 수정 전 Node 재현 | `node --test --test-name-pattern='initial tracking write racing' rhwp-chrome/sw/download-interceptor.test.mjs`를 기존 어댑터에서 실행 | HWP 두 사례 viewer 0, autoOpen=false 처리 마커 누락. 의도한 실패 3건 |
| Chrome/shared 회귀 | `node --test rhwp-chrome/sw/*.test.mjs rhwp-shared/sw/*.test.js` | 170 passed, 0 failed |
| JS 문법 | 변경한 adapter와 E2E의 `node --check` | 통과 |
| Chrome 패키지 계약 | `node --test --test-name-pattern='chrome extension dist contract' scripts/frontend-extension-dist.test.mjs` | 1 passed |
| 확장 빌드·실제 다운로드 | `PUPPETEER_EXECUTABLE_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' npm --prefix rhwp-chrome run test:e2e:download` | 정상/유도 XLSX 2건 탭 0, 기존 HWP 2건 각각 탭 1, 경합 대조군 3/3·지연군 3/3 각각 탭 1, 자체 Blob 탭 0 |
| 수정 전 실제 Chrome 대조 | 같은 dist의 adapter만 기준 SHA 원본으로 교체하고 `RHWP_EXTENSION_DOWNLOAD_CASE=initial-state-delayed-1 node rhwp-chrome/e2e/download-interceptor.test.mjs` 실행 | 실제 8,704 bytes 다운로드 완료·원본 일치 후 viewer 대기 30초 실패. 종료 후 수정본으로 복구, source/dist 바이트 일치 확인 |
| 공백·패키지 상태 | `git diff --check`, source/dist adapter 비교 | 통과 |

추가 Node 계약: 최초 read 보류, 최초 write 보류 중 filename/complete, 중복 created/complete,
다른 ID 독립 처리, 초기 storage 오류 후 같은 ID 복구를 검증했다. 기존 과거 다운로드 제외,
worker 재시작 mock의 session 추적/handled 보존, 자체 Blob, autoOpen=false도 통과했다.

실제 Chrome 테스트는 격리 profile·loopback 서버와 실제 browser 다운로드를 사용했다. 최초
storage write만 테스트가 보류하고 실제 complete 이벤트를 수신한 뒤 재개한다. 제품 소스에
테스트 API를 추가하거나 browser 이벤트를 합성하지 않았다. 대조·지연 각각 3회 모두 원본 바이트를
비교하고 1.5초 quiet window에 ID당 viewer 1개를 확인했다. profile·다운로드 파일은 종료 후 정리됐다.

## 빌드 입력과 재현 자산

- 기존 fixture: `samples/re-font-dotum-empty-hancom.hwp`, 8,704 bytes
- fixture SHA-256: `1ee8871f37bec2e97d0928709dc411c0eacad656f35aa3d92c5cf89f61c5761b`
- 최신 기준 Rust에서 native dev WASM 빌드:
  `CARGO_TARGET_DIR=<공유 target/pr-review> scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`
- WASM SHA-256: `49c1df076d7f5aee8218b2cf77072e4f3f546e6f93fc793b33c3329902eeee78`
- 빌드된 확장: `rhwp-chrome/dist` (생성물, 커밋 제외)
- 상세 임시 로그: 작업공간의 `output/issue6988/{wasm,before,unit-all,e2e,e2e-before}.log` (커밋 제외)
- 회귀 실행 절차: [Chrome 빌드 가이드](../manual/chrome_edge_extension_build_deploy.md)
- 구현 계획: [#6988 계획](../plans/task_m100_6988_plan.md)

Rust/Studio source와 Rust 검증 입력은 변경하지 않았다. Cargo 전체 회귀·Clippy와 Studio 단위 검증은
이 변경의 검증 범위가 아니다. WASM은 실제 확장 패키지를 구성하기 위해 새로 빌드했으며 native
`--no-opt` 산출물이다. 배포용 Docker/wasm-opt 최적화 결과로 표시하지 않는다.

## 한계와 후속

지연 주입 없는 자연 발생 빈도, 실제 worker suspend/resume 도중 이벤트 생존, Edge/Firefox/Safari의
실행 결과는 이번 검증에 포함하지 않는다. Node의 worker 재시작 mock과 실제 브라우저의 실행 중
경합 검증을 구분한다. 저장 자체가 영구 실패할 때 자동 복구하는 기능을 추가한 변경은 아니다.

검증한 구현 이후 변경은 보고서·PR 초안뿐이다. 승인 후 `origin` 작업 브랜치에 push하고
`edwardkim/rhwp:devel` 대상 PR을 생성한다. 최신 required CI와 검토는 PR 생성 후 확인한다.

승인 후 사용할 PR 제목: `fix(chrome): 최초 상태 저장 중 다운로드 완료 이벤트 보존 (#6988)`

```sh
git push origin codex/issue-6988-download-event-race
gh pr create --repo edwardkim/rhwp --base devel --head postmelee:codex/issue-6988-download-event-race \
  --title 'fix(chrome): 최초 상태 저장 중 다운로드 완료 이벤트 보존 (#6988)' \
  --body-file mydocs/working/task_m100_6988_pr_body.md
```
