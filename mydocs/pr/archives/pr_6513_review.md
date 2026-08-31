---
kind: pr-review
status: approved-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6513
author: jeong-sik
---

# PR #6513 검토 기록 - 글뒤로/글앞으로 전면 그림의 #1995 후보 및 분모 정렬

## 범위

- 대상: [#6513](https://github.com/edwardkim/rhwp/pull/6513), code head
  `ef8b0b66d8ef14d92015b1471d6151553af12b2a`, base `devel`.
- 변경은 `src/renderer/typeset.rs`의 그림 낱장 배치 후보 및 전면 그림 과반 분모와,
  `tests/cases/issue_6511_behindtext_background_fullpage_exclusion.rs`의 HWP5 왕복 계약이다.
- PR 본문의 `Closes #6511`은 병합 뒤 실제 자동 종료 여부를 다시 확인한다.

## 검토 근거와 판정

- [#703](https://github.com/edwardkim/rhwp/issues/703)과
  [#1955](https://github.com/edwardkim/rhwp/issues/1955)는 `BehindText`와
  `InFrontOfText` 개체를 본문 플로우에서 제외하는 기존 불변식을 확정했다.
- [#4654](https://github.com/edwardkim/rhwp/issues/4654)는 #1995 낱장 배치의 전면 그림
  과반 정책과 `noninline_pic_count` 모집단의 경계를 다룬 선행 실측이다. 이번 변경은
  새 휴리스틱이 아니라 후보 필터와 분모를 같은 flow 참여 그림 모집단으로 정렬한다.
- 새 계약은 배경 전면 그림 2장, 기존 Square 전면 그림 2장, 두 경우의 혼합을 각각
  HWP5 직렬화 왕복 뒤 1쪽, 3쪽, 3쪽으로 고정한다. 기존 #703/#1955/#4654 근거와 함께
  변경 범위를 충분히 판정할 수 있으므로 별도 private 원문 PDF 또는 visual sweep을
  수용 조건으로 요구하지 않는다.
- **판정: 승인.** 차단 finding과 메인터너 코드 보정은 없다. 이 문서 trailing commit으로
  PR head가 바뀌므로, 병합 전 새 head의 required CI와 `MERGEABLE/CLEAN`만 다시 확인한다.

## 검증 기록

- code head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33353127284),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33353127252),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33353127093),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/33353127199),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33353127190)가 성공했다.
- CI의 Build & Test, Lint, Native Skia와 archive worker는 성공했고, 경로 정책상 WASM 및
  frontend gate는 expected skip이다. CodeQL Rust, JavaScript/TypeScript, Python worker도 성공했다.
- 이 검토에서는 코드를 변경하거나 로컬 회귀를 다시 실행하지 않았다. 위 CI는 code head에 대한
  실제 실행 결과이며, trailing 문서 head의 결과와 혼동하지 않는다.
