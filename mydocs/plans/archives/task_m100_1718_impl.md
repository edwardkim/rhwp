# Task #1718 구현계획서

## 개요
`advance_row_cut` / `advance_row_block_cut` 의 `visible_tail_before_spacer` grace 조건을
`.any()` → `.all()` 로 정정하여 대형 RowBreak 셀 연속 텍스트의 over-fill 을 제거한다.

## 단계

### Stage 1 — 원인 확정 재현 + 게이트 베이스라인
- 대표 파일 dump-pages = 40쪽, byeolpyo1=4?, byeolpyo4=27? 현재값 기록.
- 수정 전 `cargo test` green 확인(기존 테스트 baseline).

### Stage 2 — 소스 수정
- table_layout.rs 5232, 5357: `units[j + 1..].iter().any(...)` → `.all(...)`.
- 주석 갱신(의도: 뒤가 전부 spacer 인 진짜 꼬리줄만 grace).

### Stage 3 — 단위테스트 + 검증
- 신규 단위테스트 2개: (a) 연속 가시라인 remainder → grace 미적용(정상 break), (b) tail 뒤 전부 spacer → grace 유지.
- 대표 파일 dump-pages 40 → ~48 확인.
- byeolpyo1/byeolpyo4 무회귀 확인.
- `cargo test` 전체 green.

### Stage 4 — 회귀 전수 + 보고
- verify_pi_page_vs_hangul 샘플(수백 건) 재실행 — MATCH율·대형 케이스 개선/무회귀.
- 재현샘플 samples/task1718/ + README.
- 단계보고·최종보고 작성.

## 롤백 기준
byeolpyo1/byeolpyo4 회귀 또는 MATCH율 하락 시 `.all()` 대신 `h <= avail_height`(첫 경계라인만 grace, 오버필 1줄 상한) 대안으로 전환.
