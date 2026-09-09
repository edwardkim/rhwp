---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_3682.md
last_verified: 2026-08-04
---

# Task #3682 최종 보고 — 차트 Track A: 실측이 뒤집은 전제와 공통 갭 1건

- Issue: [#3682](https://github.com/edwardkim/rhwp/issues/3682) (M100) — #1431 이관
- 단계 기록: `mydocs/working/task_m100_3682_stage{1,2}.md`

## 결과

| 단계 | 이슈 기술 | 실측 | 수정 후 |
|---|---|---|---|
| P0 레이아웃 | 미구현 | **됨** | 됨 |
| P1 클릭 선택 | 불가 | **됨** | 됨 |
| **P2 속성** | 불가 | **안 됨** | **열림** ← 유일 갭 |
| P3 이동 | 불가 | **됨** | 됨 |
| P4 복사·삭제·undo | 불가 | **됨** | 됨 |
| P5 z-order | 미구현 | **됨** | 됨 |
| P6 저장 왕복 | — | OLE 보존 1/1/1 | 동일 |

**이슈 전제("Track A 미착수, 구현 흔적 0")는 반증됐다.** P0 는 7/8 task #2069 로,
studio 배선은 `'ole'` 참조 45건으로 이미 존재했다.

## 갭의 정체 — 차트 문제가 아니라 개체 공통 배선 결함

두 겹이었다:

1. 커맨드 `format:object-properties`(shortcutLabel 'P')가 `shortcut-map.ts` 에
   **매핑 없음** — 정의만 있고 트리거 부재.
2. 매핑을 넣어도 **개체 선택 분기가 미처리 키에서 선택을 먼저 해제**하고 폴스루하므로,
   일반 단축키 경로 도달 시 `canExecute(inPictureObjectSelection)` 가 거짓 —
   구조적으로 실행 불가.

수정: 매핑 추가 + 폴스루 **직전** `dispatcher.isEnabled(cmdId)` 커맨드 우선 시도.
비활성이면 종전 폴스루라 본문 'p' 타이핑과 충돌하지 않는다(dispatcher 는 canExecute
실패 시 키 미소비).

**파급**: 차트뿐 아니라 그림·도형도 개체 속성이 열리지 않던 상태였다.

## 검증

- 행동 실측 프로브 신설(`e2e/issue-3682-chart-object-probe.test.mjs`) — P0~P5 일괄.
- 회귀 e2e 3종: issue-2069(PASS 27) · textbox-picture-1171(PASS 7) · shape-inline —
  **FAIL 0**.
- P6: HWP→HWPX→HWP 왕복 OLE 보존. 한컴 개방 검증은 원격 Windows 필요 — 미수행.

## 방법 교훈 — 프로브가 가짜 갭을 3건 만들었다

1차 프로브는 P1·P3(추측한 API 이름 오류), P4 삭제(붙여넣기 단계 간섭)를 "안 됨"으로
오보고했다. 특히 "코어가 OLE 삭제를 거부한다"는 경로 추적은 **틀렸다** — 코어 직접
호출로 `{ok:true}`·ole 1→0 실증(OLE 는 `Control::Shape(ShapeObject::Ole)` 라 가드를
정상 통과).

원칙: **다단계 UI 프로브는 단계 간 상태 간섭이 기본값**이다. 각 단계는 선택을
재확립하고, 실패 항목은 하위 계층(코어 API) 직접 호출로 교차 검증한 뒤에만 갭으로
확정한다. 로그 판정은 `^(PASS|FAIL)` 앵커로 — 성공 메시지에 "실패" 문자열이 섞인다.

## 남은 것

- P6 한컴 개방 검증(원격 Windows).
- Track B(내용 편집, #3683)의 선행 조건이던 "선택 진입점"은 이미 충족 — 그 이슈의
  착수 전제도 재검토 대상.
