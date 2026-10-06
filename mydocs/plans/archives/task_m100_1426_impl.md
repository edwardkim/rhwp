# Task M100 #1426 구현계획서 — rhwp-studio 쪽 테두리/배경 미리보기 버튼 토글 복구

- 이슈: #1426
- 브랜치: `local/task1426`
- 작성일: 2026-06-21
- 수행계획서: `mydocs/plans/task_m100_1426.md`

## 구현 개요

`PageBorderDialog`의 방향별 테두리 상태는 이미 `borderEdits: Record<Side, BorderLineProps>`에
보관되어 있으므로 새 상태 저장소를 만들지 않는다. `borderEdits[side].type !== 0`을 켜짐 상태로
판정하고, 방향 버튼 클릭과 선 속성 즉시 적용 경로만 이 상태 모델에 맞춰 정리한다.

문서 모델, WASM bridge, 렌더러, serializer는 변경하지 않는다.

---

## 1단계 — 테두리 토글 로직 구현

**대상**: `rhwp-studio/src/ui/page-border-dialog.ts`

- `applyToSides()`는 켜기 전용 함수로 유지하거나 의미가 드러나는 이름으로 제한한다.
- 개별 방향 버튼 클릭 경로를 토글로 변경한다.
  - 현재 `borderEdits[side].type !== 0`이면 `noneBorder()`로 해제한다.
  - 현재 꺼져 있으면 `currentBorder()` 값을 해당 방향에 적용한다.
- 전체 버튼 클릭 경로를 전체 토글로 변경한다.
  - 네 방향이 모두 켜져 있으면 네 방향을 `noneBorder()`로 해제한다.
  - 일부라도 꺼져 있으면 네 방향에 `currentBorder()`를 적용한다.
- 토글 후 `borderNoneCheck.checked`는 활성 방향 존재 여부에 맞춘다.
  - 활성 방향이 없으면 체크
  - 하나라도 활성 방향이 있으면 해제
- `테두리 사용 안 함` 체크 시 preview 표시만 숨기지 않고 네 방향 `borderEdits`도 모두 `noneBorder()`로 초기화한다.
- `updateBorderPreview()` 호출은 기존 흐름처럼 상태 변경 뒤 한 번만 수행한다.

**완료 기준**

- 위쪽/왼쪽/오른쪽/아래쪽 버튼이 재클릭으로 해제된다.
- 전체 버튼이 전체 적용과 전체 해제를 번갈아 수행한다.
- `테두리 사용 안 함` 체크 상태가 방향별 활성 상태와 모순되지 않는다.
- `테두리 사용 안 함` 체크 후 개별/전체 버튼 클릭이 한 번에 기대 상태로 적용된다.
- 1단계 완료보고서 `mydocs/working/task_m100_1426_stage1.md` 작성.

## 2단계 — 선 모양 바로 적용 회귀 수정 및 E2E 추가

**대상**

- `rhwp-studio/src/ui/page-border-dialog.ts`
- `rhwp-studio/e2e/page-border-toggle.test.mjs` 신규 또는 기존 E2E 확장

**구현**

- `lineTypeSelect`, `lineWidthSelect`, `lineColorInput` 변경 핸들러의 즉시 적용 대상을
  사방 전체가 아니라 현재 켜진 방향 목록으로 제한한다.
- 현재 켜진 방향이 없으면 선 속성 변경만 저장하고 preview에는 새 선을 추가하지 않는다.
- 필요 시 `applyCurrentBorderToActiveSides()` 같은 작은 private helper를 둔다.

**E2E 시나리오**

1. `page:page-border` 명령으로 쪽 테두리/배경 대화상자를 연다.
2. 위쪽 버튼을 2회 클릭하여 preview SVG `line` 개수가 원래 상태로 복귀하는지 확인한다.
3. 전체 버튼을 2회 클릭하여 preview SVG `line` 개수가 원래 상태로 복귀하는지 확인한다.
4. 위쪽만 켠 상태에서 선 종류 또는 굵기를 변경해도 `line` 개수가 1개로 유지되는지 확인한다.
5. `테두리 사용 안 함` 체크 후 전체 버튼 1회 클릭으로 `line` 개수가 4개가 되는지 확인한다.
6. `테두리 사용 안 함` 체크 후 위쪽 버튼 클릭으로 `line` 개수가 1개가 되는지 확인한다.

**완료 기준**

- 신규 E2E가 실패 재현 후 수정 코드에서 통과한다.
- 기존 `dialog-theme.test.mjs`의 쪽 테두리 테마 검증과 충돌하지 않는다.
- 2단계 완료보고서 `mydocs/working/task_m100_1426_stage2.md` 작성.

## 3단계 — 검증, 보고, 커밋 준비

**검증 명령**

- `cd rhwp-studio && node --check e2e/page-border-toggle.test.mjs`
- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && node e2e/page-border-toggle.test.mjs --mode=headless`
- 필요 시 `cd rhwp-studio && node e2e/dialog-theme.test.mjs --mode=headless`
- `git diff --check`

**보고**

- 최종 보고서 `mydocs/report/task_m100_1426_report.md` 작성.
- 오늘할일 `mydocs/orders/20260621.md`의 #1426 상태와 비고 갱신.
- 모든 변경 파일을 확인하고, 소스 변경과 단계별 보고서를 함께 커밋할 준비를 한다.

**완료 기준**

- 관련 빌드/E2E 검증 통과.
- 최종 보고서와 오늘할일 갱신 완료.
- 작업지시자 승인 후 커밋 진행.

---

## 변경 파일 예상

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/ui/page-border-dialog.ts` | 방향 버튼 토글, 전체 버튼 토글, none 체크 상태 초기화, 활성 방향 즉시 적용 |
| `rhwp-studio/e2e/page-border-toggle.test.mjs` | 쪽 테두리 preview 토글 및 none 체크 회귀 테스트 |
| `mydocs/working/task_m100_1426_stage1.md` | 1단계 완료보고서 |
| `mydocs/working/task_m100_1426_stage2.md` | 2단계 완료보고서 |
| `mydocs/report/task_m100_1426_report.md` | 최종 보고서 |
| `mydocs/orders/20260621.md` | 작업 상태 갱신 |

## 주의사항

- 소스 수정은 이 구현계획서 승인 후 시작한다.
- `borderNoneCheck`가 체크된 상태에서 preview가 early return하는 기존 구조를 유지한다.
- `type = 0`은 방향 해제 상태로 취급한다.
- 테스트는 SVG `line` 개수와 버튼 title 기준으로 동작을 검증하고, 색상/테마 표현에는 의존하지 않는다.
