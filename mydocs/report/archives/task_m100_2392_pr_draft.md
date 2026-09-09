# Task M100 #2392 GitHub 게시 초안

- 작성일: 2026-07-19
- 상태: local Stage 4 완료, commit/push/PR/comment 승인 대기
- 원칙: 승인 전 push, PR 생성, GitHub comment와 issue body 편집을 수행하지 않는다.

## 1. 권장 게시 순서

1. Stage 4 문서 commit과 local clean 상태 확인
2. 작업지시자에게 아래 PR 본문·댓글 초안 승인 요청
3. 승인 후 branch push와 draft PR 생성
4. 실제 PR 번호·head SHA로 placeholder 치환
5. #2392 진행 댓글과 maintainer review 요청 댓글 게시
6. #2022에는 중복 상세 대신 실행 PR 진입 사실과 결산 기준만 게시
7. CI와 review 반영 뒤 Ready for review·merge는 별도 판단
8. merge 후 완료 근거를 갱신하고 작업지시자 승인 후 #2392 close

## 2. PR 제목 초안

```text
[프론트] 그림 속성 적용 파이프라인 책임 분리
```

## 3. PR 본문 초안

```md
## 요약

- `PicturePropsDialog.handleOk`의 form 수집, patch 계산, target 판정, WASM mutation과 undo orchestration을 분리했습니다.
- field/default/diff/always-send 정책을 pure internal model과 20개 data-driven fixture로 고정했습니다.
- shape/cell, shape/body, image/header-footer, image/cell, image/body 5개 target과 기존 setter 인자 순서를 유지했습니다.
- snapshot undo, no-service fallback, empty patch와 `Through` 보존을 기존 계약대로 유지했습니다.

Closes #2392
Related #2022

## 변경 구조

| 경계 | 책임 |
|------|------|
| `picture-props-apply-model.ts` | grouped form snapshot -> minimal patch, 5종 pure target resolver |
| `picture-props-dialog.ts` | DOM form capture, 기존 WASM setter 5회, snapshot/fallback orchestration |
| model test | object type별 patch 20개와 target 6개 fixture |
| undo test | empty patch, snapshot/fallback, setter method·인자 순서와 mutation surface guard |

model은 runtime import가 없고 core type만 `import type`으로 사용합니다. package/barrel re-export도 추가하지
않아 internal UI ownership을 유지했습니다.

## 보존한 계약

- image/shape/line/group/OLE별 field와 default·diff/always-send 의미
- `TakePlace` -> `TopAndBottom`, `Through` 보존과 숫자·색·alpha 변환
- image에서 header/footer가 cell path보다 먼저 선택되는 우선순위
- `kind: snapshot`, `operationType: objectProps`, cursor return과 fallback `document-changed`
- dialog UI/DOM/CSS, `populateFromProps`, public npm/embed, Rust/WASM schema와 generated binding
- runtime UI framework와 runtime dependency 0

## Metrics

maintainer의 #1904 결산과 #2130 산식 교훈을 반영해 Max나 threshold 개수만 보지 않고 Total CC, Top 20 합,
CC>25 합·개수, CC>100 개수와 stable function diff를 함께 비교했습니다.

| 지표 | same-base pre | post | delta |
|------|--------------:|-----:|------:|
| Total CC | 12,369 | 12,093 | -276 |
| Top 20 합 | 2,660 | 2,359 | -301 |
| CC>25 개수 / 합 | 70 / 4,297 | 69 / 3,949 | -1 / -348 |
| CC>100 개수 | 7 | 6 | -1 |
| Max CC | 453 | 453 | 0 |
| target dialog/model Total CC | 647 | 371 | -276 |
| `handleOk` CC / LOC | 348 / 381 | 2 / 11 | -346 / -370 |

stable changed function은 `handleOk` 하나이며 대상 밖 stable changed function은 0입니다. 새 helper Max CC는
13이고 신규 CC>25 함수는 없습니다.

#2124 official snapshot 대비 현재 repository 누적치는 Total CC +288, Top 20 -222, CC>25 합 +17이지만,
이 값에는 이후 다른 frontend merge와 legacy `/web` 제거가 포함됩니다. 따라서 #2392 성과로 귀속하지 않고
동일 `af5902b6` base의 pre/post만 완료 판정에 사용했습니다.

## SOLID review 단위

frontend 전체 예비 총점은 사용하지 않고 apply pipeline diff만 평가했습니다.

- SRP: form capture, policy, target, mutation과 undo 변경 이유 분리
- DIP: pure model의 DOM/WASM/EventBus/CommandServices runtime 의존 0
- OCP/ISP: field·target 규칙을 제한된 model/union에서 검토하고 거대 context 전달 방지
- LSP: 명시된 대체 계약이 없어 임의 점수화하지 않음

공식 SOLID 점수는 reviewer가 같은 평가 단위와 근거에 합의할 때만 기록합니다.

## 검증

- pure patch fixture 20/20, target fixture 6/6 PASS
- Stage 3 focused model/undo/Through/mutation 38/38 PASS
- `npm --prefix rhwp-studio test`: 390/390 PASS
- `npm --prefix rhwp-studio run build`: PASS
- Chrome headless undo contracts: 6개 시나리오 PASS
- 그림 속성 실제 Ctrl+Z command failure: 0
- setter method·인자 순서: 5/5, dialog mutation count 5 유지
- `git diff --check`: PASS

별도 visual sweep은 DOM/CSS/rendering diff가 없어 수행하지 않았습니다. 실제 dialog apply/undo와 screenshot은
headless browser 계약에서 확인했습니다.

## 범위 밖과 잔여 위험

- `populateFromProps` CC 212 해체
- omitted/stale control의 current UX 정책 변경
- 공통 `WasmBridge`/history API, package/extension/embed 변경

위 항목은 이 PR에 섞지 않았습니다. 후속 구현은 #2022에서 다른 hotspot과 위험·효과를 다시 비교한 뒤
별도 승인으로 결정합니다.

## 문서

- `mydocs/tech/investigations/issue-2392/task_m100_2392_picture_props_apply_contract.md`
- `mydocs/working/task_m100_2392_stage1.md`부터 `stage4.md`
- `mydocs/report/task_m100_2392_report.md`
```

