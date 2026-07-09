# 최종 보고서 — Task M100 #2023: 프론트 웹 리팩터링 계획 수립

- 이슈: #2023
- 상위 이슈: #2022
- 작성일: 2026-07-07
- 브랜치: `local/task2023`
- 기준 커밋: `upstream/devel` `a23b8ae1c85fc80547c40d8e7bbbf37a07283468`
- 문서 PR 기준 커밋: 2026-07-09 fetch 기준 `upstream/devel` `3728abfb`
- 상태: 계획 수립 완료, maintainer/collaborator 리뷰 요청
- GitHub 리뷰 요청 댓글: https://github.com/edwardkim/rhwp/issues/2023#issuecomment-4901359667
- GitHub 명시적 멘션 리뷰 요청 댓글: https://github.com/edwardkim/rhwp/issues/2023#issuecomment-4901538662
- v2 보완 계획서: `mydocs/plans/task_m100_2023_frontend_refactoring_plan_v2.md`
- GitHub v2 리뷰 요청 댓글: https://github.com/edwardkim/rhwp/issues/2023#issuecomment-4917066612
- GitHub 문서 전용 PR: https://github.com/edwardkim/rhwp/pull/2080

## 1. 수행 결과

#1883의 Rust 리팩터링 접근 방식과 동일하게, 프론트 웹 계층도 **SOLID + 복잡도** 2축으로
진단하고 실행 이슈를 분리할 수 있도록 계획을 수립했다.

이번 이슈에서는 코드 변경을 하지 않았고, 다음 산출물만 작성했다.

- 수행계획서: `mydocs/plans/task_m100_2023.md`
- 공개 계약·금지 목록 인벤토리: `mydocs/tech/task_m100_2023_frontend_contract_guardrails.md`
- SOLID·복잡도 재진단: `mydocs/tech/task_m100_2023_frontend_diagnosis.md`
- 프론트 웹 리팩터링 계획 초안: `mydocs/plans/task_m100_2023_frontend_refactoring_plan.md`

## 2. 핵심 결론

프론트 웹 리팩터링은 진행할 가치가 있다. 다만 `@rhwp/editor` 하나의 문제가 아니라
`rhwp-studio`, `/web` legacy, `web/fonts`, 브라우저 확장, VS Code extension, npm 공개 계약이
함께 얽힌 구조 문제로 보는 것이 맞다.

따라서 실행 순서는 다음처럼 제한하는 것이 적절하다.

1. 공개 계약과 회귀 baseline을 먼저 고정한다.
2. `/web` legacy 삭제보다 `web/fonts` canonical 이전을 먼저 분리한다.
3. `rhwp-studio` 고복잡도 함수는 `InputHandler` typed context와 dialog state/apply 분리부터
   시작한다.
4. 확장·VS Code·npm은 구조 미화보다 보안·배포 계약 유지가 우선이다.
5. 실행 이슈는 Phase 단위로 작게 만들고, 각 Phase 완료 후 재측정·승인을 받는다.

## 3. 현재 baseline

예비 진단 기준 커밋은 `a23b8ae1c85fc80547c40d8e7bbbf37a07283468`이다.

| 항목 | 측정값 | 의미 |
|------|-------:|------|
| `rhwp-studio/src` 코드 LOC | 53,525 | 대형화 |
| `rhwp-studio/src` 1,200 LOC 초과 파일 | 10 | 해체 대상 |
| `rhwp-studio/src` 2,000 LOC 초과 파일 | 3 | 최상위 해체 대상 |
| `rhwp-studio/src` CC>25 함수 | 49 | 복잡도 집중 |
| `rhwp-studio/src` 최대 CC | 321 | `onKeyDown` |
| legacy `/web` 코드 LOC(font 제외) | 4,550 | 삭제/보존 판단 필요 |
| `rhwp-shared` CC>25 함수 | 0 | 공통화 기반은 양호 |
| `@rhwp/editor` 코드 LOC | 241 | 리팩터링 타깃보다 public guardrail |

SOLID 예비 점수는 **54/100**으로 평가했다.

| 원칙 | 예비 점수 | 핵심 근거 |
|------|----------:|-----------|
| S | 8/20 | God coordinator/function 다수, `/web`와 font asset ownership 혼재 |
| O | 12/20 | asset/font/build 변경 시 studio/extension/VS Code/npm 동시 수정 |
| L | 14/20 | renderer/e2e 계약은 있으나 embed/webview/font 계약 문서화 부족 |
| I | 8/20 | `core/types.ts`, `wasm-bridge.ts`, `this: any` 표면 과대 |
| D | 12/20 | thin wrapper는 좋지만 asset/platform detail 의존이 강함 |

## 4. 실행 전 금지 목록 후보

계획 승인 전에는 아래 작업을 하지 않는 것이 적절하다.

