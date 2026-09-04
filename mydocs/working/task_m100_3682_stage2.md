---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3682.md
last_verified: 2026-08-04
---

# Task #3682 Stage 2 보고 — P2 갭 수정 + P6 측정

## 수정 — 개체 속성이 어떤 경로로도 열리지 않던 원인

갭은 차트 전용이 아니라 **그림·도형 공통**이었다. 두 겹이었다:

1. **단축키 매핑 부재** — 커맨드 `format:object-properties`(shortcutLabel 'P',
   `canExecute: inPictureObjectSelection`)가 `shortcut-map.ts` 에 없었다. 정의만
   있고 트리거가 없는 상태.
2. **개체 선택 분기가 키를 삼킴** — `input-handler-keyboard.ts` 의 개체 선택 분기
   (730~)는 미처리 키에서 **선택을 해제한 뒤** 폴스루한다. 1을 고쳐도 일반 단축키
   경로에 닿을 때는 이미 `inPictureObjectSelection=false` 라 `canExecute` 가 거짓 —
   영영 실행되지 않는 구조였다.

수정: 매핑 추가(`p`/`ㅔ` → `format:object-properties`) + 선택 분기 폴스루 **직전**에
`dispatcher.isEnabled(cmdId)` 인 커맨드를 먼저 시도. 비활성이면 종전대로 폴스루하므로
본문 타이핑의 'p' 와 충돌하지 않는다(dispatcher 는 `canExecute` 실패 시 false 반환,
키 미소비).

## 실측 (dev 7700 + CDP)

| 단계 | 수정 전 | 수정 후 |
|---|---|---|
| P2 속성 다이얼로그 | 안 됨 | **열림** (`format:object-properties`) |
| P0·P1·P3·P4·P5 | 됨 | 됨 (불변) |

## P6 저장 라운드트립 — 통과

`묶은세로막대형.hwp` → HWPX → HWP 왕복에서 OLE 컨트롤 1개가 전 구간 보존
(dump `[OLE]` 카운트 1/1/1). 한컴 개방 검증은 원격 Windows 필요 — 미수행으로 남긴다.

## 회귀 검증

관련 e2e 3종 재실행: `issue-2069-ole-object-selection` PASS 27, `textbox-picture-1171`
PASS 7, `shape-inline` 통과 — **FAIL 0**.

주의: 성공 로그에 "검증 실패: undefined" 같은 문자열이 포함돼 grep 오판을 유발했다.
판정은 `^(PASS|FAIL)` 앵커로 세야 한다.
