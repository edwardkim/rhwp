# Task M100 #4771 Stage 3 — renderer cache ownership

- `Table.dirty`를 제거하고 외곽 paragraph revision이 측정 invalidation을 소유하게 했다.
- 단일줄 overflow memo를 source `Paragraph`에서 renderer layout/measurement session cache로 옮겼다.
- `clear_table_cells_at_cursor`도 외곽 문단 revision과 page-tree invalidation 경계를 통과시켰다.
- HwpCtrl 표·단일 셀 resize도 같은 owner를 거쳐 측정과 page-tree geometry를 갱신한다.
- 셀 text reflow pagination provenance를 source `Table`에서 live Box-identity render state로 옮겼다.
- control/문단 이동은 identity를 유지하고, 표 나누기·붙이기·clipboard clone은 provenance를 명시 승계한다.
- 저장 text partition provenance는 renderer memo와 분리된 boolean 계약으로 남겼다.