- `/web` 전체 삭제.
- `web/fonts` 위치 변경.
- `rhwp-studio/public/fonts` symlink 변경.
- Chrome/Firefox/Safari/VS Code 빌드 경로 변경.
- `@rhwp/editor` iframe/postMessage 계약 변경.
- `@rhwp/editor` runtime dependency 추가.
- React/Vue/Svelte 등 runtime UI framework 도입.
- 확장 CSP 완화.
- inline script 추가.
- `web_accessible_resources` 확대.
- sender/URL/file signature/size 검증 약화.
- 폰트 fallback 변경.
- 저작권 보호 대상 한컴/MS 폰트 번들.
- 대형 모듈 분해와 버그 수정 혼합.

## 5. 권장 실행 Phase

### Phase 0 — Frontend Baseline Freeze

실행 전 정량 복잡도, public contract, font asset inventory, extension security snapshot,
studio render/e2e smoke 기준을 먼저 고정한다.

### Phase A — `/web` font ownership 정리

`web/fonts` canonical 위치를 먼저 확정하고, studio/extension/VS Code/npm 문서와 빌드 경로를
따로 갱신한다. `/web` legacy 삭제는 이후 별도 이슈에서 판단한다.

### Phase B — `rhwp-studio` 복잡도 해체

우선순위는 `InputHandler` typed context, keyboard/mouse 고복잡도 함수 해체, dialog state/apply
분리, `WasmBridge` typed adapter, `diff-engine` 책임 분리 순서가 적절하다.

### Phase C — 확장·VS Code·npm 계약 정리

브라우저 확장은 CSP/security guardrail 유지가 최우선이며, VS Code webview와 npm 패키지는
공개 계약 문서화와 smoke 기준을 분리한다.

### Phase D — 재측정과 6차 프론트 리뷰

실행 Phase 완료 후 SOLID 점수와 복잡도 지표를 재측정하고 다음 사이클을 결정한다.

## 6. 리뷰 요청 사항

maintainer/collaborator에게 다음 항목의 리뷰를 요청한다.

1. #1883과 동일하게 SOLID + 복잡도 2축을 프론트 리팩터링 거버넌스로 적용해도 되는가.
2. 예비 SOLID 점수 54/100과 S/I 우선순위 판단이 타당한가.
3. `/web` legacy 삭제보다 `web/fonts` canonical 이전을 먼저 하는 순서가 맞는가.
4. `web/fonts` 새 canonical 위치는 `rhwp-studio/public/fonts` 실체화와 공통 `assets/fonts`
   신설 중 어느 쪽이 적절한가.
5. React/Vue/Svelte 등 runtime UI framework 도입 금지를 이번 리팩터링 금지 목록에 명시해도
   되는가.
6. `@rhwp/editor` iframe/무의존 계약을 public contract로 고정해도 되는가.
7. 확장 CSP/`web_accessible_resources`/sender·URL 검증 guardrail을 실행 PR 체크리스트에
   넣어도 되는가.
8. 실행 하위 이슈는 Phase 0과 Phase A만 먼저 만들고, Phase B 이후는 재평가 후 분리하는
   방식이 적절한가.

## 7. 다음 단계

리뷰를 받은 뒤 바로 모든 실행 이슈를 만들기보다, 먼저 다음 두 종류만 분리하는 것이 적절하다.

- `[프론트] baseline freeze: 계약·렌더·확장·폰트 기준선 고정`
- `[프론트] web/fonts canonical 위치 이전 계획 및 경로 갱신`

Phase B 이후의 대형 모듈 해체 이슈는 Phase 0/A 결과와 리뷰 피드백을 반영한 뒤 생성한다.

## 8. 검증

이번 작업은 계획 문서 작성만 포함하므로 빌드와 테스트는 실행하지 않았다.

확인 항목:

- 작업 브랜치 `local/task2023`는 최신 `upstream/devel` 기준으로 생성됨.
- 계획 산출물은 문서 파일만 추가/수정함.
- 코드, 빌드 스크립트, 패키지 manifest, 확장 manifest는 변경하지 않음.

## 9. 리뷰 반영 v2 진행

2026-07-09에 maintainer/collaborator 리뷰를 반영해 v2 계획서를 작성했다.

v2 결정 사항:

- TS 복잡도 측정 도구는 `eslint` + `sonarjs/cognitive-complexity`를 우선 후보로 둔다.
- v1 수치는 공식 baseline이 아니라 heuristic baseline으로 격하한다.
- 2026-07-09 fetch 결과 `upstream/devel`은 `3728abfb`까지 전진했으므로, Phase 0 공식 baseline은
  최신 upstream 기준으로 재측정한다.
- font canonical은 공통 `assets/fonts` 신설 방향으로 둔다.
- 금지 목록 일부를 후보가 아니라 확정 규칙으로 승격한다.
- 3-browser sync 규칙과 stage-gate를 명시한다.
- v2 승인 전 실행 하위 이슈는 만들지 않는다.
- v2 승인 후에도 우선 Phase 0 baseline freeze와 Phase A `assets/fonts` canonical 이슈만 만든다.
- #2023에 v2 리뷰 요청 코멘트를 게시했다.
- GitHub에서 리뷰 가능한 문서 전용 PR #2080을 생성했다.