## 4. maintainer review 요청 코멘트 초안

```md
@edwardkim Phase B 첫 실행 단위 #2392의 구현과 local gate를 완료했습니다. 리뷰 부탁드립니다.

특히 다음 항목을 확인 부탁드립니다.

1. form capture, pure patch policy, 5-target routing과 undo orchestration의 책임 경계
2. diff-only/always-send, OLE 제한, header/footer 우선과 setter 인자 순서를 보존한 판단
3. actual WASM setter 5회를 dialog에 남기고 pure model의 runtime 의존과 package re-export를 0으로 둔 범위
4. Total CC, Top 20, CC>25/100과 stable function diff를 함께 사용해 복잡도 이동을 차단한 결산
5. #2124 official 누적 변화와 #2392 same-base 직접 delta를 분리한 판정
6. frontend 전체 SOLID 총점 없이 apply pipeline diff만 원칙별 review 단위로 둔 판단
7. `populateFromProps`와 omitted/stale control 정책을 이번 PR에 섞지 않은 후속 경계

local에서는 Studio 390 tests/build와 Chrome headless undo contracts 6개 시나리오를 통과했습니다.
```

## 5. #2392 진행 댓글 초안

```md
## Stage 4 완료 및 draft PR review 진입

#2392의 local 구현·검증을 완료하고 draft PR #PR_NUMBER를 생성했습니다.

| 완료 항목 | 결과 |
|-----------|------|
| apply 책임 분리 | form capture, pure patch, 5-target routing, mutation과 undo orchestration |
| behavior | 20 patch + 6 target fixture, setter 5회/인자 순서, snapshot/fallback 보존 |
| local gate | Studio 390/390, build, Chrome headless undo 6개 시나리오 PASS |
| same-base metrics | Total CC -276, Top 20 -301, CC>25 합 -348, `handleOk` 348 -> 2 |
| contract audit | UI/CSS/core/Rust/WASM/package/dependency diff 0 |

- PR: #PR_NUMBER
- head: `HEAD_SHA`
- 최종 보고서: `mydocs/report/task_m100_2392_report.md`

현재는 maintainer/collaborator review와 PR CI 대기 상태입니다. merge와 완료 근거 확인 전에는 이 이슈를
close하지 않습니다.
```

## 6. #2022 진행 댓글 초안

```md
## Phase B 첫 실행 단위 review 진입

#2022에서 승인한 첫 실행 단위 #2392가 draft PR #PR_NUMBER로 review에 진입했습니다.

- `picture-props-dialog` apply pipeline을 pure policy, 5-target routing과 undo orchestration으로 분리
- same-base Total CC -276, Top 20 -301, CC>25 합 -348
- Studio 390 tests/build와 Chrome headless undo 계약 PASS
- public/WASM/package/UI/runtime dependency 변화 0

#2124 official 대비 누적 변화는 이후 repository 변경과 분리했고, #2392 완료 여부는 PR review·CI와 merge
근거가 확보된 뒤 갱신합니다. 다음 Phase B 후보는 #2392 merge 전 자동 착수하지 않습니다.
```

## 7. PR CI 결과 코멘트 틀

```md
@edwardkim PR #PR_NUMBER CI 결과를 공유드립니다.

| 항목 | 결과 | 링크 |
|------|------|------|
| Frontend package gates | RESULT | URL |
| Build & Test aggregate | RESULT | URL |
| CodeQL | RESULT | URL |

head `HEAD_SHA`, base `BASE_SHA` 기준 mergeability와 unresolved review thread도 함께 재확인했습니다.
```

placeholder 상태로 게시하지 않는다.

## 8. merge와 close 경계

- draft 해제는 initial review feedback과 CI 상태를 보고 작업지시자가 결정한다.
- merge는 required CI와 review 승인 뒤 작업지시자 판단으로 수행한다.
- merge 후 #2392 완료 표, merge SHA와 CI 링크를 게시한다.
- #2392 close와 #2022 checkbox 갱신은 merge 근거 확인과 작업지시자 승인 후 수행한다.
- `populateFromProps` 또는 다른 Phase B 후보 이슈를 #2392 merge 전에 자동 생성하지 않는다.
