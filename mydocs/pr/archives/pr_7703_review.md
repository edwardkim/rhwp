---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-11
---

# PR #7703 검토 — 저장 위치 확인 중 로컬 문서 열기

## 최종 판정

**승인 — 저장 위치 확인을 켠 로컬 문서 드래그·드롭 열기를 메인테이너가 직접 확인했다.**
서비스 워커·옵션·패키징 및 실제 Chrome 원격 다운로드 대조군 검증도 통과했다. 직접 확인은 아래 인용한 문서 열기 결과에 한정하며 대화상자 종료·중복 탭·원본 보존을 모두 직접 확인한 것으로 확대하지 않는다. 사용자 병합·후속 처리 승인 뒤 원 기여 commit을 보존하는 merge 방식으로 병합했다. merge SHA는 `dde17f13996f1dae7d9192a36a75e3eeeb248186`, 시각은 `2026-10-11T13:24:54Z`다. 메인터너 코드 보정은 없다.

## 접수와 트리아지

- PR: [#7703](https://github.com/edwardkim/rhwp/pull/7703), 작성자 `oksure`, base `devel`, 관련 이슈 [#7664](https://github.com/edwardkim/rhwp/issues/7664) (`closes`). 첫 외부 기여가 아니다.
- 검토 head: `79d6d983462c4e2688ccb605a37b098ce69db769`.
- 최신 비교 base: `f79f2e1d6831a2187657b9c60deabc5d7c8b8e0d`.
- 담당자 `oksure`, reviewer `edwardkim`, milestone `v1.0.0`, labels `browser-extension`, `bug`, `documentation`, `javascript`, `test` 반영 후 전체 열린 PR 목록과 재대조했다.
- 선택 경로: maintainer 일반 + 접수/검토 + 로컬 검증 + 다수 PR. 관련 없는 #7704와 승인·검증을 분리한다.
- 최신 base의 `merge-tree --write-tree`는 충돌 없이 `e38138c027e4642c9b763fb190ee0314c546cc16`을 산출했다. PR/통합 diff의 whitespace 검사 통과.

## 문제와 구현 검토

이슈의 다운로드 기록은 로컬 HWP의 `filename`이 비어 있고 MIME이 generic인 상태에서 대화상자 취소로 종료한다. 기존 URL 힌트 판정은 `defer`이고 이 취소 이벤트는 재판정 기회를 주지 않는다.

수정은 로컬 파일 URL의 pathname을 확정 근거로 인정한다. 다운로드 상태 머신의 과거 항목 제외, 동일 ID 큐, handled 상태, 자동 열기 옵션과 자체 Blob 제외는 유지한다. 확정 비-HWP 파일명/MIME 및 DEXT5 제외가 새 판정보다 앞서므로 XLSX 보호 조건을 약화하지 않는다. 원격 URL의 HWP 힌트는 계속 defer한다.

`classifyDownload`의 intercept → adapter `processDownloadCandidateOnce`의 settings/handled 저장 → `openViewer`의 URLSearchParams → `tabs.create` 경로를 대조했다. `item.filename`이 비어 있을 때만 로컬 pathname 끝을 디코딩해 표시 이름으로 전달한다. 취소/erase는 기존 로컬 다운로드 억제 경로를 재사용한다. 이후 filename/interrupted 이벤트가 다시 뷰어를 만들지 않는지 검사한다.

Rust/parser/model/Studio 조판 속성과 저장 정보는 변경하지 않는다. Visual Sweep은 비해당이다. `file://` 링크를 다른 이름으로 저장하는 동작도 자동 열기로 처리한다는 기존 관찰자 API의 제약을 PR과 매뉴얼에 명시했다.

## 실행한 검증

검증 당시 임시 증적 디렉터리: `/home/edward/mygithub/rhwp/output/pr-review/oksure-20261011/`. #7703 소유 로그·검증 패키지는 후속 처리 뒤 정리하며, 아래 실행 결과·source SHA·CI 링크·빌드 해시를 이 archive에 보존한다. 같은 디렉터리의 보류 중인 #7704 증적은 정리하지 않는다.

| 검사 | 대상·결과 | 증적 |
| --- | --- | --- |
| 공유/Chrome/Firefox 서비스 워커 | 정확한 PR head, 205 passed | `pr7703-js-tests.log` |
| Chrome 옵션 UI | 정확한 PR head, 4 passed | `pr7703-options-tests.log` |
| 새 #7664 두 adapter 검사, 수정 전 | merge-base `21ed64071f81f35f37618717693bd30f727392c9`의 구현 + PR 검사, 2 failed (`viewer count 0 != 1`) | `pr7703-before-7664-tests.log` |
| 같은 검사, 수정 후 | 정확한 PR head, 2 passed | `pr7703-after-7664-tests.log` |
| GitHub frontend package gate | [38019276247](https://github.com/edwardkim/rhwp/actions/runs/38019276247) 성공. fresh WASM, SW/옵션, Studio, 확장 빌드/배포 파일 검사 실제 실행 | REST job steps 확인 |
| GitHub checks | 실행된 CI/CodeQL/adapter/proptest 성공. 영향 없는 Rust job은 skipped이며 전체 Rust 재실행으로 표현하지 않는다 | head별 체크 재조회 |

수정 전 재현은 별도의 ignored 모듈 사본에서 수행했다. 원 worktree 구현을 교체하지 않았다. 단순 import/build 실패가 아니라 두 브라우저 adapter의 뷰어 미생성 assertion으로 실패했다.

## Docker 패키지와 실제 browser 검증

`docker compose --project-name rhwp --env-file .env.docker run --rm wasm`을 review worktree의 저장소 루트에서 실행했다. 최적화를 포함해 성공했고, wrapper가 Studio public으로 동기화한 JS/WASM 및 Chrome/Firefox dist의 WASM SHA-256이 일치한다. `pr7703-docker-wasm.log`, `pr7703-package-provenance.json`에 source SHA·명령·해시를 보존했다.

Chrome·Firefox `build.mjs` 및 `scripts/frontend-extension-dist.test.mjs` 3건 통과. 기존 의존성 node_modules를 재사용했으며 PR은 lockfile을 변경하지 않는다. 공유 npm editor의 devel 차이는 README뿐이고 번들 소비 코드는 같다.

Chromium 154의 실제 설치된 확장으로 `rhwp-chrome/e2e/download-interceptor.test.mjs`를 실행했다. 일반/오도된 XLSX 두 건은 뷰어 0, HWP 8건(초기 상태 지연 포함)은 각 다운로드 ID마다 뷰어 1, 자체 Blob 저장은 내용 보존·추가 뷰어 0이었다. `pr7703-browser-download-e2e.log`에 보존했다. 원격 다운로드 대조군이며 native 로컬 Save As 대화상자 검증으로 쓰지 않는다.

직접 확인용 Chrome unpacked 폴더는 `/tmp/rhwp-pr7703-review-20261011/rhwp-chrome/dist/`, ZIP은 `/home/edward/mygithub/rhwp/output/pr-review/oksure-20261011/pr7703-chrome-local-review.zip`이다. Docker core와 PR head의 확장 소스로 만든 검증본이며 스토어 배포를 수행하지 않았다. 공개 `samples/hwp_table_test.hwp`를 이용한 직접 확인을 요청했다. 메인테이너 답변: “다운로드 전에 각 파일의 저장 위치 확인 을 키고 드래그&드롭하면 열립니다.” 설정이 켜진 상태의 문서 열기 실패 재현 시나리오가 해소됐다는 근거로 기록한다. 브라우저 버전과 대화상자 종료 여부는 답변에 없으므로 미확인으로 남긴다.

## Merge 후 contributor PR comment 계획

원 PR에 한국어 존댓말로 원 기여 감사, 실제 merge SHA와 관련 이슈 자동 종료, 실행한 CI와 SW 205건·옵션 4건·배포 검사 3건·실제 다운로드 E2E, 수정 전 실패/후 통과 및 메인테이너 직접 문서 열기 확인을 남긴다. 코드 보정 없이 수용했으며 조판·시각 검증은 비해당이다. 직접 답변에 없는 대화상자 종료·브라우저 버전을 확인했다고 주장하지 않는다. #7664에도 동일한 merge·검증 근거의 maintainer 댓글을 남기고, 두 본문을 파일로 게시한 뒤 REST API로 UTF-8 본문을 대조한다.

## 병합과 후속 처리

직접 확인 답변 수신 뒤 원격 PR은 같은 head `79d6d983462c4e2688ccb605a37b098ce69db769`, OPEN/MERGEABLE이며 실행 중·실패 체크가 없음을 재조회했다. 코드·build·테스트는 이후 변경하지 않았다. 최신 base와 충돌 없는 통합 결과도 앞 절의 증적을 유지한다.

사용자 승인 뒤 같은 head의 성공 체크와 충돌 없는 최신-base 통합 결과를 재확인하고 `--match-head-commit`으로 병합했다. 관련 [#7664](https://github.com/edwardkim/rhwp/issues/7664)는 `2026-10-11T13:25:05Z`에 자동 종료됐다. 최신 devel에서 merge SHA와 원 기여 head의 ancestor 포함을 확인했다.

[duration 갱신 작업](https://github.com/edwardkim/rhwp/actions/runs/38143075373)은 success이며 `ready:false / no-verified-pr-duration-measurements`를 기록했다. 이번 JS 변경 CI의 Rust worker 실측이 없어 metadata 갱신은 보류했다. 검증 CI를 병합 뒤 다시 실행하지 않았다. [이슈 종료 작업](https://github.com/edwardkim/rhwp/actions/runs/38143075353)도 success다.

후속 문서는 maintainer 직접 반영 경로로 이 archive와 오늘할일만 한 운영 기록 commit으로 devel에 반영한다. 최종 동기화 뒤 위 계획의 PR/issue 댓글을 게시·대조하고, 이 검토가 소유한 clean worktree `/tmp/rhwp-pr7703-review-20261011`, local branch `review/oksure-pr7703-20261011`, fetch ref `refs/remotes/upstream/oksure-pr7703-head` 및 `pr7703-*` 임시 증적을 정리한다. contributor fork `oksure/rhwp`의 `contrib/7664-local-file-download-prompt`, 공용 `target/pr-review`, 다른 worktree와 보류 중인 #7704 증적은 보존한다. 실제 댓글 permalink와 정리 확인은 일괄 검토 상태 기록에 남긴다.

Docker 산출물 SHA-256: JS `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`, WASM `cc5e9c9d86d2d3ac117a9184510bca661fb8cfaa6aef660707de2c9063575de2`. 루트 pkg·Studio public의 JS/WASM과 Chrome/Firefox dist WASM의 동일성을 확인했다.

#7704의 별도 시각 게이트와 승인에는 영향을 주지 않는다.
