# Task M100 #2392 최종 보고서 - 그림 속성 적용 파이프라인 책임 분리

- 이슈: #2392
- 상위 추적: #2022
- 브랜치: `issue-2392-picture-props-apply-pipeline`
- 기준: `upstream/devel@af5902b659be9a4d86ad458d79c63353dba88167`
- 구현 source: `deb87ad98a5e76b86f486333d22bc518091265f6`
- 작성일: 2026-07-19
- 상태: local 구현·검증 완료, 원격 게시 승인 대기

## 1. 결론

`PicturePropsDialog.handleOk`가 함께 소유하던 form 수집, patch 계산, target 판정, WASM mutation과
undo/fallback 책임을 current behavior를 유지한 채 분리했다. `handleOk`는 CC 348/381 LOC에서 CC 2/11 LOC로
축소됐고 대상 dialog/model 합산 Total CC는 647에서 371로 감소했다.

same-base frontend 전체 Total CC, Top 20 합과 CC>25 합도 각각 276, 301, 348 감소했다. 따라서 복잡도를
작은 함수로 단순 이동한 리팩터링으로 보지 않는다. Studio unit/build와 실제 Chrome undo E2E를 모두 통과해
draft PR review를 요청할 수 있는 상태다.

## 2. 계획 대비 결과

| 계획 | 결과 |
|------|------|
| field/default/diff 정책 characterization | 10개 책임군과 object type별 계약 고정 |
| pure patch 계산 경계 | internal model과 20개 data-driven fixture로 분리 |
| target routing | 5종 discriminated union과 header/footer 우선 fixture 포함 6개 검증 |
| WASM mutation | dialog adapter에 기존 setter 5회와 인자 순서 유지 |
| undo/fallback | snapshot `objectProps`, cursor return, fallback event와 empty patch guard 유지 |
| 얇은 coordinator | `handleOk` CC 2/LOC 11 |
| public/UI/WASM 보존 | package/barrel/CSS/DOM/core/Rust/generated binding diff 0 |
| 정량 결산 | same-base aggregate와 stable function diff 공개 |
| browser 회귀 | 그림 속성 undo·실키 Ctrl+Z·Through 포함 Chrome headless PASS |

## 3. 구현 경계

### 3.1 internal pure model

`rhwp-studio/src/ui/picture-props-apply-model.ts`가 grouped raw form snapshot을 받아 최소 patch를 만들고,
object type과 위치 context를 받아 5종 apply target을 반환한다. runtime import는 없고 core type만
`import type`으로 사용한다.

계산 helper는 common size/position, transform, outer margin, caption, line, fill/shadow, image geometry/effect로
나뉜다. 기존 `||`/nullish default, diff-only/always-send, OLE 제한, `TakePlace` 변환, `Through` 보존과
header/footer 우선순위를 바꾸지 않았다.

### 3.2 dialog side effect

dialog는 DOM control을 `PicturePropsApplyForm`으로 수집하고 pure model을 호출한다. target별 실제 WASM
setter 5회는 `applyPropertyPatchToWasm`에 남겨 mutation ownership을 새 파일로 확장하지 않았다.
`applyPropertyPatch`는 target을 한 번 snapshot한 뒤 기존 command history 또는 no-service fallback을 실행한다.

### 3.3 범위 밖

`populateFromProps`, dialog UI/DOM/CSS, core type와 WASM schema, package/extension/embed 계약, renderer와
fonts는 변경하지 않았다. React/Vue/Svelte 등 UI framework와 runtime dependency도 추가하지 않았다.

## 4. 테스트

| 구분 | 결과 |
|------|------|
| pure patch fixture | 20/20 PASS |
| target fixture | 6/6 PASS |
| Stage 3 focused model/undo/Through/mutation | 38/38 PASS |
| Studio 전체 unit | 390/390 PASS |
| Studio production build | PASS |
| Chrome headless undo contracts | 6개 시나리오 PASS |
| mutation setter/인자 source guard | 5/5 PASS |
| command failure collection | 0 |
| `git diff --check` | PASS |

