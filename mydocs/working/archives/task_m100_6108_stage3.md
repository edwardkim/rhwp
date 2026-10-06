---
kind: working
status: complete
issue: 6108
stage: 3
last_verified: 2026-08-28
---

# Task M100 #6108 Stage 3 완료 보고 — 통합 검증과 PR review 보정

- **이슈**: [#6108](https://github.com/edwardkim/rhwp/issues/6108)
- **PR**: [#6289](https://github.com/edwardkim/rhwp/pull/6289)
- **브랜치**: `codex/issue-6108-zoom-fit`
- **통합 기준**: `upstream/devel@94ff48d2b8`
- **선행 Stage**: Stage 1 계산 계약, Stage 2 명령·UI 진입점 단일화

## PR review에서 확인한 정확성 결함

여러 쪽을 고르면 대화상자는 지정한 `columns×rows` 전체를 쪽 맞춤으로 계산하지만 비율 라디오는
비활성화된다. 기존 확인 경로는 비활성 라디오에 남아 있던 선택값을 `zoomFitMode`로 저장해 다음 문제가
있었다.

- 프리셋·사용자 정의가 남아 있으면 계산된 배율은 전체 맞춤이어도 저장 규칙은 `none`이 됐다.
- 새 세션이나 다른 쪽 크기의 문서를 열 때 지정 배열 전체 맞춤을 다시 계산할 수 없었다.

`resolveZoomDialogFitMode()`를 계산 resolver 옆에 두어 여러 쪽은 비활성 비율 선택과 무관하게
`fitPage`를 저장하고, 다른 배치는 기존 비율 선택 규칙을 유지하도록 보정했다. 상태 표시줄의
`is-neutral` class는 동적 토글을 제거한 뒤 HTML literal만 남은 죽은 상태였으므로 함께 제거하고 회귀
검사를 추가했다.

## 회귀 계약

단위 테스트는 다음을 고정한다.

- 여러 쪽 2×2와 4×1은 프리셋·사용자 정의 선택값과 무관하게 `fitPage`를 저장한다.
- 한 쪽의 사용자 정의 배율은 기존처럼 `none`을 저장한다.
- HTML과 런타임 모두 소비되지 않는 `is-neutral` 상태를 만들지 않는다.

Chrome E2E는 기존 배치별 맞춤 계약에 다음 네 assertion을 추가했다.

1. 여러 쪽 2×2 확인 직후 `zoomFitMode=fitPage`
2. 새 세션에서 2×2 배치 복원
3. 새 세션에서 `fitPage` 규칙 복원
4. 다른 쪽 크기의 문서를 열 때 2×2 전체 맞춤 재계산

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| focused unit (`zoom-fit-mode-persistence`, `zoom-fit`) | 20 pass / 0 fail |
| Studio 전체 `npm test` | 1,226 tests, 1,225 pass / 1 skip / 0 fail |
| `npm run build` | TypeScript·Vite production build 통과, 238 modules |
| Chrome `e2e:zoom-fit-mode` | 32/32 assertion 통과 |
| `git diff --check` | 통과 |

E2E 실측은 자동 폭 맞춤 `1.537`, 한 쪽 쪽 맞춤 `0.640`, 두 쪽·맞쪽 폭 맞춤 `0.762`, 여러 쪽
2×2 전체 맞춤 `0.315`였다. 새 문서에서는 저장된 2×2 `fitPage` 규칙으로 `0.446`을 다시 계산했다.

로컬 HTML 보고서는 `output/e2e/zoom-fit-mode-persistence-report.html`, 화면은
`rhwp-studio/e2e/screenshots/issue-6108-*.png`에 생성했다. 둘 다 재현 가능한 gitignore 산출물이며 PR
source에는 포함하지 않는다.

## 범위와 다음 단계

- 이 Stage는 #6108의 계산·저장 정확성과 사용되지 않는 상태 정리만 다룬다.
- 입력 오류 표시와 원자 view-settings transaction은 상위 stacked PR
  [#6290](https://github.com/edwardkim/rhwp/pull/6290), 이슈 #6109에 유지한다.
- GitHub Actions 완료 판정은 최신 stack head push 뒤 작업지시자가 확인한다.
