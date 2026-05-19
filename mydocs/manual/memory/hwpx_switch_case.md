---
name: HWPX switch/case와 줄간격 유형
description: HWPX의 switch/case 네임스페이스 분기가 문단 간격 유형(글자에따라 vs 고정값)과 관련됨
type: project
---

HWPX paraPr에서 `<switch>/<case>/<default>` 구조는 줄간격/문단간격 유형과 관련:
- `HwpUnitChar` case: "글자에 따라"(Percent) 모드의 값 — 글자 크기 기준 비율
- `default` case: "고정값"(Fixed) 모드의 값 — 절대 HWPUNIT 값

**Why:** 현재 파서가 default 케이스만 읽어서 고정값 기준의 큰 값이 항상 적용됨. 한컴은 HwpUnitChar 케이스를 우선 사용.

**How to apply:** HWPX 파서에서 switch/case 파싱 시 HwpUnitChar 네임스페이스 케이스를 우선 적용. 문서 전체 레이아웃 정확도에 직접 영향.

고정값(Fixed) 줄간격의 레이아웃 동작:
- LINE_SEG vpos가 절대 좌표로 동작 — 표/그림 높이와 무관하게 고정 간격 배치
- TAC 표는 문단 위에 겹쳐서(병행) 렌더링됨
- 표 높이만큼 y를 밀어내면 안 됨 → vpos 기반 절대 배치 필요