별도 visual sweep은 실행하지 않았다. DOM/CSS/rendering diff가 없고, 실제 dialog apply 경로와 screenshot은
기존 undo E2E가 검증한다는 계획상 판정에 따른 것이다.

## 5. 복잡도 결산

### 5.1 #2392 직접 delta

| 지표 | pre | post | delta |
|------|----:|-----:|------:|
| Total CC | 12,369 | 12,093 | -276 |
| Top 20 합 | 2,660 | 2,359 | -301 |
| CC>25 개수 / 합 | 70 / 4,297 | 69 / 3,949 | -1 / -348 |
| CC>100 개수 | 7 | 6 | -1 |
| Max CC | 453 | 453 | 0 |
| target dialog/model Total CC | 647 | 371 | -276 |
| target CC>25 개수 / 합 | 2 / 560 | 1 / 212 | -1 / -348 |
| `handleOk` CC / LOC | 348 / 381 | 2 / 11 | -346 / -370 |

stable `changed` function은 `handleOk` 하나이고 대상 밖 stable changed function은 0이다. 새 model Max CC는
13이며 신규 CC>25 함수가 없다. Total CC, Top 20과 threshold 합을 함께 사용한 결과는 #1904/#2130 이후
maintainer가 공식화한 복잡도 이동 방지 원칙을 반영한다.

### 5.2 official baseline

#2124 official snapshot 대비 현재 repository 누적치는 Total CC +288, Top 20 -222, CC>25 합 +17이다.
이는 #2124 이후 다른 frontend merge와 legacy `/web` 제거를 포함하므로 #2392의 성과나 회귀로 귀속하지
않는다. #2392 완료 판정은 같은 `af5902b6` base의 pre/post만 사용했다.

## 6. SOLID 판정

SOLID 평가는 frontend 전체 점수가 아니라 apply pipeline diff를 대상으로 한다.

- SRP: form capture, 계산 정책, target routing, mutation과 undo orchestration의 변경 이유를 분리했다.
- DIP: 계산 정책은 DOM/WASM/EventBus/CommandServices runtime detail 없이 검증된다.
- OCP: field와 target 규칙은 제한된 model/union에서 추가·검토할 수 있다.
- ISP: 새 함수에 dialog 전체나 거대 runtime context를 전달하지 않는다.
- LSP: 명시된 substitutable contract가 없어 임의 점수를 부여하지 않는다.

`solid_scoring_guide.md`를 review 질문에 사용했지만 자체 총점은 만들지 않았다. 공식 점수는 reviewer가 동일한
평가 단위와 근거를 합의할 때만 기록한다.

## 7. 완료 조건

- [x] apply field/target/undo characterization
- [x] patch 계산과 side-effect orchestration 분리
- [x] `populateFromProps`와 범위 밖 기능 diff 0
- [x] target/global complexity 정량 조건 충족
- [x] stable function diff로 단순 이동이 아님을 확인
- [x] Studio unit/build와 Chrome headless undo contracts
- [x] public/WASM/package/runtime dependency/UI 무변동
- [x] 단계별 문서와 최종 보고서
- [ ] 작업지시자 승인 후 push·draft PR·GitHub 진행 댓글
- [ ] maintainer/collaborator review와 PR CI
- [ ] merge 후 #2392/#2022 결산과 close 승인

## 8. 잔여 위험과 다음 판단

`populateFromProps` CC 212는 남아 있지만 이번 변경의 회귀 위험을 낮추기 위해 제외한 결정이 유효하다.
Stage 1에서 확인한 omitted/stale control behavior도 기능 정책 변경 없이 보존했다. 두 항목은 이 PR의 후속
수정으로 자동 확정하지 않고 #2022에서 다른 hotspot과 위험·효과를 비교한다.

PR 생성 전 `devel`이 움직이면 대상 중첩과 product 영향부터 확인한다. 관련 없는 문서 변경마다 전체 gate를
반복하지 않으며, 대상 source나 WASM/browser behavior가 달라질 때만 통합 후 proportional gate를 다시
실행한다.
