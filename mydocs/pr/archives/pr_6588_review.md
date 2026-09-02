# PR #6588 검토 기록 — v0.8.6 기능 기준선 계측 범위 명확화

- PR: [#6588](https://github.com/edwardkim/rhwp/pull/6588)
- 이슈: [#6584](https://github.com/edwardkim/rhwp/issues/6584)
- 작성자·검토자: `edwardkim` collaborator self-review
- base: `devel@a5f3bf6950c2a282a06ea3e2303299236e38c707`
- 검토 대상 code head: `512f2d98455cea3a19374c6035fbb910fdc5182f`
- 규모: 4 files, +23 / -9, 1 commit
- 검토일: 2026-09-02 KST

## 1. 변경 범위

- 한·영 CHANGELOG와 GitHub Release note 초안에서 `2,214 commits`를 최종 릴리스 전체 commit 수가 아니라
  `v0.8.4..063041a2c` 기능 기준선의 고정 계측값으로 명시한다.
- 이후 버전·검증·릴리스 기록 commit은 위 계측 범위와 분리해, PR #6585 병합 뒤의 실제
  `main..devel` 범위와 공개 수치가 충돌하지 않게 한다.
- PR #6585 병합·후속 CI와 Stage R6 사전 대사 결과를 오늘할일에 기록한다.
- 제품 source, test, fixture, workflow, package manifest와 기여자 20명 집합은 바꾸지 않는다.

PR 본문은 #6584를 `Refs`로 연결한다. `devel -> main`, tag·GitHub Release와 공식 배포 채널 정산이
남았으므로 이 PR merge만으로 #6584를 닫지 않는 것이 맞다.

## 2. metadata와 base 정합

- 작성 시점 PR 상태: Open, non-Draft, `MERGEABLE / CLEAN`.
- PR head와 local·remote task branch는 exact code head와 일치했다.
- code head의 parent는 PR #6585 merge commit이자 최신 base인 `a5f3bf695`다.
- `upstream/main..code head`는 2,230 commits이며, 이 값은 기능 기준선 계측값 2,214와 의도적으로
  다르다. merge 전에는 최신 base·head·mergeability를 다시 확인한다.

## 3. 기록 정확성 검토

`git rev-list --count v0.8.4..063041a2ced54085b5cf94c2e646ac7aa0e1960d`는 2,214를 반환했다.
따라서 2,214를 “v0.8.4 이후 기능 기준선”으로 한정하고 이후 release-prep 기록 commit을 분리한 정정은
실제 계측 경계와 일치한다.

`CHANGELOG.md`, `CHANGELOG_EN.md`와 `mydocs/working/task_m100_6584_release_notes.md`는 모두
262개 PR provenance가 2,214-commit feature baseline에서 확인된 값이라는 동일한 의미를 유지한다.
릴리스의 기능 설명, 호환성, 기여자 목록과 버전은 바꾸지 않았다.

## 4. 렌더 영향과 시각 검증

제품 renderer/layout/typeset/paint, WASM API, Studio·확장 화면과 배포 manifest를 변경하지 않았다.
문구 정정만 있으므로 새 시각 검증은 필요하지 않다.

## 5. 검증

- exact code head의 PR-triggered check: 성공 14, 정책상 생략 15, 실패·대기 0.
- CI [run #33573796610](https://github.com/edwardkim/rhwp/actions/runs/33573796610), CodeQL
  [run #33573796581](https://github.com/edwardkim/rhwp/actions/runs/33573796581), Proptest
  [run #33573796584](https://github.com/edwardkim/rhwp/actions/runs/33573796584), Adapter inter-diff
  [run #33573796616](https://github.com/edwardkim/rhwp/actions/runs/33573796616)가 성공했다.
- CI의 Rust·frontend·archive·Native Skia·WASM heavy job은 변경 분류에 따라 생략되고 required
  aggregate가 성공했다. root CHANGELOG가 review-only 허용 경로가 아니므로 Proptest와 Adapter는
  fast-pass 대신 worker를 실행해 각각 성공했다.
- `git diff --check upstream/devel...512f2d98455cea3a19374c6035fbb910fdc5182f`가 통과했다.
- 변경 Markdown 4개의 내부 상대 링크 검사와 release channel·contributor 계약 19/19가 통과했다.
- 기준선 commit 수 2,214와 현재 `main..code head` commit 수 2,230을 독립 재계산했다.

PR은 제품 코드·테스트·workflow를 바꾸지 않고, PR #6585의 exact candidate와 광범위 검증 결과도 이미
존재한다. 따라서 이 self-review에서 Rust·WASM·Studio 전체 회귀를 중복 실행하지 않았다.

## 6. 발견 사항과 잔여 위험

- 차단 발견 사항은 없다.
- 이 정정은 전체 릴리스 commit 수를 새 정본 수치로 고정하지 않는다. 실제 `devel -> main` PR 직전
  최신 범위는 Stage R6에서 다시 대사해야 한다.
- #5949는 실제 Linux AArch64 asset 확인 뒤, #6243은 post-release Render Diff canary 성공 뒤,
  #6584는 필수 배포 채널과 기록 정산 뒤에만 닫는다.
- 이 review·오늘할일 trailing commit은 `mydocs/`만 바꾼다. push 뒤 latest head의 review-only
  fast-pass aggregate 성공과 mergeability를 별도로 확인한다.

## 7. 최종 판정

- 판정: 승인
- 검증 대상: code head `512f2d98455cea3a19374c6035fbb910fdc5182f`.
- merge 전 조건: 이 review 기록을 포함한 최신 trailing head의 GitHub Actions 성공, 최신 `devel` 대사,
  `MERGEABLE / CLEAN` 재확인과 메인테이너의 별도 merge 승인.
- 이 기록 자체는 GitHub review comment, 원격 push 또는 merge를 수행하지 않는다.
