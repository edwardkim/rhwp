---
kind: pr-review
status: merged-post-merge-record
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6522
author: jangster77
---

# PR #6522 병합 후 검토 기록 - Dependabot 12건 통합 및 quick-xml 0.42 보정

## 범위와 통합 이력

- 통합 PR: [#6522](https://github.com/edwardkim/rhwp/pull/6522), base `devel`, 작성자
  `jangster77`.
- Dependabot 원 PR [#6499](https://github.com/edwardkim/rhwp/pull/6499)부터
  [#6510](https://github.com/edwardkim/rhwp/pull/6510)까지 12건을 최신 `devel` 위에
  `cherry-pick -x`로 누적했다. 원 source branch는 보존한다.
- Studio lockfile 충돌은 `@types/chrome`와 `puppeteer-core`의 두 Dependabot 갱신을 모두
  유지하도록 해소했다.
- `quick-xml` 0.42의 문자열 기반 XML API에 맞춘 HWPX, HML, OOXML 및 암호화 파서 호환 보정은
  maintainer commit `f7ebe3366eb41eb068edf2b4cfe9de865227ddad`에 반영했다.
- 병합 commit은 `15ea9ab0a8aaa09cd6692b1700f73fe14a308e20`이며, PR 본문에는 closing
  issue keyword가 없다.

## 검증

- code head `f7ebe3366eb41eb068edf2b4cfe9de865227ddad`의 PR CI는
  [CI](https://github.com/edwardkim/rhwp/actions/runs/33368865465),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33368865184),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33368865017),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/33368865305),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33368865140)가 모두 성공했다.
- 로컬에서는 Rust format, manifest, native/wasm clippy, workspace build와 all-target clippy를
  통과했고, `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review
  --tests --test-threads 8 --no-fail-fast`는 8,862 passed, 46 skipped였다.
- 격리된 Studio `npm ci`, test 1,324건과 production build, Chrome extension build, VS Code
  extension typecheck 및 webpack compile을 통과했다. Studio production dependency audit은
  취약점 0건이었다.
- merge commit의 devel push [CI](https://github.com/edwardkim/rhwp/actions/runs/33370035962)와
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33370035915)는 성공했다. Rust CodeQL
  worker [job 99418880973](https://github.com/edwardkim/rhwp/actions/runs/33370035915/job/99418880973)도
  성공했다.
- devel이 후속 PR #6461의 `cfa4ccacab63b470771720ebed33503cdd62adb6`으로 전진하면서
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/33370035936)와
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33370036003)의 aggregate는 취소됐다.
  그러나 실제 `adapter inter-diff` worker와 `prop roundtrip` worker는 각각 성공했으며, 이는
  테스트 실패가 아니라 superseding `devel` push에 따른 controller 취소다.

## 시각 증적과 판단

- parser/의존성 호환과 lockfile을 다루는 통합이며 신규 fixture, 기준 PDF, renderer golden 또는
  제품 UI 시각 계약을 추가하지 않는다. 따라서 새 visual sweep asset은 만들지 않았다.
- PR head의 Render Diff 성공은 renderer 회귀 탐지 결과로만 기록하며, 이 기록은 별도 PDF/SVG
  충실도 수용 근거를 주장하지 않는다.
- **수용 완료.** 원 Dependabot PR은 직접 merge하지 않고 #6522의 provenance-preserving 통합으로
  수용한다. 원 PR별 후속 comment와 close는 이 문서 기록 PR이 `devel`에 반영되고 그 검증이 끝난 뒤
  한 번만 처리한다.
