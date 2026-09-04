# 단계 완료 보고 — Task M100 #2091 (R12): 표 컨트롤 블록 통이동

- 작성일: 2026-07-09 / goal 루프 3/4 (자체 검증)

## 수행 내용

표 컨트롤 배치 if-let 블록(663줄·분기 101)을 `layout_table_control_block` 통이동.
- 캐리 3(y_offset/tac_seg_applied/para_float_lane_info) = `TableControlOut` 반환,
  **함수 조기 return 1곳**은 `early_return: Option<(f64,bool)>` 프로토콜 — caller 에서
  `if let Some(ret) { return ret; }` 복원 (신규 패턴, 라운드 기록).
- `TableControlVars`(Copy 9, §6). ctx 재-destructure 로 파라미터 절약.
- `comp` 파라미터 오판 1건 — 지역 스코프 산물로 판명, 제거 (컴파일러 검출).
- 운영 사고 1건: 게이트를 트리 안정화 전에 착수 → 중단·재실행 (결과 오염 방지).
  브랜치 이동(devel 워킹트리 → local/task2091)도 stash 로 정리.

## 게이트 (전수 통과, 재실행분)

fmt ✓ / clippy 0 / `--tests` **2,945/0** / issue_1116 13/13 / OVR 5샘플 회귀 **0건**.

## 계측 (표적 공식 CC)

| 함수 | 시작 (r11) | 현재 |
|---|---|---|
| `layout_table_item` | **121** (전체 1위) | **48** |
| 신규 `layout_table_control_block` | — | **75** — §5 심사: 표 anchor/TAC/float 단일 국면, 후속 후보 등재 |
