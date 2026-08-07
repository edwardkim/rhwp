---
kind: review-implementation
status: completed-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4119 메인터너 보정 기록

## commit 경계

| 순서 | SHA | 역할 |
| --- | --- | --- |
| 1 | `ea0d4feace2b8df271ab883061f60681d676233b` | 셀 블록 산출과 Ctrl+클릭 제외 키를 공통 helper로 정리 |
| 2 | `0d95fd271f128adc31bca7ca11ec42e7b4e7be07` | 셀 블록 전체 문단 서식 적용 |
| 3 | `2a874d79dc2cf0b5407c1b5d91a5c4ac5a98c08c` | 셀 블록 글자 서식 적용 |
| 4 | `33dfd6e3a9425a3d25df8d62f5aa702e9d79443c` | 빈 블록의 앵커 셀 fallback 차단과 회귀 세 건 추가 |
| 5 | `5e66cf2c01a58d3135044176e4b36ff9fc80a0b6` | 사용자 지시에 따른 최신 `upstream/devel` 병합 |

`maintainerCanModify=true`를 확인한 뒤 `review/jeong-sik-20260807` 가시성 브랜치에서만 보정을
추가했다. 보정 시작 시 PR head와 fork 원격은 모두 contributor 원 head `2a874d79`였고, contributor의
세 commit은 수정하거나 재작성하지 않았다.

## 보정 내용

1. `getSelectedCellBlock()`이 전체 제외 상태를 일반 커서 상태와 혼동하지 않도록 빈
   `cellIndices` 블록을 반환하게 했다.
2. 문단 서식은 빈 셀 블록에서 빈 대상 목록을 받아 어느 문단도 변경하지 않는다.
3. 글자 서식은 빈 셀 블록에서 command/history를 만들지 않고 no-op으로 종료한다.
4. 전체 제외, 문단 서식 fallback 차단, 글자 서식 history 없음의 단위 회귀를 추가했다.

## 완료한 검증

| 항목 | 결과 |
| --- | --- |
| `node --test tests/cell-block-format.test.ts` | 16 passed |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 779 passed, 0 failed |
| 최신 `devel` merge simulation | conflict 없음, `git diff --check` 통과 |
| `npm run build` | 통과, Vite chunk 크기 경고만 존재 |
| Chromium F5 정상 블록 | 굵게·가운데 정렬 4/4 적용, Undo로 4/4 원복 |
| Chromium 전체 제외 블록 | 값 변경 0건, undo history 생성 0건 |

## 원격 반영과 다음 조건

보정 `33dfd6e3`은 contributor source `fix/cell-block-format`에 fast-forward push했다. 이번
`upstream/devel` 병합과 review·오늘할일 기록을 같은 source branch에 추가한 뒤 다시 push한다.
`33dfd6e3`의 같은 PR/source CI·CodeQL·Render Diff가 녹색이고 current-base merge tree가 일치하면,
뒤의 문서-only commit은 #4102 current-base fast-pass aggregate를 재사용할 수 있다. 그렇지 않으면
workflow의 full CI fallback을 확인한다. 어느 경로이든 최신 mergeability와 작업지시자 승인 뒤에만
병합과 후속 정리를 수행한다.
