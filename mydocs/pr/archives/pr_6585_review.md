# PR #6585 검토 기록 — v0.8.6 release candidate 준비

- PR: [#6585](https://github.com/edwardkim/rhwp/pull/6585)
- 이슈: [#6584](https://github.com/edwardkim/rhwp/issues/6584)
- 작성자·검토자: `edwardkim` collaborator self-review
- base: `devel@063041a2ced54085b5cf94c2e646ac7aa0e1960d`
- 검토 대상 code head: `4280831d1f25a189416c2fcec14e0d252dfb90c3`
- 규모: 45 files, +6,861 / -70, 13 commits
- 검토일: 2026-09-02 KST

## 1. 변경 범위

- Cargo·npm editor·Studio·VS Code·Chrome/Edge·Firefox·Safari의 정본 버전을 0.8.6으로 맞춘다.
- v0.8.4 이후 2,214 commits와 262 merged PR provenance를 대사한 결정론적 contributor ledger와
  변경 inventory를 추가한다.
- 한·영 CHANGELOG, GitHub Release note, root·npm·브라우저 확장 README, VS Code changelog,
  third-party license와 스토어 심사 문서를 현행화한다.
- contributor·release channel 계약 검사와 민감값 거부 fixture를 보강한다.
- renderer/parser/save 제품 실행 코드, integration test source, fixture·baseline·sample, CI workflow와
  GitHub 권한은 바꾸지 않는다.

PR 본문은 #6584를 `Refs`로 연결한다. 실제 `devel -> main`, tag·GitHub Release, 자동·수동 공식 채널 게시와
후속 이슈 정산이 남았으므로 이 PR merge만으로 #6584를 닫지 않는 것이 맞다.

## 2. metadata와 base 정합

- 작성 시점 PR 상태: Open, non-Draft, `MERGEABLE / CLEAN`.
- PR head와 remote task branch는 exact code head와 일치했다.
- `upstream/devel`은 계획에 고정한 release base에서 전진하지 않았다.
- `git merge-tree --write-tree upstream/devel HEAD`가 충돌 없이 tree
  `00f4cd19e2195ae74315cba643e08331709b0d0c`를 생성했다.
- 위 상태는 작성 시점 참고값이며 merge 직전에 최신 head·base·mergeability를 다시 확인한다.

## 3. 코드·기록 검토

### 3.1 버전과 배포 표면

Cargo root와 lock root package, npm editor, Studio, VS Code, Chrome package·manifest, Firefox
package·manifest, Safari manifest가 0.8.6으로 일치했다. 파생 `pkg/package.json`은 직접 수정하지 않았고,
브라우저 확장 권한과 외부 endpoint도 추가하지 않았다.

### 3.2 변경 기록과 기여자

기여자 audit는 Git author만 세지 않고 GitHub PR author, co-author, cherry-pick·통합 provenance를 함께
대사한다. 미해결 번호·정체성은 0이며 사람 20명·bot 1개·AI 공동작성 disposition을 분리했다. ledger,
`CHANGELOG.md`, `CHANGELOG_EN.md`, release note의 공개 사람 집합과 순서는 계약 검사로 고정됐다.

### 3.3 보안·라이선스·패키징

민감값 거부 테스트는 token 형태의 fixture를 source에 직접 쓰지 않도록 런타임 조합으로 바꿨지만 실제
거부 계약은 유지한다. AMO source archive와 npm package의 allowlist·denylist, symlink, 비밀정보,
저작권 폰트와 라이선스 검사는 Stage R4에서 통과했다. `THIRD_PARTY_LICENSES.md`의 Studio·VS Code
`@noble/hashes`와 `canvaskit-wasm` 버전도 각 lock과 대사됐다.

## 4. 렌더 영향과 시각 검증

제품 renderer/layout/typeset/paint, WASM API와 Studio 화면 코드는 변경하지 않았다. 이 PR 자체의 새
시각 sweep은 필요하지 않다. release candidate 전체 안전망으로 Stage R4에서 Native Skia 3종, Chrome CDP,
responsive headless E2E 1,082/1,082와 기존 렌더 회귀를 통과한 결과를 확인했다.

## 5. 검증

- exact code head의 PR-triggered check: 성공 29, 정책상 생략 3, 실패·대기 0.
- CI [run #33568191662](https://github.com/edwardkim/rhwp/actions/runs/33568191662), CodeQL
  [run #33568191630](https://github.com/edwardkim/rhwp/actions/runs/33568191630), Render Diff,
  Proptest와 Adapter inter-diff가 성공했다.
- 로컬 Stage R4: full nextest 8,925 pass/0 fail, native·WASM·workspace Clippy, workspace build,
  Native Skia, Docker WASM, npm package, Studio 1,362 pass/0 fail, 확장·VS Code build와 CDP E2E를
  통과했다.
- self-review focused 재검증: release·contributor Python 19/19, font decision trace Node 12/12,
  `git diff --check`가 통과했다.
- exact head `Release Binary(tag=test)` [run #33569503350](https://github.com/edwardkim/rhwp/actions/runs/33569503350):
  Windows x86_64, Linux x86_64/AArch64, macOS x86_64/AArch64 build 5/5 성공, release job은 의도대로
  생략됐다.
- 다섯 payload archive의 내부 4파일, 실행 형식·권한을 독립 검사했다. Linux AArch64는
  `ubuntu-24.04-arm`에서 native `rhwp --version`이 `rhwp v0.8.6`을 출력했다.

상세 명령·payload SHA-256·binary 크기는
[Stage R5 결과](../../working/task_m100_6584_stage_r5.md)에 고정했다. exact code head의 GitHub Full CI와
Stage R4 광범위 회귀를 재사용했으며, self-review 중 code·test·fixture·workflow 보정이 없었으므로 같은
전체 회귀를 중복 실행하지 않았다.

## 6. 발견 사항과 잔여 위험

- 차단 발견 사항은 없다.
- 실제 tag와 Release는 아직 없으므로 정식 다섯 asset, `SHA256SUMS.txt`, immutable npm version,
  Marketplace·Open VSX와 브라우저 스토어 결과는 Stage R6~R8에서 검증해야 한다.
- #5949는 실제 v0.8.6 Linux AArch64 asset 확인 뒤, #6243은 post-release Render Diff canary 성공 뒤,
  #6584는 필수 채널과 기록 정산 뒤에만 닫는다.
- 이 review·Stage R5·오늘할일 trailing commit은 `mydocs/`만 바꾼다. push 뒤 latest head의
  review-only fast-pass aggregate 성공과 mergeability를 별도로 확인한다.

## 7. 최종 판정

- 판정: 승인
- 검증 대상: code head `4280831d1f25a189416c2fcec14e0d252dfb90c3`와 동일 head의 5플랫폼
  Release Binary dry-run.
- merge 전 조건: 이 review 기록을 포함한 최신 trailing head의 GitHub Actions 성공, 최신 `devel` 대사,
  `MERGEABLE / CLEAN` 재확인과 메인테이너의 별도 merge 승인.
- 이 기록 자체는 GitHub review comment, 원격 push 또는 merge를 수행하지 않는다.
