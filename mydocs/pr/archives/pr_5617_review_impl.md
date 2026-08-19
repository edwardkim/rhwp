---
kind: pr-review-implementation
status: pending-ci
pr: 5617
---

# PR #5617 구현·통합 검토 기록

## 적용 순서

1. 최신 `upstream/devel`에서 `review/planet6897-excluding-5610-20260819`를 만들었다.
2. #5580, #5591, #5594, #5604, #5608, #5609 head를 순서대로 체리픽했다.
3. #5609의 source-side test 두 건을 `tests/cases/issue_5537_titlemark_run_ownership.rs` integration contract로 이관했다.
4. maintainer 보정을 `b250788c6` (`fix(test): titleMark 회귀를 integration suite로 이동한다`)으로 별도 고정했다.

## 보정 이유

저장소 정책은 새 source-side `#[cfg(test)]`를 금지한다. titleMark의 serializer 의미를 약화하지 않고, 공용 integration harness에서 run 경계의 소유·비소유 두 경우를 검증하도록 바꿨다.

## 산출물 정책

`rust-test-suite-manifest --prepare`가 만든 generated suite와 manifest는 review worktree 검증 산출물이며 PR에 stage하지 않았다.

## 검증 결과

- focused titleMark: 2 passed
- source test policy: 통과
- full integration nextest: 7,782 passed
- native-Skia lib: 58 passed
- Studio unit/build: 통과
- 회전 그림·Square 표 native-Skia PNG fixture: 확인

## 병합 후 체크리스트

1. 최신 head CI 성공을 확인한다.
2. #5617을 병합한다.
3. 원본 PR #5580, #5591, #5594, #5604, #5608, #5609와 완료 issue에 통합 결과를 댓글로 남기고 close한다.
4. `review/planet6897-excluding-5610-20260819` 원격·로컬 브랜치와 review worktree, 전용 target을 정리한다.
