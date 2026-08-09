---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4119 검토 - F5 셀 블록 직접 서식 적용

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4119](https://github.com/edwardkim/rhwp/pull/4119) / @jeong-sik |
| contributor 원 head | `2a874d79dc2cf0b5407c1b5d91a5c4ac5a98c08c` |
| 메인터너 보정 commit | `33dfd6e3a9425a3d25df8d62f5aa702e9d79443c` |
| 최신 `devel` 병합 | `5e66cf2c01a58d3135044176e4b36ff9fc80a0b6` |
| 가시성 검토 브랜치 | `review/jeong-sik-20260807` |
| 원 변경 범위 | `rhwp-studio`의 셀 블록 산출, 커서 제외 셀, 직접 글자·문단 서식, 회귀 테스트 |
| 시각 검증 | 정적 renderer 변경은 아니다. 2x2 표를 사용하는 실제 Chromium 동작으로 F5 선택, 서식 적용, 제외, Undo를 검증했다. |

작성자 @jeong-sik은 현재까지 merge된 rhwp PR이 없어, 이 PR이 첫 병합 기여 후보임을 확인했다.
동시에 열린 후속 PR은 있으나, 이 판정은 첫 병합 기여 여부와 이 PR의 검증 범위를 바꾸지 않는다.

원 contributor 변경은 F5 셀 블록 선택이 텍스트 selection과 별도 축이라는 점을 반영한다. 선택 범위의
셀 산출과 Ctrl+클릭 제외 키를 `cell-block-format.ts`로 모으고, 문단 서식은 블록 전체 문단을 대상으로,
글자 서식은 블록 선택도 명시적 적용 대상으로 처리한다. 따라서 정렬·줄 간격뿐 아니라 굵게, 기울임,
밑줄, 글꼴 크기, 장평, 자간이 같은 대상 범위를 사용한다.

## 발견 사항과 메인터너 보정

원 PR은 Ctrl+클릭으로 블록의 모든 셀을 제외하면 `getSelectedCellBlock()`이 `null`을 반환했다. 이후
문단 서식 경로는 이를 일반 커서 경로로 해석해 앵커 셀 하나에 적용했다. 2x2 표에서 네 셀을 모두 제외한
뒤 가운데 정렬을 적용하면 `(0,0)` 셀만 `justify`에서 `center`로 바뀌는 것을 실제 브라우저에서 재현했다.

메인터너 보정 `33dfd6e3`은 빈 블록을 `null`과 구분한다. 빈 `cellIndices`는 문단 서식 대상 목록을
빈 배열로 유지하고, 글자 서식은 history를 만들지 않는 no-op으로 반환한다. 이로써 사용자가 명시적으로
제외한 셀에 fallback으로 서식을 쓰지 않는다. 빈 블록 문단·글자 서식과 undo history 회귀 세 건을
추가했다.

최신 `upstream/devel` `5119ea498`은 원 PR의 공통 조상이 아니었다. 사용자 지시에 따라 보정 뒤에
`5e66cf2c`로 최신 `devel`을 병합했고, 충돌 없이 PR 변경과 현재 기준선을 함께 보존했다. contributor
원 commit은 rebase, amend, reset, force-push하지 않았다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| focused Studio test | `node --test tests/cell-block-format.test.ts` 16 passed |
| TypeScript | `npx tsc --noEmit` 통과 |
| Studio 전체 단위 테스트 | `npm test` 779 passed, 0 failed |
| 최신 `devel` 병합 simulation | conflict 없이 완료, `git diff --check` 통과 |
| Studio production build | `npm run build` 통과. Vite chunk 크기 경고만 있고 exit 0 |
| 브라우저: 정상 F5 블록 | 2x2 전 셀에 굵게·가운데 정렬이 적용되고 Undo 두 번으로 네 셀이 모두 원복됨 |
| 브라우저: 전체 Ctrl+클릭 제외 | 정렬·굵게 적용 뒤 네 셀 값이 모두 불변이고 `canUndo()`도 false로 유지됨 |

브라우저 검증은 최신 `devel` merge simulation tree에서 수행했다. 이 변경은 input/command 대상 선정과
history 제어만 다루며 Canvas paint, pagination, HWP/HWPX fixture, 기준 PDF를 변경하지 않으므로 별도
PDF/SVG 시각 sweep 대상은 아니다.

## 수용 판단과 merge 조건

**메인터너 보정 포함 수용 권고.** F5 셀 블록의 정상 적용과 의도적으로 비운 블록의 무변경 계약을 모두
회귀·브라우저 수준에서 확인했다. `33dfd6e3` code head 뒤에 current-base merge와 single-parent review
기록을 추가했으므로, [#4102](https://github.com/edwardkim/rhwp/pull/4102)의 current-base review-only
fast-pass 조건으로 같은 PR/source의 녹색 code candidate를 재사용할 수 있다. 후보 조회가 실패하거나
자동 merge tree가 일치하지 않으면 workflow는 full CI로 fallback해야 한다.

문서 작성 시점의 원격 상태는 변동 가능하다. merge 전에는 `33dfd6e3` 후보의 CI·CodeQL·Render Diff와
문서 push 뒤 최신 head의 preflight·branch protection aggregate가 모두 성공했는지,
`mergeable=MERGEABLE` 및 `mergeStateStatus=CLEAN`인지, 그리고 작업지시자의 merge 승인이 있는지를
다시 확인한다.
