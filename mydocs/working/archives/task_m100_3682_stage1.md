---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3682.md
last_verified: 2026-08-04
---

# Task #3682 Stage 1 보고 — P0~P5 행동 실측

CDP e2e 프로브 `rhwp-studio/e2e/issue-3682-chart-object-probe.test.mjs` 신설,
`samples/chart/세로막대형/묶은세로막대형.hwp` 로 실측(dev 7700, 호스트 Chrome CDP).

## 현황표 — 이슈 전제("Track A 미착수")는 **틀렸다**

| 단계 | 동작 | 실측 | 비고 |
|---|---|---|---|
| P0 | ole 레이아웃 방출 | **됨** | `{x:113.4,y:132.3,w:430,h:250,secIdx:0,paraIdx:0,controlIdx:2}` |
| P1 | 클릭 선택 | **됨** | `type=ole ref={sec:0,ppi:0,ci:2}` |
| P2 | 속성 다이얼로그 | **미검증** | 프로브가 커맨드 레지스트리 전역 진입점을 못 찾음 — 코드에는 존재(`format.ts:476` PicturePropsDialog) |
| P3 | 드래그 이동 | **됨** | Δx=60 Δy=40 (지시대로 이동) |
| P4 | 복사·붙여넣기 | **됨** | ole 1→2 |
| P4 | **삭제** | **안 됨** | ole 2→2 |
| P4 | undo | (삭제 미발생이라 무의미) | 2→1 은 붙여넣기 취소 |
| P5 | z-order | **미검증** | `input-handler` 전역 함수는 없으나 브리지 `changeShapeZOrder` 존재 |

## 최종 현황표 (단계 격리 재측정 후)

| 단계 | 동작 | 실측 |
|---|---|---|
| P0 | ole 레이아웃 방출 | **됨** |
| P1 | 클릭 선택 | **됨** (`type=ole ref={0,0,2}`) |
| **P2** | **속성 다이얼로그** | **안 됨** ← 유일한 갭 |
| P3 | 드래그 이동 | **됨** (Δ60,40) |
| P4 | 복사·붙여넣기 | **됨** (1→2) |
| P4 | 삭제 | **됨** (격리 측정 1→0) |
| P4 | undo | **됨** (0→1) |
| P5 | z-order | **됨** (`changeShapeZOrder front` → zOrder=8) |

## 유일한 갭 — P2 속성 다이얼로그 (studio 배선)

커맨드 `format:object-properties`(`command/commands/format.ts:457`, shortcutLabel 'P',
`canExecute: inPictureObjectSelection`)는 **정의만 있고 어디서도 호출되지 않는다**
(grep: id 정의 1건 외 참조 0). 키보드 핸들러의 개체 선택 분기(input-handler-keyboard.ts
:730~)에 Escape·Enter·Delete 는 있으나 **P 키 분기가 없다.**

코어·다이얼로그는 준비됨 — `PicturePropsDialog.open(sec, ppi, ci, type)` 가 type 을
받으므로 'ole' 전달 경로만 열면 된다.

## 반증 이력 — 프로브가 만든 가짜 갭 3건

1차 프로브는 P1·P3(API 이름 오류), P4 삭제(붙여넣기 단계 간섭)를 "안 됨"으로 오보고했다.
- P4 는 코어 직접 호출 실측으로 반증: `deleteShapeControl(0,0,2)` → `{"ok":true}`, ole 1→0.
  (OLE 는 `Control::Shape(ShapeObject::Ole)` 이므로 코어 가드를 정상 통과한다 —
  "코어가 OLE 를 거부한다"는 내 1차 경로 추적은 **틀렸다**.)
- 교훈: 다단계 UI 프로브는 **단계 간 상태 간섭**이 기본값이다. 각 단계는 선택을
  재확립하고, 실패 항목은 하위 계층(코어 API) 직접 호출로 교차 검증한 뒤에만 갭으로
  확정한다.

## Stage 2 제안

- 실제 갭은 **P2 속성 다이얼로그 1건**. 이슈의 "선택·이동·리사이즈·복사·삭제 전부 불가"는
  실측으로 반증됐다(P6 저장 라운드트립만 미검증).
- 수정: 키보드 핸들러 개체 선택 분기에 P 키 → `format:object-properties` 상당 경로 배선.
  (커맨드 재사용이 이상적이나 핸들러에서 커맨드 레지스트리 접근 경로 확인 필요.)
- P6(저장 라운드트립)는 CLI 로 별도 측정.
